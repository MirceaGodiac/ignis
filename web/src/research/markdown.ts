// Pure part of the answer's Markdown (node --test runs it). Agent text is parsed into blocks and
// inline parts; the renderer builds React elements from them and never renders agent text as HTML.
// Supported: paragraphs (single newlines kept as breaks), headings, bullet and numbered lists,
// block quotes, fenced code, rules, GFM tables (alignment row, numeric columns right-aligned),
// **bold**, *italic*, `code`, [links](https://…) (http and https only) and known run names.

import { splitRich, type RichPart } from "../shell/richParts.ts";

export type Align = "left" | "right" | "center" | null;

export type MdBlock =
  | { kind: "heading"; level: number; text: string }
  | { kind: "paragraph"; text: string }
  | { kind: "list"; ordered: boolean; start: number; items: { text: string; level: number }[] }
  | { kind: "code"; lang: string; text: string }
  | { kind: "quote"; text: string }
  | { kind: "rule" }
  | { kind: "table"; header: string[]; align: Align[]; rows: string[][]; numeric: boolean[] };

export type InlinePart =
  | Exclude<RichPart, { kind: "bold" }>
  | { kind: "bold"; parts: InlinePart[] }
  | { kind: "italic"; parts: InlinePart[] }
  | { kind: "code"; text: string };

// ------------------------------------------------------------------ tables

/** Cells of one table row: outer pipes optional, `\|` is a literal pipe, pipes inside `code` do not split. */
export function splitTableRow(line: string): string[] {
  let s = line.trim();
  if (s.startsWith("|")) s = s.slice(1);
  if (s.endsWith("|") && !s.endsWith("\\|")) s = s.slice(0, -1);
  const cells: string[] = [];
  let cur = "";
  let inCode = false;
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if (c === "\\" && s[i + 1] === "|") {
      cur += "|";
      i++;
    } else if (c === "`") {
      inCode = !inCode;
      cur += c;
    } else if (c === "|" && !inCode) {
      cells.push(cur.trim());
      cur = "";
    } else cur += c;
  }
  cells.push(cur.trim());
  return cells;
}

/** The alignment row under a table header (`| :--- | ---: | :---: |`), or null when the line is not one. */
export function parseAlignRow(line: string): Align[] | null {
  if (!line.includes("-")) return null;
  const cells = splitTableRow(line);
  if (!cells.length || !cells.every((c) => /^:?-+:?$/.test(c))) return null;
  // A lone `---` without pipes is a rule, not a table.
  if (cells.length === 1 && !line.includes("|")) return null;
  return cells.map((c) => (c.startsWith(":") && c.endsWith(":") ? "center" : c.endsWith(":") ? "right" : c.startsWith(":") ? "left" : null));
}

const EMPTY_CELL = /^(|—|–|-|n\/a|na|nan|null|none|\?)$/i;
// A number, optionally signed or approximate, optionally followed by a unit after a space (or %, °).
// "21106A4" and "2010-03-23" are names, not numbers.
const NUMERIC_CELL = /^(?:[~≈<>≤≥±]\s?)?[+\-−]?(?:\d{1,3}(?:,\d{3})+|\d+)?(?:\.\d+)?(?:[eE][+\-−]?\d+)?(?:\s?(?:%|‰|°[CFK]?)|\s+[^\s\d][^|]{0,15})?$/;

/** A cell that reads as a number (with or without its unit). Markup around it (`**1.2**`) is ignored. */
export function isNumericCell(s: string): boolean {
  const t = s.replace(/[*_`]/g, "").trim();
  if (!/\d/.test(t.split(/\s/)[0] ?? "")) return false;
  return NUMERIC_CELL.test(t);
}

export const isEmptyCell = (s: string) => EMPTY_CELL.test(s.replace(/[*_`]/g, "").trim());

/** Columns whose every non-empty cell is a number (at least one). */
export function numericColumns(rows: string[][], n: number): boolean[] {
  return Array.from({ length: n }, (_, j) => {
    let any = false;
    for (const r of rows) {
      const c = r[j] ?? "";
      if (isEmptyCell(c)) continue;
      if (!isNumericCell(c)) return false;
      any = true;
    }
    return any;
  });
}

// ------------------------------------------------------------------ blocks

const FENCE = /^\s{0,3}(`{3,}|~{3,})\s*([\w+.-]*)\s*$/;
const HEADING = /^\s{0,3}(#{1,6})\s+(.*?)\s*#*\s*$/;
const RULE = /^\s{0,3}([-*_])(?:\s*\1){2,}\s*$/;
const QUOTE = /^\s{0,3}>\s?(.*)$/;
const LIST = /^(\s*)([-*+•]|\d{1,9}[.)])\s+(.*)$/;

const isTableStart = (lines: string[], i: number) => lines[i].includes("|") && i + 1 < lines.length && parseAlignRow(lines[i + 1]) != null;

function startsBlock(lines: string[], i: number): boolean {
  const l = lines[i];
  return FENCE.test(l) || HEADING.test(l) || RULE.test(l) || QUOTE.test(l) || LIST.test(l) || isTableStart(lines, i);
}

export function parseMarkdown(src: string): MdBlock[] {
  const lines = src.replace(/\r\n?/g, "\n").split("\n");
  const out: MdBlock[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) {
      i++;
      continue;
    }
    const fence = FENCE.exec(line);
    if (fence) {
      const body: string[] = [];
      i++;
      while (i < lines.length && !lines[i].trim().startsWith(fence[1])) body.push(lines[i++]);
      i++; // closing fence (or end of text)
      out.push({ kind: "code", lang: fence[2] ?? "", text: body.join("\n") });
      continue;
    }
    if (isTableStart(lines, i)) {
      const header = splitTableRow(line);
      const n = header.length;
      const alignRow = parseAlignRow(lines[i + 1])!;
      const align = Array.from({ length: n }, (_, j) => alignRow[j] ?? null);
      const rows: string[][] = [];
      i += 2;
      while (i < lines.length && lines[i].trim() && lines[i].includes("|")) {
        const cells = splitTableRow(lines[i++]);
        rows.push(Array.from({ length: n }, (_, j) => (j === n - 1 && cells.length > n ? cells.slice(j).join(" | ") : cells[j] ?? "")));
      }
      out.push({ kind: "table", header, align, rows, numeric: numericColumns(rows, n) });
      continue;
    }
    const h = HEADING.exec(line);
    if (h) {
      out.push({ kind: "heading", level: h[1].length, text: h[2] });
      i++;
      continue;
    }
    if (RULE.test(line)) {
      out.push({ kind: "rule" });
      i++;
      continue;
    }
    if (QUOTE.test(line)) {
      const body: string[] = [];
      while (i < lines.length && QUOTE.test(lines[i])) body.push(QUOTE.exec(lines[i++])![1]);
      out.push({ kind: "quote", text: body.join("\n") });
      continue;
    }
    const li = LIST.exec(line);
    if (li) {
      const ordered = /\d/.test(li[2]);
      const start = ordered ? parseInt(li[2], 10) : 1;
      const base = li[1].length;
      const items: { text: string; level: number }[] = [];
      while (i < lines.length) {
        const m = LIST.exec(lines[i]);
        if (m && (m[1].length > base || /\d/.test(m[2]) === ordered)) {
          items.push({ text: m[3], level: Math.min(3, Math.floor(Math.max(0, m[1].length - base) / 2)) });
          i++;
        } else if (!m && lines[i].trim() && /^\s+/.test(lines[i]) && items.length) {
          items[items.length - 1].text += "\n" + lines[i++].trim(); // continuation of the item
        } else break;
      }
      out.push({ kind: "list", ordered, start, items });
      continue;
    }
    const para: string[] = [line];
    i++;
    while (i < lines.length && lines[i].trim() && !startsBlock(lines, i)) para.push(lines[i++]);
    out.push({ kind: "paragraph", text: para.join("\n") });
  }
  return out;
}

// ------------------------------------------------------------------ inline

const INLINE =
  /`([^`\n]+)`|\[([^\]\n]{1,300})\]\(([^\s)]+)\)|\*\*(?=\S)([^\n]+?)(?<=\S)\*\*|__(?=\S)([^\n]+?)(?<=\S)__|\*(?=[^\s*])([^*\n]*?[^\s*\\])\*|(?<![\p{L}\p{N}_])_(?=[^\s_])([^_\n]*?[^\s_])_(?![\p{L}\p{N}_])/gu;

export function httpUrl(u: string): boolean {
  try {
    const p = new URL(u).protocol;
    return p === "https:" || p === "http:";
  } catch {
    return false;
  }
}

/** Inline parts of one run of agent text; plain stretches go through the run-name linker. */
export function parseInline(text: string, runs: Map<string, string>): InlinePart[] {
  const out: InlinePart[] = [];
  let last = 0;
  const plain = (s: string) => {
    if (s) out.push(...(splitRich(s, runs) as InlinePart[]));
  };
  for (const m of text.matchAll(INLINE)) {
    plain(text.slice(last, m.index));
    last = m.index! + m[0].length;
    if (m[1] != null) out.push({ kind: "code", text: m[1] });
    else if (m[2] != null) {
      if (httpUrl(m[3])) out.push({ kind: "link", label: m[2], href: m[3] });
      else plain(m[2]); // an unsafe link keeps its label, loses its target
    } else if (m[4] != null || m[5] != null) out.push({ kind: "bold", parts: parseInline(m[4] ?? m[5], runs) });
    else out.push({ kind: "italic", parts: parseInline(m[6] ?? m[7], runs) });
  }
  plain(text.slice(last));
  return out;
}

/** Plain text of inline markup (for titles, sorting and copy). */
export function inlineText(parts: InlinePart[]): string {
  return parts
    .map((p) => {
      switch (p.kind) {
        case "text":
        case "code":
        case "run":
          return p.text;
        case "link":
          return p.label;
        case "break":
          return "\n";
        default:
          return inlineText(p.parts);
      }
    })
    .join("");
}
