//! Scorecard: a Markdown view of one `EvaluationResult` (the contract, not an internal format).
//!
//! The card never shows a conditional metric without the case accounting next to it: scored-case
//! quality and coverage over all cases are read together.

use std::collections::BTreeMap;
use std::fmt::Write;

use ignis_core::contracts::{
    CaseFailureKind, EvaluationResult, MetricValue, Origin, Population, Verdict,
};

fn fmt_value(m: &MetricValue) -> String {
    match &m.value {
        None => "undefined (no cases)".into(),
        Some(v) => {
            let x = v.get();
            let s = if x != 0.0 && (x.abs() < 1e-3 || x.abs() >= 1e4) {
                format!("{x:.4e}")
            } else {
                format!("{x:.4}")
            };
            if m.unit == "1" {
                s
            } else {
                format!("{s} {}", m.unit)
            }
        }
    }
}

fn short(id: &str) -> String {
    let c: Vec<char> = id.chars().collect();
    if c.len() > 23 {
        format!(
            "{}…{}",
            c[..17].iter().collect::<String>(),
            c[c.len() - 4..].iter().collect::<String>()
        )
    } else {
        id.to_string()
    }
}

/// The wire name of a contract enum (`development_feedback`, `liquid_diameter`, ...).
fn wire<T: serde::Serialize>(x: &T) -> String {
    match serde_json::to_value(x) {
        Ok(serde_json::Value::String(s)) => s,
        Ok(v) => v.to_string(),
        Err(_) => "?".into(),
    }
}

pub fn render(r: &EvaluationResult) -> String {
    let q = &r.request;
    let mut s = String::new();
    let _ = writeln!(s, "# Scorecard `{}`\n", r.result_id);
    // This card is the model scorecard; a harness comparison (T11) has its own view.
    let model = match q.model() {
        Ok(m) => m,
        Err(e) => {
            let _ = writeln!(s, "Not a model evaluation: {e}");
            return s;
        }
    };
    if r.origin == Origin::SyntheticFixture {
        let _ = writeln!(
            s,
            "> **Synthetic fixture.** These numbers come from generated data or fixtures. They are not NASA results and qualify nothing.\n"
        );
    }
    let frozen = match &q.protocol.frozen_hash {
        Some(h) => format!("frozen `{h}`"),
        None => "**not frozen** (development protocol)".into(),
    };
    let _ = writeln!(s, "| | |\n|---|---|");
    let _ = writeln!(s, "| Purpose | {} |", wire(&q.purpose));
    let _ = writeln!(
        s,
        "| Protocol | `{}@{}`, {frozen} |",
        q.protocol.protocol_id, q.protocol.version
    );
    let _ = writeln!(s, "| Target | {} in {} |", wire(&model.target), model.unit);
    let _ = writeln!(
        s,
        "| Dataset | `{}` (`{}`) |",
        model.dataset_id,
        short(model.dataset_hash.as_str())
    );
    let _ = writeln!(
        s,
        "| Split / partition | `{}` / {} |",
        model.split_id,
        wire(&q.partition)
    );
    let cut: Vec<String> = q.cutoffs_s.iter().map(|c| format!("{}", c.get())).collect();
    let _ = writeln!(s, "| Cutoffs (s) | {} |", cut.join(", "));
    let _ = writeln!(s, "| Candidate | `{}` |", model.candidate.release_id);
    for b in &q.baselines {
        let _ = writeln!(s, "| Baseline | `{}` |", b.release_id);
    }
    let _ = writeln!(s, "| Request hash | `{}` |", r.request_hash);
    let verdict = match r.verdict {
        Verdict::NotAssessed => "not assessed (development feedback does not qualify)",
        Verdict::Inconclusive => "inconclusive",
        Verdict::NotQualified => "not qualified",
        Verdict::Qualified => "qualified",
    };
    let _ = writeln!(s, "| Verdict | {verdict} |");
    let _ = writeln!(s, "| Created | {} |\n", r.created_at.as_str());

    let k = r.cases;
    let _ = writeln!(s, "## Cases (candidate)\n");
    let _ = writeln!(
        s,
        "{} cases: {} scored, {} refused, {} failed. Coverage {:.1} % (scored / all cases).\n",
        k.total,
        k.scored,
        k.refused,
        k.failed,
        100.0 * r.coverage.get()
    );

    // subject -> metric rows
    let mut by_subject: BTreeMap<(bool, String), Vec<&MetricValue>> = BTreeMap::new();
    for m in &r.metrics {
        let is_baseline = m.subject != model.candidate.release_id;
        by_subject
            .entry((is_baseline, m.subject.to_string()))
            .or_default()
            .push(m);
    }
    let _ = writeln!(s, "## Metrics\n");
    let _ = writeln!(s, "Scored-case metrics are conditional on the cases a model answered; read them with its coverage.\n");
    for ((is_baseline, subject), rows) in &by_subject {
        let _ = writeln!(
            s,
            "### {} `{}`\n",
            if *is_baseline {
                "Baseline"
            } else {
                "Candidate"
            },
            short(subject)
        );
        let _ = writeln!(
            s,
            "| Metric | Group | Population | Value | n |\n|---|---|---|---|---|"
        );
        for m in rows {
            let group = if m.group.is_empty() {
                "all".to_string()
            } else {
                m.group
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let pop = match m.population {
                Population::ScoredCases => "scored cases",
                Population::AllCases => "all cases",
            };
            let _ = writeln!(
                s,
                "| {} (v{}) | {group} | {pop} | {} | {} |",
                m.metric,
                m.version,
                fmt_value(m),
                m.n
            );
        }
        let _ = writeln!(s);
    }

    let _ = writeln!(s, "## Refused and failed cases\n");
    if r.failures.is_empty() {
        let _ = writeln!(s, "None.");
    } else {
        let _ = writeln!(s, "| Case | Subject | Kind | Reason |\n|---|---|---|---|");
        for f in &r.failures {
            let kind = match f.kind {
                CaseFailureKind::Refused => "refused",
                CaseFailureKind::Failed => "failed",
            };
            let _ = writeln!(
                s,
                "| `{}` | `{}` | {kind} | {} |",
                f.case_id,
                short(f.subject.as_str()),
                f.reason.replace('|', "\\|")
            );
        }
    }
    s
}
