//! Human-readable report from a `score` or `compare` JSON result (numbers only from the result).

use serde_json::{json, Map, Value};

use crate::util::fmt_g_prec;

fn fmt(x: &Value) -> String {
    match x {
        Value::Null => "—".into(),
        Value::Number(n) if n.is_f64() => fmt_g_prec(n.as_f64().unwrap(), 4),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        o => o.to_string(),
    }
}

fn metric_rows(metrics: &Value) -> Vec<String> {
    let mut rows = vec![
        "| metric | group-equal mean | 95% CI | cases | clusters |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ];
    if let Some(m) = metrics.as_object() {
        for (k, v) in m {
            let ci = match v.get("ci").and_then(Value::as_array) {
                Some(c) if c.len() == 2 => format!("[{}, {}]", fmt(&c[0]), fmt(&c[1])),
                _ => "—".into(),
            };
            rows.push(format!(
                "| {k} | {} | {ci} | {} | {} |",
                fmt(v.get("estimate").unwrap_or(&Value::Null)),
                fmt(v.get("n_cases").unwrap_or(&Value::Null)),
                fmt(v.get("n_clusters").unwrap_or(&Value::Null))
            ));
        }
    }
    rows
}

fn accounting(acc: &Value) -> Vec<String> {
    let g = |k: &str| fmt(acc.get(k).unwrap_or(&Value::Null));
    let mut lines = vec![format!(
        "- presented: {}, refused: {} (rate {}), admitted denominator: {}, scored: {}",
        g("presented"),
        g("refused"),
        g("refusal_rate"),
        g("admitted_denominator"),
        g("scored")
    )];
    if let Some(r) = acc.get("refusal_reasons").and_then(Value::as_object) {
        for (reason, n) in r {
            lines.push(format!("  - refused ({}): {reason}", fmt(n)));
        }
    }
    if let Some(u) = acc.get("admitted_unscorable").and_then(Value::as_array) {
        for c in u {
            lines.push(format!(
                "  - admitted but unscorable: {} — {}",
                fmt(&c["case_id"]),
                fmt(&c["reason"])
            ));
        }
    }
    lines
}

fn events(ev: &Value) -> Vec<String> {
    let Some(e) = ev.as_object().filter(|o| !o.is_empty()) else {
        return vec![];
    };
    let h: Vec<f64> = e
        .get("horizons")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default();
    let mut out = vec![format!(
        "Events ({}), horizons {}:",
        fmt(e.get("estimator").unwrap_or(&Value::Null)),
        json!(h)
    )];
    if let Some(c) = e.get("causes").and_then(Value::as_object) {
        for (cause, b) in c {
            let parts: Vec<String> = h
                .iter()
                .zip(
                    b.get("brier")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default(),
                )
                .map(|(hh, x)| {
                    format!(
                        "BS({})={} (n={})",
                        crate::util::fmt_g(*hh),
                        fmt(&x["brier"]),
                        fmt(&x["n"])
                    )
                })
                .collect();
            out.push(format!(
                "- {cause}: integrated Brier {}; {}",
                fmt(b.get("integrated_brier").unwrap_or(&Value::Null)),
                parts.join(", ")
            ));
        }
    }
    out
}

pub fn render(result: &Value) -> String {
    let mut lines: Vec<String> = vec![];
    if let Some(per) = result.get("per_release").and_then(Value::as_object) {
        lines.push(format!(
            "# Comparison on {} ({} common cases)",
            fmt(&result["partition"]),
            fmt(&result["n_cases"])
        ));
        lines.push(String::new());
        if result
            .get("final_test")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            lines.push("**Final test partition (logged reveal).**".into());
        }
        for m in result
            .get("allowed_mismatches")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
        {
            lines.push(format!("- explicitly allowed mismatch: {}", fmt(&m)));
        }
        if let Some(ex) = result
            .get("excluded_incomplete_cases")
            .and_then(Value::as_array)
            .filter(|a| !a.is_empty())
        {
            lines.push(format!(
                "- excluded (not sealed by all releases): {}",
                ex.len()
            ));
        }
        for (rid, agg) in per {
            lines.push(String::new());
            lines.push(format!("## {rid}"));
            lines.extend(accounting(&agg["accounting"]));
            lines.push(String::new());
            lines.extend(metric_rows(&agg["metrics"]));
            lines.push(String::new());
            lines.extend(events(agg.get("events").unwrap_or(&Value::Null)));
        }
        if let Some(p) = result.get("paired_vs_reference").and_then(Value::as_object) {
            for (rid, d) in p {
                lines.push(String::new());
                lines.push(format!(
                    "## Paired differences: {rid} − {}",
                    fmt(&d["minus"])
                ));
                lines.push(String::new());
                lines.extend(metric_rows(&d["differences"]));
            }
        }
    } else {
        let agg = &result["aggregate"];
        let n = result
            .get("cases")
            .and_then(Value::as_array)
            .map_or(0, |a| a.len());
        lines.push(format!("# Scores ({n} cases)"));
        lines.push(String::new());
        lines.extend(accounting(&agg["accounting"]));
        lines.push(String::new());
        lines.extend(metric_rows(&agg["metrics"]));
        lines.push(String::new());
        lines.extend(events(agg.get("events").unwrap_or(&Value::Null)));
    }
    lines.push(String::new());
    lines.push(
        "Lower is better except lpd. Intervals: cluster bootstrap over independent groups (sessions merged); few clusters ⇒ wide, unstable intervals. Read with the per-case table."
            .into(),
    );
    lines.join("\n") + "\n"
}

/// The JSON summary `ignis eval report` prints.
pub fn summary(data: &Value, md: Option<&str>) -> Value {
    let mut groups = Map::new();
    if let Some(per) = data.get("per_release").and_then(Value::as_object) {
        for (k, v) in per {
            groups.insert(k.clone(), v.clone());
        }
    } else if let Some(a) = data.get("aggregate").filter(|a| !a.is_null()) {
        groups.insert("scores".into(), a.clone());
    }
    let mut summary = Map::new();
    for (k, v) in groups {
        let metrics: Map<String, Value> = v["metrics"]
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(n, x)| {
                        (
                            n.clone(),
                            json!({"estimate": x.get("estimate"), "ci": x.get("ci")}),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        summary.insert(
            k,
            json!({"accounting": v["accounting"], "metrics": metrics}),
        );
    }
    json!({"markdown": md, "summary": summary})
}
