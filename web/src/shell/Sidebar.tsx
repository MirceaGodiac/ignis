// Left rail (Legora / Harvey): data-source switcher, new investigation and search, the workspace's
// tools, then the investigations; the open one expands into its Thread, Evidence and Route.
import { useEffect, useRef, useState } from "react";
import type { EpisodeStatus } from "../contracts/generated";
import { useData, type Source } from "./data";
import { href, type Route, type View } from "./router";
import { useEpisodes } from "./queries";
import { useThemeChoice, type ThemeChoice } from "./theme";
import {
  IconClose,
  IconMoon,
  IconSun,
  IconSystem,
  IconCompare,
  IconEvidence,
  IconHome,
  IconLab,
  IconMap,
  IconMatrix,
  IconMenu,
  IconModels,
  IconPlus,
  IconRoute,
  IconRuns,
  IconSearch,
  IconSources,
  IconThread,
  IconUpDown,
} from "./icons";

const TOOLS: { view: View; label: string; Icon: typeof IconMap }[] = [
  { view: "home", label: "Home", Icon: IconHome },
  { view: "runs", label: "Runs", Icon: IconRuns },
  { view: "map", label: "Atmosphere map", Icon: IconMap },
  { view: "conditions", label: "Conditions", Icon: IconMatrix },
  { view: "compare", label: "Compare", Icon: IconCompare },
  { view: "lab", label: "Forecast lab", Icon: IconLab },
  { view: "sources", label: "Sources", Icon: IconSources },
  { view: "models", label: "Models", Icon: IconModels },
];

const ACTIVE: EpisodeStatus[] = ["ready", "running", "waiting_tools", "waiting_approval", "waiting_user"];
const dotFor = (s: EpisodeStatus) => (ACTIVE.includes(s) ? (s.startsWith("waiting") ? "dot dot-bad" : "dot dot-run") : s === "completed" ? "dot dot-ok" : "dot dot-off");

const SOURCES: Record<Source, { name: string; note: string }> = {
  synthetic: { name: "Synthetic data", note: "Development fixtures. Not NASA data." },
  live: { name: "Live", note: "PSI registry and the research runtime." },
};

export function Sidebar({ route, compareCount, onCommand }: { route: Route; compareCount: number; onCommand: () => void }) {
  const { source, setSource, experiments, caps } = useData();
  const episodes = useEpisodes();
  const [open, setOpen] = useState(false);
  const [menu, setMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!menu) return;
    const on = (e: MouseEvent) => !menuRef.current?.contains(e.target as Node) && setMenu(false);
    window.addEventListener("mousedown", on);
    return () => window.removeEventListener("mousedown", on);
  }, [menu]);
  const close = () => setOpen(false);
  const tab = route.params.get("tab") ?? "thread";
  const services: [string, boolean | undefined, string | undefined][] = [
    ["Agent", caps.data?.agent_runtime.available, caps.data?.agent_runtime.detail],
    ["Knowledge", caps.data?.knowledge.available, caps.data?.knowledge.detail],
    ["Models", caps.data?.models.available, caps.data?.models.detail],
    ["Datasets", caps.data?.datasets.available, caps.data?.datasets.detail],
  ];

  return (
    <nav className={`sidebar${open ? " open" : ""}`} aria-label="Workspace">
      <div className="side-head">
        <div className="ws" ref={menuRef}>
          <button className="ws-btn" onClick={() => setMenu((m) => !m)} aria-haspopup="menu" aria-expanded={menu}>
            <span className="brand-mark" aria-hidden="true">
              <svg width="10" height="12" viewBox="0 0 10 12" fill="currentColor">
                <path d="M5 0c.6 2.2 4.5 4 4.5 7.5A4.5 4.5 0 0 1 .5 7.5C.5 5.8 1.6 4.6 2.6 3.8 2.6 5.4 3.4 6.3 4.2 6.6 3.6 4.2 4.4 1.8 5 0z" />
              </svg>
            </span>
            <span className="t">
              <b>IGNIS</b>
              <span>{SOURCES[source].name}</span>
            </span>
            <IconUpDown />
          </button>
          {menu && (
            <div className="ws-menu" role="menu">
              {(["synthetic", "live"] as Source[]).map((s) => (
                <button
                  key={s}
                  role="menuitemradio"
                  aria-checked={source === s}
                  onClick={() => {
                    setSource(s);
                    setMenu(false);
                  }}
                >
                  <span className={`dot ${source === s ? "dot-ok" : "dot-off"}`} style={{ marginTop: 5 }} aria-hidden="true" />
                  <b style={{ fontSize: 13, fontWeight: 550 }}>{SOURCES[s].name}</b>
                  <span>{SOURCES[s].note}</span>
                </button>
              ))}
            </div>
          )}
        </div>
        <button className="btn btn-quiet btn-icon only-narrow" aria-label={open ? "Close menu" : "Open menu"} aria-expanded={open} onClick={() => setOpen((o) => !o)}>
          {open ? <IconClose /> : <IconMenu />}
        </button>
        <div className="side-new">
          <a className="btn btn-primary" href={href("research", null, { new: "1" })} onClick={close}>
            <IconPlus /> New investigation
          </a>
          <button className="btn btn-icon" onClick={onCommand} aria-label="Search (Ctrl K)" title="Search  Ctrl K">
            <IconSearch />
          </button>
        </div>
      </div>

      <div className="side-scroll">
        <div className="tree">
          {TOOLS.map(({ view, label, Icon }) => (
            <a key={view} className="nav-item" href={href(view)} aria-current={route.view === view ? "page" : undefined} onClick={close}>
              <Icon />
              <span className="grow">{label}</span>
              {view === "runs" && experiments.data && <span className="count">{experiments.data.count}</span>}
              {view === "compare" && compareCount > 0 && <span className="count">{compareCount}</span>}
            </a>
          ))}
        </div>

        <div className="side-section">
          <div className="side-label">
            <span>Investigations</span>
            <a className="btn btn-quiet btn-icon btn-s" href={href("research")} aria-label="All investigations" title="All investigations" onClick={close}>
              <IconThread width={14} height={14} />
            </a>
          </div>
          <div className="tree">
            {episodes.isPending && <span className="muted small" style={{ padding: "4px 8px" }}>Loading…</span>}
            {episodes.error && <span className="muted small" style={{ padding: "4px 8px" }}>Not reachable</span>}
            {episodes.data?.length === 0 && <span className="muted small" style={{ padding: "4px 8px" }}>None yet</span>}
            {episodes.data?.map((e) => {
              const here = route.view === "research" && route.id === e.episode_id;
              return (
                <div key={e.episode_id}>
                  <a className="nav-item" href={href("research", e.episode_id)} aria-current={here && tab === "thread" ? "page" : undefined} title={e.goal} onClick={close}>
                    <span className={dotFor(e.status)} aria-hidden="true" style={{ margin: "0 3px" }} />
                    <span className="grow">{e.goal}</span>
                  </a>
                  {here && (
                    <div className="sub">
                      {(
                        [
                          ["thread", "Thread", IconThread],
                          ["evidence", "Evidence", IconEvidence],
                          ["route", "Route", IconRoute],
                        ] as const
                      ).map(([t, label, I]) => (
                        <a key={t} className="nav-item" href={href("research", e.episode_id, t === "thread" ? undefined : { tab: t })} aria-current={tab === t ? "page" : undefined} onClick={close}>
                          <I width={14} height={14} />
                          <span className="grow">{label}</span>
                        </a>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      </div>

      <div className="side-foot">
        <ThemeSwitch />
        <div className="services" aria-label="Services">
          {services.map(([name, ok, detail]) => (
            <span key={name} title={detail ?? "not loaded"}>
              <span className={`dot ${ok ? "dot-ok" : "dot-off"}`} aria-hidden="true" />
              {name}
              <span className="sr-only">{ok ? "available" : "not configured"}</span>
            </span>
          ))}
        </div>
      </div>
    </nav>
  );
}

const THEMES: [ThemeChoice, string, (p: { width?: number; height?: number }) => JSX.Element][] = [
  ["system", "System", IconSystem],
  ["light", "Light", IconSun],
  ["dark", "Dark", IconMoon],
];

function ThemeSwitch() {
  const [choice, setChoice] = useThemeChoice();
  return (
    <div className="seg theme-seg" role="group" aria-label="Theme">
      {THEMES.map(([c, label, Icon]) => (
        <button key={c} aria-pressed={choice === c} onClick={() => setChoice(c)} title={`${label} theme`}>
          <Icon width={14} height={14} />
          {label}
        </button>
      ))}
    </div>
  );
}
