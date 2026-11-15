// The original viewer numbers CSV records as the server does (1-based, header = 1, a quoted field may
// span lines, CRLF is one break, blank lines are records), so a locator's `row` marks the same record.
import test from "node:test";
import assert from "node:assert/strict";
import { csvEnd, csvFeed, csvFields, csvStart } from "../src/research/csvRecords.ts";

test("records split across chunks, quoted newlines and CRLF", () => {
  const st = csvStart();
  const text = 'Run,Note,O2\r\nFLEX-093,"two\nlines",0.21\r\n\r\nFLEX-094,"a ""q""",0.4';
  const recs = [...csvFeed(st, text.slice(0, 17)), ...csvFeed(st, text.slice(17, 40)), ...csvFeed(st, text.slice(40)), ...csvEnd(st)];
  assert.equal(recs.length, 4);
  assert.deepEqual(csvFields(recs[0]), ["Run", "Note", "O2"]);
  assert.deepEqual(csvFields(recs[1]), ["FLEX-093", "two\nlines", "0.21"]);
  assert.equal(recs[2], "");
  assert.deepEqual(csvFields(recs[3]), ["FLEX-094", 'a "q"', "0.4"]);
});
