// "Original ↗": opens the NASA original behind a locator at the cited page, region, record or time,
// in one browser tab next to IGNIS that every later click reuses, so the page you were on stays put
// and you flip between two tabs. Delivery
// goes through SourceAccess (M12); a link carries only the source id and the locator, never a
// signed URL.
import type { SourceLocator } from "../contracts/generated";
import { href } from "./router";

export function originalHref(sourceId: string, loc?: SourceLocator | null, title?: string | null): string {
  const p: Record<string, string | undefined> = { title: title ?? undefined };
  if (loc?.page != null) p.page = String(loc.page);
  if (loc?.bbox && loc.bbox.length === 4) p.bbox = loc.bbox.map((v) => +v.toFixed(5)).join(",");
  if (loc?.row != null) p.row = String(loc.row);
  if (loc?.column) p.col = loc.column;
  if (loc?.pts_s != null) p.t = String(loc.pts_s);
  if (loc?.path) p.path = loc.path;
  return href("original", sourceId, p);
}

/** Source id of a locator's asset (`sha256:<hex>`). */
export const locatorSource = (loc: SourceLocator) => `sha256:${loc.asset_sha256}`;

/** The one browser tab originals open in: every "Original" click reuses it (no rel=noopener, which
 * would make the browser open a fresh tab each time; it is our own page on our own origin). */
export const ORIGINAL_TAB = "ignis-original";

export function OpenOriginal({
  sourceId,
  loc,
  title,
  label = "Original ↗",
  className = "btn btn-s",
}: {
  sourceId: string;
  loc?: SourceLocator | null;
  title?: string | null;
  label?: string;
  className?: string;
  /** Accepted for older call sites; every original opens in the shared tab. */
  newTab?: boolean;
}) {
  return (
    <a className={className} href={originalHref(sourceId, loc, title)} target={ORIGINAL_TAB} onClick={(e) => e.stopPropagation()} title="Open the NASA original in the IGNIS originals tab (reused for every original)">
      {label}
    </a>
  );
}
