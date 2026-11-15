// Pure part of RichText (node --test runs it): split agent text into text, line breaks, bold, safe
// links and known run names.

export type RichPart =
  | { kind: "text"; text: string }
  | { kind: "break" }
  | { kind: "bold"; parts: RichPart[] }
  | { kind: "link"; label: string; href: string }
  | { kind: "run"; text: string; id: string };

/** The comparison form of a run name: no case, spaces, hyphens, dashes or underscores. */
export const runNameKey = (s: string) => s.replace(/[\s\-‐-―_]/g, "").toUpperCase();

/** Candidate run mentions: run ids (PSI-<n>:…), FLEX-<n> (any dash), CFI-G test names (21088 A1). */
const RUN = /\bPSI-\d+:[A-Za-z0-9_:.#-]*[A-Za-z0-9]|\bFLEX\s?[-‐-―]\s?\d{2,4}\b|\b\d{5}\s?[A-Z]\d{1,2}\b/g;
const LINK = /\[([^\]\n]{1,300})\]\((https?:\/\/[^\s)]+)\)/g;

function httpUrl(u: string): boolean {
  try {
    const p = new URL(u).protocol;
    return p === "https:" || p === "http:";
  } catch {
    return false;
  }
}

function runs(text: string, index: Map<string, string>): RichPart[] {
  const out: RichPart[] = [];
  let last = 0;
  for (const m of text.matchAll(RUN)) {
    const id = index.get(runNameKey(m[0]));
    if (!id) continue;
    if (m.index! > last) out.push({ kind: "text", text: text.slice(last, m.index) });
    out.push({ kind: "run", text: m[0], id });
    last = m.index! + m[0].length;
  }
  if (last < text.length) out.push({ kind: "text", text: text.slice(last) });
  return out;
}

function links(text: string, index: Map<string, string>): RichPart[] {
  const out: RichPart[] = [];
  let last = 0;
  for (const m of text.matchAll(LINK)) {
    if (!httpUrl(m[2])) continue;
    if (m.index! > last) out.push(...runs(text.slice(last, m.index), index));
    out.push({ kind: "link", label: m[1], href: m[2] });
    last = m.index! + m[0].length;
  }
  if (last < text.length) out.push(...runs(text.slice(last), index));
  return out;
}

export function splitRich(text: string, index: Map<string, string>): RichPart[] {
  const out: RichPart[] = [];
  text.split("\n").forEach((line, li) => {
    if (li > 0) out.push({ kind: "break" });
    // **bold** spans; an unmatched ** stays literal.
    const segs = line.split(/(\*\*[^*\n]+\*\*)/g);
    for (const s of segs) {
      if (!s) continue;
      if (s.startsWith("**") && s.endsWith("**") && s.length > 4) out.push({ kind: "bold", parts: links(s.slice(2, -2), index) });
      else out.push(...links(s, index));
    }
  });
  return out;
}
