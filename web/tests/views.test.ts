// T8: an answer view renders only what the answer cites, and series share a chart only on one clock
// and one unit. Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { ResearchAnswer, SeriesRef, ViewSpec } from "../src/contracts/generated";
import { readSeries, seriesProblems, tableOf, viewProblems, type Series } from "../src/research/viewSpec.ts";

const fixture = <T>(p: string): T => JSON.parse(readFileSync(fileURLToPath(new URL(`../../fixtures/contracts/${p}`, import.meta.url)), "utf8")) as T;
const answer = fixture<ResearchAnswer>("research/answer_v2.json");
const views = answer.blocks.flatMap((b) => (b.type === "view" ? [b.view] : []));

test("every view in the v2 fixture answer passes; each kind it uses is covered", () => {
  assert.deepEqual(
    views.map((v) => v.kind),
    ["source_excerpt", "series_comparison", "conditions_matrix", "release_card"],
  );
  for (const v of views) assert.deepEqual(viewProblems(v, answer), [], v.kind);
});

test("a view that references what the answer does not cite is refused, with the reason", () => {
  const bad: ViewSpec[] = [
    { kind: "source_excerpt", evidence_id: "not-cited" },
    { kind: "release_card", release_id: "sha256:other" },
    { kind: "artifact_table", artifact_id: "not-cited", pointer: "/rows" },
    { kind: "atmosphere_map", table: "fixture-artifact-forecast-01" },
  ];
  assert.deepEqual(
    bad.map((v) => viewProblems(v, answer)),
    [
      ["evidence not-cited is not cited by the answer"],
      ["release sha256:other is not listed by the answer"],
      ["artifact not-cited is not cited by the answer"],
      ["artifact fixture-artifact-forecast-01 has contract AnalysisArtifact, not ConditionsTable"],
    ],
  );
  const series = views.find((v) => v.kind === "series_comparison") as Extract<ViewSpec, { kind: "series_comparison" }>;
  const mislabelled = { kind: "series_comparison" as const, series: series.series.map((s) => ({ ...s, role: "observed" as const })) };
  assert.ok(viewProblems(mislabelled, answer).some((p) => p.includes("is model output but is labelled observed")));
  // An artifact that names no contract cannot stand in for one (the observed fixture artifact has none).
  const predictedFromUntyped = { kind: "series_comparison" as const, series: [{ ...series.series[0], artifact_id: "fixture-artifact-observed-01", role: "predicted" as const }] };
  assert.deepEqual(viewProblems(predictedFromUntyped, answer), ["artifact fixture-artifact-observed-01 has contract none, not AnalysisArtifact"]);
});

const ref = (over: Partial<SeriesRef> = {}): SeriesRef => ({ artifact_id: "a", role: "observed", time_pointer: "/t", value_pointer: "/v", unit_pointer: "/u", time_origin_pointer: "/o", ...over });

test("series are read by pointer and checked for length, unit and origin", () => {
  const doc = { t: [0, 1, 2], v: [1, null, 3], u: "m", o: "ignition", lo: [0, 0, 0], hi: [2, 2, 2] };
  const s = readSeries(doc, ref({ band: { lower_pointer: "/lo", upper_pointer: "/hi" } })) as Series;
  assert.deepEqual([s.times, s.values, s.unit, s.timeOrigin, s.band?.upper], [[0, 1, 2], [1, null, 3], "m", "ignition", [2, 2, 2]]);
  assert.equal(readSeries({ ...doc, v: [1, 2] }, ref()), "2 values for 3 times");
  assert.equal(readSeries({ ...doc, u: "" }, ref()), "/u does not name a unit");
  assert.equal(readSeries({ ...doc, o: null }, ref()), "/o does not name a time origin");
  const other = readSeries({ ...doc, o: "igniter_retracted", u: "mm" }, ref()) as Series;
  assert.deepEqual(seriesProblems([s, s]), []);
  assert.deepEqual(seriesProblems([s, other]), ["the series start their clocks at different origins (ignition, igniter_retracted)", "the series are in different units (m, mm)"]);
});

test("artifact_table turns rows or an object into a table, nothing else", () => {
  assert.deepEqual(tableOf([{ a: 1 }, { a: 2, b: 3 }]), { columns: ["a", "b"], rows: [[1, undefined], [2, 3]] });
  assert.deepEqual(tableOf({ x: 1 }), { columns: ["field", "value"], rows: [["x", 1]] });
  assert.equal(tableOf("text"), null);
});
