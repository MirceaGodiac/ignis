//! M5 metrics over a stored fixture log with fixed host timestamps: exact nearest-rank latency
//! percentiles, cache hit per turn and question, thread totals turned into per-turn usage (a
//! thread restart included), price-list cost, and explicit `unpriced` / `no_usage` statuses.

mod common;

use common::scope;
use ignis_core::contracts::*;
use ignis_research::metrics::{aggregate, measure_episode, PriceList};
use ignis_research::ResearchStore;

const EP: &str = "fixture-metrics-01";

struct Log {
    events: Vec<ResearchEvent>,
}

impl Log {
    /// Append an event at `ms` milliseconds after 2026-11-15T10:00:00Z.
    fn at(&mut self, ms: u64, turn: Option<&str>, body: EventBody) {
        let seq = self.events.len() as u64 + 1;
        let t = chrono::DateTime::parse_from_rfc3339("2026-11-15T10:00:00Z").unwrap()
            + chrono::Duration::milliseconds(ms as i64);
        self.events.push(ResearchEvent {
            schema_version: ResearchSchema::V2,
            event_id: EventId::new(format!("fixture-ev-{seq}")).unwrap(),
            episode_id: EpisodeId::new(EP).unwrap(),
            seq,
            occurred_at: Timestamp::parse(
                &t.to_utc()
                    .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            )
            .unwrap(),
            causation: Causation {
                runtime_turn_id: turn.map(str::to_string),
                ..Default::default()
            },
            body,
            origin: Origin::SyntheticFixture,
        });
    }

    fn user(&mut self, ms: u64, n: u32) {
        self.at(
            ms,
            None,
            EventBody::UserMessage {
                message_id: MessageId::new(format!("fixture-msg-{n}")).unwrap(),
                text: format!("question {n}"),
            },
        );
    }

    fn tool(&mut self, start: u64, took: u64, turn: &str, call: &str, tool: &str) {
        let call_id = ToolCallId::new(call).unwrap();
        self.at(
            start,
            Some(turn),
            EventBody::ToolStarted {
                call_id: call_id.clone(),
                tool: tool.into(),
                arguments: None,
            },
        );
        self.at(
            start + took,
            Some(turn),
            EventBody::ToolFinished {
                result: ToolResult {
                    schema_version: ToolsSchema::V1,
                    call_id,
                    tool: tool.into(),
                    execution_status: ExecutionStatus::Completed,
                    scientific_status: ScientificStatus::NotAssessed,
                    result: None,
                    job: None,
                    artifact_refs: vec![],
                    error: None,
                    origin: Origin::SyntheticFixture,
                },
            },
        );
    }

    fn usage(&mut self, ms: u64, turn: &str, u: UsageReport) {
        self.at(ms, Some(turn), EventBody::UsageReported { usage: u });
    }
}

fn increment(model: &str, input: u64, cached: u64, written: u64, output: u64) -> UsageReport {
    UsageReport {
        accounting: UsageAccounting::Increment,
        model: Some(model.into()),
        input_tokens: input,
        cached_input_tokens: Some(cached),
        cache_write_tokens: Some(written),
        output_tokens: output,
        reasoning_tokens: None,
        reported_cost: None,
    }
}

fn total(input: u64, cached: u64, output: u64) -> UsageReport {
    UsageReport {
        accounting: UsageAccounting::ThreadTotal,
        model: None,
        input_tokens: input,
        cached_input_tokens: Some(cached),
        cache_write_tokens: None,
        output_tokens: output,
        reasoning_tokens: None,
        reported_cost: None,
    }
}

fn episode(harness_model: &str) -> ResearchEpisode {
    let t = Timestamp::parse("2026-11-15T10:00:00Z").unwrap();
    ResearchEpisode {
        schema_version: ResearchSchema::V2,
        episode_id: EpisodeId::new(EP).unwrap(),
        workspace_id: WorkspaceId::new("ws-m").unwrap(),
        goal: "metrics fixture".into(),
        status: EpisodeStatus::Ready,
        runtime: None,
        parent: None,
        pins: EpisodePins {
            harness: Some(HarnessRef {
                digest: Digest::parse(&format!("sha256:{}", "4a".repeat(32))).unwrap(),
                label: "h1".into(),
                runtime_build: RuntimeBuildId::new("fixture-runtime-build").unwrap(),
                model: harness_model.into(),
                effort: None,
            }),
            ..Default::default()
        },
        scope_id: ScopeId::new("scope-ws-m").unwrap(),
        revision: 0,
        last_seq: 0,
        created_at: t.clone(),
        updated_at: t,
        origin: Origin::SyntheticFixture,
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn metrics_of_a_fixture_log_are_measured_exactly_and_fail_closed() {
    let mut log = Log { events: vec![] };
    // Q1, turn t1 (Claude-style increments): 20 searches taking 10, 20, …, 200 ms.
    log.user(0, 1);
    for i in 1..=20u64 {
        log.tool(
            i * 1000,
            i * 10,
            "t1",
            &format!("fixture-call-{i}"),
            "sources_search",
        );
    }
    log.usage(
        30_000,
        "t1",
        increment("claude-opus-5-5", 10_000, 6_000, 1_000, 2_000),
    );
    log.usage(
        30_500,
        "t1",
        increment("claude-opus-5-5", 5_000, 3_000, 0, 1_000),
    );
    // An unfinished call.
    log.at(
        31_000,
        Some("t1"),
        EventBody::ToolStarted {
            call_id: ToolCallId::new("fixture-call-open").unwrap(),
            tool: "analysis_execute".into(),
            arguments: None,
        },
    );
    // Q2, turn t2 (Codex-style running totals, model from the harness pin): two partials.
    log.user(40_000, 2);
    log.usage(41_000, "t2", total(1_000, 200, 100));
    log.usage(42_000, "t2", total(4_000, 2_000, 300));
    // Q3, turn t3: the totals continue (delta 2000 input, 1000 cached) …
    log.user(50_000, 3);
    log.usage(51_000, "t3", total(6_000, 3_000, 500));
    // … then a new thread starts (totals go down): counted from zero.
    log.usage(52_000, "t3", total(500, 100, 50));
    // Q4: no usage reported at all.
    log.user(60_000, 4);
    log.at(
        60_500,
        Some("t4"),
        EventBody::AssistantMessage {
            message_id: MessageId::new("fixture-msg-a4").unwrap(),
            text: "no usage".into(),
        },
    );
    check_event_log(&log.events).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let store = ResearchStore::open(dir.path().join("state.sqlite")).unwrap();
    let sc = scope("ws-m", false);
    // The harness model is not in the built-in list: Codex-style usage is unpriced.
    assert!(store
        .import_episode(&sc, episode("gpt-fixture"), log.events.clone())
        .unwrap());
    let prices = PriceList::builtin();
    let id = EpisodeId::new(EP).unwrap();
    let measure = measure_episode(&store, &sc, &id, &prices).unwrap();
    let m = &measure.metrics;
    m.validate().unwrap();
    assert_eq!(m.last_seq, log.events.len() as u64);
    assert_eq!(m.price_list, "ignis-llm-prices@2026-11-15");

    // Latency: nearest rank over the 20 measured samples.
    let search = m.tools.iter().find(|t| t.tool == "sources_search").unwrap();
    let lat = search.latency.as_ref().unwrap();
    assert_eq!((search.calls, search.unfinished, lat.samples), (20, 0, 20));
    assert!(close(lat.p50_ms.get(), 100.0), "{lat:?}");
    assert!(close(lat.p95_ms.get(), 190.0), "{lat:?}");
    assert!(close(lat.max_ms.get(), 200.0), "{lat:?}");
    let open = m
        .tools
        .iter()
        .find(|t| t.tool == "analysis_execute")
        .unwrap();
    assert_eq!((open.calls, open.unfinished), (1, 1));
    assert!(open.latency.is_none());

    // Turn t1: increments add up; cache hit = 9000 / 15000.
    let t1 = &m.turns[0];
    assert_eq!((t1.runtime_turn_id.as_str(), t1.tool_calls), ("t1", 21));
    assert_eq!(t1.tokens.input_tokens, 15_000);
    assert_eq!(t1.tokens.cached_input_tokens, Some(9_000));
    assert!(close(t1.tokens.cache_hit_pct.unwrap().get(), 60.0));
    let CostStatus::Priced { cost } = &t1.cost else {
        panic!("{:?}", t1.cost)
    };
    // fresh 5000 × 4 + cached 9000 × 0.20 + written 1000 × 5 + output 3000 × 20, per million.
    assert!(close(
        cost.amount.get(),
        (20_000.0 + 1_800.0 + 5_000.0 + 60_000.0) / 1e6
    ));

    // Turn t2: the last total minus nothing before it (the partial is not counted twice).
    let t2 = &m.turns[1];
    assert_eq!(
        (t2.tokens.input_tokens, t2.tokens.cached_input_tokens),
        (4_000, Some(2_000))
    );
    assert_eq!(t2.tokens.output_tokens, 300);
    assert!(close(t2.tokens.cache_hit_pct.unwrap().get(), 50.0));
    match &t2.cost {
        CostStatus::Unpriced { models, reasons } => {
            assert_eq!(models, &["gpt-fixture"]);
            assert!(reasons[0].contains("not in price list"), "{reasons:?}");
        }
        other => panic!("{other:?}"),
    }
    // Turn t3: 2000 more on the old thread, then 500 on a new one.
    let t3 = &m.turns[2];
    assert_eq!(
        (t3.tokens.input_tokens, t3.tokens.cached_input_tokens),
        (2_500, Some(1_100))
    );

    // Questions: one turn each; Q4 has no usage (not zero dollars).
    assert_eq!(m.questions.len(), 4);
    assert_eq!(
        m.questions.iter().map(|q| q.turns).collect::<Vec<_>>(),
        [1, 1, 1, 1]
    );
    assert_eq!(m.questions[0].tool_calls, 21);
    assert!(matches!(m.questions[0].cost, CostStatus::Priced { .. }));
    assert!(close(m.questions[0].wall_time_s.get(), 31.0));
    assert_eq!(m.questions[3].cost, CostStatus::NoUsage);
    assert_eq!(m.questions[3].tokens.reports, 0);
    // The episode mixes priced and unpriced usage: the total is unpriced.
    assert!(matches!(&m.cost, CostStatus::Unpriced { models, .. } if models == &["gpt-fixture"]));
    assert_eq!(m.tokens.input_tokens, 15_000 + 4_000 + 2_500);

    // Aggregate: the per-question mean is unpriced as long as any usage is.
    let w = aggregate(std::slice::from_ref(&measure), &prices);
    w.validate().unwrap();
    assert_eq!((w.episodes, w.questions), (1, 4));
    assert!(matches!(w.cost_per_question, CostStatus::Unpriced { .. }));

    // Priced when the list prices the harness model (a different, versioned list).
    let list = PriceList::parse(
        r#"
name = "fixture-prices"
version = "1"
currency = "USD"
source = "synthetic fixture (not a real price)"
[[models]]
id = "gpt-fixture"
input_per_mtok = 1.0
cached_input_per_mtok = 0.5
output_per_mtok = 10.0
[[models]]
id = "claude-opus-5-5"
input_per_mtok = 4.0
cached_input_per_mtok = 0.20
cache_write_per_mtok = 5.0
output_per_mtok = 20.0
"#,
    )
    .unwrap();
    let m2 = measure_episode(&store, &sc, &id, &list).unwrap();
    let CostStatus::Priced { cost } = &m2.metrics.turns[1].cost else {
        panic!()
    };
    // fresh 2000 × 1 + cached 2000 × 0.5 + output 300 × 10, per million.
    assert!(close(
        cost.amount.get(),
        (2_000.0 + 1_000.0 + 3_000.0) / 1e6
    ));
    assert_eq!(cost.price_list.as_deref(), Some("fixture-prices@1"));
    let w2 = aggregate(&[m2], &list);
    let (CostStatus::Priced { cost: total }, CostStatus::Priced { cost: per_q }) =
        (&w2.cost, &w2.cost_per_question)
    else {
        panic!("{w2:?}")
    };
    assert!(close(per_q.amount.get(), total.amount.get() / 4.0));

    // A list with an unknown field, a negative price or a duplicate id is refused.
    for bad in [
        "name='x'\nversion='1'\ncurrency='USD'\nsource='s'\nextra=1",
        "name='x'\nversion='1'\ncurrency='USD'\nsource='s'\n[[models]]\nid='m'\ninput_per_mtok=-1.0\noutput_per_mtok=1.0",
        "name='x'\nversion='1'\ncurrency='USD'\nsource='s'\n[[models]]\nid='m'\ninput_per_mtok=1.0\noutput_per_mtok=1.0\n[[models]]\nid='m'\ninput_per_mtok=1.0\noutput_per_mtok=1.0",
    ] {
        assert!(PriceList::parse(bad).is_err(), "{bad}");
    }
}
