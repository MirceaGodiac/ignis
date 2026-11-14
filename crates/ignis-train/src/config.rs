//! Model configs: the expert's `default_config()` (built-in defaults + `configs/models/...`) merged
//! over `configs/training/defaults.toml` (model wins), then `--set a.b=value` overrides.

use anyhow::Context;
use serde_json::Value;

use ignis_core::expert::Expert;
use ignis_core::resources::configs_dir;

pub fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for (k, v) in y {
                match x.get_mut(k) {
                    Some(slot) if slot.is_object() && v.is_object() => merge(slot, v),
                    _ => {
                        x.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

pub fn toml_file(path: &std::path::Path) -> anyhow::Result<Value> {
    let text = std::fs::read_to_string(path).with_context(|| format!("{}", path.display()))?;
    let t: toml::Value = toml::from_str(&text).with_context(|| format!("{}", path.display()))?;
    Ok(serde_json::to_value(t)?)
}

pub fn training_defaults() -> Value {
    toml_file(&configs_dir().join("training").join("defaults.toml"))
        .unwrap_or(Value::Object(Default::default()))
}

/// `a.b.c=value`, value parsed as a TOML scalar/array (strings may be bare).
pub fn apply_overrides(cfg: &mut Value, sets: &[String]) -> anyhow::Result<()> {
    for s in sets {
        let (key, raw) = s
            .split_once('=')
            .with_context(|| format!("--set {s:?}: expected key=value"))?;
        let val = toml::from_str::<toml::Value>(&format!("v = {raw}"))
            .ok()
            .and_then(|t| t.get("v").cloned())
            .map(|v| serde_json::to_value(v).unwrap())
            .unwrap_or(Value::String(raw.to_string()));
        let mut d = &mut *cfg;
        let parts: Vec<&str> = key.split('.').collect();
        for p in &parts[..parts.len() - 1] {
            if !d.get(*p).is_some_and(|v| v.is_object()) {
                d[*p] = Value::Object(Default::default());
            }
            d = d.get_mut(*p).unwrap();
        }
        d[parts[parts.len() - 1]] = val;
    }
    Ok(())
}

pub fn load_config(expert: &dyn Expert, sets: &[String]) -> anyhow::Result<Value> {
    let mut cfg = training_defaults();
    merge(&mut cfg, &expert.default_config()?);
    apply_overrides(&mut cfg, sets)?;
    Ok(cfg)
}
