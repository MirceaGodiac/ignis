// Side-by-side conditions for the selected runs. The server decides whether a numeric comparison
// is permitted for a quantity; the verdict appears only once a quantity is chosen, so a green box
// never stands over a table that holds no numbers.
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { Quantity } from "../contracts/generated";
import { call, type CompareResult } from "../api";
import { fmtGravity, fmtMeasured } from "../format";
import { asRefusal } from "../hooks";
import { useCompare } from "../shell/compare";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { FamilyMark, IconClose } from "../shell/icons";
import { href } from "../shell/router";
import { RefusalNote, Synthetic, words } from "../ui/common";

const QUANTITIES: Quantity[] = ["liquid_diameter", "visible_flame_diameter", "uv_flame_diameter", "front_position", "front_contour", "pressure", "temperature"];

export function Compare() {
  const { origin, synthetic } = useData();
  const cmp = useCompare();
  const panel = usePanel();
  const [quantity, setQuantity] = useState<string>("");
  const ids = cmp.ids;
  const enabled = ids.length >= 2;
  const res = useQuery({
    queryKey: [origin, "compare", ids, quantity],
    queryFn: ({ signal }) => call<CompareResult>(origin, "/compare", { json: { run_ids: ids, ...(quantity ? { quantity } : {}) }, signal }),
    enabled,
    retry: false,
  });
  const r = res.data;
  const err = asRefusal(res.error);

  return (
    <div className="view">
      <div className="view-head">
        <p className="muted small">Conditions as recorded (SI). Differences describe the runs; they are not causal effects.</p>
        {enabled && (
          <div className="toolbar">
            <label className="toolbar" style={{ fontSize: 13 }}>
              Check a quantity
              <select className="field" value={quantity} onChange={(e) => setQuantity(e.target.value)}>
                <option value="">Conditions only</option>
                {QUANTITIES.map((q) => (
                  <option key={q} value={q}>
                    {words(q)}
                  </option>
                ))}
              </select>
            </label>
            <button className="btn btn-quiet" onClick={cmp.clear}>
              Clear selection
            </button>
          </div>
        )}
      </div>

      {!enabled && (
        <div className="empty">
          <b>Select at least two runs</b>
          <p>Tick runs in the run index, or open a run on the map and choose “Add to compare”. {ids.length === 1 && `One run is selected: ${ids[0]}.`}</p>
          <a className="btn" href={href("runs")}>
            Open the run index
          </a>
        </div>
      )}

      {err && <RefusalNote refusal={err} />}

      {r && (
        <>
          {!quantity ? (
            <div className="note" role="note">
              <b>Choose a quantity to check a numeric comparison</b>
              <p>{r.numeric_comparison.note}</p>
            </div>
          ) : r.numeric_comparison.allowed ? (
            <div className="note note-ok" role="status">
              <b>
                Numeric comparison of {words(quantity)} is permitted <span className="tag">{words(r.comparison_kind)}</span>
              </b>
              <p>{r.numeric_comparison.note}</p>
            </div>
          ) : (
            <div className="panel-sec" role="status">
              {r.numeric_comparison.refusals.map((x) => (
                <RefusalNote key={x.code} refusal={x} title={`Numeric comparison of ${words(quantity)} refused`} next="Pick runs of one family with the same measured quantity." />
              ))}
              <p className="muted" style={{ fontSize: 12 }}>{r.numeric_comparison.note}</p>
            </div>
          )}
          <div className="tbl-wrap">
            <table className="tbl">
              <caption>Conditions side by side (SI)</caption>
              <thead>
                <tr>
                  <th>Field</th>
                  {r.columns.map((c) => (
                    <th key={c.id}>
                      <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
                        <FamilyMark family={c.family} size={9} />
                        <button className="linkish cell-id" onClick={() => panel.open({ kind: "run", id: c.id })}>
                          {c.id}
                        </button>
                        {c.synthetic && !synthetic && <Synthetic />}
                        <button className="btn btn-quiet btn-icon btn-s" aria-label={`Remove ${c.id}`} onClick={() => cmp.toggle(c.id)}>
                          <IconClose width={12} height={12} />
                        </button>
                      </span>
                    </th>
                  ))}
                  <th>Comparable</th>
                </tr>
              </thead>
              <tbody>
                {(["family", "investigation_id", "fuel_or_material", "configuration"] as const).map((k) => (
                  <tr key={k}>
                    <th scope="row" className="dim" style={{ fontWeight: 500 }}>
                      {words(k)}
                    </th>
                    {r.columns.map((c) => (
                      <td key={c.id}>{c[k] == null ? <span className="dim">—</span> : words(String(c[k]))}</td>
                    ))}
                    <td />
                  </tr>
                ))}
                <tr>
                  <th scope="row" className="dim" style={{ fontWeight: 500 }}>
                    gravity
                  </th>
                  {r.columns.map((c) => (
                    <td key={c.id} className="num">
                      {fmtGravity(c.gravity)}
                    </td>
                  ))}
                  <td />
                </tr>
                {r.rows.map((row) => (
                  <tr key={row.key}>
                    <th scope="row" className="dim" style={{ fontWeight: 500 }}>
                      {words(row.key)}
                    </th>
                    {row.values.map((v, i) => (
                      <td key={i} className="num">
                        {v ? (v.value_si == null ? <span className="dim">{words(v.resolution)}</span> : fmtMeasured(v.value_si, v.unit_si)) : <span className="dim">not recorded</span>}
                      </td>
                    ))}
                    <td>{row.comparable ? <span className="tag tag-ok">yes</span> : <span className="tag tag-warn">no: {row.reason}</span>}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}
    </div>
  );
}
