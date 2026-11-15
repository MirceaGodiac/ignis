// A declined step's message read as a head and a list; a limit read as its first sentence and the rest.
import test from "node:test";
import assert from "node:assert/strict";
import { compactKeys, leadSentence, refusalSummary, splitListMessage } from "../src/research/noteText.ts";

test("claim keys sort numerically and collapse into ranges; a long list becomes a count", () => {
  const keys = [...Array.from({ length: 17 }, (_, i) => String(i + 2)), "1"];
  assert.deepEqual(compactKeys(keys), { short: "Claims 1–18", full: "Claims 1–18" });
  assert.equal(compactKeys(["3", "1", "2", "7", "9", "10"]).short, "Claims 1–3, 7, 9, 10");
  assert.equal(compactKeys(["4"]).short, "Claim 4");
  const scattered = compactKeys(["1", "3", "5", "7", "9", "11", "13", "15", "17"]);
  assert.equal(scattered.short, "9 claims");
  assert.equal(scattered.full, "Claims 1, 3, 5, 7, 9, 11, 13, 15, 17");
});

test("a dataset not_found message becomes a head, the readable bundles and the tail", () => {
  const msg =
    "dataset sensors-psi-99 is not in this store; datasets readable in this session (pass the id, or a family name that names exactly one of them): " +
    "sensors-psi-10-flex (flex, PSI-10, fuel heptane, partitions train, 3 views); sensors-psi-69-acme (acme_normal, partitions train+validation, 2 views); " +
    "sensors-psi-159-bass (bass, partitions train, 1 views); 2 more in the store are not readable in this session (a partition it does not hold)";
  const r = splitListMessage(msg);
  assert.ok(r);
  assert.deepEqual(r.head, [
    "Dataset sensors-psi-99 is not in this store",
    "Datasets readable in this session (pass the id, or a family name that names exactly one of them)",
  ]);
  assert.deepEqual(
    r.items.map((i) => i.name),
    ["sensors-psi-10-flex", "sensors-psi-69-acme", "sensors-psi-159-bass"],
  );
  assert.equal(r.items[0].note, "flex, PSI-10, fuel heptane, partitions train, 3 views");
  assert.deepEqual(r.tail, ["2 more in the store are not readable in this session (a partition it does not hold)"]);
});

test("colons inside ids or parentheses do not start a list; plain messages stay as they came", () => {
  assert.equal(splitListMessage("run PSI-69:193F001:2010-03-23 is not in this store"), null);
  assert.equal(splitListMessage("invalid input: limit must be positive"), null);
  assert.equal(splitListMessage("not found"), null);
  const r = splitListMessage("known families (see docs: catalogue): flex; bass; acme_normal.");
  assert.ok(r);
  assert.deepEqual(r.head, ["Known families (see docs: catalogue)"]);
  assert.deepEqual(r.items, [
    { name: "flex", note: null },
    { name: "bass", note: null },
    { name: "acme_normal", note: null },
  ]);
  assert.deepEqual(r.tail, []);
});

test("one item counts as a list only with its parenthetical", () => {
  assert.equal(splitListMessage("readable: flex")?.items, undefined);
  assert.deepEqual(splitListMessage("readable: sensors-1 (flex, 3 views)")?.items, [{ name: "sensors-1", note: "flex, 3 views" }]);
});

test("a limit's first sentence leads; abbreviations do not end it", () => {
  assert.deepEqual(leadSentence("Only two runs were read, e.g. FLEX-093. The rest were not opened."), {
    lead: "Only two runs were read, e.g. FLEX-093.",
    rest: "The rest were not opened.",
  });
  assert.deepEqual(leadSentence("One sentence only"), { lead: "One sentence only", rest: "" });
});

test("every error code reads as one human line", () => {
  assert.equal(refusalSummary("not_found", "datasets_inspect"), "This dataset isn't available here");
  assert.equal(refusalSummary("not_found", "web_fetch"), "What this step asked for isn't available here");
  assert.equal(refusalSummary("capability_unavailable"), "This isn't set up on this server");
  assert.equal(refusalSummary(undefined), "This step did not run");
});
