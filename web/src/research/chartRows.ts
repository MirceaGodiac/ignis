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

// ------------------------------------------------------------------ display units

/** SI units the reader sees in a friendlier prefix: the base, how many times the prefix applies
 * (m² scales twice) and what follows ("/s"). Only these are rescaled; anything else shows as given. */
const SCALABLE: { re: RegExp; base: "m" | "Pa" | "s"; power: number; tail: string }[] = [
  { re: /^m$/, base: "m", power: 1, tail: "" },
  { re: /^m(\^?2|²)$/, base: "m", power: 2, tail: "" },
  { re: /^m(\^?3|³)$/, base: "m", power: 3, tail: "" },
  { re: /^m\/s$/, base: "m", power: 1, tail: "/s" },
  { re: /^m(\^?2|²)\/s$/, base: "m", power: 2, tail: "/s" },
  { re: /^Pa$/, base: "Pa", power: 1, tail: "" },
  { re: /^s$/, base: "s", power: 1, tail: "" },
];
const PREFIXES: Record<"m" | "Pa" | "s", [string, number][]> = {
  m: [["k", 1e3], ["", 1], ["m", 1e-3], ["µ", 1e-6], ["n", 1e-9]],
  Pa: [["M", 1e6], ["k", 1e3], ["", 1]],
  s: [["", 1], ["m", 1e-3], ["µ", 1e-6]],
};
const POW = ["", "", "²", "³"];

/**
 * The unit a set of SI values reads best in, by magnitude: droplet diameters near 3.4e-3 m become
 * 3.4 mm, a burning-rate constant near 1e-7 m²/s becomes 0.1 mm²/s, 101325 Pa becomes 101 kPa.
 * `factor` multiplies a value into the shown unit. Units it does not know come back unchanged.
 */
export function displayUnit(unit: string, values: Iterable<number | null>): { unit: string; factor: number } {
  const u = unit.trim();
  const spec = SCALABLE.find((s) => s.re.test(u));
  let max = 0;
  for (const v of values) if (v != null && Number.isFinite(v)) max = Math.max(max, Math.abs(v));
  if (!spec || max === 0) return { unit: u, factor: 1 };
  const steps = PREFIXES[spec.base].map(([p, k]) => ({ p, scale: k ** spec.power }));
  // The largest prefix that brings the peak to at least 1; when that overshoots past 1000 (m² jumps
  // by 10⁶ per step) and the step before already reads as at least 0.1, keep the step before.
  let pick = steps.length - 1;
  for (let i = 0; i < steps.length; i++)
    if (max / steps[i].scale >= 1) {
      pick = i > 0 && max / steps[i].scale >= 1000 && max / steps[i - 1].scale >= 0.1 ? i - 1 : i;
      break;
    }
  const { p, scale } = steps[pick];
  return { unit: `${p}${spec.base}${POW[spec.power] ?? ""}${spec.tail}`, factor: 1 / scale };
}

/** A value times a display factor, without the binary float tail (3.4e-3 × 1e3 is 3.4, not 3.4000000000000004). */
export const scaled = (v: number, factor: number) => (factor === 1 ? v : +(v * factor).toPrecision(12));

/** The physical quantity an artifact names (`liquid_diameter`), as a reader says it. */
const QUANTITY_LABEL: Record<string, string> = {
  liquid_diameter: "Droplet diameter",
  visible_flame_diameter: "Flame diameter",
  uv_flame_diameter: "Flame diameter (UV)",
  droplet_position: "Droplet position",
  mole_fraction: "Mole fraction",
  mass_flow: "Mass flow",
  volume_flow: "Volume flow",
  std_volume_flow: "Standard volume flow",
  front_contour: "Flame front",
  front_position: "Flame front position",
  spread_rate: "Spread rate",
  flow_velocity: "Flow velocity",
  gravity_level: "Gravity level",
  phase_end_time: "Phase end time",
};
export function quantityLabel(q: unknown): string | null {
  if (typeof q !== "string" || !q.trim()) return null;
  const k = q.trim().toLowerCase();
  if (QUANTITY_LABEL[k]) return QUANTITY_LABEL[k];
  const w = k.replace(/[_.]+/g, " ");
  return w.charAt(0).toUpperCase() + w.slice(1);
}

/** Physical quantities a chart caption may name, most specific first. */
const CAPTION_QUANTITIES: [RegExp, string][] = [
  [/\bflame (stand-?off|standoff)\b/i, "Flame standoff"],
  [/\bflame diameter\b/i, "Flame diameter"],
  [/\bdroplet diameter\b/i, "Droplet diameter"],
  [/\bburning[- ]rate constant\b/i, "Burning-rate constant"],
  [/\bspread rate\b/i, "Spread rate"],
  [/\bextinction diameter\b/i, "Extinction diameter"],
  [/\bdiameter\b/i, "Diameter"],
  [/\bradius\b/i, "Radius"],
  [/\btemperature\b/i, "Temperature"],
  [/\bpressure\b/i, "Pressure"],
  [/\boxygen\b|\bO2\b/, "Oxygen"],
  [/\bflow velocity\b|\bflow speed\b/i, "Flow velocity"],
];

/** The quantity a caption names ("Forecast of droplet diameter…" → "Droplet diameter"), or null. */
export function quantityFromText(text: string | null | undefined): string | null {
  if (!text) return null;
  for (const [re, name] of CAPTION_QUANTITIES) if (re.test(text)) return name;
  return null;
}

/** "5–95%" from quantile levels (0.05, 0.95); null when they are not a pair of levels. */
export function rangeLabel(lo: number | null | undefined, hi: number | null | undefined): string | null {
  if (lo == null || hi == null || !(lo > 0 && hi < 1 && lo < hi)) return null;
  const pc = (x: number) => String(+(x * 100).toPrecision(3));
  return `${pc(lo)}–${pc(hi)}%`;
}

/**
 * Whether a label names an identifier rather than a thing: an artifact id (`art_6629c9e5…`), a
 * content hash, a fixture or uuid-like id. Such labels are never shown as a legend entry.
 */
export function looksLikeId(s: string | null | undefined): boolean {
  if (!s) return true;
  const t = s.trim();
  return /^(art|artifact|rel|release|run|ev|evidence|fixture-artifact)[_-][\w.-]*$/i.test(t) || /^sha256:/i.test(t) || /\b[0-9a-f]{12,}\b/i.test(t) || /^[\w-]+…[\w-]*(\s*\(\w+\))?$/.test(t);
}

// ------------------------------------------------------------------ percentile bands

const STAT_WORDS = /\b(median|mean|average|avg|central|middle|lower|upper|low|high|lo|hi|min|max|minimum|maximum|bound|bounds|percentile|percentiles|pctl|pct|quantile|quantiles|p\d{1,2}(?:\.\d+)?|q\d{1,3}(?:\.\d+)?|\d{1,2}(?:\.\d+)?(?:st|nd|rd|th)?|ci|band|value|values)\b/g;

/** The percentile level an axis title or field names (5th percentile → 5, p95 → 95, median → 50), or a side ("lower"/"upper"). */
function levelOf(text: string): number | "lower" | "upper" | null {
  const raw = text.toLowerCase();
  // A level written as a fraction: q0.05, quantile 0.95, quantile_0.5.
  const f = /(?:^|[^a-z])(?:q|quantile)[\s_-]*(0\.\d+)/.exec(raw);
  if (f) return +(+f[1] * 100).toPrecision(6);
  const t = raw.replace(/[_/.-]+/g, " ");
  if (/\bmedian\b|\bp50\b|\bq50\b|\b50(?:th)?\s*(?:percentile|pct|%)/.test(t)) return 50;
  const m = /\b(\d{1,2}(?:\.\d+)?)\s*(?:st|nd|rd|th)?\s*(?:percentile|pctl|pct|%|quantile)/.exec(raw.replace(/[_/-]+/g, " ")) ?? /\b[pq]\s?(\d{1,2})\b/.exec(t);
  if (m) return +m[1];
  if (/\b(lower|low|lo|min|minimum)\b/.test(t)) return "lower";
  if (/\b(upper|high|hi|max|maximum)\b/.test(t)) return "upper";
  return null;
}

export interface PercentileBand {
  /** Index into `view.y` of the central line, and of the band's lower and upper edges. */
  mid: number;
  lo: number;
  hi: number;
  /** The quantity the three axes measure, when their fields or titles name one ("Diameter"). */
  quantity: string | null;
  /** The band in words: "5th–95th percentile", or "Range" when only lower/upper are named. */
  range: string;
  /** What the central line is: "Median" or the axis's own title. */
  midTitle: string;
}

const ord = (n: number) => {
  const s = String(+n.toPrecision(3));
  if (!Number.isInteger(n)) return `${s}th`;
  const t = n % 100;
  return `${s}${t >= 11 && t <= 13 ? "th" : n % 10 === 1 ? "st" : n % 10 === 2 ? "nd" : n % 10 === 3 ? "rd" : "th"}`;
};

/**
 * When a chart's y axes are percentiles (or lower/central/upper bounds) of one quantity, which axis
 * is the central line and which two bound the shaded band. Three axes in one unit, at least one of
 * them named by a percentile, quantile or bound; otherwise null and the chart draws every axis as
 * its own series.
 */
export function percentileBand(y: { field: string; title: string; unit?: string | null }[]): PercentileBand | null {
  if (y.length !== 3) return null;
  const units = new Set(y.map((a) => (a.unit ?? "").trim()));
  if (units.size > 1) return null;
  const levels = y.map((a) => levelOf(a.title) ?? levelOf(a.field));
  if (!levels.some((l) => l != null)) return null;
  const num = levels.map((l) => (typeof l === "number" ? l : null));
  let lo = levels.indexOf("lower");
  let hi = levels.indexOf("upper");
  const nums = num.flatMap((l, i) => (l != null ? [{ l, i }] : [])).sort((a, b) => a.l - b.l);
  if (lo < 0 && hi < 0 && nums.length === 3) {
    lo = nums[0].i;
    hi = nums[2].i;
  } else {
    if (lo < 0 && nums.length) lo = nums[0].l < 50 ? nums[0].i : -1;
    if (hi < 0 && nums.length) hi = nums[nums.length - 1].l > 50 ? nums[nums.length - 1].i : -1;
  }
  if (lo < 0 || hi < 0 || lo === hi) return null;
  const mid = [0, 1, 2].find((i) => i !== lo && i !== hi)!;
  // The central line sits between the edges when its level is known.
  if (num[mid] != null && ((num[lo] != null && num[mid]! <= num[lo]!) || (num[hi] != null && num[mid]! >= num[hi]!))) return null;
  const range = num[lo] != null && num[hi] != null ? `${ord(num[lo]!)}–${ord(num[hi]!)} percentile` : "Range";
  const midTitle = num[mid] === 50 ? "Median" : y[mid].title;
  return { mid, lo, hi, quantity: sharedQuantity(y), range, midTitle };
}

const QUANTITY_ALIAS: Record<string, string> = { d: "diameter", dia: "diameter", diam: "diameter", r: "radius", temp: "temperature" };

/** The words every field (or else every title) shares once statistics and unit suffixes are removed: "diameter". */
function sharedQuantity(y: { field: string; title: string }[]): string | null {
  const strip = (s: string) =>
    s
      .toLowerCase()
      .replace(/^.*\//, "")
      .replace(/[_.-]+/g, " ")
      .replace(STAT_WORDS, " ")
      .split(/\s+/)
      .filter((w) => w && !UNIT_SUFFIX[w] && !/^\d+$/.test(w))
      .map((w) => QUANTITY_ALIAS[w] ?? w)
      .filter((w) => w.length > 1);
  for (const pick of [(a: { field: string }) => a.field, (a: { title: string }) => a.title.replace(/\(.*?\)/g, " ")]) {
    const sets = y.map((a) => strip(pick(a as { field: string; title: string })));
    const common = sets[0].filter((w) => sets.every((s) => s.includes(w)));
    if (common.length) {
      const w = common.join(" ");
      return w.charAt(0).toUpperCase() + w.slice(1);
    }
  }
  return null;
}

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
  /** What one row is to a reader ("17 tests" for a conditions table, else "rows"). */
  noun: "test" | "row";
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
  return { points, categorical, categories, groups, skipped, skippedWhy: why, total: rows.length, noun: rowNoun(rows) };
}

/** What a row is, for the reader: a test when rows name their run (a conditions table), else a row. */
export function rowNoun(rows: unknown[]): "test" | "row" {
  const r = rows.find((x) => x && typeof x === "object" && !Array.isArray(x)) as Record<string, unknown> | undefined;
  return r && ["run", "run_id", "test", "test_id", "run_key"].some((k) => k in r) ? "test" : "row";
}

/** "17 tests", "1 row". */
export const rowCount = (n: number, noun: "test" | "row") => `${n} ${noun}${n === 1 ? "" : "s"}`;

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
