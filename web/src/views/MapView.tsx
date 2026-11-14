// Atmosphere map: runs by recorded ambient oxygen and pressure against crew atmospheres.
import { useMemo, useState } from "react";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { FamilyMark } from "../shell/icons";
import { href } from "../shell/router";
import { ErrorNote } from "../ui/common";
import { AtmosMap, FamilyLegend, HABITATS, moonDistance, placeRuns } from "./AtmosMap";

export function MapView() {
  const { experiments, synthetic } = useData();
  const panel = usePanel();
  const [family, setFamily] = useState<string | null>(null);
  const runs = experiments.data?.experiments ?? [];
  const { placed, missing } = useMemo(() => placeRuns(runs), [runs]);
  const counts = useMemo(() => runs.reduce<Record<string, number>>((m, r) => ((m[r.family] = (m[r.family] ?? 0) + 1), m), {}), [runs]);
  const selected = panel.current?.kind === "run" ? panel.current.id : null;
  const kpaSpan = Math.max(110, ...placed.map((p) => p.kpa));
  const nearest = useMemo(() => placed.map((p) => ({ p, d: moonDistance(p.o2, p.kpa, kpaSpan) })).sort((a, b) => a.d - b.d).slice(0, 8), [placed, kpaSpan]);
  const unplaced = missing.o2 + missing.pressure + missing.both;

  return (
    <div className="view">
      <div className="view-head">
        <h1>Atmosphere map</h1>
        <div className="toolbar">
          <FamilyLegend counts={counts} active={family} onPick={setFamily} />
          <a className="btn btn-s" href={href("runs", null, family ? { family } : undefined)}>
            Table
          </a>
        </div>
      </div>
      {experiments.error && <ErrorNote error={experiments.error} title="Runs could not be loaded" />}
      {experiments.data && runs.length === 0 && (
        <div className="empty">
          <b>No runs in this data source</b>
          <span>{synthetic ? "Start the server with --dev." : "Run ignis data ingest, or switch to synthetic data."}</span>
        </div>
      )}
      {runs.length > 0 && (
        <section className="card map-grid" aria-label="Runs by oxygen and pressure">
          <div>
            <div className="card-body">
              <AtmosMap runs={runs} selected={selected} familyFilter={family} onSelect={(id) => panel.open({ kind: "run", id })} />
            </div>
            <div className="map-legend">
              <span>
                {placed.length} of {runs.length} runs placed
                {unplaced > 0 && ` · ${unplaced} lack pressure or oxygen`}
              </span>
              <span>Moon base and rovers {HABITATS.moon.sub}</span>
              <span>No partial-gravity data in PSI yet</span>
            </div>
          </div>
          <aside className="map-side" aria-label="Closest runs to the Moon-base atmosphere">
            <div className="panel-sec">
              <h3>Closest to Moon-base air</h3>
              <ol className="rank">
                {nearest.map(({ p, d }) => (
                  <li key={p.run.id}>
                    <button onClick={() => panel.open({ kind: "run", id: p.run.id })} aria-pressed={selected === p.run.id}>
                      <FamilyMark family={p.run.family} size={9} />
                      <span className="cell-id">{p.run.id}</span>
                      <span className="v">{d === 0 ? "inside" : `${(p.o2 * 100).toFixed(0)}% · ${p.kpa.toFixed(0)} kPa`}</span>
                    </button>
                  </li>
                ))}
              </ol>
              <p className="muted small">Distance on both axes; geometry, not a model.</p>
            </div>
          </aside>
        </section>
      )}
    </div>
  );
}
