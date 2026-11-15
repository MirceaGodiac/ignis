// "Original ↗": opens the NASA original behind a locator at the cited page, region, record or time,
// inside the app's side panel, so the page you were on stays put (from inside the panel it is pushed
// on the panel's trail, so Back returns). `newTab` opens #/original in a new tab instead. Delivery
// goes through SourceAccess (M12); a link carries only the source id and the locator, never a
// signed URL.
import type { SourceLocator } from "../contracts/generated";
import { usePanel } from "./inspector";
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

export function OpenOriginal({
  sourceId,
  loc,
  title,
  label = "Original",
  className = "btn btn-s",
  newTab = false,
}: {
  sourceId: string;
  loc?: SourceLocator | null;
  title?: string | null;
  label?: string;
  className?: string;
  newTab?: boolean;
}) {
  const panel = usePanel();
  if (newTab)
    return (
      <a className={className} href={originalHref(sourceId, loc, title)} target="_blank" rel="noopener" onClick={(e) => e.stopPropagation()} title="Open the NASA original in a new tab">
        {label}
      </a>
    );
  return (
    <button
      className={className}
      title="Show the NASA original at this place"
      onClick={(e) => {
        e.stopPropagation();
        const item = { kind: "original" as const, sourceId, loc: loc ?? null, title: title ?? loc?.path ?? null };
        if (panel.current) panel.push(item);
        else panel.open(item);
      }}
    >
      {label}
    </button>
  );
}
