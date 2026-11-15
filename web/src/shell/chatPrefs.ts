// Per-viewer chat list preferences: a local name, pins and questions removed from the list.
// The server has no rename or delete for an episode (its event log is the audit record), so these
// live in localStorage as a convenience of this browser only. The list renders correctly without them.
import { useEffect, useState } from "react";
import type { ResearchEpisode } from "../contracts/generated";

export type ChatPrefs = { titles: Record<string, string>; pinned: string[]; hidden: string[] };

const KEY = "ignis.chats";
const EVENT = "ignis:chats";
const EMPTY: ChatPrefs = { titles: {}, pinned: [], hidden: [] };
export const TITLE_MAX = 120;

const strings = (v: unknown) => (Array.isArray(v) ? v.filter((x): x is string => typeof x === "string") : []);

export function readChatPrefs(): ChatPrefs {
  try {
    const raw = JSON.parse(localStorage.getItem(KEY) ?? "null") as Partial<ChatPrefs> | null;
    if (!raw || typeof raw !== "object") return EMPTY;
    const titles: Record<string, string> = {};
    for (const [k, v] of Object.entries(raw.titles ?? {})) if (typeof v === "string" && v.trim()) titles[k] = v.slice(0, TITLE_MAX);
    return { titles, pinned: strings(raw.pinned), hidden: strings(raw.hidden) };
  } catch {
    return EMPTY;
  }
}

function write(next: ChatPrefs) {
  try {
    localStorage.setItem(KEY, JSON.stringify(next));
  } catch {
    /* storage blocked: the change lasts for this page only */
  }
  window.dispatchEvent(new CustomEvent<ChatPrefs>(EVENT, { detail: next }));
}

const toggle = (xs: string[], id: string, on: boolean) => (on ? (xs.includes(id) ? xs : [id, ...xs]) : xs.filter((x) => x !== id));

export function rename(prefs: ChatPrefs, id: string, title: string): ChatPrefs {
  const titles = { ...prefs.titles };
  const t = title.trim().slice(0, TITLE_MAX);
  if (t) titles[id] = t;
  else delete titles[id];
  return { ...prefs, titles };
}
export const setPinned = (prefs: ChatPrefs, id: string, on: boolean): ChatPrefs => ({ ...prefs, pinned: toggle(prefs.pinned, id, on) });
export const setHidden = (prefs: ChatPrefs, id: string, on: boolean): ChatPrefs => ({
  ...prefs,
  hidden: toggle(prefs.hidden, id, on),
  pinned: on ? prefs.pinned.filter((x) => x !== id) : prefs.pinned,
});

/** The name a question shows: the viewer's own name for it, else its goal. */
export const chatTitle = (prefs: ChatPrefs, e: Pick<ResearchEpisode, "episode_id" | "goal">) => prefs.titles[e.episode_id] ?? e.goal;

export function useChatPrefs(): [ChatPrefs, (f: (p: ChatPrefs) => ChatPrefs) => void] {
  const [prefs, setPrefs] = useState<ChatPrefs>(readChatPrefs);
  useEffect(() => {
    const local = (e: Event) => setPrefs((e as CustomEvent<ChatPrefs>).detail);
    const other = (e: StorageEvent) => e.key === KEY && setPrefs(readChatPrefs());
    window.addEventListener(EVENT, local);
    window.addEventListener("storage", other);
    return () => {
      window.removeEventListener(EVENT, local);
      window.removeEventListener("storage", other);
    };
  }, []);
  return [prefs, (f) => write(f(readChatPrefs()))];
}

export type ChatGroup = { label: string; items: ResearchEpisode[] };

/** Pinned first, then by last activity: Today, Yesterday, Previous 7 days, Previous 30 days, Older. */
export function groupChats(episodes: ResearchEpisode[], prefs: ChatPrefs, now = new Date()): ChatGroup[] {
  const visible = episodes.filter((e) => !prefs.hidden.includes(e.episode_id));
  const at = (e: ResearchEpisode) => Date.parse(e.updated_at || e.created_at) || 0;
  const sorted = [...visible].sort((a, b) => at(b) - at(a));
  const day = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const D = 86_400_000;
  const buckets: [string, (t: number) => boolean][] = [
    ["Today", (t) => t >= day],
    ["Yesterday", (t) => t >= day - D],
    ["Previous 7 days", (t) => t >= day - 7 * D],
    ["Previous 30 days", (t) => t >= day - 30 * D],
    ["Older", () => true],
  ];
  const pinned = prefs.pinned.map((id) => sorted.find((e) => e.episode_id === id)).filter((e): e is ResearchEpisode => !!e);
  const groups: ChatGroup[] = pinned.length ? [{ label: "Pinned", items: pinned }] : [];
  const rest = sorted.filter((e) => !prefs.pinned.includes(e.episode_id));
  for (const [label, fits] of buckets) {
    const items = rest.filter((e) => fits(at(e)) && !groups.some((g) => g.label !== "Pinned" && g.items.includes(e)));
    if (items.length) groups.push({ label, items });
  }
  return groups;
}
