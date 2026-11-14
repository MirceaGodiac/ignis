// T9: how near each investigation's tested envelope is to the crew atmospheres the challenge names
// (Flame in Freefall brief, "Spacesuits and oxygen concentration"). Pure: reads a K1.13
// `InvestigationEnvelope`, never a run's outcome. Nothing is extrapolated: an envelope that does not
// reach an atmosphere says by how much, and a missing condition is "not stated", never "inside".

import type { ConditionKey, EnvelopeEntry, InvestigationEnvelope } from "../contracts/generated";

export const PSI_PA = 6894.757;
const ATM_PA = 101325;

export type HabitatId = "moon" | "suit" | "iss";
export interface Habitat {
  id: HabitatId;
  label: string;
  /** Column heading. */
  short: string;
  sub: string;
  /** Allowed O2 mole fraction and pressure (Pa); a point habitat has min = max. */
  o2: [number, number];
  pa: [number, number];
}

export const HABITAT_LIST: Habitat[] = [
  { id: "moon", label: "Moon base and rovers", short: "Moon / rover", sub: "≥ 30 % O₂ · ≤ 10.2 psi", o2: [0.3, 1], pa: [4.3 * PSI_PA, 10.2 * PSI_PA] },
  { id: "suit", label: "Spacesuit", short: "Suit", sub: "100 % O₂ · 4.3 psi", o2: [1, 1], pa: [4.3 * PSI_PA, 4.3 * PSI_PA] },
  { id: "iss", label: "ISS and Earth", short: "ISS / Earth", sub: "21 % O₂ · 14.7 psi", o2: [0.21, 0.21], pa: [14.7 * PSI_PA, 14.7 * PSI_PA] },
];

/** Tolerance for "reaches": 0.5 percentage points of O2 and 1 % of the habitat pressure. */
const TOL_O2 = 0.005;
const TOL_PA_REL = 0.01;

/** Gravity is stored as a fraction of Earth gravity (SI unit "1"). Partial gravity: Moon (0.17 g) to Mars (0.38 g). */
export const PARTIAL_G: [number, number] = [0.1, 0.5];

export type Range = { min: number; max: number; runs: number };
export type Slot = { kind: "range"; range: Range } | { kind: "missing"; reason: string };

function slot(entries: EnvelopeEntry[], key: ConditionKey): Slot {
  const e = entries.find((x) => x.key === key);
  if (!e) return { kind: "missing", reason: "not reported in the envelope" };
  if (e.status === "missing") return { kind: "missing", reason: e.reason };
  if (e.status === "range") return { kind: "range", range: { min: e.min_si, max: e.max_si, runs: e.runs } };
  return { kind: "missing", reason: "reported as text, not as a number" };
}

export interface EnvelopeAxes {
  investigation_id: string;
  runs: number;
  o2: Slot;
  pa: Slot;
  gravity: Slot;
}

export const axesOf = (env: InvestigationEnvelope): EnvelopeAxes => ({
  investigation_id: env.investigation_id,
  runs: env.runs,
  o2: slot(env.entries, "oxygen_mole_fraction"),
  pa: slot(env.entries, "pressure"),
  gravity: slot(env.entries, "gravity_level"),
});

/** Signed gap from an interval [lo, hi] to a target interval: positive = target is above, 0 = they overlap. */
function gap(lo: number, hi: number, tLo: number, tHi: number, tol: number): number {
  if (hi + tol < tLo) return tLo - hi;
  if (lo - tol > tHi) return -(lo - tHi);
  return 0;
}

export type Fit = { status: "reached" } | { status: "outside"; o2Pts: number; psi: number; distance: number } | { status: "unknown"; reason: string };

/**
 * Whether the tested envelope reaches a habitat atmosphere. `o2Pts` and `psi` say how far the tests
 * would have to move (positive = higher); `distance` ranks them, with O2 as a fraction (0–1) and
 * pressure as a fraction of one atmosphere, so both axes count on a comparable scale.
 */
export function habitatFit(a: EnvelopeAxes, h: Habitat): Fit {
  if (a.o2.kind === "missing") return { status: "unknown", reason: `O₂ ${a.o2.reason}` };
  if (a.pa.kind === "missing") return { status: "unknown", reason: `pressure ${a.pa.reason}` };
  const o = a.o2.range;
  const p = a.pa.range;
  const dO2 = gap(o.min, o.max, h.o2[0], h.o2[1], TOL_O2);
  const dPa = gap(p.min, p.max, h.pa[0], h.pa[1], h.pa[1] * TOL_PA_REL);
  if (dO2 === 0 && dPa === 0) return { status: "reached" };
  return { status: "outside", o2Pts: dO2 * 100, psi: dPa / PSI_PA, distance: Math.hypot(dO2, dPa / ATM_PA) };
}

export type GravityStatus = { status: "partial_tested" } | { status: "no_partial_data"; tested: Range } | { status: "not_stated"; reason: string };

export function gravityStatus(a: EnvelopeAxes): GravityStatus {
  if (a.gravity.kind === "missing") return { status: "not_stated", reason: a.gravity.reason };
  const g = a.gravity.range;
  return g.max >= PARTIAL_G[0] && g.min <= PARTIAL_G[1] ? { status: "partial_tested" } : { status: "no_partial_data", tested: g };
}

/** Investigations ordered by how near they come to one habitat: reached, then by distance, unknown last. */
export function rankFor(axes: EnvelopeAxes[], h: Habitat): { axes: EnvelopeAxes; fit: Fit }[] {
  const order = (f: Fit) => (f.status === "reached" ? 0 : f.status === "outside" ? 1 : 2);
  return axes
    .map((x) => ({ axes: x, fit: habitatFit(x, h) }))
    .sort(
      (a, b) =>
        order(a.fit) - order(b.fit) ||
        (a.fit.status === "outside" && b.fit.status === "outside" ? a.fit.distance - b.fit.distance : 0) ||
        a.axes.investigation_id.localeCompare(b.axes.investigation_id),
    );
}

/** "+9.0 pts O₂, −4.5 psi": how far the tests would have to move to reach the habitat. */
export function fitText(f: Fit): string {
  if (f.status === "reached") return "within tested envelope";
  if (f.status === "unknown") return `not decidable: ${f.reason}`;
  const parts: string[] = [];
  if (f.o2Pts !== 0) parts.push(`${f.o2Pts > 0 ? "+" : "−"}${Math.abs(f.o2Pts).toFixed(1)} pts O₂`);
  if (f.psi !== 0) parts.push(`${f.psi > 0 ? "+" : "−"}${Math.abs(f.psi).toFixed(1)} psi`);
  return `outside tested envelope: ${parts.join(", ")}`;
}
