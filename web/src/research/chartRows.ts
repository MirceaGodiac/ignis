// Pure part of the answer views that draw numbers (node --test runs it): the rows a `chart` view
// plots, compact number display, axis ticks, and the model of an `artifact_table`.

import type { ViewSpec } from "../contracts/generated";
import { resolvePointer } from "./units.ts";

export type ChartView = Extract<ViewSpec, { kind: "chart" }>;

// ------------------------------------------------------------------ numbers

/**
 * A number shown in at most 4 significant digits (the full value goes in a title tooltip).
 * Integers below a million are shown whole (counts, years, sample numbers); very large and very
 * small magnitudes switch to exponent form.
 */
export function fmtNum(v: number): string {
  if (!Number.isFinite(v)) return String(v);
  if (v === 0) return "0";
  const a = Math.abs(v);
  if (Number.isInteger(v) && a < 1e6) return String(v);
  if (a >= 1e6 || a < 1e-3) {
    const [m, e] = v.toExponential(3).split("e");
    return `${String(+m)}e${e.replace("+", "")}`;
  }
  return String(+v.toPrecision(4));
}

/** The full value, for a tooltip, when the shown one is shorter. */
export const fullNum = (v: number): string | undefined => (fmtNum(v) === String(v) ? undefined : String(v));

// ------------------------------------------------------------------ axis ticks

/** Round tick values covering [min, max] (about `count` of them). */
export function niceTicks(min: number, max: number, count = 5): number[] {
  if (!Number.isFinite(min) || !Number.isFinite(max)) return [];
  if (min === max) {
    const d = min === 0 ? 1 : Math.abs(min) * 0.1;
    min -= d;
    max += d;
  }
  const span = max - min;
  const raw = span / Math.max(1, count);
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((k) => k * mag).find((s) => span / s <= count + 0.5) ?? 10 * mag;
  const out: number[] = [];
  for (let t = Math.floor(min / step) * step; t <= max + step * 1e-9; t += step) out.push(+t.toPrecision(12));
  if (out[0] > min) out.unshift(+(out[0] - step).toPrecision(12));
  if (out[out.length - 1] < max) out.push(+(out[out.length - 1] + step).toPrecision(12));
  return out;
}

/** Whole decades covering [min, max] (both > 0), and ticks at each power of ten (1-2-5 steps when fewer than three decades). */
export function logTicks(min: number, max: number): { domain: [number, number]; ticks: number[] } | null {
  if (!(min > 0) || !(max > 0)) return null;
  const lo = Math.floor(Math.log10(min));
  let hi = Math.ceil(Math.log10(max));
  if (hi === lo) hi = lo + 1;
  const top = +(10 ** hi).toPrecision(12);
  const fine = hi - lo < 3;
  const ticks: number[] = [];
  for (let e = lo; e <= hi; e++) for (const k of fine ? [1, 2, 5] : [1]) ticks.push(+(k * 10 ** e).toPrecision(12));
  return { domain: [+(10 ** lo).toPrecision(12), top], ticks: ticks.filter((t) => t <= top) };
}

// ------------------------------------------------------------------ chart rows

export interface ChartPoint {
  /** Index of the row in the artifact array. */
  row: number;
  x: number | string;
  /** One value per y axis; null where that axis has no plottable value in this row. */
  y: (number | null)[];
  group: string | null;
  label: string | null;
}

export interface ChartData {
  points: ChartPoint[];
  /** x is categorical (bar mark, or text x in every row): drawn as bars or dots at category positions. */
  categorical: boolean;
  /** Category order of first appearance (categorical only). */
  categories: string[];
  /** Group values in order of first appearance; [null] when the view names no group. */
  groups: (string | null)[];
  /** Rows not drawn, and why (counted per reason). */
  skipped: number;
  skippedWhy: Record<string, number>;
  total: number;
}

const finite = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v);
const keyText = (v: unknown): string | null => (typeof v === "string" ? v : finite(v) || typeof v === "boolean" ? String(v) : null);

/** The rows a chart view plots, read by JSON Pointer relative to each row, or the reason it cannot be drawn. */
export function chartRows(doc: unknown, view: ChartView): ChartData | string {
  const rows = resolvePointer(doc, view.pointer);
  if (rows === undefined) return `${view.pointer || "the document"} does not exist in the artifact`;
  if (!Array.isArray(rows)) return `${view.pointer || "the document"} is not an array of rows`;
  if (!view.y.length) return "the chart names no y field";
  const ylog = view.y.some((a) => a.log);
  const xs = rows.map((r) => resolvePointer(r, view.x.field));
  // Bars always read x as categories; otherwise x is categorical only when no row has a numeric x
  // (a stray text value in a numeric column is skipped, not turned into a category).
  const categorical = view.mark === "bar" || (!xs.some(finite) && xs.some((x) => typeof x === "string"));
  const why: Record<string, number> = {};
  const skip = (reason: string) => (why[reason] = (why[reason] ?? 0) + 1);
  const points: ChartPoint[] = [];
  rows.forEach((r, row) => {
    const xv = xs[row];
    let x: number | string;
    if (categorical) {
      const k = keyText(xv);
      if (k == null) return skip(`no ${view.x.title}`);
      x = k;
    } else {
      if (!finite(xv)) return skip(`${view.x.title} is not a number`);
      if (view.x.log && xv <= 0) return skip(`${view.x.title} ≤ 0 on a log axis`);
      x = xv;
    }
    let anyY = false;
    let nonPositive = false;
    const y = view.y.map((a) => {
      const v = resolvePointer(r, a.field);
      if (!finite(v)) return null;
      if (ylog && v <= 0) {
        nonPositive = true;
        return null;
      }
      anyY = true;
      return v;
    });
    if (!anyY) return skip(nonPositive ? "value ≤ 0 on a log axis" : view.y.length === 1 ? `${view.y[0].title} is not a number` : "no y value is a number");
    const group = view.group ? keyText(resolvePointer(r, view.group)) : null;
    const label = view.label ? keyText(resolvePointer(r, view.label)) : null;
    points.push({ row, x, y, group, label });
  });
  const categories = categorical ? [...new Set(points.map((p) => p.x as string))] : [];
  const groups = view.group ? [...new Set(points.map((p) => p.group))] : [null];
  const skipped = rows.length - points.length;
  return { points, categorical, categories, groups, skipped, skippedWhy: why, total: rows.length };
}

/** "3 rows not drawn (2: pressure is not a number; 1: …)" or null. */
export function skippedNote(d: ChartData): string | null {
  if (!d.skipped) return null;
  const parts = Object.entries(d.skippedWhy).map(([w, n]) => `${n}: ${w}`);
  return `${d.skipped} of ${d.total} rows not drawn (${parts.join("; ")})`;
}

/** Axis title with its unit: "Pressure (kPa)". */
export const axisTitle = (a: { title: string; unit?: string | null }) => (shownUnit(a.unit) ? `${a.title} (${a.unit})` : a.title);

/** The unit shown next to a value; the dimensionless unit "1" (a fraction such as O2) is not printed. */
export const shownUnit = (u: string | null | undefined): string | null => (u && u !== "1" ? u : null);

// ------------------------------------------------------------------ artifact tables

export interface TableColumn {
  key: string;
  title: string;
  unit: string | null;
  numeric: boolean;
}

const UNIT_SUFFIX: Record<string, string> = {
  s: "s", ms: "ms", us: "µs", min: "min", h: "h", m: "m", mm: "mm", cm: "cm", um: "µm", px: "px",
  kpa: "kPa", pa: "Pa", mpa: "MPa", bar: "bar", atm: "atm", psi: "psi", torr: "Torr",
  k: "K", c: "°C", pct: "%", percent: "%", hz: "Hz", kg: "kg", g: "g", mg: "mg", w: "W", kw: "kW", j: "J", kj: "kJ",
  mps: "m/s", mmps: "mm/s", cms: "cm/s", mm2s: "mm²/s", mm2ps: "mm²/s", m2: "m²", mm2: "mm²", m3: "m³", l: "L", ml: "mL", slpm: "slpm", sccm: "sccm",
};

/** "pressure_kpa" → title "pressure", unit "kPa"; "o2_mole_fraction" stays as it is, with spaces. */
export function columnTitle(key: string): { title: string; unit: string | null } {
  const m = /^(.+?)[_.\s]([a-z0-9]+)$/i.exec(key);
  const unit = m ? UNIT_SUFFIX[m[2].toLowerCase()] : undefined;
  const base = unit ? m![1] : key;
  return { title: base.replace(/[_]+/g, " ").replace(/\./g, " · ").trim(), unit: unit ?? null };
}

const plainObj = (v: unknown): v is Record<string, unknown> => !!v && typeof v === "object" && !Array.isArray(v);

/**
 * The model of an `artifact_table`: an array of objects becomes rows (one level of nested objects
 * flattened into `parent.child` columns), an object becomes field/value rows, an array of scalars
 * one value column. Columns whose every present value is a number are numeric.
 */
export function tableModel(v: unknown): { columns: TableColumn[]; rows: unknown[][] } | null {
  let keys: string[];
  let rows: unknown[][];
  if (Array.isArray(v) && v.length > 0 && v.every(plainObj)) {
    const flat = (v as Record<string, unknown>[]).map((r) => {
      const o: Record<string, unknown> = {};
      for (const [k, x] of Object.entries(r)) {
        if (plainObj(x) && Object.keys(x).length > 0 && Object.keys(x).length <= 12) for (const [k2, x2] of Object.entries(x)) o[`${k}.${k2}`] = x2;
        else o[k] = x;
      }
      return o;
    });
    keys = [...new Set(flat.flatMap((r) => Object.keys(r)))];
    rows = flat.map((r) => keys.map((k) => r[k]));
  } else if (plainObj(v)) {
    keys = ["field", "value"];
    rows = Object.entries(v);
  } else if (Array.isArray(v)) {
    keys = ["value"];
    rows = v.map((x) => [x]);
  } else return null;
  const columns = keys.map((key, j) => {
    const present = rows.map((r) => r[j]).filter((x) => x != null);
    return { key, ...columnTitle(key), numeric: present.length > 0 && present.every(finite) };
  });
  return { columns, rows };
}

/** Rows sorted by one column: numbers numerically, text naturally, empty cells last either way. */
export function sortRows(rows: unknown[][], col: number, dir: "asc" | "desc"): unknown[][] {
  const sign = dir === "asc" ? 1 : -1;
  const coll = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });
  return rows
    .map((r, i) => ({ r, i }))
    .sort((a, b) => {
      const x = a.r[col];
      const y = b.r[col];
      if (x == null || y == null) return x == null && y == null ? a.i - b.i : x == null ? 1 : -1;
      const c = finite(x) && finite(y) ? x - y : coll.compare(typeof x === "object" ? JSON.stringify(x) : String(x), typeof y === "object" ? JSON.stringify(y) : String(y));
      return c * sign || a.i - b.i;
    })
    .map((e) => e.r);
}
