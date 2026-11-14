// T7 part 3: the leading-edge contour of a `VisualObservationBundle`, drawn over its video.
// Records are `ImageContourObservation` (NDJSON): image coordinates, never metres. `s` is the
// transverse grid (pixel rows, so the vertical axis), `f` the leading edge along the propagation axis
// at each `s` (the horizontal axis); an absent `f` means no edge on that row and breaks the line.
// A `metric` bundle has `ContourObservation` records instead and is refused here.

import type { ContourStatus, ImageContourObservation, VisualObservationBundle } from "../contracts/generated";

export function parseNdjson<T>(text: string): T[] {
  const out: T[] = [];
  const lines = text.split(/\r?\n/);
  for (let i = 0; i < lines.length; i++) {
    const l = lines[i].trim();
    if (!l) continue;
    try {
      out.push(JSON.parse(l) as T);
    } catch {
      throw new Error(`Contour file, line ${i + 1}: not valid JSON.`);
    }
  }
  return out;
}

/** Only image-space bundles are drawn here; a metric bundle's records are a different contract. */
export function overlayUnits(b: VisualObservationBundle): "pixel_only" | "normalized" | null {
  return b.units_status === "pixel_only" || b.units_status === "normalized" ? b.units_status : null;
}

/** Half the median gap between presentation times: how far from a frame a contour still belongs to it. */
export function frameTolerance(records: readonly ImageContourObservation[]): number {
  const gaps: number[] = [];
  for (let i = 1; i < records.length; i++) gaps.push(records[i].pts_s - records[i - 1].pts_s);
  const g = gaps.filter((x) => x > 0).sort((a, b) => a - b);
  return g.length ? g[Math.floor(g.length / 2)] / 2 : 1 / 60;
}

/** The record for the frame shown at `t` (container presentation time), or null between contoured frames. */
export function contourAt(sorted: readonly ImageContourObservation[], t: number, tol: number): ImageContourObservation | null {
  let lo = 0;
  let hi = sorted.length - 1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (sorted[mid].pts_s < t) lo = mid + 1;
    else hi = mid - 1;
  }
  let best: ImageContourObservation | null = null;
  for (const i of [lo - 1, lo]) {
    const r = sorted[i];
    if (r && Math.abs(r.pts_s - t) <= tol && (!best || Math.abs(r.pts_s - t) < Math.abs(best.pts_s - t))) best = r;
  }
  return best;
}

/** Runs of consecutive rows that have an edge, as [x, y] = [f, s] points. A gap in `f` starts a new run. */
export function polylines(r: ImageContourObservation): [number, number][][] {
  const runs: [number, number][][] = [];
  let cur: [number, number][] = [];
  r.s.forEach((s, i) => {
    const f = r.f[i];
    if (f == null) {
      if (cur.length) runs.push(cur);
      cur = [];
    } else cur.push([f, s]);
  });
  if (cur.length) runs.push(cur);
  return runs;
}

export const STATUS_TEXT: Record<ContourStatus, string> = {
  single_valued: "one edge per row",
  multivalued: "the edge folds back: several edges on a row",
  detached: "the flame is detached",
  occluded: "the edge is hidden",
  out_of_field: "no luminous edge in the field of view",
};
