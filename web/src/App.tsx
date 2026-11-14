// IGNIS shell: sidebar (where), top bar (where am I), centre (what), side panel (why, from where).
import { useEffect, useState } from "react";
import { CompareProvider, useCompare } from "./shell/compare";
import { CommandMenu } from "./shell/CommandMenu";
import { DataProvider } from "./shell/data";
import { PanelProvider, usePanel } from "./shell/inspector";
import { Panel } from "./shell/Panel";
import { go, href, useRoute, VIEW_SECTION, VIEW_TITLE } from "./shell/router";
import { Sidebar } from "./shell/Sidebar";
import { IconChevron, IconSearch } from "./shell/icons";
import { useEpisodes } from "./shell/queries";
import { Compare } from "./views/Compare";
import { Home } from "./views/Home";
import { Lab } from "./views/Lab";
import { ConditionsPage, type ConditionsTab } from "./views/Conditions";
import { Models } from "./views/Models";
import { Research } from "./views/Research";
import { Runs } from "./views/Runs";
import { Sources } from "./views/Sources";

export function App() {
  return (
    <DataProvider>
      <CompareProvider>
        <PanelProvider>
          <Shell />
        </PanelProvider>
      </CompareProvider>
    </DataProvider>
  );
}

function Shell() {
  const route = useRoute();

  const panel = usePanel();
  const cmp = useCompare();
  const episodes = useEpisodes();
  const [cmd, setCmd] = useState(false);

  useEffect(() => {
    const on = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setCmd((o) => !o);
      }
    };
    window.addEventListener("keydown", on);
    return () => window.removeEventListener("keydown", on);
  }, []);

  // The old atmosphere-map link opens its tab on the conditions page.
  useEffect(() => {
    if (route.view === "map") go("conditions", null, { tab: "map" });
  }, [route.view]);

  // A new view starts with a closed side panel; the panel belongs to what was selected on it.
  const { close } = panel;
  useEffect(() => close(), [route.view, route.id, close]);

  const goal = route.view === "research" && route.id ? episodes.data?.find((e) => e.episode_id === route.id)?.goal : null;
  useEffect(() => {
    document.title = `${goal ?? VIEW_TITLE[route.view]} · IGNIS`;
  }, [route.view, goal]);

  return (
    <div className={`shell${panel.current ? " has-panel" : ""}`}>
      <Sidebar route={route} compareCount={cmp.ids.length} onCommand={() => setCmd(true)} />
      <main className="main" id="main">
        {route.view !== "home" && (
          <header className="topbar">
            <nav className="crumbs" aria-label="Breadcrumb">
              {VIEW_SECTION[route.view] && (
                <>
                  <span>{VIEW_SECTION[route.view]}</span>
                  <IconChevron width={12} height={12} />
                </>
              )}
              {route.view === "research" && route.id ? (
                <a href={href("research")} className="here">
                  Questions
                </a>
              ) : route.view === "lab" && route.id ? (
                <>
                  <a href={href("lab")}>Forecast lab</a>
                  <IconChevron width={12} height={12} />
                  <span className="here">{route.id}</span>
                </>
              ) : (
                <span className="here">{VIEW_TITLE[route.view]}</span>
              )}
            </nav>
            <span className="spacer" />
            <button className="search-btn" onClick={() => setCmd(true)}>
              <IconSearch width={14} height={14} /> <span className="lbl">Search</span> <kbd>Ctrl K</kbd>
            </button>
          </header>
        )}
        {route.view === "home" && <Home />}
        {route.view === "research" && <Research episodeId={route.id} newQuestion={route.params.get("q")} wantNew={route.params.get("new") === "1"} tab={route.params.get("tab")} />}
        {route.view === "conditions" && <ConditionsPage tab={(["map", "runs"].includes(route.params.get("tab") ?? "") ? route.params.get("tab") : "habitats") as ConditionsTab} />}
        {route.view === "runs" && <Runs initialFamily={route.params.get("family")} />}
        {route.view === "compare" && <Compare />}
        {route.view === "lab" && <Lab runId={route.id} />}
        {route.view === "sources" && <Sources key={route.params.get("q") ?? ""} initial={route.params.get("q")} />}
        {route.view === "models" && <Models />}
      </main>
      <Panel />
      <CommandMenu open={cmd} onClose={() => setCmd(false)} />
    </div>
  );
}
