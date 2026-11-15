// T7 part 2: one PDF page from a NASA original, with the cited region marked. Loaded lazily, so
// PDF.js and its worker are fetched only when a citation is opened.
//
// Bytes: a custom PDFDataRangeTransport fetches only the ranges PDF.js asks for. PSI's S3 exposes only
// `ETag` to scripts, so PDF.js's own loader cannot read Content-Length and would download whole files;
// with the transport, the length comes from the caller (source manifest) or from a 1-byte probe whose
// Content-Range is readable (same-origin proxy), else from the Content-Length of a GET dropped after its
// headers (PSI). Without any of these, the file is read whole.
import { useEffect, useRef, useState } from "react";
import { getDocument, GlobalWorkerOptions, PDFDataRangeTransport, TextLayer, type PDFDocumentLoadingTask, type PDFDocumentProxy, type RenderTask } from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { bboxStyle } from "../research/units";
import { fmtBytes } from "../ui/common";
import { clampPage, contentLength, contentRangeLength, rangeHeader } from "../research/sourceAccess";

GlobalWorkerOptions.workerSrc = workerUrl;

const CHUNK = 256 * 1024;

class FetchRangeTransport extends PDFDataRangeTransport {
  constructor(
    length: number,
    private url: string,
    private signal: AbortSignal,
    private onError: (e: unknown) => void,
  ) {
    super(length, null);
  }
  override requestDataRange(begin: number, end: number) {
    fetch(this.url, { headers: { Range: rangeHeader(begin, end) }, signal: this.signal })
      .then(async (r) => {
        if (r.status !== 206 && r.status !== 200) throw new Error(`The source answered ${r.status} for bytes ${begin}–${end - 1}.`);
        const buf = new Uint8Array(await r.arrayBuffer());
        // A 200 means the server ignored Range and sent the whole file: hand PDF.js the slice it asked for.
        // A 206 longer than asked (some servers ignore the end of the range) is cut to the request.
        this.onDataRange(begin, r.status === 200 ? buf.subarray(begin, end) : buf.subarray(0, end - begin));
      })
      .catch((e) => {
        if (!this.signal.aborted) this.onError(e);
      });
  }
}

export interface PdfLoadStats {
  mode: "range" | "whole";
  bytes: number;
}

/**
 * The server honours Range but hides `Content-Range` (PSI's S3 exposes only `ETag`). `Content-Length`
 * is CORS-safelisted, so a plain GET reveals the full size; the body is dropped once the headers are
 * in. HEAD is not an option: a presigned S3 URL is signed for GET only.
 */
async function lengthFromGet(url: string, signal: AbortSignal): Promise<number | null> {
  const ac = new AbortController();
  const stop = () => ac.abort();
  signal.addEventListener("abort", stop, { once: true });
  try {
    const r = await fetch(url, { signal: ac.signal });
    return r.ok ? contentLength(r.headers.get("Content-Length")) : null;
  } finally {
    ac.abort();
    signal.removeEventListener("abort", stop);
  }
}

async function openPdf(url: string, range: boolean, knownLength: number | null, signal: AbortSignal, onError: (e: unknown) => void, stats: PdfLoadStats): Promise<PDFDocumentLoadingTask> {
  let length = knownLength;
  if (range && length == null) {
    const probe = await fetch(url, { headers: { Range: "bytes=0-0" }, signal });
    if (probe.status === 206) {
      length = contentRangeLength(probe.headers.get("Content-Range"));
      await probe.body?.cancel();
      if (length == null) length = await lengthFromGet(url, signal);
    } else if (probe.ok) {
      const data = new Uint8Array(await probe.arrayBuffer());
      stats.mode = "whole";
      stats.bytes = data.byteLength;
      return getDocument({ data });
    } else {
      throw new Error(`The source answered ${probe.status}.`);
    }
  }
  if (range && length != null) {
    const transport = new FetchRangeTransport(length, url, signal, onError);
    const counted = transport.onDataRange.bind(transport);
    transport.onDataRange = (begin, chunk) => {
      stats.bytes += chunk?.byteLength ?? 0;
      counted(begin, chunk);
    };
    stats.mode = "range";
    return getDocument({ range: transport, rangeChunkSize: CHUNK, disableAutoFetch: true, disableStream: true });
  }
  const res = await fetch(url, { signal });
  if (!res.ok) throw new Error(`The source answered ${res.status}.`);
  const data = new Uint8Array(await res.arrayBuffer());
  stats.mode = "whole";
  stats.bytes = data.byteLength;
  return getDocument({ data });
}

export default function PdfPage({ url, range, length, page, bbox, title }: { url: string; range: boolean; length: number | null; page: number; bbox: number[] | null | undefined; title: string }) {
  const [doc, setDoc] = useState<PDFDocumentProxy | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [current, setCurrent] = useState(page);
  const [width, setWidth] = useState(0);
  const [stats] = useState<PdfLoadStats>({ mode: "range", bytes: 0 });
  const [, tick] = useState(0);
  const wrap = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const text = useRef<HTMLDivElement>(null);

  useEffect(() => setCurrent(page), [page]);

  useEffect(() => {
    const ac = new AbortController();
    let loading: PDFDocumentLoadingTask | null = null;
    setDoc(null);
    setError(null);
    stats.bytes = 0;
    const fail = (e: unknown) => setError(e instanceof Error ? e.message : String(e));
    openPdf(url, range, length, ac.signal, fail, stats)
      .then((t) => {
        loading = t;
        if (ac.signal.aborted) void t.destroy();
        return t.promise;
      })
      .then((d) => {
        if (!ac.signal.aborted) setDoc(d);
      })
      .catch((e) => {
        if (!ac.signal.aborted) fail(e);
      });
    return () => {
      ac.abort();
      void loading?.destroy();
    };
  }, [url, range, length, stats]);

  useEffect(() => {
    const el = wrap.current;
    if (!el) return;
    // Measure now: ResizeObserver reports only on a rendered frame, which a background tab may not have.
    setWidth(el.clientWidth);
    const ro = new ResizeObserver(([e]) => setWidth(Math.floor(e.contentRect.width)));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  useEffect(() => {
    if (!doc || !width || !canvas.current || !text.current) return;
    let task: RenderTask | null = null;
    let layer: TextLayer | null = null;
    let cancelled = false;
    (async () => {
      const p = await doc.getPage(clampPage(current, doc.numPages));
      if (cancelled) return;
      const base = p.getViewport({ scale: 1 });
      const viewport = p.getViewport({ scale: width / base.width });
      const dpr = window.devicePixelRatio || 1;
      const c = canvas.current!;
      c.width = Math.floor(viewport.width * dpr);
      c.height = Math.floor(viewport.height * dpr);
      c.style.width = `${Math.floor(viewport.width)}px`;
      c.style.height = `${Math.floor(viewport.height)}px`;
      task = p.render({ canvas: c, viewport, transform: dpr !== 1 ? [dpr, 0, 0, dpr, 0, 0] : undefined });
      await task.promise;
      if (cancelled) return;
      // Text layer: selectable, searchable and readable by assistive technology (the canvas is not).
      const t = text.current!;
      t.replaceChildren();
      t.style.setProperty("--scale-factor", String(viewport.scale));
      layer = new TextLayer({ textContentSource: p.streamTextContent(), container: t, viewport });
      await layer.render();
      tick((n) => n + 1);
    })().catch((e) => {
      if (!cancelled && (e as { name?: string })?.name !== "RenderingCancelledException") setError(e instanceof Error ? e.message : String(e));
    });
    return () => {
      cancelled = true;
      task?.cancel();
      layer?.cancel();
    };
  }, [doc, current, width]);

  const pages = doc?.numPages ?? 0;
  const box = current === page ? bboxStyle(bbox) : null;
  return (
    <div className="pdf">
      <div className="toolbar">
        <button className="btn btn-s" disabled={!doc || current <= 1} onClick={() => setCurrent((c) => c - 1)} aria-label="Previous page">
          ‹
        </button>
        <span className="small num" aria-live="polite">
          {doc ? `Page ${clampPage(current, pages)} of ${pages}` : error ? "Not loaded" : "Loading…"}
        </span>
        <button className="btn btn-s" disabled={!doc || current >= pages} onClick={() => setCurrent((c) => c + 1)} aria-label="Next page">
          ›
        </button>
        {current !== page && (
          <button className="btn btn-s btn-quiet" onClick={() => setCurrent(page)}>
            Back to cited page
          </button>
        )}
        {doc && (
          <span className="muted small num" style={{ marginLeft: "auto" }} title={stats.mode === "range" ? "Fetched by byte ranges" : "Fetched whole"}>
            {fmtBytes(stats.bytes)} {stats.mode === "range" ? "by range" : "whole"}
          </span>
        )}
      </div>
      {error && <p className="note note-refusal small">The PDF could not be opened here: {error}</p>}
      <div ref={wrap} className="pdf-page" hidden={!!error}>
        <canvas ref={canvas} role="img" aria-label={`${title}, page ${clampPage(current, pages || 1)}`} />
        <div ref={text} className="textLayer" />
        {box && <div className="bbox" style={box} title="Cited region" />}
      </div>
    </div>
  );
}
