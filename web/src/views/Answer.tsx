// A ResearchAnswer set as a page of record: serif text with a key after each claim it carries, a
// margin that lists the keys, numbered figures for artifacts, and the limits under it all.
// Level 1 = key; level 2 = hover card (quote + locator); level 3 = side panel (page, row, value).
import { useLayoutEffect, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ArtifactRef, Claim, ClaimSupport, EvidenceRef, ResearchAnswer } from "../contracts/generated";
import { ApiError, type ResearchClient } from "../research/client";
import { locatorKind, pageCaption } from "../research/units";
import { usePanel, itemKey } from "../shell/inspector";
import { useCompare } from "../shell/compare";
import { go } from "../shell/router";
import { IconArrow } from "../shell/icons";
import { CLAIM_LABEL, ClaimGlyph, EvidenceClassTag, fmtBytes, OriginMark, shortId, words } from "../ui/common";

export function locatorText(e: EvidenceRef): string {
  const l = e.locator;
  if (!l) return "metadata only (source bytes not obtained)";
  switch (locatorKind(l)) {
    case "page":
      return pageCaption(l);
    case "table":
      return `${l.path ?? "table"}${l.row != null ? `, row ${l.row}` : ""}${l.column ? `, column ${l.column}` : ""}`;
    case "video":
      return `frame ${l.frame ?? "?"} at ${l.pts_s ?? "?"} s`;
    default:
      return "locator without position";
  }
}

const STATUS_TAG: Record<string, string> = { complete: "tag-ok", partial: "tag-warn", insufficient_evidence: "tag-warn", failed: "tag-bad" };

export function AnswerView({ client, answer, pageSynthetic }: { client: ResearchClient; answer: ResearchAnswer; pageSynthetic: boolean }) {
  const panel = usePanel();
  const claims = new Map((answer.claims ?? []).map((c) => [c.claim_id, c]));
  const evidence = new Map((answer.evidence ?? []).map((e) => [e.evidence_id, e]));
  const artifacts = new Map((answer.artifacts ?? []).map((a) => [a.artifact_id, a]));
  // Keys are numbered in reading order of first appearance.
  const order: string[] = [];
  for (const b of answer.blocks)
    if (b.type === "text" || b.type === "view") for (const id of b.claim_ids ?? []) if (!order.includes(id)) order.push(id);
  for (const c of answer.claims ?? []) if (!order.includes(c.claim_id)) order.push(c.claim_id);
  const keyOf = (id: string) => String(order.indexOf(id) + 1);
  let fig = 0;
  const [hover, setHover] = useState<{ claim: Claim; rect: DOMRect } | null>(null);
  const hideTimer = useRef<number>();

  const openClaim = (c: Claim) => {
    const ev = (c.support ?? []).flatMap((s) => (s.kind === "evidence" && evidence.get(s.evidence_id) ? [evidence.get(s.evidence_id)!] : []));
    panel.open({ kind: "claim", claim: c, keyLabel: keyOf(c.claim_id), evidence: ev, artifacts: answer.artifacts ?? [] });
  };
  const showCard = (c: Claim, el: HTMLElement) => {
    window.clearTimeout(hideTimer.current);
    setHover({ claim: c, rect: el.getBoundingClientRect() });
  };
  const hideCard = () => {
    hideTimer.current = window.setTimeout(() => setHover(null), 120);
  };

  return (
    <section className="answer" aria-label="Answer">
      <div className="answer-main">
        <div className="answer-head">
          <b>Answer</b>
          <span className={`tag ${STATUS_TAG[answer.status] ?? ""}`}>{words(answer.status)}</span>
          <OriginMark origin={answer.origin} pageSynthetic={pageSynthetic} />
          <span>
            {answer.claims?.length ?? 0} claims · {answer.evidence?.length ?? 0} sources · {answer.artifacts?.length ?? 0} artifacts
          </span>
        </div>
        {answer.blocks.map((b, i) => {
          if (b.type === "text")
            return (
              <p key={i} className="answer-text">
                {b.text}
                {(b.claim_ids ?? []).map((id) => {
                  const c = claims.get(id);
                  if (!c)
                    return (
                      <span key={id} className="key" title="This claim is cited but missing from the answer">
                        ?
                      </span>
                    );
                  const on = panel.isOpen(itemKey({ kind: "claim", claim: c, keyLabel: "", evidence: [], artifacts: [] }));
                  return (
                    <button
                      key={id}
                      className={`key${on ? " on" : ""}`}
                      aria-label={`Claim ${keyOf(id)}: ${CLAIM_LABEL[c.kind]}`}
                      onClick={() => openClaim(c)}
                      onMouseEnter={(e) => showCard(c, e.currentTarget)}
                      onMouseLeave={hideCard}
                      onFocus={(e) => showCard(c, e.currentTarget)}
                      onBlur={hideCard}
                    >
                      <ClaimGlyph kind={c.kind} />
                      {keyOf(id)}
                    </button>
                  );
                })}
              </p>
            );
          if (b.type === "view")
            // research.v2 catalog view: placeholder until the catalog renderer (T8) exists.
            return (
              <p key={i} className="note">
                View <code>{b.view.kind}</code> placed by the agent{b.caption ? `: ${b.caption}` : ""} (not rendered yet).
              </p>
            );
          fig++;
          const a = artifacts.get(b.artifact_id);
          return a ? (
            <FigureBlock key={i} n={fig} artifact={a} caption={b.caption ?? undefined} pageSynthetic={pageSynthetic} />
          ) : (
            <p key={i} className="note note-refusal">
              Figure {fig}: artifact {b.artifact_id} is cited but not listed in the answer.
            </p>
          );
        })}
        <NextSteps answer={answer} keyOf={keyOf} evidence={evidence} />
      </div>

      <aside className="margin" aria-label="Claims in this answer">
        <span className="margin-title">Claims</span>
        <ol>
          {order.map((id) => {
            const c = claims.get(id);
            if (!c) return null;
            const on = panel.current?.kind === "claim" && panel.current.claim.claim_id === id;
            return (
              <li key={id}>
                <button className={on ? "on" : ""} onClick={() => openClaim(c)}>
                  <span className="k">{keyOf(id)}</span>
                  <span className="kind">
                    <ClaimGlyph kind={c.kind} /> {CLAIM_LABEL[c.kind]}
                    {(c.support ?? []).length === 0 && " · no support"}
                  </span>
                  <span className="txt">{c.text}</span>
                </button>
              </li>
            );
          })}
        </ol>
      </aside>

      {hover && (
        <HoverCard
          client={client}
          claim={hover.claim}
          keyLabel={keyOf(hover.claim.claim_id)}
          rect={hover.rect}
          evidence={evidence}
          onEnter={() => window.clearTimeout(hideTimer.current)}
          onLeave={hideCard}
        />
      )}
    </section>
  );
}

/** Put a drafted follow-up into the composer; the researcher edits and sends it. */
export const draftFollowUp = (text: string) => window.dispatchEvent(new CustomEvent("ignis:followup", { detail: text }));

/**
 * The harness proposes what to do next, derived only from this answer: the runs its figures were
 * computed from, its deciding source, its stated limits, its unsupported claims and its status.
 */
function NextSteps({ answer, keyOf, evidence }: { answer: ResearchAnswer; keyOf: (id: string) => string; evidence: Map<string, EvidenceRef> }) {
  const panel = usePanel();
  const cmp = useCompare();
  const runs = [...new Set((answer.artifacts ?? []).flatMap((a) => (a.inputs ?? []).filter((i) => i.kind === "run").map((i) => String(i.id))))];
  const firstPage = (answer.evidence ?? []).find((e) => e.locator?.page != null) ?? (answer.evidence ?? [])[0];
  const unsupported = (answer.claims ?? []).filter((c) => c.kind !== "interpretation" && (c.support ?? []).length === 0);
  const steps: { label: string; detail: string; run: () => void }[] = [];
  if (firstPage)
    steps.push({ label: "Read the cited passage", detail: locatorText(firstPage), run: () => panel.open({ kind: "evidence", evidence: evidence.get(firstPage.evidence_id) ?? firstPage }) });
  for (const r of runs.slice(0, 2)) steps.push({ label: `Test the forecast on ${r}`, detail: "seal, then reveal the real continuation", run: () => go("lab", r) });
  if (runs.length >= 2)
    steps.push({
      label: "Compare the runs this answer used",
      detail: runs.slice(0, 4).join(", "),
      run: () => {
        cmp.set(runs.slice(0, 8));
        go("compare");
      },
    });
  // Every stated limit is listed here in full: it changes how the answer reads, and each has an action.
  for (const l of answer.limitations ?? []) steps.push({ label: "Resolve limit", detail: l, run: () => draftFollowUp(`Can you resolve this limitation: ${l}`) });
  for (const c of unsupported.slice(0, 2)) steps.push({ label: `Support claim ${keyOf(c.claim_id)}`, detail: c.text, run: () => draftFollowUp(`Claim ${keyOf(c.claim_id)} has no support. Find a source or computation for it, or withdraw it.`) });
  if (answer.status === "partial" || answer.status === "insufficient_evidence")
    steps.push({ label: "Ask what evidence is missing", detail: `the answer is ${words(answer.status)}`, run: () => draftFollowUp("Which data or sources are missing to make this answer complete, and where could they come from?") });
  if (!steps.length) return null;
  return (
    <section className="next" aria-label="Suggested next steps">
      <h3>Next steps</h3>
      {steps.slice(0, 7).map((s) => (
        <button key={s.label + s.detail} onClick={s.run}>
          <span className="l">{s.label}</span>
          <span className="d">{s.detail}</span>
          <IconArrow width={14} height={14} />
        </button>
      ))}
    </section>
  );
}

function HoverCard({
  client,
  claim,
  keyLabel,
  rect,
  evidence,
  onEnter,
  onLeave,
}: {
  client: ResearchClient;
  claim: Claim;
  keyLabel: string;
  rect: DOMRect;
  evidence: Map<string, EvidenceRef>;
  onEnter: () => void;
  onLeave: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState({ left: rect.left, top: rect.bottom + 8 });
  useLayoutEffect(() => {
    const h = ref.current?.offsetHeight ?? 0;
    const w = ref.current?.offsetWidth ?? 360;
    const below = rect.bottom + 8 + h < window.innerHeight;
    setPos({ left: Math.max(12, Math.min(rect.left - 12, window.innerWidth - w - 12)), top: below ? rect.bottom + 8 : Math.max(12, rect.top - h - 8) });
  }, [rect]);
  const first = (claim.support ?? [])[0];
  return (
    <div ref={ref} className="hovercard" style={pos} role="tooltip" onMouseEnter={onEnter} onMouseLeave={onLeave}>
      <div className="answer-head">
        <b>Claim {keyLabel}</b>
        <span className="kind" style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
          <ClaimGlyph kind={claim.kind} /> {CLAIM_LABEL[claim.kind]}
        </span>
      </div>
      <p>{claim.text}</p>
      {!first && <p className="muted">{claim.kind === "interpretation" ? "The agent's interpretation: no source is cited." : "No support is given: treat as unverified."}</p>}
      {first?.kind === "evidence" && evidence.get(first.evidence_id) && (
        <>
          {evidence.get(first.evidence_id)!.quote ? <blockquote className="quote">{evidence.get(first.evidence_id)!.quote}</blockquote> : <p className="muted">No quote is carried with this citation; open it to read the page.</p>}
          <p className="muted">
            {locatorText(evidence.get(first.evidence_id)!)} · <code>{shortId(evidence.get(first.evidence_id)!.source_id)}</code>
          </p>
        </>
      )}
      {first?.kind === "artifact_value" && <ArtifactValue client={client} support={first} />}
      <p className="muted" style={{ fontSize: 12 }}>
        Click the key to open the evidence beside the answer.
        {(claim.support ?? []).length > 1 && ` ${(claim.support ?? []).length} supports in total.`}
      </p>
    </div>
  );
}

/** A number bound to an artifact field: read by JSON Pointer, never transcribed, never shown without its unit. */
export function ArtifactValue({ client, support }: { client: ResearchClient; support: Extract<ClaimSupport, { kind: "artifact_value" }> }) {
  const { artifact_id, pointer } = support;
  const q = useQuery({ queryKey: [client.mode, "artifact-value", artifact_id, pointer], queryFn: () => client.readArtifactValue(artifact_id, pointer), retry: false });
  const unit = useQuery({ queryKey: [client.mode, "artifact-value", artifact_id, "/unit"], queryFn: () => client.readArtifactValue(artifact_id, "/unit"), enabled: q.isSuccess, retry: false });
  const unitText = typeof unit.data?.value === "string" ? unit.data.value : null;
  return (
    <p>
      <span className="tag tag-flame">
        <span className="glyph g-predicted" aria-hidden="true" /> value
      </span>{" "}
      {q.isPending ? (
        <span className="muted">reading {pointer}…</span>
      ) : q.error ? (
        <span className="muted">
          not readable here ({q.error instanceof ApiError ? q.error.envelope.code : "error"}): the number is not shown. <code>{shortId(artifact_id)}{pointer}</code>
        </span>
      ) : (
        <>
          <b className="num">{JSON.stringify(q.data.value)}</b> {unitText ?? <span className="muted">(unit stated in the artifact)</span>}
          {q.data.partial && <span className="muted"> · preview only</span>} <span className="muted">from <code>{shortId(artifact_id)}{pointer}</code></span>
        </>
      )}
    </p>
  );
}

function FigureBlock({ n, artifact, caption, pageSynthetic }: { n: number; artifact: ArtifactRef; caption?: string; pageSynthetic: boolean }) {
  const panel = usePanel();
  const isModel = artifact.contract === "AnalysisArtifact";
  return (
    <figure className="figure-block">
      <figcaption>
        <b>Figure {n}.</b> {caption ?? "Artifact without a caption."}
      </figcaption>
      <div className="figure-meta">
        {isModel ? (
          <span className="tag tag-flame">
            <span className="glyph g-predicted" aria-hidden="true" /> Model output
          </span>
        ) : (
          <span className="tag">{artifact.contract ?? artifact.media_type}</span>
        )}
        <OriginMark origin={artifact.origin} pageSynthetic={pageSynthetic} />
        <span>{fmtBytes(artifact.byte_length)}</span>
        {artifact.produced_by?.release_id && <span>release {shortId(artifact.produced_by.release_id, 16)}</span>}
        {(artifact.inputs ?? []).map((i) =>
          i.kind === "run" ? (
            <button key={String(i.id)} className="linkish" onClick={() => panel.open({ kind: "run", id: String(i.id) })}>
              run {String(i.id)}
            </button>
          ) : null,
        )}
        <button className="btn btn-s" style={{ marginLeft: "auto" }} onClick={() => panel.open({ kind: "artifact", artifact, caption })}>
          Open figure
        </button>
      </div>
    </figure>
  );
}

export function EvidenceSummary({ e }: { e: EvidenceRef }) {
  return (
    <span style={{ display: "inline-flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
      <EvidenceClassTag cls={e.evidence_class} />
      <span className="muted">{locatorText(e)}</span>
    </span>
  );
}
