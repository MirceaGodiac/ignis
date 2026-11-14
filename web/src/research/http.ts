// The HTTP transport of the research client: `/api/fixtures/v1` and `/api/v1` (crates/ignis-server
// README). Pure of runtime imports except types, so it runs under node --test with an injected fetch.

import type {
  ConditionsQuery,
  ConditionsTable,
  ErrorEnvelope,
  InlinePayload,
  ModelRelease,
  Origin,
  ReadResult,
  ResearchEpisode,
  SearchPage,
  SearchRequest,
  SourceAccess,
} from "../contracts/generated";
import type { Capabilities, CreateEpisode, EpisodeAction, HostDecision, MessageAccepted, PostMessage } from "./serverTypes";
import type { StreamTransport } from "./stream";

export type Mode = "mock" | "fixtures" | "live";

export const MODE_LABEL: Record<Mode, string> = { mock: "Mock", fixtures: "Server fixtures", live: "Live" };

/** Origin of what a mode shows: both fixture modes are synthetic. */
export function modeOrigin(m: Mode): Origin {
  return m === "live" ? "system" : "synthetic_fixture";
}

export class ApiError extends Error {
  // Plain fields, not parameter properties: node's type stripping runs this module in tests.
  readonly envelope: ErrorEnvelope;
  readonly httpStatus: number | null;
  constructor(envelope: ErrorEnvelope, httpStatus: number | null) {
    super(`${envelope.code}: ${envelope.message}`);
    this.envelope = envelope;
    this.httpStatus = httpStatus;
  }
}

export interface ResearchClient extends StreamTransport {
  mode: Mode;
  capabilities(): Promise<Capabilities>;
  listEpisodes(): Promise<ResearchEpisode[]>;
  createEpisode(goal: string, clientRequestId: string): Promise<ResearchEpisode>;
  act(episodeId: string, action: EpisodeAction): Promise<ResearchEpisode>;
  resolveApproval(episodeId: string, approvalId: string, decision: HostDecision): Promise<void>;
  getArtifactJson(artifactId: string): Promise<unknown>;
  /** A text artifact (NDJSON records, CSV) as its bytes decoded as UTF-8. */
  getArtifactText(artifactId: string): Promise<string>;
  /** One value of a JSON artifact by RFC 6901 pointer, without downloading the artifact. */
  readArtifactValue(artifactId: string, pointer: string): Promise<InlinePayload>;
  search(req: SearchRequest): Promise<SearchPage>;
  readSource(sourceId: string, page: number): Promise<ReadResult>;
  /** How to open the original (LANES_V2 M12, K1.18). The URL is short-lived; never store or cite it. */
  sourceAccess(sourceId: string): Promise<SourceAccess>;
  /** Per-run conditions and tested envelopes (K1.13). Route from M11; until it lands, a 404 here is expected. */
  conditions(query: ConditionsQuery): Promise<ConditionsTable>;
  listModels(): Promise<ModelRelease[]>;
  postMessage(episodeId: string, text: string, clientRequestId: string): Promise<MessageAccepted>;
}

// ---------------------------------------------------------------------------------------------- http

const BASE: Record<Exclude<Mode, "mock">, string> = { fixtures: "/api/fixtures/v1", live: "/api/v1" };

async function envelopeOf(res: Response, base: string): Promise<ErrorEnvelope> {
  try {
    const j = (await res.json()) as Partial<ErrorEnvelope>;
    if (j && typeof j.code === "string" && typeof j.message === "string" && j.retry) return j as ErrorEnvelope;
  } catch {
    /* not JSON */
  }
  return {
    code: res.status === 404 ? "not_found" : "upstream_unavailable",
    message: `HTTP ${res.status} from ${base} without an ErrorEnvelope (is \`ignis serve\` running${base.includes("fixtures") ? " with --dev" : ""}?)`,
    retry: { kind: "never" },
  };
}

/** The HTTP transport. `fetchImpl` is injectable for tests. */
export function httpClient(mode: Exclude<Mode, "mock">, fetchImpl: typeof fetch = (...a) => fetch(...a)): ResearchClient {
  const base = BASE[mode];
  const enc = encodeURIComponent;
  const raw = async (path: string, init?: RequestInit): Promise<Response> => {
    let res: Response;
    try {
      res = await fetchImpl(`${base}${path}`, init);
    } catch (e) {
      if ((e as Error)?.name === "AbortError") throw e;
      throw new ApiError({ code: "upstream_unavailable", message: `network: ${String(e)}`, retry: { kind: "after_delay", seconds: 5 } }, null);
    }
    if (!res.ok) throw new ApiError(await envelopeOf(res, base), res.status);
    return res;
  };
  const call = async <T>(path: string, init?: RequestInit): Promise<T> => (await (await raw(path, init)).json()) as T;
  const post = <T>(path: string, body?: unknown) =>
    call<T>(path, { method: "POST", headers: { "content-type": "application/json" }, body: body === undefined ? undefined : JSON.stringify(body) });
  return {
    mode,
    openEvents: (id, after, signal) =>
      fetchImpl(`${base}/episodes/${enc(id)}/events?after=${after}`, { signal, headers: { accept: "text/event-stream", "last-event-id": String(after) } }),
    getEpisode: (id, signal) => call<ResearchEpisode>(`/episodes/${enc(id)}`, { signal }),
    capabilities: () => call<Capabilities>(`/capabilities`),
    listEpisodes: () => call<ResearchEpisode[]>(`/episodes`),
    createEpisode: (goal, client_request_id) => post<ResearchEpisode>(`/episodes`, { goal, client_request_id } satisfies CreateEpisode),
    act: (id, action) => post<ResearchEpisode>(`/episodes/${enc(id)}/${action}`),
    resolveApproval: async (id, approvalId, decision) => {
      await post<unknown>(`/episodes/${enc(id)}/approvals/${enc(approvalId)}`, { decision });
    },
    getArtifactJson: (id) => call<unknown>(`/artifacts/${enc(id)}`),
    getArtifactText: async (id) => (await raw(`/artifacts/${enc(id)}`)).text(),
    readArtifactValue: (id, pointer) => call<InlinePayload>(`/artifacts/${enc(id)}?pointer=${enc(pointer)}`),
    search: (req) => post<SearchPage>(`/search`, req),
    readSource: (id, page) => call<ReadResult>(`/sources/${enc(id)}/pages/${page}`),
    sourceAccess: (id) => call<SourceAccess>(`/sources/${enc(id)}/content`),
    conditions: (query) => post<ConditionsTable>(`/conditions`, query),
    listModels: () => call<ModelRelease[]>(`/models`),
    postMessage: (id, text, client_request_id) => post<MessageAccepted>(`/episodes/${enc(id)}/messages`, { text, client_request_id } satisfies PostMessage),
  };
}
