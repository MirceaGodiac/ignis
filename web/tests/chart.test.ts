// T8 `chart` rows, number display, ticks and artifact tables. Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import type { ResearchAnswer } from "../src/contracts/generated";
import { axisTitle, chartRows, rowCount, rowNoun, columnTitle, fmtNum, logTicks, niceTicks, skippedNote, sortRows, tableModel, type ChartView } from "../src/research/chartRows.ts";
import { viewProblems } from "../src/research/viewSpec.ts";

test("a chart footer counts tests when rows name their run, rows otherwise", () => {
  assert.equal(rowNoun([{ run: { run_id: "r1" }, o2: 0.21 }]), "test");
  assert.equal(rowNoun([{ t_s: 1, d_mm: 2 }]), "row");
  assert.equal(rowCount(17, "test"), "17 tests");
  assert.equal(rowCount(1, "row"), "1 row");
});

const doc = {
  rows: [
    { run: { run_id: "A" }, p_kpa: 50, o2: 0.21, inv: "flex" },
    { run: { run_id: "B" }, p_kpa: 101.3, o2: 0.4, inv: "cfi" },
    { run: { run_id: "C" }, p_kpa: "high", o2: 0.3, inv: "flex" },
    { run: { run_id: "D" }, p_kpa: 70, o2: null, inv: "cfi" },
    { run: { run_id: "E" }, p_kpa: 0, o2: 0.5, inv: "flex" },
  ],
};
const view: ChartView = {
  kind: "chart",
  artifact_id: "art-1",
  pointer: "/rows",
  mark: "scatter",
  x: { field: "/p_kpa", title: "Pressure", unit: "kPa" },
  y: [{ field: "/o2", title: "O2 fraction" }],
  group: "/inv",
  label: "/run/run_id",
};

test("chart rows: x, y, group and label by pointer relative to each row; non-numeric rows skipped and counted", () => {
  const d = chartRows(doc, view);
  assert.ok(typeof d !== "string");
  assert.equal(d.categorical, false);
  assert.deepEqual(
    d.points.map((p) => [p.label, p.x, p.y[0], p.group]),
    [
      ["A", 50, 0.21, "flex"],
      ["B", 101.3, 0.4, "cfi"],
      ["E", 0, 0.5, "flex"],
    ],
  );
  assert.deepEqual(d.groups, ["flex", "cfi"]);
  assert.equal(d.skipped, 2);
  assert.equal(skippedNote(d), "2 of 5 rows not drawn (1: Pressure is not a number; 1: O2 fraction is not a number)");
});

test("a log axis skips values ≤ 0; a bar mark makes x categorical", () => {
  const log = chartRows(doc, { ...view, x: { ...view.x, log: true } });
  assert.ok(typeof log !== "string");
  assert.deepEqual(log.points.map((p) => p.label), ["A", "B"]);
  assert.equal(log.skippedWhy["Pressure ≤ 0 on a log axis"], 1);
  const bars = chartRows(doc, { ...view, mark: "bar", x: { field: "/run/run_id", title: "Run" }, group: null });
  assert.ok(typeof bars !== "string");
  assert.equal(bars.categorical, true);
  assert.deepEqual(bars.categories, ["A", "B", "C", "E"]);
  assert.deepEqual(bars.groups, [null]);
});

test("a pointer that is not an array of rows is refused with the reason", () => {
  assert.equal(chartRows(doc, { ...view, pointer: "/nope" }), "/nope does not exist in the artifact");
  assert.equal(chartRows(doc, { ...view, pointer: "/rows/0" }), "/rows/0 is not an array of rows");
});

test("a chart view must cite its artifact and use JSON Pointers", () => {
  const answer = { artifacts: [{ artifact_id: "art-1" }] } as unknown as ResearchAnswer;
  assert.deepEqual(viewProblems(view, answer), []);
  assert.deepEqual(viewProblems({ ...view, artifact_id: "other", x: { field: "p_kpa", title: "P" } }, answer), ["artifact other is not cited by the answer", "x field p_kpa is not a JSON Pointer"]);
});

test("numbers show at most 4 significant digits; integers below a million whole", () => {
  assert.deepEqual(
    [0.50665, 0.3926, 101.325, 2010, 123456, 1234567, 0.000123456, -3.14159, 0].map(fmtNum),
    ["0.5067", "0.3926", "101.3", "2010", "123456", "1.235e6", "1.235e-4", "-3.142", "0"],
  );
});

test("ticks: round linear steps covering the range; whole decades on a log axis", () => {
  assert.deepEqual(niceTicks(0.21, 0.5, 5), [0.2, 0.3, 0.4, 0.5]);
  assert.deepEqual(niceTicks(0, 101.3, 5), [0, 20, 40, 60, 80, 100, 120]);
  assert.deepEqual(logTicks(3, 4000), { domain: [1, 10000], ticks: [1, 10, 100, 1000, 10000] });
  assert.deepEqual(logTicks(3, 40)?.ticks, [1, 2, 5, 10, 20, 50, 100]);
});

test("artifact tables: nested objects flattened, units from key suffixes, numeric columns, sorting", () => {
  const t = tableModel(doc.rows)!;
  assert.deepEqual(
    t.columns.map((c) => [c.key, c.title, c.unit, c.numeric]),
    [
      ["run.run_id", "run · run id", null, false],
      ["p_kpa", "p", "kPa", false],
      ["o2", "o2", null, true],
      ["inv", "inv", null, false],
    ],
  );
  assert.deepEqual(columnTitle("time_s"), { title: "time", unit: "s" });
  const byO2 = sortRows(t.rows, 2, "desc").map((r) => r[0]);
  assert.deepEqual(byO2, ["E", "B", "C", "A", "D"]); // empty last
  assert.deepEqual(tableModel({ a: 1 })!.rows, [["a", 1]]);
  assert.equal(tableModel(3), null);
});

test("ConditionsTable rows: a cell without a value drops that series' point only, and is counted when no y is left", () => {
  const table = {
    rows: [
      { run: { run_id: "R1" }, investigation_id: "FLEX", cells: [{ value: { value_si: 0.21 } }, { value: { value_si: 101325 } }] },
      { run: { run_id: "R2" }, investigation_id: "CFI", cells: [{ status: "conflict" }, { value: { value_si: 50000 } }] },
      { run: { run_id: "R3" }, investigation_id: "CFI", cells: [{ value: { value_si: 0.4 } }, { status: "missing" }] },
    ],
  };
  const v: ChartView = {
    kind: "chart",
    artifact_id: "t",
    pointer: "/rows",
    mark: "scatter",
    x: { field: "/cells/1/value/value_si", title: "Pressure", unit: "Pa" },
    y: [{ field: "/cells/0/value/value_si", title: "O2", unit: "1" }],
    group: "/investigation_id",
    label: "/run/run_id",
  };
  const d = chartRows(table, v);
  assert.ok(typeof d !== "string");
  assert.deepEqual(d.points.map((p) => [p.label, p.x, p.y[0]]), [["R1", 101325, 0.21]]);
  assert.equal(d.skipped, 2);
  assert.equal(axisTitle(v.y[0]), "O2"); // dimensionless "1" is not printed; the value is shown as given
  assert.equal(axisTitle(v.x), "Pressure (Pa)");
});
