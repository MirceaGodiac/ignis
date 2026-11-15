// The side panel (Linear peek / Slack thread): the selected run, claim, source, figure, route step
// or release, with a back trail. Every link inside it pushes onto the trail; Escape closes it.
import { useEffect, useRef, useState, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import type { AnswerRefusal, ArtifactRef, Claim, EvidenceRef, InlinePayload, ReadResult, ResearchAnswer, SearchHit, SourceLocator } from "../contracts/generated";
import type { ToolCallState } from "../research/eventLog";
import { call, RefusalError, type ExperimentDetail } from "../api";
import { asRefusal } from "../hooks";
import { bboxStyle, locatorKind, pageCaption, sliceSpan } from "../research/units";
import { fmtNum } from "../format";
import { ArtifactValue, EvidenceSummary, locatorText, placeShort } from "../views/Answer";
import { CLAIM_LABEL, ClaimGlyph, ErrorNote, EvidenceClassTag, fmtBytes, Gap, OriginMark, Qualified, RefusalNote, shortId, Synthetic, words } from "../ui/common";
import { builtParagraph, inputWords, scoreParts, modelTitle, readerLimits, useServedCard, writtenBy } from "./models";
import { useCompare } from "./compare";
import { INVESTIGATIONS } from "../data/investigations";
import { CONDITION_GROUPS, conditionLabel, conditionRows, gapText, humanKey, type ConditionRow } from "../research/runConditions";
import { useData } from "./data";
import { itemKey, usePanel, type PanelItem } from "./inspector";
import { FamilyMark, FAMILY_LABEL, IconBack, IconChevron, IconClose, IconExternal, IconSources, IconTable, IconVideo } from "./icons";
import { href } from "./router";
import { callArgs, callQuery, callValue, describeCall, rangeText, sourceTitle, TOOL_KIND, TOOL_LABEL } from "../views/turns";
import "../views/trail.css";
import { isTypedArtifact } from "../research/artifactViews";
import { refusalSummary, splitListMessage } from "../research/noteText";
import { RefusalProblems, ReleaseConstruction, TypedArtifactBody } from "./ArtifactViews";
import { MODE_TEXT as ORIGINAL_MODE_TEXT, PdfSkeleton, SourceOriginal, httpUrl } from "./SourceOriginal";
import { OriginalBody } from "../views/Original";
import { OpenOriginal } from "./OpenOriginal";
import { runSources } from "./runSources";
import { JsonView } from "./JsonView";

const TITLE: Record<PanelItem["kind"], string> = { run: "Run", evidence: "Source", claim: "Claim", artifact: "Figure", step: "Step", release: "Model", sources: "Sources", investigation: "Investigation", original: "NASA original" };

/** The panel heading of a step: what kind of work it was; the sentence below says exactly what. */
const STEP_TITLE: Record<string, string> = { search: "Search", read: "Reading", data: "Test conditions", model: "Model", compute: "Analysis", web: "Web" };

const ARTIFACT_TITLE: Record<string, string> = { AnalysisArtifact: "Figure", ExecutionRecord: "Analysis run", DatasetBundle: "Dataset", EvaluationResult: "Evaluation", ResearchAnswer: "Answer", AnswerRefusal: "Refused answer", VisualObservationBundle: "Video contours" };

function itemTitle(i: PanelItem): string {
  switch (i.kind) {
    case "run":
      return `Run ${i.id.split(":").pop()}`;
    case "claim":
      return `Claim ${i.keyLabel}`;
    case "step":
      return STEP_TITLE[TOOL_KIND[i.call.tool] ?? ""] ?? TOOL_LABEL[i.call.tool] ?? words(i.call.tool);
    case "release":
      return "Model";
    case "investigation":
      return INVESTIGATIONS.get(i.id)?.acronym ?? i.id;
    case "original":
      return i.title ?? "NASA original";
    case "artifact":
      return ARTIFACT_TITLE[i.artifact.contract ?? ""] ?? "Artifact";
    default:
      return TITLE[i.kind];
  }
}

export function Panel() {
  const { current, stack, back, close } = usePanel();
  const body = useRef<HTMLDivElement>(null);
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    body.current?.scrollTo({ top: 0 });
    // Keyboard users land on what they opened, not where they clicked.
    if (current) heading.current?.focus({ preventScroll: true });
  }, [current]);
  useEffect(() => {
    const on = (e: KeyboardEvent) => {
      if (e.key === "Escape" && current && !document.querySelector(".cmd-back")) close();
    };
    window.addEventListener("keydown", on);
    return () => window.removeEventListener("keydown", on);
  }, [current, close]);
  if (!current) return null;
  return (
    <aside className="panel" aria-label="Details">
      <div className="panel-head">
        {stack.length > 1 && (
          <button className="btn btn-quiet btn-icon" onClick={back} aria-label={`Back to ${itemTitle(stack[stack.length - 2])}`} title={`Back to ${itemTitle(stack[stack.length - 2])}`}>
            <IconBack />
          </button>
        )}
        <h2 ref={heading} tabIndex={-1} style={{ outline: "none" }}>
          {/* A run names itself large in the body; the head says only what kind of item it is. */}
          {current.kind === "run" ? TITLE.run : itemTitle(current)}
        </h2>
        <button className="btn btn-quiet btn-icon" onClick={close} aria-label="Close the panel" title="Close (Esc)">
          <IconClose />
        </button>
      </div>
      <div className="panel-body" ref={body}>
        {current.kind === "run" && <RunItem id={current.id} />}
        {current.kind === "evidence" && <EvidenceItem evidence={current.evidence} />}
        {current.kind === "claim" && <ClaimItem item={current} />}
        {current.kind === "artifact" && <ArtifactItem artifact={current.artifact} caption={current.caption} />}
        {current.kind === "step" && <StepItem item={current} />}
        {current.kind === "release" && <ReleaseItem item={current} />}
        {current.kind === "sources" && <SourcesItem answers={current.answers} />}
        {current.kind === "investigation" && <InvestigationItem id={current.id} />}
        {current.kind === "original" && <OriginalItem key={itemKey(current)} sourceId={current.sourceId} loc={current.loc} title={current.title} />}
      </div>
    </aside>
  );
}

// ---------------------------------------------------------------- an investigation (atlas card)

const NEED: Record<string, string> = { prevention: "Prevention", detection: "Detection", suppression: "Suppression" };
const where = (s: { file?: string; page?: number } | null) => (s?.file ? `${s.file}${s.page ? `, p. ${s.page}` : ""}` : null);

/** The atlas card of one investigation: NASA's objective and needs in NASA's words, then the narrative. */
function InvestigationItem({ id }: { id: string }) {
  const { push } = usePanel();
  const inv = INVESTIGATIONS.get(id);
  if (!inv) return <Gap title="No card for this investigation">{id}</Gap>;
  return (
    <>
      <div className="panel-sec">
        <p style={{ fontSize: 15, fontWeight: 500, lineHeight: 1.4 }}>{inv.title}</p>
        <p className="muted small">
          {id}
          {inv.sub_area ? ` · ${inv.sub_area}` : ""}
          {inv.hardware ? ` · ${inv.hardware}` : ""}
        </p>
        <div className="toolbar">
          <a className="btn btn-s" href={href("conditions", null, { tab: "map", inv: id })}>
            On the map
          </a>
          <a className="btn btn-s" href={href("sources", null, { q: inv.acronym ?? id })}>
            Documents
          </a>
        </div>
      </div>
      {inv.needs.length > 0 && (
        <div className="panel-sec">
          <h3>Fire safety</h3>
          {inv.needs.map((n) => (
            <div key={n.need} className="need-row">
              <span className={`need${n.inference ? " inferred" : ""}`}>{NEED[n.need] ?? n.need}</span>
              {n.basis && <blockquote className="quote">{n.basis}</blockquote>}
              {n.inference && <p className="muted small">{n.inference}</p>}
            </div>
          ))}
        </div>
      )}
      {inv.relevance && (
        <div className="panel-sec">
          <h3>Why it matters</h3>
          <p className="reading">{inv.relevance}</p>
        </div>
      )}
      {inv.how_tests_differ && (
        <div className="panel-sec">
          <h3>How the tests differ</h3>
          <p className="reading">{inv.how_tests_differ}</p>
        </div>
      )}
      {inv.findings.length > 0 && (
        <div className="panel-sec">
          <h3>Findings</h3>
          <ol className="findings">
            {inv.findings.map((f, i) => (
              <li key={i}>
                <p>{f.statement}</p>
                <span className="muted small">
                  {f.kind}
                  {where(f.source) ? ` · ${where(f.source)}` : ""}
                </span>
              </li>
            ))}
          </ol>
        </div>
      )}
      {inv.sensors.length > 0 && (
        <div className="panel-sec">
          <h3>Instruments</h3>
          <dl className="kv">
            {inv.sensors.slice(0, 8).map((s, i) => (
              <div key={i} style={{ display: "contents" }}>
                <dt>{s.instrument}</dt>
                <dd>{s.measures}</dd>
              </div>
            ))}
          </dl>
        </div>
      )}
      {inv.not_found.length > 0 && (
        <div className="panel-sec">
          <h3>Not in the documents</h3>
          <ul className="plain-list">
            {inv.not_found.map((t, i) => (
              <li key={i}>{t}</li>
            ))}
          </ul>
        </div>
      )}
      {inv.related.length > 0 && (
        <div className="panel-sec">
          <h3>Related</h3>
          <div className="toolbar">
            {inv.related.map((r) => (
              <button key={r.id} className="btn btn-s" title={r.shared.join("; ")} onClick={() => push({ kind: "investigation", id: r.id })}>
                {r.acronym ?? r.id}
              </button>
            ))}
          </div>
        </div>
      )}
      <p className="muted small">Summaries written by Claude Opus 5.5 from NASA's documents; each finding names its source.</p>
    </>
  );
}

// ---------------------------------------------------------------- sources of a question

/**
 * Every source the question's answers cite, numbered in order of first citation across answers, with
 * the claims each passage supports. A passage opens its original at the cited place.
 */
function SourcesItem({ answers }: { answers: ResearchAnswer[] }) {
  const { push } = usePanel();
  const rows = new Map<string, { n: number; cites: { e: EvidenceRef; claims: Claim[] }[] }>();
  for (const a of answers) {
    const claimsOf = new Map<string, Claim[]>();
    for (const c of a.claims ?? []) for (const s of c.support ?? []) if (s.kind === "evidence") claimsOf.set(s.evidence_id, [...(claimsOf.get(s.evidence_id) ?? []), c]);
    for (const e of a.evidence ?? []) {
      const r = rows.get(e.source_id) ?? { n: rows.size + 1, cites: [] };
      r.cites.push({ e, claims: claimsOf.get(e.evidence_id) ?? [] });
      rows.set(e.source_id, r);
    }
  }
  if (!rows.size) return <Gap title="No sources yet">Sources appear when an answer cites them.</Gap>;
  return (
    <ol className="src-panel">
      {[...rows.entries()].map(([id, r]) => (
        <li key={id}>
          <div className="src-panel-head">
            <span className="src-no">{r.n}</span>
            <code>{shortId(id, 30)}</code>
          </div>
          {r.cites.map(({ e, claims }) => (
            <button key={e.evidence_id} className="src-cite" onClick={() => push({ kind: "evidence", evidence: e, claim: claims[0] })}>
              <span className="place">{placeShort(e)}</span>
              <span className="txt">{e.quote ?? claims[0]?.text ?? locatorText(e)}</span>
            </button>
          ))}
        </li>
      ))}
    </ol>
  );
}

// ---------------------------------------------------------------- run

/** A question about one run, naming it as NASA does when its record says how. */
function askAboutRun(id: string, s: { investigation_id: string; aliases?: { scheme: string; name: string }[] }): string {
  const nasa = (s.aliases ?? []).find((a) => !["pi", "campaign_day", "sample_number"].includes(a.scheme))?.name;
  const name = nasa ? `${nasa} (${id})` : id;
  return `What do the NASA sources and data report for run ${name} of ${s.investigation_id}: its conditions, what was observed, and which files record it?`;
}

const TABLE_EXT = ["csv", "tsv", "xlsx", "xls", "parquet", "json"];
const VIDEO_EXT = ["mp4", "mov", "avi", "mkv", "webm"];
const extOf = (path: string | null) => (path?.match(/\.([a-z0-9]{1,8})$/i)?.[1] ?? "").toLowerCase();

/** A NASA file name as a reader scans it: no folder, extension or investigation prefix, parts split
 * at underscores ("PSI-159_Experimental table_CFI-G_Normal Flames.csv" → "Experimental table · CFI-G · Normal Flames"). */
function fileDisplayName(path: string, investigation: string): string {
  const base = path.split(/[\\/]/).pop() ?? path;
  const stem = base.replace(/\.[a-z0-9]{1,8}$/i, "");
  const bare = stem.startsWith(`${investigation}_`) ? stem.slice(investigation.length + 1) : stem;
  return bare.split("_").map((p) => p.trim()).filter(Boolean).join(" · ") || base;
}

/** The NASA files this run's record was read from, each opened at the place the record cites. */
function RunOriginals({ record, investigation }: { record: unknown; investigation: string }) {
  const files = runSources(record);
  if (files.length === 0) return null;
  return (
    <div className="panel-sec">
      <h3>NASA originals</h3>
      <ul className="file-list">
        {files.map((f) => {
          const ext = extOf(f.path);
          const Icon = TABLE_EXT.includes(ext) ? IconTable : VIDEO_EXT.includes(ext) ? IconVideo : IconSources;
          const names = f.uses.filter((u) => u.startsWith("name ")).map((u) => u.slice(5));
          const fields = f.uses.filter((u) => !u.startsWith("name ")).map((u) => conditionLabel(u.replace(/ /g, "_")));
          const place = f.loc.page != null ? `p. ${f.loc.page}` : f.loc.row != null ? `row ${f.loc.row}` : null;
          const gives = [...fields, ...(names.length ? [names.length === 1 ? "run name" : "run names"] : [])].map((x, i) => (i === 0 ? x.charAt(0).toUpperCase() + x.slice(1) : x.charAt(0).toLowerCase() + x.slice(1)));
          const meta = [ext ? ext.toUpperCase() : null, place, gives.length ? gives.join(", ") : null].filter(Boolean).join(" · ");
          const full = [f.path ?? f.sourceId, names.length ? `Names: ${names.join(", ")}` : null, fields.length ? `Conditions: ${fields.join(", ")}` : null].filter(Boolean).join("\n");
          return (
            <li key={f.sourceId} className="file-card">
              <span className="file-icon" aria-hidden="true">
                <Icon />
              </span>
              <span className="file-main" title={full}>
                <span className="file-name">{f.path ? fileDisplayName(f.path, investigation) : <code>{f.sourceId.slice(0, 19)}…</code>}</span>
                {meta && <span className="file-meta">{meta}</span>}
              </span>
              <OpenOriginal sourceId={f.sourceId} loc={f.loc} title={f.path} label="Open ↗" />
            </li>
          );
        })}
      </ul>
    </div>
  );
}

/** A label, then the value with its unit in a muted tone; a long label keeps one line and gives way to the value. */
/** A model that can forecast this run: its plain name and its qualified state. */
function RunModelRow({ id, modelName }: { id: string; modelName: string }) {
  const card = useServedCard(id);
  return <SpecRow label={modelTitle(modelName, card)} value={<Qualified basis={card?.qualification.basis} />} />;
}

function SpecRow({ label, value, unit, note }: { label: string; value: ReactNode; unit?: string; note?: string | null }) {
  return (
    <div className="spec-row">
      <dt title={label}>{label}</dt>
      <dd>
        <span className="v">{value}</span>
        {unit && <span className="u">{unit}</span>}
        {note && <span className="spec-note">{note}</span>}
      </dd>
    </div>
  );
}

function RunItem({ id }: { id: string }) {
  const { origin } = useData();
  const cmp = useCompare();
  const q = useQuery({ queryKey: [origin, "experiment", id], queryFn: () => call<ExperimentDetail>(origin, `/experiments/${encodeURIComponent(id)}`), retry: false });
  if (q.isPending) return <p className="muted">Loading the run…</p>;
  if (q.error) {
    const r = asRefusal(q.error);
    if (q.error instanceof RefusalError && (q.error.status === 404 || r?.kind === "not_found"))
      return (
        <Gap title="This run is not in the current data source">
          <code>{id}</code> is named by an artifact or link from another source. Switch the data source in the sidebar to open it.
        </Gap>
      );
    return r ? <RefusalNote refusal={r} /> : null;
  }
  const { record, summary, releases } = q.data;
  const rows: ConditionRow[] = conditionRows(summary.conditions);
  if (summary.gravity?.value_si != null) rows.push({ key: "gravity", label: "Gravity", value: fmtNum(summary.gravity.value_si), unit: "g₀", note: null, group: "Atmosphere", text: "" });
  const groups = CONDITION_GROUPS.map((g) => ({ g, rows: rows.filter((r) => r.group === g) })).filter((x) => x.rows.length);
  const gaps = summary.gaps.flatMap((g) => gapText(g) ?? []);
  const family = FAMILY_LABEL[summary.family] ?? summary.family;
  const config = summary.configuration?.replace(/_/g, " ");
  const sub = [summary.fuel_or_material, config && !family.toLowerCase().includes(config.toLowerCase()) ? config : null, summary.investigation_id].filter(Boolean).join(" · ");
  return (
    <>
      <div className="run-head">
        <div className="run-title">
          <p className="run-id">{id.split(":").pop()}</p>
          <span className="tag">
            <FamilyMark family={summary.family} size={9} /> {family}
          </span>
          {summary.synthetic && <Synthetic label="Synthetic run" />}
        </div>
        {sub && <p className="run-sub">{sub}</p>}
        <div className="run-actions">
          <a className="btn btn-s btn-primary" href={href("research", null, { new: "1", q: askAboutRun(id, summary) })}>
            Ask about this run
          </a>
          <button className="btn btn-s" onClick={() => cmp.toggle(id)} aria-pressed={cmp.has(id)}>
            {cmp.has(id) ? "Remove from compare" : "Add to compare"}
          </button>
          {releases.length > 0 && (
            <a className="btn btn-s" href={href("lab", id)}>
              Forecast
            </a>
          )}
        </div>
      </div>
      {groups.map(({ g, rows: rs }) => (
        <div key={g} className="panel-sec">
          <h3>{groups.length === 1 ? "Conditions" : g}</h3>
          <dl className="spec">
            {rs.map((r) => (
              <SpecRow key={r.key} label={r.label} value={r.value} unit={r.unit} note={r.note} />
            ))}
          </dl>
        </div>
      ))}
      <RunOriginals record={record} investigation={summary.investigation_id} />
      {summary.quantities.length > 0 && (
        <div className="panel-sec">
          <h3>Measured series</h3>
          <dl className="spec">
            {summary.quantities.map((qq) => (
              <SpecRow key={qq.quantity} label={humanKey(qq.quantity)} value={`${qq.count} points`} unit={`${fmtNum(+qq.archive_t_min_s.toFixed(2))}–${fmtNum(+qq.archive_t_max_s.toFixed(2))} s`} />
            ))}
          </dl>
        </div>
      )}
      {releases.length > 0 && (
        <div className="panel-sec">
          <h3>Models</h3>
          <dl className="spec">
            {releases.map((r) => (
              <RunModelRow key={r.id} id={r.id} modelName={r.model_name} />
            ))}
          </dl>
        </div>
      )}
      {gaps.length > 0 && (
        <div className="panel-sec">
          <h3>Not available</h3>
          <ul className="gap-list">
            {gaps.map((g) => (
              <li key={g}>{g}</li>
            ))}
          </ul>
        </div>
      )}
    </>
  );
}

// ---------------------------------------------------------------- evidence

function EvidenceItem({ evidence }: { evidence: EvidenceRef }) {
  const { client, researchMode, synthetic } = useData();
  const loc = evidence.locator ?? null;
  const kind = locatorKind(loc);
  const page = loc?.page ?? null;
  const read = useQuery({ queryKey: [researchMode, "read", evidence.source_id, page], queryFn: () => client.readSource(evidence.source_id, page!), enabled: kind === "page" && page != null, retry: false });
  const seg = read.data?.segments.find((s) => s.locator.page === page);
  const span =
    seg && loc?.char_start != null && loc.char_end != null && seg.locator.char_start != null
      ? sliceSpan(seg.text, loc.char_start - seg.locator.char_start, loc.char_end - seg.locator.char_start, loc.offset_unit)
      : null;
  return (
    <>
      <div className="panel-sec">
        <div className="toolbar">
          <EvidenceClassTag cls={evidence.evidence_class} />
          <OriginMark origin={evidence.origin} pageSynthetic={synthetic} />
        </div>
        {evidence.quote ? (
          <blockquote className="quote">{evidence.quote}</blockquote>
        ) : span ? (
          <blockquote className="quote">{span}</blockquote>
        ) : (
          <p className="muted" style={{ fontSize: 13 }}>
            {kind === "metadata" ? "Metadata-only citation: the source bytes were not obtained, so there is no passage to show." : "No quote is carried with this citation."}
          </p>
        )}
      </div>
      <div className="panel-sec">
        <h3>Where it is</h3>
        <dl className="kv">
          <dt>Source</dt>
          <dd>
            <code>{evidence.source_id}</code>
          </dd>
          {evidence.parse_view_id && (
            <>
              <dt>Parse view</dt>
              <dd>
                <code>{evidence.parse_view_id}</code>
              </dd>
            </>
          )}
          <dt>Locator</dt>
          <dd>{locatorText(evidence)}</dd>
          {loc?.char_start != null && (
            <>
              <dt>Characters</dt>
              <dd className="num">
                {loc.char_start}–{loc.char_end} ({loc.offset_unit ? words(loc.offset_unit) : "unit not declared: not citable"})
              </dd>
            </>
          )}
          {loc && (
            <>
              <dt>Asset</dt>
              <dd>
                <code>{shortId(loc.asset_sha256, 26)}</code>
              </dd>
            </>
          )}
        </dl>
      </div>
      {(kind === "page" || kind === "video") && <SourceOriginal evidence={evidence} />}
      {kind === "page" && loc && (
        <div className="panel-sec">
          <h3>Parsed text, {pageCaption(loc)}</h3>
          <div className="page-frame" aria-label={`Text of ${pageCaption(loc)} with the cited region marked`}>
            {read.isPending && <p className="muted">Reading the page…</p>}
            {read.error && <ErrorNote error={read.error} title="The page could not be read" />}
            {seg && <p>{seg.text}</p>}
            {bboxStyle(loc.bbox) && <div className="bbox" style={bboxStyle(loc.bbox)!} title="Cited region" />}
          </div>
          {read.data && !read.data.complete && <p className="muted" style={{ fontSize: 12 }}>Partial read: the source continues on later pages.</p>}
          <p className="muted" style={{ fontSize: 12 }}>What IGNIS read from this page (parse view); the box marks the cited region.</p>
        </div>
      )}
    </>
  );
}

// ---------------------------------------------------------------- claim

const KIND_NOTE: Record<string, string> = {
  source_reports: "What a source states. It needs a citation to a passage.",
  observed: "A measured value. It must cite an original or source-derived observation, never a model.",
  computed: "A value IGNIS computed. It points to the artifact field that holds it.",
  predicted: "A model prediction. It points into an analysis artifact; it is not a measurement.",
  interpretation: "The agent's reading of the evidence. It may cite anything or nothing.",
  background: "Stated from the model's own knowledge, not retrieved or computed in this episode. Not from the NASA corpus; IGNIS did not verify it, including any number in it.",
};

const VERDICT_TONE: Record<string, string> = { supported: "tag-ok", partially_supported: "tag-warn", not_supported: "tag-bad", unverifiable: "tag-line" };
const CHECK_METHOD: Record<string, string> = { host_rule: "checked by a host rule", nli_model: "checked by an entailment model", llm_judge: "checked by a language-model judge", human: "checked by a person" };

function ClaimItem({ item }: { item: Extract<PanelItem, { kind: "claim" }> }) {
  const { client } = useData();
  const panel = usePanel();
  const { claim } = item;
  const ev = new Map(item.evidence.map((e) => [e.evidence_id, e]));
  return (
    <>
      <div className="panel-sec">
        <span className="tag" style={{ justifySelf: "start" }}>
          <ClaimGlyph kind={claim.kind} /> {CLAIM_LABEL[claim.kind]}
        </span>
        <p style={{ fontFamily: "var(--font-text)", fontSize: 16, lineHeight: 1.55 }}>{claim.text}</p>
        <p className="muted" style={{ fontSize: 12 }}>{KIND_NOTE[claim.kind]}</p>
      </div>
      <div className="panel-sec">
        <h3>Support</h3>
        {(claim.support ?? []).length === 0 && <Gap title="No support">{claim.kind === "interpretation" ? "This is the agent's interpretation only." : claim.kind === "background" ? "Model knowledge, not from the NASA corpus: no source is cited and IGNIS did not verify it." : "Treat this claim as unverified."}</Gap>}
        {(claim.support ?? []).map((s, i) =>
          s.kind === "evidence" ? (
            ev.get(s.evidence_id) ? (
              <button key={i} className="stop" style={{ gridTemplateColumns: "minmax(0,1fr) auto", padding: "8px 10px", border: "1px solid var(--rule)" }} onClick={() => panel.push({ kind: "evidence", evidence: ev.get(s.evidence_id)!, claim })}>
                <span className="what">
                  <EvidenceSummary e={ev.get(s.evidence_id)!} />
                  {ev.get(s.evidence_id)!.quote && <span style={{ fontFamily: "var(--font-text)", fontSize: 14, color: "var(--ink-2)" }}>“{ev.get(s.evidence_id)!.quote}”</span>}
                </span>
                <span className="when">open</span>
              </button>
            ) : (
              <p key={i} className="note note-refusal">
                Evidence {s.evidence_id} is cited but missing from the answer.
              </p>
            )
          ) : (
            <div key={i} className="panel-sec">
              <ArtifactValue client={client} support={s} />
              {item.artifacts.find((a) => a.artifact_id === s.artifact_id) && (
                <button className="linkish" style={{ fontSize: 13 }} onClick={() => panel.push({ kind: "artifact", artifact: item.artifacts.find((a) => a.artifact_id === s.artifact_id)! })}>
                  Open the artifact that holds this value
                </button>
              )}
            </div>
          ),
        )}
      </div>
      <div className="panel-sec">
        <h3>Support check</h3>
        {claim.checked_by ? (
          <>
            <div className="toolbar">
              <span className={`tag ${VERDICT_TONE[claim.checked_by.verdict]}`}>{words(claim.checked_by.verdict)}</span>
              <span className="muted small">
                {CHECK_METHOD[claim.checked_by.method]}
                {claim.checked_by.model_id ? ` · ${claim.checked_by.model_id}` : ""}
                {claim.checked_by.confidence != null ? ` · confidence ${claim.checked_by.confidence.toFixed(2)}` : ""}
              </span>
            </div>
            <p className="muted small">Whether the cited support says what the claim says. A check by a model can be wrong; open the support to judge.</p>
          </>
        ) : (
          <p className="muted small">Not checked: no rule, model or person verified that the support says what this claim says.</p>
        )}
      </div>
      <Gap title="Why the agent made this claim">
        The model's own explanation is not carried by the answer contract yet. When the runtime records it, it appears here, marked as the model's explanation.
      </Gap>
    </>
  );
}

// ---------------------------------------------------------------- artifact

function ArtifactItem({ artifact, caption }: { artifact: ArtifactRef; caption?: string }) {
  const { client, researchMode, synthetic } = useData();
  const panel = usePanel();
  const q = useQuery({ queryKey: [researchMode, "artifact", artifact.artifact_id], queryFn: () => client.getArtifactJson(artifact.artifact_id), retry: false, enabled: artifact.media_type.includes("json") });
  return (
    <>
      <div className="panel-sec">
        <div className="toolbar">
          {artifact.contract === "AnalysisArtifact" ? (
            <span className="tag tag-flame">
              <span className="glyph g-predicted" aria-hidden="true" /> Model output
            </span>
          ) : (
            <span className="tag">{artifact.contract ?? artifact.media_type}</span>
          )}
          <OriginMark origin={artifact.origin} pageSynthetic={synthetic} />
        </div>
        {caption && <p style={{ fontSize: 13 }}>{caption}</p>}
      </div>
      {isTypedArtifact(artifact.contract) && q.data != null && <TypedArtifactBody contract={artifact.contract} data={q.data} />}
      {isTypedArtifact(artifact.contract) && q.error != null && <ErrorNote error={q.error} title="The typed view needs the payload, which is not available here" />}
      <dl className="kv">
        <dt>Artifact</dt>
        <dd>
          <code>{artifact.artifact_id}</code>
        </dd>
        <dt>Size</dt>
        <dd>
          {fmtBytes(artifact.byte_length)} · {artifact.media_type}
        </dd>
        <dt>Created</dt>
        <dd className="num">{artifact.created_at}</dd>
        {artifact.produced_by?.job_id && (
          <>
            <dt>Job</dt>
            <dd>
              <code>{artifact.produced_by.job_id}</code>
            </dd>
          </>
        )}
        {artifact.produced_by?.release_id && (
          <>
            <dt>Release</dt>
            <dd>
              <code>{shortId(artifact.produced_by.release_id, 26)}</code>
            </dd>
          </>
        )}
        <dt>SHA-256</dt>
        <dd>
          <code>{shortId(artifact.sha256, 26)}</code>
        </dd>
      </dl>
      {(artifact.inputs ?? []).length > 0 && (
        <div className="panel-sec">
          <h3>Computed from</h3>
          {artifact.inputs!.map((i) => (
            <div key={`${i.kind}${String(i.id)}`} className="stat-line">
              <span className="muted">{words(i.kind)}</span>
              {i.kind === "run" ? (
                <button className="linkish" onClick={() => panel.push({ kind: "run", id: String(i.id) })}>
                  {String(i.id)}
                </button>
              ) : (
                <code>{shortId(String(i.id))}</code>
              )}
            </div>
          ))}
        </div>
      )}
      <details>
        <summary className="muted" style={{ fontSize: 12, cursor: "pointer" }}>
          Payload
        </summary>
        {q.isPending && artifact.media_type.includes("json") && <p className="muted">Loading…</p>}
        {q.error && <ErrorNote error={q.error} title="The payload is not available here" />}
        {q.data != null && <JsonView value={q.data} />}
      </details>
    </>
  );
}

// ---------------------------------------------------------------- route step

/** A refused `answer_submit` (K1.3): every problem the host found, so the agent (and the reader) see what to fix. */
function SubmissionRefusal({ refusalRef, fallback }: { refusalRef: ArtifactRef; fallback: string | null }) {
  const { client, researchMode } = useData();
  const q = useQuery({ queryKey: [researchMode, "artifact", refusalRef.artifact_id], queryFn: () => client.getArtifactJson(refusalRef.artifact_id) as Promise<AnswerRefusal>, retry: false, staleTime: Infinity });
  return (
    <div className="panel-sec">
      <h3>Why the answer was refused</h3>
      {q.isPending && <p className="muted small">Reading the refusal…</p>}
      {q.data && <RefusalProblems refusal={q.data} />}
      {q.error && (
        <>
          <ErrorNote error={q.error} title="The refusal artifact could not be read; the tool's message follows" />
          {fallback && <p className="small" style={{ whiteSpace: "pre-wrap" }}>{fallback}</p>}
        </>
      )}
    </div>
  );
}


const EXEC_NOTE: Record<string, string> = {
  completed: "The call ran.",
  accepted: "Long work started as a job.",
  refused: "The host or service declined the call (scope, input or support).",
  failed: "A technical failure: no result.",
  cancelled: "The call was cancelled.",
};
const SCI_NOTE: Record<string, string> = {
  not_assessed: "Not assessed.",
  conclusive: "The result answers what was asked.",
  inconclusive: "The result does not decide the question.",
  insufficient_evidence: "A valid conclusion: the data to answer does not exist yet.",
  out_of_domain: "A valid conclusion: the question is outside the tool's domain.",
};

function Payload({ payload, onArtifact, title = "Result" }: { payload: InlinePayload; onArtifact: (a: ArtifactRef) => void; title?: string | null }) {
  const v = payload.value as unknown;
  const flat = v && typeof v === "object" && !Array.isArray(v) && Object.values(v as object).every((x) => x === null || ["string", "number", "boolean"].includes(typeof x));
  return (
    <div className="panel-sec">
      {title && <h3>{title}</h3>}
      {payload.partial && (
        <p style={{ fontSize: 13 }}>
          <span className="tag tag-warn">preview only</span>{" "}
          {payload.full && (
            <button className="linkish" onClick={() => onArtifact(payload.full!)}>
              Open the full result
            </button>
          )}
        </p>
      )}
      {flat ? (
        <dl className="kv">
          {Object.entries(v as Record<string, unknown>).map(([k, x]) => (
            <div key={k} style={{ display: "contents" }}>
              <dt>{words(k)}</dt>
              <dd className="num">{String(x)}</dd>
            </div>
          ))}
        </dl>
      ) : (
        <JsonView value={v} />
      )}
    </div>
  );
}

const STEP_STATE_NOTE: Record<string, { title: string; tone: string }> = {
  failed: { title: "This step failed", tone: "note-declined" },
  refused: { title: "IGNIS declined this step", tone: "note-declined" },
  cancelled: { title: "This step was cancelled", tone: "note-gap" },
};

/** The tool's message, read: a list it names after a colon becomes a short list; otherwise the text as it came. */
function StepMessage({ message }: { message: string }) {
  const l = splitListMessage(message);
  if (!l) return <p className="declined-msg">{message.charAt(0).toUpperCase() + message.slice(1)}</p>;
  const last = l.head.length - 1;
  return (
    <>
      {l.head.map((h, i) => (
        <p key={i} className="declined-msg">
          {h}
          {i === last ? ":" : "."}
        </p>
      ))}
      <ul className="declined-list">
        {l.items.map((it) => (
          <li key={it.name}>
            <code>{it.name}</code>
            {it.note && <span className="muted"> {it.note}</span>}
          </li>
        ))}
      </ul>
      {l.tail.map((t, i) => (
        <p key={i} className="declined-msg muted">
          {t}.
        </p>
      ))}
    </>
  );
}

/** The call's result value: inline, or read from its stored artifact when only a preview travelled. */
function useResultValue(call: ToolCallState): Record<string, unknown> | null {
  const { client, researchMode } = useData();
  const inline = callValue(call);
  const full = call.result?.result?.partial ? call.result.result.full ?? null : null;
  const q = useQuery({
    queryKey: [researchMode, "artifact", full?.artifact_id],
    queryFn: () => client.getArtifactJson(full!.artifact_id),
    enabled: !!full && (full.media_type ?? "").includes("json"),
    retry: false,
    staleTime: Infinity,
  });
  const loaded = q.data && typeof q.data === "object" && !Array.isArray(q.data) ? (q.data as Record<string, unknown>) : null;
  return loaded ?? inline;
}

const strs = (v: unknown): string[] => (Array.isArray(v) ? v.filter((x): x is string => typeof x === "string" && x.trim() !== "") : []);

/** What a search was given and what came back, as a reader would want it: the query, its limits, the passages. */
function SearchStep({ call }: { call: ToolCallState }) {
  const panel = usePanel();
  const a = callArgs(call);
  const v = callValue(call);
  const full = useResultValue(call);
  const q = callQuery(call);
  const terms = Array.isArray(a?.expected_terms) ? (a!.expected_terms as { term?: unknown }[]).flatMap((t) => (typeof t?.term === "string" ? [t.term] : [])) : [];
  const f = (a?.filters ?? null) as Record<string, unknown> | null;
  const limits = [...strs(f?.families).map((x) => FAMILY_LABEL[x] ?? x), ...strs(f?.investigation_ids)];
  const hits = Array.isArray(full?.hits) ? (full!.hits as SearchHit[]) : null;
  const total = typeof full?.total_count === "number" ? full.total_count : null;
  const down = Array.isArray(full?.backends) ? (full!.backends as { backend?: string; status?: string; detail?: string | null }[]).filter((b) => b?.status === "unavailable") : [];
  return (
    <>
      {q && (
        <div className="panel-sec">
          <h3>Query</h3>
          <p className="step-query">{q}</p>
          {terms.length > 0 && <p className="muted small">Also looking for: {terms.join(", ")}</p>}
          {limits.length > 0 && <p className="muted small">Limited to {limits.join(", ")}</p>}
        </div>
      )}
      {(down.length > 0 || v?.degraded === true) && (
        <div className="note note-gap">
          <b>The search ran degraded</b>
          {down.map((b, i) => (
            <span key={i}>
              {words(b.backend ?? "a backend")} was unavailable{b.detail ? `: ${b.detail}` : "."}
            </span>
          ))}
          {!down.length && <span>The service reported degraded results.</span>}
        </div>
      )}
      {hits && (
        <div className="panel-sec">
          <h3>
            {hits.length === 0 ? "No passages came back" : `${hits.length} passage${hits.length === 1 ? "" : "s"}${total != null && total > hits.length ? ` of ${total}` : ""}`}
          </h3>
          {hits.length === 0 && <p className="muted small">An empty page does not prove the corpus lacks it.</p>}
          <ol className="step-hits">
            {hits.map((h) => (
              <li key={`${h.rank}-${h.evidence.evidence_id}`}>
                <button className="step-hit" onClick={() => panel.push({ kind: "evidence", evidence: h.evidence })}>
                  <span className="step-hit-title">{h.title ?? sourceTitle(h.evidence.source_id) ?? "Untitled source"}</span>
                  <span className="muted small">{placeShort(h.evidence)}</span>
                  {h.snippet && <span className="step-hit-snippet">{h.snippet}</span>}
                </button>
              </li>
            ))}
          </ol>
          {typeof full?.next_cursor === "string" && <p className="muted small">More results exist past this page.</p>}
        </div>
      )}
    </>
  );
}

/** What a read looked at and the text that came back, page by page. */
function ReadStep({ call }: { call: ToolCallState }) {
  const panel = usePanel();
  const a = callArgs(call);
  const v = useResultValue(call) as Partial<ReadResult> | null;
  const sid = typeof a?.source_id === "string" ? a.source_id : null;
  const title = sourceTitle(sid);
  const where = rangeText(a?.range);
  const segs = Array.isArray(v?.segments) ? v!.segments! : [];
  const range = a?.range as { kind?: string; first?: number } | undefined;
  const loc: SourceLocator | null = segs[0]?.locator ?? (sid && range?.kind === "pages" && typeof range.first === "number" ? { asset_sha256: sid.replace(/^sha256:/, ""), page: range.first } : null);
  return (
    <>
      {sid && (
      <div className="panel-sec">
        <h3>Source</h3>
        <p className="step-source">{title ?? "A source without a known title"}</p>
        {where && <p className="muted small">{where.charAt(0).toUpperCase() + where.slice(1)}</p>}
          <div className="toolbar">
            <button className="btn btn-s" onClick={() => panel.push({ kind: "original", sourceId: sid, loc, title })}>
              Open the original{loc?.page != null ? ` at page ${loc.page}` : ""}
            </button>
          </div>
      </div>
      )}
      {segs.length > 0 && (
        <div className="panel-sec">
          <h3>What it read</h3>
          {segs.map((s, i) => (
            <Passage key={i} caption={s.locator ? pageCaption(s.locator) : null} text={s.text} />
          ))}
          {v?.complete === false && <p className="muted small">Part of the range: the rest was left for a next read.</p>}
        </div>
      )}
    </>
  );
}

function Passage({ caption, text }: { caption: string | null; text: string }) {
  const [open, setOpen] = useState(false);
  const long = text.length > 700;
  return (
    <figure className={`step-passage${open || !long ? " open" : ""}`}>
      {caption && caption !== "no page" && <figcaption className="muted small">{caption.charAt(0).toUpperCase() + caption.slice(1)}</figcaption>}
      <blockquote className="quote">{text}</blockquote>
      {long && (
        <button className="linkish small" onClick={() => setOpen((o) => !o)} aria-expanded={open}>
          {open ? "Show less" : "Show the whole passage"}
        </button>
      )}
    </figure>
  );
}

/** A conditions lookup: what was asked for, and how much came back. */
function ConditionsStep({ call, onArtifact }: { call: ToolCallState; onArtifact: (a: ArtifactRef) => void }) {
  const a = callArgs(call);
  const v = callValue(call);
  const p = call.result?.result;
  const inv = strs(a?.investigation_ids);
  const keys = strs(a?.keys);
  const rows = Array.isArray(v?.rows) ? v!.rows.length : null;
  const total = typeof v?.total_count === "number" ? v.total_count : null;
  return (
    <div className="panel-sec">
      <h3>Asked for</h3>
      <p style={{ fontSize: 13.5 }}>
        {inv.length ? `Runs of ${inv.join(", ")}` : "Runs of every investigation"}
        {keys.length ? `, with ${keys.map(words).join(", ")}` : ", every recorded condition"}
      </p>
      {rows != null && (
        <p style={{ fontSize: 13.5 }}>
          {rows === 0 ? "No run matched." : `${total != null && total > rows ? `${rows} of ${total}` : rows} run${rows === 1 ? "" : "s"} came back.`}
        </p>
      )}
      {p?.partial && (
        <p style={{ fontSize: 13 }}>
          <span className="tag tag-warn">preview only</span>{" "}
          {p.full && (
            <button className="linkish" onClick={() => onArtifact(p.full!)}>
              Open the full table
            </button>
          )}
        </p>
      )}
    </div>
  );
}

function StepItem({ item }: { item: Extract<PanelItem, { kind: "step" }> }) {
  const panel = usePanel();
  const { synthetic } = useData();
  const call = item.call;
  const r = call.result;
  const d = describeCall(call, { full: true });
  const onArtifact = (a: ArtifactRef) => panel.push({ kind: "artifact", artifact: a });
  // A refused answer_submit carries its problems as an AnswerRefusal artifact (M7); the error message
  // repeats them as text, so it is shown only when that artifact cannot be read.
  const refusalRef = r?.tool === "answer_submit" && r.execution_status === "refused" ? (r.artifact_refs ?? []).find((a) => a.contract === "AnswerRefusal") ?? null : null;
  const bad = r ? STEP_STATE_NOTE[r.execution_status] : undefined;
  const sci = r?.scientific_status;
  const verdict = sci && sci !== "not_assessed" && sci !== "conclusive" ? SCI_NOTE[sci] : null;
  const generic = !["sources_search", "sources_read", "conditions_query"].includes(call.tool);
  const v = r?.result?.value;
  const flat = v && typeof v === "object" && !Array.isArray(v) && !("stored_as_artifact" in v) && Object.values(v as object).every((x) => x === null || ["string", "number", "boolean"].includes(typeof x));
  // A finished job's outputs travel on the job, not repeated in artifact_refs.
  const allRefs = [...(r?.artifact_refs ?? []), ...(r?.job?.outputs ?? [])].filter((a, i, xs) => xs.findIndex((b) => b.artifact_id === a.artifact_id) === i);
  const outputs = allRefs.filter((a) => a.contract !== "AnswerRefusal");
  return (
    <>
      <div className="panel-sec step-lead">
        <p className="step-sentence">{d.text}</p>
        {!r && <p className="muted small">Working: no result yet.</p>}
        {r?.execution_status === "accepted" && <p className="muted small">Started as a background job.</p>}
        {verdict && <p className="small">{verdict}</p>}
        {call.tool === "models_forecast" && <p className="muted small">A forecast is model output, never a measurement.</p>}
        {r && <OriginMark origin={r.origin} pageSynthetic={synthetic} />}
      </div>
      {bad && !refusalRef && (
        <div className={`note ${bad.tone}`} role="status">
          <b>{r?.error ? refusalSummary(r.error.code, call.tool) : bad.title}</b>
          {r?.error?.message && <StepMessage message={r.error.message} />}
        </div>
      )}
      {refusalRef && <SubmissionRefusal refusalRef={refusalRef} fallback={r?.error?.message ?? null} />}
      {call.tool === "sources_search" && <SearchStep call={call} />}
      {call.tool === "sources_read" && <ReadStep call={call} />}
      {call.tool === "conditions_query" && <ConditionsStep call={call} onArtifact={onArtifact} />}
      {generic && flat && <Payload payload={r!.result!} onArtifact={onArtifact} />}
      {call.tool !== "conditions_query" && r?.result?.partial && r.result.full && (
        <div className="panel-sec">
          <p style={{ fontSize: 13 }}>
            <span className="tag tag-warn">preview only</span>{" "}
            <button className="linkish" onClick={() => onArtifact(r.result!.full!)}>
              Open the full result
            </button>
          </p>
        </div>
      )}
      {r?.job && (
        <div className="panel-sec">
          <h3>Background job</h3>
          <p style={{ fontSize: 13.5 }}>
            <span className="muted">When the call returned: </span>
            {words(r.job.status)}
            {r.job.progress && (
              <span className="muted">
                {" "}
                · {r.job.progress.completed}
                {r.job.progress.total != null ? ` of ${r.job.progress.total}` : " (total unknown)"} {r.job.progress.unit}
              </span>
            )}
          </p>
        </div>
      )}
      {outputs.length > 0 && (
        <div className="panel-sec">
          <h3>What it produced</h3>
          <div className="toolbar" style={{ flexWrap: "wrap" }}>
            {outputs.map((a) => (
              <button key={a.artifact_id} className="btn btn-s" onClick={() => onArtifact(a)}>
                Open {(ARTIFACT_TITLE[a.contract ?? ""] ?? "result").toLowerCase()}
              </button>
            ))}
          </div>
        </div>
      )}
      {call.rationale.length > 0 && (
        <div className="panel-sec">
          <h3>Why, in the agent's words</h3>
          {call.rationale.map((n) => (
            <blockquote key={n.seq} className="quote step-why">
              {n.text}
            </blockquote>
          ))}
          <p className="muted small">The model's own explanation. It may not match what it computed.</p>
        </div>
      )}
      <details className="step-tech">
        <summary>
          Technical details <IconChevron className="chev" aria-hidden="true" />
        </summary>
        <div className="step-tech-body">
          <dl className="kv">
            <dt>Tool</dt>
            <dd>
              <code>{call.tool}</code>
            </dd>
            <dt>Call</dt>
            <dd>
              <code>{call.call_id}</code>
            </dd>
            {call.started_seq != null && (
              <>
                <dt>Event</dt>
                <dd className="num">{call.started_seq}</dd>
              </>
            )}
            {r && (
              <>
                <dt>Execution</dt>
                <dd>
                  {words(r.execution_status)} <span className="muted">{EXEC_NOTE[r.execution_status]}</span>
                </dd>
                <dt>Science</dt>
                <dd>
                  {words(r.scientific_status)} <span className="muted">{SCI_NOTE[r.scientific_status]}</span>
                </dd>
                <dt>Origin</dt>
                <dd>{words(r.origin)}</dd>
                <dt>Contract</dt>
                <dd>
                  <code>{r.schema_version}</code>
                </dd>
              </>
            )}
            {r?.error && (
              <>
                <dt>Error</dt>
                <dd>
                  <code>{r.error.code}</code> <span className="muted">retry {words(r.error.retry.kind)}</span>
                </dd>
                <dt>Message</dt>
                <dd className="step-raw">{r.error.message}</dd>
              </>
            )}
            {r?.job && (
              <>
                <dt>Job</dt>
                <dd>
                  <code>{r.job.job_id}</code> ({words(r.job.kind)})
                </dd>
              </>
            )}
            {allRefs.map((a) => (
              <div key={a.artifact_id} style={{ display: "contents" }}>
                <dt>Artifact</dt>
                <dd>
                  <button className="linkish" onClick={() => onArtifact(a)}>
                    {a.contract ?? a.media_type} · {shortId(a.artifact_id)}
                  </button>
                </dd>
              </div>
            ))}
          </dl>
          {call.arguments ? (
            <>
              <Payload payload={call.arguments.value} title="Arguments" onArtifact={onArtifact} />
              {(call.arguments.redacted ?? []).length > 0 && (
                <p className="muted small">
                  Removed by scope before recording: {call.arguments.redacted!.map((p) => <code key={p}>{p} </code>)}
                </p>
              )}
            </>
          ) : (
            <p className="muted small">Arguments not recorded for this call: the runtime sent none with it.</p>
          )}
          {r?.result && <Payload payload={r.result} onArtifact={onArtifact} />}
        </div>
      </details>
    </>
  );
}

// ---------------------------------------------------------------- release

/** A model, as a researcher reads it: what it forecasts, what it reads, how it was built and how it
 * scored. Ids, features, construction and build checks wait behind "Technical details". */
function ReleaseItem({ item }: { item: Extract<PanelItem, { kind: "release" }> }) {
  const { synthetic } = useData();
  const r = item.release;
  const card = useServedCard(r.id);
  const trained = r.trained_on === "not_trained" ? "hand-written (literal weights)" : r.trained_on === "synthetic" ? "synthetic data" : r.trained_on;
  const limits = card ? readerLimits(card) : [];
  return (
    <>
      <div className="model-head">
        <p className="model-name">{modelTitle(r.model_name, card)}</p>
        <div className="model-tags">
          <Qualified basis={card?.qualification.basis} />
          {r.construction && <OriginMark origin={r.construction.origin} pageSynthetic={synthetic} />}
        </div>
        {card && <p className="model-sum">{card.summary}</p>}
        <p className="model-by">
          <span className="glyph g-predicted" aria-hidden="true" /> {writtenBy(r)}
        </p>
      </div>

      {card && card.scores.length > 0 && (
        <div className="panel-sec">
          <h3>How it scored</h3>
          <ul className="model-scores">
            {card.scores.map((s) => {
              const p = scoreParts(s);
              return (
                <li key={s.metric}>
                  <span className="model-score-v">
                    <b className="num">{p.value}</b>
                    {p.vs && <span className="muted"> vs {p.vs}</span>}
                  </span>
                  <span className="model-score-m">{s.metric}</span>
                </li>
              );
            })}
          </ul>
          <p className="muted small">Lower is better. {card.qualification.basis}.</p>
        </div>
      )}

      {card && (
        <div className="panel-sec">
          <h3>What it reads</h3>
          <ul className="model-inputs">
            {card.inputs.map((i) => (
              <li key={i.name}>{inputWords(i.note, i.name)}</li>
            ))}
          </ul>
          <p className="small muted">{card.stored_runs ? "It forecasts a stored NASA run directly, or conditions you type in the forecast lab." : "You type its conditions in the forecast lab."}</p>
        </div>
      )}

      <div className="panel-sec">
        <h3>How it was built</h3>
        <p className="model-para">{builtParagraph(card, r)}</p>
      </div>

      {limits.length > 0 && (
        <div className="panel-sec">
          <h3>Where it stops</h3>
          <ul className="model-limits">
            {limits.map((l) => (
              <li key={l}>{l}</li>
            ))}
          </ul>
        </div>
      )}

      <details className="step-tech">
        <summary>
          Technical details <IconChevron className="chev" aria-hidden="true" />
        </summary>
        <div className="step-tech-body">
          <dl className="kv">
            <dt>Model</dt>
            <dd>
              <code>
                {r.model_name}
                {card ? ` ${card.candidate}` : ""}
              </code>
            </dd>
            {card && (
              <>
                <dt>Investigation</dt>
                <dd>{card.investigation}</dd>
                <dt>Authorship</dt>
                <dd>
                  {card.authorship === "zero_shot" ? "Zero-shot" : "Data-informed"}: {card.authorship_note}
                </dd>
                <dt>Forecasts</dt>
                <dd>{card.forecasts}</dd>
                <dt>Inputs</dt>
                <dd>{card.inputs_note}</dd>
                <dt>Cite as</dt>
                <dd>{card.cite}</dd>
                {card.card_doc && (
                  <>
                    <dt>Model card</dt>
                    <dd>
                      <code>{card.card_doc}</code>
                    </dd>
                  </>
                )}
              </>
            )}
          </dl>
          {card && card.limits.length > limits.length && (
            <ul className="model-limits small">
              {card.limits
                .filter((l) => !limits.includes(l))
                .map((l) => (
                  <li key={l}>{l}</li>
                ))}
            </ul>
          )}
          <dl className="kv">
            {card && card.scores.length > 0 && (
              <>
                <dt>Scores from</dt>
                <dd>
                  {[...new Set(card.scores.map((s) => s.source))].map((s) => (
                    <code key={s} style={{ display: "block" }}>
                      {s}
                    </code>
                  ))}
                </dd>
              </>
            )}
        <dt>Family</dt>
        <dd>{FAMILY_LABEL[r.family] ?? r.family}</dd>
        <dt>Trained on</dt>
        <dd>{trained}</dd>
        <dt>Lineage</dt>
        <dd>{r.lineage ? words(r.lineage) : "not recorded"}</dd>
        <dt>Features</dt>
        <dd>{r.feature_order.join(", ") || "none"}</dd>
        <dt>Baseline</dt>
        <dd>{r.is_baseline ? "yes" : "no"}</dd>
        <dt>Release</dt>
        <dd>
          <code>{r.id}</code>
        </dd>
          </dl>
          <ReleaseConstruction release={r} />
        </div>
      </details>
    </>
  );
}

/** A NASA original inside the panel (the page stays where it is): the PDF page with the cited region,
 * the video at its time, or the table record, streamed through SourceAccess. */
function OriginalItem({ sourceId, loc, title }: { sourceId: string; loc: SourceLocator | null; title: string | null }) {
  const { client, researchMode } = useData();
  const access = useQuery({
    queryKey: [researchMode, "access", sourceId],
    queryFn: () => client.sourceAccess(sourceId),
    retry: false,
    staleTime: (q) => {
      const a = q.state.data;
      return a?.expires_at ? Math.max(0, Date.parse(a.expires_at) - Date.now() - 30_000) : 5 * 60_000;
    },
  });
  const a = access.data;
  const l: SourceLocator = loc ?? { asset_sha256: sourceId.replace(/^sha256:/, "") };
  return (
    <div className="panel-sec">
      <div className="toolbar" style={{ flexWrap: "wrap" }}>
        <OpenOriginal sourceId={sourceId} loc={loc} title={title} label="New tab" className="btn btn-s btn-quiet" />
        {a && httpUrl(a.landing_url) && (
          <a className="btn btn-s" href={a.landing_url} target="_blank" rel="noopener noreferrer" title={ORIGINAL_MODE_TEXT[a.mode]}>
            Open at NASA <IconExternal width={13} height={13} />
          </a>
        )}
      </div>
      {access.isPending && (locatorKind(l) === "page" ? <PdfSkeleton label="Finding the original…" /> : <p className="muted">Finding the original…</p>)}
      {access.error && <ErrorNote error={access.error} title="The original cannot be opened from here" />}
      {a && <OriginalBody a={a} loc={l} kind={locatorKind(l)} title={title ?? l.path ?? sourceId} />}
    </div>
  );
}
