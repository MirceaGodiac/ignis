// Group the ordered event log into turns: a user message, then everything the agent did for it
// (the route), then its reply and answer. Derived only from EpisodeState; nothing is invented.
// The second half turns a recorded call into the plain sentence a researcher's notebook would
// hold ("Read the BRE report, page 80"), from the call's own arguments and result only.
import type { ReadRange, ResearchEvent, SearchHit } from "../contracts/generated";
import type { AnswerState, EpisodeState, MessageState, ToolCallState } from "../research/eventLog";

export type Stop =
  | { kind: "tool"; seq: number; at: string; call: ToolCallState; /** When its `tool.finished` arrived. */ end: string | null }
  | { kind: "job"; seq: number; at: string; jobId: string; status: string; jobKind: string; end: string | null }
  | { kind: "approval"; seq: number; at: string; approvalId: string; action: string; decision: string | null }
  | { kind: "answer"; seq: number; at: string; answer: AnswerState }
  /** A `step.rationale` that names no call: the model's own note on its route (K1.2). */
  | { kind: "note"; seq: number; at: string; text: string }
  /** An assistant message, at the point it began (narration between calls, drafts, the closing reply). */
  | { kind: "said"; seq: number; at: string; message: MessageState };

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
  const toolStop = new Map<string, Extract<Stop, { kind: "tool" }>>();
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
          const stop: Extract<Stop, { kind: "tool" }> = { kind: "tool", seq: e.seq, at: e.occurred_at, call: s.tools[id], end: null };
          toolStop.set(id, stop);
          t.stops.push(stop);
        }
        if (b.kind === "tool.finished") {
          const stop = toolStop.get(id);
          if (stop) stop.end = e.occurred_at;
        }
        break;
      }
      case "job.updated": {
        const j = b.payload.job;
        const prev = t.stops.find((x) => x.kind === "job" && x.jobId === j.job_id);
        const done = ["succeeded", "failed", "cancelled"].includes(j.status) ? e.occurred_at : null;
        if (prev && prev.kind === "job") {
          prev.status = j.status;
          prev.end = done;
        } else t.stops.push({ kind: "job", seq: e.seq, at: e.occurred_at, jobId: j.job_id, status: j.status, jobKind: j.kind, end: done });
        break;
      }
      case "step.rationale":
        if (!b.payload.call_id) t.stops.push({ kind: "note", seq: e.seq, at: e.occurred_at, text: b.payload.text });
        break;
      case "approval.requested":
        t.stops.push({ kind: "approval", seq: e.seq, at: e.occurred_at, approvalId: b.payload.approval_id, action: b.payload.action, decision: s.approvals[b.payload.approval_id]?.decision ?? null });
        break;
      case "message.delta":
      case "message.completed": {
        const id = b.payload.message_id;
        if (!seenMsgs.has(id) && s.messages[id]) {
          seenMsgs.add(id);
          t.replies.push(s.messages[id]);
          t.stops.push({ kind: "said", seq: e.seq, at: e.occurred_at, message: s.messages[id] });
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

/** A duration for people: "under 1 s", "8 s", "1 min 4 s", "2 h 5 min". Null when a stamp is missing. */
export function duration(a: string | null, b: string | null): string | null {
  if (!a || !b) return null;
  const ms = Date.parse(b) - Date.parse(a);
  if (!Number.isFinite(ms)) return null;
  return seconds(Math.max(0, Math.round(ms / 1000)));
}

/** Whole seconds for people: "under 1 s", "8 s", "1 min 4 s", "2 h 5 min". */
export function seconds(s: number): string {
  if (s < 1) return "under 1 s";
  if (s < 60) return `${s} s`;
  if (s < 3600) return `${Math.floor(s / 60)} min${s % 60 ? ` ${s % 60} s` : ""}`;
  return `${Math.floor(s / 3600)} h${Math.floor((s % 3600) / 60) ? ` ${Math.floor((s % 3600) / 60)} min` : ""}`;
}

export const TOOL_LABEL: Record<string, string> = {
  sources_search: "Searched the sources",
  answer_submit: "Recorded the answer",
  sources_outline: "Read the outline",
  conditions_query: "Looked up test conditions",
  graph_neighbors: "Followed the knowledge graph",
  wiki_read: "Read the wiki",
  web_search: "Searched the web",
  web_fetch: "Opened a web page",
  sources_read: "Read a source",
  datasets_inspect: "Inspected a dataset",
  models_inspect: "Inspected a model release",
  models_forecast: "Ran a model forecast",
  analysis_execute: "Ran an analysis",
  artifacts_read: "Read an earlier result",
  jobs_get: "Checked a job",
  wiki_patch: "Wrote to the wiki",
};

// ---------------------------------------------------------------- the notebook sentence of a step

/** What a step did, for its icon. */
export type StepKind = "search" | "read" | "data" | "model" | "compute" | "web" | "note" | "other";

export const TOOL_KIND: Record<string, StepKind> = {
  sources_search: "search",
  graph_neighbors: "search",
  sources_read: "read",
  sources_outline: "read",
  wiki_read: "read",
  artifacts_read: "read",
  conditions_query: "data",
  datasets_inspect: "data",
  models_inspect: "model",
  models_forecast: "model",
  analysis_execute: "compute",
  jobs_get: "compute",
  web_search: "web",
  web_fetch: "web",
  wiki_patch: "note",
};

/** Calls that only file what the agent already did: not research, so the trail folds them away
 * unless they did not succeed (a refused answer is part of the story). */
const BOOKKEEPING = new Set(["answer_submit", "jobs_get"]);

const obj = (v: unknown): Record<string, unknown> | null => (v && typeof v === "object" && !Array.isArray(v) ? (v as Record<string, unknown>) : null);
const str = (v: unknown): string | null => (typeof v === "string" && v.trim() ? v.trim() : null);
const clip = (s: string, n: number) => (s.length > n ? `${s.slice(0, n - 1).trimEnd()}…` : s);

/** The recorded arguments of a call, as an object (null when not recorded or not an object). */
export const callArgs = (call: ToolCallState) => obj(call.arguments?.value.value);
/** The inline result value of a call, unless it is only a pointer to a stored artifact. */
export function callValue(call: ToolCallState): Record<string, unknown> | null {
  const v = obj(call.result?.result?.value);
  return v && !("stored_as_artifact" in v) ? v : null;
}

/** The search query a call was given, when its recorded arguments carry one (K1.1). */
export function callQuery(call: ToolCallState): string | null {
  return str(callArgs(call)?.query);
}

/** "page 80", "pages 80–82", "the cited passage", "table T3". */
export function rangeText(r: unknown): string | null {
  const range = obj(r) as ReadRange | null;
  if (!range) return null;
  switch (range.kind) {
    case "pages":
      return range.first === range.last ? `page ${range.first}` : `pages ${range.first}–${range.last}`;
    case "passage":
      return "the cited passage";
    case "table":
      return `table ${range.table_id}`;
    default:
      return null;
  }
}

// Titles are only known where a search hit carried one; a source id is never shown as a title.
const TITLES = new Map<string, string>();

/** Remember the titles every inline search hit of this episode carried (source id → title). */
export function learnSourceTitles(s: EpisodeState): void {
  for (const id of s.toolOrder) {
    const hits = callValue(s.tools[id] ?? ({} as ToolCallState))?.hits;
    if (!Array.isArray(hits)) continue;
    for (const h of hits as SearchHit[]) {
      const t = str(h?.title);
      const sid = h?.evidence?.source_id;
      if (t && sid && !TITLES.has(sid)) TITLES.set(sid, t);
    }
  }
}
export const sourceTitle = (sourceId: string | null | undefined): string | null => (sourceId ? TITLES.get(sourceId) ?? null : null);

/** A count of hits or rows when the result carries one; null when it does not say. */
function countOf(v: Record<string, unknown> | null, key: string): number | null {
  if (!v) return null;
  const x = v[key];
  if (Array.isArray(x)) return typeof v.total_count === "number" && v.total_count >= x.length ? v.total_count : x.length;
  return typeof x === "number" ? x : null;
}

const plural = (n: number, one: string, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;

export interface StepText {
  kind: StepKind;
  /** The notebook sentence. */
  text: string;
  /** What came back, in a few words ("12 passages", "preview"); null when there is nothing to add. */
  detail: string | null;
  /** The source a read looked at, when known. */
  sourceId: string | null;
}

/** The plain sentence for one call, from its recorded arguments and result only. */
export function describeCall(call: ToolCallState, opts: { full?: boolean } = {}): StepText {
  const n = opts.full ? 400 : 72;
  const a = callArgs(call);
  const v = callValue(call);
  const r = call.result;
  const kind = TOOL_KIND[call.tool] ?? "other";
  const preview = r?.result?.partial ? "preview" : r?.execution_status === "accepted" ? "started as a job" : null;
  const join = (...xs: (string | null)[]) => xs.filter(Boolean).join(" · ") || null;
  switch (call.tool) {
    case "sources_search": {
      const q = str(a?.query);
      const hits = countOf(v, "hits");
      const degraded = v?.degraded === true || (Array.isArray(v?.backends) && (v!.backends as { status?: string }[]).some((b) => b?.status === "unavailable"));
      return {
        kind,
        text: q ? `Searched the NASA sources for “${clip(q, n)}”` : "Searched the NASA sources",
        detail: join(hits == null ? null : hits === 0 ? "no passages" : plural(hits, "passage"), degraded ? "degraded search" : null, preview),
        sourceId: null,
      };
    }
    case "sources_read":
    case "sources_outline": {
      const sid = str(a?.source_id);
      const title = sourceTitle(sid);
      const name = title ? clip(title, opts.full ? 200 : 56) : "a source";
      if (call.tool === "sources_outline") return { kind, text: `Looked at the outline of ${title ? name : "a source"}`, detail: preview, sourceId: sid };
      const where = rangeText(a?.range);
      const incomplete = v && v.complete === false ? "more remains" : null;
      return { kind, text: `Read ${name}${where ? `, ${where}` : ""}`, detail: join(incomplete, preview), sourceId: sid };
    }
    case "conditions_query": {
      const inv = Array.isArray(a?.investigation_ids) ? (a!.investigation_ids as unknown[]).filter((x): x is string => typeof x === "string") : [];
      const rows = countOf(v, "rows");
      return {
        kind,
        text: inv.length ? `Looked up the test conditions of ${clip(inv.join(", "), n)}` : "Looked up the recorded test conditions",
        detail: join(rows == null ? null : plural(rows, "run"), preview),
        sourceId: null,
      };
    }
    case "datasets_inspect": {
      const id = str(a?.dataset_id);
      return { kind, text: id ? `Inspected the dataset ${clip(id, 48)}` : "Inspected a dataset", detail: preview, sourceId: null };
    }
    case "models_forecast":
      return { kind, text: "Asked for a model forecast", detail: join("model output", preview), sourceId: null };
    case "analysis_execute": {
      const lang = str(a?.language);
      return { kind, text: lang ? `Ran an analysis in ${lang}` : "Ran an analysis", detail: preview, sourceId: null };
    }
    case "web_search": {
      const q = str(a?.query);
      return { kind, text: q ? `Searched the web for “${clip(q, n)}”` : "Searched the web", detail: preview, sourceId: null };
    }
    case "web_fetch": {
      const u = str(a?.url);
      let host: string | null = null;
      try {
        host = u ? new URL(u).hostname : null;
      } catch {
        host = null;
      }
      return { kind, text: host ? `Opened a page on ${host}` : "Opened a web page", detail: preview, sourceId: null };
    }
    default:
      return { kind, text: TOOL_LABEL[call.tool] ?? sentence(call.tool), detail: preview, sourceId: null };
  }
}

const sentence = (id: string) => {
  const w = id.replace(/_/g, " ");
  return w.charAt(0).toUpperCase() + w.slice(1);
};

/** How a step ended, only when it is not a plain success: what a reader should notice. */
export type StepState = "ok" | "running" | "failed" | "refused" | "cancelled" | "waiting";

export function stepState(s: Stop): StepState {
  switch (s.kind) {
    case "tool": {
      const r = s.call.result;
      // An accepted call handed its work to a job; the job's own stop carries how that went.
      if (!r) return "running";
      if (r.execution_status === "failed") return "failed";
      if (r.execution_status === "refused") return "refused";
      if (r.execution_status === "cancelled") return "cancelled";
      return "ok";
    }
    case "job":
      return s.status === "succeeded" ? "ok" : s.status === "failed" ? "failed" : s.status === "cancelled" ? "cancelled" : "running";
    case "approval":
      return s.decision === "approved" ? "ok" : s.decision ? "refused" : "waiting";
    default:
      return "ok";
  }
}

/** A step the trail leaves out: bookkeeping that succeeded. Its outcome shows in the end marker. */
export function isBookkeeping(s: Stop): boolean {
  return s.kind === "tool" && BOOKKEEPING.has(s.call.tool) && stepState(s) === "ok";
}

/** "Searched 3 times, read 2 sources, looked up conditions once". Null when there was no tool work. */
export function routeSummary(stops: Stop[]): string | null {
  const tried = stops.filter((s) => s.kind === "tool" && !isBookkeeping(s));
  if (!tried.length) return null;
  // Counted are the steps that ran; the trail counts the others apart ("1 step did not run").
  const calls = tried.flatMap((s) => (s.kind === "tool" && stepState(s) === "ok" ? [s.call] : []));
  if (!calls.length) return `Tried ${plural(tried.length, "step")}`;
  const by = (k: StepKind) => calls.filter((c) => (TOOL_KIND[c.tool] ?? "other") === k);
  const times = (n: number) => (n === 1 ? "once" : n === 2 ? "twice" : `${n} times`);
  const parts: string[] = [];
  const search = by("search").length + by("web").filter((c) => c.tool === "web_search").length;
  if (search) parts.push(`searched ${times(search)}`);
  const reads = by("read");
  if (reads.length) {
    const sources = new Set(reads.map((c) => str(callArgs(c)?.source_id) ?? c.call_id)).size;
    parts.push(reads.every((c) => c.tool === "sources_read" || c.tool === "sources_outline") ? `read ${plural(sources, "source")}` : `read ${times(reads.length)}`);
  }
  const data = by("data").length;
  if (data) parts.push(`looked up conditions ${times(data)}`);
  const fc = calls.filter((c) => c.tool === "models_forecast").length;
  if (fc) parts.push(`asked for ${plural(fc, "forecast")}`);
  const an = calls.filter((c) => c.tool === "analysis_execute").length;
  if (an) parts.push(`ran ${plural(an, "analysis", "analyses")}`);
  const rest = calls.length - search - reads.length - data - fc - an;
  if (rest > 0) parts.push(`${plural(rest, "other step")}`);
  const out = parts.join(", ");
  return out.charAt(0).toUpperCase() + out.slice(1);
}
