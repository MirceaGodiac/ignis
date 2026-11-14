// T9: habitat relevance never calls an untested atmosphere "within", and says by how much it misses.
// Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { ConditionsTable, InvestigationEnvelope } from "../src/contracts/generated";
import { axesOf, fitText, gravityStatus, HABITAT_LIST, habitatFilters, habitatFit, PSI_PA, rankFor } from "../src/research/habitats.ts";
import { flagTags, quotedText } from "../src/research/conditionCells.ts";

const fixture = <T>(p: string): T => JSON.parse(readFileSync(fileURLToPath(new URL(`../../fixtures/contracts/${p}`, import.meta.url)), "utf8")) as T;
const [moon, suit, iss] = ["moon", "suit", "iss"].map((id) => HABITAT_LIST.find((h) => h.id === id)!);

const env = (id: string, o2: [number, number] | null, psi: [number, number] | null, g?: [number, number]): InvestigationEnvelope => ({
  investigation_id: id,
  runs: 3,
  entries: [
    o2 ? { status: "range", key: "oxygen_mole_fraction", min_si: o2[0], max_si: o2[1], unit_si: "1", runs: 3 } : { status: "missing", key: "oxygen_mole_fraction", reason: "not in the table" },
    psi ? { status: "range", key: "pressure", min_si: psi[0] * PSI_PA, max_si: psi[1] * PSI_PA, unit_si: "Pa", runs: 3 } : { status: "missing", key: "pressure", reason: "not in the table" },
    ...(g ? [{ status: "range" as const, key: "gravity_level" as const, min_si: g[0], max_si: g[1], unit_si: "1", runs: 3 }] : []),
  ],
});

test("the C0 fixture: an ISS-air investigation reaches ISS and misses the Moon by O2 and pressure", () => {
  const t = fixture<ConditionsTable>("data/conditions_table.json");
  const a = axesOf(t.envelopes![0]);
  assert.equal(habitatFit(a, iss).status, "reached");
  const m = habitatFit(a, moon);
  assert.equal(m.status, "outside");
  assert.equal(fitText(m), "outside tested envelope: +9.0 pts O₂, −4.5 psi");
  assert.deepEqual(gravityStatus(a), { status: "not_stated", reason: "Synthetic fixture: no run states a gravity level." });
});

test("a missing condition is not decidable, never within, and ranks last", () => {
  const noPressure = axesOf(env("c-no-pressure", [0.3, 0.4], null));
  assert.deepEqual(habitatFit(noPressure, moon), { status: "unknown", reason: "pressure not in the table" });
  const near = axesOf(env("b-near", [0.25, 0.28], [10.5, 11]));
  const inside = axesOf(env("a-inside", [0.3, 0.4], [8, 10]));
  assert.deepEqual(
    rankFor([noPressure, near, inside], moon).map((r) => r.axes.investigation_id),
    ["a-inside", "b-near", "c-no-pressure"],
  );
});

test("the Moon limits are inclusive within tolerance; a point habitat needs its exact atmosphere", () => {
  assert.equal(habitatFit(axesOf(env("edge", [0.296, 0.3], [10.25, 12])), moon).status, "reached", "0.4 pts and 0.5 % inside tolerance");
  assert.equal(habitatFit(axesOf(env("o2-rich", [0.5, 1], [3, 5])), suit).status, "reached");
  const lowO2 = habitatFit(axesOf(env("air", [0.21, 0.21], [4.3, 4.3])), suit);
  assert.equal(lowO2.status === "outside" && Math.round(lowO2.o2Pts), 79);
});

test("partial gravity: only a tested level between 0.1 g and 0.5 g counts", () => {
  assert.equal(gravityStatus(axesOf(env("micro", [0.21, 0.21], [14.7, 14.7], [1e-6, 1e-3]))).status, "no_partial_data");
  assert.equal(gravityStatus(axesOf(env("lunar", [0.21, 0.21], [14.7, 14.7], [0.16, 0.17]))).status, "partial_tested");
  assert.equal(gravityStatus(axesOf(env("ground", [0.21, 0.21], [14.7, 14.7], [1, 1]))).status, "no_partial_data");
});

test("habitat filters select the same region the ranking calls within, with its tolerance", () => {
  const [o2, pa] = habitatFilters(moon);
  assert.deepEqual([o2.key, o2.min_si, o2.max_si], ["oxygen_mole_fraction", 0.295, 1]);
  assert.equal(pa.key, "pressure");
  assert.ok(Math.abs(pa.max_si! / PSI_PA - 10.2 * 1.01) < 1e-9 && Math.abs(pa.min_si! / PSI_PA - 4.3 * 0.99) < 1e-9);
  const [o2Iss] = habitatFilters(iss);
  assert.deepEqual([o2Iss.min_si, +o2Iss.max_si!.toFixed(3)], [0.205, 0.215]);
});

test("a gap shows the table's own text; flags stay on the cell, burner-stream O2 marked as not the atmosphere", () => {
  assert.equal(quotedText('several values in one cell: "21; 26.5; 34"'), "21; 26.5; 34");
  assert.equal(quotedText("impossible value “3400%”: spreadsheet slip"), "3400%");
  assert.equal(quotedText("no row of the experimental table states it"), null);
  assert.deepEqual(
    flagTags(["approximate", "oxidizer_stream:burner", "composition_sum=0.980", "testsheet_agrees", "original_header:O", "new_flag"]).map((f) => [f.label, f.inline, f.notAtmosphere]),
    [
      ["~", true, false],
      ["burner stream", true, true],
      ["Σ 0.980", true, false],
      ["test sheet agrees", false, false],
      ["header reconciled", false, false],
      ["new flag", false, false],
    ],
    "meaning-changing flags on the cell, provenance notes in the detail, unknown flags kept by name",
  );
});
