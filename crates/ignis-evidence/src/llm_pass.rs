//! Model passes over the corpus (R7b section summaries, R8 labels, R12b expansion vocabulary): the
//! requests, a cost estimate printed before anything runs, and a resumable runner over a pluggable
//! [`Model`]. **Nothing here calls a model unless the caller passes `--execute`**; the spend needs
//! the user's approval (see `docs/implementation/LLM_PASS.md`).
//!
//! * Requests are provider-neutral JSONL ([`Request`]): a static system prefix (instructions,
//!   schema, examples) first, then the document part, so provider prompt caching applies.
//! * [`estimate`] counts tokens as characters / 4 (rounded up) and prices them per million tokens
//!   at the rates the caller passes; it is an estimate, printed with its assumptions.
//! * [`run`] appends one result line per request to an output file and skips `custom_id`s already
//!   there, so an interrupted run resumes without paying twice. Every result carries the model id
//!   and the prompt hash; results are generated text, never evidence.

use std::collections::BTreeSet;
use std::io::{BufRead, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use ignis_core::contracts::Digest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// What a request asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Task {
    SectionSummary,
    SourceLabel,
    ExpansionVocabulary,
    /// R7b + R8 + R12b in one call per chunk.
    Combined,
}

/// One model call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub custom_id: String,
    pub task: Task,
    pub system: String,
    pub user: String,
    pub max_output_tokens: u32,
}

/// One answer, as written by [`run`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Answer {
    pub custom_id: String,
    pub task: Task,
    pub model_id: String,
    pub prompt_hash: Digest,
    pub text: String,
}

/// The digest of a request's static prefix (system prompt): which instructions produced an answer.
pub fn prompt_hash(system: &str) -> Digest {
    Digest::of_bytes(system.as_bytes())
}

/// A model that answers one request. Implementations: [`ClaudeCli`]; tests use a fake.
pub trait Model: Send + Sync {
    fn id(&self) -> &str;
    fn call(&self, system: &str, user: &str, max_output_tokens: u32) -> Result<String>;
}

/// `claude -p` on the logged-in account: no tools, no settings, no MCP, no session persistence,
/// no inherited `CLAUDE*`/`ANTHROPIC*` variables (the same isolation as `ignis data review auto`).
pub struct ClaudeCli {
    pub model: String,
    pub bin: String,
}

impl ClaudeCli {
    pub fn new(model: &str) -> ClaudeCli {
        ClaudeCli {
            model: model.to_string(),
            bin: std::env::var("IGNIS_CLAUDE_BIN").unwrap_or_else(|_| "claude".into()),
        }
    }
}

impl Model for ClaudeCli {
    fn id(&self) -> &str {
        &self.model
    }
    fn call(&self, system: &str, user: &str, _max_output_tokens: u32) -> Result<String> {
        let mut cmd = Command::new(&self.bin);
        cmd.args([
            "-p",
            "--output-format",
            "json",
            "--model",
            &self.model,
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
            .with_context(|| format!("cannot start {}", self.bin))?;
        let mut stdin = child.stdin.take().unwrap();
        let input = user.as_bytes().to_vec();
        let writer = std::thread::spawn(move || stdin.write_all(&input));
        let mut out = Vec::new();
        child.stdout.take().unwrap().read_to_end(&mut out)?;
        let mut err = Vec::new();
        child.stderr.take().unwrap().read_to_end(&mut err)?;
        let status = child.wait()?;
        let _ = writer.join();
        if !status.success() {
            bail!(
                "{} exited with {status}: {}",
                self.bin,
                String::from_utf8_lossy(&err)
            );
        }
        let v: Value = serde_json::from_slice(&out).context("claude -p output is not JSON")?;
        if v["is_error"].as_bool() == Some(true) {
            bail!("model error: {}", v["result"]);
        }
        Ok(v["result"]
            .as_str()
            .context("claude -p output has no result")?
            .to_string())
    }
}

/// What a pass would cost.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Estimate {
    pub requests: usize,
    pub input_tokens: u64,
    /// Of which the repeated static prefix (cacheable).
    pub prefix_tokens: u64,
    pub output_tokens: u64,
    pub usd: f64,
    pub assumptions: String,
}

fn tokens(text: &str) -> u64 {
    (text.chars().count() as u64).div_ceil(4)
}

/// Tokens as characters / 4; every request's output at its `max_output_tokens` (an upper bound);
/// the static prefix priced in full (caching would lower it).
pub fn estimate(reqs: &[Request], usd_per_mtok_in: f64, usd_per_mtok_out: f64) -> Estimate {
    let input: u64 = reqs
        .iter()
        .map(|r| tokens(&r.system) + tokens(&r.user))
        .sum();
    let prefix: u64 = reqs.iter().map(|r| tokens(&r.system)).sum();
    let output: u64 = reqs.iter().map(|r| r.max_output_tokens as u64).sum();
    Estimate {
        requests: reqs.len(),
        input_tokens: input,
        prefix_tokens: prefix,
        output_tokens: output,
        usd: input as f64 / 1e6 * usd_per_mtok_in + output as f64 / 1e6 * usd_per_mtok_out,
        assumptions: format!(
            "tokens = chars/4; output at max_output_tokens (upper bound); ${usd_per_mtok_in}/Mtok in, ${usd_per_mtok_out}/Mtok out; no prompt-cache discount"
        ),
    }
}

/// Writes the requests as JSONL (for review, or for a provider's batch API).
pub fn write_requests(path: &Path, reqs: &[Request]) -> Result<()> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    for r in reqs {
        serde_json::to_writer(&mut f, r)?;
        f.write_all(b"\n")?;
    }
    f.flush()?;
    Ok(())
}

/// Answers already in an output file.
pub fn read_answers(path: &Path) -> Result<Vec<Answer>> {
    let Ok(f) = std::fs::File::open(path) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (i, line) in std::io::BufReader::new(f).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(
            serde_json::from_str(&line)
                .with_context(|| format!("{} line {}", path.display(), i + 1))?,
        );
    }
    Ok(out)
}

/// What a run did.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunReport {
    pub answered: usize,
    pub skipped_done: usize,
    pub failed: Vec<(String, String)>,
}

/// Answers every request not yet in `out` (appending), one at a time; a failure is recorded and the
/// run continues. Stops after `max_failures` consecutive failures (an outage, a quota).
pub fn run(
    reqs: &[Request],
    model: &dyn Model,
    out: &Path,
    max_failures: usize,
) -> Result<RunReport> {
    let done: BTreeSet<String> = read_answers(out)?
        .into_iter()
        .map(|a| a.custom_id)
        .collect();
    if let Some(d) = out.parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out)?;
    let mut report = RunReport {
        answered: 0,
        skipped_done: 0,
        failed: Vec::new(),
    };
    let mut streak = 0;
    for r in reqs {
        if done.contains(&r.custom_id) {
            report.skipped_done += 1;
            continue;
        }
        match model.call(&r.system, &r.user, r.max_output_tokens) {
            Ok(text) => {
                let a = Answer {
                    custom_id: r.custom_id.clone(),
                    task: r.task,
                    model_id: model.id().to_string(),
                    prompt_hash: prompt_hash(&r.system),
                    text,
                };
                serde_json::to_writer(&mut f, &a)?;
                f.write_all(b"\n")?;
                f.flush()?;
                report.answered += 1;
                streak = 0;
            }
            Err(e) => {
                report.failed.push((r.custom_id.clone(), format!("{e:#}")));
                streak += 1;
                if streak >= max_failures {
                    bail!("{streak} consecutive failures; last: {e:#}");
                }
            }
        }
    }
    Ok(report)
}
