// Conditions views: test points group repeated runs by a fixed rule, and an atmosphere counts actual
// tests inside it, never the product of an O2 range and a pressure range.
// Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import type { ConditionsRow } from "../src/contracts/generated";
import { HABITAT_LIST, PSI_PA } from "../src/research/habitats.ts";
import { conditionsView, groupTestPoints, offset, rankForAtmosphere } from "../src/research/testPoints.ts";

const [moon, , iss] = ["moon", "suit", "iss"].map((id) => HABITAT_LIST.find((h) => h.id === id)!);
let n = 0;
const v = (si: number, unit: string, flags: string[] = []) => ({ status: "value" as const, value: { value_si: si, unit_si: unit, flags, raw_value: si, raw_unit: unit, evidence_class: "source_reports" } });
const row = (inv: string, o2: number | null, pa: number | null, fuel = "ethylene", o2flags: string[] = []): ConditionsRow =>
  ({
    investigation_id: inv,
    run: { run_id: `${inv}:r${++n}`, resolution: "resolved" },
    cells: [
      ...(o2 == null ? [] : [{ key: "oxygen_mole_fraction", ...v(o2, "1", o2flags) }]),
      ...(pa == null ? [] : [{ key: "pressure", ...v(pa, "Pa") }]),
      { key: "fuel", status: "value", value: { value_si: null, unit_si: null, raw_value: fuel, raw_unit: null, evidence_class: "source_reports" } },
    ],
  }) as unknown as ConditionsRow;

test("repeated runs at one atmosphere are one test point; a different fuel or pressure is another", () => {
  const rows = [row("A", 0.21, 101_300), row("A", 0.212, 101_500), row("A", 0.21, 101_300, "propane"), row("A", 0.21, 70_000)];
  const { points } = conditionsView(rows);
  assert.equal(points.length, 3);
  assert.equal(points.find((p) => p.runs.length === 2)!.fuel, "ethylene");
});

test("a range pair that spans the Moon box does not count: only tests inside do", () => {
  // O2 from 20 to 40 %, pressure from 6 to 45 psi, but no single test at >= 30 % and <= 10.2 psi.
  const rows = [row("A", 0.2, 6 * PSI_PA), row("A", 0.4, 45 * PSI_PA), row("B", 0.35, 8 * PSI_PA), row("B", 0.36, 8 * PSI_PA)];
  const ranked = rankForAtmosphere(conditionsView(rows), moon);
  assert.equal(ranked[0].investigation, "B");
  assert.equal(ranked[0].runsInside, 2);
  const a = ranked.find((s) => s.investigation === "A")!;
  assert.equal(a.runsInside, 0);
  assert.ok(a.nearest && !a.nearest.offset.inside);
});

test("burner-stream oxygen is never the atmosphere; an investigation without placeable tests says why", () => {
  const rows = [row("C", 0.21, 110_000, "ethane", ["oxidizer_stream:burner"]), row("D", null, 101_300)];
  const view = conditionsView(rows);
  assert.equal(view.points.length, 0);
  assert.equal(view.byInvestigation.get("C")!.undecidable, "burner_only");
  assert.equal(view.byInvestigation.get("D")!.undecidable, "no_oxygen");
  assert.equal(rankForAtmosphere(view, iss).every((s) => s.nearest === null), true);
});

test("offsets say how far a test would move, in O2 points and psi", () => {
  const o = offset(0.21, 101.3, moon);
  assert.equal(o.inside, false);
  assert.ok(Math.abs(o.o2Pts - 9) < 1e-9);
  assert.ok(o.psi < -4 && o.psi > -5);
  assert.equal(offset(0.21, 101.325, iss).inside, true);
  assert.equal(groupTestPoints([]).length, 0);
});
