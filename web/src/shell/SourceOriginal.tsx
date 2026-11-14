// T7 part 2: the original behind a citation, opened through K1.18 `SourceAccess` (route M12).
// NASA bytes are never stored by IGNIS: `direct` streams from NASA's S3, `proxy` passes through the
// Worker, `link_out` only links. "Open at NASA" always goes to the provider's landing page, never to
// the signed URL (PSI forces a download on it).
import { lazy, Suspense, useEffect, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { EvidenceRef, SourceAccess } from "../contracts/generated";
import { accessExpired, videoTimeS, viewerFor } from "../research/sourceAccess";
import { ErrorNote } from "../ui/common";
import { useData } from "./data";

const PdfPage = lazy(() => import("./PdfPage"));

const MODE_TEXT: Record<SourceAccess["mode"], string> = {
  direct: "Streamed from NASA",
  proxy: "Through the IGNIS proxy; nothing is stored",
  link_out: "Opens at the provider",
};

export function SourceOriginal({ evidence }: { evidence: EvidenceRef }) {
  const { client, researchMode } = useData();
  const access = useQuery({
    queryKey: [researchMode, "access", evidence.source_id],
    queryFn: () => client.sourceAccess(evidence.source_id),
    retry: false,
    // A signed URL is short-lived: never serve a cached one past its expiry.
    staleTime: (q) => {
      const a = q.state.data;
      return a?.expires_at ? Math.max(0, Date.parse(a.expires_at) - Date.now() - 30_000) : 5 * 60_000;
    },
  });
  const a = access.data;
  const expired = !!a && accessExpired(a, Date.now());
  // An expired signed URL is resolved again once by itself; after that the researcher decides.
  const retried = useRef(false);
  useEffect(() => {
    if (expired && !retried.current) {
      retried.current = true;
      void access.refetch();
    }
  }, [expired, access]);
  if (access.isPending) return <p className="muted small">Finding the original…</p>;
  if (access.error || !a) return <ErrorNote error={access.error} title="The original cannot be opened from here; the parsed text below is what IGNIS read" />;
  const kind = viewerFor(a, evidence.locator);
  const loc = evidence.locator;
  return (
    <div className="panel-sec">
      <div className="toolbar">
        <h3 style={{ margin: 0 }}>Original</h3>
        <span className="muted small">{MODE_TEXT[a.mode]}</span>
        {httpUrl(a.landing_url) && (
          <a className="btn btn-s" style={{ marginLeft: "auto" }} href={a.landing_url} target="_blank" rel="noopener noreferrer">
            Open at NASA ↗
          </a>
        )}
      </div>
      {expired ? (
        <p className="muted small">
          The access link expired at {a.expires_at}.{" "}
          <button className="linkish" disabled={access.isFetching} onClick={() => void access.refetch()}>
            {access.isFetching ? "Resolving…" : "Resolve again"}
          </button>
        </p>
      ) : kind === "pdf" && loc?.page != null ? (
        <Suspense fallback={<p className="muted small">Loading the PDF viewer…</p>}>
          <PdfPage url={a.url} range={a.range} length={null} page={loc.page} bbox={loc.bbox} title={`Source ${evidence.source_id}`} />
        </Suspense>
      ) : kind === "video" ? (
        <VideoOriginal url={a.url} startS={videoTimeS(loc)} title={`Source ${evidence.source_id}`} />
      ) : (
        <p className="muted small">{a.mode === "link_out" ? "This source is not streamed by IGNIS. Open it at the provider." : "This citation has no page or video position to open."}</p>
      )}
    </div>
  );
}

/** Only http(s) links are rendered; a `javascript:` or `data:` landing URL from a manifest is never a link. */
function httpUrl(u: string | null | undefined): boolean {
  try {
    const p = new URL(u ?? "").protocol;
    return p === "https:" || p === "http:";
  } catch {
    return false;
  }
}

function VideoOriginal({ url, startS, title }: { url: string; startS: number | null; title: string }) {
  const v = useRef<HTMLVideoElement>(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => setFailed(false), [url]);
  useEffect(() => {
    const el = v.current;
    if (!el || startS == null) return;
    const seek = () => {
      el.currentTime = startS;
    };
    if (el.readyState >= 1) seek();
    else el.addEventListener("loadedmetadata", seek, { once: true });
    return () => el.removeEventListener("loadedmetadata", seek);
  }, [url, startS]);
  return (
    <>
      {/* Explicit width and height keep the box stable before metadata arrives; preload only metadata. */}
      {failed && <p className="note note-refusal small">The video could not be played here. Open it at NASA.</p>}
      <video ref={v} src={url} onError={() => setFailed(true)} hidden={failed} controls preload="metadata" playsInline width={640} height={360} style={{ width: "100%", height: "auto", background: "#000", borderRadius: 6 }} aria-label={title} />
      {startS != null && <p className="muted small num">Cited time: {startS.toFixed(2)} s</p>}
    </>
  );
}
