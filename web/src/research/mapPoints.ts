// T9: the Atmosphere map's points, from K1.13 `ConditionsTable` rows. A run is placed only where it
// recorded both an ambient O2 value and a pressure value; anything else is counted, never placed at
// a guessed point. A burner-stream O2 (inverse flames) is not the atmosphere and is never placed.

import type { ConditionsRow, ConditionsTable, Measured } from "../contracts/generated";

export interface MapPoint {
  id: string;
  investigation: string;
  o2: number;
  kpa: number;
  fuel: string | null;
  approximate: boolean;
}

export interface Unplaced {
  o2: number;
  pressure: number;
  both: number;
  /** O2 recorded, but for the burner stream, not the atmosphere. */
  notAtmosphere: number;
}

const NOT_ATMOSPHERE = ["oxidizer_stream:burner"];
const APPROX = ["approximate", "approximate_per_header"];

function value(row: ConditionsRow, key: string): Measured | null {
  const c = row.cells.find((x) => x.key === key);
  return c && c.status === "value" && c.value.value_si != null && Number.isFinite(c.value.value_si) ? c.value : null;
}

export function pointsFromConditions(t: Pick<ConditionsTable, "rows">): { points: MapPoint[]; unplaced: Unplaced } {
  const points: MapPoint[] = [];
  const unplaced: Unplaced = { o2: 0, pressure: 0, both: 0, notAtmosphere: 0 };
  t.rows.forEach((r, i) => {
    const o2 = value(r, "oxygen_mole_fraction");
    const p = value(r, "pressure");
    if (o2 && (o2.flags ?? []).some((f) => NOT_ATMOSPHERE.includes(f))) {
      unplaced.notAtmosphere++;
      return;
    }
    if (!o2 && !p) unplaced.both++;
    else if (!o2) unplaced.o2++;
    else if (!p) unplaced.pressure++;
    else {
      const fuel = r.cells.find((c) => c.key === "fuel");
      points.push({
        id: r.run.run_id ?? `${r.investigation_id} row ${i + 1}`,
        investigation: r.investigation_id,
        o2: o2.value_si!,
        kpa: p.value_si! / 1000,
        fuel: fuel && fuel.status === "value" && typeof fuel.value.raw_value === "string" ? fuel.value.raw_value : null,
        approximate: [...(o2.flags ?? []), ...(p.flags ?? [])].some((f) => APPROX.includes(f)),
      });
    }
  });
  return { points, unplaced };
}
