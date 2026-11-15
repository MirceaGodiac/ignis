// The 19 flight investigations as the atlas describes them (SpaceApps-Ardealu docs/wsp/atlas, exported
// by tools/wsp/atlas/export_web.py): names, hardware, typed fire-safety needs with NASA's words, and
// the narrative (written by Claude Opus 5.5 from NASA's documents; each finding names its source).
// Numbers about conditions never come from here: they come from the server's conditions table.
import data from "./investigations.json";

export interface Need {
  need: "prevention" | "detection" | "suppression" | string;
  tier: string | null;
  basis: string | null;
  inference: string | null;
}
export interface Sourced {
  file?: string;
  page?: number;
}
export interface Investigation {
  id: string;
  acronym: string | null;
  title: string | null;
  sub_area: string | null;
  hardware: string | null;
  period: { start?: string; end?: string } | null;
  objective: string | null;
  fuels: string[];
  needs: Need[];
  gravity: string | null;
  how_tests_differ: string | null;
  relevance: string | null;
  findings: { statement: string; kind: string | null; source: Sourced | null }[];
  sensors: { instrument: string | null; measures: string | null; source: Sourced | null }[];
  not_found: string[];
  related: { id: string; acronym: string | null; shared: string[] }[];
  written_by: string | null;
}

export const INVESTIGATIONS = new Map<string, Investigation>((data.investigations as Investigation[]).map((i) => [i.id, i]));

export const invName = (id: string) => INVESTIGATIONS.get(id)?.acronym ?? id;

/** Fixed colour slot per investigation (validated palette, styles.css --cat-1..5); the rest are neutral. */
const SLOT: Record<string, number> = { "PSI-69": 1, "PSI-10": 2, "PSI-159": 3, "PSI-39": 4, "PSI-68": 5 };
export const invColor = (id: string) => (SLOT[id] ? `var(--cat-${SLOT[id]})` : "var(--ink-3)");

/** Marker shape by hardware, the secondary encoding to colour: droplets, burner flames, solid samples, other. */
export type Shape = "circle" | "triangle" | "square" | "diamond";
const SHAPE: Record<string, Shape> = {
  "PSI-69": "circle", "PSI-68": "circle", "PSI-39": "circle",
  "PSI-10": "triangle", "PSI-159": "triangle", "PSI-20": "triangle", "PSI-21": "triangle", "PSI-22": "triangle", "PSI-23": "triangle",
  "PSI-25": "square", "PSI-26": "square", "PSI-98": "square", "PSI-99": "square", "PSI-100": "square",
};
export const invShape = (id: string): Shape => SHAPE[id] ?? "diamond";

/** The first sentence of a text, for a card. */
export const firstSentence = (t: string | null) => (t ? (t.match(/^.*?[.!?](\s|$)/)?.[0] ?? t).trim() : null);
