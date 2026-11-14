// Run: npm --prefix web test   (node --test with native TypeScript type stripping; no extra deps)
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { ResearchEpisode, ResearchEvent, VisualObservationBundle } from "../src/contracts/generated";
import { applyEvent, emptyState, reduceLog, type EpisodeState } from "../src/research/eventLog.ts";
import { encodeFrame, HEARTBEAT, SseParser } from "../src/research/sse.ts";
import { followEpisode, type StreamNotice, type StreamTransport } from "../src/research/stream.ts";
import { bboxStyle, locatorKind, sliceSpan, spatialAxis } from "../src/research/units.ts";

const fixture = <T>(p: string): T => JSON.parse(readFileSync(fileURLToPath(new URL(`../../fixtures/contracts/${p}`, import.meta.url)), "utf8")) as T;
const EVENTS = fixture<ResearchEvent[]>("research/events.json");
const EPISODE = fixture<ResearchEpisode>("research/episode.json");
const EP = EVENTS[0].episode_id;

/** SSE body split into small chunks, so frames cross chunk boundaries. */
function sseResponse(events: ResearchEvent[], chunk = 37): Response {
  const text = HEARTBEAT + events.map((e) => encodeFrame(e.seq, e.body.kind, JSON.stringify(e))).join(HEARTBEAT);
  const bytes = new TextEncoder().encode(text);
  let i = 0;
  const body = new ReadableStream<Uint8Array>({
    pull(c) {
      if (i >= bytes.length) return c.close();
      c.enqueue(bytes.slice(i, i + chunk));
      i += chunk;
    },
  });
  return new Response(body, { status: 200, headers: { "content-type": "text/event-stream" } });
}

/** Transport that answers each open() from a script keyed by the call index. */
function scripted(steps: ((after: number) => Response)[], episode: ResearchEpisode = EPISODE) {
  const calls: number[] = [];
  const t: StreamTransport = {
    async openEvents(_id, after) {
      calls.push(after);
      const step = steps[Math.min(calls.length - 1, steps.length - 1)];
      return step(after);
    },
    async getEpisode() {
      return episode;
    },
  };
  return { t, calls };
}

async function follow(t: StreamTransport) {
  const notices: StreamNotice[] = [];
  const state = await followEpisode(t, emptyState(EP), {
    signal: new AbortController().signal,
    onState: () => {},
    onNotice: (n) => notices.push(n),
    sleep: async () => {},
  });
  return { state, notices };
}

const FULL: EpisodeState = reduceLog(EP, EVENTS).state;

test("full replay of the fixture log gives the expected final state", () => {
  const { state, stoppedAt } = reduceLog(EP, EVENTS);
  assert.equal(stoppedAt, null);
  assert.equal(state.cursor, 15);
  assert.equal(state.status, "completed");
  assert.equal(state.messages["fixture-msg-02"].text, "Synthetic fixture answer, complete text snapshot.");
  assert.deepEqual(
    state.toolOrder.map((id) => state.tools[id].result?.execution_status),
    ["completed", "accepted", "failed"],
  );
  assert.equal(state.jobs["fixture-job-02"].status, "succeeded");
  assert.ok(state.artifacts["fixture-artifact-forecast-01"]);
  assert.equal(state.answers.length, 1);
  assert.equal(state.answers[0].status, "partial");
  assert.deepEqual(state.missing, []);
});

test("stream cut at a cursor and resumed with overlap equals the full replay, no duplicates applied", async () => {
  for (const k of [1, 6, 11, 14]) {
    const { t, calls } = scripted([
      () => sseResponse(EVENTS.slice(0, k)), // connection drops after seq k
      (after) => sseResponse(EVENTS.slice(Math.max(0, after - 3))), // server resends an overlap
    ]);
    const { state, notices } = await follow(t);
    assert.deepEqual(calls, [0, k], `k=${k}`);
    assert.deepEqual(state, FULL, `k=${k}`);
    assert.equal(notices.filter((n) => n.kind === "duplicate").length, Math.min(3, k), `k=${k}`);
  }
});

test("a gap is not applied; the client resumes from its cursor", async () => {
  const withGap = [...EVENTS.slice(0, 5), EVENTS[7], EVENTS[8]]; // seq 1..5 then 8, 9
  const { t, calls } = scripted([() => sseResponse(withGap), (after) => sseResponse(EVENTS.slice(after))]);
  const { state, notices } = await follow(t);
  assert.deepEqual(notices.find((n) => n.kind === "gap"), { kind: "gap", expected: 6, got: 8 });
  assert.deepEqual(calls, [0, 5]);
  assert.deepEqual(state, FULL);
});

test("410 reloads the episode snapshot and records the unseen range", async () => {
  const snapshot: ResearchEpisode = { ...EPISODE, status: "completed" };
  const gone = () => new Response(JSON.stringify({ code: "conflict", message: "log truncated", retry: { kind: "never" } }), { status: 410 });
  const { t, calls } = scripted([() => sseResponse(EVENTS.slice(0, 5)), gone], snapshot);
  const { state, notices } = await follow(t);
  assert.deepEqual(calls, [0, 5]);
  assert.equal(state.cursor, snapshot.last_seq);
  assert.equal(state.status, "completed");
  assert.deepEqual(state.missing, [{ from: 6, to: 15 }]);
  assert.ok(notices.some((n) => n.kind === "resync"));
  assert.equal(state.events.length, 5, "events never seen are not invented");
});

test("an HTTP error carries its ErrorEnvelope and the loop gives up without progress", async () => {
  const err = () => new Response(JSON.stringify({ code: "scope_denied", message: "no", retry: { kind: "never" } }), { status: 403 });
  const { t } = scripted([err]);
  const { state, notices } = await follow(t);
  assert.equal(state.cursor, 0);
  const e = notices.find((n) => n.kind === "error");
  assert.ok(e && e.kind === "error" && e.error.code === "scope_denied" && e.http_status === 403);
  assert.ok(notices.some((n) => n.kind === "gave_up"));
});

test("message.completed replaces deltas; a late delta never grows a completed message", () => {
  const delta = EVENTS.find((e) => e.body.kind === "message.delta")!;
  let s = reduceLog(EP, EVENTS).state;
  const late: ResearchEvent = { ...delta, seq: 16, event_id: "late-delta" };
  s = applyEvent(s, late).state;
  assert.equal(s.cursor, 16);
  assert.equal(s.messages["fixture-msg-02"].text, "Synthetic fixture answer, complete text snapshot.");
  // A replayed delta (same seq) is a duplicate and is not applied.
  assert.equal(applyEvent(s, delta).outcome, "duplicate");
});

test("events of another episode are rejected", () => {
  const foreign: ResearchEvent = { ...EVENTS[0], episode_id: "other" };
  assert.equal(applyEvent(emptyState(EP), foreign).outcome, "foreign_episode");
});

test("SSE parser handles split chunks, CRLF, comments and multi-line data", () => {
  const p = new SseParser();
  const frames = [...p.push(": hb\r\nid: 7\r\nevent: x\r\nda"), ...p.push("ta: a\r\ndata: b\r\n\r\n")];
  assert.deepEqual(frames, [{ id: "7", event: "x", data: "a\nb" }]);
});

test("pixel_only observations never get a metre axis", () => {
  const bundle = fixture<VisualObservationBundle>("data/visual_bundle_pixel_only.json");
  const axis = spatialAxis(bundle.units_status);
  assert.equal(axis.unit, "px");
  assert.notEqual(axis.unit, "m");
  assert.match(axis.label, /not metres/);
  assert.equal(spatialAxis("metric").unit, "m");
});

test("locator spans are cut in their declared unit", () => {
  const t = "a😀b";
  assert.equal(sliceSpan(t, 1, 2, "unicode_scalar"), "😀");
  assert.equal(sliceSpan(t, 1, 3, "utf16_code_unit"), "😀");
  assert.equal(sliceSpan(t, 1, 5, "utf8_byte"), "😀");
  assert.equal(sliceSpan(t, 1, 3, "utf8_byte"), null, "a span inside a multi-byte char is refused");
  assert.equal(sliceSpan(t, 0, 1, null), null, "offsets without a unit are not citable");
  assert.equal(sliceSpan(t, 0, 9, "unicode_scalar"), null);
});

test("locator kind and bbox follow the SourceLocator conventions", () => {
  const answer = fixture<{ evidence: { locator: Parameters<typeof locatorKind>[0] }[] }>("research/answer.json");
  assert.equal(locatorKind(answer.evidence[0].locator), "page");
  assert.equal(locatorKind(answer.evidence[1].locator), "table");
  assert.equal(locatorKind(null), "metadata");
  assert.deepEqual(bboxStyle([0.12, 0.21, 0.88, 0.44]), { left: "12.00%", top: "21.00%", width: "76.00%", height: "23.00%" });
  assert.equal(bboxStyle([0.5, 0.5, 0.4, 0.6]), null);
});
