//! The harness bundle (task M6): the operational instructions, skills and tool section an episode's
//! runtime is started with, rendered once into one instruction prefix and identified by its digest.
//!
//! - Sources live in `harness/` at the repository root (`investigator.md`,
//!   `skills/<name>/SKILL.md` in the agentskills.io format). The default bundle is compiled into the
//!   binary, so the deployed bundle is exactly the one of the build; `IGNIS_HARNESS_DIR` (read by
//!   the host) loads a variant from a directory instead.
//! - The tool section is generated from the descriptors the tool host serves (the catalog with the
//!   deployment's notes, e.g. the served model releases; [`HarnessShelf::tools_for`]), restricted
//!   to the tools the runtime is actually attached to, so descriptions cannot drift from what the
//!   MCP server lists.
//! - Skills load on demand. Both runtimes run without a shell, file tools or the Skill tool
//!   (`--tools ""`), so a skill on disk could be listed but never read; instead the prefix carries
//!   a one-line skill index and the IGNIS tool `skills_read` serves a skill's full text from the
//!   bundle the episode is pinned to (the host keeps its rendered bundles on a [`HarnessShelf`]).
//!   Every model request re-sends the prefix, so a skill the question does not need costs nothing.
//!   A runtime not attached to `skills_read` gets the skills inlined instead, as before.
//! - The prefix is byte-stable for prompt caching: no timestamps, episode ids or user data, line
//!   endings normalized to `\n`, tools in name order, skills in directory-name order.
//! - The digest identifies everything the agent can be given: the sha256 of the prefix bytes, followed,
//!   when skills are served on demand, by the served skill texts (each as a `<skill>` block, in
//!   name order). Without on-demand skills it is the sha256 of exactly the bytes injected.
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
//! (its coding-agent base instructions are replaced by `runtime/codex_base.md` through
//! `model_instructions_file`, see `preflight.rs`); Claude Code receives it with
//! `--append-system-prompt-file` (its default system prompt stays, its prompt snapshot keeps the
//! session's first system prompt verbatim on resume).

use std::path::Path;
use std::sync::{Arc, RwLock};

use ignis_core::contracts::{
    tool_catalog, Digest, EpisodeId, ExecutionScope, HarnessRef, RuntimeBuildId, SkillText,
    ToolDescriptor,
};
use ignis_core::services::ServiceResult;

use crate::store::HarnessPinChange;
use crate::ResearchStore;

/// Label of the bundle compiled from `harness/`.
pub const EMBEDDED_LABEL: &str = "h1";

/// Model name recorded when IGNIS did not choose one and the runtime has not reported it.
pub const RUNTIME_DEFAULT_MODEL: &str = "runtime-default";

/// The IGNIS tool that serves a skill's full text; when the runtime is attached to it, the prefix
/// carries only the skill index.
pub const SKILLS_TOOL: &str = "skills_read";

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

/// One skill of a bundle, parsed from its SKILL.md (line endings `\n`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub name: String,
    pub description: String,
    /// The text after the frontmatter, trimmed.
    pub body: String,
}

impl Skill {
    fn block(&self) -> String {
        format!(
            "\n<skill name=\"{}\">\n{}\n\n{}\n</skill>\n",
            self.name, self.description, self.body
        )
    }
}

/// A rendered harness bundle: the instruction prefix, the skills it serves on demand and its
/// digest. Cheap to clone.
#[derive(Clone, PartialEq, Eq)]
pub struct HarnessBundle {
    label: String,
    text: Arc<str>,
    skills: Arc<[Skill]>,
    digest: Digest,
}

impl std::fmt::Debug for HarnessBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HarnessBundle")
            .field("label", &self.label)
            .field("digest", &self.digest)
            .field("bytes", &self.text.len())
            .field("skills", &self.skills.len())
            .finish()
    }
}

fn lf(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// `---\nkey: value\n---\nbody` → (`name`, `description`, body). Fails closed on a malformed skill.
fn parse_skill(dir_name: &str, text: &str) -> Result<Skill, String> {
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
    let body = body.trim().to_string();
    if body.is_empty() {
        return Err(bad("empty body"));
    }
    Ok(Skill {
        name,
        description,
        body,
    })
}

/// How the prefix's tool section describes each attached tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolDescriptions {
    /// Each tool's full description and contract names: for a runtime that defers MCP tool
    /// definitions. Codex 0.169 always does (the model sees only the `mcp__ignis` namespace until
    /// it looks a tool up from `exec`), so the prefix is where its model learns the tools.
    #[default]
    Full,
    /// One line per tool (name and traits): for a runtime that sends every MCP tool definition,
    /// description and input schema with each request (Claude Code 2.1), where a full description
    /// here would be sent twice.
    Index,
}

/// The catalog descriptors of the tools named `tools` (wire names; unknown names are left out).
fn catalog_of(tools: &[String]) -> Vec<ToolDescriptor> {
    tool_catalog()
        .into_iter()
        .filter(|t| tools.iter().any(|n| n == &t.name))
        .collect()
}

/// The tool section: one entry per tool the runtime is attached to, in name order, with the
/// description the tool host serves.
fn tool_section(tools: &[ToolDescriptor], descriptions: ToolDescriptions) -> String {
    let mut catalog: Vec<&ToolDescriptor> = tools.iter().collect();
    catalog.sort_by(|a, b| a.name.cmp(&b.name));
    catalog.dedup_by(|a, b| a.name == b.name);
    let mut out = String::from(
        "## IGNIS tools\n\nGenerated from the IGNIS tool catalog; these are the IGNIS tools attached to this runtime.\n",
    );
    if descriptions == ToolDescriptions::Index {
        out.push_str("Each tool's description and arguments come with its definition.\n");
    }
    if catalog.is_empty() {
        out.push_str(
            "\nNo IGNIS tools are attached in this deployment: you cannot retrieve evidence, compute or submit a bound answer. Say so, and label anything you state from your own knowledge as such.\n",
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
        out.push_str(&match descriptions {
            ToolDescriptions::Full => format!(
                "\n- `{}` ({}): {} Arguments: `{}`; result: `{}`.",
                t.name,
                traits.join(", "),
                t.summary,
                t.input_contract,
                t.output_contract
            ),
            ToolDescriptions::Index => format!("\n- `{}` ({})", t.name, traits.join(", ")),
        });
    }
    out.push('\n');
    out
}

impl HarnessBundle {
    /// Render `source` for a runtime attached to `tools` (wire names; empty: no IGNIS tools), with
    /// the full tool descriptions in the prefix.
    pub fn render(label: &str, source: &HarnessSource, tools: &[String]) -> Result<Self, String> {
        Self::render_with(label, source, tools, ToolDescriptions::Full)
    }

    /// [`Self::render`] with the tool section `descriptions` the runtime needs.
    pub fn render_with(
        label: &str,
        source: &HarnessSource,
        tools: &[String],
        descriptions: ToolDescriptions,
    ) -> Result<Self, String> {
        Self::render_tools(label, source, &catalog_of(tools), descriptions)
    }

    /// [`Self::render_with`] over the descriptors the tool host serves (the catalog with the
    /// deployment's notes, [`HarnessShelf::tools_for`]), so the prefix describes each tool exactly
    /// as the MCP server lists it.
    pub fn render_tools(
        label: &str,
        source: &HarnessSource,
        tools: &[ToolDescriptor],
        descriptions: ToolDescriptions,
    ) -> Result<Self, String> {
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
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        if let Some(w) = skills.windows(2).find(|w| w[0].name == w[1].name) {
            return Err(format!("skill {} appears twice", w[0].name));
        }
        let on_demand = !skills.is_empty() && tools.iter().any(|t| t.name == SKILLS_TOOL);
        let mut text = investigator;
        text.push_str("\n\n");
        text.push_str(&tool_section(tools, descriptions));
        let mut identity = None;
        if on_demand {
            text.push_str(&format!(
                "\n## Skills\n\nA skill is the method for one kind of step. Read it with `{SKILLS_TOOL}` before your first tool call of that kind and follow it; read each skill once per episode.\n"
            ));
            for s in &skills {
                text.push_str(&format!("\n- `{}`: {}", s.name, s.description));
            }
            text.push('\n');
            // The digest covers the served texts too, so the pin identifies what the agent read.
            let mut all = text.clone();
            all.push_str("\n## Skills served on demand\n");
            skills.iter().for_each(|s| all.push_str(&s.block()));
            identity = Some(all);
        } else if !skills.is_empty() {
            text.push_str(
                "\n## Skills\n\nApply a skill when its description matches the question.\n",
            );
            skills.iter().for_each(|s| text.push_str(&s.block()));
        }
        let digest = Digest::of_bytes(identity.as_deref().unwrap_or(&text).as_bytes());
        Ok(HarnessBundle {
            label: label.to_string(),
            text: text.into(),
            // Both forms cover these exact texts with the digest (inline in the prefix, or appended).
            skills: skills.into(),
            digest,
        })
    }

    /// The bundle compiled into this binary, for a runtime attached to `tools`.
    pub fn embedded(tools: &[String]) -> Self {
        Self::embedded_with(tools, ToolDescriptions::Full)
    }

    /// [`Self::embedded`] with the tool section `descriptions` the runtime needs.
    pub fn embedded_with(tools: &[String], descriptions: ToolDescriptions) -> Self {
        Self::embedded_tools(&catalog_of(tools), descriptions)
    }

    /// [`Self::embedded_with`] over the descriptors the tool host serves.
    pub fn embedded_tools(tools: &[ToolDescriptor], descriptions: ToolDescriptions) -> Self {
        Self::render_tools(
            EMBEDDED_LABEL,
            &HarnessSource::embedded(),
            tools,
            descriptions,
        )
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

    /// The bundle's skills, in name order.
    pub fn skills(&self) -> &[Skill] {
        &self.skills
    }

    /// One skill's full text as `skills_read` returns it; `None` for a name the bundle lacks.
    pub fn skill_text(&self, name: &str) -> Option<SkillText> {
        self.skills
            .iter()
            .find(|s| s.name == name)
            .map(|s| SkillText {
                name: s.name.clone(),
                description: s.description.clone(),
                body: s.body.clone(),
                harness_digest: self.digest.clone(),
            })
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

/// The bundles a host rendered for its runtimes, by digest: where `skills_read` finds the skill
/// texts of the bundle an episode is pinned to. It also holds the tool descriptors the host serves,
/// so a bundle's tool section is rendered from them. Cheap to clone (shared).
#[derive(Clone, Default)]
pub struct HarnessShelf {
    bundles: Arc<RwLock<Vec<HarnessBundle>>>,
    served: Arc<RwLock<Vec<ToolDescriptor>>>,
}

impl std::fmt::Debug for HarnessShelf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let held = self.bundles.read().map(|v| v.len()).unwrap_or_default();
        f.debug_struct("HarnessShelf")
            .field("bundles", &held)
            .finish()
    }
}

impl HarnessShelf {
    /// The descriptors the tool host serves (the catalog with the deployment's notes).
    pub fn set_tools(&self, tools: Vec<ToolDescriptor>) {
        *self.served.write().unwrap_or_else(|e| e.into_inner()) = tools;
    }

    /// The descriptor of each tool named in `names` as the host serves it, or as the catalog
    /// states it when the host set none for that name.
    pub fn tools_for(&self, names: &[String]) -> Vec<ToolDescriptor> {
        let served = self.served.read().unwrap_or_else(|e| e.into_inner());
        catalog_of(names)
            .into_iter()
            .map(|d| {
                served
                    .iter()
                    .find(|s| s.name == d.name)
                    .cloned()
                    .unwrap_or(d)
            })
            .collect()
    }

    /// Hold `bundle` (once per digest); the last one added is the host's current bundle.
    pub fn add(&self, bundle: HarnessBundle) {
        let mut v = self.bundles.write().unwrap_or_else(|e| e.into_inner());
        v.retain(|b| b.digest != bundle.digest);
        v.push(bundle);
    }

    /// The bundle with this digest, when this host holds it.
    pub fn get(&self, digest: &Digest) -> Option<HarnessBundle> {
        let v = self.bundles.read().unwrap_or_else(|e| e.into_inner());
        v.iter().find(|b| &b.digest == digest).cloned()
    }

    /// The bundle added last (what a call outside an episode reads).
    pub fn current(&self) -> Option<HarnessBundle> {
        let v = self.bundles.read().unwrap_or_else(|e| e.into_inner());
        v.last().cloned()
    }
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

    /// With `skills_read` attached the prefix carries a one-line index per skill and no skill body,
    /// stays byte-stable, and the digest still moves with any skill's text (which the bundle
    /// serves byte-for-byte), so a pin identifies what `skills_read` returned.
    #[test]
    fn skills_on_demand_leave_the_prefix_but_stay_in_the_digest() {
        let src = HarnessSource::embedded();
        let mut with: Vec<String> = tools();
        with.push(SKILLS_TOOL.into());
        let lazy = HarnessBundle::render("h1", &src, &with).unwrap();
        let eager = HarnessBundle::render("h1", &src, &tools()).unwrap();
        let t = lazy.text();
        assert!(!t.contains("<skill "), "no skill body in the prefix");
        assert!(t.len() < eager.text().len());
        for s in lazy.skills() {
            assert!(t.contains(&format!("\n- `{}`: {}", s.name, s.description)));
            assert!(!t.contains(&s.body));
            let served = lazy.skill_text(&s.name).unwrap();
            served.validate().unwrap();
            assert_eq!(served.body, s.body);
            assert_eq!(&served.harness_digest, lazy.digest());
        }
        assert_eq!(lazy.skills().len(), 3);
        assert!(lazy.skill_text("no-such-skill").is_none());

        let mut crlf = src.clone();
        crlf.skills.reverse();
        for s in &mut crlf.skills {
            s.1 = s.1.replace('\n', "\r\n");
        }
        let again = HarnessBundle::render("h1", &crlf, &with).unwrap();
        assert_eq!(again.text(), t);
        assert_eq!(again.digest(), lazy.digest());

        let mut edited = src.clone();
        edited.skills[0].1.push_str("\nOne more step.\n");
        let moved = HarnessBundle::render("h1", &edited, &with).unwrap();
        assert_eq!(moved.text(), t, "a body edit leaves the prefix alone");
        assert_ne!(moved.digest(), lazy.digest(), "but moves the digest");

        let shelf = HarnessShelf::default();
        shelf.add(lazy.clone());
        shelf.add(moved.clone());
        shelf.add(lazy.clone());
        assert_eq!(shelf.current().as_ref(), Some(&lazy));
        assert_eq!(shelf.get(moved.digest()).as_ref(), Some(&moved));

        // Size with every catalog tool attached (`--nocapture` shows it).
        let all: Vec<String> = tool_catalog().into_iter().map(|t| t.name).collect();
        let before: Vec<String> = all.iter().filter(|n| *n != SKILLS_TOOL).cloned().collect();
        let (b, a) = (
            HarnessBundle::render("h1", &src, &before).unwrap(),
            HarnessBundle::render("h1", &src, &all).unwrap(),
        );
        eprintln!(
            "prefix bytes: skills inlined {}, skills on demand {}",
            b.text().len(),
            a.text().len()
        );
        assert!(a.text().len() < b.text().len());
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

    /// For a runtime that sends the MCP tool definitions itself, the prefix lists the tools by
    /// name and traits only: the same tools, none of the descriptions, its own digest.
    #[test]
    fn an_index_tool_section_lists_the_tools_without_their_descriptions() {
        let all: Vec<String> = tool_catalog().into_iter().map(|t| t.name).collect();
        let full = HarnessBundle::embedded(&all);
        let index = HarnessBundle::embedded_with(&all, ToolDescriptions::Index);
        for t in tool_catalog() {
            assert!(
                index.text().contains(&format!("- `{}` (", t.name)),
                "{}",
                t.name
            );
            assert!(full.text().contains(&t.summary));
            assert!(!index.text().contains(&t.summary), "{}", t.name);
        }
        assert!(index.text().contains("come with its definition"));
        assert_ne!(full.digest(), index.digest());
        eprintln!(
            "prefix bytes: full tool section {}, index {}",
            full.text().len(),
            index.text().len()
        );
        assert!(index.text().len() + 10_000 < full.text().len());
    }
}
