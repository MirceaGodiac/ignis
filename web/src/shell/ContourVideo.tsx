// T7 part 3: a BASS-style video with our leading-edge contour drawn over it, frame by frame.
// The video streams from NASA through `SourceAccess`; the contour is our own artifact. The layer is
// SVG in the video's own pixel space (no pixel readback, so no CORS on the video is needed), and it is
// labelled as image pixels: a `pixel_only` contour is never put on a metric axis.
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ImageContourObservation, VisualObservationBundle } from "../contracts/generated";
import { contourAt, frameTolerance, overlayUnits, parseNdjson, polylines, STATUS_TEXT } from "../research/contours";
import { accessExpired } from "../research/sourceAccess";
import { ErrorNote } from "../ui/common";
import { useData } from "./data";
import { useLinkedTime } from "./linked";

type FrameVideo = HTMLVideoElement & {
  requestVideoFrameCallback?: (cb: (now: number, meta: { mediaTime: number }) => void) => number;
  cancelVideoFrameCallback?: (h: number) => void;
};

const NO_RECORDS: ImageContourObservation[] = [];

export function ContourVideo({ bundle }: { bundle: VisualObservationBundle }) {
  const { client, researchMode } = useData();
  const units = overlayUnits(bundle);
  const access = useQuery({ queryKey: [researchMode, "access", bundle.video_source_id], queryFn: () => client.sourceAccess(bundle.video_source_id), retry: false });
  const records = useQuery({
    queryKey: [researchMode, "contours", bundle.contours.artifact_id],
    queryFn: async () => parseNdjson<ImageContourObservation>(await client.getArtifactText(bundle.contours.artifact_id)).sort((a, b) => a.pts_s - b.pts_s),
    enabled: units != null,
    retry: false,
    staleTime: Infinity, // content-addressed artifact
  });
  if (!units) return <p className="note note-refusal small">This bundle is metric; its records are a different contract and are not drawn here.</p>;
  return (
    <div className="panel-sec">
      <h3>Video with contour</h3>
      {access.error && <ErrorNote error={access.error} title="The video cannot be opened from here" />}
      {records.error && <ErrorNote error={records.error} title="The contour records could not be read" />}
      {access.data && accessExpired(access.data, Date.now()) && <p className="muted small">The video link expired at {access.data.expires_at}.</p>}
      {access.data && !accessExpired(access.data, Date.now()) && access.data.mode !== "link_out" && (
        <Player url={access.data.url} records={records.data ?? NO_RECORDS} units={units} timeOrigin={bundle.time_origin} />
      )}
      {access.data?.mode === "link_out" && <p className="muted small">This video is not streamed by IGNIS; the contour cannot be drawn over it here.</p>}
    </div>
  );
}

function Player({ url, records, units, timeOrigin }: { url: string; records: ImageContourObservation[]; units: "pixel_only" | "normalized"; timeOrigin: string }) {
  const v = useRef<FrameVideo>(null);
  const [size, setSize] = useState<{ w: number; h: number } | null>(null);
  const [current, setCurrent] = useState<ImageContourObservation | null>(null);
  const [failed, setFailed] = useState(false);
  const tol = useMemo(() => frameTolerance(records), [records]);
  // Linked views (T8): our frame's time since the origin goes to the answer's store; a time set by
  // another view on the same origin moves the paused video there (pts = time_s + the bundle's offset).
  const linkId = useId();
  const { time, setTime } = useLinkedTime();
  const setRef = useRef(setTime);
  setRef.current = setTime;
  useEffect(() => {
    const el = v.current;
    if (!el || !time || time.by === linkId || time.origin !== timeOrigin || records.length === 0 || !el.paused) return;
    const offset = records[0].pts_s - records[0].time_s;
    el.currentTime = Math.max(0, time.t + offset);
  }, [time, linkId, timeOrigin, records]);

  useEffect(() => {
    const el = v.current;
    if (!el) return;
    let handle = 0;
    let last: string | null = null;
    const show = (t: number) => {
      const r = contourAt(records, t, tol);
      // Re-render only when the shown frame changes, not on every painted frame.
      if ((r?.id ?? null) !== last) {
        last = r?.id ?? null;
        setCurrent(r);
        if (r && !el.paused) setRef.current({ origin: timeOrigin, t: r.time_s, by: linkId });
      }
    };
    const onMeta = () => setSize({ w: el.videoWidth, h: el.videoHeight });
    el.addEventListener("loadedmetadata", onMeta);
    if (el.readyState >= 1) onMeta();
    // requestVideoFrameCallback gives the presentation time of the frame actually painted; where it
    // is missing, timeupdate and seeked are coarser but still correct on pause and seek.
    if (el.requestVideoFrameCallback) {
      const tick = (_: number, meta: { mediaTime: number }) => {
        show(meta.mediaTime);
        handle = el.requestVideoFrameCallback!(tick);
      };
      handle = el.requestVideoFrameCallback(tick);
    }
    const onTime = () => show(el.currentTime);
    el.addEventListener("seeked", onTime);
    el.addEventListener("timeupdate", onTime);
    show(el.currentTime);
    return () => {
      el.removeEventListener("loadedmetadata", onMeta);
      el.removeEventListener("seeked", onTime);
      el.removeEventListener("timeupdate", onTime);
      if (handle) el.cancelVideoFrameCallback?.(handle);
    };
  }, [records, tol, url, timeOrigin, linkId]);

  const seekTo = (r: ImageContourObservation) => {
    const el = v.current;
    if (el) el.currentTime = r.pts_s;
  };
  const vb = units === "normalized" ? "0 0 1 1" : size ? `0 0 ${size.w} ${size.h}` : null;
  const stroke = units === "normalized" ? 0.004 : Math.max(1.5, (size?.w ?? 640) / 320);

  return (
    <>
      {failed && <p className="note note-refusal small">The video could not be played here. Open it at NASA.</p>}
      <div className="contour-video" hidden={failed}>
        <video ref={v} src={url} controls preload="metadata" playsInline width={640} height={360} onError={() => setFailed(true)} aria-label="Source video" />
        {vb && current && (
          <svg viewBox={vb} preserveAspectRatio="none" aria-hidden="true">
            {polylines(current).map((pts, i) =>
              pts.length === 1 ? (
                <circle key={i} cx={pts[0][0]} cy={pts[0][1]} r={stroke * 1.5} />
              ) : (
                <polyline key={i} points={pts.map((p) => p.join(",")).join(" ")} strokeWidth={stroke} />
              ),
            )}
          </svg>
        )}
      </div>
      <div className="stat-line small" aria-live="polite">
        {current ? (
          <>
            <span className="num">frame {current.frame}</span>
            <span className="num">{current.time_s.toFixed(3)} s since {timeOrigin.replace(/_/g, " ")}</span>
            <span className="muted">{STATUS_TEXT[current.status]}</span>
          </>
        ) : (
          <span className="muted">No contour on this frame. {records.length} contoured frames in this bundle.</span>
        )}
      </div>
      <p className="muted small">Contour in {units === "normalized" ? "normalized image coordinates" : "image pixels"}, not metres: the camera is not calibrated for this bundle.</p>
      {records.length > 0 && (
        <div className="stat-line small">
          <span className="muted">Jump to</span>
          {records.slice(0, 12).map((r) => (
            <button key={r.id} className="linkish num" onClick={() => seekTo(r)}>
              {r.frame}
            </button>
          ))}
          {records.length > 12 && <span className="muted">+{records.length - 12} more</span>}
        </div>
      )}
    </>
  );
}
