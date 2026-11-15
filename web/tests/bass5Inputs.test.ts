// The bass5 inputs table the forecast lab fills BASS runs from (tools/web/bass5_inputs.py): every run is either
// a row or excluded with a reason, a "complete" row holds every input the served card requires, and a "partial"
// row lists exactly the inputs it lacks (never a guessed value). Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const served = JSON.parse(readFileSync(new URL("../../configs/models/served/served.json", import.meta.url), "utf8"));
const table = JSON.parse(readFileSync(new URL("../public/data/bass5_inputs.json", import.meta.url), "utf8"));
const card = served.releases.find((r: { model_name: string }) => r.model_name === "bass5.wsp");
const required: string[] = card.inputs.filter((i: { required: boolean }) => i.required).map((i: { name: string }) => i.name);

test("every BASS run is a row or excluded with a reason, never both", () => {
  const rows = Object.keys(table.runs);
  const out = Object.keys(table.excluded);
  assert.equal(rows.length + out.length, table.counts.registry);
  assert.equal(rows.filter((id) => out.includes(id)).length, 0);
  for (const id of [...rows, ...out]) assert.match(id, /^PSI-25:[A-Za-z0-9]+$|^PSI-26:asrun\d*:test[\w-]*$/);
  for (const why of Object.values(table.excluded)) assert.ok(typeof why === "string" && why.length > 0);
});

test("a complete row holds every required input; a partial row lists exactly what it lacks", () => {
  for (const [id, row] of Object.entries(table.runs) as [string, { coverage: string; kind: string; inputs: Record<string, number>; missing?: string[] }][]) {
    const lacking = required.filter((n) => row.inputs[n] === undefined);
    assert.deepEqual(lacking, row.missing ?? [], id);
    assert.equal(row.coverage, lacking.length ? "partial" : "complete", id);
    assert.ok(["nose", "flat", "fabric"].includes(row.kind), id);
    for (const v of Object.values(row.inputs)) assert.ok(Number.isFinite(v), id);
  }
});
