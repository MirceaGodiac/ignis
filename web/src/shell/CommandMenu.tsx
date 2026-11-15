// Ctrl/⌘ K (Linear): every view, run, investigation and source switch, by name, from the keyboard.
import { useEffect, useMemo, useRef, useState } from "react";
import { useData } from "./data";
import { useEpisodes } from "./queries";
import { go, SECTIONS, VIEW_TITLE } from "./router";
import { usePanel } from "./inspector";

interface Cmd {
  id: string;
  group: string;
  label: string;
  hint?: string;
  run: () => void;
}

export function CommandMenu({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { experiments, setSource, source } = useData();
  const episodes = useEpisodes();
  const panel = usePanel();
  const [q, setQ] = useState("");
  const [sel, setSel] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLUListElement>(null);
  const returnTo = useRef<Element | null>(null);

  const cmds = useMemo<Cmd[]>(() => {
    const views = SECTIONS.flatMap((s) => s.views);
    const out: Cmd[] = [
      { id: "ask", group: "Actions", label: "New question", run: () => go("home") },
      { id: "src", group: "Actions", label: source === "synthetic" ? "Switch to live data" : "Switch to synthetic data", run: () => setSource(source === "synthetic" ? "live" : "synthetic") },
      ...views.map((v) => ({ id: `v:${v}`, group: "Go to", label: VIEW_TITLE[v], run: () => go(v) })),
      ...(episodes.data ?? []).map((e) => ({ id: `e:${e.episode_id}`, group: "Questions", label: e.goal, hint: e.status.replace(/_/g, " "), run: () => go("research", e.episode_id) })),
      ...(experiments.data?.experiments ?? []).map((x) => ({
        id: `r:${x.id}`,
        group: "Runs",
        label: x.id,
        hint: `${x.family.replace("_", " ")} · ${x.fuel_or_material ?? "material unknown"}`,
        run: () => panel.open({ kind: "run", id: x.id }),
      })),
    ];
    return out;
  }, [episodes.data, experiments.data, source, setSource, panel]);

  const shown = useMemo(() => {
    const t = q.trim().toLowerCase();
    const hits = t ? cmds.filter((c) => `${c.label} ${c.hint ?? ""} ${c.group}`.toLowerCase().includes(t)) : cmds.filter((c) => c.group !== "Runs");
    return hits.slice(0, 60);
  }, [q, cmds]);

  useEffect(() => {
    if (open) {
      returnTo.current = document.activeElement;
      setQ("");
      setSel(0);
      requestAnimationFrame(() => input.current?.focus());
    } else if (returnTo.current instanceof HTMLElement) {
      returnTo.current.focus();
    }
  }, [open]);
  useEffect(() => setSel(0), [q]);
  useEffect(() => {
    list.current?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" });
  }, [sel]);

  if (!open) return null;
  const pick = (c: Cmd | undefined) => {
    if (!c) return;
    onClose();
    c.run();
  };
  let lastGroup = "";
  return (
    <div className="cmd-back" onMouseDown={(e) => e.target === e.currentTarget && onClose()}>
      <div className="cmd" role="dialog" aria-modal="true" aria-label="Go to">
        <input
          ref={input}
          value={q}
          onChange={(e) => setQ(e.target.value)}
          placeholder="Search views, runs and investigations…"
          role="combobox"
          aria-expanded="true"
          aria-controls="cmd-list"
          aria-activedescendant={shown[sel] ? `cmd-${sel}` : undefined}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown") {
              e.preventDefault();
              setSel((s) => Math.min(s + 1, shown.length - 1));
            } else if (e.key === "ArrowUp") {
              e.preventDefault();
              setSel((s) => Math.max(s - 1, 0));
            } else if (e.key === "Enter") {
              e.preventDefault();
              pick(shown[sel]);
            } else if (e.key === "Escape") {
              e.preventDefault();
              onClose();
            }
          }}
        />
        <ul id="cmd-list" role="listbox" ref={list}>
          {shown.length === 0 && <li className="none">Nothing matches “{q}”.</li>}
          {shown.map((c, i) => {
            const head = c.group !== lastGroup;
            lastGroup = c.group;
            return (
              <li key={c.id}>
                {head && <div className="grp">{c.group}</div>}
                <div id={`cmd-${i}`} role="option" aria-selected={i === sel} className="opt" onMouseEnter={() => setSel(i)} onClick={() => pick(c)}>
                  <span>{c.label}</span>
                  {c.hint && <span className="hint">{c.hint}</span>}
                </div>
              </li>
            );
          })}
        </ul>
      </div>
    </div>
  );
}
