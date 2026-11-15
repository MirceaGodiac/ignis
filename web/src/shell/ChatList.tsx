// The recent questions in the left rail, grouped by last activity, each with a "…" menu:
// rename, pin, copy link, remove from list. Names, pins and removals are this browser's own
// (chatPrefs.ts); a removal can be undone for a few seconds.
import { useEffect, useRef, useState } from "react";
import type { EpisodeStatus, ResearchEpisode } from "../contracts/generated";
import { chatTitle, groupChats, rename, setHidden, setPinned, TITLE_MAX, useChatPrefs } from "./chatPrefs";
import { IconMore, IconThread } from "./icons";
import { href, type Route } from "./router";

const ACTIVE: EpisodeStatus[] = ["ready", "running", "waiting_tools", "waiting_approval", "waiting_user"];
const dotFor = (s: EpisodeStatus) => (ACTIVE.includes(s) ? (s.startsWith("waiting") ? "dot dot-bad" : "dot dot-run") : s === "completed" ? "dot dot-ok" : "dot dot-off");
const LIMIT = 30;
const UNDO_MS = 6000;

export function ChatList({ episodes, route, onNavigate }: { episodes: ResearchEpisode[]; route: Route; onNavigate: () => void }) {
  const [prefs, update] = useChatPrefs();
  const [editing, setEditing] = useState<string | null>(null);
  const [menu, setMenu] = useState<string | null>(null);
  const [removed, setRemoved] = useState<string | null>(null);

  useEffect(() => {
    if (!removed) return;
    const t = window.setTimeout(() => setRemoved(null), UNDO_MS);
    return () => window.clearTimeout(t);
  }, [removed]);

  let left = LIMIT;
  const groups = groupChats(episodes, prefs)
    .map((g) => {
      const items = g.label === "Pinned" ? g.items : g.items.slice(0, Math.max(0, left));
      if (g.label !== "Pinned") left -= items.length;
      return { ...g, items };
    })
    .filter((g) => g.items.length);
  const shown = groups.reduce((n, g) => n + g.items.length, 0);
  const total = episodes.filter((e) => !prefs.hidden.includes(e.episode_id)).length;

  return (
    <div className="side-section chats">
      <div className="side-label">
        <a href={href("research")} onClick={onNavigate}>
          Recent questions
        </a>
      </div>
      {groups.map((g) => (
        <div key={g.label} className="chat-group">
          <div className="chat-group-label">{g.label}</div>
          <div className="tree">
            {g.items.map((e) => (
              <ChatRow
                key={e.episode_id}
                episode={e}
                title={chatTitle(prefs, e)}
                current={route.view === "research" && route.id === e.episode_id}
                pinned={prefs.pinned.includes(e.episode_id)}
                renamed={e.episode_id in prefs.titles}
                editing={editing === e.episode_id}
                menuOpen={menu === e.episode_id}
                onMenu={(open) => setMenu(open ? e.episode_id : null)}
                onEdit={(on) => setEditing(on ? e.episode_id : null)}
                onRename={(t) => update((p) => rename(p, e.episode_id, t))}
                onPin={(on) => update((p) => setPinned(p, e.episode_id, on))}
                onRemove={() => {
                  update((p) => setHidden(p, e.episode_id, true));
                  setRemoved(e.episode_id);
                }}
                onNavigate={onNavigate}
              />
            ))}
          </div>
        </div>
      ))}
      {total > shown && (
        <a className="nav-item chat-more" href={href("research")} onClick={onNavigate}>
          <span className="grow">All {total} questions</span>
        </a>
      )}
      {removed && (
        <div className="chat-undo" role="status">
          <span className="grow">Removed from this list</span>
          <button
            className="btn btn-quiet"
            onClick={() => {
              update((p) => setHidden(p, removed, false));
              setRemoved(null);
            }}
          >
            Undo
          </button>
        </div>
      )}
      {prefs.hidden.length > 0 && !removed && (
        <button className="chat-restore" onClick={() => update((p) => ({ ...p, hidden: [] }))} title="Show every removed question in this list again">
          Show {prefs.hidden.length} removed
        </button>
      )}
    </div>
  );
}

type RowProps = {
  episode: ResearchEpisode;
  title: string;
  current: boolean;
  pinned: boolean;
  renamed: boolean;
  editing: boolean;
  menuOpen: boolean;
  onMenu: (open: boolean) => void;
  onEdit: (on: boolean) => void;
  onRename: (title: string) => void;
  onPin: (on: boolean) => void;
  onRemove: () => void;
  onNavigate: () => void;
};

function ChatRow({ episode: e, title, current, pinned, renamed, editing, menuOpen, onMenu, onEdit, onRename, onPin, onRemove, onNavigate }: RowProps) {
  const ref = useRef<HTMLDivElement>(null);
  const moreRef = useRef<HTMLButtonElement>(null);
  const [up, setUp] = useState(false);

  useEffect(() => {
    if (!menuOpen) return;
    const away = (ev: MouseEvent) => !ref.current?.contains(ev.target as Node) && onMenu(false);
    const key = (ev: KeyboardEvent) => {
      if (ev.key !== "Escape") return;
      onMenu(false);
      moreRef.current?.focus();
    };
    window.addEventListener("mousedown", away);
    window.addEventListener("keydown", key);
    // Open upward when the menu would pass the bottom of the scrolling rail.
    const m = ref.current?.querySelector<HTMLElement>(".chat-menu");
    const box = ref.current?.closest<HTMLElement>(".side-scroll") ?? document.documentElement;
    if (m) setUp(m.getBoundingClientRect().bottom > Math.min(box.getBoundingClientRect().bottom, window.innerHeight));
    ref.current?.querySelector<HTMLButtonElement>('[role="menuitem"]')?.focus();
    return () => {
      window.removeEventListener("mousedown", away);
      window.removeEventListener("keydown", key);
      setUp(false);
    };
  }, [menuOpen]); // eslint-disable-line react-hooks/exhaustive-deps

  const item = (label: string, act: () => void, cls?: string) => (
    <button
      role="menuitem"
      className={cls}
      onClick={() => {
        onMenu(false);
        act();
      }}
    >
      {label}
    </button>
  );

  const link = `${window.location.origin}${window.location.pathname}${href("research", e.episode_id)}`;

  return (
    <div className={`chat-row${menuOpen ? " menu-open" : ""}`} ref={ref}>
      {editing ? (
        <RenameField
          initial={title}
          onDone={(t) => {
            if (t !== null && t.trim() !== title) onRename(t);
            onEdit(false);
          }}
        />
      ) : (
        <a
          className="nav-item recent"
          href={href("research", e.episode_id)}
          aria-current={current ? "page" : undefined}
          title={renamed ? `${title}\n${e.goal}` : e.goal}
          onClick={onNavigate}
          onDoubleClick={(ev) => {
            ev.preventDefault();
            onEdit(true);
          }}
        >
          {ACTIVE.includes(e.status) ? <span className={dotFor(e.status)} aria-hidden="true" /> : <IconThread />}
          <span className="grow">{title}</span>
        </a>
      )}
      {!editing && (
        <button ref={moreRef} className="chat-more-btn" aria-label={`Options for ${title}`} aria-haspopup="menu" aria-expanded={menuOpen} onClick={() => onMenu(!menuOpen)}>
          <IconMore />
        </button>
      )}
      {menuOpen && (
        <div className={`menu chat-menu${up ? " up" : ""}`} role="menu" aria-label={`Options for ${title}`}>
          {item("Rename", () => onEdit(true))}
          {renamed && item("Use the original question", () => onRename(""))}
          {item(pinned ? "Unpin" : "Pin to top", () => onPin(!pinned))}
          {item("Copy link", () => void navigator.clipboard?.writeText(link))}
          <div className="menu-rule" role="separator" />
          {item("Remove from list", onRemove, "danger")}
        </div>
      )}
    </div>
  );
}

/** Inline rename: Enter or leaving the field saves, Escape cancels; an empty name restores the question. */
function RenameField({ initial, onDone }: { initial: string; onDone: (title: string | null) => void }) {
  const [v, setV] = useState(initial);
  const done = useRef(false);
  const finish = (t: string | null) => {
    if (done.current) return;
    done.current = true;
    onDone(t);
  };
  return (
    <input
      className="chat-rename"
      aria-label="Question name"
      value={v}
      maxLength={TITLE_MAX}
      autoFocus
      onFocus={(ev) => ev.currentTarget.select()}
      onChange={(ev) => setV(ev.target.value)}
      onBlur={() => finish(v)}
      onKeyDown={(ev) => {
        if (ev.key === "Enter") finish(v);
        if (ev.key === "Escape") finish(null);
      }}
    />
  );
}
