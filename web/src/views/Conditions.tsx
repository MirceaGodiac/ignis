// T9: the challenge's two direct questions over K1.13 `ConditionsTable`:
//   1. How near does each investigation's tested envelope come to the Moon/rover, suit and ISS
//      atmospheres? (reached, outside by how much, or not decidable; never extrapolated)
//   2. Which conditions did each run record? (value with its original's locator, or an explicit gap)
import { memo, useCallback, useEffect, useRef, useState } from "react";
import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import type { ConditionCell, ConditionKey, ConditionsRow, ConditionsTable, Measured, SourceLocator } from "../contracts/generated";
import { axesOf, fitText, gravityStatus, HABITAT_LIST, habitatFilters, PARTIAL_G, PSI_PA, rankFor, type HabitatId, type EnvelopeAxes, type Fit, type Slot } from "../research/habitats";
import { flagTags, quotedText } from "../research/conditionCells";
import { rowWindow } from "../research/windowing";
import { OpenOriginal, locatorSource } from "../shell/OpenOriginal";
import { useData } from "../shell/data";
import { ErrorNote, EvidenceClassTag, Gap, OriginMark, words } from "../ui/common";

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

const PAGE_SIZE = 200;

type Preset = "all" | HabitatId;
const PRESET_LABEL: Record<Preset, string> = { all: "All runs", moon: "Moon base and rover air", suit: "Suit air", iss: "ISS and Earth air" };

export function Conditions() {
  const { client, researchMode, synthetic } = useData();
  const [preset, setPreset] = useState<Preset>("all");
  const [includeUnknown, setIncludeUnknown] = useState(false);
  const habitat = HABITAT_LIST.find((h) => h.id === preset);
  const filters = habitat ? habitatFilters(habitat) : [];
  // Envelopes (the habitat ranking) cover every investigation; the matrix pages through the filtered runs.
  const envelopes = useQuery({
    queryKey: [researchMode, "conditions", "envelopes"],
    queryFn: () => client.conditions({ schema_version: "dataset.v2", include_envelopes: true, page_size: 1 }),
    retry: false,
  });
  const runs = useInfiniteQuery({
    queryKey: [researchMode, "conditions", "runs", preset, includeUnknown],
    queryFn: ({ pageParam }) =>
      client.conditions({ schema_version: "dataset.v2", filters, include_unknown: preset === "all" ? true : includeUnknown, page_size: PAGE_SIZE, cursor: pageParam ?? null }),
    initialPageParam: null as string | null,
    getNextPageParam: (last) => last.next_cursor ?? undefined,
    retry: false,
  });
  const first = runs.data?.pages[0];
  const table: ConditionsTable | undefined = first && { ...first, rows: runs.data!.pages.flatMap((p) => p.rows) };
  return (
    <div className="view">
      {envelopes.isPending && <p className="muted">Loading conditions…</p>}
      {envelopes.error && (
        <>
          <ErrorNote error={envelopes.error} title="The conditions service did not answer" />
          <Gap title="Conditions come from the PSI experiment tables and the documents">The conditions query (R9) and its route (M11) are not on this server yet.</Gap>
        </>
      )}
      {envelopes.data && (
        <>
          <div className="toolbar">
            <OriginMark origin={envelopes.data.origin} pageSynthetic={synthetic} />
            {envelopes.data.corpus_snapshot && <span className="muted small">corpus {envelopes.data.corpus_snapshot}</span>}
          </div>
          <HabitatRanking axes={(envelopes.data.envelopes ?? []).map(axesOf)} />
        </>
      )}
      <section className="panel-sec" aria-labelledby="h-matrix">
        <h2 id="h-matrix" style={{ fontSize: 15, margin: "16px 0 0" }}>
          Conditions per run
        </h2>
        <div className="toolbar" role="group" aria-label="Runs to show">
          {(Object.keys(PRESET_LABEL) as Preset[]).map((p) => (
            <button key={p} className={`btn btn-s${p === preset ? " btn-primary" : ""}`} aria-pressed={p === preset} onClick={() => setPreset(p)}>
              {PRESET_LABEL[p]}
            </button>
          ))}
          {preset !== "all" && (
            <label className="small" style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
              <input type="checkbox" checked={includeUnknown} onChange={(e) => setIncludeUnknown(e.target.checked)} /> include runs where O₂ or pressure is not stated
            </label>
          )}
        </div>
        {habitat && (
          <p className="muted small">
            Runs whose recorded O₂ and pressure lie in {habitat.label.toLowerCase()} ({habitat.sub}). A burner-stream O₂ is never counted as the atmosphere.
          </p>
        )}
        {runs.isPending && <p className="muted small">Loading runs…</p>}
        {runs.error && <ErrorNote error={runs.error} title="The runs could not be loaded" />}
        {table && <ConditionsMatrix t={table} />}
        {runs.hasNextPage && (
          <button className="btn btn-s" disabled={runs.isFetchingNextPage} onClick={() => void runs.fetchNextPage()}>
            {runs.isFetchingNextPage ? "Loading…" : `Load ${PAGE_SIZE} more`}
          </button>
        )}
      </section>
    </div>
  );
}

// ---------------------------------------------------------------- habitat relevance

const pct = (v: number) => `${+(v * 100).toFixed(1)} %`;
const psi = (pa: number) => `${+(pa / PSI_PA).toFixed(1)} psi`;
const span = (s: Slot, f: (v: number) => string) => (s.kind === "missing" ? null : s.range.min === s.range.max ? f(s.range.min) : `${f(s.range.min)} – ${f(s.range.max)}`);

function FitCell({ fit }: { fit: Fit }) {
  if (fit.status === "reached") return <span className="tag tag-ok">within envelope</span>;
  if (fit.status === "unknown")
    return (
      <span className="tag tag-line" title={fitText(fit)}>
        not decidable
      </span>
    );
  return (
    <span title={fitText(fit)}>
      <span className="tag tag-warn">outside</span>
      <span className="dim small num" style={{ display: "block", marginTop: 2 }}>
        {fitText(fit).replace("outside tested envelope: ", "")}
      </span>
    </span>
  );
}

export function HabitatRanking({ axes }: { axes: EnvelopeAxes[] }) {
  const moon = HABITAT_LIST[0];
  const ranked = rankFor(axes, moon);
  return (
    <section className="panel-sec" aria-labelledby="h-habitats">
      <h2 id="h-habitats" style={{ fontSize: 15, margin: "8px 0 0" }}>
        Habitat relevance
      </h2>
      <p className="muted small">
        Each investigation's tested range of O₂ and pressure, compared with the crew atmospheres in the challenge brief. Sorted by nearness to the Moon base and rover atmosphere.
      </p>
      {ranked.length === 0 ? (
        <Gap title="No tested envelopes">The conditions service returned no per-investigation envelope.</Gap>
      ) : (
        <div className="tbl-wrap">
          <table className="tbl">
            <thead>
              <tr>
                <th>Investigation</th>
                <th className="r">Runs</th>
                <th>Tested O₂ · pressure</th>
                <th>Gravity</th>
                {HABITAT_LIST.map((h) => (
                  <th key={h.id} title={h.sub} className="wrap">
                    {h.short}
                    <div className="dim" style={{ fontWeight: 400 }}>
                      {h.sub}
                    </div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {ranked.map(({ axes: a }) => {
                const g = gravityStatus(a);
                return (
                  <tr key={a.investigation_id}>
                    <td>
                      <code>{a.investigation_id}</code>
                    </td>
                    <td className="r num">{a.runs}</td>
                    <td className="num">
                      <span title={a.o2.kind === "range" ? `${a.o2.range.runs} of ${a.runs} runs` : undefined}>{span(a.o2, pct) ?? <Missing reason={`O₂ ${(a.o2 as { reason: string }).reason}`} />}</span>
                      <span className="dim"> · </span>
                      <span title={a.pa.kind === "range" ? `${a.pa.range.runs} of ${a.runs} runs` : undefined}>{span(a.pa, psi) ?? <Missing reason={`pressure ${(a.pa as { reason: string }).reason}`} />}</span>
                    </td>
                    <td>
                      {g.status === "partial_tested" ? (
                        <span className="tag tag-ok">partial gravity tested</span>
                      ) : g.status === "no_partial_data" ? (
                        <span className="tag tag-warn" title={`tested ${g.tested.min}–${g.tested.max} g`}>
                          no partial-gravity data
                        </span>
                      ) : (
                        <Missing reason={g.reason} />
                      )}
                    </td>
                    {HABITAT_LIST.map((h) => (
                      <td key={h.id} className="wrap">
                        <FitCell fit={rankFor([a], h)[0].fit} />
                      </td>
                    ))}
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
      <p className="muted small">
        "Within envelope": the tested O₂ and pressure ranges reach the atmosphere (to 0.5 points of O₂ and 1 % of pressure). "Outside" says how far the tests would have to move; nothing is
        extrapolated. Partial gravity means a tested level between {PARTIAL_G[0]} g and {PARTIAL_G[1]} g (Moon 0.17 g, Mars 0.38 g). Hover a tested range to see how many runs gave it.
      </p>
    </section>
  );
}

function Missing({ reason }: { reason: string }) {
  return (
    <span className="tag tag-line" title={reason}>
      not stated
    </span>
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
      <p className="muted small">
        Inputs, not outcomes. Each value keeps its original's locator; gaps are marked, never filled. {t.total_count != null && `${t.rows.length} of ${t.total_count} runs.`}
      </p>
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
