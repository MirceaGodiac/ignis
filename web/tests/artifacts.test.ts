// T7: contract coverage and the typed artifact helpers. Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { DatasetBundle, EvaluationResult, ExecutionRecord, ModelRelease } from "../src/contracts/generated";
import { checkSummary, columnUnit, CONTRACT_COVERAGE, exposureRows, UNCERTAINTY_TEXT, durationS, evaluationSummary, fmtMetric, isTypedArtifact, limitRows, outcomeText } from "../src/research/artifactViews.ts";

const fixture = <T>(p: string): T => JSON.parse(readFileSync(fileURLToPath(new URL(`../../fixtures/contracts/${p}`, import.meta.url)), "utf8")) as T;

test("every contract in IgnisContracts has a view or a reason it is not shown", () => {
  const src = readFileSync(fileURLToPath(new URL("../src/contracts/generated.ts", import.meta.url)), "utf8");
  const body = src.slice(src.indexOf("export interface IgnisContracts {"));
  const keys = [...body.slice(0, body.indexOf("\n}")).matchAll(/^\s{2}(\w+)\?:/gm)].map((m) => m[1]);
  assert.ok(keys.length > 40, "parsed the contract index");
  const covered = Object.keys(CONTRACT_COVERAGE);
  assert.deepEqual(keys.filter((k) => !covered.includes(k)), [], "unclassified contracts");
  assert.deepEqual(covered.filter((k) => !keys.includes(k)), [], "stale entries");
  for (const [k, c] of Object.entries(CONTRACT_COVERAGE)) {
    const text = "view" in c ? c.view : c.notShown;
    assert.ok(text.length > 8, `${k} needs a real view location or reason`);
  }
});

test("typed artifacts are recognised by contract name only", () => {
  assert.equal(isTypedArtifact("ExecutionRecord"), true);
  assert.equal(isTypedArtifact("ResearchAnswer"), false);
  assert.equal(isTypedArtifact(null), false);
});

test("execution record: exit code, duration and limits in words", () => {
  const r = fixture<ExecutionRecord>("research/execution_record.json");
  assert.deepEqual(outcomeText(r.outcome), { text: "Exited with code 0", ok: true });
  assert.deepEqual(outcomeText({ kind: "exited", exit_code: 2 }), { text: "Exited with code 2", ok: false });
  assert.equal(outcomeText({ kind: "timed_out" }).ok, false);
  assert.equal(durationS(r.started_at, r.finished_at), 2);
  assert.equal(durationS("bad", r.finished_at), null);
  const rows = Object.fromEntries(limitRows(r.limits));
  assert.equal(rows["wall time"], "600 s");
  assert.equal(rows.memory, "2 GiB");
  assert.equal(rows.CPU, "2 cores");
  assert.equal(r.rejected_outputs?.length, 1, "the refused symlink stays visible");
});

test("dataset bundle: a value column without a unit is flagged, never assumed", () => {
  const b = fixture<DatasetBundle>("data/dataset_bundle.json");
  const cols = b.views[0].columns;
  assert.deepEqual(columnUnit(cols.find((c) => c.name === "liquid_diameter_m")!), { text: "m", missing: false });
  assert.deepEqual(columnUnit(cols.find((c) => c.name === "run_id")!), { text: "—", missing: false });
  assert.deepEqual(columnUnit({ name: "x", role: "value", nullable: false }), { text: "no unit", missing: true });
});

test("evaluation result: coverage beside quality, refused cases kept in the denominator", () => {
  const r = fixture<EvaluationResult>("evaluation/result_dev.json");
  assert.deepEqual(evaluationSummary(r), { coveragePct: "75 %", scoredOf: "3 of 4 cases scored", denominatorKept: true });
  assert.equal(evaluationSummary({ ...r, cases: { total: 5, scored: 3, refused: 1, failed: 0 } }).denominatorKept, false);
  assert.equal(fmtMetric(r.metrics[0].value, r.metrics[0].unit), "4.00e-5 m");
  assert.equal(fmtMetric(null, "m"), "not computed");
  assert.equal(fmtMetric(0.25, "1"), "0.25");
});

test("release construction: checks are counted, not-run stays visible, no calibrated band is claimed", () => {
  const r = fixture<ModelRelease>("models/release_w0.json");
  const m = r.construction!;
  assert.deepEqual(checkSummary(m.checks), { passed: 3, failed: 0, notRun: 1, text: "3 passed, 1 not run" });
  assert.equal(checkSummary(undefined).text, "no build checks recorded");
  assert.equal(UNCERTAINTY_TEXT[m.uncertainty_kind], "No uncertainty band");
  assert.deepEqual(Object.fromEntries(exposureRows(m.exposure)), { sources: 1, datasets: 0, "dev feedback": 0, "prior candidates": 0 });
});
