import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { Quantity } from "../contracts/generated";
import { call, type CompareResult, type Origin } from "../api";
import { asRefusal } from "../hooks";
import { RefusalBox } from "../components/Panels";
import { fmtGravity, fmtMeasured } from "../format";

const QUANTITIES: Quantity[] = ["liquid_diameter", "visible_flame_diameter", "uv_flame_diameter", "front_position", "front_contour", "pressure", "temperature"];

export function Compare({ origin, runIds, clear }: { origin: Origin; runIds: string[]; clear: () => void }) {
  const [quantity, setQuantity] = useState<string>("");
  const enabled = runIds.length >= 2;
  const res = useQuery({
    queryKey: [origin, "compare", runIds, quantity],
    queryFn: ({ signal }) => call<CompareResult>(origin, "/compare", { json: { run_ids: runIds, ...(quantity ? { quantity } : {}) }, signal }),
    enabled,
    retry: false,
  });
  if (!enabled) return <p className="muted">Select at least two experiments in Explore to compare them.</p>;
  const r = res.data;
  return (
    <div className="compare">
      <div className="row">
        <label>
          Quantity for numeric comparison{" "}
          <select value={quantity} onChange={(e) => setQuantity(e.target.value)}>
            <option value="">conditions only</option>
            {QUANTITIES.map((q) => (
              <option key={q}>{q}</option>
            ))}
          </select>
        </label>
        <button onClick={clear}>Clear selection</button>
      </div>
      {res.error && <RefusalBox refusal={asRefusal(res.error)!} />}
      {r && (
        <>
          <section className={`card ${r.numeric_comparison.allowed ? "ok" : "refused"}`} aria-labelledby="num-h">
            <h3 id="num-h">
              Numeric comparison: {r.numeric_comparison.allowed ? "allowed" : "refused"} <span className="muted">({r.comparison_kind})</span>
            </h3>
            {r.numeric_comparison.refusals.map((x) => (
              <RefusalBox key={x.code} refusal={x} />
            ))}
            <p className="muted">{r.numeric_comparison.note}</p>
          </section>
          <div className="table-wrap">
            <table className="data compare-table">
              <caption>Conditions side by side (SI)</caption>
              <thead>
                <tr>
                  <th scope="col">field</th>
                  {r.columns.map((c) => (
                    <th scope="col" key={c.id}>
                      {c.id}
                      {c.synthetic && <span className="badge badge-synthetic">synthetic</span>}
                    </th>
                  ))}
                  <th scope="col">comparable?</th>
                </tr>
              </thead>
              <tbody>
                {(["family", "investigation_id", "fuel_or_material", "configuration", "gravity"] as const).map((k) => (
                  <tr key={k}>
                    <th scope="row">{k}</th>
                    {r.columns.map((c) => (
                      <td key={c.id}>{k === "gravity" ? fmtGravity(c.gravity) : c[k] == null ? "—" : String(c[k])}</td>
                    ))}
                    <td />
                  </tr>
                ))}
                {r.rows.map((row) => (
                  <tr key={row.key} className={row.comparable ? "" : "incomparable"}>
                    <th scope="row">{row.key}</th>
                    {row.values.map((v, i) => (
                      <td key={i}>{v ? (v.value_si == null ? "unresolved" : fmtMeasured(v.value_si, v.unit_si)) : "not recorded"}</td>
                    ))}
                    <td>{row.comparable ? "yes" : `no — ${row.reason}`}</td>
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
