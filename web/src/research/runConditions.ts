// How a run's recorded conditions read in the UI: each family's record keys (x_o2, x_o2_ambient,
// injected_mass_flow, tad, …) as named quantities in the units researchers use. Values stay NASA's
// SI values; only the display unit changes. Keys that only restate another (`*_role`) are not shown.

import type { ConditionCell } from "../api";

interface Show {
  label: string;
  /** SI value → display text. */
  fmt: (si: number) => string;
  order: number;
}

const n = (v: number, d = 3) => String(+v.toPrecision(d));
const pct = (v: number) => `${n(v * 100)} %`;

const SHOW: Record<string, Show> = {
  pressure: { label: "Pressure", fmt: (v) => `${n(v / 1000, 4)} kPa`, order: 1 },
  x_o2: { label: "Oxygen", fmt: pct, order: 2 },
  x_o2_ambient: { label: "Oxygen", fmt: pct, order: 2 },
  oxygen_mole_fraction: { label: "Oxygen", fmt: pct, order: 2 },
  x_o2_injected: { label: "Oxygen, burner stream", fmt: pct, order: 3 },
  x_fuel_ambient: { label: "Fuel in the atmosphere", fmt: pct, order: 4 },
  x_fuel_injected: { label: "Fuel, burner stream", fmt: pct, order: 4 },
  x_n2: { label: "Nitrogen", fmt: pct, order: 5 },
  x_co2: { label: "CO₂", fmt: pct, order: 6 },
  x_co: { label: "CO", fmt: pct, order: 6 },
  x_he: { label: "Helium", fmt: pct, order: 6 },
  flow_velocity: { label: "Air flow", fmt: (v) => `${n(v * 100)} cm/s`, order: 7 },
  injected_mass_flow: { label: "Burner flow", fmt: (v) => `${n(v * 1e6)} mg/s`, order: 7 },
  ambient_temperature: { label: "Ambient temperature", fmt: (v) => `${n(v - 273.15)} °C`, order: 8 },
  initial_diameter: { label: "Initial droplet diameter", fmt: (v) => `${n(v * 1000)} mm`, order: 9 },
  tad: { label: "Adiabatic flame temperature", fmt: (v) => `${n(v, 4)} K`, order: 10 },
  zst: { label: "Stoichiometric mixture fraction", fmt: (v) => n(v), order: 11 },
};

export interface ConditionRow {
  key: string;
  label: string;
  text: string;
}

/** The run's conditions with a value, named and ordered; restated (`*_role`) and value-less keys left out. */
export function conditionRows(conditions: Record<string, ConditionCell>): ConditionRow[] {
  const rows: (ConditionRow & { order: number })[] = [];
  for (const [k, c] of Object.entries(conditions)) {
    if (k.endsWith("_role") || c.value_si == null) continue;
    const s = SHOW[k];
    const label = s?.label ?? k.replace(/_/g, " ").replace(/^x /, "");
    const text = s ? s.fmt(c.value_si) : `${n(c.value_si)}${c.unit_si && c.unit_si !== "1" ? ` ${c.unit_si}` : ""}`;
    rows.push({ key: k, label, text: c.resolution !== "resolved" ? `${text} (${c.resolution.replace(/_/g, " ")})` : text, order: s?.order ?? 50 });
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
