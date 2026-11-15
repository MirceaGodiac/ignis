// Run picker for the forecast lab: search by NASA test name, fuel or id, narrow by family, and see
// at a glance which runs carry a recorded time series (a forecast from a cutoff needs one; a model
// that reads conditions only does not). Runs with data are listed first.
import { useEffect, useMemo, useRef, useState } from "react";
import type { ExperimentSummary } from "../api";
import { FamilyMark } from "../shell/icons";
import { words } from "../ui/common";

const FAMILY_LABEL: Record<string, string> = { flex: "FLEX", acme_normal: "ACME normal", acme_inverse: "ACME inverse", bass: "BASS" };
export const familyLabel = (f: string) => FAMILY_LABEL[f] ?? words(f);

/** NASA's own name for a run when the registry has one (FLEX-093, ...), else null. */
export function nasaName(r: ExperimentSummary): string | null {
  const a = r.aliases ?? [];
  return (a.find((x) => /flex|testsheet|as_run|table/i.test(x.scheme) && /[A-Za-z]/.test(x.name)) ?? null)?.name ?? null;
}

export const hasSeries = (r: ExperimentSummary) => (r.n_observations ?? 0) > 0;

export function runLabel(r: ExperimentSummary): string {
  const n = nasaName(r);
  const what = r.fuel_or_material ? words(r.fuel_or_material) : familyLabel(r.family);
  return n ? `${n} · ${what}` : what;
}

interface Props {
  runs: ExperimentSummary[];
  value: string | null;
  onPick: (id: string) => void;
  loading?: boolean;
  autoFocus?: boolean;
}

export function RunPicker({ runs, value, onPick, loading, autoFocus }: Props) {
  const [q, setQ] = useState("");
  const [family, setFamily] = useState<string>("all");
  const [onlyData, setOnlyData] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (autoFocus) input.current?.focus();
  }, [autoFocus]);

  const families = useMemo(() => {
    const m = new Map<string, number>();
    for (const r of runs) if (!onlyData || hasSeries(r)) m.set(r.family, (m.get(r.family) ?? 0) + 1);
    return [...m.entries()].sort((a, b) => b[1] - a[1]);
  }, [runs, onlyData]);
  const nData = useMemo(() => runs.filter(hasSeries).length, [runs]);

  const shown = useMemo(() => {
    const needle = q.trim().toLowerCase();
    return runs
      .filter((r) => (family === "all" || r.family === family) && (!onlyData || hasSeries(r)))
      .filter((r) => !needle || [r.id, r.fuel_or_material ?? "", r.family, ...(r.aliases ?? []).map((a) => a.name)].some((s) => s.toLowerCase().includes(needle)))
      .sort((a, b) => Number(hasSeries(b)) - Number(hasSeries(a)) || (b.n_observations ?? 0) - (a.n_observations ?? 0) || a.id.localeCompare(b.id));
  }, [runs, q, family, onlyData]);

  return (
    <div className="runpick">
      <div className="runpick-bar">
        <input
          ref={input}
          className="field"
          type="search"
          placeholder="Search by test name (FLEX-093), fuel or run id"
          aria-label="Search runs"
          value={q}
          onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && shown[0]) onPick(shown[0].id);
          }}
        />
        <label className="runpick-toggle">
          <input type="checkbox" checked={onlyData} onChange={(e) => setOnlyData(e.target.checked)} />
          Only runs with a recorded time series <span className="muted num">({nData})</span>
        </label>
      </div>
      <div className="seg" role="group" aria-label="Family">
        <button aria-pressed={family === "all"} onClick={() => setFamily("all")}>
          All <span className="count num">{families.reduce((s, [, n]) => s + n, 0)}</span>
        </button>
        {families.map(([f, n]) => (
          <button key={f} aria-pressed={family === f} onClick={() => setFamily(f)}>
            <FamilyMark family={f} size={8} /> {familyLabel(f)} <span className="count num">{n}</span>
          </button>
        ))}
      </div>
      <ul className="runpick-list" aria-label="Runs">
        {loading && !runs.length && <li className="muted runpick-empty">Loading runs…</li>}
        {!loading && !shown.length && (
          <li className="muted runpick-empty">
            No run matches.{onlyData && " "}
            {onlyData && (
              <button className="linkish" onClick={() => setOnlyData(false)}>
                Include runs without recorded data
              </button>
            )}
          </li>
        )}
        {shown.map((r) => (
          <li key={r.id}>
            <button className={`runpick-row${r.id === value ? " on" : ""}`} onClick={() => onPick(r.id)} aria-current={r.id === value ? "true" : undefined}>
              <FamilyMark family={r.family} size={9} />
              <span className="runpick-main">
                <b>{runLabel(r)}</b>
                <code>{r.id}</code>
              </span>
              {hasSeries(r) ? (
                <span className="tag tag-ok num" title={(r.quantities ?? []).map((x) => `${words(x.quantity)}: ${x.count}`).join("\n")}>
                  {r.n_observations} points
                </span>
              ) : (
                <span className="tag" title="The registry holds this run's conditions but no measured time series yet">
                  conditions only
                </span>
              )}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
