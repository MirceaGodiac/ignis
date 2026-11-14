// Left rail: the wordmark, a new question and search, the pages grouped by what a researcher does
// (router SECTIONS), then the recent questions; the open one expands into Thread, Evidence and Route.
// A page whose service this deployment does not serve stays reachable and is drawn dimmed.
import { useState } from "react";
import type { EpisodeStatus } from "../contracts/generated";
import { useData } from "./data";
import { href, SECTIONS, VIEW_SERVICE, VIEW_TITLE, type Route, type View } from "./router";
import { useEpisodes } from "./queries";
import { useThemeChoice, type ThemeChoice } from "./theme";
import {
  IconClose,
  IconCompare,
  IconConditions,
  IconEvidence,
  IconHome,
  IconLab,
  IconMenu,
  IconModels,
  IconMoon,
  IconPlus,
  IconRoute,
  IconRuns,
  IconSearch,
  IconSources,
  IconSun,
  IconSystem,
  IconThread,
} from "./icons";

const ICON: Partial<Record<View, typeof IconHome>> = {
  home: IconHome,
  sources: IconSources,
  runs: IconRuns,
  conditions: IconConditions,
  lab: IconLab,
  compare: IconCompare,
  models: IconModels,
};

const ACTIVE: EpisodeStatus[] = ["ready", "running", "waiting_tools", "waiting_approval", "waiting_user"];
const dotFor = (s: EpisodeStatus) => (ACTIVE.includes(s) ? (s.startsWith("waiting") ? "dot dot-bad" : "dot dot-run") : s === "completed" ? "dot dot-ok" : "dot dot-off");

export function Sidebar({ route, compareCount, onCommand }: { route: Route; compareCount: number; onCommand: () => void }) {
  const { experiments, caps, health } = useData();
  const episodes = useEpisodes();
  const [open, setOpen] = useState(false);
  const close = () => setOpen(false);
  const tab = route.params.get("tab") ?? "thread";
  const here = (v: View) => route.view === v || (v === "conditions" && route.view === "map");
  // Unknown (still loading) counts as served; only a definite "not available" dims a page.
  const served = (v: View) => {
    const s = VIEW_SERVICE[v];
    if (!s) return true;
    if (s === "registry") return !(health.error && !health.data);
    return caps.data ? caps.data[s].available : true;
  };

  return (
    <nav className={`sidebar${open ? " open" : ""}`} aria-label="Workspace">
      <div className="side-head">
        <a className="wordmark" href={href("home")} onClick={close}>
          IGNIS
        </a>
        <button className="btn btn-quiet btn-icon only-narrow" aria-label={open ? "Close menu" : "Open menu"} aria-expanded={open} onClick={() => setOpen((o) => !o)}>
          {open ? <IconClose /> : <IconMenu />}
        </button>
        <div className="side-new">
          <a className="btn btn-primary" href={href("home")} onClick={close}>
            <IconPlus /> New question
          </a>
          <button className="btn btn-icon" onClick={onCommand} aria-label="Search (Ctrl K)" title="Search  Ctrl K">
            <IconSearch />
          </button>
        </div>
      </div>

      <div className="side-scroll">
        {SECTIONS.map((s) => (
          <div key={s.label ?? "top"} className={s.label ? "side-section" : undefined}>
            {s.label && <div className="side-label">{s.label}</div>}
            <div className="tree">
              {s.views.map((v) => {
                const Icon = ICON[v]!;
                const on = served(v);
                return (
                  <a key={v} className={`nav-item${on ? "" : " off"}`} href={href(v)} aria-current={here(v) ? "page" : undefined} title={on ? undefined : "Not available on this server"} onClick={close}>
                    <Icon />
                    <span className="grow">{VIEW_TITLE[v]}</span>
                    {v === "runs" && experiments.data && <span className="count">{experiments.data.count}</span>}
                    {v === "compare" && compareCount > 0 && <span className="count">{compareCount}</span>}
                  </a>
                );
              })}
            </div>
          </div>
        ))}

        {(episodes.data?.length ?? 0) > 0 && (
          <div className="side-section">
            <div className="side-label">
              <a href={href("research")} onClick={close}>
                Recent questions
              </a>
            </div>
            <div className="tree">
              {episodes.data!.map((e) => {
                const isHere = route.view === "research" && route.id === e.episode_id;
                return (
                  <div key={e.episode_id}>
                    <a className="nav-item recent" href={href("research", e.episode_id)} aria-current={isHere && tab === "thread" ? "page" : undefined} title={e.goal} onClick={close}>
                      <span className={dotFor(e.status)} aria-hidden="true" />
                      <span className="grow">{e.goal}</span>
                    </a>
                    {isHere && (
                      <div className="sub">
                        {(
                          [
                            ["thread", "Thread", IconThread],
                            ["evidence", "Evidence", IconEvidence],
                            ["route", "Route", IconRoute],
                          ] as const
                        ).map(([t, label, I]) => (
                          <a key={t} className="nav-item" href={href("research", e.episode_id, t === "thread" ? undefined : { tab: t })} aria-current={tab === t ? "page" : undefined} onClick={close}>
                            <I />
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
        )}
      </div>

      <div className="side-foot">
        <ThemeSwitch />
      </div>
    </nav>
  );
}

const THEMES: [ThemeChoice, string, typeof IconSun][] = [
  ["system", "System theme", IconSystem],
  ["light", "Light theme", IconSun],
  ["dark", "Dark theme", IconMoon],
];

function ThemeSwitch() {
  const [choice, setChoice] = useThemeChoice();
  return (
    <div className="seg theme-seg" role="group" aria-label="Theme">
      {THEMES.map(([c, label, Icon]) => (
        <button key={c} aria-pressed={choice === c} onClick={() => setChoice(c)} aria-label={label} title={label}>
          <Icon />
        </button>
      ))}
    </div>
  );
}
