//! Resource profiles, the opt-in execution guard and the job/cost ledger.
//!
//! Nothing expensive runs by default. A command that would train, fit, decode video or download in
//! bulk calls [`guard`]: without `--execute` it prints the plan and the estimated cost and stops.
//! Executed jobs are wrapped in [`Job`], which records wall time, peak memory and estimated cost
//! under `runs/jobs/` (estimated and billed cost are separate fields).

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub name: String,
    /// "cpu" | "gpu"
    pub device: String,
    #[serde(default)]
    pub usd_per_hour: f64,
    /// Hard limit for one job; 0 = unlimited (local only).
    #[serde(default)]
    pub max_hours: f64,
    /// Worker threads for numeric pools; 0 = all logical cores (of the machine running the command:
    /// set it for remote pods, or a plan made here assumes this machine's core count).
    #[serde(default)]
    pub cpu_threads: usize,
    /// Multiplies configured batch sizes (memory knob).
    #[serde(default = "one")]
    pub batch_scale: f64,
    /// Fraction of device memory a job may use (GPU profiles).
    #[serde(default = "half")]
    pub mem_fraction: f64,
    #[serde(default)]
    pub vram_gb: Option<f64>,
}

fn one() -> f64 {
    1.0
}
fn half() -> f64 {
    0.5
}

pub fn configs_dir() -> PathBuf {
    std::env::var_os("IGNIS_CONFIGS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("configs"))
}

pub fn runs_dir() -> PathBuf {
    std::env::var_os("IGNIS_RUNS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("runs"))
}

pub fn load_profile(name: &str) -> anyhow::Result<Profile> {
    let p = configs_dir().join("resources").join(format!("{name}.toml"));
    let text =
        std::fs::read_to_string(&p).with_context(|| format!("resource profile {}", p.display()))?;
    let mut prof: Profile = toml::from_str(&text)?;
    prof.name = name.to_string();
    Ok(prof)
}

impl Profile {
    pub fn threads(&self) -> usize {
        if self.cpu_threads == 0 {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        } else {
            self.cpu_threads
        }
    }
    /// A bounded Rayon pool for this profile (never the global pool: no oversubscription).
    pub fn pool(&self) -> rayon::ThreadPool {
        crate::batch::bounded_pool(self.threads())
    }
}

/// Outcome of [`guard`]: run, or stop with the plan already printed.
#[derive(Debug, PartialEq)]
pub enum Guard {
    Execute,
    DryRun,
}

/// Print the plan; refuse estimates above the profile limit; run only with `execute`.
pub fn guard(
    action: &str,
    profile: &Profile,
    estimated_hours: f64,
    execute: bool,
    detail: Value,
) -> anyhow::Result<Guard> {
    let mut plan = json!({
        "action": action, "profile": profile.name, "device": profile.device,
        "estimated_hours": estimated_hours, "estimated_usd": (estimated_hours * profile.usd_per_hour * 1e4).round() / 1e4,
    });
    if let (Value::Object(p), Value::Object(d)) = (&mut plan, detail) {
        p.extend(d);
    }
    if profile.max_hours > 0.0 && estimated_hours > profile.max_hours {
        plan["refused"] = json!(format!(
            "estimate exceeds profile max_hours={}",
            profile.max_hours
        ));
        println!("{}", serde_json::to_string_pretty(&plan)?);
        anyhow::bail!("refused by resource guard");
    }
    if !execute {
        plan["status"] = json!("dry-run; pass --execute to run");
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(Guard::DryRun);
    }
    Ok(Guard::Execute)
}

/// Ledger entry for an executed job. Call [`Job::finish`]; dropping unfinished records `failed`.
pub struct Job {
    path: PathBuf,
    record: Value,
    started: Instant,
    usd_per_hour: f64,
    /// metered API spend (estimated before the calls, billed as reported by the provider)
    api_usd: Option<(f64, f64)>,
    done: bool,
}

impl Job {
    pub fn start(action: &str, profile: &Profile, config: Value) -> anyhow::Result<Job> {
        let dir = runs_dir().join("jobs");
        std::fs::create_dir_all(&dir)?;
        let now = chrono::Utc::now();
        let id = format!("job-{}-{:06x}", now.format("%Y%m%dT%H%M%SZ"), rand_suffix());
        let record = json!({
            "id": id, "action": action, "profile": profile.name, "device": profile.device,
            "host": hostname(), "config": config, "started_at": now.to_rfc3339(), "checkpoints": [],
            "usd_per_hour": profile.usd_per_hour, "billed_usd": null,
        });
        let path = dir.join(format!("{id}.json"));
        std::fs::write(&path, serde_json::to_vec_pretty(&record)?)?;
        Ok(Job {
            path,
            record,
            started: Instant::now(),
            usd_per_hour: profile.usd_per_hour,
            api_usd: None,
            done: false,
        })
    }
    pub fn id(&self) -> &str {
        self.record["id"].as_str().unwrap_or_default()
    }
    pub fn checkpoint(&mut self, path: &Path) {
        if let Some(a) = self.record["checkpoints"].as_array_mut() {
            a.push(json!(path.display().to_string()));
        }
        let _ = std::fs::write(
            &self.path,
            serde_json::to_vec_pretty(&self.record).unwrap_or_default(),
        );
    }
    /// Record metered API cost: `estimated_usd` adds to the compute estimate, `billed_usd` is what the
    /// provider reported (kept separate, as everywhere in the ledger).
    pub fn set_api_cost(&mut self, estimated_usd: f64, billed_usd: f64) {
        self.api_usd = Some((estimated_usd, billed_usd));
    }
    fn close(&mut self, state: &str, extra: Value) {
        let hours = self.started.elapsed().as_secs_f64() / 3600.0;
        self.record["ended_at"] = json!(chrono::Utc::now().to_rfc3339());
        self.record["exit_state"] = json!(state);
        self.record["wall_hours"] = json!(hours);
        let (api_est, api_billed) = self.api_usd.unzip();
        self.record["estimated_usd"] =
            json!(((hours * self.usd_per_hour + api_est.unwrap_or(0.0)) * 1e8).round() / 1e8);
        if let Some(b) = api_billed {
            self.record["api_estimated_usd"] = json!(api_est);
            self.record["billed_usd"] = json!(b);
        }
        self.record["peak_rss_mb"] = json!(peak_rss_mb());
        self.record["result"] = extra;
        let _ = std::fs::write(
            &self.path,
            serde_json::to_vec_pretty(&self.record).unwrap_or_default(),
        );
        self.done = true;
    }
    pub fn finish(mut self, result: Value) {
        self.close("completed", result);
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        if !self.done {
            self.close("failed", Value::Null);
        }
    }
}

fn rand_suffix() -> u32 {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (n ^ std::process::id()) & 0xff_ffff
}

fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

/// Peak resident memory of this process in MiB, when the platform exposes it.
pub fn peak_rss_mb() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        let s = std::fs::read_to_string("/proc/self/status").ok()?;
        let line = s.lines().find(|l| l.starts_with("VmHWM:"))?;
        let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
        return Some(kb / 1024.0);
    }
    #[allow(unreachable_code)]
    None
}
