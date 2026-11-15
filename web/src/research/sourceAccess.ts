// T7 part 2: how the UI opens a NASA original from a K1.18 `SourceAccess` (LANES_V2 §10).
// The originals stay at NASA: PSI bytes stream straight from S3 (`direct`), NTRS through the Worker
// pass-through proxy (`proxy`), anything else opens at the provider (`link_out`). The access URL is a
// short-lived delivery detail: it is never stored, cited, or put in a plain <a> (PSI sends
// `Content-Disposition: attachment`, so a link would download the file instead of showing it).

import type { SourceAccess, SourceLocator } from "../contracts/generated";
import { locatorKind } from "./units.ts";

export type ViewerKind = "pdf" | "video" | "link";

/** Which viewer opens this citation. The locator decides the medium; `link_out` never streams. */
export function viewerFor(access: SourceAccess, locator: SourceLocator | null | undefined): ViewerKind {
  if (access.mode === "link_out" || !access.cors) return "link";
  const k = locatorKind(locator);
  if (k === "page") return "pdf";
  if (k === "video") return "video";
  return "link";
}

/** A signed URL past `expires_at` must be resolved again before use. */
export function accessExpired(access: SourceAccess, nowMs: number): boolean {
  if (!access.expires_at) return false;
  const t = Date.parse(access.expires_at);
  return Number.isFinite(t) && t <= nowMs;
}

/** Total length from `Content-Range: bytes 0-0/12345`. Null when absent, unknown (`*`) or malformed. */
export function contentRangeLength(header: string | null): number | null {
  const m = header?.match(/^bytes\s+(?:\d+-\d+|\*)\/(\d+)\s*$/i);
  if (!m) return null;
  const n = Number(m[1]);
  return Number.isSafeInteger(n) && n > 0 ? n : null;
}

/** Body length from a `Content-Length` header (CORS-safelisted, so readable when `Content-Range` is not). */
export function contentLength(header: string | null): number | null {
  if (!header || !/^\s*\d+\s*$/.test(header)) return null;
  const n = Number(header);
  return Number.isSafeInteger(n) && n > 0 ? n : null;
}

/** HTTP Range header for the half-open byte span [begin, end) that PDF.js asks for. */
export const rangeHeader = (begin: number, end: number) => `bytes=${begin}-${end - 1}`;

export const clampPage = (p: number, pages: number) => Math.min(Math.max(1, Math.round(p)), Math.max(1, pages));

/** Video time for a locator: presentation time when present, else nothing (a frame index needs a known rate). */
export const videoTimeS = (l: SourceLocator | null | undefined): number | null => (l?.pts_s != null && Number.isFinite(l.pts_s) && l.pts_s >= 0 ? l.pts_s : null);
