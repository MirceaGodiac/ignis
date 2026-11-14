// Hash routes: every view is addressable, reloadable and shareable without a server route.
//   #/                          home (ask, recent work, shortcuts)
//   #/research[/<episode>]      investigations; ?tab=thread|evidence|route
//   #/map                       atmosphere map       #/runs   run index
//   #/compare                   comparison           #/lab[/<run>]  forecast lab
//   #/sources[?q=…]             source search        #/models model releases
import { useEffect, useState } from "react";

export type View = "home" | "research" | "map" | "runs" | "compare" | "lab" | "sources" | "models";

export interface Route {
  view: View;
  /** Path segment after the view (episode id, run id). */
  id: string | null;
  params: URLSearchParams;
}

const VIEWS: View[] = ["home", "research", "map", "runs", "compare", "lab", "sources", "models"];

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

export const VIEW_TITLE: Record<View, string> = {
  home: "Home",
  research: "Investigations",
  map: "Atmosphere map",
  runs: "Runs",
  compare: "Compare",
  lab: "Forecast lab",
  sources: "Sources",
  models: "Models",
};
