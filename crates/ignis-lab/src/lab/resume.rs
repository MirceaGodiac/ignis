//! Hand a release back to the training CLI (`ignis fit run ... --resume <release_id>`).
//!
//! The lab does not train. Without `execute` the fit command runs in its own dry-run mode (cost plan
//! from its resource guard); with `execute`, `--execute` is forwarded. The resulting release is written
//! by the training lane with `lineage=fine_tune` and `parent_id` = this release.

use serde_json::{json, Value};

use super::inspect::load;
use crate::access::LabSession;

pub fn build_command(
    exe: &str,
    release_id: &str,
    profile: &str,
    extra: &[String],
    execute: bool,
) -> Vec<String> {
    let mut cmd = vec![exe.to_string(), "fit".into(), "run".into()];
    cmd.extend(extra.iter().cloned());
    cmd.extend([
        "--resume".into(),
        release_id.to_string(),
        "--profile".into(),
        profile.to_string(),
    ]);
    if execute {
        cmd.push("--execute".into());
    }
    cmd
}

fn tail(s: &str, n: usize) -> String {
    let c: Vec<char> = s.chars().collect();
    c[c.len().saturating_sub(n)..].iter().collect()
}

pub fn resume(
    release_id: &str,
    profile: &str,
    extra: &[String],
    execute: bool,
) -> anyhow::Result<Value> {
    let _lab = LabSession::enter();
    let (r, _, _) = load(release_id)?;
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "ignis".into());
    let cmd = build_command(&exe, &r.id, profile, extra, execute);
    let mut notes = vec![];
    if !r.posterior_valid_for_parameters {
        notes.push(
            "posterior draws of this release are invalid for its parameters; resumed fit must produce new draws before any posterior claim",
        );
    }
    let mut out = json!({"release": r.id, "model_name": r.model_name, "profile": profile, "execute": execute,
                         "command": cmd, "notes": notes});
    let p = std::process::Command::new(&cmd[0]).args(&cmd[1..]).output();
    match p {
        Err(e) => {
            out["status"] = json!("unavailable");
            out["reason"] = json!(format!("cannot start {}: {e}", cmd[0]));
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            if !o.status.success()
                && stderr.contains("unrecognized subcommand")
                && stderr.contains("'fit'")
            {
                out["status"] = json!("unavailable");
                out["reason"] = json!("`ignis fit` is not present in this binary");
            } else {
                out["status"] = json!(if o.status.success() {
                    "completed"
                } else {
                    "failed"
                });
            }
            out["returncode"] = json!(o.status.code());
            out["stdout"] = json!(tail(&String::from_utf8_lossy(&o.stdout), 20000));
            out["stderr"] = json!(tail(&stderr, 8000));
        }
    }
    Ok(out)
}
