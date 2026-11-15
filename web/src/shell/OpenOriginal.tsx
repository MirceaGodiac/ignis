// "Original ↗": opens the NASA original behind a locator in a new tab (#/original, views/Original),
// at the cited page, region, record or time. Delivery goes through SourceAccess (M12); the link
// itself carries only the source id and the locator, never a signed URL.
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

export function OpenOriginal({ sourceId, loc, title, label = "Original ↗", className = "btn btn-s" }: { sourceId: string; loc?: SourceLocator | null; title?: string | null; label?: string; className?: string }) {
  return (
    <a className={className} href={originalHref(sourceId, loc, title)} target="_blank" rel="noopener" onClick={(e) => e.stopPropagation()} title="Open the NASA original at this place in a new tab">
      {label}
    </a>
  );
}
