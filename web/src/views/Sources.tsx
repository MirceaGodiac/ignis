// Source search over the pinned corpus snapshot. A hit is a pointer with a snippet; hits of one
// document are gathered under one result. The NASA original opens beside the list only when a passage
// is selected: results paint first, and PDF.js (chunk, worker, byte ranges) is never on that path. On
// idle the first document's access and the viewer chunk are warmed, so the first click opens fast.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useInfiniteQuery, useQuery, useQueryClient } from "@tanstack/react-query";
import { ApiError } from "../research/client";
import { groupHits, hitSourceName, markTerms, queryTerms, type HitGroup, type SourceName } from "../research/sourceTitle";
import { locatorKind } from "../research/units";
import { useData } from "../shell/data";
import { IconExternal } from "../shell/icons";
import { usePanel } from "../shell/inspector";
import { OpenOriginal } from "../shell/OpenOriginal";
import { accessOptions, searchOptions, STARTERS, whenIdle } from "../shell/queries";
import { go } from "../shell/router";
import { httpUrl, loadPdfPage, MODE_TEXT, PdfSkeleton } from "../shell/SourceOriginal";
import { ErrorNote, EvidenceClassTag, Gap, OriginMark, words } from "../ui/common";
import type { SearchHit } from "../contracts/generated";
import { locatorText } from "./Answer";
import { OriginalBody } from "./Original";

/** Passages a document shows before "more passages". */
const FOLD = 3;

export function Sources({ initial }: { initial: string | null }) {
  const { client, researchMode, replay, caps, synthetic } = useData();
  const qc = useQueryClient();
  const panel = usePanel();
  // An empty page shows the first starter, with its words in the field, so it reads as a search.
  const submitted = initial?.trim() || STARTERS[0]!;
  const [text, setText] = useState(submitted);
  const unavailable = caps.data?.knowledge.available === false;
  const q = useInfiniteQuery({ ...searchOptions(client, researchMode, replay, submitted), enabled: !unavailable });
  const page = q.data?.pages[0];
  const hits = useMemo(() => q.data?.pages.flatMap((p) => p.hits) ?? [], [q.data]);
  const groups = useMemo(() => groupHits(hits), [hits]);
  const terms = useMemo(() => queryTerms(submitted), [submitted]);
  const [sel, setSel] = useState<string | null>(null);
  const selected = sel ? hits.find((h) => h.evidence.evidence_id === sel) : undefined;
  const degraded = page?.backends.filter((b) => b.status !== "used" && b.status !== "not_requested") ?? [];
  const aside = useRef<HTMLElement>(null);

  useEffect(() => {
    if (page) performance.mark?.("ignis:sources:results");
  }, [page]);

  // Warming costs one small JSON call and a cached chunk; the PDF bytes wait for the click.
  const warm = useCallback(
    (h: SearchHit) => {
      void qc.prefetchQuery(accessOptions(client, researchMode, h.evidence.source_id));
      if (h.evidence.locator && locatorKind(h.evidence.locator) === "page") void loadPdfPage().catch(() => undefined);
    },
    [qc, client, researchMode],
  );
  const first = hits[0];
  useEffect(() => {
    if (!first) return;
    return whenIdle(() => warm(first), 4000);
  }, [first, warm]);

  const choose = (h: SearchHit) => {
    setSel(h.evidence.evidence_id);
    // In one column the original sits under the list: bring it into view.
    if (window.matchMedia?.("(max-width: 900px)").matches) requestAnimationFrame(() => aside.current?.scrollIntoView({ behavior: "smooth", block: "start" }));
  };

  const count = page?.total_count != null ? page.total_count : hits.length;
  return (
    <div className="view">
      <div className="view-head">
        <h1>Sources</h1>
      </div>
      <form
        className="toolbar"
        role="search"
        onSubmit={(e) => {
          e.preventDefault();
          if (text.trim()) go("sources", null, { q: text.trim() });
        }}
      >
        <label className="sr-only" htmlFor="src-q">
          Search NASA documents
        </label>
        <input id="src-q" className="field src-field" type="search" value={text} onChange={(e) => setText(e.target.value)} placeholder="Search NASA documents" disabled={unavailable} />
        <button className="btn btn-primary" type="submit" disabled={!text.trim() || unavailable}>
          Search
        </button>
      </form>
      <div className="src-starters" aria-label="Example searches">
        <span className="muted small">Try</span>
        {STARTERS.map((t) => (
          <button key={t} className={`btn btn-s${t === submitted ? " btn-on" : " btn-quiet"}`} aria-pressed={t === submitted} onClick={() => go("sources", null, { q: t })}>
            {t}
          </button>
        ))}
      </div>
      {unavailable && <Gap title="Search is not available on this server">{caps.data?.knowledge.detail}</Gap>}
      {q.error && <ErrorNote error={q.error} title={q.error instanceof ApiError && q.error.envelope.code === "capability_unavailable" ? "Search is not configured" : "The search failed"} />}
      {q.isPending && !unavailable && !q.error && <ResultsSkeleton />}
      {page && (
        <>
          <p className="src-count muted small">
            {hits.length === 0 ? "No passages" : `${count.toLocaleString()}${page.total_count == null && q.hasNextPage ? "+" : ""} passage${count === 1 ? "" : "s"} in ${groups.length}${q.hasNextPage ? "+" : ""} document${groups.length === 1 ? "" : "s"}`}
            {degraded.length > 0 && (
              <span title={degraded.map((b) => `${words(b.backend)} ${words(b.status)}${b.detail ? `: ${b.detail}` : ""}`).join("\n")}> · partial ranking ({degraded.map((b) => words(b.backend)).join(", ")} {degraded.length === 1 ? "is" : "are"} off)</span>
            )}{" "}
            <OriginMark origin={page.origin} pageSynthetic={synthetic} />
          </p>
          <div className="src-split">
            <ol className="src-docs" aria-label="Search results">
              {groups.map((g) => (
                <DocResult key={g.sourceId} g={g} terms={terms} sel={sel} onSelect={choose} onWarm={warm} />
              ))}
              {hits.length === 0 && <li className="muted small">No passage matched. An empty result does not show that the corpus lacks the information; try other words.</li>}
              {q.hasNextPage && (
                <li>
                  <button className="btn btn-s" style={{ width: "100%", justifyContent: "center" }} disabled={q.isFetchingNextPage} onClick={() => void q.fetchNextPage()}>
                    {q.isFetchingNextPage ? "Loading…" : "More results"}
                  </button>
                </li>
              )}
            </ol>
            <aside ref={aside} className="src-preview" aria-label="Selected source">
              {selected ? (
                <SourcePreview key={selected.evidence.evidence_id} hit={selected} name={hitSourceName(selected)} onPassage={() => panel.open({ kind: "evidence", evidence: selected.evidence })} />
              ) : (
                hits.length > 0 && <p className="src-empty">Select a passage to open its page in the NASA original, with the passage marked.</p>
              )}
            </aside>
          </div>
        </>
      )}
    </div>
  );
}

function ResultsSkeleton() {
  return (
    <ol className="src-docs src-skel" aria-busy="true" aria-label="Searching">
      {[0, 1, 2, 3].map((i) => (
        <li key={i} className="src-doc">
          <span className="skel" style={{ width: `${62 - i * 7}%`, height: 14 }} />
          <span className="skel" style={{ width: "92%" }} />
          <span className="skel" style={{ width: "78%" }} />
        </li>
      ))}
    </ol>
  );
}

/** The query's words in bold ink inside a snippet. */
function Marked({ text, terms }: { text: string; terms: string[] }) {
  return (
    <>
      {markTerms(text, terms).map((p, i) =>
        p.match ? (
          <mark key={i} className="src-mark">
            {p.text}
          </mark>
        ) : (
          p.text
        ),
      )}
    </>
  );
}

/** Where a passage sits, short: "p. 135" for a page, the locator's own words otherwise. */
function place(h: SearchHit): string {
  const l = h.evidence.locator;
  if (l && locatorKind(l) === "page" && l.page != null) return `p. ${l.page_label ?? l.page}`;
  return locatorText(h.evidence);
}

function DocResult({ g, terms, sel, onSelect, onWarm }: { g: HitGroup; terms: string[]; sel: string | null; onSelect: (h: SearchHit) => void; onWarm: (h: SearchHit) => void }) {
  const holdsSel = g.hits.some((h) => h.evidence.evidence_id === sel);
  const [open, setOpen] = useState(false);
  const shown = open || g.hits.length <= FOLD ? g.hits : g.hits.slice(0, FOLD);
  return (
    <li className={`src-doc${holdsSel ? " has-sel" : ""}`}>
      <div className="src-doc-head">
        <span className="src-doc-title" title={g.hits[0]?.title ?? undefined}>
          {g.name.title}
        </span>
        <span className="src-doc-meta">
          {g.name.investigation && <span className="tag">{g.name.investigation}</span>}
          {g.hits.length > 1 && <span className="muted">{g.hits.length} passages</span>}
        </span>
      </div>
      <ul className="src-passages">
        {shown.map((h) => {
          const on = sel === h.evidence.evidence_id;
          const cls = h.evidence.evidence_class;
          return (
            <li key={h.evidence.evidence_id}>
              <button className={`src-pass${on ? " on" : ""}`} aria-pressed={on} onClick={() => onSelect(h)} onPointerEnter={() => onWarm(h)} onFocus={() => onWarm(h)}>
                <span className="src-pass-where num">
                  {place(h)}
                  {/* Every search hit is source text; a class is named only when it says something else. */}
                  {cls !== "source_derived" && <EvidenceClassTag cls={cls} />}
                </span>
                <span className="src-pass-quote">
                  <Marked text={h.evidence.quote ?? h.snippet} terms={terms} />
                </span>
              </button>
            </li>
          );
        })}
      </ul>
      {g.hits.length > FOLD && (
        <button className="btn btn-s btn-quiet src-more" aria-expanded={open} onClick={() => setOpen((o) => !o)}>
          {open ? "Fewer passages" : `${g.hits.length - FOLD} more passage${g.hits.length - FOLD === 1 ? "" : "s"}`}
        </button>
      )}
    </li>
  );
}

/** The selected passage's original, at its page with the cited region marked. */
function SourcePreview({ hit, name, onPassage }: { hit: SearchHit; name: SourceName; onPassage: () => void }) {
  const { client, researchMode } = useData();
  const e = hit.evidence;
  const raw = hit.title ?? e.locator?.path ?? e.source_id;
  const access = useQuery(accessOptions(client, researchMode, e.source_id));
  const a = access.data;
  const isPage = !!e.locator && locatorKind(e.locator) === "page";
  return (
    <div className="src-prev">
      <div className="src-prev-head">
        <div className="src-prev-name">
          <h2 title={raw}>{name.title}</h2>
          <span className="src-doc-meta">
            {name.investigation && <span className="tag">{name.investigation}</span>}
            <span className="muted">{locatorText(e)}</span>
          </span>
        </div>
        <div className="src-prev-actions">
          <button className="btn btn-s btn-quiet" onClick={onPassage} title="The passage as IGNIS read it, in the side panel">
            Passage
          </button>
          {e.locator && <OpenOriginal sourceId={e.source_id} loc={e.locator} title={raw} label="New tab" className="btn btn-s btn-quiet" />}
          {a && httpUrl(a.landing_url) && (
            <a className="btn btn-s" href={a.landing_url} target="_blank" rel="noopener noreferrer" title={MODE_TEXT[a.mode]}>
              Open at NASA <IconExternal width={13} height={13} />
            </a>
          )}
        </div>
      </div>
      {access.isPending && (isPage ? <PdfSkeleton label="Finding the original…" /> : <p className="muted small">Finding the original…</p>)}
      {access.error && <ErrorNote error={access.error} title="The original cannot be opened from here; the passage is still readable" />}
      {a && e.locator && <OriginalBody a={a} loc={e.locator} kind={locatorKind(e.locator)} title={raw} />}
      {a && !e.locator && <p className="muted small">This passage has no page to open.</p>}
    </div>
  );
}
