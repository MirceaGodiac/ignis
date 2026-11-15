// T9: the Conditions page (by crew atmosphere, the atmosphere map) and the conditions matrix that
// answers can embed. Both read K1.13 `ConditionsTable`; nothing is extrapolated.
import { memo, useCallback, useEffect, useRef, useState } from "react";
import type { ConditionCell, ConditionKey, ConditionsRow, ConditionsTable, Measured, SourceLocator } from "../contracts/generated";
import { flagTags, quotedText } from "../research/conditionCells";
import { rowWindow } from "../research/windowing";
import { href } from "../shell/router";
import { MapView } from "./MapView";
import { ByAtmosphere } from "./ByAtmosphere";
import { EvidenceClassTag, words } from "../ui/common";
import { OpenOriginal, locatorSource } from "../shell/OpenOriginal";

const KEY_LABEL: Record<ConditionKey, string> = {
  oxygen_mole_fraction: "O₂",
  pressure: "Pressure",
  flow_velocity: "Flow",
  gravity_level: "Gravity",
  ambient_temperature: "Temperature",
  fuel: "Fuel",
  diluent: "Diluent",
  sample_geometry: "Geometry",
  hardware: "Hardware",
  diagnostics: "Diagnostics",
};

export type ConditionsTab = "habitats" | "map";
const TABS: [ConditionsTab, string][] = [
  ["habitats", "By atmosphere"],
  ["map", "Atmosphere map"],
];

/** One page for the conditions table (K1.13): its three readings share one service and one header. */
export function ConditionsPage({ tab, habitat, inv }: { tab: ConditionsTab; habitat?: string | null; inv?: string | null }) {
  return (
    <div className="view">
      <div className="page-title">
        <h1>Conditions</h1>
        <nav className="tabs" aria-label="Conditions views">
          {TABS.map(([t, label]) => (
            <a key={t} href={href("conditions", null, t === "habitats" ? undefined : { tab: t })} aria-current={t === tab ? "page" : undefined}>
              {label}
            </a>
          ))}
        </nav>
      </div>
      {tab === "map" ? <MapView focus={inv} /> : <ByAtmosphere initial={habitat} />}
    </div>
  );
}


// ---------------------------------------------------------------- conditions matrix

type Picked = { run: string; investigation: string; cell: ConditionCell };

function cellText(c: ConditionCell): string {
  switch (c.status) {
    case "value":
      return measuredText(c.value);
    case "conflicting":
      return `conflict · ${c.candidates.length}`;
    case "missing": {
      const q = quotedText(c.reason);
      return q ? `“${q.length > 28 ? `${q.slice(0, 26)}…` : q}”` : "missing";
    }
    case "not_applicable":
      return "n/a";
  }
}

function measuredText(m: Measured): string {
  // A dimensionless raw unit ("1") is not printed: "0.21", not "0.21 1".
  return typeof m.raw_value === "number" ? `${m.raw_value}${m.raw_unit && m.raw_unit !== "1" ? ` ${m.raw_unit}` : ""}` : m.raw_value;
}

function locText(l: SourceLocator | null | undefined): string {
  if (!l) return "no locator";
  if (l.path) return `${l.path}${l.row != null ? `, row ${l.row}` : ""}${l.column ? `, column “${l.column}”` : ""}`;
  if (l.page != null) return `page ${l.page}${l.bbox ? ", region" : ""}`;
  if (l.pts_s != null) return `video at ${l.pts_s} s`;
  return "asset";
}

/** From this many rows, the matrix scrolls in its own frame and renders only the rows in view. */
const WINDOW_FROM = 300;
const ROW_HEIGHT = 33; // .tbl td height 32px + 1px border; measured below once rows exist

export function ConditionsMatrix({ t }: { t: ConditionsTable }) {
  const [picked, setPicked] = useState<Picked | null>(null);
  const windowed = t.rows.length >= WINDOW_FROM;
  const frame = useRef<HTMLDivElement>(null);
  const [view, setView] = useState({ top: 0, height: 600, rowH: ROW_HEIGHT });
  useEffect(() => {
    const el = frame.current;
    if (!el || !windowed) return;
    let raf = 0;
    const read = () => {
      const row = el.querySelector<HTMLTableRowElement>("tbody tr[aria-rowindex]");
      const rowH = row?.getBoundingClientRect().height || ROW_HEIGHT;
      setView({ top: el.scrollTop, height: el.clientHeight, rowH });
    };
    const onScroll = () => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(read);
    };
    read();
    el.addEventListener("scroll", onScroll, { passive: true });
    const ro = new ResizeObserver(onScroll);
    ro.observe(el);
    return () => {
      cancelAnimationFrame(raf);
      el.removeEventListener("scroll", onScroll);
      ro.disconnect();
    };
  }, [windowed]);
  const w = windowed ? rowWindow(view.top, view.height, view.rowH, t.rows.length) : { start: 0, end: t.rows.length, top: 0, bottom: 0 };
  // Stable across renders, so a click re-renders only the rows whose selection changed (memo below).
  const pick = useCallback((p: Picked) => setPicked((cur) => (cur && cur.run === p.run && cur.cell.key === p.cell.key ? null : p)), []);
  return (
    <div className="panel-sec">
      {t.total_count != null && (
        <p className="muted small">
          {t.rows.length} of {t.total_count} runs
        </p>
      )}
      <div className="tbl-wrap" ref={frame} style={windowed ? { maxHeight: "70vh", overflowY: "auto" } : undefined}>
        <table className="tbl" aria-rowcount={t.rows.length + 1}>
          <thead>
            <tr aria-rowindex={1}>
              <th>Investigation</th>
              <th>Run</th>
              {t.keys.map((k) => (
                <th key={k}>{KEY_LABEL[k]}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {w.top > 0 && <tr aria-hidden="true" style={{ height: w.top }} />}
            {t.rows.slice(w.start, w.end).map((r, j) => {
              const i = w.start + j;
              // A run whose identity is not resolved has no id; it is listed, never merged with another.
              const run = r.run.run_id ?? `unresolved run (row ${i + 1})`;
              return <MatrixRow key={`${r.investigation_id}/${run}`} index={i + 2} row={r} run={run} keys={t.keys} pickedKey={picked?.run === run ? picked.cell.key : null} onPick={pick} />;
            })}
            {w.bottom > 0 && <tr aria-hidden="true" style={{ height: w.bottom }} />}
          </tbody>
        </table>
      </div>
      {picked && <CellDetail p={picked} />}
    </div>
  );
}

const MatrixRow = memo(function MatrixRow({ index, row: r, run, keys, pickedKey, onPick }: { index: number; row: ConditionsRow; run: string; keys: ConditionKey[]; pickedKey: ConditionKey | null; onPick: (p: Picked) => void }) {
  return (
    <tr aria-rowindex={index}>
      <td className="dim">{r.investigation_id}</td>
      <td>{r.run.run_id ? <code>{run}</code> : <span className="tag tag-line">{run}</span>}</td>
      {keys.map((k) => {
        const c = r.cells.find((x) => x.key === k);
        if (!c)
          return (
            <td key={k} className="dim">
              —
            </td>
          );
        return (
          <td key={k}>
            <button
              className={`cond cond-${c.status}`}
              aria-pressed={pickedKey === k}
              onClick={() => onPick({ run, investigation: r.investigation_id, cell: c })}
              title={c.status === "value" ? locText(c.value.locator) : "reason" in c ? c.reason : undefined}
            >
              {cellText(c)}
              {c.status === "value" &&
                flagTags(c.value.flags)
                  .filter((f) => f.inline)
                  .map((f, i) => (
                  <span key={i} className={`tag ${f.notAtmosphere ? "tag-warn" : "tag-line"}`} title={f.title} style={{ marginLeft: 4, height: 16 }}>
                    {f.label}
                  </span>
                ))}
            </button>
          </td>
        );
      })}
    </tr>
  );
});

function CellDetail({ p }: { p: Picked }) {
  const c = p.cell;
  const values = c.status === "value" ? [c.value] : c.status === "conflicting" ? c.candidates : [];
  return (
    <div className="note" role="region" aria-label="Selected condition">
      <b>
        {KEY_LABEL[c.key]} · run <code>{p.run}</code> · {words(c.status)}
      </b>
      {"reason" in c && <p>{c.reason}</p>}
      {values.map((m, i) => (
        <dl key={i} className="kv" style={{ marginTop: 6 }}>
          <dt>{c.status === "conflicting" ? `candidate ${i + 1}` : "value"}</dt>
          <dd>
            {measuredText(m)}
            {m.value_si != null && m.unit_si && (
              <span className="dim">
                {" "}
                = {m.value_si}
                {m.unit_si === "1" ? " (SI, fraction)" : ` ${m.unit_si} (SI)`}
              </span>
            )}
          </dd>
          {m.evidence_class && (
            <>
              <dt>evidence</dt>
              <dd>
                <EvidenceClassTag cls={m.evidence_class} />
              </dd>
            </>
          )}
          <dt>from</dt>
          <dd>
            {locText(m.locator)}{" "}
            {m.locator?.asset_sha256 && <OpenOriginal sourceId={locatorSource(m.locator)} loc={m.locator} title={m.locator.path ?? `${p.investigation} ${KEY_LABEL[c.key]}`} />}
          </dd>
          {m.uncertainty_si != null && (
            <>
              <dt>uncertainty</dt>
              <dd>
                ± {m.uncertainty_si} {m.unit_si}
              </dd>
            </>
          )}
          {(m.flags ?? []).length > 0 && (
            <>
              <dt>flags</dt>
              <dd>
                {flagTags(m.flags).map((f, i) => (
                  <div key={i}>{f.title}</div>
                ))}
              </dd>
            </>
          )}
        </dl>
      ))}
    </div>
  );
}
