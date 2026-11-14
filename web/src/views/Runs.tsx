// Run index: one row per run, every recorded condition in its own sortable column, the same
// selection as the map and Compare. A row opens the run in the side panel.
import { useMemo, useState } from "react";
import type { ExperimentSummary } from "../api";
import { fmtNum } from "../format";
import { MAX_COMPARE, useCompare } from "../shell/compare";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { FamilyMark } from "../shell/icons";
import { go } from "../shell/router";
import { ErrorNote, Synthetic } from "../ui/common";
import { FamilyLegend } from "./AtmosMap";

type Col = "id" | "family" | "fuel" | "o2" | "pressure" | "flow" | "obs" | "endpoint";
const cond = (r: ExperimentSummary, k: string) => r.conditions[k]?.value_si ?? null;
const val: Record<Col, (r: ExperimentSummary) => string | number | null> = {
  id: (r) => r.id,
  family: (r) => r.family,
  fuel: (r) => r.fuel_or_material,
  o2: (r) => cond(r, "oxygen_mole_fraction"),
  pressure: (r) => cond(r, "pressure"),
  flow: (r) => cond(r, "flow_velocity") ?? cond(r, "fuel_flow_rate"),
  obs: (r) => r.n_observations,
  endpoint: (r) => (r.has_endpoint ? 1 : 0),
};

export function Runs({ initialFamily }: { initialFamily: string | null }) {
  const { experiments, synthetic } = useData();
  const panel = usePanel();
  const cmp = useCompare();
  const [family, setFamily] = useState<string | null>(initialFamily);
  const [text, setText] = useState("");
  const [sort, setSort] = useState<{ col: Col; dir: 1 | -1 }>({ col: "id", dir: 1 });
  const runs = experiments.data?.experiments ?? [];
  const counts = useMemo(() => runs.reduce<Record<string, number>>((m, r) => ((m[r.family] = (m[r.family] ?? 0) + 1), m), {}), [runs]);
  const rows = useMemo(() => {
    const t = text.trim().toLowerCase();
    const f = runs.filter((r) => (!family || r.family === family) && (!t || `${r.id} ${r.fuel_or_material ?? ""} ${r.configuration ?? ""} ${r.investigation_id}`.toLowerCase().includes(t)));
    return [...f].sort((a, b) => {
      const x = val[sort.col](a);
      const y = val[sort.col](b);
      if (x == null && y == null) return 0;
      if (x == null) return 1; // missing values sort last, whatever the direction
      if (y == null) return -1;
      return (x < y ? -1 : x > y ? 1 : 0) * sort.dir;
    });
  }, [runs, family, text, sort]);
  const selectedRun = panel.current?.kind === "run" ? panel.current.id : null;

  const Th = ({ col, children, r, hide }: { col: Col; children: string; r?: boolean; hide?: boolean }) => (
    <th className={[r ? "r" : "", hide ? "narrow-hide" : ""].join(" ").trim() || undefined} aria-sort={sort.col === col ? (sort.dir === 1 ? "ascending" : "descending") : "none"}>
      <button className={`sort${sort.col === col ? " on" : ""}`} onClick={() => setSort((s) => ({ col, dir: s.col === col ? (-s.dir as 1 | -1) : 1 }))}>
        {children}
        {sort.col === col ? (sort.dir === 1 ? " ↑" : " ↓") : ""}
      </button>
    </th>
  );
  const num = (v: number | null, k = 1, digits?: number) => (v == null ? <span className="dim">—</span> : digits != null ? (v * k).toFixed(digits) : fmtNum(v * k));

  return (
    <div className="view">
      <div className="view-head">
        <h1>Runs</h1>
      </div>
      <div className="toolbar">
        <FamilyLegend counts={counts} active={family} onPick={setFamily} />
        <label className="sr-only" htmlFor="run-filter">
          Filter runs
        </label>
        <input id="run-filter" className="field" placeholder="Filter runs" value={text} onChange={(e) => setText(e.target.value)} style={{ width: 260 }} />
        <span className="muted" style={{ marginLeft: "auto", fontSize: 12 }}>
          {rows.length} of {runs.length}
        </span>
      </div>
      {experiments.error && <ErrorNote error={experiments.error} title="Runs could not be loaded" />}
      <div className="tbl-wrap">
        <table className="tbl">
          <thead>
            <tr>
              <th style={{ width: 36 }}>
                <span className="sr-only">Compare</span>
              </th>
              <Th col="id">Run</Th>
              <Th col="family">Family</Th>
              <Th col="fuel" hide>
                Fuel or material
              </Th>
              <Th col="o2" r>
                O₂ %
              </Th>
              <Th col="pressure" r>
                Pressure kPa
              </Th>
              <Th col="flow" r hide>
                Flow
              </Th>
              <Th col="obs" r hide>
                Points
              </Th>
              <Th col="endpoint" hide>Endpoint</Th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => {
              const flow = cond(r, "flow_velocity");
              const fuelFlow = cond(r, "fuel_flow_rate");
              return (
                <tr key={r.id} className="clickable" aria-selected={selectedRun === r.id} onClick={() => panel.open({ kind: "run", id: r.id })}>
                  <td onClick={(e) => e.stopPropagation()}>
                    <input
                      type="checkbox"
                      aria-label={`Compare ${r.id}`}
                      checked={cmp.has(r.id)}
                      disabled={!cmp.has(r.id) && cmp.ids.length >= MAX_COMPARE}
                      onChange={() => cmp.toggle(r.id)}
                    />
                  </td>
                  <td>
                    <span className="cell-id">{r.id}</span> {r.synthetic && !synthetic && <Synthetic />}
                  </td>
                  <td>
                    <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
                      <FamilyMark family={r.family} size={9} /> <span className="narrow-hide">{r.family.replace("_", " ")}</span>
                    </span>
                  </td>
                  <td className="narrow-hide">{r.fuel_or_material ?? <span className="dim">unknown</span>}</td>
                  <td className="r num">{num(cond(r, "oxygen_mole_fraction"), 100, 0)}</td>
                  <td className="r num">{num(cond(r, "pressure"), 0.001, 1)}</td>
                  <td className="r num narrow-hide">
                    {flow != null ? `${fmtNum(flow)} m/s` : fuelFlow != null ? `${fmtNum(fuelFlow * 6e7)} mL/min` : <span className="dim">—</span>}
                  </td>
                  <td className="r num narrow-hide">{r.n_observations}</td>
                  <td className="narrow-hide">{r.has_endpoint ? "recorded" : <span className="dim">missing</span>}</td>
                </tr>
              );
            })}
            {rows.length === 0 && runs.length > 0 && (
              <tr>
                <td colSpan={9} className="dim">
                  No run matches. <button className="linkish" onClick={() => (setText(""), setFamily(null))}>Clear the filters</button>
                </td>
              </tr>
            )}
            {experiments.data && runs.length === 0 && (
              <tr>
                <td colSpan={9} className="dim">
                  {synthetic ? "No synthetic runs are loaded. Start the server with --dev." : "The registry has no runs yet. Run ignis data ingest, or switch to synthetic data."}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
      {cmp.ids.length > 0 && (
        <div className="tray" role="region" aria-label="Compare selection">
          <span>
            {cmp.ids.length} selected{cmp.ids.length >= MAX_COMPARE ? ` (maximum ${MAX_COMPARE})` : ""}
          </span>
          <button className="btn btn-s" onClick={() => go("compare")} disabled={cmp.ids.length < 2}>
            Compare
          </button>
          <button className="btn btn-s btn-quiet" onClick={cmp.clear}>
            Clear
          </button>
        </div>
      )}
    </div>
  );
}
