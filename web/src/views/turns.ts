// Group the ordered event log into turns: a user message, then everything the agent did for it
// (the route), then its reply and answer. Derived only from EpisodeState; nothing is invented.
import type { ResearchEvent } from "../contracts/generated";
import type { AnswerState, EpisodeState, MessageState, ToolCallState } from "../research/eventLog";

export type Stop =
  | { kind: "tool"; seq: number; at: string; call: ToolCallState }
  | { kind: "job"; seq: number; at: string; jobId: string; status: string; jobKind: string }
  | { kind: "approval"; seq: number; at: string; approvalId: string; action: string; decision: string | null }
  | { kind: "answer"; seq: number; at: string; answer: AnswerState };

export interface Turn {
  user: MessageState | null;
  stops: Stop[];
  replies: MessageState[];
  answers: AnswerState[];
  startedAt: string | null;
  endedAt: string | null;
}

export function toTurns(s: EpisodeState): Turn[] {
  const turns: Turn[] = [];
  let cur: Turn | null = null;
  const seenTools = new Set<string>();
  const seenMsgs = new Set<string>();
  const start = (user: MessageState | null, at: string | null): Turn => {
    const t: Turn = { user, stops: [], replies: [], answers: [], startedAt: at, endedAt: at };
    turns.push(t);
    return t;
  };
  const ensure = (e: ResearchEvent) => (cur ??= start(null, e.occurred_at));
  for (const e of s.events) {
    const b = e.body;
    if (b.kind === "message.user") {
      cur = start(s.messages[b.payload.message_id] ?? null, e.occurred_at);
      continue;
    }
    const t = ensure(e);
    t.endedAt = e.occurred_at;
    switch (b.kind) {
      case "tool.started":
      case "tool.finished": {
        const id = b.kind === "tool.started" ? b.payload.call_id : b.payload.result.call_id;
        if (!seenTools.has(id) && s.tools[id]) {
          seenTools.add(id);
          t.stops.push({ kind: "tool", seq: e.seq, at: e.occurred_at, call: s.tools[id] });
        }
        break;
      }
      case "job.updated": {
        const j = b.payload.job;
        const prev = t.stops.find((x) => x.kind === "job" && x.jobId === j.job_id);
        if (prev && prev.kind === "job") prev.status = j.status;
        else t.stops.push({ kind: "job", seq: e.seq, at: e.occurred_at, jobId: j.job_id, status: j.status, jobKind: j.kind });
        break;
      }
      case "approval.requested":
        t.stops.push({ kind: "approval", seq: e.seq, at: e.occurred_at, approvalId: b.payload.approval_id, action: b.payload.action, decision: s.approvals[b.payload.approval_id]?.decision ?? null });
        break;
      case "message.delta":
      case "message.completed": {
        const id = b.payload.message_id;
        if (!seenMsgs.has(id) && s.messages[id]) {
          seenMsgs.add(id);
          t.replies.push(s.messages[id]);
        }
        break;
      }
      case "answer.completed": {
        const a = s.answers.find((x) => x.answer_id === b.payload.answer_id);
        if (a) {
          t.answers.push(a);
          t.stops.push({ kind: "answer", seq: e.seq, at: e.occurred_at, answer: a });
        }
        break;
      }
      default:
        break;
    }
  }
  return turns;
}

/** Elapsed time between two RFC 3339 stamps, as m:ss. */
export function elapsed(a: string | null, b: string | null): string | null {
  if (!a || !b) return null;
  const s = Math.max(0, Math.round((Date.parse(b) - Date.parse(a)) / 1000));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

export const TOOL_LABEL: Record<string, string> = {
  sources_search: "Searched the sources",
  sources_read: "Read a source",
  datasets_inspect: "Inspected a dataset",
  models_inspect: "Inspected a model release",
  models_forecast: "Ran a forecast",
  artifacts_read: "Read an artifact",
  jobs_get: "Checked a job",
  wiki_patch: "Wrote to the wiki",
};
