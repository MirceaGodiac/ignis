// T3: the live client and episode controls. Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { ResearchEpisode, ResearchEvent } from "../src/contracts/generated";
import { availableActions, DraftKey } from "../src/research/episodes.ts";
import { emptyState, reduceLog } from "../src/research/eventLog.ts";
import { ApiError, httpClient } from "../src/research/http.ts";
import { encodeFrame } from "../src/research/sse.ts";
import { followEpisode, type StreamTransport } from "../src/research/stream.ts";

const fixture = <T>(p: string): T => JSON.parse(readFileSync(fileURLToPath(new URL(`../../fixtures/contracts/${p}`, import.meta.url)), "utf8")) as T;
const EVENTS = fixture<ResearchEvent[]>("research/events.json");
const EP = EVENTS[0].episode_id;

type Call = { url: string; method: string; body: unknown; headers: Record<string, string> };

function recorder(reply: (url: string) => Response = () => Response.json({})) {
  const calls: Call[] = [];
  const f = (async (input: RequestInfo | URL, init?: RequestInit) => {
    const headers: Record<string, string> = {};
    new Headers(init?.headers).forEach((v, k) => (headers[k] = v));
    calls.push({ url: String(input), method: init?.method ?? "GET", body: init?.body ? JSON.parse(String(init.body)) : undefined, headers });
    return reply(String(input));
  }) as typeof fetch;
  return { f, calls };
}

test("the draft key is kept on retry and replaced after a success or an edit", () => {
  let n = 0;
  const k = new DraftKey(() => `k${++n}`);
  assert.equal(k.current, "k1");
  assert.equal(k.current, "k1", "a retry of the same draft reuses its key");
  assert.equal(k.succeeded(), "k2");
  assert.equal(k.current, "k2");
  assert.equal(k.edited(), "k3", "a changed text is a different request");
});

test("only the actions the server accepts are offered", () => {
  assert.deepEqual(availableActions("running"), ["pause", "cancel"]);
  assert.deepEqual(availableActions("ready"), ["pause", "cancel"]);
  assert.deepEqual(availableActions("waiting_approval"), ["pause", "cancel"]);
  assert.deepEqual(availableActions("paused"), ["resume", "cancel"]);
  for (const s of ["completed", "failed", "cancelled"] as const) assert.deepEqual(availableActions(s), [], s);
  assert.deepEqual(availableActions(null), []);
});

test("the HTTP client sends the server's routes and bodies under the selected origin", async () => {
  const { f, calls } = recorder();
  const c = httpClient("fixtures", f);
  await c.createEpisode("Goal", "req-1");
  await c.postMessage("fixture-episode-01~ws_1", "Hello", "msg-1");
  await c.act("ep_1", "pause");
  await c.resolveApproval("ep_1", "apr 1", "denied");
  await c.readArtifactValue("art_1", "/mean/3");
  await c.openEvents("ep_1", 7, new AbortController().signal);
  assert.deepEqual(
    calls.map((x) => `${x.method} ${x.url}`),
    [
      "POST /api/fixtures/v1/episodes",
      "POST /api/fixtures/v1/episodes/fixture-episode-01~ws_1/messages",
      "POST /api/fixtures/v1/episodes/ep_1/pause",
      "POST /api/fixtures/v1/episodes/ep_1/approvals/apr%201",
      "GET /api/fixtures/v1/artifacts/art_1?pointer=%2Fmean%2F3",
      "GET /api/fixtures/v1/episodes/ep_1/events?after=7",
    ],
  );
  assert.deepEqual(calls[0].body, { goal: "Goal", client_request_id: "req-1" });
  assert.deepEqual(calls[1].body, { text: "Hello", client_request_id: "msg-1" });
  assert.equal(calls[2].body, undefined, "an action has no body");
  assert.deepEqual(calls[3].body, { decision: "denied" });
  assert.equal(calls[5].headers["last-event-id"], "7");
  assert.equal(calls[5].headers.accept, "text/event-stream");

  const live = recorder();
  await httpClient("live", live.f).capabilities();
  assert.equal(live.calls[0].url, "/api/v1/capabilities");
});

test("a 503 keeps the server's ErrorEnvelope; a body without one gets a clear fallback", async () => {
  const env = { code: "capability_unavailable", message: "KnowledgeService is not configured", retry: { kind: "never" } };
  const { f } = recorder(() => Response.json(env, { status: 503 }));
  await assert.rejects(httpClient("live", f).search({ schema_version: "evidence.v1", query: "q" }), (e: unknown) => {
    assert.ok(e instanceof ApiError);
    assert.equal(e.httpStatus, 503);
    assert.deepEqual(e.envelope, env);
    return true;
  });
  const bare = recorder(() => new Response("proxy error", { status: 502 }));
  await assert.rejects(httpClient("live", bare.f).listEpisodes(), (e: unknown) => {
    assert.ok(e instanceof ApiError);
    assert.equal(e.envelope.code, "upstream_unavailable");
    assert.match(e.envelope.message, /HTTP 502 from \/api\/v1/);
    return true;
  });
});

/** An SSE response that sends `first`, then `later` after `delayMs`, and never closes. */
function openStream(first: ResearchEvent[], later: ResearchEvent[] = [], delayMs = 0): Response {
  const enc = new TextEncoder();
  const frame = (e: ResearchEvent) => enc.encode(encodeFrame(e.seq, e.body.kind, JSON.stringify(e)));
  const body = new ReadableStream<Uint8Array>({
    start(c) {
      first.forEach((e) => c.enqueue(frame(e)));
      if (later.length) setTimeout(() => later.forEach((e) => c.enqueue(frame(e))), delayMs);
    },
  });
  return new Response(body, { status: 200 });
}

const transport = (r: () => Response): StreamTransport => ({
  openEvents: async () => r(),
  getEpisode: async () => fixture<ResearchEpisode>("research/episode.json"),
});

test("a stream that stays open after a terminal status is closed once it settles", async () => {
  const t0 = Date.now();
  const s = await followEpisode(transport(() => openStream(EVENTS)), emptyState(EP), {
    signal: new AbortController().signal,
    onState: () => {},
    settleMs: 60,
  });
  assert.equal(s.status, "completed");
  assert.equal(s.cursor, 15);
  assert.ok(Date.now() - t0 < 2000, "the follow loop returned instead of holding the stream");
});

test("events after a terminal status, inside the settle window, are still applied", async () => {
  const reopen: ResearchEvent = { ...EVENTS[0], seq: 16, event_id: "reopen", body: { kind: "message.user", payload: { message_id: "m16", text: "One more question" } } };
  const s = await followEpisode(transport(() => openStream(EVENTS, [reopen], 20)), emptyState(EP), {
    signal: new AbortController().signal,
    onState: () => {},
    settleMs: 200,
  });
  assert.equal(s.cursor, 16);
  assert.equal(s.messages.m16.text, "One more question");
});

test("following again from a completed state opens the stream and shows a new message", async () => {
  // The user sends a message to a completed episode; the UI follows again from the cursor.
  const completed = reduceLog(EP, EVENTS).state;
  assert.equal(completed.status, "completed");
  const next: ResearchEvent = { ...EVENTS[0], seq: 16, event_id: "next", body: { kind: "message.user", payload: { message_id: "m16", text: "New question" } } };
  const opened: number[] = [];
  const t: StreamTransport = {
    openEvents: async (_id, after) => (opened.push(after), openStream(after < 16 ? [next] : [])),
    getEpisode: async () => fixture<ResearchEpisode>("research/episode.json"),
  };
  const s = await followEpisode(t, completed, { signal: new AbortController().signal, onState: () => {}, settleMs: 50 });
  assert.equal(opened[0], 15, "the stream is opened from the cursor, not skipped");
  assert.equal(s.cursor, 16);
  assert.equal(s.messages.m16.text, "New question");
});
