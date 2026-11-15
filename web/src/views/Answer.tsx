// A ResearchAnswer read like a message: the text, and after each claim a citation chip that names
// where it comes from (source number in reading order and the place in it). Hovering a chip shows
// the quoted passage, with 1/N across the claim's supports; "Open" shows the original at that place
// in the side panel. Under the answer: its sources, its follow-ups, copy.
import { RichText } from "../shell/RichText";
import { OpenOriginal } from "../shell/OpenOriginal";
import { useLayoutEffect, useMemo, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ArtifactRef, Claim, ClaimSupport, EvidenceRef, ResearchAnswer } from "../contracts/generated";
import { ApiError, type ResearchClient } from "../research/client";
import { locatorKind, pageCaption } from "../research/units";
import { usePanel } from "../shell/inspector";
import { useCompare } from "../shell/compare";
import { go } from "../shell/router";
import { IconArrow, IconBack, IconCheck, IconChevron, IconCopy } from "../shell/icons";
import { EvidenceClassTag, fmtBytes, OriginMark, shortId } from "../ui/common";
import { LinkedViews } from "../shell/linked";
import { ViewBlock } from "./AnswerViews";

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

/** The place inside a source, as short as a chip allows. */
export function placeShort(e: EvidenceRef): string {
  const l = e.locator;
  if (!l) return "record";
  switch (locatorKind(l)) {
    case "page":
      return `p. ${l.page_label ?? l.page}`;
    case "table":
      return l.row != null ? `row ${l.row}` : "table";
    case "video":
      return l.pts_s != null ? `${(+l.pts_s).toFixed(1)} s` : `frame ${l.frame}`;
    default:
      return "source";
  }
}

/**
 * Sources numbered in order of first citation. The evidence contract carries a source id, not a
 * title, so a source is named by its number; the panel shows its passage and original.
 */
export function useSourceNumbers(answer: ResearchAnswer, order: string[]) {
  return useMemo(() => {
    const ev = new Map((answer.evidence ?? []).map((e) => [e.evidence_id, e]));
    const claims = new Map((answer.claims ?? []).map((c) => [c.claim_id, c]));
    const n = new Map<string, number>();
    for (const id of order)
      for (const s of claims.get(id)?.support ?? []) {
        const e = s.kind === "evidence" ? ev.get(s.evidence_id) : undefined;
        if (e && !n.has(e.source_id)) n.set(e.source_id, n.size + 1);
      }
    for (const e of answer.evidence ?? []) if (!n.has(e.source_id)) n.set(e.source_id, n.size + 1);
    return n;
  }, [answer, order]);
}

const STATUS_NOTE: Record<string, string> = { partial: "Partial answer", insufficient_evidence: "Insufficient evidence", failed: "Failed" };

export function AnswerView({ client, answer, pageSynthetic }: { client: ResearchClient; answer: ResearchAnswer; pageSynthetic: boolean }) {
  const claims = new Map((answer.claims ?? []).map((c) => [c.claim_id, c]));
  const evidence = new Map((answer.evidence ?? []).map((e) => [e.evidence_id, e]));
  const artifacts = new Map((answer.artifacts ?? []).map((a) => [a.artifact_id, a]));
  const order: string[] = [];
  for (const b of answer.blocks) if (b.type === "text" || b.type === "view") for (const id of b.claim_ids ?? []) if (!order.includes(id)) order.push(id);
  for (const c of answer.claims ?? []) if (!order.includes(c.claim_id)) order.push(c.claim_id);
  const keyOf = (id: string) => String(order.indexOf(id) + 1);
  const srcNo = useSourceNumbers(answer, order);
  let fig = 0;
  const [hover, setHover] = useState<{ claim: Claim; rect: DOMRect } | null>(null);
  const hideTimer = useRef<number>();
  const show = (c: Claim, el: HTMLElement) => {
    window.clearTimeout(hideTimer.current);
    setHover({ claim: c, rect: el.getBoundingClientRect() });
  };
  const hide = () => {
    hideTimer.current = window.setTimeout(() => setHover(null), 140);
  };

  return (
    <LinkedViews>
      <section className="answer" aria-label="Answer">
        {STATUS_NOTE[answer.status] && <span className={`tag ${answer.status === "failed" ? "tag-bad" : "tag-warn"}`}>{STATUS_NOTE[answer.status]}</span>}
        <OriginMark origin={answer.origin} pageSynthetic={pageSynthetic} />
        {answer.blocks.map((b, i) => {
          if (b.type === "text")
            return (
              <p key={i} className="answer-text">
                <RichText text={b.text} />{" "}
                {(b.claim_ids ?? []).map((id) => {
                  const c = claims.get(id);
                  return c ? <Cite key={id} claim={c} evidence={evidence} srcNo={srcNo} onEnter={show} onLeave={hide} /> : null;
                })}
              </p>
            );
          if (b.type === "view") return <ViewBlock key={i} view={b.view} caption={b.caption} answer={answer} client={client} claimKeys={(b.claim_ids ?? []).map(keyOf)} />;
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
        <SourcesRow answer={answer} srcNo={srcNo} />
        <AnswerActions answer={answer} />
        <NextSteps answer={answer} keyOf={keyOf} />
        {hover && (
          <CiteCard
            client={client}
            claim={hover.claim}
            rect={hover.rect}
            evidence={evidence}
            srcNo={srcNo}
            onEnter={() => window.clearTimeout(hideTimer.current)}
            onLeave={hide}
          />
        )}
      </section>
    </LinkedViews>
  );
}

/** The chip after a claim: "2 · p. 14", "+1" for more supports; "computed" for an artifact value. */
function Cite({ claim, evidence, srcNo, onEnter, onLeave }: { claim: Claim; evidence: Map<string, EvidenceRef>; srcNo: Map<string, number>; onEnter: (c: Claim, el: HTMLElement) => void; onLeave: () => void }) {
  const panel = usePanel();
  const sup = claim.support ?? [];
  const first = sup[0];
  let label: string;
  let cls = "cite";
  if (!first) {
    if (claim.kind !== "interpretation") return null;
    label = "interpretation";
    cls += " cite-interp";
  } else if (first.kind === "evidence") {
    const e = evidence.get(first.evidence_id);
    label = e ? `${srcNo.get(e.source_id) ?? "?"} · ${placeShort(e)}` : "?";
  } else {
    label = "computed";
    cls += " cite-computed";
  }
  const more = sup.length > 1 ? ` +${sup.length - 1}` : "";
  const open = () => {
    const e = first?.kind === "evidence" ? evidence.get(first.evidence_id) : undefined;
    if (e) panel.open({ kind: "evidence", evidence: e, claim });
  };
  const on = panel.current?.kind === "evidence" && panel.current.claim?.claim_id === claim.claim_id;
  return (
    <button
      className={`${cls}${on ? " on" : ""}`}
      onClick={open}
      onMouseEnter={(e) => onEnter(claim, e.currentTarget)}
      onMouseLeave={onLeave}
      onFocus={(e) => onEnter(claim, e.currentTarget)}
      onBlur={onLeave}
      aria-label={`Citation ${label}${more}`}
    >
      {label}
      {more}
    </button>
  );
}

function CiteCard({
  client,
  claim,
  rect,
  evidence,
  srcNo,
  onEnter,
  onLeave,
}: {
  client: ResearchClient;
  claim: Claim;
  rect: DOMRect;
  evidence: Map<string, EvidenceRef>;
  srcNo: Map<string, number>;
  onEnter: () => void;
  onLeave: () => void;
}) {
  const panel = usePanel();
  const ref = useRef<HTMLDivElement>(null);
  const [i, setI] = useState(0);
  const [pos, setPos] = useState({ left: rect.left, top: rect.bottom + 8 });
  useLayoutEffect(() => {
    const h = ref.current?.offsetHeight ?? 0;
    const w = ref.current?.offsetWidth ?? 380;
    const below = rect.bottom + 8 + h < window.innerHeight;
    setPos({ left: Math.max(12, Math.min(rect.left - 12, window.innerWidth - w - 12)), top: below ? rect.bottom + 8 : Math.max(12, rect.top - h - 8) });
  }, [rect, i]);
  const sup = claim.support ?? [];
  const s: ClaimSupport | undefined = sup[Math.min(i, sup.length - 1)];
  const e = s?.kind === "evidence" ? evidence.get(s.evidence_id) : undefined;
  return (
    <div ref={ref} className="cite-card" style={pos} role="tooltip" onMouseEnter={onEnter} onMouseLeave={onLeave}>
      <div className="cite-card-head">
        <span className="src-no">{e ? srcNo.get(e.source_id) : "ƒ"}</span>
        <span className="t">
          <b>{e ? `Source ${srcNo.get(e.source_id)}` : s ? "Computed value" : "Interpretation"}</b>
          <span>{e ? locatorText(e) : s?.kind === "artifact_value" ? <code>{shortId(s.artifact_id)}{s.pointer}</code> : "no source is cited"}</span>
        </span>
        {sup.length > 1 && (
          <span className="pager">
            <button className="btn btn-quiet btn-icon btn-s" onClick={() => setI((k) => (k - 1 + sup.length) % sup.length)} aria-label="Previous support">
              <IconBack />
            </button>
            {Math.min(i, sup.length - 1) + 1}/{sup.length}
            <button className="btn btn-quiet btn-icon btn-s" onClick={() => setI((k) => (k + 1) % sup.length)} aria-label="Next support">
              <IconChevron />
            </button>
          </span>
        )}
      </div>
      {e && (e.quote ? <p className="cite-quote">{e.quote}</p> : <p className="muted">No quote with this citation.</p>)}
      {s?.kind === "artifact_value" && <ArtifactValue client={client} support={s} />}
      {e && (
        <div className="cite-card-foot">
          <EvidenceClassTag cls={e.evidence_class} />
          <span style={{ display: "flex", gap: 6 }}>
            {e.locator && <OpenOriginal sourceId={e.source_id} loc={e.locator} />}
            <button className="btn btn-s" onClick={() => panel.open({ kind: "evidence", evidence: e, claim })}>
              Open <IconChevron />
            </button>
          </span>
        </div>
      )}
    </div>
  );
}

/** Under the answer: its sources by number, each opening its first cited passage. */
function SourcesRow({ answer, srcNo }: { answer: ResearchAnswer; srcNo: Map<string, number> }) {
  const panel = usePanel();
  const [open, setOpen] = useState(false);
  const bySource = useMemo(() => {
    const m = new Map<string, EvidenceRef[]>();
    for (const e of answer.evidence ?? []) m.set(e.source_id, [...(m.get(e.source_id) ?? []), e]);
    return [...m.entries()].sort((a, b) => (srcNo.get(a[0]) ?? 0) - (srcNo.get(b[0]) ?? 0));
  }, [answer, srcNo]);
  if (!bySource.length) return null;
  return (
    <div className="sources-row">
      <button className="sources-sum" onClick={() => setOpen((o) => !o)} aria-expanded={open}>
        <span className="src-stack" aria-hidden="true">
          {bySource.slice(0, 4).map(([id]) => (
            <span key={id} className="src-no">
              {srcNo.get(id)}
            </span>
          ))}
        </span>
        <b>Sources</b>
        <span className="muted">
          {bySource.length} {bySource.length === 1 ? "source" : "sources"} · {(answer.evidence ?? []).length} passages
        </span>
        <IconChevron className="chev" />
      </button>
      {open && (
        <ol className="sources-list">
          {bySource.map(([id, evs]) => (
            <li key={id}>
              <span className="src-no">{srcNo.get(id)}</span>
              <span className="t">
                <code>{shortId(id, 28)}</code>
                <span className="places">
                  {evs.map((e) => (
                    <button key={e.evidence_id} className="linkish" onClick={() => panel.open({ kind: "evidence", evidence: e })}>
                      {placeShort(e)}
                    </button>
                  ))}
                  {evs[0]?.locator && <OpenOriginal sourceId={id} loc={evs[0].locator} label="Original ↗" className="linkish" />}
                </span>
              </span>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}

function AnswerActions({ answer }: { answer: ResearchAnswer }) {
  const [copied, setCopied] = useState(false);
  const text = answer.blocks.flatMap((b) => (b.type === "text" ? [b.text] : [])).join("\n\n");
  return (
    <div className="answer-actions">
      <button
        className="btn btn-quiet btn-icon btn-s"
        aria-label={copied ? "Copied" : "Copy the answer"}
        title={copied ? "Copied" : "Copy"}
        onClick={() => {
          void navigator.clipboard?.writeText(text).then(() => {
            setCopied(true);
            window.setTimeout(() => setCopied(false), 1400);
          });
        }}
      >
        {copied ? <IconCheck /> : <IconCopy />}
      </button>
    </div>
  );
}

/** Put a drafted follow-up into the composer; the researcher edits and sends it. */
export const draftFollowUp = (text: string) => window.dispatchEvent(new CustomEvent("ignis:followup", { detail: text }));

/**
 * The harness proposes what to do next, derived only from this answer: the runs its figures were
 * computed from, its stated limits, its unsupported claims and its status.
 */
function NextSteps({ answer, keyOf }: { answer: ResearchAnswer; keyOf: (id: string) => string }) {
  const cmp = useCompare();
  const runs = [...new Set((answer.artifacts ?? []).flatMap((a) => (a.inputs ?? []).filter((i) => i.kind === "run").map((i) => String(i.id))))];
  const unsupported = (answer.claims ?? []).filter((c) => c.kind !== "interpretation" && (c.support ?? []).length === 0);
  const steps: { label: string; run: () => void }[] = [];
  for (const r of runs.slice(0, 2)) steps.push({ label: `Forecast ${r} from its first seconds`, run: () => go("lab", r) });
  if (runs.length >= 2)
    steps.push({
      label: `Compare ${runs.slice(0, 4).join(", ")}`,
      run: () => {
        cmp.set(runs.slice(0, 8));
        go("compare");
      },
    });
  for (const l of answer.limitations ?? []) steps.push({ label: l, run: () => draftFollowUp(`Can you resolve this limitation: ${l}`) });
  for (const c of unsupported.slice(0, 2)) steps.push({ label: `Find support for claim ${keyOf(c.claim_id)}`, run: () => draftFollowUp(`Claim ${keyOf(c.claim_id)} has no support. Find a source or computation for it, or withdraw it.`) });
  if (answer.status === "partial" || answer.status === "insufficient_evidence")
    steps.push({ label: "Which evidence is missing?", run: () => draftFollowUp("Which data or sources are missing to make this answer complete, and where could they come from?") });
  if (!steps.length) return null;
  return (
    <section className="followups" aria-label="Follow-ups">
      {steps.slice(0, 6).map((s) => (
        <button key={s.label} onClick={s.run}>
          <IconArrow width={14} height={14} />
          <span>{s.label}</span>
        </button>
      ))}
    </section>
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
      {q.isPending ? (
        <span className="muted">Reading…</span>
      ) : q.error ? (
        <span className="muted">
          Not readable here ({q.error instanceof ApiError ? q.error.envelope.code : "error"}). <code>{shortId(artifact_id)}{pointer}</code>
        </span>
      ) : (
        <>
          <b className="num" style={{ fontSize: 18, fontWeight: 500 }}>
            {JSON.stringify(q.data.value)}
          </b>{" "}
          {unitText}
          {q.data.partial && <span className="muted"> · preview</span>}
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

