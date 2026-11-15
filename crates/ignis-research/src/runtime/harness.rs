//! The harness bundle (task M6): the operational instructions, skills and tool section an episode's
//! runtime is started with, rendered once into one instruction prefix and identified by its digest.
//!
//! - Sources live in `harness/` at the repository root (`investigator.md`,
//!   `skills/<name>/SKILL.md` in the agentskills.io format). The default bundle is compiled into the
//!   binary, so the deployed bundle is exactly the one of the build; `IGNIS_HARNESS_DIR` (read by
//!   the host) loads a variant from a directory instead.
//! - The tool section is generated from [`tool_catalog`], restricted to the tools the runtime is
//!   actually attached to, so descriptions cannot drift from the contract.
//! - Skills are inlined, not installed for discovery: both runtimes run without a shell, file tools
//!   or the Skill tool (`--tools ""`), so a skill on disk could be listed but never read.
//! - The prefix is byte-stable for prompt caching: no timestamps, episode ids or user data, line
//!   endings normalized to `\n`, tools in name order, skills in directory-name order. The digest is
//!   the sha256 of exactly the bytes injected.
//!
//! **Prefix and tail.** The bundle is the whole cached prefix and nothing else: the same bytes for
//! every episode, scope and turn, so the runtime's prompt cache serves it (caching only the stable
//! system prompt is the most consistent cost/latency win; Lumer et al., arXiv 2601.06007).
//! Anything episode-specific (the user's messages today; later a scope summary, datasets in view,
//! working memory — M9/M10) belongs in the turn input, after the prefix, never in the bundle. The
//! two places that build that tail are `CodexBridge` (`turn/start` input) and `ClaudeBridge`
//! (the prompt on stdin).
//!
//! Codex receives the prefix as `developerInstructions` on `thread/start` and `thread/resume`
//! (Codex keeps its own base instructions); Claude Code receives it with
//! `--append-system-prompt-file` (its default system prompt stays, its prompt snapshot keeps the
//! session's first system prompt verbatim on resume).

use std::path::Path;
use std::sync::Arc;

use ignis_core::contracts::{
    tool_catalog, Digest, EpisodeId, ExecutionScope, HarnessRef, RuntimeBuildId,
};
use ignis_core::services::ServiceResult;

use crate::store::HarnessPinChange;
use crate::ResearchStore;

/// Label of the bundle compiled from `harness/`.
pub const EMBEDDED_LABEL: &str = "h1";

/// Model name recorded when IGNIS did not choose one and the runtime has not reported it.
pub const RUNTIME_DEFAULT_MODEL: &str = "runtime-default";

const EMBEDDED_INVESTIGATOR: &str = include_str!("../../../../harness/investigator.md");
const EMBEDDED_SKILLS: &[(&str, &str)] = &[
    (
        "model-qualification",
        include_str!("../../../../harness/skills/model-qualification/SKILL.md"),
    ),
    (
        "quantitative-analysis",
        include_str!("../../../../harness/skills/quantitative-analysis/SKILL.md"),
    ),
    (
        "source-grounding",
        include_str!("../../../../harness/skills/source-grounding/SKILL.md"),
    ),
];

/// The unrendered contents of a harness directory.
#[derive(Debug, Clone)]
pub struct HarnessSource {
    pub investigator: String,
    /// `(directory name, SKILL.md contents)`; order does not matter.
    pub skills: Vec<(String, String)>,
}

impl HarnessSource {
    /// The bundle compiled into this binary.
    pub fn embedded() -> Self {
        HarnessSource {
            investigator: EMBEDDED_INVESTIGATOR.to_string(),
            skills: EMBEDDED_SKILLS
                .iter()
                .map(|(n, s)| (n.to_string(), s.to_string()))
                .collect(),
        }
    }

    /// `<dir>/investigator.md` and every `<dir>/skills/<name>/SKILL.md`.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let read = |p: &Path| {
            std::fs::read_to_string(p).map_err(|e| format!("harness {}: {e}", p.display()))
        };
        let investigator = read(&dir.join("investigator.md"))?;
        let mut skills = vec![];
        let sdir = dir.join("skills");
        if sdir.is_dir() {
            for entry in std::fs::read_dir(&sdir).map_err(|e| format!("harness skills: {e}"))? {
                let entry = entry.map_err(|e| format!("harness skills: {e}"))?;
                let file = entry.path().join("SKILL.md");
                if file.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    skills.push((name, read(&file)?));
                }
            }
        }
        Ok(HarnessSource {
            investigator,
            skills,
        })
    }
}

/// A rendered harness bundle: the instruction prefix and its digest. Cheap to clone.
#[derive(Clone, PartialEq, Eq)]
pub struct HarnessBundle {
    label: String,
    text: Arc<str>,
    digest: Digest,
}

impl std::fmt::Debug for HarnessBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HarnessBundle")
            .field("label", &self.label)
            .field("digest", &self.digest)
            .field("bytes", &self.text.len())
            .finish()
    }
}

fn lf(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// `---\nkey: value\n---\nbody` → (`name`, `description`, body). Fails closed on a malformed skill.
fn parse_skill(dir_name: &str, text: &str) -> Result<(String, String, String), String> {
    let bad = |why: &str| format!("skill {dir_name}: {why}");
    let rest = text
        .strip_prefix("---\n")
        .ok_or_else(|| bad("no frontmatter"))?;
    let (front, body) = rest
        .split_once("\n---\n")
        .ok_or_else(|| bad("unterminated frontmatter"))?;
    let field = |k: &str| {
        front
            .lines()
            .find_map(|l| l.strip_prefix(k)?.strip_prefix(':'))
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    let name = field("name").ok_or_else(|| bad("no name"))?;
    if name != dir_name {
        return Err(bad(&format!("name {name:?} differs from its directory")));
    }
    let description = field("description").ok_or_else(|| bad("no description"))?;
    Ok((name, description, body.trim().to_string()))
}

/// The tool section: one entry per catalog tool the runtime is attached to, in name order.
fn tool_section(tools: &[String]) -> String {
    let mut catalog: Vec<_> = tool_catalog()
        .into_iter()
        .filter(|t| tools.iter().any(|n| n == &t.name))
        .collect();
    catalog.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out = String::from(
        "## IGNIS tools\n\nGenerated from the IGNIS tool catalog; these are the IGNIS tools attached to this runtime.\n",
    );
    if catalog.is_empty() {
        out.push_str(
            "\nNo IGNIS tools are attached in this deployment: you cannot retrieve evidence, compute or submit a bound answer. Say so instead of answering from memory.\n",
        );
        return out;
    }
    for t in catalog {
        let effect = serde_json::to_value(t.effect)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        let mut traits = vec![format!("effect `{effect}`")];
        if t.idempotent {
            traits.push("idempotent".into());
        }
        if t.may_return_job {
            traits.push("may return a job".into());
        }
        out.push_str(&format!(
            "\n- `{}` ({}): {} Arguments: `{}`; result: `{}`.",
            t.name,
            traits.join(", "),
            t.summary,
            t.input_contract,
            t.output_contract
        ));
    }
    out.push('\n');
    out
}

impl HarnessBundle {
    /// Render `source` for a runtime attached to `tools` (wire names; empty: no IGNIS tools).
    pub fn render(label: &str, source: &HarnessSource, tools: &[String]) -> Result<Self, String> {
        if label.trim().is_empty() {
            return Err("harness label is empty".into());
        }
        let investigator = lf(&source.investigator).trim().to_string();
        if investigator.is_empty() {
            return Err("harness investigator.md is empty".into());
        }
        let mut skills = source
            .skills
            .iter()
            .map(|(n, s)| parse_skill(n, &lf(s)))
            .collect::<Result<Vec<_>, _>>()?;
        skills.sort_by(|a, b| a.0.cmp(&b.0));
        let mut text = investigator;
        text.push_str("\n\n");
        text.push_str(&tool_section(tools));
        if !skills.is_empty() {
            text.push_str(
                "\n## Skills\n\nApply a skill when its description matches the question.\n",
            );
            for (name, description, body) in &skills {
                text.push_str(&format!(
                    "\n<skill name=\"{name}\">\n{description}\n\n{body}\n</skill>\n"
                ));
            }
        }
        let digest = Digest::of_bytes(text.as_bytes());
        Ok(HarnessBundle {
            label: label.to_string(),
            text: text.into(),
            digest,
        })
    }

    /// The bundle compiled into this binary, for a runtime attached to `tools`.
    pub fn embedded(tools: &[String]) -> Self {
        Self::render(EMBEDDED_LABEL, &HarnessSource::embedded(), tools)
            .expect("the embedded harness is well-formed (checked by the tests)")
    }

    /// The instruction prefix, exactly as injected: the cached part of the context. Episode data
    /// never goes here (see the module docs on prefix and tail).
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    /// The episode pin for this bundle injected into `build` running `model`.
    pub fn pin(
        &self,
        build: &RuntimeBuildId,
        model: Option<&str>,
        effort: Option<&str>,
    ) -> HarnessRef {
        HarnessRef {
            digest: self.digest.clone(),
            label: self.label.clone(),
            runtime_build: build.clone(),
            model: model
                .filter(|m| !m.trim().is_empty())
                .unwrap_or(RUNTIME_DEFAULT_MODEL)
                .to_string(),
            effort: effort.filter(|e| !e.trim().is_empty()).map(str::to_string),
        }
    }

    /// Short form for the event log: `h1 sha256:0123456789ab`.
    pub fn short(&self) -> String {
        short(&self.label, &self.digest)
    }
}

pub(crate) fn short(label: &str, digest: &Digest) -> String {
    format!("{label} sha256:{}", &digest.hex()[..12])
}

/// Pin `bundle` on the episode as injected into `build`, and return the note for the log when the
/// pin is new or the bundle changed. `model`/`effort` absent keep what an earlier pin of the same
/// bundle recorded (the runtime may report them later). `reapplies`: whether the runtime applies
/// new instructions to a conversation it already has; when it does not, a different bundle is not
/// pinned and the note says the episode stays on its pin.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pin_episode(
    store: &ResearchStore,
    scope: &ExecutionScope,
    ep: &EpisodeId,
    bundle: &HarnessBundle,
    build: &RuntimeBuildId,
    model: Option<&str>,
    effort: Option<&str>,
    reapplies: bool,
) -> ServiceResult<Option<String>> {
    let current = store.episode(scope, ep)?.pins.harness;
    let same = current.as_ref().filter(|h| h.digest == bundle.digest);
    if same.is_none() && !reapplies {
        let was = current
            .as_ref()
            .map_or("no harness".to_string(), |h| short(&h.label, &h.digest));
        return Ok(Some(format!(
            "this host runs harness {}, but the episode stays on {was}: the runtime keeps the instructions its conversation started with",
            bundle.short()
        )));
    }
    let model = model
        .filter(|m| !m.trim().is_empty())
        .map(str::to_string)
        .or_else(|| same.map(|h| h.model.clone()));
    let effort = effort
        .filter(|e| !e.trim().is_empty())
        .map(str::to_string)
        .or_else(|| same.and_then(|h| h.effort.clone()));
    let pin = bundle.pin(build, model.as_deref(), effort.as_deref());
    Ok(match store.pin_harness(scope, ep, pin)? {
        HarnessPinChange::Pinned => Some(format!("harness {} injected", bundle.short())),
        HarnessPinChange::Replaced(old) if old.digest != bundle.digest => Some(format!(
            "harness switched from {} to {}",
            short(&old.label, &old.digest),
            bundle.short()
        )),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tools() -> Vec<String> {
        vec![
            "sources_search".into(),
            "answer_submit".into(),
            "analysis_execute".into(),
        ]
    }

    /// The prefix is a pure function of the sources and the attached tool set: rendering it again,
    /// from CRLF checkouts, in another skill or tool order, gives the same bytes and digest, and
    /// the digest is the sha256 of those bytes. Any content change moves the digest.
    #[test]
    fn the_prefix_is_byte_stable_and_its_digest_is_its_content() {
        let src = HarnessSource::embedded();
        let a = HarnessBundle::render("h1", &src, &tools()).unwrap();
        let mut crlf = src.clone();
        crlf.investigator = crlf.investigator.replace('\n', "\r\n");
        crlf.skills.reverse();
        for s in &mut crlf.skills {
            s.1 = s.1.replace('\n', "\r\n");
        }
        let mut reordered = tools();
        reordered.reverse();
        let b = HarnessBundle::render("h1", &crlf, &reordered).unwrap();
        assert_eq!(a.text(), b.text());
        assert_eq!(a.digest(), b.digest());
        assert_eq!(a.digest(), &Digest::of_bytes(a.text().as_bytes()));
        assert!(!a.text().contains('\r'));

        // The tool section comes from the catalog, only for attached tools, in name order.
        let t = a.text();
        let pos = |n: &str| t.find(&format!("- `{n}`")).unwrap();
        assert!(pos("analysis_execute") < pos("answer_submit"));
        assert!(pos("answer_submit") < pos("sources_search"));
        assert!(!t.contains("`wiki_patch`"));
        let catalog = tool_catalog();
        let search = catalog.iter().find(|d| d.name == "sources_search").unwrap();
        assert!(t.contains(&search.summary));
        assert!(t.contains("<skill name=\"source-grounding\">"));
        assert!(t.contains("<skill name=\"quantitative-analysis\">"));

        let mut edited = src.clone();
        edited.investigator.push_str("\nOne more rule.\n");
        let c = HarnessBundle::render("h1", &edited, &tools()).unwrap();
        assert_ne!(a.digest(), c.digest());
        let fewer = HarnessBundle::render("h1", &src, &tools()[..1]).unwrap();
        assert_ne!(a.digest(), fewer.digest());
    }

    #[test]
    fn a_malformed_skill_is_refused() {
        let mut src = HarnessSource::embedded();
        src.skills.push((
            "other-name".into(),
            "---\nname: x\ndescription: d\n---\nbody".into(),
        ));
        let e = HarnessBundle::render("h1", &src, &[]).unwrap_err();
        assert!(e.contains("differs from its directory"), "{e}");
        src.skills.pop();
        src.skills.push(("x".into(), "no frontmatter".into()));
        assert!(HarnessBundle::render("h1", &src, &[]).is_err());
    }
}
