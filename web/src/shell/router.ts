// Hash routes: every view is addressable, reloadable and shareable without a server route.
//   #/                          home (ask, recent work, shortcuts)
//   #/research[/<episode>]      questions (research episodes); ?tab=thread|evidence|route
//   #/runs                      run index
//   #/conditions[?tab=…]        habitats | map | runs, one page over the conditions table (T9)
//   #/map                       old link: opens #/conditions?tab=map
//   #/compare                   comparison           #/lab[/<run>]  forecast lab
//   #/sources[?q=…]             source search        #/models model releases
import { useEffect, useState } from "react";

export type View = "home" | "research" | "map" | "conditions" | "runs" | "compare" | "lab" | "sources" | "models";

export interface Route {
  view: View;
  /** Path segment after the view (episode id, run id). */
  id: string | null;
  params: URLSearchParams;
}

const VIEWS: View[] = ["home", "research", "map", "conditions", "runs", "compare", "lab", "sources", "models"];

export function parse(hash: string): Route {
  const raw = hash.replace(/^#\/?/, "");
  const [path, query = ""] = raw.split("?");
  const [head, ...rest] = path.split("/").filter(Boolean);
  const view = (VIEWS as string[]).includes(head ?? "") ? (head as View) : "home";
  return { view, id: rest.length ? decodeURIComponent(rest.join("/")) : null, params: new URLSearchParams(query) };
}

export function href(view: View, id?: string | null, params?: Record<string, string | undefined>): string {
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(params ?? {})) if (v) q.set(k, v);
  const qs = q.toString();
  const base = view === "home" ? "#/" : `#/${view}${id ? `/${encodeURIComponent(id)}` : ""}`;
  return qs ? `${base}?${qs}` : base;
}

export function go(view: View, id?: string | null, params?: Record<string, string | undefined>) {
  const next = href(view, id, params);
  if (window.location.hash !== next) window.location.hash = next;
}

export function useRoute(): Route {
  const [route, setRoute] = useState(() => parse(window.location.hash));
  useEffect(() => {
    const on = () => setRoute(parse(window.location.hash));
    window.addEventListener("hashchange", on);
    return () => window.removeEventListener("hashchange", on);
  }, []);
  return route;
}

/** Which service a page reads. A page whose service is not configured says so in place. */
export type Service = "agent_runtime" | "knowledge" | "datasets" | "models" | "registry";

/**
 * The information architecture, in one place: the rail, the breadcrumbs, the command menu and the
 * window title all read it. Sections follow what a researcher does: ask; look at what NASA recorded
 * (documents, runs, conditions); work with models on it.
 */
export const SECTIONS: { label: string | null; views: View[] }[] = [
  { label: null, views: ["home"] },
  { label: "NASA data", views: ["sources", "runs", "conditions"] },
  { label: "Models", views: ["lab", "compare", "models"] },
];

export const VIEW_SECTION: Partial<Record<View, string>> = Object.fromEntries(SECTIONS.flatMap((s) => (s.label ? s.views.map((v) => [v, s.label]) : [])));

export const VIEW_SERVICE: Record<View, Service | null> = {
  home: null,
  research: "agent_runtime",
  map: "datasets",
  conditions: "datasets",
  runs: "registry",
  compare: "registry",
  lab: "registry",
  sources: "knowledge",
  models: "models",
};

export const VIEW_TITLE: Record<View, string> = {
  home: "Home",
  research: "Questions",
  map: "Atmosphere map",
  conditions: "Conditions",
  runs: "Runs",
  compare: "Compare",
  lab: "Forecast lab",
  sources: "Sources",
  models: "Models",
};
