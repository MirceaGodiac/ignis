// Presentational pieces of the Research workspace. Everything renders discriminated on contract
// fields (`body.kind`, `execution_status` x `scientific_status`, `Claim.kind`, `units_status`).

import { useQuery } from "@tanstack/react-query";
import type {
  ArtifactRef,
  Claim,
  ClaimSupport,
  ErrorEnvelope,
  EvidenceRef,
  InlinePayload,
  JobRef,
  ModelRelease,
  Origin,
  ResearchAnswer,
  ResearchEvent,
  SearchPage,
  ToolResult,
  VisualObservationBundle,
} from "../contracts/generated";
import { ApiError, type ResearchClient } from "./client";
import { bboxStyle, locatorKind, pageCaption, sliceSpan, spatialAxis } from "./units";

// ------------------------------------------------------------------------------------------ small

export function OriginBadge({ origin }: { origin: Origin }) {
  return origin === "synthetic_fixture" ? (
    <span className="badge badge-synthetic" title="Hand-made contract example: not NASA data, not a result">
      synthetic fixture
    </span>
  ) : null;
}

export function ErrorBox({ error, title }: { error: unknown; title?: string }) {
  const env: ErrorEnvelope | null = error instanceof ApiError ? error.envelope : null;
  return (
    <div className="refusal" role="status">
      <strong>{title ?? "Request failed"}</strong> {env && <code>{env.code}</code>}
      {error instanceof ApiError && error.httpStatus != null && <span className="muted"> HTTP {error.httpStatus}</span>}
      <p>{env ? env.message : String(error)}</p>
      {env && <p className="muted">Retry: {retryText(env)}</p>}
    </div>
  );
}

function retryText(e: ErrorEnvelope): string {
  switch (e.retry.kind) {
    case "never":
      return "do not retry";
    case "after_delay":
      return `after ${e.retry.seconds} s`;
    case "after_user_action":
      return "after a user action";
    case "reconcile":
      return "effect uncertain: inspect state before any retry";
  }
}

const fmtBytes = (n: number) => (n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(1)} KiB` : `${(n / 1048576).toFixed(1)} MiB`);
const short = (id: string) => (id.length > 24 ? `${id.slice(0, 18)}…${id.slice(-4)}` : id);

/** RFC 6901 JSON Pointer. Returns undefined when the path does not exist. */
export function resolvePointer(doc: unknown, pointer: string): unknown {
  if (pointer === "") return doc;
  if (!pointer.startsWith("/")) return undefined;
  let cur: unknown = doc;
  for (const raw of pointer.slice(1).split("/")) {
    const key = raw.replace(/~1/g, "/").replace(/~0/g, "~");
    if (Array.isArray(cur)) cur = /^(0|[1-9]\d*)$/.test(key) ? cur[Number(key)] : undefined;
    else if (cur && typeof cur === "object") cur = (cur as Record<string, unknown>)[key];
    else return undefined;
    if (cur === undefined) return undefined;
  }
  return cur;
}

// ------------------------------------------------------------------------------------- tool states

const EXEC_LABEL: Record<ToolResult["execution_status"], string> = {
  completed: "executed",
  accepted: "accepted — running as a job",
  refused: "refused",
  failed: "failed (technical)",
  cancelled: "cancelled",
};
const SCI_LABEL: Record<ToolResult["scientific_status"], string> = {
  not_assessed: "not assessed",
  conclusive: "conclusive",
  inconclusive: "inconclusive",
  insufficient_evidence: "insufficient evidence",
  out_of_domain: "out of domain",
};

/** Execution and scientific outcome are separate axes: `completed` + `insufficient_evidence` is a valid result, not an error. */
export function ToolResultCard({ result, tool, onArtifact }: { result: ToolResult | null; tool: string; onArtifact?: (a: ArtifactRef) => void }) {
  if (!result)
    return (
      <div className="tool tool-running">
        <b>{tool}</b> <span className="badge">started — no result yet</span>
      </div>
    );
  const exec = result.execution_status;
  const cls = exec === "failed" || exec === "refused" ? "refused" : exec === "completed" ? "ok" : "";
  return (
    <div className={`tool card-inner ${cls}`}>
      <div className="exp-head">
        <b>{result.tool}</b>
        <span className={`badge ${cls ? `badge-${cls}` : ""}`}>execution: {EXEC_LABEL[exec]}</span>
        <span className={`badge ${result.scientific_status === "conclusive" ? "badge-ok" : ""}`}>science: {SCI_LABEL[result.scientific_status]}</span>
        <OriginBadge origin={result.origin} />
      </div>
      {result.job && <JobLine job={result.job} />}
      {result.result && <InlineResult payload={result.result} onArtifact={onArtifact} />}
      {result.error && <ErrorBox error={new ApiError(result.error, null)} title="Tool error" />}
      {(result.artifact_refs ?? []).length > 0 && (
        <p className="muted">
          Artifacts:{" "}
          {result.artifact_refs!.map((a) => (
            <button key={a.artifact_id} className="link" onClick={() => onArtifact?.(a)}>
              {short(a.artifact_id)}
            </button>
          ))}
        </p>
      )}
    </div>
  );
}

function InlineResult({ payload, onArtifact }: { payload: InlinePayload; onArtifact?: (a: ArtifactRef) => void }) {
  return (
    <div className="inline-result">
      {payload.partial && (
        <p className="warn">
          Preview only (partial).{" "}
          {payload.full && (
            <button className="link" onClick={() => onArtifact?.(payload.full!)}>
              Open the full artifact {short(payload.full.artifact_id)}
            </button>
          )}
        </p>
      )}
      <pre className="json">{JSON.stringify(payload.value, null, 2)}</pre>
    </div>
  );
}

export function JobLine({ job }: { job: JobRef }) {
  const p = job.progress;
  return (
    <p className="status-line">
      Job <code>{job.job_id}</code> ({job.kind}): <b>{job.status}</b>
      {p && ` — ${p.completed}${p.total != null ? ` / ${p.total}` : " / total unknown"} ${p.unit ?? ""}`}
      {job.error && <span className="warn"> — {job.error.code}: {job.error.message}</span>}
    </p>
  );
}

// ---------------------------------------------------------------------------------------- timeline

export function eventSummary(e: ResearchEvent): string {
  const b = e.body;
  switch (b.kind) {
    case "episode.status_changed":
      return `status ${b.payload.from} → ${b.payload.to}${b.payload.reason ? ` (${b.payload.reason})` : ""}`;
    case "message.user":
      return `user message ${b.payload.message_id}`;
    case "message.delta":
      return `assistant text (partial) ${b.payload.message_id}`;
    case "message.completed":
      return `assistant message complete ${b.payload.message_id}`;
    case "tool.started":
      return `tool ${b.payload.tool} started (${b.payload.call_id})`;
    case "tool.finished":
      return `tool ${b.payload.result.tool}: ${b.payload.result.execution_status} / ${b.payload.result.scientific_status}`;
    case "job.updated":
      return `job ${b.payload.job.job_id}: ${b.payload.job.status}`;
    case "artifact.created":
      return `artifact ${b.payload.artifact.artifact_id} (${b.payload.artifact.contract ?? b.payload.artifact.media_type})`;
    case "approval.requested":
      return `approval requested: ${b.payload.action} [${b.payload.effect}]`;
    case "approval.resolved":
      return `approval ${b.payload.approval_id}: ${b.payload.decision}`;
    case "answer.completed":
      return `answer ${b.payload.answer_id}: ${b.payload.status}`;
    case "runtime.opaque":
      return `runtime event ${b.payload.method} (archived, not interpreted)`;
    default: {
      // Unknown kinds from a newer server: shown neutrally, never interpreted.
      const k = (b as { kind?: string }).kind ?? "unknown";
      return `${k} (unknown event kind)`;
    }
  }
}

// ----------------------------------------------------------------------------------------- answer

const CLAIM_LABEL: Record<Claim["kind"], string> = {
  source_reports: "source reports",
  observed: "observed",
  computed: "computed by IGNIS",
  predicted: "model prediction",
  interpretation: "interpretation",
};

export function AnswerView({
  client,
  answer,
  onEvidence,
  onArtifact,
}: {
  client: ResearchClient;
  answer: ResearchAnswer;
  onEvidence: (e: EvidenceRef) => void;
  onArtifact: (a: ArtifactRef) => void;
}) {
  const claims = new Map((answer.claims ?? []).map((c) => [c.claim_id, c]));
  const evidence = new Map((answer.evidence ?? []).map((e) => [e.evidence_id, e]));
  const artifacts = new Map((answer.artifacts ?? []).map((a) => [a.artifact_id, a]));
  return (
    <div>
      <div className="exp-head">
        <span className={`badge ${answer.status === "complete" ? "badge-ok" : "badge-refused"}`}>answer {answer.status.replace("_", " ")}</span>
        <OriginBadge origin={answer.origin} />
      </div>
      {answer.blocks.map((b, i) =>
        b.type === "text" ? (
          <p key={i}>
            {b.text}{" "}
            {(b.claim_ids ?? []).map((id) => {
              const c = claims.get(id);
              return (
                <a key={id} href={`#claim-${id}`} className={`claim-chip claim-${c?.kind ?? "missing"}`}>
                  {c ? CLAIM_LABEL[c.kind] : "claim not found"}
                </a>
              );
            })}
          </p>
        ) : (
          <div key={i} className="answer-artifact">
            {artifacts.get(b.artifact_id) ? (
              <ArtifactCard artifact={artifacts.get(b.artifact_id)!} caption={b.caption ?? undefined} onOpen={onArtifact} />
            ) : (
              <p className="warn">Artifact {b.artifact_id} is cited but not listed in the answer.</p>
            )}
          </div>
        ),
      )}
      <h3>Claims</h3>
      <ol className="claims">
        {(answer.claims ?? []).map((c) => (
          <li key={c.claim_id} id={`claim-${c.claim_id}`} className={`claim claim-${c.kind}`}>
            <span className={`claim-chip claim-${c.kind}`}>{CLAIM_LABEL[c.kind]}</span> {c.text}
            <SupportList client={client} claim={c} evidence={evidence} onEvidence={onEvidence} />
          </li>
        ))}
      </ol>
      {(answer.limitations ?? []).length > 0 && (
        <>
          <h3>Limitations</h3>
          <ul>
            {answer.limitations!.map((l, i) => (
              <li key={i}>{l}</li>
            ))}
          </ul>
        </>
      )}
      {(answer.model_releases ?? []).length > 0 && (
        <p className="muted">
          Releases used: {answer.model_releases!.map((r) => <code key={r}>{short(r)} </code>)}
        </p>
      )}
    </div>
  );
}

function SupportList({
  client,
  claim,
  evidence,
  onEvidence,
}: {
  client: ResearchClient;
  claim: Claim;
  evidence: Map<string, EvidenceRef>;
  onEvidence: (e: EvidenceRef) => void;
}) {
  const support = claim.support ?? [];
  if (support.length === 0)
    return <p className="unsupported">{claim.kind === "interpretation" ? "No cited support: the agent's interpretation only." : "No support given: treat as unverified."}</p>;
  return (
    <ul className="support-list">
      {support.map((s, i) => (
        <li key={i}>
          <SupportItem client={client} s={s} evidence={evidence} onEvidence={onEvidence} />
        </li>
      ))}
    </ul>
  );
}

function SupportItem({ client, s, evidence, onEvidence }: { client: ResearchClient; s: ClaimSupport; evidence: Map<string, EvidenceRef>; onEvidence: (e: EvidenceRef) => void }) {
  if (s.kind === "evidence") {
    const e = evidence.get(s.evidence_id);
    if (!e) return <span className="warn">Evidence {s.evidence_id} is cited but missing from the answer.</span>;
    return (
      <>
        <button className="link" onClick={() => onEvidence(e)}>
          Open source
        </button>{" "}
        <span className="muted">
          {e.evidence_class.replace(/_/g, " ")} · {e.locator ? (locatorKind(e.locator) === "page" ? pageCaption(e.locator) : e.locator.path ?? "locator") : "metadata only"}
        </span>
      </>
    );
  }
  return <ArtifactValue client={client} artifactId={s.artifact_id} pointer={s.pointer} />;
}

function ArtifactValue({ client, artifactId, pointer }: { client: ResearchClient; artifactId: string; pointer: string }) {
  const q = useQuery({ queryKey: [client.mode, "artifact", artifactId], queryFn: () => client.getArtifactJson(artifactId), retry: false });
  const value = q.data !== undefined ? resolvePointer(q.data, pointer) : undefined;
  return (
    <span>
      Value <code>{short(artifactId)}#{pointer}</code>:{" "}
      {q.isPending ? (
        <span className="muted">loading…</span>
      ) : q.error ? (
        <span className="warn">not available ({q.error instanceof ApiError ? q.error.envelope.code : "error"}) — the number is not shown</span>
      ) : value === undefined ? (
        <span className="warn">pointer not found in the artifact</span>
      ) : (
        <b>{JSON.stringify(value)}</b>
      )}
    </span>
  );
}

// -------------------------------------------------------------------------------------- artifacts

export function ArtifactCard({ artifact, caption, onOpen }: { artifact: ArtifactRef; caption?: string; onOpen?: (a: ArtifactRef) => void }) {
  return (
    <div className="card-inner">
      <div className="exp-head">
        <b>{artifact.contract ?? artifact.media_type}</b>
        <code>{short(artifact.artifact_id)}</code>
        <OriginBadge origin={artifact.origin} />
      </div>
      {caption && <p>{caption}</p>}
      <p className="muted">
        {artifact.media_type} · {fmtBytes(artifact.byte_length)} · {artifact.created_at}
        {artifact.produced_by?.job_id && <> · job {artifact.produced_by.job_id}</>}
        {artifact.produced_by?.release_id && <> · release {short(artifact.produced_by.release_id)}</>}
      </p>
      {(artifact.inputs ?? []).length > 0 && (
        <p className="muted">Inputs: {artifact.inputs!.map((i) => `${i.kind} ${short(String(i.id))}`).join(", ")}</p>
      )}
      {onOpen && <button onClick={() => onOpen(artifact)}>Inspect payload</button>}
    </div>
  );
}

export function ArtifactPayload({ client, artifact }: { client: ResearchClient; artifact: ArtifactRef }) {
  const q = useQuery({ queryKey: [client.mode, "artifact", artifact.artifact_id], queryFn: () => client.getArtifactJson(artifact.artifact_id), retry: false });
  if (q.isPending) return <p className="muted">Loading payload…</p>;
  if (q.error) return <ErrorBox error={q.error} title="Payload not available" />;
  return <pre className="json">{JSON.stringify(q.data, null, 2)}</pre>;
}

// --------------------------------------------------------------------------------------- evidence

export function EvidenceViewer({ client, evidence }: { client: ResearchClient; evidence: EvidenceRef }) {
  const loc = evidence.locator ?? null;
  const kind = locatorKind(loc);
  const page = loc?.page ?? null;
  const q = useQuery({
    queryKey: [client.mode, "read", evidence.source_id, page],
    queryFn: () => client.readSource(evidence.source_id, page!),
    enabled: kind === "page" && page != null,
    retry: false,
  });
  return (
    <div>
      <div className="exp-head">
        <b>{evidence.evidence_id}</b>
        <span className="badge">{evidence.evidence_class.replace(/_/g, " ")}</span>
        <OriginBadge origin={evidence.origin} />
      </div>
      <p className="muted">
        Source <code>{evidence.source_id}</code>
        {evidence.parse_view_id && (
          <>
            {" "}
            · parse view <code>{evidence.parse_view_id}</code>
          </>
        )}
      </p>
      {evidence.quote && <blockquote className="quote">{evidence.quote}</blockquote>}
      {kind === "metadata" && <p className="warn">Metadata-only citation: the source bytes were not obtained, so there is nothing to open.</p>}
      {kind === "table" && loc && (
        <dl className="kv">
          <dt>File</dt>
          <dd>
            <code>{loc.path ?? "(asset)"}</code>
          </dd>
          {loc.table_id && (
            <>
              <dt>Table</dt>
              <dd>{loc.table_id}</dd>
            </>
          )}
          <dt>Row</dt>
          <dd>{loc.row ?? "—"}</dd>
          <dt>Column</dt>
          <dd>
            <code>{loc.column ?? "—"}</code>
          </dd>
          <dt>Asset</dt>
          <dd>
            <code>{short(loc.asset_sha256)}</code>
          </dd>
        </dl>
      )}
      {kind === "video" && loc && (
        <p>
          Video frame {loc.frame ?? "?"} at PTS {loc.pts_s ?? "?"} s (asset <code>{short(loc.asset_sha256)}</code>).
        </p>
      )}
      {kind === "page" && loc && (
        <>
          <p>
            <b>{pageCaption(loc)}</b>
            {loc.char_start != null && loc.char_end != null && (
              <span className="muted">
                {" "}
                · characters {loc.char_start}–{loc.char_end} ({loc.offset_unit ?? "unit not declared: not citable"})
              </span>
            )}
          </p>
          <div className="page-frame" aria-label={`Rendered ${pageCaption(loc)} with the cited region`}>
            {q.isPending && <p className="muted page-text">Reading the page…</p>}
            {q.error && <ErrorBox error={q.error} title="The page could not be read" />}
            {q.data &&
              q.data.segments
                .filter((s) => s.locator.page === page)
                .map((s, i) => {
                  const spanText =
                    loc.char_start != null && loc.char_end != null && s.locator.char_start != null
                      ? sliceSpan(s.text, loc.char_start - s.locator.char_start, loc.char_end - s.locator.char_start, loc.offset_unit)
                      : null;
                  return (
                    <div key={i} className="page-text">
                      <p>{s.text}</p>
                      {loc.char_start != null && spanText === null && (
                        <p className="warn">The cited span is outside the text returned by the read (or its unit is not supported); no quote is reconstructed.</p>
                      )}
                    </div>
                  );
                })}
            {bboxStyle(loc.bbox) && <div className="bbox" style={bboxStyle(loc.bbox)!} title="Cited region (bbox)" />}
          </div>
          {q.data && !q.data.complete && (
            <p className="muted">
              Partial read{q.data.next && "first" in q.data.next ? ` — continues at pages ${String((q.data.next as { first: number }).first)}–${String((q.data.next as { last?: number }).last ?? "?")}` : ""}.
            </p>
          )}
          <p className="muted">The PDF renderer is not connected yet: the page shows the parse-view text, and the box marks the bbox region on the page.</p>
        </>
      )}
    </div>
  );
}

// ----------------------------------------------------------------------------------------- search

export function SearchResults({ page, onEvidence }: { page: SearchPage; onEvidence: (e: EvidenceRef) => void }) {
  const degraded = page.backends.filter((b) => b.status !== "used" && b.status !== "not_requested");
  return (
    <div>
      <p className="muted">
        Mode {page.mode} · corpus <code>{page.corpus_snapshot}</code> ·{" "}
        {page.total_count != null ? `${page.total_count} results in total` : "total not counted"} <OriginBadge origin={page.origin} />
      </p>
      {degraded.length > 0 && (
        <div className="refusal" role="status">
          <strong>Degraded search</strong>
          {degraded.map((b) => (
            <p key={b.backend}>
              {b.backend}: {b.status}
              {b.detail ? ` — ${b.detail}` : ""}
            </p>
          ))}
        </div>
      )}
      <ul className="backends">
        {page.backends.map((b) => (
          <li key={b.backend}>
            <code>{b.backend}</code>: {b.status}
          </li>
        ))}
      </ul>
      <ol className="hits">
        {page.hits.map((h) => (
          <li key={h.evidence.evidence_id} className="hit">
            <b>{h.title ?? h.evidence.source_id}</b> <OriginBadge origin={h.evidence.origin} />
            {h.snippet && <p className="snippet">{h.snippet}</p>}
            <p className="locator">
              {h.evidence.locator ? (locatorKind(h.evidence.locator) === "page" ? pageCaption(h.evidence.locator) : h.evidence.locator.path) : "metadata only"} · scores{" "}
              {(h.scores ?? []).map((s) => `${s.kind} ${s.value}`).join(", ")}
            </p>
            <button onClick={() => onEvidence(h.evidence)}>Open source</button>
          </li>
        ))}
      </ol>
      {page.hits.length === 0 && <p className="muted">No hits. An empty page does not show that the corpus lacks the information.</p>}
      {page.next_cursor && <p className="muted">More results exist (next cursor <code>{page.next_cursor}</code>).</p>}
    </div>
  );
}

// ----------------------------------------------------------------------------------------- models

/** States that an evaluation granted. Everything else (experimental, fit_only, ...) is not qualified. */
const QUALIFIED: string[] = ["measurement_qualified", "forecast_qualified", "event_qualified"];

export function ReleaseCard({ release }: { release: ModelRelease }) {
  const trained =
    release.trained_on === "not_trained" ? "not trained (literal weights)" : release.trained_on === "synthetic" ? "trained on synthetic data" : `trained on ${release.trained_on}`;
  const state = release.state ?? "unknown";
  const qualified = QUALIFIED.includes(state);
  return (
    <div className="card-inner">
      <div className="exp-head">
        <b>{release.model_name}</b>
        <span className="badge">{release.family}</span>
        <span className={`badge ${qualified ? "badge-ok" : "badge-refused"}`}>{state.replace(/_/g, " ")}</span>
        {release.is_baseline && <span className="badge">baseline</span>}
      </div>
      <p>
        {trained} · lineage {release.lineage}
        {release.construction && <> · construction {release.construction.construction_method.replace(/_/g, " ")}</>}
      </p>
      {!qualified && <p className="warn">Not qualified: no evaluation supports its predictions. Show its outputs as experimental model output only.</p>}
      <p className="muted">
        <code>{short(release.id)}</code> · features {release.feature_order.join(", ")} · posterior valid for parameters: {String(release.posterior_valid_for_parameters)}
      </p>
    </div>
  );
}

// ------------------------------------------------------------------------------------ observations

export function VisualBundleCard({ bundle }: { bundle: VisualObservationBundle }) {
  const axis = spatialAxis(bundle.units_status);
  return (
    <div className="card-inner">
      <div className="exp-head">
        <b>Visual observation bundle</b> <code>{bundle.bundle_id}</code>
        <span className={`badge ${bundle.units_status === "metric" ? "badge-ok" : "badge-refused"}`}>units: {bundle.units_status}</span>
        <OriginBadge origin={bundle.origin} />
      </div>
      <p>
        Axis for these contours: <b>{axis.label}</b>
      </p>
      {bundle.units_status !== "metric" && <p className="warn">No calibration: these contours are never drawn or compared in metres.</p>}
      <p className="muted">
        Camera {bundle.camera_id} · time basis {bundle.timestamp_basis} · origin {bundle.time_origin} · run identity {bundle.run.resolution}
      </p>
    </div>
  );
}
