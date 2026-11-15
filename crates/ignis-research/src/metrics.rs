//! Observability of episodes (task M5): metrics derived from the event log, never stored and
//! never estimated.
//!
//! - Tool latency: `tool.started` → `tool.finished` of the same call, from the host's `occurred_at`
//!   stamps; p50/p95 by nearest rank over the measured samples.
//! - Tokens: the `usage.reported` events the runtimes' own usage fields became (Codex thread
//!   totals, Claude Code per-turn increments), per turn, per question (a `message.user` and the
//!   turns that answered it), per episode and per workspace; cache hit = cached / input.
//! - Money: tokens × a versioned [`PriceList`] (`configs/pricing/llm_prices.toml`, built in, or
//!   `--price-list`). A model the list does not price is `unpriced`, never zero.
//!
//! Pure functions over events ([`measure`], [`aggregate`]) plus thin readers over the store
//! ([`measure_episode`], [`measure_workspace`]); `/api/v1`, `ignis metrics` and the structured log
//! lines all call these.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use ignis_core::contracts::{
    CostBasis, CostFigure, CostStatus, EpisodeId, EpisodeMetrics, ErrorCode, ErrorEnvelope,
    EventBody, EventRange, ExecutionScope, Finite, LatencyStats, QuestionMetrics, ResearchEpisode,
    ResearchEvent, RetryPolicy, Timestamp, TokenTotals, ToolMetrics, TurnMetrics, UsageAccounting,
    UsageReport, WorkspaceMetrics,
};
use ignis_core::services::ServiceResult;
use serde::Deserialize;

use crate::store::Replay;
use crate::ResearchStore;

/// The price list built into the binary (the reviewed file in `configs/pricing`).
const BUILTIN_PRICES: &str = include_str!("../../../configs/pricing/llm_prices.toml");

/// Events read per replay page.
const PAGE: usize = 1000;

/// Model key of usage whose model neither the runtime nor the harness pin named.
pub const UNREPORTED_MODEL: &str = "unreported";

/// Turn key of usage or tool calls the runtime did not attribute to a turn.
pub const UNATTRIBUTED_TURN: &str = "unattributed";

// ------------------------------------------------------------------------------ price list

/// Prices of one model, USD (or the list's currency) per million tokens.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelPrice {
    pub id: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Input that was neither read from nor written to the prompt cache.
    pub input_per_mtok: f64,
    #[serde(default)]
    pub cached_input_per_mtok: Option<f64>,
    #[serde(default)]
    pub cache_write_per_mtok: Option<f64>,
    pub output_per_mtok: f64,
}

/// A dated, versioned price list. Its identity (`<name>@<version>`) travels with every figure.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceList {
    pub name: String,
    pub version: String,
    pub currency: String,
    pub source: String,
    #[serde(default)]
    pub models: Vec<ModelPrice>,
}

impl PriceList {
    /// The list built into the binary.
    pub fn builtin() -> Self {
        Self::parse(BUILTIN_PRICES).expect("the built-in price list is valid")
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let list: PriceList = toml::from_str(text).map_err(|e| format!("price list: {e}"))?;
        list.validate()?;
        Ok(list)
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("price list {}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// `<name>@<version>`.
    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }

    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.version.trim().is_empty() {
            return Err("a price list has a name and a version".into());
        }
        if self.currency.len() != 3 || !self.currency.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(format!("{:?} is not an ISO 4217 code", self.currency));
        }
        let mut ids = BTreeSet::new();
        for m in &self.models {
            for id in std::iter::once(&m.id).chain(&m.aliases) {
                if id.trim().is_empty() || !ids.insert(id.as_str()) {
                    return Err(format!("model id {id:?} is empty or listed twice"));
                }
            }
            let prices = [Some(m.input_per_mtok), m.cached_input_per_mtok]
                .into_iter()
                .chain([m.cache_write_per_mtok, Some(m.output_per_mtok)]);
            if prices.flatten().any(|p| !p.is_finite() || p < 0.0) {
                return Err(format!("{}: prices are finite and not negative", m.id));
            }
        }
        Ok(())
    }

    fn find(&self, model: &str) -> Option<&ModelPrice> {
        self.models
            .iter()
            .find(|m| m.id == model || m.aliases.iter().any(|a| a == model))
    }

    /// The cost of `u` tokens of `model`, or why it cannot be priced.
    fn price(&self, model: &str, u: &Usage) -> Result<f64, String> {
        let Some(p) = self.find(model) else {
            return Err(format!(
                "model {model:?} is not in price list {}",
                self.id()
            ));
        };
        let cached = u.cached.unwrap_or(0);
        let written = u.written.unwrap_or(0);
        let class = |n: u64, price: Option<f64>, what: &str| match (n, price) {
            (0, _) => Ok(0.0),
            (n, Some(p)) => Ok(n as f64 * p),
            (_, None) => Err(format!("{}: no {what} price in {}", model, self.id())),
        };
        let fresh = u.input.saturating_sub(cached).saturating_sub(written);
        let total = fresh as f64 * p.input_per_mtok
            + class(cached, p.cached_input_per_mtok, "cached-input")?
            + class(written, p.cache_write_per_mtok, "cache-write")?
            + u.output as f64 * p.output_per_mtok;
        Ok(total / 1e6)
    }

    fn figure(&self, amount: f64) -> CostFigure {
        CostFigure {
            amount: Finite::new(amount).unwrap_or_else(|_| Finite::new(0.0).expect("finite")),
            currency: self.currency.clone(),
            basis: CostBasis::PriceList,
            price_list: Some(self.id()),
        }
    }
}

static PRICES: std::sync::OnceLock<std::sync::Arc<PriceList>> = std::sync::OnceLock::new();

/// Make `list` the process's price list (at startup, before the first turn). Installing the same
/// list again is a no-op; a different one, or any list after [`prices`] was first read, is
/// refused. Without it, [`prices`] is the built-in list.
pub fn install_prices(list: PriceList) -> Result<(), String> {
    let installed = PRICES.get_or_init(|| std::sync::Arc::new(list.clone()));
    if **installed == list {
        Ok(())
    } else {
        Err(format!(
            "price list {} refused: the process already uses {}",
            list.id(),
            installed.id()
        ))
    }
}

/// The process's price list.
pub fn prices() -> std::sync::Arc<PriceList> {
    PRICES
        .get_or_init(|| std::sync::Arc::new(PriceList::builtin()))
        .clone()
}

// ------------------------------------------------------------------------------ token sums

/// Summed usage of one model. Optional counts stay unknown until a report gives one.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Usage {
    reports: u64,
    input: u64,
    cached: Option<u64>,
    written: Option<u64>,
    output: u64,
    reasoning: Option<u64>,
}

fn add_opt(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (None, None) => None,
        (a, b) => Some(a.unwrap_or(0).saturating_add(b.unwrap_or(0))),
    }
}

impl Usage {
    fn of(r: &UsageReport) -> Self {
        Usage {
            reports: 1,
            input: r.input_tokens,
            cached: r.cached_input_tokens,
            written: r.cache_write_tokens,
            output: r.output_tokens,
            reasoning: r.reasoning_tokens,
        }
    }

    fn add(&mut self, o: &Usage) {
        self.reports += o.reports;
        self.input = self.input.saturating_add(o.input);
        self.cached = add_opt(self.cached, o.cached);
        self.written = add_opt(self.written, o.written);
        self.output = self.output.saturating_add(o.output);
        self.reasoning = add_opt(self.reasoning, o.reasoning);
    }

    /// `self - before` for running totals, or `None` when any count went down (a new thread).
    fn since(&self, before: &Usage) -> Option<Usage> {
        let opt = |a: Option<u64>, b: Option<u64>| match (a, b) {
            (Some(a), b) => a.checked_sub(b.unwrap_or(0)).map(Some),
            (None, _) => Some(None),
        };
        Some(Usage {
            reports: 1,
            input: self.input.checked_sub(before.input)?,
            cached: opt(self.cached, before.cached)?,
            written: opt(self.written, before.written)?,
            output: self.output.checked_sub(before.output)?,
            reasoning: opt(self.reasoning, before.reasoning)?,
        })
    }
}

/// Usage per model key.
type ByModel = BTreeMap<String, Usage>;

fn merge(into: &mut ByModel, from: &ByModel) {
    for (m, u) in from {
        into.entry(m.clone()).or_default().add(u);
    }
}

fn totals(by: &ByModel) -> TokenTotals {
    let mut u = Usage::default();
    by.values().for_each(|x| u.add(x));
    let cache_hit_pct = match u.cached {
        Some(c) if u.input > 0 => Finite::new(100.0 * c as f64 / u.input as f64).ok(),
        _ => None,
    };
    TokenTotals {
        reports: u.reports,
        input_tokens: u.input,
        cached_input_tokens: u.cached,
        cache_write_tokens: u.written,
        output_tokens: u.output,
        reasoning_tokens: u.reasoning,
        cache_hit_pct,
    }
}

fn cost(by: &ByModel, prices: &PriceList) -> CostStatus {
    if by.values().all(|u| u.reports == 0) {
        return CostStatus::NoUsage;
    }
    let (mut amount, mut models, mut reasons) = (0.0, vec![], vec![]);
    for (m, u) in by {
        match prices.price(m, u) {
            Ok(a) => amount += a,
            Err(why) => {
                models.push(m.clone());
                reasons.push(why);
            }
        }
    }
    if models.is_empty() {
        CostStatus::Priced {
            cost: prices.figure(amount),
        }
    } else {
        CostStatus::Unpriced { models, reasons }
    }
}

// ------------------------------------------------------------------------------ latency

/// Nearest-rank percentile of sorted samples (`q` in (0, 1]).
fn nearest_rank(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    let rank = ((q * n as f64).ceil() as usize).clamp(1, n);
    sorted[rank - 1]
}

fn latency(samples: &[f64]) -> Option<LatencyStats> {
    if samples.is_empty() {
        return None;
    }
    let mut v = samples.to_vec();
    v.sort_by(f64::total_cmp);
    let f = |x: f64| Finite::new(x).ok();
    Some(LatencyStats {
        samples: v.len() as u64,
        p50_ms: f(nearest_rank(&v, 0.50))?,
        p95_ms: f(nearest_rank(&v, 0.95))?,
        max_ms: f(*v.last()?)?,
    })
}

fn millis(t: &Timestamp) -> Option<f64> {
    chrono::DateTime::parse_from_rfc3339(t.as_str())
        .ok()
        .map(|d| d.timestamp_micros() as f64 / 1000.0)
}

/// Raw tool samples, kept so aggregates use every call rather than averaged percentiles.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToolSamples {
    pub calls: u64,
    pub unfinished: u64,
    /// Measured latencies in milliseconds.
    pub latencies_ms: Vec<f64>,
}

fn tool_metrics(tools: &BTreeMap<String, ToolSamples>) -> Vec<ToolMetrics> {
    tools
        .iter()
        .map(|(tool, s)| ToolMetrics {
            tool: tool.clone(),
            calls: s.calls,
            unfinished: s.unfinished,
            latency: latency(&s.latencies_ms),
        })
        .collect()
}

// ------------------------------------------------------------------------------ one episode

/// Metrics of one episode plus what an aggregate needs (raw samples, usage by model).
#[derive(Debug, Clone, PartialEq)]
pub struct EpisodeMeasure {
    pub metrics: EpisodeMetrics,
    pub tools: BTreeMap<String, ToolSamples>,
    usage: ByModel,
}

#[derive(Default)]
struct TurnAcc {
    first_seq: u64,
    last_seq: u64,
    question_seq: Option<u64>,
    tool_calls: u64,
    usage: ByModel,
}

struct QuestionAcc {
    message_seq: u64,
    started_ms: Option<f64>,
    last_ms: Option<f64>,
    tool_calls: u64,
}

/// Metrics of one episode from its full log (from seq 1, in order).
pub fn measure(
    episode: &ResearchEpisode,
    events: &[ResearchEvent],
    prices: &PriceList,
) -> EpisodeMeasure {
    let pinned_model = episode.pins.harness.as_ref().map(|h| h.model.clone());
    let mut turns: Vec<(String, TurnAcc)> = vec![];
    let mut turn_ix: HashMap<String, usize> = HashMap::new();
    let mut questions: Vec<QuestionAcc> = vec![];
    let mut tools: BTreeMap<String, ToolSamples> = BTreeMap::new();
    // call id → (tool, started at ms)
    let mut open: HashMap<String, (String, Option<f64>)> = HashMap::new();
    // Running thread totals per model key (`thread_total` reports).
    let mut thread_totals: HashMap<String, Usage> = HashMap::new();

    for e in events {
        let at = millis(&e.occurred_at);
        if let EventBody::UserMessage { .. } = e.body {
            questions.push(QuestionAcc {
                message_seq: e.seq,
                started_ms: at,
                last_ms: at,
                tool_calls: 0,
            });
        } else if let Some(q) = questions.last_mut() {
            q.last_ms = at.or(q.last_ms);
        }
        let question_seq = questions.last().map(|q| q.message_seq);
        let turn_key = e.causation.runtime_turn_id.clone();
        let turn = |key: &str,
                    turns: &mut Vec<(String, TurnAcc)>,
                    ix: &mut HashMap<String, usize>|
         -> usize {
            *ix.entry(key.to_string()).or_insert_with(|| {
                turns.push((
                    key.to_string(),
                    TurnAcc {
                        first_seq: e.seq,
                        last_seq: e.seq,
                        question_seq,
                        ..Default::default()
                    },
                ));
                turns.len() - 1
            })
        };
        if let Some(key) = &turn_key {
            let i = turn(key, &mut turns, &mut turn_ix);
            turns[i].1.last_seq = e.seq;
        }
        match &e.body {
            EventBody::ToolStarted { call_id, tool, .. } => {
                tools.entry(tool.clone()).or_default().calls += 1;
                open.insert(call_id.to_string(), (tool.clone(), at));
                let key = turn_key.as_deref().unwrap_or(UNATTRIBUTED_TURN);
                let i = turn(key, &mut turns, &mut turn_ix);
                turns[i].1.tool_calls += 1;
                if let Some(q) = questions.last_mut() {
                    q.tool_calls += 1;
                }
            }
            EventBody::ToolFinished { result } => {
                if let Some((tool, started)) = open.remove(result.call_id.as_str()) {
                    if let (Some(s), Some(f)) = (started, at) {
                        tools
                            .entry(tool)
                            .or_default()
                            .latencies_ms
                            .push((f - s).max(0.0));
                    }
                }
            }
            EventBody::UsageReported { usage } => {
                let model = usage
                    .model
                    .clone()
                    .or_else(|| pinned_model.clone())
                    .unwrap_or_else(|| UNREPORTED_MODEL.to_string());
                let now = Usage::of(usage);
                let delta = match usage.accounting {
                    UsageAccounting::Increment => now,
                    UsageAccounting::ThreadTotal => {
                        let before = thread_totals.insert(model.clone(), now);
                        before.and_then(|b| now.since(&b)).unwrap_or(now)
                    }
                };
                let key = turn_key.as_deref().unwrap_or(UNATTRIBUTED_TURN);
                let i = turn(key, &mut turns, &mut turn_ix);
                turns[i].1.usage.entry(model).or_default().add(&delta);
            }
            _ => {}
        }
    }
    for (tool, _) in open.into_values() {
        tools.entry(tool).or_default().unfinished += 1;
    }

    let turn_metrics: Vec<TurnMetrics> = turns
        .iter()
        .map(|(id, t)| TurnMetrics {
            runtime_turn_id: id.clone(),
            question_seq: t.question_seq,
            events: EventRange {
                first_seq: t.first_seq,
                last_seq: t.last_seq,
            },
            tool_calls: t.tool_calls,
            tokens: totals(&t.usage),
            cost: cost(&t.usage, prices),
        })
        .collect();
    let mut usage = ByModel::new();
    turns.iter().for_each(|(_, t)| merge(&mut usage, &t.usage));
    let question_metrics = questions
        .iter()
        .map(|q| {
            // A turn belongs to the question it started under.
            let mine: Vec<&TurnAcc> = turns
                .iter()
                .map(|(_, t)| t)
                .filter(|t| t.question_seq == Some(q.message_seq))
                .collect();
            let mut by = ByModel::new();
            mine.iter().for_each(|t| merge(&mut by, &t.usage));
            let wall = match (q.started_ms, q.last_ms) {
                (Some(a), Some(b)) => ((b - a) / 1000.0).max(0.0),
                _ => 0.0,
            };
            QuestionMetrics {
                message_seq: q.message_seq,
                turns: mine.len() as u32,
                tool_calls: q.tool_calls,
                tokens: totals(&by),
                cost: cost(&by, prices),
                wall_time_s: Finite::new(wall).unwrap_or_else(|_| Finite::new(0.0).unwrap()),
            }
        })
        .collect();
    EpisodeMeasure {
        metrics: EpisodeMetrics {
            episode_id: episode.episode_id.clone(),
            last_seq: events.last().map_or(0, |e| e.seq),
            harness: episode.pins.harness.clone(),
            price_list: prices.id(),
            tools: tool_metrics(&tools),
            turns: turn_metrics,
            questions: question_metrics,
            tokens: totals(&usage),
            cost: cost(&usage, prices),
        },
        tools,
        usage,
    }
}

impl EpisodeMeasure {
    /// Usage per model as a trace reports it (`TraceBundle.usage`), model ids sorted. Cache
    /// writes have no field there; they stay in the input count.
    pub fn token_usage(&self) -> Vec<ignis_core::contracts::TokenUsage> {
        self.usage
            .iter()
            .filter(|(_, u)| u.reports > 0)
            .map(|(m, u)| ignis_core::contracts::TokenUsage {
                model: m.clone(),
                input_tokens: u.input,
                cached_input_tokens: u.cached,
                output_tokens: u.output,
                reasoning_tokens: u.reasoning,
            })
            .collect()
    }
}

/// Aggregate over episodes: latencies over every call, token sums, mean cost per question.
pub fn aggregate(measures: &[EpisodeMeasure], prices: &PriceList) -> WorkspaceMetrics {
    let mut tools: BTreeMap<String, ToolSamples> = BTreeMap::new();
    let mut usage = ByModel::new();
    let mut questions = 0u64;
    for m in measures {
        for (t, s) in &m.tools {
            let a = tools.entry(t.clone()).or_default();
            a.calls += s.calls;
            a.unfinished += s.unfinished;
            a.latencies_ms.extend(&s.latencies_ms);
        }
        merge(&mut usage, &m.usage);
        questions += m.metrics.questions.len() as u64;
    }
    let total = cost(&usage, prices);
    let per_question = match &total {
        CostStatus::Priced { cost } if questions > 0 => CostStatus::Priced {
            cost: prices.figure(cost.amount.get() / questions as f64),
        },
        CostStatus::Priced { .. } => CostStatus::NoUsage,
        other => other.clone(),
    };
    WorkspaceMetrics {
        episodes: measures.len() as u64,
        questions,
        price_list: prices.id(),
        tools: tool_metrics(&tools),
        tokens: totals(&usage),
        cost: total,
        cost_per_question: per_question,
    }
}

// ------------------------------------------------------------------------------ store readers

/// The whole log of an episode, from seq 1, read page by page.
pub fn read_log(
    store: &ResearchStore,
    scope: &ExecutionScope,
    id: &EpisodeId,
) -> ServiceResult<Vec<ResearchEvent>> {
    let mut events = vec![];
    loop {
        let after = events.last().map_or(0, |e: &ResearchEvent| e.seq);
        match store.replay(scope, id, after, PAGE)? {
            Replay::Events {
                events: page,
                last_seq,
            } => {
                let done = page.is_empty() || page.last().is_some_and(|e| e.seq >= last_seq);
                events.extend(page);
                if done {
                    return Ok(events);
                }
            }
            Replay::Gone { .. } => {
                return Err(ErrorEnvelope::new(
                    ErrorCode::Conflict,
                    format!("the log of {id} no longer starts at seq 1; metrics need all of it"),
                    RetryPolicy::Never,
                ))
            }
        }
    }
}

pub fn measure_episode(
    store: &ResearchStore,
    scope: &ExecutionScope,
    id: &EpisodeId,
    prices: &PriceList,
) -> ServiceResult<EpisodeMeasure> {
    let episode = store.episode(scope, id)?;
    let events = read_log(store, scope, id)?;
    Ok(measure(&episode, &events, prices))
}

/// Every episode of the scope's workspace.
pub fn measure_workspace(
    store: &ResearchStore,
    scope: &ExecutionScope,
    prices: &PriceList,
) -> ServiceResult<WorkspaceMetrics> {
    let episodes = store.episodes(scope, i64::MAX as usize)?;
    let measures = episodes
        .iter()
        .map(|e| measure_episode(store, scope, &e.episode_id, prices))
        .collect::<ServiceResult<Vec<_>>>()?;
    Ok(aggregate(&measures, prices))
}

/// One structured log line per finished turn (target `ignis::metrics`): its tokens, cache hit,
/// cost status and tool latencies. Called by the host when a turn's status change is recorded.
pub fn log_turn(m: &EpisodeMetrics, runtime_turn_id: &str) {
    let Some(t) = m
        .turns
        .iter()
        .find(|t| t.runtime_turn_id == runtime_turn_id)
    else {
        return;
    };
    let cost = match &t.cost {
        CostStatus::Priced { cost } => format!("{:.6} {}", cost.amount.get(), cost.currency),
        CostStatus::Unpriced { models, .. } => format!("unpriced ({})", models.join(", ")),
        CostStatus::NoUsage => "no_usage".into(),
    };
    tracing::info!(
        target: "ignis::metrics",
        episode = %m.episode_id,
        turn = runtime_turn_id,
        tool_calls = t.tool_calls,
        input_tokens = t.tokens.input_tokens,
        cached_input_tokens = t.tokens.cached_input_tokens,
        output_tokens = t.tokens.output_tokens,
        cache_hit_pct = t.tokens.cache_hit_pct.map(|p| p.get()),
        cost = %cost,
        price_list = %m.price_list,
        "turn metrics"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles_are_measured_samples() {
        let v: Vec<f64> = (1..=20).map(|i| i as f64 * 10.0).collect();
        assert_eq!(nearest_rank(&v, 0.5), 100.0);
        assert_eq!(nearest_rank(&v, 0.95), 190.0);
        assert_eq!(nearest_rank(&[7.0], 0.95), 7.0);
    }

    #[test]
    fn the_builtin_price_list_parses_and_unknown_models_are_unpriced() {
        let p = PriceList::builtin();
        let mut by = ByModel::new();
        by.insert(
            "claude-opus-5-5".into(),
            Usage {
                reports: 1,
                input: 1_000_000,
                cached: Some(500_000),
                written: Some(100_000),
                output: 100_000,
                reasoning: None,
            },
        );
        let CostStatus::Priced { cost: c } = cost(&by, &p) else {
            panic!()
        };
        // 400k fresh × 4 + 500k × 0.20 + 100k × 5 + 100k × 20, per million.
        assert!((c.amount.get() - (1.6 + 0.1 + 0.5 + 2.0)).abs() < 1e-9);
        by.insert(
            "gpt-unknown".into(),
            Usage {
                reports: 1,
                ..Default::default()
            },
        );
        assert!(
            matches!(cost(&by, &p), CostStatus::Unpriced { models, .. } if models == ["gpt-unknown"])
        );
        assert_eq!(cost(&ByModel::new(), &p), CostStatus::NoUsage);
    }
}
