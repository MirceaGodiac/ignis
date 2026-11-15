// NASA originals a run record points at: every locator in its aliases (test sheets, experimental
// tables, file names) and its recorded conditions, one entry per file, cited at the first place
// the record uses it. Pure, so node --test runs it.
import type { SourceLocator } from "../contracts/generated";

export interface RunSource {
  sourceId: string;
  path: string | null;
  loc: SourceLocator;
  /** What the record takes from this file (alias schemes and condition keys). */
  uses: string[];
}

const isLoc = (l: unknown): l is SourceLocator => !!l && typeof l === "object" && typeof (l as SourceLocator).asset_sha256 === "string" && /^[0-9a-f]{64}$/.test((l as SourceLocator).asset_sha256);

export function runSources(record: unknown): RunSource[] {
  const by = new Map<string, RunSource>();
  const add = (l: unknown, use: string) => {
    if (!isLoc(l)) return;
    const s = by.get(l.asset_sha256);
    if (!s) by.set(l.asset_sha256, { sourceId: `sha256:${l.asset_sha256}`, path: l.path ?? null, loc: l, uses: [use] });
    else {
      if (!s.uses.includes(use)) s.uses.push(use);
      // Prefer a locator that points at a place (row or page) over a whole-file one.
      if (s.loc.row == null && s.loc.page == null && (l.row != null || l.page != null)) s.loc = l;
    }
  };
  const r = (record ?? {}) as { aliases?: { locator?: unknown; scheme?: string; name?: string }[]; conditions?: Record<string, { locator?: unknown } | null> };
  for (const a of r.aliases ?? []) add(a.locator, `name ${a.name ?? ""}`.trim());
  for (const [k, c] of Object.entries(r.conditions ?? {})) add(c?.locator, k.replace(/_/g, " "));
  return [...by.values()];
}
