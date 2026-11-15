// The side panel (Linear peek / Slack thread): the selected run, claim, source, figure, route step
// or release, with a back trail. Every link inside it pushes onto the trail; Escape closes it.
import { useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import type { AnswerRefusal, ArtifactRef, Claim, EvidenceRef, InlinePayload, ResearchAnswer, SourceLocator } from "../contracts/generated";
import { call, RefusalError, type ExperimentDetail } from "../api";
import { asRefusal } from "../hooks";
import { bboxStyle, locatorKind, pageCaption, sliceSpan } from "../research/units";
import { fmtGravity } from "../format";
import { ArtifactValue, EvidenceSummary, locatorText, placeShort } from "../views/Answer";
import { CLAIM_LABEL, ClaimGlyph, ErrorNote, EvidenceClassTag, fmtBytes, Gap, OriginMark, RefusalNote, ReleaseState, shortId, Synthetic, words } from "../ui/common";
import { useCompare } from "./compare";
import { INVESTIGATIONS } from "../data/investigations";
import { conditionRows, gapText } from "../research/runConditions";
import { useData } from "./data";
import { itemKey, usePanel, type PanelItem } from "./inspector";
import { FamilyMark, FAMILY_LABEL, IconBack, IconClose } from "./icons";
import { href } from "./router";
import { TOOL_LABEL } from "../views/turns";
import { isTypedArtifact } from "../research/artifactViews";
import { RefusalProblems, ReleaseConstruction, TypedArtifactBody } from "./ArtifactViews";
import { SourceOriginal, httpUrl } from "./SourceOriginal";
import { MODE_TEXT as ORIGINAL_MODE_TEXT, OriginalBody } from "../views/Original";
import { OpenOriginal } from "./OpenOriginal";
import { runSources } from "./runSources";
import { JsonView } from "./JsonView";

const TITLE: Record<PanelItem["kind"], string> = { run: "Run", evidence: "Source", claim: "Claim", artifact: "Figure", step: "Step", release: "Model release", sources: "Sources", investigation: "Investigation", original: "NASA original" };

const ARTIFACT_TITLE: Record<string, string> = { AnalysisArtifact: "Figure", ExecutionRecord: "Analysis run", DatasetBundle: "Dataset", EvaluationResult: "Evaluation", ResearchAnswer: "Answer", AnswerRefusal: "Refused answer", VisualObservationBundle: "Video contours" };

function itemTitle(i: PanelItem): string {
  switch (i.kind) {
    case "run":
      return `Run ${i.id.split(":").pop()}`;
    case "claim":
      return `Claim ${i.keyLabel}`;
    case "step":
      return `Step ${i.index}: ${TOOL_LABEL[i.call.tool] ?? words(i.call.tool)}`;
    case "release":
      return i.release.model_name;
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
          {itemTitle(current)}
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

/** The NASA files this run's record was read from, each opened at the place the record cites. */
function RunOriginals({ record }: { record: unknown }) {
  const files = runSources(record);
  if (files.length === 0) return null;
  return (
    <div className="panel-sec">
      <h3>NASA originals</h3>
      <ul style={{ display: "grid", gap: 6, fontSize: 13, paddingLeft: 0, listStyle: "none" }}>
        {files.map((f) => (
          <li key={f.sourceId} style={{ display: "flex", gap: 8, alignItems: "baseline", justifyContent: "space-between" }}>
            <span style={{ overflowWrap: "anywhere" }}>
              {f.path ?? <code>{f.sourceId.slice(0, 19)}…</code>}
              <span className="muted small"> · {f.uses.slice(0, 4).join(", ")}{f.uses.length > 4 ? ", …" : ""}</span>
            </span>
            <OpenOriginal sourceId={f.sourceId} loc={f.loc} title={f.path} />
          </li>
        ))}
      </ul>
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
  const rows = conditionRows(summary.conditions);
  const gaps = summary.gaps.flatMap((g) => gapText(g) ?? []);
  return (
    <>
      <div className="panel-sec">
        <div className="toolbar">
          <span className="tag">
            <FamilyMark family={summary.family} size={9} /> {FAMILY_LABEL[summary.family] ?? summary.family}
          </span>
          {summary.synthetic && <Synthetic label="Synthetic run" />}
        </div>
        <p style={{ fontSize: 14 }}>
          {[summary.fuel_or_material, summary.configuration?.replace(/_/g, " "), summary.investigation_id].filter(Boolean).join(" · ")}
        </p>
        <div className="toolbar">
          <button className="btn btn-s" onClick={() => cmp.toggle(id)} aria-pressed={cmp.has(id)}>
            {cmp.has(id) ? "Remove from compare" : "Add to compare"}
          </button>
          {releases.length > 0 && (
            <a className="btn btn-s" href={href("lab", id)}>
              Forecast
            </a>
          )}
          <a className="btn btn-s btn-primary" href={href("research", null, { new: "1", q: askAboutRun(id, summary) })}>
            Ask about this run
          </a>
        </div>
      </div>
      <div className="panel-sec">
        <h3>Conditions</h3>
        <dl className="kv">
          {rows.map((r) => (
            <div key={r.key} style={{ display: "contents" }}>
              <dt>{r.label}</dt>
              <dd className="num">{r.text}</dd>
            </div>
          ))}
          {summary.gravity?.value_si != null && (
            <>
              <dt>Gravity</dt>
              <dd className="num">{fmtGravity(summary.gravity.value_si)}</dd>
            </>
          )}
        </dl>
      </div>
      <RunOriginals record={record} />
      {summary.quantities.length > 0 && (
        <div className="panel-sec">
          <h3>Measured series</h3>
          <dl className="kv">
            {summary.quantities.map((qq) => (
              <div key={qq.quantity} style={{ display: "contents" }}>
                <dt>{words(qq.quantity)}</dt>
                <dd className="num">
                  {qq.count} points · {qq.archive_t_min_s.toFixed(2)}–{qq.archive_t_max_s.toFixed(2)} s
                </dd>
              </div>
            ))}
          </dl>
        </div>
      )}
      {releases.length > 0 && (
        <div className="panel-sec">
          <h3>Models</h3>
          {releases.map((r) => (
            <div key={r.id} className="stat-line">
              <span>{r.model_name}</span>
              <ReleaseState state={r.state} />
            </div>
          ))}
        </div>
      )}
      {gaps.length > 0 && (
        <div className="panel-sec">
          <h3>Not available</h3>
          <ul style={{ display: "grid", gap: 4, fontSize: 13, paddingLeft: 18, color: "var(--ink-2)" }}>
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
          <dt>source</dt>
          <dd>
            <code>{evidence.source_id}</code>
          </dd>
          {evidence.parse_view_id && (
            <>
              <dt>parse view</dt>
              <dd>
                <code>{evidence.parse_view_id}</code>
              </dd>
            </>
          )}
          <dt>locator</dt>
          <dd>{locatorText(evidence)}</dd>
          {loc?.char_start != null && (
            <>
              <dt>characters</dt>
              <dd className="num">
                {loc.char_start}–{loc.char_end} ({loc.offset_unit ? words(loc.offset_unit) : "unit not declared: not citable"})
              </dd>
            </>
          )}
          {loc && (
            <>
              <dt>asset</dt>
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
        <dt>artifact</dt>
        <dd>
          <code>{artifact.artifact_id}</code>
        </dd>
        <dt>size</dt>
        <dd>
          {fmtBytes(artifact.byte_length)} · {artifact.media_type}
        </dd>
        <dt>created</dt>
        <dd className="num">{artifact.created_at}</dd>
        {artifact.produced_by?.job_id && (
          <>
            <dt>job</dt>
            <dd>
              <code>{artifact.produced_by.job_id}</code>
            </dd>
          </>
        )}
        {artifact.produced_by?.release_id && (
          <>
            <dt>release</dt>
            <dd>
              <code>{shortId(artifact.produced_by.release_id, 26)}</code>
            </dd>
          </>
        )}
        <dt>sha256</dt>
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

function StepItem({ item }: { item: Extract<PanelItem, { kind: "step" }> }) {
  const panel = usePanel();
  const { synthetic } = useData();
  const r = item.call.result;
  // A refused answer_submit carries its problems as an AnswerRefusal artifact (M7); the error message
  // repeats them as text, so it is shown only when that artifact cannot be read.
  const refusalRef = r?.tool === "answer_submit" && r.execution_status === "refused" ? (r.artifact_refs ?? []).find((a) => a.contract === "AnswerRefusal") ?? null : null;
  return (
    <>
      <div className="panel-sec">
        <p style={{ fontSize: 13 }}>
          Tool <code>{item.call.tool}</code> · call <code>{shortId(item.call.call_id)}</code>
          {item.call.started_seq != null && <span className="muted"> · event {item.call.started_seq}</span>}
        </p>
        {!r && <p className="muted">Running: no result yet.</p>}
        {r && (
          <dl className="kv">
            <dt>execution</dt>
            <dd>
              <b>{words(r.execution_status)}</b> <span className="muted">{EXEC_NOTE[r.execution_status]}</span>
            </dd>
            <dt>science</dt>
            <dd>
              <b>{words(r.scientific_status)}</b> <span className="muted">{SCI_NOTE[r.scientific_status]}</span>
            </dd>
          </dl>
        )}
        {r && <OriginMark origin={r.origin} pageSynthetic={synthetic} />}
      </div>
      <div className="panel-sec">
        <h3>Arguments</h3>
        {item.call.arguments ? (
          <>
            <Payload payload={item.call.arguments.value} title={null} onArtifact={(a) => panel.push({ kind: "artifact", artifact: a })} />
            {(item.call.arguments.redacted ?? []).length > 0 && (
              <p className="muted small">
                Removed by scope before recording: {item.call.arguments.redacted!.map((p) => <code key={p}>{p} </code>)}
              </p>
            )}
          </>
        ) : (
          <p className="muted small">Not recorded for this call: the runtime sent no arguments with it.</p>
        )}
      </div>
      {r?.job && (
        <div className="panel-sec">
          <h3>Job</h3>
          <dl className="kv">
            <dt>job</dt>
            <dd>
              <code>{r.job.job_id}</code> ({words(r.job.kind)})
            </dd>
            <dt>status</dt>
            <dd>{words(r.job.status)}</dd>
            {r.job.progress && (
              <>
                <dt>progress</dt>
                <dd className="num">
                  {r.job.progress.completed}
                  {r.job.progress.total != null ? ` of ${r.job.progress.total}` : " (total unknown)"} {r.job.progress.unit}
                </dd>
              </>
            )}
          </dl>
        </div>
      )}
      {r?.result && <Payload payload={r.result} onArtifact={(a) => panel.push({ kind: "artifact", artifact: a })} />}
      {refusalRef && <SubmissionRefusal refusalRef={refusalRef} fallback={r?.error?.message ?? null} />}
      {r?.error && !refusalRef && (
        <div className="note note-refusal">
          <b>
            Tool error <code>{r.error.code}</code>
          </b>
          <p>{r.error.message}</p>
        </div>
      )}
      {(r?.artifact_refs ?? []).length > 0 && (
        <div className="panel-sec">
          <h3>Artifacts</h3>
          {r!.artifact_refs!.map((a) => (
            <button key={a.artifact_id} className="linkish" onClick={() => panel.push({ kind: "artifact", artifact: a })}>
              {a.contract ?? a.media_type} · {shortId(a.artifact_id)}
            </button>
          ))}
        </div>
      )}
      <div className="panel-sec">
        <h3>Why this step</h3>
        {item.call.rationale.length === 0 ? (
          <p className="muted small">The model gave no explanation for this call.</p>
        ) : (
          <>
            {item.call.rationale.map((n) => (
              <blockquote key={n.seq} className="quote">
                {n.text}
              </blockquote>
            ))}
            <p className="muted small">The model's own explanation of this step. It may not match what the model computed.</p>
          </>
        )}
      </div>
    </>
  );
}

// ---------------------------------------------------------------- release

function ReleaseItem({ item }: { item: Extract<PanelItem, { kind: "release" }> }) {
  const { synthetic } = useData();
  const r = item.release;
  const trained = r.trained_on === "not_trained" ? "not trained (literal weights)" : r.trained_on === "synthetic" ? "synthetic data" : r.trained_on;
  return (
    <>
      <div className="toolbar">
        <ReleaseState state={r.state} />
        {r.construction && <OriginMark origin={r.construction.origin} pageSynthetic={synthetic} />}
      </div>
      {!["measurement_qualified", "forecast_qualified", "event_qualified"].includes(r.state ?? "") && (
        <p style={{ fontSize: 13 }}>No evaluation supports this release's predictions: show its outputs as experimental model output only.</p>
      )}
      <dl className="kv">
        <dt>family</dt>
        <dd>{FAMILY_LABEL[r.family] ?? r.family}</dd>
        <dt>trained on</dt>
        <dd>{trained}</dd>
        <dt>lineage</dt>
        <dd>{r.lineage ? words(r.lineage) : "not recorded"}</dd>
        <dt>features</dt>
        <dd>{r.feature_order.join(", ") || "none"}</dd>
        <dt>baseline</dt>
        <dd>{r.is_baseline ? "yes" : "no"}</dd>
        <dt>release</dt>
        <dd>
          <code>{r.id}</code>
        </dd>
      </dl>
      <ReleaseConstruction release={r} />
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
        {a && <span className="muted small">{ORIGINAL_MODE_TEXT[a.mode]}</span>}
        <OpenOriginal sourceId={sourceId} loc={loc} title={title} label="Originals tab ↗" />
        {a && httpUrl(a.landing_url) && (
          <a className="btn btn-s" href={a.landing_url} target="_blank" rel="noopener noreferrer">
            Open at NASA ↗
          </a>
        )}
      </div>
      {access.isPending && <p className="muted">Finding the original…</p>}
      {access.error && <ErrorNote error={access.error} title="The original cannot be opened from here" />}
      {a && <OriginalBody a={a} loc={l} kind={locatorKind(l)} title={title ?? l.path ?? sourceId} />}
    </div>
  );
}
