// One research client, three origins with the same routes (crates/ignis-server README, /api/v1):
//   mock     -> the C0 fixtures in the browser (no server), SSE as real wire text;
//   fixtures -> the server's fixture origin `/api/fixtures/v1` (`ignis serve --dev`): real routes,
//               C0 fixture data;
//   live     -> `/api/v1`, the research runtime (M2) with whatever services are configured.
// Contract types come from contracts/generated.ts; server-only shapes from serverTypes.ts.

import type { ErrorEnvelope, InlinePayload, ResearchAnswer } from "../contracts/generated";
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

function mockSse(after: number, pacingMs: number, dropAfter: number | null, overlap: number, signal: AbortSignal): Response {
  const from = Math.max(0, after - overlap);
  const pending = FIXTURES.events.filter((e) => e.seq > from);
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
      if (id !== FIXTURES.episode.episode_id) return new Response(JSON.stringify(notInFixtures(`episode ${id}`)), { status: 404 });
      if (after > 0 && gonePending) {
        gonePending = false;
        const env: ErrorEnvelope = { code: "conflict", message: "mock: the event log no longer covers this cursor", retry: { kind: "reconcile" } };
        return new Response(JSON.stringify(env), { status: 410 });
      }
      const drop = dropPending;
      dropPending = null;
      return mockSse(after, faults.pacingMs, drop, after > 0 ? faults.resumeOverlap : 0, signal);
    },
    getEpisode: (id) => (id === FIXTURES.episode.episode_id ? ok(FIXTURES.episode) : fail(notInFixtures(`episode ${id}`), 404)),
    capabilities: () => ok(MOCK_CAPABILITIES),
    listEpisodes: () => ok([FIXTURES.episode]),
    createEpisode: () => fail(MOCK_ONLY, 501),
    act: () => fail(MOCK_ONLY, 501),
    resolveApproval: () => fail(MOCK_ONLY, 501),
    getArtifactJson: (id) => {
      if (id === FIXTURES.answerArtifactId) return ok<ResearchAnswer>(FIXTURES.answer);
      const typed = FIXTURES.typedArtifacts.find((t) => t.ref.artifact_id === id);
      return typed ? ok(typed.payload) : fail(notInFixtures(`artifact ${id} payload`), 404);
    },
    readArtifactValue: (id, pointer) => {
      if (id !== FIXTURES.answerArtifactId) return fail(notInFixtures(`artifact ${id} payload`), 404);
      const v = resolvePointer(FIXTURES.answer, pointer);
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
    listModels: () => ok(FIXTURES.releases),
    postMessage: () => fail(MOCK_ONLY, 501),
  };
}
