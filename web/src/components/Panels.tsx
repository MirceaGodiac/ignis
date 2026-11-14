import type { ReactNode } from "react";
import type { AnalysisArtifact } from "../contracts/generated";
import type { AnalysisContext, Refusal } from "../api";

export function Legend({ revealed }: { revealed: boolean }) {
  return (
    <ul className="legend" aria-label="Chart legend">
      <li>
        <span className="mark mark-observed" aria-hidden="true" /> <b>Observed</b> — measurements admitted at the cutoff
      </li>
      <li>
        <span className="mark mark-model" aria-hidden="true" /> <b>Model output</b> — quantile bands (dashed median), not data
      </li>
      <li>
        <span className="mark mark-revealed" aria-hidden="true" /> <b>Revealed continuation</b> — {revealed ? "shown after explicit reveal" : "hidden until reveal"}
      </li>
      <li>
        <span className="mark mark-cutoff" aria-hidden="true" /> Cutoff
      </li>
    </ul>
  );
}

export function RefusalBox({ refusal, title }: { refusal: Refusal; title?: string }) {
  return (
    <div className={`refusal refusal-${refusal.kind}`} role="status">
      <strong>{title ?? labelFor(refusal.kind)}</strong> <code>{refusal.code}</code>
      <p>{refusal.reason}</p>
    </div>
  );
}

export function labelFor(kind: string): string {
  switch (kind) {
    case "unsupported":
      return "Refused — unsupported";
    case "insufficient_inference":
      return "Refused — insufficient inference";
    case "unavailable":
      return "Unavailable";
    case "precondition_failed":
      return "Not allowed yet";
    case "invalid_request":
      return "Invalid request";
    case "not_found":
      return "Not found";
    default:
      return "Failed";
  }
}

const STATE_NOTE: Record<string, string> = {
  fit_only: "fit only — not a qualified forecast",
  forecast_qualified: "forecast qualified by its release gates",
  event_qualified: "event probabilities qualified",
  unsupported: "outside support — no number is promoted",
  insufficient_inference: "conditioning not reliable — no number is promoted",
  failed: "failed",
};

export function SupportPanel({ artifact }: { artifact: AnalysisArtifact }) {
  const s = artifact.support;
  return (
    <section className={`card support ${s.admitted ? "ok" : "refused"}`} aria-labelledby="support-h">
      <h3 id="support-h">Support and status</h3>
      <p>
        <span className={`badge ${s.admitted ? "badge-ok" : "badge-refused"}`}>{s.admitted ? "admitted" : "refused"}</span>{" "}
        gate <code>{s.gate}</code> · status <code>{artifact.status}</code>
        {STATE_NOTE[artifact.status] ? <> — {STATE_NOTE[artifact.status]}</> : null}
      </p>
      {s.reasons?.length ? (
        <ul>
          {s.reasons.map((r) => (
            <li key={r}>{r}</li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}

/** Event CDF rows stacked; remaining mass = still active. Drawn only when the artifact has rows. */
export function EventStack({ artifact }: { artifact: AnalysisArtifact }) {
  const cats = artifact.event_categories ?? [];
  const cdf = artifact.event_cdf ?? [];
  const t = artifact.times_s ?? [];
  if (!cats.length || !t.length) {
    return (
      <section className="card" aria-labelledby="ev-h">
        <h3 id="ev-h">Event probabilities</h3>
        <p className="muted">This artifact carries no event probabilities.</p>
      </section>
    );
  }
  const W = 600, H = 140;
  const x = (i: number) => (t.length === 1 ? 0 : ((t[i] - t[0]) / (t[t.length - 1] - t[0])) * W);
  const colors = ["var(--ev1)", "var(--ev2)", "var(--ev3)", "var(--ev4)"];
  let lower = t.map(() => 0);
  const paths = cdf.map((row) => {
    const upper = row.map((p, i) => lower[i] + p);
    const pts = [...upper.map((u, i) => `${x(i)},${H - u * H}`), ...lower.map((l, i) => `${x(i)},${H - l * H}`).reverse()];
    lower = upper;
    return pts.join(" ");
  });
  const last = t.length - 1;
  return (
    <section className="card" aria-labelledby="ev-h">
      <h3 id="ev-h">Event probabilities (model output)</h3>
      <svg viewBox={`0 0 ${W} ${H}`} className="evstack" role="img" aria-label="Stacked cumulative event probabilities over the forecast horizon">
        {paths.map((p, k) => (
          <polygon key={cats[k]} points={p} fill={colors[k % colors.length]} />
        ))}
      </svg>
      <table className="data">
        <caption>Cumulative probability at the end of the horizon (t = {t[last].toPrecision(4)} s)</caption>
        <tbody>
          {cats.map((c, k) => (
            <tr key={c}>
              <th scope="row">
                <span className="swatch" style={{ background: colors[k % colors.length] }} aria-hidden="true" /> {c}
              </th>
              <td>{(cdf[k][last] * 100).toFixed(1)} %</td>
            </tr>
          ))}
          <tr>
            <th scope="row">still active</th>
            <td>{((1 - cdf.reduce((s, r) => s + r[last], 0)) * 100).toFixed(1)} %</td>
          </tr>
        </tbody>
      </table>
    </section>
  );
}

export function Provenance({ ctx, artifact, cached, open, onClose, extra }: { ctx: AnalysisContext; artifact: AnalysisArtifact; cached: boolean; open: boolean; onClose: () => void; extra?: ReactNode }) {
  if (!open) return null;
  const rows: [string, ReactNode][] = [
    ["analysis id", <code>{artifact.id}</code>],
    ["served", cached ? "cached (same release, prefix and settings hash)" : `computed at ${new Date(ctx.computed_at_unix_s * 1000).toISOString()}`],
    ["data origin", ctx.data_origin],
    ["run / cutoff", `${ctx.run_id} @ ${ctx.cutoff_s} s (time origin: ${ctx.time_origin})`],
    ["model", `${ctx.model_name} · state ${ctx.release_state} · trained on ${ctx.trained_on}`],
    ["release", <code>{ctx.model_release_id}</code>],
    ["release created / code", `${ctx.release_created_at} · ${ctx.release_code_version}`],
    ["cohort / split", <><code>{ctx.cohort_id}</code> / <code>{ctx.split_id}</code></>],
    ["source manifest", <code>{ctx.source_manifest_hash}</code>],
    ["prefix hash", <code>{ctx.prefix_hash}</code>],
    ["settings hash", <code>{artifact.settings_hash}</code>],
    ["admitted observations", `${ctx.admitted_observation_ids.length} (${ctx.admitted_contour_ids.length} contours)`],
    ["prospective conditions", ctx.known_condition_keys.join(", ") || "none"],
    ["settings", <code>{JSON.stringify(ctx.settings)}</code>],
    ["diagnostics", <code>{JSON.stringify(artifact.diagnostics ?? {})}</code>],
    ["evidence class", artifact.evidence_class ?? "model_output"],
    ["engine", <code>{ctx.engine}</code>],
  ];
  return (
    <aside className="drawer" role="dialog" aria-modal="false" aria-labelledby="prov-h" onKeyDown={(e) => e.key === "Escape" && onClose()}>
      <div className="drawer-head">
        <h3 id="prov-h">Provenance</h3>
        <button onClick={onClose} autoFocus>
          Close
        </button>
      </div>
      <dl>
        {rows.map(([k, v]) => (
          <div key={k}>
            <dt>{k}</dt>
            <dd>{v}</dd>
          </div>
        ))}
      </dl>
      {extra}
    </aside>
  );
}
