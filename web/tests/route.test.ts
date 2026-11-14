// research.v2 route: arguments and the model's explanations attach to the right call, in any order.
// Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { ResearchEvent } from "../src/contracts/generated";
import { applyEvent, reduceLog } from "../src/research/eventLog.ts";

const fixture = <T>(p: string): T => JSON.parse(readFileSync(fileURLToPath(new URL(`../../fixtures/contracts/${p}`, import.meta.url)), "utf8")) as T;
const EVENTS = fixture<ResearchEvent[]>("research/events_v2.json");
const EP = EVENTS[0].episode_id;

test("v2 fixture log: arguments on each call, an early explanation attached at start, an unnamed one as a note", () => {
  const s = reduceLog(EP, EVENTS).state;
  const search = s.tools["fixture-call-11"];
  assert.equal(search.tool, "sources_search");
  assert.equal((search.arguments?.value.value as { query: string }).query, "ambient oxygen fraction effect on droplet burning rate (synthetic fixture query)");
  assert.deepEqual(
    search.rationale.map((n) => n.seq),
    [3],
    "the explanation came before tool.started and still belongs to the call",
  );
  assert.ok(search.result, "the finished result keeps the arguments and explanation");
  assert.equal(s.tools["fixture-call-14"].arguments?.value.partial, true, "a large argument is a preview pointing at its artifact");
  assert.deepEqual(
    s.notes.map((n) => n.seq),
    [6],
  );
  assert.deepEqual(s.pendingRationale, {});
});

test("an explanation that arrives after its call finished is appended, not lost", () => {
  const base = reduceLog(EP, EVENTS.slice(0, 5)).state;
  const late: ResearchEvent = { ...EVENTS[2], seq: 6, body: { kind: "step.rationale", payload: { call_id: "fixture-call-11", text: "late", source: "model_explanation" } } };
  const { state, outcome } = applyEvent(base, late);
  assert.equal(outcome, "applied");
  assert.deepEqual(
    state.tools["fixture-call-11"].rationale.map((n) => n.text),
    [EVENTS[2].body.kind === "step.rationale" ? EVENTS[2].body.payload.text : "", "late"],
  );
});
