// Answer text: Markdown parsed into blocks and inline parts; tables keep their alignment, numeric
// columns are found, and run names stay links inside cells. Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import { isNumericCell, parseAlignRow, parseInline, parseMarkdown, splitTableRow } from "../src/research/markdown.ts";
import { runNameKey } from "../src/shell/richParts.ts";

const runs = new Map([[runNameKey("21106A4"), "PSI-159:acme:21106A4"]]);

test("a GFM table: header, alignment row, body; numeric columns right-aligned by default", () => {
  const md = [
    "Measured burns:",
    "",
    "| Run | O2 fraction | Pressure | Note |",
    "|:---|---:|---|:---:|",
    "| 21106A4 | 0.3926 | 0.50665 bar | **steady** |",
    "| 21107B1 | 0.21 | — | flame out |",
    "",
    "After the table.",
  ].join("\n");
  const b = parseMarkdown(md);
  assert.deepEqual(
    b.map((x) => x.kind),
    ["paragraph", "table", "paragraph"],
  );
  const t = b[1];
  assert.ok(t.kind === "table");
  assert.deepEqual(t.header, ["Run", "O2 fraction", "Pressure", "Note"]);
  assert.deepEqual(t.align, ["left", "right", null, "center"]);
  assert.deepEqual(t.rows, [
    ["21106A4", "0.3926", "0.50665 bar", "**steady**"],
    ["21107B1", "0.21", "—", "flame out"],
  ]);
  assert.deepEqual(t.numeric, [false, true, true, false]);
});

test("rows without outer pipes, escaped pipes, pipes in code, short and long rows", () => {
  assert.deepEqual(splitTableRow("a | b | c"), ["a", "b", "c"]);
  assert.deepEqual(splitTableRow("| a \\| b | `x|y` |"), ["a | b", "`x|y`"]);
  const t = parseMarkdown("x | y\n--- | ---\n1 |\n2 | 3 | 4\nnot a row")[0];
  assert.ok(t.kind === "table");
  assert.deepEqual(t.rows, [["1", ""], ["2", "3 | 4"]]);
});

test("alignment rows and what is not one", () => {
  assert.deepEqual(parseAlignRow("| :-: | --: |"), ["center", "right"]);
  assert.equal(parseAlignRow("---"), null); // a rule
  assert.equal(parseAlignRow("| a | b |"), null);
  assert.deepEqual(
    parseMarkdown("| a |\nno alignment row").map((x) => x.kind),
    ["paragraph"],
  );
});

test("numeric cells: numbers with units are numbers; run names and dates are not", () => {
  for (const s of ["0.3926", "1,234.5", "-3", "−3.2e-4", "12 %", "0.50665 bar", "~4 s", "**1.2**", "1e5"]) assert.ok(isNumericCell(s), s);
  for (const s of ["21106A4", "2010-03-23", "FLEX-093", "n/a", "", "steady", "v2"]) assert.ok(!isNumericCell(s), s);
});

test("blocks: headings, lists (with nesting), quotes, code, rules; single newlines stay in a paragraph", () => {
  const b = parseMarkdown("## Result\nline one\nline two\n- a\n  - a.1\n- b\n\n3. third\n4. fourth\n\n> quoted\n\n```json\n{\"a\": 1}\n```\n---");
  assert.deepEqual(
    b.map((x) => x.kind),
    ["heading", "paragraph", "list", "list", "quote", "code", "rule"],
  );
  assert.deepEqual(b[1], { kind: "paragraph", text: "line one\nline two" });
  assert.deepEqual(b[2], { kind: "list", ordered: false, start: 1, items: [{ text: "a", level: 0 }, { text: "a.1", level: 1 }, { text: "b", level: 0 }] });
  assert.ok(b[3].kind === "list" && b[3].ordered && b[3].start === 3);
  assert.deepEqual(b[5], { kind: "code", lang: "json", text: '{"a": 1}' });
});

test("inline: bold, italic, code, safe links only, run names linked inside markup", () => {
  assert.deepEqual(parseInline("**21106A4** burned *briefly* in `O2`", runs), [
    { kind: "bold", parts: [{ kind: "run", text: "21106A4", id: "PSI-159:acme:21106A4" }] },
    { kind: "text", text: " burned " },
    { kind: "italic", parts: [{ kind: "text", text: "briefly" }] },
    { kind: "text", text: " in " },
    { kind: "code", text: "O2" },
  ]);
  assert.deepEqual(parseInline("[NASA](https://nasa.gov) and [x](javascript:alert(1))", runs), [
    { kind: "link", label: "NASA", href: "https://nasa.gov" },
    { kind: "text", text: " and " },
    { kind: "text", text: "x" },
    { kind: "text", text: ")" },
  ]);
  // snake_case is not italic; a lone asterisk stays literal
  assert.deepEqual(parseInline("run_id_x * 2", runs), [{ kind: "text", text: "run_id_x * 2" }]);
});
