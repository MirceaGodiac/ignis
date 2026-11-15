// Plain text helpers for the quiet notes: an answer's limits and a declined step. Pure functions, so
// the shape a server message takes on screen is tested without a browser.
import type { ErrorCode } from "../contracts/generated";

/** A limit read as its first sentence (emphasised) and the rest (muted). */
export function leadSentence(text: string): { lead: string; rest: string } {
  const t = text.trim();
  // The first sentence end followed by a new sentence; "e.g." and "approx. 3" do not end one.
  const re = /[.!?](?=\s+["“(]?[A-Z0-9])/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(t))) {
    const end = m.index + 1;
    const word = t.slice(0, end).split(/\s/).pop() ?? "";
    if (/^(e\.g|i\.e|cf|vs|approx|ca|fig|eq|no|al)\.$/i.test(word)) continue;
    if (end < 12) continue;
    return { lead: t.slice(0, end), rest: t.slice(end).trim() };
  }
  return { lead: t, rest: "" };
}

export interface ListItem {
  /** The item's name (an id, a family), shown in mono. */
  name: string;
  /** Its parenthetical, without the parentheses, shown muted. */
  note: string | null;
}

export interface ListMessage {
  /** The sentences before the list, each capitalised, without the closing colon. */
  head: string[];
  items: ListItem[];
  /** Sentences after the list ("3 more are not readable ..."). */
  tail: string[];
}

/** Split `s` at `sep` where it is not inside parentheses or brackets. */
function splitTop(s: string, sep: string): string[] {
  const out: string[] = [];
  let depth = 0;
  let from = 0;
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if (c === "(" || c === "[") depth++;
    else if ((c === ")" || c === "]") && depth > 0) depth--;
    else if (depth === 0 && s.startsWith(sep, i)) {
      out.push(s.slice(from, i));
      from = i + sep.length;
      i += sep.length - 1;
    }
  }
  out.push(s.slice(from));
  return out;
}

const cap = (s: string) => (s ? s.charAt(0).toUpperCase() + s.slice(1) : s);
const clean = (s: string) => s.trim().replace(/[.;]+$/, "").trim();

/** One list segment: `name (note)` with a name that has no spaces, or null when it reads as a sentence. */
function asItem(seg: string): ListItem | null {
  const s = clean(seg);
  const m = /^(\S+)\s*\((.*)\)$/.exec(s);
  if (m) return { name: m[1], note: m[2].trim() || null };
  return /^\S+$/.test(s) ? { name: s, note: null } : null;
}

/**
 * A server message that names a list after a colon, items separated by semicolons
 * ("dataset x is not in this store; datasets readable in this session (...): a (...); b (...)"),
 * as a head, the items and any sentences after them. Null when the message has no such list:
 * then it is shown as it came.
 */
export function splitListMessage(message: string): ListMessage | null {
  const msg = message.trim();
  // The first colon outside parentheses that a list follows.
  let depth = 0;
  for (let i = 0; i < msg.length; i++) {
    const c = msg[i];
    if (c === "(" || c === "[") depth++;
    else if ((c === ")" || c === "]") && depth > 0) depth--;
    else if (c === ":" && depth === 0 && /\s/.test(msg[i + 1] ?? "")) {
      const segs = splitTop(msg.slice(i + 1), ";").map((x) => x.trim()).filter(Boolean);
      const items: ListItem[] = [];
      let k = 0;
      for (; k < segs.length; k++) {
        const it = asItem(segs[k]);
        if (!it) break;
        items.push(it);
      }
      const listed = items.length >= 2 || (items.length === 1 && items[0].note != null);
      if (!listed) continue;
      const head = splitTop(msg.slice(0, i), "; ").map(clean).filter(Boolean).map(cap);
      const tail = segs.slice(k).map(clean).filter(Boolean).map(cap);
      return { head, items, tail };
    }
  }
  return null;
}

// What a step's error means, in a reader's words; the tool's own message follows it.
const NOUN: Record<string, string> = {
  datasets_inspect: "dataset",
  sources_read: "source",
  sources_outline: "source",
  conditions_query: "run",
  models_forecast: "model",
};

export function refusalSummary(code: ErrorCode | string | null | undefined, tool?: string | null): string {
  const noun = tool ? NOUN[tool] : undefined;
  switch (code) {
    case "not_found":
      return noun ? `This ${noun} isn't available here` : "What this step asked for isn't available here";
    case "scope_denied":
      return noun ? `This session can't read that ${noun}` : "This session can't read that";
    case "invalid_request":
    case "invalid_input":
      return "The tool didn't accept the request as asked";
    case "unsupported":
      return "The tool doesn't support this request";
    case "capability_unavailable":
      return "This isn't set up on this server";
    case "conflict":
      return "The request conflicts with the current state";
    case "rate_limited":
      return "The service asked to slow down";
    case "upstream_unavailable":
      return "A service this step needs didn't respond";
    case "cancelled":
      return "This step was cancelled";
    case "source_missing":
      return "The source's file isn't available here";
    case "source_gone":
      return "The source is no longer available";
    case "internal":
      return "Something went wrong inside IGNIS";
    default:
      return "This step did not run";
  }
}
