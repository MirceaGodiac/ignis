// Source search over the pinned corpus snapshot. A hit is a pointer with a snippet; the passage is
// read in the side panel. A degraded search says which backend was missing.
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { ApiError } from "../research/client";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { go } from "../shell/router";
import { Caution, ErrorNote, Gap, OriginMark, words } from "../ui/common";
import { EvidenceSummary } from "./Answer";
import { OpenOriginal } from "../shell/OpenOriginal";

export function Sources({ initial }: { initial: string | null }) {
  const { client, researchMode, replay, caps, synthetic } = useData();
  const panel = usePanel();
  const [text, setText] = useState(initial ?? "");
  const submitted = initial;
  const unavailable = caps.data?.knowledge.available === false;
  const q = useQuery({
    queryKey: [researchMode, replay, "search", submitted],
    queryFn: () => client.search({ schema_version: "evidence.v1", query: submitted!, mode: "hybrid" }),
    enabled: !!submitted && !unavailable,
    retry: false,
  });
  const page = q.data;
  const degraded = page?.backends.filter((b) => b.status !== "used" && b.status !== "not_requested") ?? [];

  return (
    <div className="view">
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
        <input id="src-q" className="field" style={{ flex: "1 1 320px", height: 34, fontSize: 14 }} value={text} onChange={(e) => setText(e.target.value)} placeholder="e.g. droplet extinction diameter at elevated oxygen" disabled={unavailable} />
        <button className="btn btn-primary" type="submit" disabled={!text.trim() || unavailable}>
          Search
        </button>
      </form>
      {unavailable && <Gap title="Search is not configured in this deployment">{caps.data?.knowledge.detail}</Gap>}
      {!submitted && !unavailable && <p className="muted">Results show the hit's evidence class, where it sits in its source, and how it was ranked.</p>}
      {q.isFetching && <p className="muted">Searching…</p>}
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
          <div className="tbl-wrap">
            <table className="tbl">
              <thead>
                <tr>
                  <th className="r">#</th>
                  <th>Source and passage</th>
                  <th>Where</th>
                  <th className="r">Scores</th>
                </tr>
              </thead>
              <tbody>
                {page.hits.map((h) => (
                  <tr key={h.evidence.evidence_id} className="clickable" onClick={() => panel.open({ kind: "evidence", evidence: h.evidence })}>
                    <td className="r num dim">{h.rank}</td>
                    <td style={{ whiteSpace: "normal" }}>
                      <b style={{ fontWeight: 600 }}>{h.title ?? h.evidence.source_id}</b>
                      <p style={{ fontFamily: "var(--font-text)", color: "var(--ink-2)", marginTop: 2 }}>{h.evidence.quote ?? h.snippet}</p>
                    </td>
                    <td style={{ whiteSpace: "normal" }}>
                      <EvidenceSummary e={h.evidence} />
                      {h.evidence.locator && (
                        <div style={{ marginTop: 4 }}>
                          <OpenOriginal sourceId={h.evidence.source_id} loc={h.evidence.locator} title={h.title} />
                        </div>
                      )}
                    </td>
                    <td className="r num dim">{(h.scores ?? []).map((s) => `${s.kind} ${s.value}`).join(" · ")}</td>
                  </tr>
                ))}
                {page.hits.length === 0 && (
                  <tr>
                    <td colSpan={4} className="dim">
                      No hits. An empty page does not show that the corpus lacks the information.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
          {page.next_cursor && <p className="muted" style={{ fontSize: 12 }}>More results exist beyond this page.</p>}
        </>
      )}
    </div>
  );
}
