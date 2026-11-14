//! Content addressing. Every artifact identity is a hash of canonical JSON, never of prose.
//! Rust is the only producer of identity hashes (`input_hash`, release/artifact/cohort/split ids).

use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Canonical JSON with recursively sorted keys (independent of serde_json's map feature flags).
pub fn canonical_json(v: &Value) -> String {
    let mut out = String::new();
    write_canonical(v, &mut out);
    out
}

fn write_canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String((*k).clone()).to_string());
                out.push(':');
                write_canonical(&m[*k], out);
            }
            out.push('}');
        }
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(x, out);
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// `sha256:<hex>` of the canonical JSON of `v`.
pub fn key_of(v: &Value) -> String {
    format!("sha256:{}", sha256_hex(canonical_json(v).as_bytes()))
}

/// Strip an optional `sha256:` prefix.
pub fn hex_part(id: &str) -> &str {
    id.split_once(':').map(|(_, h)| h).unwrap_or(id)
}

/// True for a 64-char lowercase hex digest (optionally `sha256:`-prefixed). Used to keep ids that
/// become file names inside their directory.
pub fn is_digest(id: &str) -> bool {
    let h = hex_part(id);
    h.len() == 64
        && h.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// `sha256:<hex>` of the canonical JSON of `obj` with the given top-level keys removed.
/// Non-finite floats serialize as JSON null; callers must record missingness explicitly instead.
pub fn content_hash<T: serde::Serialize + ?Sized>(obj: &T, exclude: &[&str]) -> String {
    let mut v = serde_json::to_value(obj).expect("contract types serialize");
    if let Value::Object(m) = &mut v {
        for k in exclude {
            m.remove(*k);
        }
    }
    key_of(&v)
}

/// SHA-256 of a file's bytes (streamed).
pub fn file_sha256(path: &std::path::Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

/// A copy of `v` with every object's keys sorted (stable pretty output).
pub fn sorted(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            Value::Object(
                keys.into_iter()
                    .map(|k| (k.clone(), sorted(&m[k])))
                    .collect(),
            )
        }
        Value::Array(a) => Value::Array(a.iter().map(sorted).collect()),
        o => o.clone(),
    }
}
