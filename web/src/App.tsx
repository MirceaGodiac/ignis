// IGNIS shell: sidebar (where), top bar (where am I), centre (what), side panel (why, from where).
import { useEffect, useState } from "react";
import { CompareProvider, useCompare } from "./shell/compare";
import { CommandMenu } from "./shell/CommandMenu";
import { DataProvider, useData } from "./shell/data";
import { runLabel } from "./views/RunPicker";
import { PanelProvider, usePanel } from "./shell/inspector";
import { Panel } from "./shell/Panel";
import { go, href, useRoute, VIEW_SECTION, VIEW_TITLE } from "./shell/router";
import { Original } from "./views/Original";
import { Sidebar } from "./shell/Sidebar";
import { IconChevron } from "./shell/icons";
import { useEpisodes } from "./shell/queries";
import { Compare } from "./views/Compare";
import { Home } from "./views/Home";
import { Lab } from "./views/Lab";
import { ConditionsPage } from "./views/Conditions";
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

  // Old links: the atmosphere map is a tab of Conditions; the conditions runs table is the Runs page.
  useEffect(() => {
    if (route.view === "map") go("conditions", null, { tab: "map" });
    else if (route.view === "conditions" && route.params.get("tab") === "runs") go("runs", null, { habitat: route.params.get("habitat") ?? undefined });
  }, [route.view, route.params]);

  // A new view starts with a closed side panel; the panel belongs to what was selected on it.
  const { experiments } = useData();
  const { close } = panel;
  useEffect(() => close(), [route.view, route.id, close]);
  // The main column is one scroller shared by every view: a new page starts at its top, so a question
  // opened from a scrolled page does not land with its first message under the sticky header.
  useEffect(() => {
    document.getElementById("main")?.scrollTo({ top: 0 });
  }, [route.view, route.id]);

  const goal = route.view === "research" && route.id ? episodes.data?.find((e) => e.episode_id === route.id)?.goal : null;
  useEffect(() => {
    document.title = `${goal ?? VIEW_TITLE[route.view]} · IGNIS`;
  }, [route.view, goal]);

  const labRun = route.view === "lab" && route.id ? experiments.data?.experiments.find((x) => x.id === route.id) : null;

  return (
    <div className={`shell${panel.current ? " has-panel" : ""}`}>
      <Sidebar route={route} compareCount={cmp.ids.length} onCommand={() => setCmd(true)} />
      <main className="main" id="main">
        {route.view !== "home" && !(route.view === "research" && route.id) && (
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
                  <span className="here">{labRun ? runLabel(labRun) : route.id}</span>
                </>
              ) : (
                <span className="here">{VIEW_TITLE[route.view]}</span>
              )}
            </nav>
            <span className="spacer" />
          </header>
        )}
        {route.view === "home" && <Home />}
        {route.view === "research" && <Research episodeId={route.id} newQuestion={route.params.get("q")} wantNew={route.params.get("new") === "1"} />}
        {route.view === "conditions" && <ConditionsPage habitat={route.params.get("habitat")} inv={route.params.get("inv")} tab={route.params.get("tab") === "map" ? "map" : "habitats"} />}
        {route.view === "runs" && <Runs key={route.params.toString()} initialFamily={route.params.get("family")} initialHabitat={route.params.get("habitat")} initialInv={route.params.get("inv")} />}
        {route.view === "compare" && <Compare />}
        {route.view === "lab" && <Lab runId={route.id} />}
        {route.view === "sources" && <Sources key={route.params.get("q") ?? ""} initial={route.params.get("q")} />}
        {route.view === "models" && <Models />}
        {route.view === "original" && <Original sourceId={route.id} params={route.params} />}
      </main>
      <Panel />
      <CommandMenu open={cmd} onClose={() => setCmd(false)} />
    </div>
  );
}
