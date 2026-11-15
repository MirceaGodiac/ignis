// The served hand-written models, as a person reads them: a plain name, what each forecasts, who
// wrote it and how it scored. The card (`/model-cards`) carries the words; the release record
// (`listModels`) carries ids, construction and checks, which stay behind "Technical details".
import { useQuery } from "@tanstack/react-query";
import type { ModelCard, ModelRelease, ReportedScore } from "../contracts/generated";
import { useData } from "./data";

interface Envelope {
  code: string;
  message: string;
}

export class V1Error extends Error {
  constructor(public envelope: Envelope) {
    super(`${envelope.code}: ${envelope.message}`);
  }
}

export async function v1<T>(base: string, path: string, body?: unknown, signal?: AbortSignal): Promise<T> {
  const res = await fetch(base + path, {
    method: body === undefined ? "GET" : "POST",
    headers: body === undefined ? undefined : { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
    signal,
  });
  let j: unknown = null;
  try {
    j = await res.json();
  } catch {
    /* not JSON */
  }
  if (!res.ok && res.status !== 202) {
    const e = j as Partial<Envelope> | null;
    throw new V1Error(e && typeof e.code === "string" ? (e as Envelope) : { code: `http_${res.status}`, message: "the server answered without an error envelope" });
  }
  return j as T;
}

/** The served hand-written releases, and the v1 base they are served from (null when none is). */
export function useModelCards() {
  const { researchMode, caps } = useData();
  const base = researchMode === "live" ? "/api/v1" : researchMode === "fixtures" ? "/api/fixtures/v1" : null;
  // Only the live server says whether the model families are switched on; fixtures always serve theirs.
  const modelsCap = researchMode === "live" ? caps.data?.models : undefined;
  const cards = useQuery({
    queryKey: [researchMode, "model-cards"],
    queryFn: () => v1<ModelCard[]>(base!, "/model-cards"),
    enabled: !!base && modelsCap?.available !== false,
    retry: false,
    staleTime: 300_000,
  });
  return { base, modelsCap, cards };
}

/** The card of one release, when this server serves it. */
export function useServedCard(releaseId: string): ModelCard | null {
  const { cards } = useModelCards();
  return cards.data?.find((c) => c.release_id === releaseId) ?? null;
}

const ACRONYM = new Set(["flex", "acme", "bass", "same", "daft", "cld", "bre", "slice", "spice", "cfi", "sflame"]);

/** A plain name from a model name when no card names it: "flex_methanol.wsp2" → "FLEX methanol model". */
export function plainModelName(modelName: string): string {
  const stem = modelName.replace(/\.[a-z]+\d*$/i, "").replace(/\d+$/, "");
  const parts = stem.split(/[_\s-]+/).filter(Boolean);
  if (!parts.length) return modelName;
  const named = parts.map((p, i) => (ACRONYM.has(p.toLowerCase()) ? p.toUpperCase() : i === 0 ? p[0].toUpperCase() + p.slice(1) : p));
  return `${named.join(" ")} model`;
}

export const modelTitle = (modelName: string, card: ModelCard | null | undefined) => card?.title ?? plainModelName(modelName);

/** "anthropic · claude-fable-5-2" → "Claude Fable 5.2". */
export function authorName(author: { provider: string; model_id: string } | null | undefined): string | null {
  if (!author) return null;
  const m = /^claude-([a-z]+)-(\d+)(?:-(\d+))?/i.exec(author.model_id);
  if (m) return `Claude ${m[1][0].toUpperCase()}${m[1].slice(1)} ${m[2]}${m[3] ? `.${m[3]}` : ""}`;
  return author.model_id;
}

/** The selling line: who wrote the weights. */
export function writtenBy(release: ModelRelease | null | undefined): string {
  const who = authorName(release?.construction?.author ?? null);
  return who ? `Hand-written by ${who}` : "Hand-written";
}

const unbracket = (s: string) => s.replace(/\s*\([^)]*\)/g, "").replace(/^the one\s+/i, "").replace(/\s{2,}/g, " ").trim();

/** A score as value and what it is set against, first clause only: "40.0 µm" vs "best fitted model 77.7 µm". */
export function scoreParts(s: ReportedScore): { value: string; vs: string | null } {
  const against = /^(.+?)\s+against\s+(.+)$/.exec(s.value);
  if (against) return { value: against[1], vs: unbracket(against[2]) };
  const vs = s.comparison ? unbracket(s.comparison.split(/;\s|:\s/)[0]) : "";
  return { value: s.value, vs: vs && /\d/.test(vs) ? vs : null };
}

const WEAK = /in-sample|fails|worse|says little|loses/i;

/** The score a card leads with: the first one set against a baseline that the card does not itself discount. */
export function headlineScore(card: ModelCard | null | undefined): ReportedScore | null {
  if (!card) return null;
  return card.scores.find((s) => scoreParts(s).vs && !WEAK.test(s.comparison ?? "") && !WEAK.test(s.metric)) ?? null;
}

/** An input in plain words: its note up to the first clause, capitalised. */
export function inputWords(note: string, name: string): string {
  const head = note.split(/;\s|\s\(/)[0].trim() || name.replace(/_/g, " ");
  return head[0].toUpperCase() + head.slice(1);
}

/** Limits a reader needs: lines addressed to the agent (target settings, formulas) stay in the technical details. */
export const readerLimits = (card: ModelCard) => card.limits.filter((l) => !/settings\.target|^Targets|^All targets|^Probability/.test(l));

/** One paragraph on how the model was built, in plain words. */
export function builtParagraph(card: ModelCard | null, release: ModelRelease | null | undefined): string {
  const who = authorName(release?.construction?.author ?? null) ?? "An author model";
  const how = card?.authorship === "zero_shot" ? "before any outcome it is scored on was opened" : card ? "after reading the training data and the published theory" : null;
  return `${who} wrote every weight of this model by hand from the physics of the experiment${how ? `, ${how}` : ""}; nothing was fitted to data, and the model cannot change at forecast time.`;
}
