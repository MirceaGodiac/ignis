// Resumable episode stream (CONTRACTS.md §Streaming): GET .../events?after=<cursor> over fetch,
// SSE parsed by hand, dedup/gap handling in eventLog, 410 -> reload the episode and continue from
// `last_seq`. Reconnecting never re-sends a command. The transport is injected, so the same loop
// runs against /api/v1 and against the fixture transport. Pure module: type-only imports.

import type { ErrorEnvelope, EpisodeStatus, ResearchEpisode, ResearchEvent } from "../contracts/generated";
import { applyEvent, resyncFromSnapshot, type EpisodeState } from "./eventLog.ts";
import { SseParser } from "./sse.ts";

export interface StreamTransport {
  /** Must resolve to the raw HTTP response of `GET /api/v1/episodes/{id}/events?after=<after>`. */
  openEvents(episodeId: string, after: number, signal: AbortSignal): Promise<Response>;
  getEpisode(episodeId: string, signal: AbortSignal): Promise<ResearchEpisode>;
}

export type StreamNotice =
  | { kind: "connected"; after: number }
  | { kind: "ended"; cursor: number }
  | { kind: "duplicate"; seq: number }
  | { kind: "gap"; expected: number; got: number }
  | { kind: "foreign_event"; episode_id: string }
  | { kind: "resync"; from: number; to: number }
  | { kind: "error"; error: ErrorEnvelope; http_status: number | null }
  | { kind: "gave_up"; reason: string };

export interface FollowOptions {
  signal: AbortSignal;
  onState: (s: EpisodeState) => void;
  onNotice?: (n: StreamNotice) => void;
  /** Reconnect attempts in a row without any applied event before giving up. */
  maxIdleReconnects?: number;
  backoffMs?: (attempt: number) => number;
  sleep?: (ms: number, signal: AbortSignal) => Promise<void>;
}

const TERMINAL: EpisodeStatus[] = ["completed", "failed", "cancelled"];

function defaultSleep(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve) => {
    if (signal.aborted || ms <= 0) return resolve();
    const t = setTimeout(resolve, ms);
    signal.addEventListener("abort", () => (clearTimeout(t), resolve()), { once: true });
  });
}

async function readEnvelope(res: Response): Promise<ErrorEnvelope> {
  try {
    const j = (await res.json()) as Partial<ErrorEnvelope>;
    if (j && typeof j.code === "string" && typeof j.message === "string" && j.retry) return j as ErrorEnvelope;
  } catch {
    /* fall through */
  }
  return { code: "upstream_unavailable", message: `HTTP ${res.status} without an ErrorEnvelope`, retry: { kind: "never" } } as ErrorEnvelope;
}

/** Follow the episode until it reaches a terminal status, the signal aborts, or reconnects stop progressing. */
export async function followEpisode(transport: StreamTransport, initial: EpisodeState, opt: FollowOptions): Promise<EpisodeState> {
  const { signal, onState } = opt;
  const notice = opt.onNotice ?? (() => {});
  const maxIdle = opt.maxIdleReconnects ?? 5;
  const backoff = opt.backoffMs ?? ((n: number) => Math.min(250 * 2 ** n, 5000));
  const sleep = opt.sleep ?? defaultSleep;
  let state = initial;
  let idle = 0;

  const publish = (s: EpisodeState) => {
    state = s;
    onState(s);
  };

  while (!signal.aborted) {
    if (state.status && TERMINAL.includes(state.status)) return state;
    if (idle > maxIdle) {
      notice({ kind: "gave_up", reason: `${idle} reconnects without progress` });
      return state;
    }
    if (idle > 0) await sleep(backoff(idle - 1), signal);
    if (signal.aborted) break;

    let res: Response;
    try {
      res = await transport.openEvents(state.episode_id, state.cursor, signal);
    } catch (err) {
      if (signal.aborted) break;
      notice({ kind: "error", error: { code: "upstream_unavailable", message: String(err), retry: { kind: "after_delay", seconds: 1 } } as ErrorEnvelope, http_status: null });
      idle++;
      continue;
    }

    if (res.status === 410) {
      // The log no longer covers the cursor: reload the snapshot and continue from last_seq.
      try {
        const ep = await transport.getEpisode(state.episode_id, signal);
        const before = state.cursor;
        const next = resyncFromSnapshot(state, ep);
        if (next.cursor > before) notice({ kind: "resync", from: before + 1, to: next.cursor });
        publish(next);
      } catch (err) {
        if (signal.aborted) break;
        notice({ kind: "error", error: { code: "conflict", message: `snapshot reload failed: ${String(err)}`, retry: { kind: "never" } } as ErrorEnvelope, http_status: 410 });
        idle++;
      }
      continue;
    }
    if (!res.ok || !res.body) {
      notice({ kind: "error", error: await readEnvelope(res), http_status: res.status });
      idle++;
      continue;
    }

    notice({ kind: "connected", after: state.cursor });
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    const parser = new SseParser();
    let progressed = false;
    let reconnect = false;
    try {
      while (!reconnect) {
        const { value, done } = await reader.read();
        if (done) break;
        for (const frame of parser.push(decoder.decode(value, { stream: true }))) {
          let ev: ResearchEvent;
          try {
            ev = JSON.parse(frame.data) as ResearchEvent;
          } catch {
            continue; // malformed frame: ignored, the next gap check catches lost data
          }
          const r = applyEvent(state, ev);
          if (r.outcome === "applied") {
            progressed = true;
            publish(r.state);
          } else if (r.outcome === "duplicate") {
            notice({ kind: "duplicate", seq: ev.seq });
          } else if (r.outcome === "gap") {
            notice({ kind: "gap", expected: state.cursor + 1, got: ev.seq });
            reconnect = true; // resume from our cursor; the server resends the missing range
            break;
          } else {
            notice({ kind: "foreign_event", episode_id: ev.episode_id });
          }
        }
      }
    } catch (err) {
      if (signal.aborted) break;
      notice({ kind: "error", error: { code: "upstream_unavailable", message: `stream interrupted: ${String(err)}`, retry: { kind: "after_delay", seconds: 1 } } as ErrorEnvelope, http_status: null });
    } finally {
      reader.cancel().catch(() => {});
    }
    if (!reconnect) notice({ kind: "ended", cursor: state.cursor });
    idle = progressed ? 0 : idle + 1;
  }
  return state;
}
