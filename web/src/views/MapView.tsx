// Atmosphere map (T9): runs by recorded ambient oxygen and pressure against the crew atmospheres,
// from the conditions table of the 19 flight investigations (K1.13). Only recorded values are placed;
// every run that cannot be placed is counted with the reason.
import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ConditionsRow, ConditionsTable } from "../contracts/generated";
import { pointsFromConditions } from "../research/mapPoints";
import { useData } from "../shell/data";
import { Gap } from "../ui/common";
import { AtmosMap, HABITATS, moonDistance, PSI } from "./AtmosMap";

const PAGE = 1000;

export function MapView() {
  const { client, researchMode } = useData();
  const [inv, setInv] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  // The map needs every run, so it reads all pages; only the three keys it draws or labels.
  const table = useQuery({
    queryKey: [researchMode, "conditions", "map"],
    queryFn: async (): Promise<ConditionsTable> => {
      const rows: ConditionsRow[] = [];
      let cursor: string | null = null;
      let first: ConditionsTable | null = null;
      do {
        const page: ConditionsTable = await client.conditions({ schema_version: "dataset.v2", keys: ["oxygen_mole_fraction", "pressure", "fuel"], include_unknown: true, page_size: PAGE, cursor });
        first ??= page;
        rows.push(...page.rows);
        cursor = page.next_cursor ?? null;
      } while (cursor);
      return { ...first!, rows, next_cursor: null };
    },
    retry: false,
  });
  const { points, unplaced } = useMemo(() => pointsFromConditions(table.data ?? { rows: [] }), [table.data]);
  const counts = useMemo(() => points.reduce<Record<string, number>>((m, p) => ((m[p.investigation] = (m[p.investigation] ?? 0) + 1), m), {}), [points]);
  const kpaSpan = Math.max(110, ...points.map((p) => p.kpa));
  const nearest = useMemo(() => points.map((p) => ({ p, d: moonDistance(p.o2, p.kpa, kpaSpan) })).sort((a, b) => a.d - b.d).slice(0, 8), [points, kpaSpan]);
  const picked = points.find((p) => p.id === selected) ?? null;
  const total = table.data?.rows.length ?? 0;

  return (
    <>
      {table.isPending && <p className="muted">Loading…</p>}
      {table.error && <Gap title="Conditions are not available on this server">{(table.error as Error).message}</Gap>}
      {table.data && (
        <>
          <div className="seg" role="group" aria-label="Highlight an investigation" style={{ flexWrap: "wrap" }}>
            <button aria-pressed={inv === null} onClick={() => setInv(null)}>
              All <span className="count">{points.length}</span>
            </button>
            {Object.entries(counts)
              .sort((a, b) => b[1] - a[1])
              .map(([i, n]) => (
                <button key={i} aria-pressed={inv === i} onClick={() => setInv(inv === i ? null : i)}>
                  {i} <span className="count">{n}</span>
                </button>
              ))}
          </div>
          <section className="card map-grid" aria-label="Runs by oxygen and pressure">
            <div>
              <div className="card-body">
                <AtmosMap points={points} selected={selected} filter={inv} onSelect={(id) => setSelected(selected === id ? null : id)} />
              </div>
              <div className="map-legend">
                <span>
                  {points.length} of {total} runs placed
                  {unplaced.both + unplaced.o2 + unplaced.pressure > 0 && ` · ${unplaced.both + unplaced.o2 + unplaced.pressure} lack O₂ or pressure`}
                  {unplaced.notAtmosphere > 0 && ` · ${unplaced.notAtmosphere} with burner-stream O₂ (not the atmosphere)`}
                </span>
                <span>Moon base and rovers {HABITATS.moon.sub}</span>
                <span>No partial-gravity data in the tables</span>
              </div>
            </div>
            <aside className="map-side" aria-label="Selected run and the runs closest to the Moon-base atmosphere">
              {picked && (
                <div className="panel-sec">
                  <h3>Selected run</h3>
                  <dl className="kv">
                    <dt>run</dt>
                    <dd>
                      <code>{picked.id}</code>
                    </dd>
                    <dt>investigation</dt>
                    <dd>{picked.investigation}</dd>
                    <dt>fuel</dt>
                    <dd>{picked.fuel ?? "not stated"}</dd>
                    <dt>O₂</dt>
                    <dd className="num">
                      {picked.approximate ? "~" : ""}
                      {(picked.o2 * 100).toFixed(1)} %
                    </dd>
                    <dt>pressure</dt>
                    <dd className="num">
                      {picked.kpa.toFixed(1)} kPa ({(picked.kpa / PSI).toFixed(1)} psi)
                    </dd>
                  </dl>
                </div>
              )}
              <div className="panel-sec">
                <h3>Closest to Moon-base air</h3>
                <ol className="rank">
                  {nearest.map(({ p, d }) => (
                    <li key={p.id}>
                      <button onClick={() => setSelected(p.id)} aria-pressed={selected === p.id}>
                        <span className="cell-id">{p.id}</span>
                        <span className="v">{d === 0 ? "inside" : `${(p.o2 * 100).toFixed(0)}% · ${p.kpa.toFixed(0)} kPa`}</span>
                      </button>
                    </li>
                  ))}
                </ol>
              </div>
            </aside>
          </section>
        </>
      )}
    </>
  );
}
