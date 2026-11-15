// Conditions as researchers read them: runs grouped into test points, and each investigation's
// standing against a crew atmosphere counted from actual tests, never from the product of its O2 range
// and its pressure range (a range pair can "cover" an atmosphere no test was run in).
//
// A test point is the runs of one investigation and one fuel at the same atmosphere: O2 within half a
// percentage point and pressure within 1 kPa of the point's first run. The rule is fixed and deterministic; nothing is fitted.
// Pure: reads K1.13 `ConditionsTable` rows through mapPoints (recorded SI values, burner-stream O2
// never taken as the atmosphere).

import type { ConditionsRow } from "../contracts/generated";
import type { Habitat } from "./habitats";
import { pointsFromConditions, type MapPoint } from "./mapPoints.ts";

export interface TestPoint {
  key: string;
  investigation: string;
  fuel: string | null;
  /** Mean of the grouped runs (they differ by less than the bin). */
  o2: number;
  kpa: number;
  runs: string[];
  approximate: boolean;
}

const O2_TOL = 0.005;
const KPA_TOL = 1;

export function groupTestPoints(points: MapPoint[]): TestPoint[] {
  // Within one investigation and fuel, in a fixed order (O2, then pressure, then id), a run joins the
  // first group whose first run is within the tolerance on both axes; grid bins would split runs that
  // straddle a bin edge.
  const sorted = [...points].sort((a, b) => a.investigation.localeCompare(b.investigation) || (a.fuel ?? "").localeCompare(b.fuel ?? "") || a.o2 - b.o2 || a.kpa - b.kpa || a.id.localeCompare(b.id));
  const groups: { head: MapPoint; members: MapPoint[] }[] = [];
  const open = new Map<string, { head: MapPoint; members: MapPoint[] }[]>();
  for (const p of sorted) {
    const k = `${p.investigation}|${(p.fuel ?? "").toLowerCase()}`;
    const cands = open.get(k) ?? [];
    const g = cands.find((c) => Math.abs(c.head.o2 - p.o2) <= O2_TOL && Math.abs(c.head.kpa - p.kpa) <= KPA_TOL);
    if (g) g.members.push(p);
    else {
      const ng = { head: p, members: [p] };
      cands.push(ng);
      groups.push(ng);
      open.set(k, cands);
    }
  }
  return groups.map(({ head, members: ps }) => ({
    key: `${head.investigation}|${head.id}`,
    investigation: head.investigation,
    fuel: head.fuel,
    o2: ps.reduce((s, p) => s + p.o2, 0) / ps.length,
    kpa: ps.reduce((s, p) => s + p.kpa, 0) / ps.length,
    runs: ps.map((p) => p.id),
    approximate: ps.some((p) => p.approximate),
  }));
}

/** Tolerance for "inside": 0.5 points of O2, 1 % of the habitat pressure (as habitats.ts). */
const TOL_O2 = 0.005;
const TOL_PA_REL = 0.01;
const ATM_KPA = 101.325;

export interface Offset {
  inside: boolean;
  /** How far the test would have to move to reach the atmosphere: O2 in percentage points, pressure in psi (positive = higher). */
  o2Pts: number;
  psi: number;
  /** Ranking distance: O2 as a fraction, pressure as a fraction of one atmosphere. Geometry, not a model. */
  distance: number;
}

const gap = (v: number, lo: number, hi: number, tol: number) => (v < lo - tol ? lo - v : v > hi + tol ? hi - v : 0);

export function offset(o2: number, kpa: number, h: Habitat): Offset {
  const dO2 = gap(o2, h.o2[0], h.o2[1], TOL_O2);
  const dKpa = gap(kpa, h.pa[0] / 1000, h.pa[1] / 1000, (h.pa[1] / 1000) * TOL_PA_REL);
  return { inside: dO2 === 0 && dKpa === 0, o2Pts: dO2 * 100, psi: dKpa / 6.894757, distance: Math.hypot(dO2, dKpa / ATM_KPA) };
}

export function offsetText(o: Offset): string {
  if (o.inside) return "inside";
  const parts: string[] = [];
  if (o.o2Pts !== 0) parts.push(`${o.o2Pts > 0 ? "+" : "−"}${Math.abs(o.o2Pts).toFixed(1)} pts O₂`);
  if (o.psi !== 0) parts.push(`${o.psi > 0 ? "+" : "−"}${Math.abs(o.psi).toFixed(1)} psi`);
  return parts.join(", ");
}

export type Undecidable = "no_oxygen" | "no_pressure" | "neither" | "burner_only";

export interface Standing {
  investigation: string;
  runs: number;
  placed: number;
  points: TestPoint[];
  o2: [number, number] | null;
  kpa: [number, number] | null;
  /** Runs at test points inside the atmosphere. */
  runsInside: number;
  pointsInside: number;
  nearest: { point: TestPoint; offset: Offset } | null;
  /** Why no test can be placed, when none can. */
  undecidable: Undecidable | null;
}

export interface ConditionsView {
  points: TestPoint[];
  byInvestigation: Map<string, Standing>;
  unplaced: ReturnType<typeof pointsFromConditions>["unplaced"];
  runs: number;
  placedRuns: number;
}

/** Everything the Conditions views need, from the table's rows in one pass. */
export function conditionsView(rows: ConditionsRow[]): ConditionsView {
  const { points: placed, unplaced } = pointsFromConditions({ rows });
  const points = groupTestPoints(placed);
  const total = new Map<string, { runs: number; o2: number; pa: number; burner: number }>();
  for (const r of rows) {
    const t = total.get(r.investigation_id) ?? { runs: 0, o2: 0, pa: 0, burner: 0 };
    t.runs++;
    const o2 = r.cells.find((c) => c.key === "oxygen_mole_fraction");
    const pa = r.cells.find((c) => c.key === "pressure");
    if (o2?.status === "value") {
      if ((o2.value.flags ?? []).includes("oxidizer_stream:burner")) t.burner++;
      else t.o2++;
    }
    if (pa?.status === "value") t.pa++;
    total.set(r.investigation_id, t);
  }
  const byInvestigation = new Map<string, Standing>();
  for (const [inv, t] of total) {
    const ps = points.filter((p) => p.investigation === inv);
    const os = ps.map((p) => p.o2);
    const ks = ps.map((p) => p.kpa);
    byInvestigation.set(inv, {
      investigation: inv,
      runs: t.runs,
      placed: ps.reduce((s, p) => s + p.runs.length, 0),
      points: ps,
      o2: ps.length ? [Math.min(...os), Math.max(...os)] : null,
      kpa: ps.length ? [Math.min(...ks), Math.max(...ks)] : null,
      runsInside: 0,
      pointsInside: 0,
      nearest: null,
      undecidable: ps.length ? null : t.o2 === 0 && t.burner > 0 ? "burner_only" : t.o2 === 0 && t.pa === 0 ? "neither" : t.o2 === 0 ? "no_oxygen" : "no_pressure",
    });
  }
  return { points, byInvestigation, unplaced, runs: rows.length, placedRuns: placed.length };
}

/** Each investigation's standing for one atmosphere: tested inside first (most runs), then nearest, undecidable last. */
export function rankForAtmosphere(v: ConditionsView, h: Habitat): Standing[] {
  const out = [...v.byInvestigation.values()].map((s) => {
    let runsInside = 0;
    let pointsInside = 0;
    let nearest: Standing["nearest"] = null;
    for (const p of s.points) {
      const o = offset(p.o2, p.kpa, h);
      if (o.inside) {
        runsInside += p.runs.length;
        pointsInside++;
      }
      if (!nearest || o.distance < nearest.offset.distance) nearest = { point: p, offset: o };
    }
    return { ...s, runsInside, pointsInside, nearest };
  });
  const tier = (s: Standing) => (s.runsInside > 0 ? 0 : s.nearest ? 1 : 2);
  return out.sort(
    (a, b) =>
      tier(a) - tier(b) ||
      b.runsInside - a.runsInside ||
      (a.nearest && b.nearest ? a.nearest.offset.distance - b.nearest.offset.distance : 0) ||
      a.investigation.localeCompare(b.investigation, undefined, { numeric: true }),
  );
}

export const UNDECIDABLE_TEXT: Record<Undecidable, string> = {
  no_oxygen: "NASA's table states no oxygen",
  no_pressure: "NASA's table states no pressure",
  neither: "NASA's table states neither oxygen nor pressure",
  burner_only: "oxygen only in the burner stream; the atmosphere is the fuel",
};
