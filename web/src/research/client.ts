// One research client, three origins with the same routes (crates/ignis-server README, /api/v1):
//   mock     -> the C0 fixtures in the browser (no server), SSE as real wire text;
//   fixtures -> the server's fixture origin `/api/fixtures/v1` (`ignis serve --dev`): real routes,
//               C0 fixture data;
//   live     -> `/api/v1`, the research runtime (M2) with whatever services are configured.
// Contract types come from contracts/generated.ts; server-only shapes from serverTypes.ts.

import type { ErrorEnvelope, InlinePayload, ResearchAnswer, ResearchEvent } from "../contracts/generated";
import { FIXTURES } from "./fixtures";
import { ApiError, type ResearchClient } from "./http";
import type { Capabilities } from "./serverTypes";
import { encodeFrame, HEARTBEAT } from "./sse";
import { resolvePointer } from "./units";

export { ApiError, httpClient, MODE_LABEL, modeOrigin, type Mode, type ResearchClient } from "./http";

/** Test switches for the mock stream (each fires once per page load, then the stream behaves). */
export interface MockStreamFaults {
  /** Close the connection right after this seq (the client must resume without duplicates). */
  dropAfterSeq: number | null;
  /** Answer the next resume with 410 (the client must reload the snapshot). */
  goneOnResume: boolean;
  /** Resend this many already-delivered events on resume (the server-side overlap case). */
  resumeOverlap: number;
  /** Delay between events, ms. */
  pacingMs: number;
}

export const DEFAULT_FAULTS: MockStreamFaults = { dropAfterSeq: null, goneOnResume: false, resumeOverlap: 2, pacingMs: 300 };

// ---------------------------------------------------------------------------------------------- mock

const notInFixtures = (what: string): ErrorEnvelope => ({
  code: "not_found",
  message: `${what} is not in the C0 contract fixtures (mock mode serves only fixtures/contracts/).`,
  retry: { kind: "never" },
});

const MOCK_ONLY: ErrorEnvelope = {
  code: "unsupported",
  message: "Mock mode replays a fixed fixture log in the browser. Use Server fixtures or Live to create episodes, steer or change them.",
  retry: { kind: "never" },
};

const MOCK_CAPABILITIES: Capabilities = {
  agent_runtime: { available: false, detail: "mock mode: no agent runtime in the browser" },
  knowledge: { available: true, detail: "C0 fixtures (search_page.json, read_result.json)" },
  datasets: { available: false, detail: "mock mode: no DatasetService" },
  models: { available: true, detail: "C0 fixture release_w0.json" },
  artifact_store: { available: true, detail: "C0 fixture answer only" },
  mcp: { available: false, detail: "mock mode" },
  tools: [],
};

function mockSse(log: ResearchEvent[], after: number, pacingMs: number, dropAfter: number | null, overlap: number, signal: AbortSignal): Response {
  const from = Math.max(0, after - overlap);
  const pending = log.filter((e) => e.seq > from);
  const enc = new TextEncoder();
  const body = new ReadableStream<Uint8Array>({
    async start(c) {
      const wait = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));
      c.enqueue(enc.encode(HEARTBEAT));
      for (const e of pending) {
        if (signal.aborted) return;
        await wait(e.seq > after ? pacingMs : 0); // replayed overlap arrives at once
        if (signal.aborted) return;
        c.enqueue(enc.encode(encodeFrame(e.seq, e.body.kind, JSON.stringify(e))));
        if (dropAfter !== null && e.seq === dropAfter) {
          c.close(); // connection lost mid-episode
          return;
        }
      }
      c.close();
    },
  });
  return new Response(body, { status: 200, headers: { "content-type": "text/event-stream" } });
}

const EPISODES = [FIXTURES.episode, FIXTURES.v2.episode];
const logOf = (id: string) => (id === FIXTURES.episode.episode_id ? FIXTURES.events : id === FIXTURES.v2.episode.episode_id ? FIXTURES.v2.events : null);
const answerOf = (artifactId: string) => (artifactId === FIXTURES.answerArtifactId ? FIXTURES.answer : artifactId === FIXTURES.v2.answerArtifactId ? FIXTURES.v2.answer : null);
const CONDITIONS_ARTIFACT = "fixture-artifact-conditions-01";

export function mockClient(faults: MockStreamFaults): ResearchClient {
  let dropPending = faults.dropAfterSeq;
  let gonePending = faults.goneOnResume;
  const latency = () => new Promise((r) => setTimeout(r, 120));
  const ok = async <T>(v: T): Promise<T> => (await latency(), structuredClone(v));
  const fail = async (env: ErrorEnvelope, status: number): Promise<never> => {
    await latency();
    throw new ApiError(env, status);
  };
  return {
    mode: "mock",
    async openEvents(id, after, signal) {
      const log = logOf(id);
      if (!log) return new Response(JSON.stringify(notInFixtures(`episode ${id}`)), { status: 404 });
      if (after > 0 && gonePending) {
        gonePending = false;
        const env: ErrorEnvelope = { code: "conflict", message: "mock: the event log no longer covers this cursor", retry: { kind: "reconcile" } };
        return new Response(JSON.stringify(env), { status: 410 });
      }
      const drop = dropPending;
      dropPending = null;
      return mockSse(log, after, faults.pacingMs, drop, after > 0 ? faults.resumeOverlap : 0, signal);
    },
    getEpisode: (id) => {
      const e = EPISODES.find((x) => x.episode_id === id);
      return e ? ok(e) : fail(notInFixtures(`episode ${id}`), 404);
    },
    capabilities: () => ok(MOCK_CAPABILITIES),
    listEpisodes: () => ok(EPISODES),
    createEpisode: () => fail(MOCK_ONLY, 501),
    act: () => fail(MOCK_ONLY, 501),
    resolveApproval: () => fail(MOCK_ONLY, 501),
    getArtifactJson: (id) => {
      const ans = answerOf(id);
      if (ans) return ok<ResearchAnswer>(ans);
      // The v2 answer cites a ConditionsTable artifact; the C0 conditions fixture is the only one there is.
      if (id === CONDITIONS_ARTIFACT) return ok(FIXTURES.conditionsTable);
      const typed = FIXTURES.typedArtifacts.find((t) => t.ref.artifact_id === id);
      return typed ? ok(typed.payload) : fail(notInFixtures(`artifact ${id} payload`), 404);
    },
    getArtifactText: (id) =>
      id === FIXTURES.visualBundle.contours.artifact_id
        ? ok(FIXTURES.imageContours.map((r) => JSON.stringify(r)).join("\n"))
        : fail(notInFixtures(`artifact ${id} text`), 404),
    readArtifactValue: (id, pointer) => {
      const ans = answerOf(id);
      if (!ans) return fail(notInFixtures(`artifact ${id} payload`), 404);
      const v = resolvePointer(ans, pointer);
      return v === undefined ? fail(notInFixtures(`pointer ${pointer} in ${id}`), 404) : ok<InlinePayload>({ value: v as InlinePayload["value"], partial: false });
    },
    search: () => ok(FIXTURES.searchPage),
    readSource: (id, page) =>
      id === FIXTURES.readResult.source_id && FIXTURES.readResult.segments.some((s) => s.locator.page === page)
        ? ok(FIXTURES.readResult)
        : fail(notInFixtures(`source ${id} page ${page}`), 404),
    sourceAccess: (id) => {
      const a = FIXTURES.sourceAccess.find((x) => x.source_id === id);
      return a ? ok(a) : fail(notInFixtures(`access to source ${id}`), 404);
    },
    conditions: (q) => {
      // The conditions service applies filters (with rules such as "a burner-stream O2 is never the
      // atmosphere"); mock mode does not re-implement them, it says so. Paging is plain slicing.
      if ((q.filters ?? []).length > 0) return fail({ code: "unsupported", message: "Mock mode cannot apply condition filters: the conditions service does. Use a server.", retry: { kind: "never" } }, 501);
      const t = FIXTURES.conditionsTable;
      const from = q.cursor?.startsWith("mock:") ? Number(q.cursor.slice(5)) || 0 : 0;
      const size = q.page_size ?? 1000;
      const rows = t.rows.slice(from, from + size);
      const next = from + rows.length;
      return ok({ ...t, rows, envelopes: q.include_envelopes ? t.envelopes : [], total_count: t.rows.length, next_cursor: next < t.rows.length ? `mock:${next}` : null });
    },
    listModels: () => ok(FIXTURES.releases),
    postMessage: () => fail(MOCK_ONLY, 501),
  };
}
