// How a run's recorded conditions read in the UI: each family's record keys (x_o2, x_o2_ambient,
// injected_mass_flow, tad, …) as named quantities in the units researchers use, grouped as a reader
// scans them (atmosphere, fuel and flow, derived quantities). Values stay NASA's SI values; only the
// display unit changes. Keys that only restate another (`*_role`) are not shown.

import type { ConditionCell } from "../api";

export type ConditionGroup = "Atmosphere" | "Fuel and flow" | "Derived quantities" | "Other conditions";
export const CONDITION_GROUPS: ConditionGroup[] = ["Atmosphere", "Fuel and flow", "Derived quantities", "Other conditions"];

interface Show {
  label: string;
  /** SI value → [display value, display unit]. */
  fmt: (si: number) => [string, string];
  group: ConditionGroup;
  order: number;
}

const n = (v: number, d = 3) => String(+v.toPrecision(d));
const pct = (v: number): [string, string] => [n(v * 100), "%"];
const A: ConditionGroup = "Atmosphere";
const F: ConditionGroup = "Fuel and flow";
const D: ConditionGroup = "Derived quantities";

const SHOW: Record<string, Show> = {
  pressure: { label: "Pressure", fmt: (v) => [n(v / 1000, 4), "kPa"], group: A, order: 1 },
  x_o2: { label: "Oxygen", fmt: pct, group: A, order: 2 },
  x_o2_ambient: { label: "Oxygen", fmt: pct, group: A, order: 2 },
  oxygen_mole_fraction: { label: "Oxygen", fmt: pct, group: A, order: 2 },
  x_n2: { label: "Nitrogen", fmt: pct, group: A, order: 3 },
  x_co2: { label: "CO₂", fmt: pct, group: A, order: 4 },
  x_co: { label: "CO", fmt: pct, group: A, order: 4 },
  x_he: { label: "Helium", fmt: pct, group: A, order: 4 },
  x_fuel_ambient: { label: "Fuel in the atmosphere", fmt: pct, group: A, order: 5 },
  ambient_temperature: { label: "Temperature", fmt: (v) => [n(v - 273.15), "°C"], group: A, order: 6 },
  fuel_bottle: { label: "Fuel in the bottle", fmt: pct, group: F, order: 10 },
  x_fuel_injected: { label: "Fuel, burner stream", fmt: pct, group: F, order: 11 },
  x_o2_injected: { label: "Oxygen, burner stream", fmt: pct, group: F, order: 12 },
  injected_mass_flow: { label: "Burner flow", fmt: (v) => [n(v * 1e6), "mg/s"], group: F, order: 13 },
  flow_velocity: { label: "Air flow", fmt: (v) => [n(v * 100), "cm/s"], group: F, order: 14 },
  initial_diameter: { label: "Initial droplet diameter", fmt: (v) => [n(v * 1000), "mm"], group: F, order: 15 },
  tad: { label: "Adiabatic flame temperature", fmt: (v) => [n(v, 4), "K"], group: D, order: 20 },
  zst: { label: "Stoichiometric mixture fraction", fmt: (v) => [n(v), ""], group: D, order: 21 },
};

const CHEM: Record<string, string> = { o2: "O₂", n2: "N₂", co2: "CO₂", co: "CO", he: "He", h2: "H₂", ch4: "CH₄" };

/** A record key in sentence case: `fuel_bottle` → "Fuel bottle", `x_o2_after` → "O₂ after". */
export function humanKey(key: string): string {
  const w = key.replace(/^x_/, "").split(/[_\s]+/).filter(Boolean).map((t) => CHEM[t.toLowerCase()] ?? t.toLowerCase());
  const s = w.join(" ");
  return s.charAt(0).toUpperCase() + s.slice(1);
}

/** The reader's name for a condition key, as the run panel shows it. */
export const conditionLabel = (key: string) => SHOW[key]?.label ?? humanKey(key);

export interface ConditionRow {
  key: string;
  label: string;
  /** The number as shown, in the display unit. */
  value: string;
  /** The display unit, or "" for a dimensionless value. */
  unit: string;
  /** How the value was settled when it is not plainly resolved (e.g. "inferred"). */
  note: string | null;
  group: ConditionGroup;
  /** Value, unit and note as one line. */
  text: string;
}

/** The run's conditions with a value, named, grouped and ordered; restated (`*_role`) and value-less keys left out. */
export function conditionRows(conditions: Record<string, ConditionCell>): ConditionRow[] {
  const rows: (ConditionRow & { order: number })[] = [];
  for (const [k, c] of Object.entries(conditions)) {
    if (k.endsWith("_role") || c.value_si == null) continue;
    const s = SHOW[k];
    const [value, unit] = s ? s.fmt(c.value_si) : [n(c.value_si), c.unit_si && c.unit_si !== "1" ? c.unit_si : ""];
    const note = c.resolution !== "resolved" ? c.resolution.replace(/_/g, " ") : null;
    const text = `${value}${unit ? ` ${unit}` : ""}${note ? ` (${note})` : ""}`;
    rows.push({ key: k, label: conditionLabel(k), value, unit, note, group: s?.group ?? "Other conditions", text, order: s?.order ?? 50 });
  }
  return rows.sort((a, b) => a.order - b.order || a.label.localeCompare(b.label));
}

/** A registry gap in plain words; gaps about a condition without a value are dropped (the table omits it). */
export function gapText(g: string): string | null {
  if (/^condition \S+ has no si value/.test(g)) return null;
  if (g.startsWith("no observations")) return "No measured time series on this server";
  if (g.startsWith("no control history")) return "Ignition time not established";
  if (g.startsWith("gravity level not established")) return "Gravity level not stated";
  const m = g.match(/^no model release for family (\S+)/);
  if (m) return "No model for this kind of run yet";
  return g.charAt(0).toUpperCase() + g.slice(1);
}
