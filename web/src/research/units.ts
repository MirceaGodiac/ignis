// Display rules for units and locators. Data are SI; display conversion happens only in chart code.
// Pure module: type-only imports (tested with node --test).

import type { OffsetUnit, SourceLocator, UnitsStatus } from "../contracts/generated";

/** Axis unit for spatial values. A `pixel_only` observation never gets a metre axis. */
export function spatialAxis(unitsStatus: UnitsStatus | null | undefined): { unit: "m" | "px" | "1" | null; label: string } {
  switch (unitsStatus) {
    case "metric":
      return { unit: "m", label: "position (m)" };
    case "pixel_only":
      return { unit: "px", label: "position (px, uncalibrated: not metres)" };
    case "normalized":
      return { unit: "1", label: "position (fraction of field, dimensionless)" };
    default:
      return { unit: null, label: "position (units unknown: not plottable)" };
  }
}

/**
 * Slice `text` by a half-open `[start, end)` span counted in `unit` (SourceLocator convention).
 * JS strings index UTF-16 code units, so a `unicode_scalar` span is cut on code points.
 * Returns null for an unsupported unit or an out-of-range span instead of a wrong quote.
 */
export function sliceSpan(text: string, start: number, end: number, unit: OffsetUnit | null | undefined): string | null {
  if (start < 0 || end < start) return null;
  switch (unit) {
    case "unicode_scalar": {
      const cps = Array.from(text);
      return end <= cps.length ? cps.slice(start, end).join("") : null;
    }
    case "utf16_code_unit":
      return end <= text.length ? text.slice(start, end) : null;
    case "utf8_byte": {
      const bytes = new TextEncoder().encode(text);
      if (end > bytes.length) return null;
      try {
        return new TextDecoder("utf-8", { fatal: true }).decode(bytes.slice(start, end));
      } catch {
        return null; // span cuts a multi-byte character
      }
    }
    default:
      return null; // legacy offsets without a unit are not citable
  }
}

export type LocatorKind = "page" | "table" | "video" | "metadata";

/** What a locator points into, so the viewer picks the right presentation. */
export function locatorKind(l: SourceLocator | null | undefined): LocatorKind {
  if (!l) return "metadata";
  if (l.page != null) return "page";
  if (l.frame != null || l.pts_s != null) return "video";
  if (l.row != null || l.column != null || l.path != null) return "table";
  return "metadata";
}

/** `bbox` is [x0, y0, x1, y1] in fractions of the displayed page, origin top-left. CSS percentages. */
export function bboxStyle(b: number[] | null | undefined): { left: string; top: string; width: string; height: string } | null {
  if (!b || b.length !== 4) return null;
  const [x0, y0, x1, y1] = b;
  if (![x0, y0, x1, y1].every((v) => Number.isFinite(v) && v >= 0 && v <= 1) || x1 <= x0 || y1 <= y0) return null;
  const pct = (v: number) => `${(v * 100).toFixed(2)}%`;
  return { left: pct(x0), top: pct(y0), width: pct(x1 - x0), height: pct(y1 - y0) };
}

/** Page shown to people: the printed label when known, with the physical index. */
export function pageCaption(l: SourceLocator): string {
  if (l.page == null) return "no page";
  return l.page_label && l.page_label !== String(l.page) ? `page ${l.page_label} (PDF page ${l.page})` : `page ${l.page}`;
}
