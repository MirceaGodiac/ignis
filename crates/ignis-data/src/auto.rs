//! Automatic adjudication of the review queue with the headless `claude -p` CLI.
//!
//! Each item gets a self-contained dossier ([`crate::dossier`]) and N independent passes (separate
//! processes, no tools, no shared context, an empty working directory). Answers must cite dossier
//! lines and pass [`validate_answer`]; a failing pass is retried once with the error message. All
//! passes valid and in agreement -> `resolved` (or `unknown` when they agree nothing can be decided);
//! anything else -> `unresolved_conflict` with every answer kept. There is no tie-break.
//!
//! Records are labelled `agent = claude-code:<model>:<passes>` and become `Method::Semantic`.
//! Output is one JSONL per shard, `data/review/<family>/drafts/auto-<i>of<n>.jsonl`, appended as
//! each item finishes, so runs resume and several people can run different shards.

use std::collections::{BTreeMap, HashSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ignis_core::resources::{load_profile, Job};
use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::common::read_jsonl_values;
use crate::dossier::{self, Dossier, MAX_FETCH_BYTES};
use crate::paths;

/// Two passes' times count as the same reading within this margin (seconds, about 8 video frames).
pub const TIME_TOLERANCE_S: f64 = 0.25;
const CALL_TIMEOUT: Duration = Duration::from_secs(900);
const RATE_LIMIT_RETRIES: u32 = 4;
const MAX_CONSECUTIVE_FAILURES: usize = 3;

#[derive(Debug, Clone)]
pub struct AutoConfig {
    pub family: String,
    /// 1-based shard index and shard count.
    pub shard: (usize, usize),
    pub model: String,
    pub passes: usize,
    pub concurrency: usize,
    pub limit: Option<usize>,
    pub only: Option<Regex>,
    pub execute: bool,
    pub dump_dossiers: Option<PathBuf>,
}

/// Stable shard (1..=n) of an item: depends only on its id and the shard count.
pub fn shard_of(item_id: &str, n: usize) -> usize {
    let h = Sha256::digest(item_id.as_bytes());
    let x = u64::from_be_bytes(h[..8].try_into().unwrap());
    (x % n as u64) as usize + 1
}

pub fn parse_shard(s: &str) -> anyhow::Result<(usize, usize)> {
    let (a, b) = s
        .split_once('/')
        .ok_or_else(|| anyhow::anyhow!("--shard must be i/n, got {s:?}"))?;
    let (i, n): (usize, usize) = (a.trim().parse()?, b.trim().parse()?);
    anyhow::ensure!(
        n >= 1 && (1..=n).contains(&i),
        "--shard {s}: need 1 <= i <= n"
    );
    Ok((i, n))
}

pub fn shard_path(family: &str, (i, n): (usize, usize)) -> PathBuf {
    paths::review(family)
        .join("drafts")
        .join(format!("auto-{i}of{n}.jsonl"))
}

fn agent_label(cfg: &AutoConfig) -> String {
    format!("claude-code:{}:{}", cfg.model, cfg.passes)
}

const SYSTEM: &str = "You adjudicate one question about NASA microgravity combustion experiments from the evidence \
dossier in the user message. Decide only from the dossier: no outside knowledge, no assumptions about what usually \
happens, no guessing. If the dossier does not decide the question, answer with the candidate that says so. Every \
decisive answer cites dossier lines. Reply with a single JSON object and nothing else: no prose, no code fences.";

fn schema(field: &str) -> String {
    let base = r#"{"value": "<one of CANDIDATES, exactly>", "rationale": "<one sentence: what the evidence shows and why it decides>", "evidence": [{"id": "<line id such as L0003>", "quote": "<text copied verbatim from that line>"}]"#;
    if field == "flex_endpoint" {
        format!(
            r#"{base}, "censoring": "event|interval|right|unknown", "last_confirmed_active_s": <number or null>, "first_terminal_s": <number or null>, "event_before_support_exit": "yes|no|unknown", "time_origin_note": "<which time base the times are in and how you know>"}}"#
        )
    } else {
        format!("{base}}}")
    }
}

fn prompt(d: &Dossier, retry: Option<(&str, &[String])>) -> String {
    let mut p = format!(
        "Answer with one JSON object of exactly this shape:\n{}\nQuotes must be short exact substrings of the cited line. Cite only line ids that exist below.\n\n{}",
        schema(&d.field),
        d.render()
    );
    if let Some((prev, errors)) = retry {
        p.push_str(&format!(
            "\n\nYour previous answer was rejected by the validator.\nPrevious answer: {prev}\nErrors:\n- {}\nAnswer again following the shape above.",
            errors.join("\n- ")
        ));
    }
    p
}

fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn opt_num(a: &Value, key: &str, errors: &mut Vec<String>) -> Option<f64> {
    match a.get(key) {
        None | Some(Value::Null) => None,
        Some(Value::Number(n)) if n.as_f64().is_some_and(|x| x.is_finite()) => n.as_f64(),
        Some(v) => {
            errors.push(format!("{key} must be a finite number or null, got {v}"));
            None
        }
    }
}

fn no_decision(a: &Value, field: &str) -> bool {
    let v = a["value"].as_str().unwrap_or("");
    if field == "flex_endpoint" {
        v == "unknown"
            && a["censoring"].as_str() == Some("unknown")
            && a["last_confirmed_active_s"].is_null()
            && a["first_terminal_s"].is_null()
    } else {
        matches!(v, "unknown" | "unclear" | "mixed_or_unclear")
    }
}

/// The automatic validator. An empty result means the answer is acceptable.
pub fn validate_answer(d: &Dossier, a: &Value) -> Vec<String> {
    let mut e = vec![];
    if !a.is_object() {
        return vec!["the answer is not a JSON object".to_string()];
    }
    match a["value"].as_str() {
        None => e.push("value is missing".to_string()),
        Some(v) if !d.candidates.iter().any(|c| c == v) => {
            e.push(format!("value {v:?} is not one of the candidates"))
        }
        _ => {}
    }
    let cites = a["evidence"].as_array().cloned().unwrap_or_default();
    if cites.is_empty() && !no_decision(a, &d.field) {
        e.push("a decisive answer must cite at least one dossier line".to_string());
    }
    for c in &cites {
        let id = c["id"].as_str().unwrap_or("");
        let Some(line) = d.line(id) else {
            e.push(format!(
                "cited locator {id:?} does not exist in this dossier"
            ));
            continue;
        };
        let quote = norm(c["quote"].as_str().unwrap_or(""));
        if quote.is_empty() {
            e.push(format!("citation {id} has an empty quote"));
        } else if !norm(&line.text).contains(&quote) {
            e.push(format!("quote {quote:?} is not verbatim in line {id}"));
        }
    }
    if d.field == "flex_endpoint" {
        validate_flex(d, a, &mut e);
    }
    e
}

fn validate_flex(d: &Dossier, a: &Value, e: &mut Vec<String>) {
    let cause = a["value"].as_str().unwrap_or("");
    let censoring = a["censoring"].as_str().unwrap_or("");
    if !["event", "interval", "right", "unknown"].contains(&censoring) {
        e.push(format!(
            "censoring {censoring:?} is not event|interval|right|unknown"
        ));
    }
    if !["yes", "no", "unknown"].contains(&a["event_before_support_exit"].as_str().unwrap_or("")) {
        e.push("event_before_support_exit must be yes|no|unknown".to_string());
    }
    let last = opt_num(a, "last_confirmed_active_s", e);
    let first = opt_num(a, "first_terminal_s", e);
    let has_note = a["time_origin_note"]
        .as_str()
        .is_some_and(|s| !s.trim().is_empty());
    if (last.is_some() || first.is_some()) && !has_note {
        e.push("time_origin_note is required when a time is given".to_string());
    }
    for (name, t) in [
        ("last_confirmed_active_s", last),
        ("first_terminal_s", first),
    ] {
        if let Some(t) = t {
            match d.observed_range {
                Some((lo, hi)) if t < lo - 1e-6 || t > hi + 1e-6 => e.push(format!(
                    "{name}={t} is outside the observed range [{lo}, {hi}]"
                )),
                None => e.push(format!(
                    "{name} is given but this run has no observed time range"
                )),
                _ => {}
            }
        }
    }
    if let (Some(l), Some(f)) = (last, first) {
        if l > f {
            e.push(format!(
                "last_confirmed_active_s={l} is after first_terminal_s={f}"
            ));
        }
    }
    match censoring {
        "event" => {
            if cause == "unknown" {
                e.push("an exact event needs a known cause".to_string());
            }
            if last.is_none() || first.is_none() || last != first {
                e.push(
                    "censoring=event needs last_confirmed_active_s equal to first_terminal_s"
                        .to_string(),
                );
            }
        }
        "interval" => {
            if last.is_none() || first.is_none() || last == first {
                e.push(
                    "censoring=interval needs last_confirmed_active_s < first_terminal_s"
                        .to_string(),
                );
            }
        }
        "right" => {
            if cause != "unknown" {
                e.push("censoring=right means no terminal event was seen, so the cause must be unknown".to_string());
            }
            if last.is_none() || first.is_some() {
                e.push(
                    "censoring=right needs last_confirmed_active_s and a null first_terminal_s"
                        .to_string(),
                );
            }
        }
        "unknown" => {
            if first.is_some() {
                e.push("censoring=unknown cannot state first_terminal_s".to_string());
            }
        }
        _ => {}
    }
}

/// Do the valid answers of all passes say the same thing?
fn answers_agree(field: &str, a: &Value, b: &Value) -> bool {
    if a["value"] != b["value"] {
        return false;
    }
    if field != "flex_endpoint" {
        return true;
    }
    let close = |k: &str| match (a[k].as_f64(), b[k].as_f64()) {
        (None, None) => true,
        (Some(x), Some(y)) => (x - y).abs() <= TIME_TOLERANCE_S,
        _ => false,
    };
    a["censoring"] == b["censoring"]
        && a["event_before_support_exit"] == b["event_before_support_exit"]
        && close("last_confirmed_active_s")
        && close("first_terminal_s")
}

/// Merged reading: the widest bounds of the passes; an exact time they differ on becomes an interval.
fn merge_flex(answers: &[&Value]) -> Value {
    let mut m = answers[0].clone();
    let reduce =
        |k: &str, f: fn(f64, f64) -> f64| answers.iter().filter_map(|a| a[k].as_f64()).reduce(f);
    let last = reduce("last_confirmed_active_s", f64::min);
    let first = reduce("first_terminal_s", f64::max);
    m["last_confirmed_active_s"] = json!(last);
    m["first_terminal_s"] = json!(first);
    if m["censoring"] == "event" && last != first {
        m["censoring"] = json!("interval");
    }
    m
}

enum Stop {
    RateLimit(String),
    Auth(String),
    Failure(String),
}

#[derive(Default)]
struct Totals {
    calls: u64,
    turns: u64,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_input_tokens: u64,
    cache_creation_input_tokens: u64,
    duration_ms: u64,
}

struct Shared {
    stop: AtomicBool,
    reason: Mutex<Option<(String, String)>>,
    totals: Mutex<Totals>,
    consecutive_failures: AtomicUsize,
    backoff_ms: u64,
}

fn claude_bin() -> String {
    std::env::var("IGNIS_CLAUDE_BIN").unwrap_or_else(|_| "claude".to_string())
}

fn work_dir() -> anyhow::Result<PathBuf> {
    let d = std::env::temp_dir().join(format!("ignis-review-auto-{}", std::process::id()));
    std::fs::create_dir_all(&d)?;
    Ok(d)
}

struct CallOutput {
    text: String,
    usage: Value,
}

/// One process, one answer. The child sees no tools, no settings and none of the variables of an
/// enclosing Claude session, and no API key, so it runs on the logged-in subscription only.
fn spawn_once(model: &str, system: &str, user: &str, cwd: &Path) -> Result<CallOutput, Stop> {
    let mut cmd = Command::new(claude_bin());
    cmd.args([
        "-p",
        "--output-format",
        "json",
        "--model",
        model,
        "--no-session-persistence",
        "--tools",
        "",
        "--strict-mcp-config",
        "--setting-sources",
        "",
        "--disable-slash-commands",
        "--system-prompt",
        system,
    ])
    .current_dir(cwd)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    for (k, _) in std::env::vars_os() {
        let up = k.to_string_lossy().to_uppercase();
        if up.starts_with("CLAUDE") || up.starts_with("ANTHROPIC") {
            cmd.env_remove(k);
        }
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| Stop::Failure(format!("cannot start {}: {e}", claude_bin())))?;
    let mut stdin = child.stdin.take().unwrap();
    let input = user.as_bytes().to_vec();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&input);
    });
    let read_all = |mut r: Box<dyn Read + Send>| {
        std::thread::spawn(move || {
            let mut b = vec![];
            let _ = r.read_to_end(&mut b);
            b
        })
    };
    let out = read_all(Box::new(child.stdout.take().unwrap()));
    let err = read_all(Box::new(child.stderr.take().unwrap()));
    let started = Instant::now();
    let status = loop {
        match child.try_wait().map_err(|e| Stop::Failure(e.to_string()))? {
            Some(s) => break s,
            None if started.elapsed() > CALL_TIMEOUT => {
                let _ = child.kill();
                return Err(Stop::Failure(format!(
                    "claude timed out after {}s",
                    CALL_TIMEOUT.as_secs()
                )));
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    };
    let _ = writer.join();
    let stdout = out.join().unwrap_or_default();
    let stderr = String::from_utf8_lossy(&err.join().unwrap_or_default()).to_string();
    let parsed: Option<Value> = serde_json::from_slice::<Value>(&stdout)
        .ok()
        .map(|v| match v {
            Value::Array(a) => a
                .into_iter()
                .rev()
                .find(|x| x["type"] == "result")
                .unwrap_or(Value::Null),
            v => v,
        });
    let message = parsed
        .as_ref()
        .and_then(|v| v["result"].as_str())
        .unwrap_or("")
        .to_string();
    let is_error = !status.success()
        || parsed
            .as_ref()
            .is_none_or(|v| v["is_error"].as_bool().unwrap_or(false));
    if is_error {
        let detail = format!("{message} {stderr}");
        return Err(classify(detail.trim()));
    }
    let v = parsed.unwrap();
    let u = &v["usage"];
    let n = |k: &str| u[k].as_u64().unwrap_or(0);
    let usage = json!({
        "duration_ms": v["duration_ms"].as_u64().unwrap_or(0), "turns": v["num_turns"].as_u64().unwrap_or(0),
        "input_tokens": n("input_tokens"), "output_tokens": n("output_tokens"),
        "cache_read_input_tokens": n("cache_read_input_tokens"), "cache_creation_input_tokens": n("cache_creation_input_tokens"),
        "models": v["modelUsage"].as_object().map(|m| m.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
    });
    Ok(CallOutput {
        text: message,
        usage,
    })
}

fn classify(detail: &str) -> Stop {
    let t = detail.to_lowercase();
    if [
        "failed to authenticate",
        "not logged in",
        "please run /login",
        "invalid api key",
        "oauth",
    ]
    .iter()
    .any(|p| t.contains(p))
    {
        return Stop::Auth(detail.to_string());
    }
    if [
        "rate limit",
        "usage limit",
        "hit your limit",
        "limit reached",
        "too many requests",
        "overloaded",
        " 429",
        " 529",
    ]
    .iter()
    .any(|p| t.contains(p))
    {
        return Stop::RateLimit(detail.to_string());
    }
    Stop::Failure(detail.to_string())
}

/// Calls the model, backing off exponentially on rate limits; ends with `Stop::RateLimit` if they persist.
fn call(cfg: &AutoConfig, shared: &Shared, user: &str, cwd: &Path) -> Result<CallOutput, Stop> {
    let mut last = String::new();
    for k in 0..=RATE_LIMIT_RETRIES {
        match spawn_once(&cfg.model, SYSTEM, user, cwd) {
            Ok(o) => {
                let mut t = shared.totals.lock().unwrap();
                t.calls += 1;
                t.turns += o.usage["turns"].as_u64().unwrap_or(0);
                t.input_tokens += o.usage["input_tokens"].as_u64().unwrap_or(0);
                t.output_tokens += o.usage["output_tokens"].as_u64().unwrap_or(0);
                t.cache_read_input_tokens +=
                    o.usage["cache_read_input_tokens"].as_u64().unwrap_or(0);
                t.cache_creation_input_tokens +=
                    o.usage["cache_creation_input_tokens"].as_u64().unwrap_or(0);
                t.duration_ms += o.usage["duration_ms"].as_u64().unwrap_or(0);
                return Ok(o);
            }
            Err(Stop::RateLimit(m)) => {
                last = m;
                if k < RATE_LIMIT_RETRIES && !shared.stop.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(shared.backoff_ms * (1 << k)));
                }
            }
            Err(e) => return Err(e),
        }
    }
    Err(Stop::RateLimit(last))
}

/// Extracts the JSON object from a model reply, tolerating code fences or stray text around it.
pub fn parse_answer(text: &str) -> Option<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(text.trim()) {
        return v.is_object().then_some(v);
    }
    let (a, b) = (text.find('{')?, text.rfind('}')?);
    serde_json::from_str::<Value>(text.get(a..=b)?)
        .ok()
        .filter(|v| v.is_object())
}

/// One independent pass: an answer, the validator's verdict, one retry with the errors.
fn run_pass(
    d: &Dossier,
    cfg: &AutoConfig,
    shared: &Shared,
    cwd: &Path,
    pass: usize,
) -> Result<Value, Stop> {
    let mut retry: Option<(String, Vec<String>)> = None;
    let (mut answer, mut errors, mut usages) = (Value::Null, vec![], vec![]);
    for attempt in 1..=2 {
        let user = prompt(d, retry.as_ref().map(|(p, e)| (p.as_str(), e.as_slice())));
        let out = call(cfg, shared, &user, cwd)?;
        usages.push(out.usage);
        match parse_answer(&out.text) {
            Some(v) => {
                errors = validate_answer(d, &v);
                answer = v;
            }
            None => {
                errors = vec!["the reply is not a JSON object".to_string()];
                answer = Value::String(out.text.chars().take(2000).collect());
            }
        }
        if errors.is_empty() {
            return Ok(
                json!({"pass": pass, "valid": true, "errors": [], "attempts": attempt, "answer": answer, "usage": usages}),
            );
        }
        retry = Some((answer.to_string(), errors.clone()));
    }
    Ok(
        json!({"pass": pass, "valid": false, "errors": errors, "attempts": 2, "answer": answer, "usage": usages}),
    )
}

fn evidence_of(d: &Dossier, passes: &[Value]) -> (Vec<Value>, Vec<Value>) {
    let mut locators = vec![];
    let mut cited = vec![];
    let mut seen = HashSet::new();
    for p in passes.iter().filter(|p| p["valid"] == true) {
        for c in p["answer"]["evidence"].as_array().into_iter().flatten() {
            let id = c["id"].as_str().unwrap_or("");
            let Some(line) = d.line(id) else { continue };
            cited
                .push(json!({"pass": p["pass"], "id": id, "quote": c["quote"], "line": line.text}));
            if let Some(l) = &line.locator {
                let v = serde_json::to_value(l).unwrap();
                if seen.insert(v.to_string()) {
                    locators.push(v);
                }
            }
        }
    }
    (locators, cited)
}

fn combine(d: &Dossier, cfg: &AutoConfig, passes: Vec<Value>) -> Value {
    let all_valid = passes.iter().all(|p| p["valid"] == true);
    let answers: Vec<&Value> = passes.iter().map(|p| &p["answer"]).collect();
    let agree = all_valid
        && answers
            .windows(2)
            .all(|w| answers_agree(&d.field, w[0], w[1]));
    let (locators, cited) = evidence_of(d, &passes);
    let (value, resolution, rationale, merged) = if agree {
        let a = answers[0];
        let merged = if d.field == "flex_endpoint" {
            merge_flex(&answers)
        } else {
            a.clone()
        };
        let res = if no_decision(&merged, &d.field) {
            "unknown"
        } else {
            "resolved"
        };
        (
            a["value"].clone(),
            res,
            a["rationale"].as_str().unwrap_or("").to_string(),
            merged,
        )
    } else if all_valid {
        let vals: Vec<String> = answers.iter().map(|a| a["value"].to_string()).collect();
        (
            json!("unknown"),
            "unresolved_conflict",
            format!("independent passes disagree: {}", vals.join(" vs ")),
            Value::Null,
        )
    } else {
        (
            json!("unknown"),
            "unresolved_conflict",
            "a pass failed validation after its retry".to_string(),
            Value::Null,
        )
    };
    let dossier_sha = hex::encode(Sha256::digest(d.render().as_bytes()));
    json!({
        "item_id": d.item_id, "value": value, "resolution": resolution, "agent": agent_label(cfg), "rationale": rationale,
        "adjudicated_at": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        "evidence": locators, "cited": cited, "answer": merged, "passes": passes, "dossier_sha256": dossier_sha,
    })
}

fn adjudicate(d: &Dossier, cfg: &AutoConfig, shared: &Shared, cwd: &Path) -> Result<Value, Stop> {
    let mut passes = vec![];
    for pass in 1..=cfg.passes {
        passes.push(run_pass(d, cfg, shared, cwd, pass)?);
    }
    Ok(combine(d, cfg, passes))
}

fn read_done(path: &Path) -> HashSet<String> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    text.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|v| v["item_id"].as_str().map(String::from))
        .collect()
}

pub struct ExecSummary {
    pub processed: usize,
    pub counts: BTreeMap<String, usize>,
    pub remaining: usize,
    pub stopped: Option<(String, String)>,
    pub usage: Value,
    pub seconds: f64,
}

/// Runs the pending dossiers with bounded concurrency, appending one line per finished item.
pub fn execute(dossiers: &[Dossier], cfg: &AutoConfig, out: &Path) -> anyhow::Result<ExecSummary> {
    anyhow::ensure!(
        cfg.passes >= 2,
        "at least two independent passes are required"
    );
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let done = read_done(out);
    let pending: Vec<&Dossier> = dossiers
        .iter()
        .filter(|d| !done.contains(&d.item_id))
        .collect();
    let file = Mutex::new(
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out)?,
    );
    let cwd = work_dir()?;
    let shared = Shared {
        stop: AtomicBool::new(false),
        reason: Mutex::new(None),
        totals: Mutex::new(Totals::default()),
        consecutive_failures: AtomicUsize::new(0),
        backoff_ms: std::env::var("IGNIS_AUTO_BACKOFF_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30_000),
    };
    let (next, written) = (
        AtomicUsize::new(0),
        Mutex::new(BTreeMap::<String, usize>::new()),
    );
    let started = Instant::now();
    let stop_with = |kind: &str, msg: String| {
        shared.stop.store(true, Ordering::SeqCst);
        shared
            .reason
            .lock()
            .unwrap()
            .get_or_insert((kind.to_string(), msg));
    };
    std::thread::scope(|s| {
        for _ in 0..cfg.concurrency.clamp(1, 16) {
            s.spawn(|| loop {
                if shared.stop.load(Ordering::SeqCst) {
                    break;
                }
                let i = next.fetch_add(1, Ordering::SeqCst);
                let Some(d) = pending.get(i) else { break };
                match adjudicate(d, cfg, &shared, &cwd) {
                    Ok(v) => {
                        shared.consecutive_failures.store(0, Ordering::SeqCst);
                        let line = ignis_core::hashing::sorted(&v).to_string();
                        let mut f = file.lock().unwrap();
                        if writeln!(f, "{line}").and_then(|_| f.flush()).is_err() {
                            stop_with("io", "cannot append to the shard file".to_string());
                        }
                        drop(f);
                        *written
                            .lock()
                            .unwrap()
                            .entry(v["resolution"].as_str().unwrap_or("").to_string())
                            .or_default() += 1;
                    }
                    Err(Stop::RateLimit(m)) => stop_with("rate_limit", m),
                    Err(Stop::Auth(m)) => stop_with("auth", m),
                    Err(Stop::Failure(m)) => {
                        if shared.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1
                            >= MAX_CONSECUTIVE_FAILURES
                        {
                            stop_with("failures", m);
                        }
                    }
                }
            });
        }
    });
    if std::fs::metadata(out).is_ok_and(|m| m.len() == 0) {
        let _ = std::fs::remove_file(out); // nothing finished: leave no empty shard behind
    }
    let finished = read_done(out);
    let remaining = dossiers
        .iter()
        .filter(|d| !finished.contains(&d.item_id))
        .count();
    let counts = written.into_inner().unwrap();
    let t = shared.totals.into_inner().unwrap();
    let usage = json!({"calls": t.calls, "turns": t.turns, "input_tokens": t.input_tokens, "output_tokens": t.output_tokens,
        "cache_read_input_tokens": t.cache_read_input_tokens, "cache_creation_input_tokens": t.cache_creation_input_tokens, "api_seconds": t.duration_ms as f64 / 1000.0});
    Ok(ExecSummary {
        processed: counts.values().sum(),
        counts,
        remaining,
        stopped: shared.reason.into_inner().unwrap(),
        usage,
        seconds: started.elapsed().as_secs_f64(),
    })
}

/// `ignis data review auto`: dry-run plan by default, adjudicates with `--execute`.
pub fn run(cfg: &AutoConfig) -> anyhow::Result<()> {
    anyhow::ensure!(
        cfg.passes >= 2,
        "--passes must be at least 2 (independent passes; no single-pass mode)"
    );
    let items = dossier::read_items(&cfg.family)?;
    anyhow::ensure!(
        !items.is_empty(),
        "no review items for {}; run `ignis data ingest` and `ignis data review export`",
        cfg.family
    );
    let out = shard_path(&cfg.family, cfg.shard);
    let done = read_done(&out);
    let in_shard: Vec<_> = items
        .iter()
        .filter(|i| shard_of(&i.id, cfg.shard.1) == cfg.shard.0)
        .filter(|i| cfg.only.as_ref().is_none_or(|r| r.is_match(&i.id)))
        .collect();
    let mut pending: Vec<_> = in_shard
        .iter()
        .copied()
        .filter(|i| !done.contains(&i.id))
        .collect();
    let pending_all = pending.len();
    if let Some(k) = cfg.limit {
        pending.truncate(k);
    }
    let ctx = dossier::Ctx::load(&cfg.family)?;
    let missing = ctx.assets.missing(&dossier::wanted_files(&ctx, &pending));
    let fetch_bytes: i64 = missing.iter().filter_map(|f| f.size_bytes).sum();
    let mut shard_sizes = BTreeMap::new();
    for i in &items {
        *shard_sizes
            .entry(shard_of(&i.id, cfg.shard.1))
            .or_insert(0usize) += 1;
    }
    let plan = json!({
        "family": cfg.family, "shard": format!("{}/{}", cfg.shard.0, cfg.shard.1), "output": out.display().to_string(),
        "items_total": items.len(), "items_in_shard": in_shard.len(), "already_done": in_shard.len() - pending_all,
        "pending": pending_all, "this_run": pending.len(), "shard_sizes": shard_sizes,
        "model": cfg.model, "passes": cfg.passes, "concurrency": cfg.concurrency,
        "estimated_turns": pending.len() * cfg.passes, "worst_case_turns_with_retries": pending.len() * cfg.passes * 2,
        "fetch_files": missing.len(), "fetch_bytes": fetch_bytes, "claude_bin": claude_bin(),
    });
    if !cfg.execute {
        let built: Vec<Dossier> = pending
            .iter()
            .map(|i| dossier::build(&ctx, i))
            .collect::<anyhow::Result<_>>()?;
        if let Some(d) = &cfg.dump_dossiers {
            dossier::dump(d, &built)?;
        }
        let sizes: Vec<usize> = built.iter().map(|d| d.render().len()).collect();
        let mut p = plan;
        p["dossier_chars_mean"] = json!(sizes.iter().sum::<usize>() / sizes.len().max(1));
        p["dossier_chars_max"] = json!(sizes.iter().max());
        p["status"] = json!("dry-run; pass --execute to run");
        println!("{}", serde_json::to_string_pretty(&p)?);
        return Ok(());
    }
    anyhow::ensure!(
        fetch_bytes <= MAX_FETCH_BYTES,
        "dossiers would fetch {fetch_bytes} bytes, above the {MAX_FETCH_BYTES} cap"
    );
    let fetched = dossier::fetch_missing(&missing)?;
    let dossiers: Vec<Dossier> = pending
        .iter()
        .map(|i| dossier::build(&ctx, i))
        .collect::<anyhow::Result<_>>()?;
    if let Some(d) = &cfg.dump_dossiers {
        dossier::dump(d, &dossiers)?;
    }
    let prof = load_profile("cpu-local")?;
    let mut job = Job::start("review.auto", &prof, plan.clone())?;
    let s = execute(&dossiers, cfg, &out)?;
    job.checkpoint(&out);
    let per_item = if s.processed > 0 {
        s.seconds / s.processed as f64 * cfg.concurrency as f64
    } else {
        0.0
    };
    let summary = json!({
        "family": cfg.family, "shard": format!("{}/{}", cfg.shard.0, cfg.shard.1), "output": out.display().to_string(),
        "processed": s.processed, "by_resolution": s.counts, "remaining_this_run": s.remaining, "remaining_in_shard": pending_all - s.processed,
        "fetched_files": fetched, "wall_seconds": s.seconds, "seconds_per_item_per_worker": per_item, "usage": s.usage,
        "stopped": s.stopped.as_ref().map(|(k, m)| json!({"kind": k, "message": m})),
    });
    job.finish(summary.clone());
    println!("{}", serde_json::to_string_pretty(&summary)?);
    if let Some(("auth", _)) = s.stopped.as_ref().map(|(k, m)| (k.as_str(), m.as_str())) {
        anyhow::bail!("the claude CLI is not authenticated; run `claude auth login` (subscription) and re-run: nothing was lost");
    }
    Ok(())
}

/// Merge every shard file into one list, one line per item: the first file in sorted order wins.
pub fn merge_shards(family: &str) -> anyhow::Result<(Vec<Value>, Vec<String>)> {
    let dir = paths::review(family).join("drafts");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    files.retain(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("auto-") && n.contains("of") && n.ends_with(".jsonl"))
    });
    files.sort();
    let (mut seen, mut merged, mut dups) = (HashSet::new(), vec![], vec![]);
    for f in &files {
        for v in read_jsonl_values(f)? {
            let id = v["item_id"].as_str().unwrap_or("").to_string();
            if seen.insert(id.clone()) {
                merged.push(v);
            } else {
                dups.push(id);
            }
        }
    }
    Ok((merged, dups))
}

/// Adjudications not already applied unchanged (so `auto-import` can run repeatedly).
pub fn new_adjudications(merged: Vec<Value>, existing: &[Value]) -> Vec<Value> {
    let mut latest: BTreeMap<&str, &Value> = BTreeMap::new();
    for a in existing {
        latest.insert(a["item_id"].as_str().unwrap_or(""), a);
    }
    merged
        .into_iter()
        .filter(|a| {
            latest
                .get(a["item_id"].as_str().unwrap_or(""))
                .is_none_or(|old| {
                    old["dossier_sha256"] != a["dossier_sha256"]
                        || old["value"] != a["value"]
                        || old["resolution"] != a["resolution"]
                })
        })
        .collect()
}
