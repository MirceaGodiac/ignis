// T9: the challenge's two direct questions over K1.13 `ConditionsTable`:
//   1. How near does each investigation's tested envelope come to the Moon/rover, suit and ISS
//      atmospheres? (reached, outside by how much, or not decidable; never extrapolated)
//   2. Which conditions did each run record? (value with its original's locator, or an explicit gap)
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { OpenOriginal, locatorSource } from "../shell/OpenOriginal";
import type { ConditionCell, ConditionKey, ConditionsTable, Measured, SourceLocator } from "../contracts/generated";
import { axesOf, fitText, gravityStatus, HABITAT_LIST, PARTIAL_G, PSI_PA, rankFor, type EnvelopeAxes, type Fit, type Slot } from "../research/habitats";
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

export function Conditions() {
  const { client, researchMode, synthetic } = useData();
  const q = useQuery({
    queryKey: [researchMode, "conditions"],
    queryFn: () => client.conditions({ schema_version: "dataset.v2", include_envelopes: true, include_unknown: true }),
    retry: false,
  });
  return (
    <div className="view">
      {q.isPending && <p className="muted">Loading conditions…</p>}
      {q.error && (
        <>
          <ErrorNote error={q.error} title="The conditions service did not answer" />
          <Gap title="Conditions come from the PSI experiment tables and the documents">The service behind this view (conditions query, R9) and its route (M11) are not on this server yet.</Gap>
        </>
      )}
      {q.data && <ConditionsBody t={q.data} synthetic={synthetic} />}
    </div>
  );
}

function ConditionsBody({ t, synthetic }: { t: ConditionsTable; synthetic: boolean }) {
  const axes = (t.envelopes ?? []).map(axesOf);
  return (
    <>
      <div className="toolbar">
        <OriginMark origin={t.origin} pageSynthetic={synthetic} />
        {t.corpus_snapshot && <span className="muted small">corpus {t.corpus_snapshot}</span>}
      </div>
      <HabitatRanking axes={axes} />
      <ConditionsMatrix t={t} />
    </>
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
    case "missing":
      return "missing";
    case "not_applicable":
      return "n/a";
  }
}

function measuredText(m: Measured): string {
  return typeof m.raw_value === "number" ? `${m.raw_value}${m.raw_unit ? ` ${m.raw_unit}` : ""}` : m.raw_value;
}

function locText(l: SourceLocator | null | undefined): string {
  if (!l) return "no locator";
  if (l.path) return `${l.path}${l.row != null ? `, row ${l.row}` : ""}${l.column ? `, column “${l.column}”` : ""}`;
  if (l.page != null) return `page ${l.page}${l.bbox ? ", region" : ""}`;
  if (l.pts_s != null) return `video at ${l.pts_s} s`;
  return "asset";
}

export function ConditionsMatrix({ t }: { t: ConditionsTable }) {
  const [picked, setPicked] = useState<Picked | null>(null);
  return (
    <section className="panel-sec" aria-labelledby="h-matrix">
      <h2 id="h-matrix" style={{ fontSize: 15, margin: "16px 0 0" }}>
        Conditions per run
      </h2>
      <p className="muted small">
        Inputs, not outcomes. Each value keeps its original's locator; gaps are marked, never filled. {t.total_count != null && `${t.rows.length} of ${t.total_count} runs.`}
      </p>
      <div className="tbl-wrap">
        <table className="tbl">
          <thead>
            <tr>
              <th>Investigation</th>
              <th>Run</th>
              {t.keys.map((k) => (
                <th key={k}>{KEY_LABEL[k]}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {t.rows.map((r, i) => {
              // A run whose identity is not resolved has no id; it is listed, never merged with another.
              const run = r.run.run_id ?? `unresolved #${i + 1}`;
              return (
                <tr key={`${r.investigation_id}/${run}`}>
                  <td className="dim">{r.investigation_id}</td>
                  <td>{r.run.run_id ? <code>{run}</code> : <span className="tag tag-line">{run}</span>}</td>
                  {t.keys.map((k) => {
                    const c = r.cells.find((x) => x.key === k);
                    if (!c)
                      return (
                        <td key={k} className="dim">
                          —
                        </td>
                      );
                    const on = !!picked && picked.run === run && picked.cell.key === k;
                    return (
                      <td key={k}>
                        <button
                          className={`cond cond-${c.status}`}
                          aria-pressed={on}
                          onClick={() => setPicked(on ? null : { run, investigation: r.investigation_id, cell: c })}
                          title={c.status === "value" ? locText(c.value.locator) : "reason" in c ? c.reason : undefined}
                        >
                          {cellText(c)}
                        </button>
                      </td>
                    );
                  })}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      {picked && <CellDetail p={picked} />}
    </section>
  );
}

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
              <dd>{m.flags!.join(", ")}</dd>
            </>
          )}
        </dl>
      ))}
    </div>
  );
}
