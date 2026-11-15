// How a search hit reads to a researcher: a human title from a PSI file stem, the query terms marked
// in its snippet, and the hits of one document gathered under one card. Pure, so it is tested.
import type { SearchHit } from "../contracts/generated";

export interface SourceName {
  /** "SPICE · Phase III Flight Safety Data Package": the document as one line (headers, tab titles). */
  label: string;
  /** The document itself: "Phase III Flight Safety Data Package", "Dissertation (revised)". */
  title: string;
  /** The investigation ("SPICE") or, when the stem names none, the PSI entry ("PSI-20"). */
  investigation: string | null;
}

/** PSI's folder names between the entry id and the document ("Reports", "Engineering Documents"). */
const CATEGORY = /^(?:(?:engineering|science|final|raw|processed|supporting|technical|other)\s+)?(?:documents?|reports?|publications?|papers?|presentations?|data|images?|videos?|files?|drawings?|theses|dissertations?)$/i;
/** Version words at the end of a stem, shown as a parenthetical. */
const QUALIFIER: Record<string, string> = { revised: "revised", rev: "revised", final: "final", draft: "draft", corrected: "corrected", updated: "updated" };
/** Release markers in PSI stems that carry nothing for a reader ("PD": public distribution). */
const NOISE = new Set(["PD"]);
const PSI_ID = /^PSI-\d+$/i;

const tidy = (s: string) => s.replace(/\s+/g, " ").trim();

/** A stem segment without spaces is hyphen-joined words: "Dissertation-PD-revised". Hyphens after
 * a single letter or between short parts ("n-heptane", "2-D", "O2-N2") stay. */
function words(seg: string): { text: string; qualifiers: string[] } {
  if (/\s/.test(seg) || !seg.includes("-")) return { text: tidy(seg), qualifiers: [] };
  const parts = seg.split("-").filter(Boolean);
  const kept: string[] = [];
  const qualifiers: string[] = [];
  parts.forEach((p, i) => {
    const q = Object.hasOwn(QUALIFIER, p.toLowerCase()) ? QUALIFIER[p.toLowerCase()] : undefined;
    if (i > 0 && q) qualifiers.push(q);
    else if (i > 0 && NOISE.has(p)) return;
    else if (kept.length && (/^\w$/.test(kept[kept.length - 1]!) || (kept[kept.length - 1]!.length <= 2 && p.length <= 2))) kept[kept.length - 1] += `-${p}`;
    else kept.push(p);
  });
  return { text: kept.join(" "), qualifiers };
}

/** An investigation name is short: an acronym or a couple of words ("SPICE", "Flame Design"). */
const looksLikeInvestigation = (s: string) => s.length <= 24 && s.split(" ").length <= 3 && s.split(" ").every((w) => /^[A-Z0-9]/.test(w));
const ACRONYM = /^[A-Z][A-Z0-9-]{1,9}$/;

/**
 * A human name for a source from its title or file stem.
 *   "PSI-20_Reports_Dissertation-PD-revised_"  -> PSI-20 · Dissertation (revised)
 *   "PSI-107_Engineering Documents_SPICE_Phase III Flight Safety Data Package"
 *                                               -> SPICE · Phase III Flight Safety Data Package
 * A title that is already prose (no underscores, no PSI id) is kept as it is.
 */
export function humanSourceName(raw: string | null | undefined): SourceName {
  const base = tidy((raw ?? "").replace(/^.*[\\/]/, "").replace(/\.(pdf|csv|txt|mp4|mov|avi|zip|xlsx?|docx?)$/i, ""));
  if (!base) return { label: "Untitled source", title: "Untitled source", investigation: null };
  const segs = base.split("_").map(tidy).filter(Boolean);
  if (segs.length <= 1 && !PSI_ID.test(segs[0]?.split(/[\s_]/)[0] ?? "")) return { label: base, title: base, investigation: null };
  let psi: string | null = null;
  if (segs[0] && PSI_ID.test(segs[0])) psi = segs.shift()!.toUpperCase();
  if (segs.length > 1 && CATEGORY.test(segs[0]!)) segs.shift();
  let investigation: string | null = null;
  if (segs.length > 1 && looksLikeInvestigation(segs[0]!)) investigation = segs.shift()!;
  // "PSI-69_Experimental table_FLEX": the investigation's acronym trails the document.
  else if (segs.length > 1 && ACRONYM.test(segs[segs.length - 1]!)) investigation = segs.pop()!;
  const qualifiers: string[] = [];
  const title =
    segs
      .map((s) => {
        const w = words(s);
        qualifiers.push(...w.qualifiers);
        return w.text;
      })
      .filter(Boolean)
      .join(" ") || (investigation ?? psi ?? base);
  const full = qualifiers.length ? `${title} (${[...new Set(qualifiers)].join(", ")})` : title;
  const by = investigation ?? psi;
  return { label: by && by !== full ? `${by} · ${full}` : full, title: full, investigation: by };
}

/** The name of a hit's source: its title, else the locator's file name, else the source id. */
export function hitSourceName(h: SearchHit): SourceName {
  return humanSourceName(h.title ?? h.evidence.locator?.path ?? h.evidence.source_id);
}

const STOP = new Set(["the", "and", "for", "with", "from", "that", "this", "into", "onto", "over", "under", "of", "in", "on", "at", "to", "by", "a", "an", "or", "is", "are", "was", "were", "be", "as", "its", "their", "than", "how", "what", "which", "why", "does", "do"]);

/** The words of a query worth marking: three letters or more, not a stop word, longest first. */
export function queryTerms(query: string): string[] {
  const t = query
    .toLowerCase()
    .split(/[^\p{L}\p{N}]+/u)
    .filter((w) => w.length >= 3 && !STOP.has(w));
  return [...new Set(t)].sort((a, b) => b.length - a.length);
}

export interface TextPart {
  text: string;
  match: boolean;
}

/** Splits text into runs, marking every word that starts with a query term ("flame" marks "flames"). */
export function markTerms(text: string, terms: string[]): TextPart[] {
  if (!terms.length || !text) return [{ text, match: false }];
  const esc = terms.map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  const re = new RegExp(`(?<![\\p{L}\\p{N}])(?:${esc.join("|")})[\\p{L}\\p{N}]*`, "giu");
  const out: TextPart[] = [];
  let at = 0;
  for (const m of text.matchAll(re)) {
    const i = m.index ?? 0;
    if (i > at) out.push({ text: text.slice(at, i), match: false });
    out.push({ text: m[0], match: true });
    at = i + m[0].length;
  }
  if (at < text.length) out.push({ text: text.slice(at), match: false });
  return out;
}

export interface HitGroup {
  sourceId: string;
  name: SourceName;
  /** The document's hits in rank order; the group sits at the rank of its best hit. */
  hits: SearchHit[];
}

/** Gathers hits of one document under one card, keeping the order of each document's best hit. */
export function groupHits(hits: SearchHit[]): HitGroup[] {
  const by = new Map<string, HitGroup>();
  for (const h of hits) {
    const id = h.evidence.source_id;
    let g = by.get(id);
    if (!g) {
      g = { sourceId: id, name: hitSourceName(h), hits: [] };
      by.set(id, g);
    }
    g.hits.push(h);
  }
  return [...by.values()];
}
