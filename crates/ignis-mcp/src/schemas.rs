//! Input schemas of the tools, generated from the contract types the catalog names. Subschemas are
//! inlined (no `$ref`/`$defs`): agent runtimes flatten MCP schemas into provider function
//! definitions, and a dangling reference would reach the model as an untyped field.
//!
//! **Slim, not different.** The runtime re-sends every input schema with every model request, so
//! what the model receives is [`slim`]med from the contract's schema: noise it does not use goes
//! (titles, numeric `format`s, empty defaults, the opaque-id pattern, task tags in doc comments), a
//! description on a structured node keeps its first sentence (leaf fields keep theirs: they say how
//! to fill the field), string-variant `oneOf`s become one `enum`, and tagged variants that differ
//! only in their tag merge into one object. Required fields, enums and every shape stay. Fields
//! the host writes and refuses in a submission (`checked_by`, `resolved`) are left out of the
//! `answer_submit` schema. Decoding is unchanged: arguments are still the full contract
//! (`deny_unknown_fields`, `validate()`), and the exported `schemas/` the web uses are untouched.

use ignis_core::contracts::{
    AnalysisExecuteRequest, AnswerSubmission, ArtifactReadRequest, ConditionsQuery, DatasetRef,
    ForecastRequest, GraphNeighborsRequest, JobLookup, OutlineRequest, ReadRequest, ReleaseRef,
    SearchRequest, SkillRef, WebFetchRequest, WebSearchRequest, WikiPatch, WikiReadRequest,
};
use schemars::generate::SchemaSettings;
use schemars::JsonSchema;
use serde_json::{json, Map, Value};

fn inline<T: JsonSchema>() -> Value {
    let mut schema = SchemaSettings::draft2020_12()
        .with(|s| s.inline_subschemas = true)
        .into_generator()
        .into_root_schema_for::<T>();
    schema.remove("$schema");
    schema.to_value()
}

/// The JSON Schema the model receives for an input contract named in
/// [`ignis_core::contracts::tool_catalog`]: [`full_input_schema`], [`slim`]med.
pub fn input_schema(contract: &str) -> Option<Value> {
    let mut v = full_input_schema(contract)?;
    slim(contract, &mut v);
    Some(v)
}

/// The contract's own inlined JSON Schema, before slimming.
pub fn full_input_schema(contract: &str) -> Option<Value> {
    Some(match contract {
        "SearchRequest" => inline::<SearchRequest>(),
        "ReadRequest" => inline::<ReadRequest>(),
        "DatasetRef" => inline::<DatasetRef>(),
        "ReleaseRef" => inline::<ReleaseRef>(),
        "ForecastRequest" => inline::<ForecastRequest>(),
        "ArtifactReadRequest" => inline::<ArtifactReadRequest>(),
        "JobLookup" => inline::<JobLookup>(),
        "AnalysisExecuteRequest" => inline::<AnalysisExecuteRequest>(),
        "WikiPatch" => inline::<WikiPatch>(),
        "AnswerSubmission" => inline::<AnswerSubmission>(),
        "OutlineRequest" => inline::<OutlineRequest>(),
        "ConditionsQuery" => inline::<ConditionsQuery>(),
        "GraphNeighborsRequest" => inline::<GraphNeighborsRequest>(),
        "WikiReadRequest" => inline::<WikiReadRequest>(),
        "WebSearchRequest" => inline::<WebSearchRequest>(),
        "WebFetchRequest" => inline::<WebFetchRequest>(),
        "SkillRef" => inline::<SkillRef>(),
        _ => return None,
    })
}

/// Fields of `answer_submit` the host writes and refuses when submitted
/// (`AnswerSubmission::validate`).
const HOST_WRITTEN: &[&str] = &["checked_by", "resolved"];

/// The fields a conditions-cell `locator` is matched on; the rest of a `SourceLocator` never
/// selects a cell.
const LOCATOR_KEYS: &[&str] = &["asset_sha256", "path", "row"];

/// Reduce a contract schema to what the model needs to call the tool correctly (module docs).
pub fn slim(contract: &str, schema: &mut Value) {
    if let Some(o) = schema.as_object_mut() {
        // The tool description already says what the argument is.
        for k in ["$schema", "title", "description"] {
            o.remove(k);
        }
    }
    if contract == "AnswerSubmission" {
        answer_submission(schema);
    }
    node(schema);
}

fn answer_submission(v: &mut Value) {
    match v {
        Value::Object(o) => {
            if let Some(Value::Object(props)) = o.get_mut("properties") {
                for k in HOST_WRITTEN {
                    props.remove(*k);
                }
                if let Some(Value::Object(loc)) = props.get_mut("locator") {
                    if let Some(Value::Object(lp)) = loc.get_mut("properties") {
                        lp.retain(|k, _| LOCATOR_KEYS.contains(&k.as_str()));
                    }
                }
            }
            o.values_mut().for_each(answer_submission);
        }
        Value::Array(a) => a.iter_mut().for_each(answer_submission),
        _ => {}
    }
}

/// Bottom-up: children first, then this node.
fn node(v: &mut Value) {
    match v {
        Value::Array(a) => a.iter_mut().for_each(node),
        Value::Object(o) => {
            for (k, child) in o.iter_mut() {
                match k.as_str() {
                    // Values, not schemas.
                    "enum" | "const" | "required" | "default" => {}
                    // A map of field name to schema: a field may be called `description`.
                    "properties" => {
                        if let Value::Object(props) = child {
                            props.values_mut().for_each(node);
                        }
                    }
                    _ => node(child),
                }
            }
            object(o);
        }
        _ => {}
    }
}

fn object(o: &mut Map<String, Value>) {
    // Unsigned integer formats: the bounds they add say nothing a model needs.
    if let Some(Value::String(f)) = o.remove("format") {
        if f.starts_with("uint") {
            if o.get("minimum").and_then(Value::as_u64) == Some(0) {
                o.remove("minimum");
            }
            o.remove("maximum");
        }
    }
    let empty_default = match o.get("default") {
        Some(Value::Null) => true,
        Some(Value::Array(a)) => a.is_empty(),
        _ => false,
    };
    if empty_default {
        o.remove("default");
    }
    // Opaque ids: non-empty without whitespace is still checked on decode.
    if o.get("pattern").and_then(Value::as_str) == Some("^\\S+$") {
        o.remove("pattern");
        if o.get("minLength").and_then(Value::as_u64) == Some(1) {
            o.remove("minLength");
        }
    }
    for key in ["oneOf", "anyOf"] {
        let Some(Value::Array(variants)) = o.get(key) else {
            continue;
        };
        if let Some(values) = string_variants(variants) {
            let nullable =
                matches!(o.get("type"), Some(Value::Array(t)) if t.iter().any(|x| x == "null"));
            o.remove(key);
            o.insert(
                "type".into(),
                if nullable {
                    json!(["string", "null"])
                } else {
                    json!("string")
                },
            );
            o.insert("enum".into(), Value::Array(values));
        } else if let Some(merged) = merge_tagged(variants) {
            o.remove(key);
            for (k, v) in merged {
                o.entry(k).or_insert(v);
            }
        }
    }
    let structured = ["properties", "oneOf", "anyOf", "enum"]
        .iter()
        .any(|k| o.contains_key(*k));
    if let Some(Value::String(d)) = o.get("description") {
        let mut d = clean(d);
        if structured {
            d = first_sentence(&d).to_string();
        }
        if d.is_empty() {
            o.remove("description");
        } else {
            o.insert("description".into(), Value::String(d));
        }
    }
}

/// A `oneOf` of string constants or enums (with or without per-value docs) as one value list.
fn string_variants(variants: &[Value]) -> Option<Vec<Value>> {
    let mut out = vec![];
    for v in variants {
        let o = v.as_object()?;
        let only = o
            .keys()
            .all(|k| matches!(k.as_str(), "type" | "const" | "enum" | "description"));
        if !only || o.get("type").and_then(Value::as_str) != Some("string") {
            return None;
        }
        match (o.get("const"), o.get("enum")) {
            (Some(c @ Value::String(_)), None) => out.push(c.clone()),
            (None, Some(Value::Array(e))) if e.iter().all(Value::is_string) => {
                out.extend(e.iter().cloned())
            }
            _ => return None,
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Object variants that are the same object except for one string-constant tag property merge
/// into that object with the tag as an enum (`EntityRef`: nine `{kind, id}` variants).
fn merge_tagged(variants: &[Value]) -> Option<Map<String, Value>> {
    if variants.len() < 2 {
        return None;
    }
    let objs: Vec<&Map<String, Value>> = variants
        .iter()
        .map(Value::as_object)
        .collect::<Option<_>>()?;
    let is_tag = |v: &Value| {
        v.as_object().is_some_and(|c| {
            c.len() == 2
                && c.get("type").and_then(Value::as_str) == Some("string")
                && c.get("const").is_some_and(Value::is_string)
        })
    };
    let tag = objs[0]
        .get("properties")?
        .as_object()?
        .iter()
        .find(|(_, v)| is_tag(v))
        .map(|(k, _)| k.clone())?;
    let without = |o: &Map<String, Value>| {
        let mut m = o.clone();
        m.remove("description");
        if let Some(Value::Object(p)) = m.get_mut("properties") {
            p.remove(&tag);
        }
        m
    };
    let base = without(objs[0]);
    let mut values = vec![];
    for o in &objs {
        let t = o.get("properties")?.get(&tag)?;
        if !is_tag(t) || without(o) != base {
            return None;
        }
        values.push(t["const"].clone());
    }
    let mut merged = base;
    if let Some(Value::Object(p)) = merged.get_mut("properties") {
        p.insert(tag, json!({"type": "string", "enum": values}));
    }
    Some(merged)
}

/// Doc-comment noise out of a description: task tags, rustdoc link brackets, hard line breaks.
fn clean(d: &str) -> String {
    let mut s = d.replace('\n', " ").replace("[`", "`").replace("`]", "`");
    for p in ["v2 (M7b): ", "v2: ", "C0. "] {
        if let Some(rest) = s.strip_prefix(p) {
            let mut c = rest.chars();
            s = c
                .next()
                .map(|f| f.to_uppercase().chain(c).collect())
                .unwrap_or_default();
        }
    }
    // ` (K1.3)`, ` (M7b)`, ` (K1.4, co-designed ...)`, ` (master §11.3)`.
    let mut out = String::with_capacity(s.len());
    let mut rest = s.as_str();
    while let Some(i) = rest.find(" (") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 2..];
        let b = after.as_bytes();
        let tag = after.starts_with("master ")
            || (b.len() > 1 && matches!(b[0], b'K' | b'M') && b[1].is_ascii_digit());
        match (tag, after.find(')')) {
            (true, Some(j)) => rest = &after[j + 1..],
            _ => {
                out.push_str(" (");
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Up to the first sentence end (`. ` before an upper-case letter, not after `e.g`/`i.e`).
fn first_sentence(d: &str) -> &str {
    let mut from = 0;
    while let Some(j) = d[from..].find(". ") {
        let at = from + j;
        let abbrev = d[..at].ends_with("e.g") || d[..at].ends_with("i.e");
        if !abbrev && d.as_bytes().get(at + 2).is_some_and(u8::is_ascii_uppercase) {
            return &d[..=at];
        }
        from = at + 2;
    }
    d
}
