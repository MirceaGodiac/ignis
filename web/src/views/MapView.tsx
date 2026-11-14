// Atmosphere map tab: the interactive map, its layers (investigations, crew atmospheres), and beside it
// the test points nearest the chosen atmosphere and the selected point's runs.
import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ConditionsRow, ConditionsTable } from "../contracts/generated";
import { invColor, invName, INVESTIGATIONS } from "../data/investigations";
import { HABITAT_LIST, type HabitatId } from "../research/habitats";
import { conditionsView, offset, offsetText, type ConditionsView, type TestPoint } from "../research/testPoints";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { Gap } from "../ui/common";
import { ConditionsMap, HOME, ShapeSwatch, type Domain } from "./ConditionsMap";

const PAGE = 1000;

/** A run's own name: the id without investigation, family token and day. */
const runLabel = (id: string) => id.split(":").slice(1).filter((p) => !/^(acme_normal|acme_inverse|flex|bass)$/.test(p) && !/^\d{4}-\d{2}-\d{2}$/.test(p)).join(" ") || id;

/** The test points on screen as CSV: one row per test point, every run id kept. */
function downloadCsv(points: TestPoint[]) {
  const q = (v: string) => (/[",\n]/.test(v) ? `"${v.replace(/"/g, '""')}"` : v);
  const rows = [["investigation", "name", "fuel", "oxygen_percent", "pressure_kpa", "pressure_psi", "runs", "run_ids"]];
  for (const p of points)
    rows.push([p.investigation, invName(p.investigation), p.fuel ?? "", (p.o2 * 100).toFixed(2), p.kpa.toFixed(2), (p.kpa / 6.894757).toFixed(2), String(p.runs.length), p.runs.join(" ")]);
  const blob = new Blob([rows.map((r) => r.map(q).join(",")).join("\n") + "\n"], { type: "text/csv" });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = "ignis-test-points.csv";
  a.click();
  URL.revokeObjectURL(a.href);
}

/** Every row of the conditions table (the keys the views read), shared by the map and the atmosphere ranking. */
export function useConditionsView() {
  const { client, researchMode } = useData();
  return useQuery({
    queryKey: [researchMode, "conditions", "view"],
    queryFn: async (): Promise<ConditionsView> => {
      const rows: ConditionsRow[] = [];
      let cursor: string | null = null;
      do {
        const page: ConditionsTable = await client.conditions({ schema_version: "dataset.v2", keys: ["oxygen_mole_fraction", "pressure", "fuel"], include_unknown: true, page_size: PAGE, cursor });
        rows.push(...page.rows);
        cursor = page.next_cursor ?? null;
      } while (cursor);
      return conditionsView(rows);
    },
    retry: false,
    staleTime: 10 * 60_000,
  });
}

export function MapView({ focus }: { focus?: string | null }) {
  const q = useConditionsView();
  const panel = usePanel();
  const [hidden, setHidden] = useState<Set<string>>(() => new Set());
  const [habitats, setHabitats] = useState<Set<HabitatId>>(() => new Set(["moon", "suit", "iss"]));
  const [near, setNear] = useState<HabitatId>("moon");
  const [selected, setSelected] = useState<string | null>(null);
  const [domain, setDomain] = useState<Domain>(HOME);

  const v = q.data;
  const invs = useMemo(() => {
    if (!v) return [];
    const m = new Map<string, number>();
    for (const p of v.points) m.set(p.investigation, (m.get(p.investigation) ?? 0) + p.runs.length);
    return [...m.entries()].sort((a, b) => b[1] - a[1]);
  }, [v]);
  // An investigation opened from a card: only it, framed.
  const [focused, setFocused] = useState<string | null>(null);
  if (v && focus && focus !== focused) {
    setFocused(focus);
    setHidden(new Set(invs.map(([id]) => id).filter((id) => id !== focus)));
    const ps = v.points.filter((p) => p.investigation === focus);
    if (ps.length) {
      const ks = ps.map((p) => p.kpa);
      setDomain({ x: [0.1, 1.03], y: [0, Math.max(150, Math.max(...ks) * 1.1)] });
    }
  }
  if (q.isPending) return <p className="muted">Loading…</p>;
  if (q.error || !v) return <Gap title="Conditions are not available on this server">{(q.error as Error)?.message}</Gap>;

  const maxKpa = Math.max(150, ...v.points.map((p) => p.kpa));
  const h = HABITAT_LIST.find((x) => x.id === near)!;
  const nearest = v.points
    .filter((p) => !hidden.has(p.investigation))
    .map((p) => ({ p, o: offset(p.o2, p.kpa, h) }))
    .sort((a, b) => a.o.distance - b.o.distance || b.p.runs.length - a.p.runs.length)
    .slice(0, 8);
  const sel = v.points.find((p) => p.key === selected) ?? null;
  const toggle = (id: string) => setHidden((s) => (s.has(id) ? new Set([...s].filter((x) => x !== id)) : new Set([...s, id])));
  const center = (o2: number, kpa: number) =>
    setDomain((d) => {
      const wx = d.x[1] - d.x[0];
      const wy = d.y[1] - d.y[0];
      return { x: [o2 - wx / 2, o2 + wx / 2], y: [Math.max(0, kpa - wy / 2), Math.max(0, kpa - wy / 2) + wy] };
    });

  return (
    <div className="mapv">
      <div className="mapv-layers">
        <div className="chips" role="group" aria-label="Investigations shown">
          {invs.map(([id, n]) => (
            <button key={id} className={`chip-t${hidden.has(id) ? " off" : ""}`} aria-pressed={!hidden.has(id)} onClick={() => toggle(id)} title={INVESTIGATIONS.get(id)?.title ?? id}>
              <ShapeSwatch id={id} />
              {invName(id)}
              <span className="n">{n}</span>
            </button>
          ))}
          <span className="chips-sep" />
          <button className="linkish small" onClick={() => setHidden(new Set())}>
            All
          </button>
          <button className="linkish small" onClick={() => setHidden(new Set(invs.map(([id]) => id)))}>
            None
          </button>
        </div>
        <div className="chips" role="group" aria-label="Crew atmospheres shown">
          {HABITAT_LIST.map((x) => (
            <button
              key={x.id}
              className={`chip-t${habitats.has(x.id) ? "" : " off"}`}
              aria-pressed={habitats.has(x.id)}
              onClick={() => setHabitats((s) => (s.has(x.id) ? new Set([...s].filter((y) => y !== x.id)) : new Set([...s, x.id])))}
            >
              <span className={`hab-sw hab-${x.id}`} aria-hidden="true" />
              {x.label}
            </button>
          ))}
        </div>
      </div>

      <div className="mapv-grid">
        <div className="card mapv-chart">
          <ConditionsMap points={v.points} hidden={hidden} habitats={habitats} selected={selected} onSelect={setSelected} domain={domain} setDomain={setDomain} maxKpa={maxKpa} />
          <p className="mapv-foot">
            <button className="linkish" onClick={() => downloadCsv(v.points.filter((p) => !hidden.has(p.investigation)))}>
              Download CSV
            </button>
            {" · "}
            {v.points.length} test points from {v.placedRuns} of {v.runs} runs · {v.unplaced.o2 + v.unplaced.pressure + v.unplaced.both} without both O₂ and pressure · {v.unplaced.notAtmosphere} with O₂ only in the burner stream · no partial-gravity tests
          </p>
        </div>

        <aside className="card mapv-side">
          {sel ? (
            <div className="panel-sec">
              <div className="mapv-sel-head">
                <ShapeSwatch id={sel.investigation} size={14} />
                <b>{invName(sel.investigation)}</b>
                <button className="btn btn-quiet btn-s" style={{ marginLeft: "auto" }} onClick={() => setSelected(null)}>
                  Close
                </button>
              </div>
              <p className="muted small">{INVESTIGATIONS.get(sel.investigation)?.title}</p>
              <dl className="kv">
                <dt>Fuel</dt>
                <dd>{sel.fuel ?? "not stated"}</dd>
                <dt>Oxygen</dt>
                <dd className="num">{(sel.o2 * 100).toFixed(1)} %</dd>
                <dt>Pressure</dt>
                <dd className="num">
                  {sel.kpa.toFixed(1)} kPa · {(sel.kpa / 6.894757).toFixed(1)} psi
                </dd>
                {HABITAT_LIST.map((x) => (
                  <div key={x.id} style={{ display: "contents" }}>
                    <dt>{x.short}</dt>
                    <dd className="num">{offsetText(offset(sel.o2, sel.kpa, x))}</dd>
                  </div>
                ))}
              </dl>
              <h3>
                {sel.runs.length} run{sel.runs.length === 1 ? "" : "s"}
              </h3>
              <div className="run-chips">
                {sel.runs.map((id) => (
                  <button key={id} className="linkish small" onClick={() => panel.open({ kind: "run", id })}>
                    {runLabel(id)}
                  </button>
                ))}
              </div>
            </div>
          ) : (
            <div className="panel-sec">
              <h3>Nearest tests</h3>
              <div className="seg" role="group" aria-label="Atmosphere">
                {HABITAT_LIST.map((x) => (
                  <button key={x.id} aria-pressed={near === x.id} onClick={() => setNear(x.id)}>
                    {x.short}
                  </button>
                ))}
              </div>
              <p className="muted small">{h.sub}</p>
              <ol className="near">
                {nearest.map(({ p, o }) => (
                  <li key={p.key}>
                    <button
                      onClick={() => {
                        setSelected(p.key);
                        center(p.o2, p.kpa);
                      }}
                    >
                      <span className="dotc" style={{ background: invColor(p.investigation) }} aria-hidden="true" />
                      <span className="t">
                        <b>{invName(p.investigation)}</b> {p.fuel ?? ""}
                        <span className="muted">
                          {(p.o2 * 100).toFixed(1)} % · {p.kpa.toFixed(0)} kPa · {p.runs.length} run{p.runs.length === 1 ? "" : "s"}
                        </span>
                      </span>
                      <span className={o.inside ? "tag tag-ok" : "v"}>{o.inside ? "inside" : offsetText(o)}</span>
                    </button>
                  </li>
                ))}
              </ol>
            </div>
          )}
        </aside>
      </div>
    </div>
  );
}
