// The NASA original behind a locator, on its own page so it opens in a new tab:
//   #/original/<source id>?page=&bbox=x0,y0,x1,y1&row=&col=&t=&title=
// Bytes come through `SourceAccess` (M12): PSI streams from S3, NTRS through the Worker proxy,
// anything else opens at the provider. IGNIS stores nothing. The page is the PDF page with the cited
// region marked, the video at the cited time, or the table record with its neighbours (read from the
// start of the file only as far as the cited record).
import { lazy, Suspense, useEffect, useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { SourceAccess, SourceLocator } from "../contracts/generated";
import { accessExpired, videoTimeS } from "../research/sourceAccess";
import { csvEnd, csvFeed, csvFields, csvStart } from "../research/csvRecords";
import { locatorKind } from "../research/units";
import { useData } from "../shell/data";
import { VideoOriginal, httpUrl } from "../shell/SourceOriginal";
import { ErrorNote, Gap } from "../ui/common";

const PdfPage = lazy(() => import("../shell/PdfPage"));

const MODE_TEXT: Record<SourceAccess["mode"], string> = {
  direct: "Streamed from NASA; IGNIS stores no copy",
  proxy: "Through the IGNIS pass-through proxy; nothing is stored",
  link_out: "Opens at the provider",
};

/** Locator fields carried in the URL (only what the viewers use). */
export function locatorFromParams(sha: string, p: URLSearchParams): SourceLocator {
  const num = (k: string) => {
    const v = p.get(k);
    if (v == null || v === "") return null;
    const n = Number(v);
    return Number.isFinite(n) ? n : null;
  };
  const bb = (p.get("bbox") ?? "").split(",").map(Number);
  return {
    asset_sha256: sha,
    page: num("page"),
    bbox: bb.length === 4 && bb.every(Number.isFinite) ? (bb as [number, number, number, number]) : null,
    row: num("row"),
    column: p.get("col"),
    pts_s: num("t"),
    path: p.get("path"),
  };
}

export function Original({ sourceId, params }: { sourceId: string | null; params: URLSearchParams }) {
  const { client, researchMode } = useData();
  const sha = sourceId?.startsWith("sha256:") ? sourceId.slice(7) : (sourceId ?? "");
  const loc = useMemo(() => locatorFromParams(sha, params), [sha, params]);
  const title = params.get("title") ?? loc.path ?? sourceId ?? "Source";
  const access = useQuery({
    queryKey: [researchMode, "access", sourceId],
    queryFn: () => client.sourceAccess(sourceId!),
    enabled: !!sourceId,
    retry: false,
    staleTime: (q) => {
      const a = q.state.data;
      return a?.expires_at ? Math.max(0, Date.parse(a.expires_at) - Date.now() - 30_000) : 5 * 60_000;
    },
  });
  useEffect(() => {
    document.title = `${title} · original · IGNIS`;
  }, [title]);
  if (!sourceId) return <Gap title="No source named">Open an original from a citation, a search hit, a condition or a run.</Gap>;
  const a = access.data;
  const kind = locatorKind(loc);
  return (
    <div className="view">
      <div className="view-head">
        <h1 style={{ overflowWrap: "anywhere" }}>{title}</h1>
        <div className="toolbar">
          {a && <span className="muted small">{MODE_TEXT[a.mode]}</span>}
          {a && httpUrl(a.landing_url) && (
            <a className="btn btn-s" href={a.landing_url} target="_blank" rel="noopener noreferrer">
              Open at NASA ↗
            </a>
          )}
        </div>
      </div>
      <p className="muted small">
        <code>{sourceId}</code>
        {loc.page != null && ` · page ${loc.page}`}
        {loc.row != null && ` · record ${loc.row}`}
        {loc.column && ` · column “${loc.column}”`}
        {loc.pts_s != null && ` · ${loc.pts_s} s`}
      </p>
      {access.isPending && <p className="muted">Finding the original…</p>}
      {access.error && <ErrorNote error={access.error} title="The original cannot be opened from here" />}
      {a && accessExpired(a, Date.now()) && (
        <p className="muted small">
          The access link expired.{" "}
          <button className="linkish" onClick={() => void access.refetch()}>
            Resolve again
          </button>
        </p>
      )}
      {a && !accessExpired(a, Date.now()) && <OriginalBody a={a} loc={loc} kind={kind} title={title} />}
    </div>
  );
}

function OriginalBody({ a, loc, kind, title }: { a: SourceAccess; loc: SourceLocator; kind: ReturnType<typeof locatorKind>; title: string }) {
  const [page, setPage] = useState(loc.page ?? 1);
  if (a.mode === "link_out" || !a.cors) return <Gap title="Not streamed by IGNIS">This source opens at the provider. Use “Open at NASA”.</Gap>;
  const isPdf = kind === "page" || /\.pdf$/i.test(title);
  if (isPdf)
    return (
      <section className="card" style={{ padding: 12 }}>
        <div className="toolbar" style={{ marginBottom: 8 }}>
          <button className="btn btn-s" disabled={page <= 1} onClick={() => setPage((p) => Math.max(1, p - 1))}>
            ← Page
          </button>
          <span className="num">page {page}</span>
          <button className="btn btn-s" onClick={() => setPage((p) => p + 1)}>
            Page →
          </button>
          {loc.page != null && page !== loc.page && (
            <button className="linkish" onClick={() => setPage(loc.page!)}>
              back to the cited page {loc.page}
            </button>
          )}
        </div>
        <Suspense fallback={<p className="muted small">Loading the PDF viewer…</p>}>
          <PdfPage url={a.url} range={a.range} length={null} page={page} bbox={page === loc.page ? loc.bbox : null} title={title} />
        </Suspense>
      </section>
    );
  if (kind === "video") return <VideoOriginal url={a.url} startS={videoTimeS(loc)} title={title} />;
  return <TableOriginal url={a.url} row={loc.row ?? null} column={loc.column ?? null} />;
}

const CONTEXT = 12;

/** The cited record of a CSV/text original with its neighbours and the header, read from the start
 * of the file only as far as needed (the stream is cancelled after the last shown record). */
function TableOriginal({ url, row, column }: { url: string; row: number | null; column: string | null }) {
  const want = row ?? 1;
  const last = want + CONTEXT;
  const q = useQuery({
    queryKey: ["original-table", url, want],
    retry: false,
    queryFn: async ({ signal }) => {
      const r = await fetch(url, { signal });
      if (!r.ok || !r.body) throw new Error(`The source answered ${r.status}.`);
      const reader = r.body.getReader();
      const dec = new TextDecoder();
      const st = csvStart();
      const recs: string[] = [];
      let bytes = 0;
      let ended = false;
      while (recs.length < last) {
        const { value, done } = await reader.read();
        if (done) {
          recs.push(...csvFeed(st, dec.decode()), ...csvEnd(st));
          ended = true;
          break;
        }
        bytes += value.byteLength;
        recs.push(...csvFeed(st, dec.decode(value, { stream: true })));
      }
      if (!ended) await reader.cancel();
      return { recs, bytes, ended };
    },
  });
  if (q.isPending) return <p className="muted">Reading the original up to record {want}…</p>;
  if (q.error) return <ErrorNote error={q.error} title="The original could not be read" />;
  const { recs, bytes, ended } = q.data;
  const header = recs[0] != null ? csvFields(recs[0]) : [];
  const col = column ? header.findIndex((h) => h.trim() === column.trim()) : -1;
  const from = Math.max(2, want - CONTEXT);
  const shown = recs.slice(from - 1, last).map((r, i) => ({ n: from + i, cells: csvFields(r) }));
  return (
    <section className="card" style={{ padding: 12 }}>
      <p className="muted small">
        Records {from}–{from + shown.length - 1} of the original (header = record 1){row != null && `; the cited record ${row} is marked`}. Read {(bytes / 1024).toFixed(0)} KiB from NASA{ended ? " (whole file)" : ", stopped after the shown records"}.
      </p>
      {row != null && row > recs.length && <Gap title="The cited record is past the end of the file">The file has {recs.length} records.</Gap>}
      <div className="tbl-wrap">
        <table className="tbl">
          <thead>
            <tr>
              <th className="r">#</th>
              {header.map((h, i) => (
                <th key={i} style={i === col ? { background: "var(--accent-soft, rgba(255,170,0,.18))" } : undefined}>
                  {h}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {shown.map((r) => (
              <tr key={r.n} style={r.n === row ? { outline: "2px solid var(--accent, #e8a33d)", outlineOffset: -2 } : undefined} aria-current={r.n === row ? "true" : undefined}>
                <td className="r num dim">{r.n}</td>
                {r.cells.map((c, i) => (
                  <td key={i} className="num" style={r.n === row && i === col ? { fontWeight: 700, background: "var(--accent-soft, rgba(255,170,0,.25))" } : undefined}>
                    {c}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
