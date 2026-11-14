// One research client, two transports with the same routes (INTEGRATION.md, proposed /api/v1 table):
//   live -> fetch('/api/v1/...') against Mircea's server (M2);
//   mock -> the C0 fixtures, served through the same route shapes, SSE as real wire text.
// Types come only from contracts/generated.ts.

import type {
  ErrorEnvelope,
  ModelRelease,
  ReadResult,
  ResearchAnswer,
  ResearchEpisode,
  SearchPage,
  SearchRequest,
} from "../contracts/generated";
import { FIXTURES } from "./fixtures";
import { encodeFrame, HEARTBEAT } from "./sse";
import type { StreamTransport } from "./stream";

export type Mode = "mock" | "live";

export class ApiError extends Error {
  constructor(
    readonly envelope: ErrorEnvelope,
    readonly httpStatus: number | null,
  ) {
    super(`${envelope.code}: ${envelope.message}`);
  }
}

export interface ResearchClient extends StreamTransport {
  mode: Mode;
  getArtifactJson(artifactId: string): Promise<unknown>;
  search(req: SearchRequest): Promise<SearchPage>;
  readSource(sourceId: string, page: number): Promise<ReadResult>;
  listModels(): Promise<ModelRelease[]>;
  postMessage(episodeId: string, text: string, clientRequestId: string): Promise<void>;
}

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

// ---------------------------------------------------------------------------------------------- live

async function liveJson<T>(path: string, init?: RequestInit): Promise<T> {
  let res: Response;
  try {
    res = await fetch(`/api/v1${path}`, init);
  } catch (e) {
    throw new ApiError({ code: "upstream_unavailable", message: `network: ${String(e)}`, retry: { kind: "after_delay", seconds: 5 } }, null);
  }
  if (!res.ok) throw new ApiError(await envelopeOf(res), res.status);
  return (await res.json()) as T;
}

async function envelopeOf(res: Response): Promise<ErrorEnvelope> {
  try {
    const j = (await res.json()) as Partial<ErrorEnvelope>;
    if (j && typeof j.code === "string" && typeof j.message === "string" && j.retry) return j as ErrorEnvelope;
  } catch {
    /* not JSON */
  }
  return {
    code: res.status === 404 ? "not_found" : "upstream_unavailable",
    message: `HTTP ${res.status} from /api/v1 without an ErrorEnvelope (the /api/v1 server is task M2 and may not exist yet)`,
    retry: { kind: "never" },
  };
}

export function liveClient(): ResearchClient {
  const enc = encodeURIComponent;
  return {
    mode: "live",
    openEvents: (id, after, signal) =>
      fetch(`/api/v1/episodes/${enc(id)}/events?after=${after}`, { signal, headers: { accept: "text/event-stream", "last-event-id": String(after) } }),
    getEpisode: (id, signal) => liveJson<ResearchEpisode>(`/episodes/${enc(id)}`, { signal }),
    getArtifactJson: (id) => liveJson<unknown>(`/artifacts/${enc(id)}`),
    search: (req) => liveJson<SearchPage>(`/search`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(req) }),
    readSource: (id, page) => liveJson<ReadResult>(`/sources/${enc(id)}/pages/${page}`),
    listModels: () => liveJson<ModelRelease[]>(`/models`),
    postMessage: async (id, text, client_request_id) => {
      await liveJson<unknown>(`/episodes/${enc(id)}/messages`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ text, client_request_id }),
      });
    },
  };
}

// ---------------------------------------------------------------------------------------------- mock

const notInFixtures = (what: string): ErrorEnvelope => ({
  code: "not_found",
  message: `${what} is not in the C0 contract fixtures (mock mode serves only fixtures/contracts/).`,
  retry: { kind: "never" },
});

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
    getArtifactJson: (id) =>
      id === FIXTURES.answerArtifactId ? ok<ResearchAnswer>(FIXTURES.answer) : fail(notInFixtures(`artifact ${id} payload`), 404),
    search: () => ok(FIXTURES.searchPage),
    readSource: (id, page) =>
      id === FIXTURES.readResult.source_id && FIXTURES.readResult.segments.some((s) => s.locator.page === page)
        ? ok(FIXTURES.readResult)
        : fail(notInFixtures(`source ${id} page ${page}`), 404),
    listModels: () => ok(FIXTURES.releases),
    postMessage: () =>
      fail({ code: "unsupported", message: "mock mode: steering messages are not sent; the fixture log is fixed.", retry: { kind: "never" } }, 501),
  };
}
