// The side panel (Linear peek / Slack thread): the selected run, claim, source, figure, route step
// or release, with a back trail. Every link inside it pushes onto the trail; Escape closes it.
import { useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ArtifactRef, EvidenceRef, InlinePayload } from "../contracts/generated";
import { call, RefusalError, type ExperimentDetail } from "../api";
import { asRefusal } from "../hooks";
import { bboxStyle, locatorKind, pageCaption, sliceSpan } from "../research/units";
import { fmtGravity, fmtMeasured } from "../format";
import { ArtifactValue, EvidenceSummary, locatorText } from "../views/Answer";
import { CLAIM_LABEL, ClaimGlyph, ErrorNote, EvidenceClassTag, fmtBytes, Gap, OriginMark, RefusalNote, ReleaseState, shortId, Synthetic, words } from "../ui/common";
import { useCompare } from "./compare";
import { useData } from "./data";
import { usePanel, type PanelItem } from "./inspector";
import { FamilyMark, FAMILY_LABEL, IconBack, IconClose } from "./icons";
import { href } from "./router";
import { TOOL_LABEL } from "../views/turns";

const TITLE: Record<PanelItem["kind"], string> = { run: "Run", evidence: "Source", claim: "Claim", artifact: "Figure", step: "Route step", release: "Model release" };

function itemTitle(i: PanelItem): string {
  switch (i.kind) {
    case "run":
      return `Run ${i.id}`;
    case "claim":
      return `Claim ${i.keyLabel}`;
    case "step":
      return `Step ${i.index}: ${TOOL_LABEL[i.call.tool] ?? words(i.call.tool)}`;
    case "release":
      return i.release.model_name;
    default:
      return TITLE[i.kind];
  }
}

export function Panel() {
  const { current, stack, back, close } = usePanel();
  const body = useRef<HTMLDivElement>(null);
  useEffect(() => {
    body.current?.scrollTo({ top: 0 });
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
        <h2>{itemTitle(current)}</h2>
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
      </div>
    </aside>
  );
}

// ---------------------------------------------------------------- run

function RunItem({ id }: { id: string }) {
  const { origin, synthetic } = useData();
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
  return (
    <>
      <div className="panel-sec">
        <div className="toolbar">
          <span className="tag">
            <FamilyMark family={summary.family} size={9} /> {FAMILY_LABEL[summary.family] ?? summary.family}
          </span>
          {summary.synthetic && !synthetic && <Synthetic />}
          {summary.synthetic && synthetic && <Synthetic label="Synthetic run" />}
        </div>
        <p style={{ fontSize: 13 }}>
          {summary.fuel_or_material ?? "Material unknown"} · {summary.configuration ?? "configuration unknown"} · investigation {summary.investigation_id}
        </p>
        <div className="toolbar">
          <a className="btn btn-s btn-primary" href={href("lab", id)}>
            Open in forecast lab
          </a>
          <button className="btn btn-s" onClick={() => cmp.toggle(id)} aria-pressed={cmp.has(id)}>
            {cmp.has(id) ? "Remove from compare" : "Add to compare"}
          </button>
        </div>
      </div>
      <div className="panel-sec">
        <h3>Conditions (SI, as recorded)</h3>
        <dl className="kv">
          <dt>gravity</dt>
          <dd>{fmtGravity(summary.gravity?.value_si)}</dd>
          {Object.entries(summary.conditions).map(([k, c]) => (
            <div key={k} style={{ display: "contents" }}>
              <dt>{words(k)}</dt>
              <dd>
                {c.value_si == null ? <span className="tag tag-synthetic" style={{ background: "none" }}>{words(c.resolution)}</span> : fmtMeasured(c.value_si, c.unit_si)}
                {c.value_si != null && c.resolution !== "resolved" && <span className="muted"> ({words(c.resolution)})</span>}
              </dd>
            </div>
          ))}
        </dl>
      </div>
      <div className="panel-sec">
        <h3>Measurements</h3>
        <dl className="kv">
          {summary.quantities.map((qq) => (
            <div key={qq.quantity} style={{ display: "contents" }}>
              <dt>{words(qq.quantity)}</dt>
              <dd className="num">
                {qq.count} points · {qq.archive_t_min_s.toFixed(2)}–{qq.archive_t_max_s.toFixed(2)} s
              </dd>
            </div>
          ))}
          <dt>endpoint</dt>
          <dd>{summary.has_endpoint ? "recorded" : "missing"}</dd>
          <dt>time origin</dt>
          <dd>{q.data.control_history?.time_origin ?? "not adjudicated"}</dd>
        </dl>
      </div>
      {summary.gaps.length > 0 && (
        <div className="panel-sec">
          <h3>Evidence gaps</h3>
          <ul style={{ display: "grid", gap: 4, fontSize: 13, paddingLeft: 18 }}>
            {summary.gaps.map((g) => (
              <li key={g}>{g}</li>
            ))}
          </ul>
        </div>
      )}
      <div className="panel-sec">
        <h3>Model releases for this family</h3>
        {releases.length === 0 && <p className="muted" style={{ fontSize: 13 }}>None: no forecast can be drawn for this run.</p>}
        {releases.map((r) => (
          <div key={r.id} className="stat-line">
            <span>{r.model_name}</span>
            <ReleaseState state={r.state} />
          </div>
        ))}
      </div>
      <details>
        <summary className="muted" style={{ fontSize: 12, cursor: "pointer" }}>
          Full record (JSON)
        </summary>
        <pre className="json">{JSON.stringify(record, null, 2)}</pre>
      </details>
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
      {kind === "page" && loc && (
        <div className="panel-sec">
          <h3>{pageCaption(loc)}</h3>
          <div className="page-frame" aria-label={`Text of ${pageCaption(loc)} with the cited region marked`}>
            {read.isPending && <p className="muted">Reading the page…</p>}
            {read.error && <ErrorNote error={read.error} title="The page could not be read" />}
            {seg && <p>{seg.text}</p>}
            {bboxStyle(loc.bbox) && <div className="bbox" style={bboxStyle(loc.bbox)!} title="Cited region" />}
          </div>
          {read.data && !read.data.complete && <p className="muted" style={{ fontSize: 12 }}>Partial read: the source continues on later pages.</p>}
          <p className="muted" style={{ fontSize: 12 }}>The page shows the parse-view text; the box marks the cited region. The PDF renderer is not connected yet.</p>
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
};

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
        {(claim.support ?? []).length === 0 && <Gap title="No support">{claim.kind === "interpretation" ? "This is the agent's interpretation only." : "Treat this claim as unverified."}</Gap>}
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
        {q.data != null && <pre className="json">{JSON.stringify(q.data, null, 2)}</pre>}
      </details>
    </>
  );
}

// ---------------------------------------------------------------- route step

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

function Payload({ payload, onArtifact }: { payload: InlinePayload; onArtifact: (a: ArtifactRef) => void }) {
  const v = payload.value as unknown;
  const flat = v && typeof v === "object" && !Array.isArray(v) && Object.values(v as object).every((x) => x === null || ["string", "number", "boolean"].includes(typeof x));
  return (
    <div className="panel-sec">
      <h3>Result</h3>
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
        <pre className="json">{JSON.stringify(v, null, 2)}</pre>
      )}
    </div>
  );
}

function StepItem({ item }: { item: Extract<PanelItem, { kind: "step" }> }) {
  const panel = usePanel();
  const { synthetic } = useData();
  const r = item.call.result;
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
      <Gap title="Arguments">The harness does not record what this call was given (for example the search query or page range). Only the tool name and its result are in the event log.</Gap>
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
      {r?.error && (
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
      <Gap title="Why this step">
        The model's reason for this call is not in the event contract yet. When the harness records it, it appears here, marked as the model's own explanation (it may not match what the model
        computed).
      </Gap>
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
    </>
  );
}
