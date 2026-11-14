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
import { FamilyLegend } from "./FamilyLegend";
import { HABITAT_LIST, type Habitat, type HabitatId } from "../research/habitats";
import { offset } from "../research/testPoints";
import { invName } from "../data/investigations";

type Col = "id" | "inv" | "date" | "family" | "fuel" | "o2" | "pressure" | "flow";
const cond = (r: ExperimentSummary, k: string) => r.conditions[k]?.value_si ?? null;

/**
 * Ambient oxygen as each family's records name it: `x_o2` (FLEX, BASS), `x_o2_ambient` (ACME normal
 * flames). An inverse flame burns in a fuel atmosphere: its oxygen is in the burner stream
 * (`x_o2_injected`), shown as such and never as the atmosphere.
 */
function oxygen(r: ExperimentSummary): { v: number | null; burner: boolean } {
  const amb = cond(r, "x_o2") ?? cond(r, "x_o2_ambient") ?? cond(r, "oxygen_mole_fraction");
  if (amb != null) return { v: amb, burner: false };
  const inj = cond(r, "x_o2_injected");
  return { v: inj, burner: inj != null };
}

/** Flow as recorded: air velocity (BASS, m/s) or injected mass flow (ACME, kg/s, shown in mg/s). */
function flow(r: ExperimentSummary): { v: number; text: string } | null {
  const u = cond(r, "flow_velocity");
  if (u != null) return { v: u, text: `${fmtNum(u * 100)} cm/s` };
  const m = cond(r, "injected_mass_flow");
  if (m != null) return { v: m, text: `${fmtNum(m * 1e6)} mg/s` };
  return null;
}

/**
 * The run's own name inside its investigation: the id without the investigation, the family token
 * and the day ("PSI-69:20CAL08:2009-03-05" → "20CAL08", "PSI-159:acme_inverse:21196A1" → "21196A1").
 */
const shortRun = (r: ExperimentSummary) =>
  r.id
    .split(":")
    .slice(1)
    .filter((p) => p !== r.family && !/^\d{4}-\d{2}-\d{2}$/.test(p))
    .join(" ") || r.id;

/**
 * NASA's own name for the run, from the record's aliases: the FLEX test sheet number (FLEX-229), the
 * CFI-G table test (21196 A1), the BASS as-run and test numbers (run 47 · test 82a) or test number (B1).
 * Falls back to the id without investigation, family and day.
 */
function runName(r: ExperimentSummary): string {
  const a = (scheme: string) => r.aliases?.find((x) => x.scheme === scheme)?.name;
  const asRun = a("as_run_number");
  return a("testsheet_header") ?? a("file_flex_number") ?? a("table_test") ?? (asRun ? `run ${asRun} · test ${a("test_number") ?? "?"}` : a("test_number")) ?? shortRun(r);
}

/** The test day from the run's session ("PSI-69:day:2009-03-05"). */
const testDate = (r: ExperimentSummary) => r.session_id?.match(/(\d{4}-\d{2}-\d{2})$/)?.[1] ?? null;

const val: Record<Col, (r: ExperimentSummary) => string | number | null> = {
  id: (r) => runName(r),
  inv: (r) => r.investigation_id,
  date: (r) => testDate(r),
  family: (r) => r.family,
  fuel: (r) => r.fuel_or_material,
  o2: (r) => oxygen(r).v,
  pressure: (r) => cond(r, "pressure"),
  flow: (r) => flow(r)?.v ?? null,
};

/** A run is in an atmosphere when its recorded ambient O2 and pressure lie in it (burner-stream O2 never counts). */
function inAtmosphere(r: ExperimentSummary, h: Habitat): boolean {
  const o2 = oxygen(r);
  const pa = cond(r, "pressure");
  return !o2.burner && o2.v != null && pa != null && offset(o2.v, pa / 1000, h).inside;
}

export function Runs({ initialFamily, initialHabitat, initialInv }: { initialFamily: string | null; initialHabitat?: string | null; initialInv?: string | null }) {
  const { experiments, synthetic } = useData();
  const panel = usePanel();
  const cmp = useCompare();
  const [family, setFamily] = useState<string | null>(initialFamily);
  const [text, setText] = useState("");
  const [habitat, setHabitat] = useState<HabitatId | null>(() => (HABITAT_LIST.some((h) => h.id === initialHabitat) ? (initialHabitat as HabitatId) : null));
  const [inv, setInv] = useState<string | null>(initialInv ?? null);
  const [sort, setSort] = useState<{ col: Col; dir: 1 | -1 }>({ col: "family", dir: 1 });
  const runs = experiments.data?.experiments ?? [];
  const counts = useMemo(() => runs.reduce<Record<string, number>>((m, r) => ((m[r.family] = (m[r.family] ?? 0) + 1), m), {}), [runs]);
  const rows = useMemo(() => {
    const t = text.trim().toLowerCase();
    const h = HABITAT_LIST.find((x) => x.id === habitat);
    const f = runs.filter(
      (r) =>
        (!family || r.family === family) &&
        (!inv || r.investigation_id === inv) &&
        (!h || inAtmosphere(r, h)) &&
        (!t || `${r.id} ${runName(r)} ${r.fuel_or_material ?? ""} ${r.configuration ?? ""} ${r.investigation_id} ${invName(r.investigation_id)}`.toLowerCase().includes(t)),
    );
    // The chosen column first, then family, investigation, day and run, so equal values keep a reading order.
    const keys: Col[] = [sort.col, ...(["family", "inv", "date", "id"] as Col[]).filter((c) => c !== sort.col)];
    return [...f].sort((a, b) => {
      for (const [i, k] of keys.entries()) {
        const x = val[k](a);
        const y = val[k](b);
        if (x === y || (x == null && y == null)) continue;
        if (x == null) return 1; // missing values sort last, whatever the direction
        if (y == null) return -1;
        return (x < y ? -1 : 1) * (i === 0 ? sort.dir : 1);
      }
      return 0;
    });
  }, [runs, family, text, sort, habitat, inv]);
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
        <input id="run-filter" className="field" placeholder="Filter runs" value={text} onChange={(e) => setText(e.target.value)} style={{ width: 240 }} />
        <span className="muted" style={{ marginLeft: "auto", fontSize: 12 }}>
          {rows.length} of {runs.length}
        </span>
      </div>
      <div className="toolbar">
        <div className="seg" role="group" aria-label="Atmosphere">
          <button aria-pressed={habitat === null} onClick={() => setHabitat(null)}>
            Any atmosphere
          </button>
          {HABITAT_LIST.map((h) => (
            <button key={h.id} aria-pressed={habitat === h.id} onClick={() => setHabitat(h.id)} title={h.sub}>
              {h.label}
            </button>
          ))}
        </div>
        {habitat && <span className="muted small">{HABITAT_LIST.find((h) => h.id === habitat)!.sub} · recorded ambient O₂ and pressure</span>}
        {inv && (
          <button className="chip-t" onClick={() => setInv(null)} aria-label={`Remove the filter ${invName(inv)}`}>
            {invName(inv)} ×
          </button>
        )}
      </div>
      {experiments.error && !experiments.data && (
        <div style={{ display: "grid", gap: 8, justifyItems: "start" }}>
          <ErrorNote error={experiments.error} title="Runs could not be loaded" />
          <button className="btn btn-s" onClick={() => void experiments.refetch()}>
            Try again
          </button>
        </div>
      )}
      {experiments.isPending && <p className="muted">Loading runs…</p>}
      <div className="tbl-wrap">
        <table className="tbl">
          <thead>
            <tr>
              <th style={{ width: 36 }}>
                <span className="sr-only">Compare</span>
              </th>
              <Th col="family">Family</Th>
              <Th col="inv" hide>
                Investigation
              </Th>
              <Th col="date">Date</Th>
              <Th col="fuel" hide>
                Fuel or material
              </Th>
              <Th col="id">Run</Th>
              <Th col="o2" r>
                O₂ %
              </Th>
              <Th col="pressure" r>
                Pressure kPa
              </Th>
              <Th col="flow" r hide>
                Flow
              </Th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => {
              const o2 = oxygen(r);
              const fl = flow(r);
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
                    <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
                      <FamilyMark family={r.family} size={9} /> {r.family.replace("_", " ").replace("acme ", "ACME ").replace("flex", "FLEX").replace("bass", "BASS")}
                    </span>
                  </td>
                  <td className="narrow-hide dim">
                    {r.investigation_id}
                    {r.configuration ? ` · ${r.configuration.replace(/_/g, " ")}` : ""}
                  </td>
                  <td className="num">{testDate(r) ?? <span className="dim">—</span>}</td>
                  <td className="narrow-hide">{r.fuel_or_material ?? <span className="dim">unknown</span>}</td>
                  <td>
                    <span className="cell-id" title={r.id}>
                      {runName(r)}
                    </span>{" "}
                    {r.synthetic && !synthetic && <Synthetic />}
                  </td>
                  <td className="r num" title={o2.burner ? "Oxygen of the burner stream; the atmosphere is the fuel" : undefined}>
                    {o2.burner && <span className="dim small">burner </span>}
                    {num(o2.v, 100, 1)}
                  </td>
                  <td className="r num">{num(cond(r, "pressure"), 0.001, 1)}</td>
                  <td className="r num narrow-hide">{fl ? fl.text : <span className="dim">—</span>}</td>
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
                  No runs on this server yet.
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
