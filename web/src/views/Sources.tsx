// Source search over the pinned corpus snapshot. A hit is a pointer with a snippet; the passage is
// shown beside the list: the NASA original at the hit's page with the passage marked (streamed from
// NASA through SourceAccess, nothing stored). A degraded search says which backend was missing.
import { useState } from "react";
import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { ApiError } from "../research/client";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { go } from "../shell/router";
import { Caution, ErrorNote, Gap, OriginMark, words } from "../ui/common";
import { EvidenceSummary } from "./Answer";
import { OpenOriginal } from "../shell/OpenOriginal";
import { httpUrl } from "../shell/SourceOriginal";
import { locatorKind } from "../research/units";
import { MODE_TEXT, OriginalBody } from "./Original";
import type { SearchHit } from "../contracts/generated";

/** Passages per page of results. */
const PAGE = 25;

/**
 * Starter searches, offered only (nothing runs until one is picked). Chosen on the staging corpus
 * (15 Nov 2026) because their first hits are passages of substance, not reference lists.
 */
const STARTERS = ["oxygen concentration flammability limit", "soot formation in microgravity flames", "flame size of gas jet flames in microgravity"];

export function Sources({ initial }: { initial: string | null }) {
  const { client, researchMode, replay, caps, synthetic } = useData();
  const panel = usePanel();
  const [text, setText] = useState(initial ?? "");
  // Nothing is searched until the user asks: an empty page offers the starters.
  const submitted = initial?.trim() || null;
  const unavailable = caps.data?.knowledge.available === false;
  // Every page of the result order, followed by its cursor ("Load more"); the first page is larger
  // than the server default so a broad query shows more than a handful of passages at once.
  const q = useInfiniteQuery({
    queryKey: [researchMode, replay, "search", submitted],
    initialPageParam: null as string | null,
    queryFn: ({ pageParam }) => client.search({ schema_version: "evidence.v1", query: submitted!, mode: "hybrid", page_size: PAGE, ...(pageParam ? { cursor: pageParam } : {}) }),
    getNextPageParam: (last) => last.next_cursor ?? undefined,
    enabled: !!submitted && !unavailable,
    retry: false,
  });
  const page = q.data?.pages[0];
  const hits = q.data?.pages.flatMap((p) => p.hits) ?? [];
  const [sel, setSel] = useState<string | null>(null);
  const selected = hits.find((h) => h.evidence.evidence_id === sel) ?? hits[0];
  const degraded = page?.backends.filter((b) => b.status !== "used" && b.status !== "not_requested") ?? [];

  return (
    <div className="view">
      <div className="view-head">
        <h1>Sources</h1>
      </div>
      <form
        className="toolbar"
        onSubmit={(e) => {
          e.preventDefault();
          if (text.trim()) go("sources", null, { q: text.trim() });
        }}
      >
        <label className="sr-only" htmlFor="src-q">
          Search query
        </label>
        <input id="src-q" className="field" style={{ flex: "1 1 320px", height: 34, fontSize: 14 }} value={text} onChange={(e) => setText(e.target.value)} placeholder="Search NASA documents" disabled={unavailable} />
        <button className="btn btn-primary" type="submit" disabled={!text.trim() || unavailable}>
          Search
        </button>
      </form>
      <div className="toolbar" style={{ flexWrap: "wrap", gap: 6, marginTop: 6 }}>
        <span className="muted small">Try:</span>
        {STARTERS.map((t) => (
          <button key={t} className={`btn btn-s${t === submitted ? " btn-primary" : ""}`} onClick={() => go("sources", null, { q: t })}>
            {t}
          </button>
        ))}
      </div>
      {!submitted && !unavailable && <p className="muted src-empty">Search the NASA reports, papers and test documents IGNIS holds. Each hit opens at its page in the original.</p>}
      {unavailable && <Gap title="Search is not available on this server">{caps.data?.knowledge.detail}</Gap>}
      {q.isFetching && !q.isFetchingNextPage && <p className="muted">Searching…</p>}
      {q.error && <ErrorNote error={q.error} title={q.error instanceof ApiError && q.error.envelope.code === "capability_unavailable" ? "Search is not configured" : "The search failed"} />}
      {page && (
        <>
          <p className="muted" style={{ fontSize: 12 }}>
            {words(page.mode)} search · corpus <code>{page.corpus_snapshot}</code> · {page.total_count != null ? `${page.total_count} results` : "total not counted"}{" "}
            <OriginMark origin={page.origin} pageSynthetic={synthetic} />
          </p>
          {degraded.length > 0 && (
            <Caution title="Degraded ranking">
              {degraded.map((b) => `${words(b.backend)} ${words(b.status)}${b.detail ? `: ${b.detail}` : ""}`).join(" · ")}
            </Caution>
          )}
          <div className="src-split">
            <ol className="src-hits" aria-label="Search results">
              {hits.map((h) => {
                const on = (sel ?? hits[0]?.evidence.evidence_id) === h.evidence.evidence_id;
                return (
                  <li key={h.evidence.evidence_id}>
                    <button className={`src-hit${on ? " on" : ""}`} onClick={() => setSel(h.evidence.evidence_id)} aria-pressed={on}>
                      <span className="src-hit-head">
                        <span className="num dim">{h.rank}</span>
                        <b>{h.title ?? h.evidence.source_id}</b>
                      </span>
                      <span className="src-hit-quote">{h.evidence.quote ?? h.snippet}</span>
                      <span className="src-hit-where">
                        <EvidenceSummary e={h.evidence} />
                      </span>
                    </button>
                  </li>
                );
              })}
              {hits.length === 0 && <li className="dim">No hits. An empty page does not show that the corpus lacks the information.</li>}
              {q.hasNextPage && (
                <li>
                  <button className="btn btn-s" style={{ width: "100%" }} disabled={q.isFetchingNextPage} onClick={() => void q.fetchNextPage()}>
                    {q.isFetchingNextPage ? "Loading…" : `Load more (${hits.length} shown)`}
                  </button>
                </li>
              )}
            </ol>
            {selected && (
              <aside className="src-preview" aria-label="Selected source">
                <SourcePreview key={selected.evidence.evidence_id} hit={selected} onPassage={() => panel.open({ kind: "evidence", evidence: selected.evidence })} />
              </aside>
            )}
          </div>
        </>
      )}
    </div>
  );
}

/** The selected hit's original, at its page with the cited region marked. */
function SourcePreview({ hit, onPassage }: { hit: SearchHit; onPassage: () => void }) {
  const { client, researchMode } = useData();
  const e = hit.evidence;
  const title = hit.title ?? e.locator?.path ?? e.source_id;
  const access = useQuery({
    queryKey: [researchMode, "access", e.source_id],
    queryFn: () => client.sourceAccess(e.source_id),
    retry: false,
    staleTime: (qq) => {
      const a = qq.state.data;
      return a?.expires_at ? Math.max(0, Date.parse(a.expires_at) - Date.now() - 30_000) : 5 * 60_000;
    },
  });
  const a = access.data;
  return (
    <div className="card" style={{ padding: 12 }}>
      <div className="toolbar" style={{ marginBottom: 8, flexWrap: "wrap" }}>
        <b style={{ flex: "1 1 200px", overflowWrap: "anywhere" }}>{title}</b>
        <button className="btn btn-s" onClick={onPassage}>
          Passage
        </button>
        {e.locator && <OpenOriginal sourceId={e.source_id} loc={e.locator} title={title} label="Originals tab ↗" />}
        {a && httpUrl(a.landing_url) && (
          <a className="btn btn-s" href={a.landing_url} target="_blank" rel="noopener noreferrer">
            Open at NASA ↗
          </a>
        )}
      </div>
      {a && <p className="muted small">{MODE_TEXT[a.mode]}</p>}
      {access.isPending && <p className="muted">Finding the original…</p>}
      {access.error && <ErrorNote error={access.error} title="The original cannot be opened from here; the passage is still readable" />}
      {a && e.locator && <OriginalBody a={a} loc={e.locator} kind={locatorKind(e.locator)} title={title} />}
      {a && !e.locator && <p className="muted small">This hit has no page to open.</p>}
    </div>
  );
}
