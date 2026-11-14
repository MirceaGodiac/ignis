// Client-side episode state, derived only from the ordered ResearchEvent log (CONTRACTS.md §Streaming).
// Mirrors the invariants of `check_event_log` in crates/ignis-core/src/contracts/research.rs:
// one episode, gap-free `seq`. Deduplication is by `(episode_id, seq)`: anything at or below the
// applied cursor is a replay and is dropped; anything past `cursor + 1` is a gap and is NOT applied
// (the caller resyncs). Pure module: type-only imports (tested with node --test).

import type {
  ApprovalDecision,
  ArtifactRef,
  AnswerStatus,
  EpisodeStatus,
  JobRef,
  ResearchEpisode,
  ResearchEvent,
  ToolEffect,
  ToolResult,
} from "../contracts/generated";

export interface MessageState {
  message_id: string;
  role: "user" | "assistant";
  text: string;
  /** true once `message.completed` (or `message.user`) gave the snapshot text. */
  complete: boolean;
}

export interface ToolCallState {
  call_id: string;
  tool: string;
  started_seq: number | null;
  result: ToolResult | null;
}

export interface ApprovalState {
  approval_id: string;
  action: string;
  effect: ToolEffect;
  decision: ApprovalDecision | null;
}

export interface AnswerState {
  answer_id: string;
  status: AnswerStatus;
  artifact: ArtifactRef;
}

/** Seq ranges the client never saw (log no longer covered the cursor: 410 + snapshot reload). */
export interface MissingRange {
  from: number;
  to: number;
}

export interface EpisodeState {
  episode_id: string;
  /** Highest applied seq (0 = nothing applied). */
  cursor: number;
  status: EpisodeStatus | null;
  /** Applied events in seq order (the timeline). */
  events: ResearchEvent[];
  messages: Record<string, MessageState>;
  messageOrder: string[];
  tools: Record<string, ToolCallState>;
  toolOrder: string[];
  jobs: Record<string, JobRef>;
  artifacts: Record<string, ArtifactRef>;
  approvals: Record<string, ApprovalState>;
  answers: AnswerState[];
  missing: MissingRange[];
}

export type ApplyOutcome = "applied" | "duplicate" | "gap" | "foreign_episode";

export function emptyState(episode_id: string): EpisodeState {
  return {
    episode_id,
    cursor: 0,
    status: null,
    events: [],
    messages: {},
    messageOrder: [],
    tools: {},
    toolOrder: [],
    jobs: {},
    artifacts: {},
    approvals: {},
    answers: [],
    missing: [],
  };
}

export function classify(s: EpisodeState, e: ResearchEvent): ApplyOutcome {
  if (e.episode_id !== s.episode_id) return "foreign_episode";
  if (e.seq <= s.cursor) return "duplicate";
  if (e.seq !== s.cursor + 1) return "gap";
  return "applied";
}

function upsertMessage(s: EpisodeState, id: string, role: MessageState["role"], f: (m: MessageState) => MessageState) {
  const prev = s.messages[id] ?? { message_id: id, role, text: "", complete: false };
  s.messages = { ...s.messages, [id]: f(prev) };
  if (!s.messageOrder.includes(id)) s.messageOrder = [...s.messageOrder, id];
}

function addArtifact(s: EpisodeState, a: ArtifactRef) {
  s.artifacts = { ...s.artifacts, [a.artifact_id]: a };
}

/** Apply one event. Returns the same state object when the event is not applied. */
export function applyEvent(state: EpisodeState, e: ResearchEvent): { state: EpisodeState; outcome: ApplyOutcome } {
  const outcome = classify(state, e);
  if (outcome !== "applied") return { state, outcome };
  const s: EpisodeState = { ...state, cursor: e.seq, events: [...state.events, e] };
  const body = e.body;
  switch (body.kind) {
    case "episode.status_changed":
      s.status = body.payload.to;
      break;
    case "message.user":
      upsertMessage(s, body.payload.message_id, "user", (m) => ({ ...m, text: body.payload.text, complete: true }));
      break;
    case "message.delta":
      // Progressive display only; a completed message never grows again.
      upsertMessage(s, body.payload.message_id, "assistant", (m) => (m.complete ? m : { ...m, text: m.text + body.payload.text }));
      break;
    case "message.completed":
      // The snapshot replaces whatever the deltas built.
      upsertMessage(s, body.payload.message_id, "assistant", (m) => ({ ...m, text: body.payload.text, complete: true }));
      break;
    case "tool.started": {
      const { call_id, tool } = body.payload;
      s.tools = { ...s.tools, [call_id]: { call_id, tool, started_seq: e.seq, result: s.tools[call_id]?.result ?? null } };
      if (!s.toolOrder.includes(call_id)) s.toolOrder = [...s.toolOrder, call_id];
      break;
    }
    case "tool.finished": {
      const r = body.payload.result;
      const prev = s.tools[r.call_id];
      s.tools = { ...s.tools, [r.call_id]: { call_id: r.call_id, tool: r.tool, started_seq: prev?.started_seq ?? null, result: r } };
      if (!s.toolOrder.includes(r.call_id)) s.toolOrder = [...s.toolOrder, r.call_id];
      if (r.job) s.jobs = { ...s.jobs, [r.job.job_id]: r.job };
      for (const a of r.artifact_refs ?? []) addArtifact(s, a);
      break;
    }
    case "job.updated":
      s.jobs = { ...s.jobs, [body.payload.job.job_id]: body.payload.job };
      for (const a of body.payload.job.outputs ?? []) addArtifact(s, a);
      break;
    case "artifact.created":
      addArtifact(s, body.payload.artifact);
      break;
    case "approval.requested": {
      const p = body.payload;
      s.approvals = { ...s.approvals, [p.approval_id]: { approval_id: p.approval_id, action: p.action, effect: p.effect, decision: null } };
      break;
    }
    case "approval.resolved": {
      const p = body.payload;
      const prev = s.approvals[p.approval_id];
      // A resolution for an unseen request still records the decision.
      s.approvals = {
        ...s.approvals,
        [p.approval_id]: { approval_id: p.approval_id, action: prev?.action ?? "(request not seen)", effect: prev?.effect ?? "read", decision: p.decision },
      };
      break;
    }
    case "answer.completed":
      addArtifact(s, body.payload.answer);
      s.answers = [...s.answers, { answer_id: body.payload.answer_id, status: body.payload.status, artifact: body.payload.answer }];
      break;
    case "step.rationale":
      // research.v2: the model's own step explanation. Kept in the timeline; no derived state yet.
      break;
    case "runtime.opaque":
      // Archived, not interpreted: it stays in the timeline only.
      if (body.payload.payload) addArtifact(s, body.payload.payload);
      break;
    default: {
      // A new EventBody kind must be handled above; this fails the typecheck until it is.
      const unknown: never = body;
      void unknown;
    }
  }
  return { state: s, outcome };
}

/**
 * The log no longer covers our cursor (410): continue from the episode snapshot. Derived state up
 * to our cursor is kept; the range we never saw is recorded, not invented.
 */
export function resyncFromSnapshot(state: EpisodeState, ep: ResearchEpisode): EpisodeState {
  if (ep.episode_id !== state.episode_id) throw new Error(`snapshot of ${ep.episode_id} for episode ${state.episode_id}`);
  if (ep.last_seq <= state.cursor) return { ...state, status: ep.status };
  return {
    ...state,
    status: ep.status,
    cursor: ep.last_seq,
    missing: [...state.missing, { from: state.cursor + 1, to: ep.last_seq }],
  };
}

/** Fold a whole log (tests and replays). Stops at the first gap or foreign event. */
export function reduceLog(episode_id: string, events: ResearchEvent[]): { state: EpisodeState; stoppedAt: ResearchEvent | null } {
  let s = emptyState(episode_id);
  for (const e of events) {
    const r = applyEvent(s, e);
    if (r.outcome === "gap" || r.outcome === "foreign_episode") return { state: s, stoppedAt: e };
    s = r.state;
  }
  return { state: s, stoppedAt: null };
}
