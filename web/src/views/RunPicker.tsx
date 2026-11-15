// Run picker for the forecast lab: search by NASA test name, fuel or id, narrow by family, and see
// at a glance what a served model can do with each run. By default only runs a model can forecast
// are listed; the rest (sealed for evaluation, or no model for the family) are one click away.
import { useEffect, useMemo, useRef, useState } from "react";
import type { ExperimentSummary } from "../api";
import { FamilyMark } from "../shell/icons";
import { words } from "../ui/common";
import type { Reach } from "./LabModels";

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
  /** What a served model can do with each run; null while it is being worked out or when no model is served. */
  reach: Map<string, Reach> | null;
}

const REACH_ORDER: Record<Reach, number> = { ready: 0, conditions: 1, typed: 2, sealed: 3, none: 4 };

export function RunPicker({ runs, value, onPick, loading, autoFocus, reach }: Props) {
  const [q, setQ] = useState("");
  const [family, setFamily] = useState<string>("all");
  const [onlyReady, setOnlyData] = useState(true);
  const can = (r: ExperimentSummary) => !reach || (reach.get(r.id) ?? "none") === "ready" || reach.get(r.id) === "conditions";
  const onlyData = onlyReady && !!reach;
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (autoFocus) input.current?.focus();
  }, [autoFocus]);

  const families = useMemo(() => {
    const m = new Map<string, number>();
    for (const r of runs) if (!onlyData || can(r)) m.set(r.family, (m.get(r.family) ?? 0) + 1);
    return [...m.entries()].sort((a, b) => b[1] - a[1]);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [runs, onlyData, reach]);
  const nData = useMemo(() => runs.filter(can).length, [runs, reach]); // eslint-disable-line react-hooks/exhaustive-deps

  const shown = useMemo(() => {
    const needle = q.trim().toLowerCase();
    return runs
      .filter((r) => (family === "all" || r.family === family) && (!onlyData || can(r)))
      .filter((r) => !needle || [r.id, r.fuel_or_material ?? "", r.family, ...(r.aliases ?? []).map((a) => a.name)].some((s) => s.toLowerCase().includes(needle)))
      .sort((a, b) => (reach ? REACH_ORDER[reach.get(a.id) ?? "none"] - REACH_ORDER[reach.get(b.id) ?? "none"] : 0) || Number(hasSeries(b)) - Number(hasSeries(a)) || (b.n_observations ?? 0) - (a.n_observations ?? 0) || a.id.localeCompare(b.id));
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [runs, q, family, onlyData, reach]);

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
        {reach && (
          <label className="runpick-toggle">
            <input type="checkbox" checked={onlyData} onChange={(e) => setOnlyData(e.target.checked)} />
            Only runs a model can forecast <span className="muted num">({nData} of {runs.length})</span>
          </label>
        )}
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
                Show the runs no model can forecast here
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
              <ReachTag r={r} reach={reach?.get(r.id) ?? null} />
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

const NL = String.fromCharCode(10);

function ReachTag({ r, reach }: { r: ExperimentSummary; reach: Reach | null }) {
  const points = (r.quantities ?? []).map((x) => `${words(x.quantity)}: ${x.count}`).join(NL);
  if (reach === "ready")
    return (
      <span className="tag tag-ok num" title={hasSeries(r) ? `A model reads this run directly.${NL}${points}` : "A model reads this run's conditions directly"}>
        {hasSeries(r) ? `forecast · ${r.n_observations} points` : "forecast"}
      </span>
    );
  if (reach === "conditions")
    return (
      <span className="tag tag-ok" title="A model forecasts from this run's conditions, filled in for you">
        forecast from conditions
      </span>
    );
  if (reach === "typed")
    return (
      <span className="tag tag-line" title="A model is written for this family, but it needs values the NASA record does not hold: you type them">
        you type its data
      </span>
    );
  if (reach === "sealed")
    return (
      <span className="tag tag-line" title="Validation or test run: kept from the models for evaluation">
        sealed for evaluation
      </span>
    );
  if (reach === "none")
    return (
      <span className="tag tag-line" title="No served model is written for this family">
        no model
      </span>
    );
  return hasSeries(r) ? (
    <span className="tag num" title={points}>
      {r.n_observations} points
    </span>
  ) : (
    <span className="tag" title="The registry holds this run's conditions but no measured time series yet">
      conditions only
    </span>
  );
}
