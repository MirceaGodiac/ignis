import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { Observation, Quantity } from "../contracts/generated";
import {
  call,
  type AcquisitionResult,
  type ExperimentDetail,
  type ExperimentList,
  type ForecastOutcome,
  type ObservationsAtCutoff,
  type Origin,
  type RevealResult,
  type SealResult,
} from "../api";
import { asRefusal, sleep, useLatest } from "../hooks";
import { ForecastChart, type Bands, type ChartPoint } from "../components/ForecastChart";
import { EventStack, Legend, Provenance, RefusalBox, SupportPanel } from "../components/Panels";

/** Display scaling: metres are shown in millimetres; the API stays SI. */
const display = (unit: string | null | undefined) => (unit === "m" ? { k: 1e3, unit: "mm" } : { k: 1, unit: unit ?? "" });

export function Lab({ origin, runId, setRunId }: { origin: Origin; runId: string | null; setRunId: (id: string) => void }) {
  const list = useQuery({ queryKey: [origin, "experiments"], queryFn: () => call<ExperimentList>(origin, "/experiments") });
  const detail = useQuery({
    queryKey: [origin, "experiment", runId],
    queryFn: () => call<ExperimentDetail>(origin, `/experiments/${encodeURIComponent(runId!)}`),
    enabled: !!runId,
  });
  const extent = useMemo(() => Math.max(1e-6, ...(detail.data?.summary.quantities.map((q) => q.archive_t_max_s) ?? [1])), [detail.data]);
  const [cutoffByRun, setCutoffByRun] = useState<Record<string, number>>({});
  const cutoff = runId ? cutoffByRun[runId] ?? +(extent * 0.4).toPrecision(3) : 0;
  const setCutoff = (c: number) => runId && setCutoffByRun((m) => ({ ...m, [runId]: Math.max(0, Math.min(c, extent)) }));
  const releases = detail.data?.releases ?? [];
  const [releasePick, setReleasePick] = useState<string>("");
  const releaseId = releases.find((r) => r.id === releasePick)?.id ?? releases[0]?.id ?? null;
  const [showTable, setShowTable] = useState(false);
  const [drawer, setDrawer] = useState(false);

  // Observations at the cutoff: debounced, cancelled, version-checked.
  const obsKey = runId ? `${origin}|${runId}|${cutoff}` : null;
  const obs = useLatest<ObservationsAtCutoff>(obsKey, (signal) =>
    call(origin, `/experiments/${encodeURIComponent(runId!)}/observations?cutoff_s=${cutoff}`, { signal }),
  );
  // Forecast: strict request (ids + cutoff + settings), job polling, stale responses rejected.
  const fKey = runId && releaseId ? `${origin}|${runId}|${releaseId}|${cutoff}` : null;
  const fc = useLatest<ForecastOutcome & { outcome: "artifact" }>(fKey, async (signal) => {
    let r = await call<ForecastOutcome>(origin, "/forecast", { json: { run_id: runId, cutoff_s: cutoff, model_release_id: releaseId, wait_ms: 1500 }, signal });
    while (r.outcome === "job") {
      await sleep(600, signal);
      r = await call<ForecastOutcome>(origin, `/jobs/${r.job_id}`, { signal });
    }
    return r;
  }, 300);

  const art = fc.data?.artifact;
  const quantity: Quantity | undefined = art?.quantity ?? detail.data?.summary.quantities[0]?.quantity;
  const unit = art?.unit ?? detail.data?.summary.quantities.find((q) => q.quantity === quantity)?.unit_si;
  const d = display(unit);
  const observed: ChartPoint[] = useMemo(
    () => (obs.data?.admitted ?? []).filter((o) => o.quantity === quantity && o.value_si != null).map((o) => ({ t: o.time_s, v: o.value_si! * d.k })),
    [obs.data, quantity, d.k],
  );
  const showBands = !!art && art.support.admitted && !fc.data!.refused_by_expert && (art.quantiles?.length ?? 0) > 0;
  const bands: Bands | null = useMemo(
    () => (showBands && art ? { times: art.times_s ?? [], levels: art.quantile_levels ?? [], q: (art.quantiles ?? []).map((row) => row.map((v) => v * d.k)) } : null),
    [showBands, art, d.k],
  );

  // Seal → reveal, per analysis id.
  const [seal, setSeal] = useState<Record<string, SealResult | { error: ReturnType<typeof asRefusal> }>>({});
  const [reveal, setReveal] = useState<Record<string, RevealResult | { error: ReturnType<typeof asRefusal> }>>({});
  const [acq, setAcq] = useState<Record<string, AcquisitionResult | { error: ReturnType<typeof asRefusal> }>>({});
  const aid = art?.id;
  const sealed = aid && seal[aid] && "sealed" in seal[aid] ? (seal[aid] as SealResult) : null;
  const revealed = aid && reveal[aid] && "continuation" in reveal[aid] ? (reveal[aid] as RevealResult) : null;
  const revealedPts: ChartPoint[] | null = useMemo(
    () => (revealed ? revealed.continuation.filter((o: Observation) => o.quantity === quantity && o.value_si != null).map((o) => ({ t: o.time_s, v: o.value_si! * d.k })) : null),
    [revealed, quantity, d.k],
  );
  const doSeal = async () => {
    if (!aid) return;
    try {
      setSeal((m) => ({ ...m, [aid]: undefined as never }));
      const r = await call<SealResult>(origin, "/seal", { json: { analysis_id: aid } });
      setSeal((m) => ({ ...m, [aid]: r }));
    } catch (e) {
      setSeal((m) => ({ ...m, [aid]: { error: asRefusal(e) } }));
    }
  };
  const doReveal = async () => {
    if (!aid) return;
    try {
      const r = await call<RevealResult>(origin, "/reveal", { json: { analysis_id: aid } });
      setReveal((m) => ({ ...m, [aid]: r }));
    } catch (e) {
      setReveal((m) => ({ ...m, [aid]: { error: asRefusal(e) } }));
    }
  };
  const doAcq = async () => {
    if (!aid) return;
    try {
      const r = await call<AcquisitionResult>(origin, "/acquisition", { json: { analysis_id: aid } });
      setAcq((m) => ({ ...m, [aid]: r }));
    } catch (e) {
      setAcq((m) => ({ ...m, [aid]: { error: asRefusal(e) } }));
    }
  };
  const acqRes = aid ? acq[aid] : undefined;
  const sealErr = aid && seal[aid] && "error" in seal[aid] ? (seal[aid] as { error: ReturnType<typeof asRefusal> }).error : null;
  const revealErr = aid && reveal[aid] && "error" in reveal[aid] ? (reveal[aid] as { error: ReturnType<typeof asRefusal> }).error : null;
  const step = +(extent / 200).toPrecision(2);
  const forecastRefusal = asRefusal(fc.error);

  return (
    <div className="lab">
      <section className="card controls" aria-label="Experiment and cutoff">
        <div className="row">
          <label>
            Run{" "}
            <select value={runId ?? ""} onChange={(e) => setRunId(e.target.value)}>
              <option value="" disabled>
                choose a run
              </option>
              {list.data?.experiments.map((x) => (
                <option key={x.id} value={x.id}>
                  {x.id} · {x.family}
                  {x.synthetic ? " · synthetic" : ""}
                </option>
              ))}
            </select>
          </label>
          <label>
            Model release{" "}
            <select value={releaseId ?? ""} onChange={(e) => setReleasePick(e.target.value)} disabled={!releases.length}>
              {!releases.length && <option value="">none for this family</option>}
              {releases.map((r) => (
                <option key={r.id} value={r.id}>
                  {r.model_name} · {r.state} · {r.trained_on}
                </option>
              ))}
            </select>
          </label>
        </div>
        {runId && (
          <div className="row cutoff">
            <label htmlFor="cutoff">Cutoff [s]</label>
            <input
              id="cutoff"
              type="range"
              min={0}
              max={extent}
              step={step}
              value={cutoff}
              onChange={(e) => setCutoff(+e.target.value)}
              aria-valuetext={`${cutoff} seconds of ${extent.toPrecision(3)} (archive extent)`}
            />
            <input type="number" aria-label="Cutoff seconds" min={0} max={extent} step={step} value={cutoff} onChange={(e) => setCutoff(+e.target.value)} />
            <span className="muted">archive extent {extent.toPrecision(3)} s (display only, never a model input)</span>
          </div>
        )}
      </section>

      {!runId && <p className="muted">Choose a run to open the lab.</p>}
      {runId && (
        <div className="lab-grid">
          <section className="card chart-card" aria-labelledby="chart-h">
            <div className="chart-head">
              <h3 id="chart-h">
                {quantity ?? "observations"} [{d.unit}]
              </h3>
              <span className="status-line" aria-live="polite">
                {fc.loading ? "updating forecast…" : obs.loading ? "updating observations…" : art ? (fc.data!.cached ? "cached artifact" : "live artifact") : ""}
              </span>
            </div>
            <Legend revealed={!!revealedPts} />
            <ForecastChart observed={observed} revealed={revealedPts} bands={bands} cutoff={cutoff} xMax={extent} yLabel={`${quantity ?? ""} [${d.unit}]`} />
            {!bands && !fc.loading && (
              <p className="no-model" role="status">
                No model output drawn{forecastRefusal ? ` — ${forecastRefusal.kind}: ${forecastRefusal.reason}` : art ? " — the expert refused (see support)" : releaseId ? "" : " — no model release for this family"}.
              </p>
            )}
            {art?.status === "fit_only" && bands && <p className="warn">Status fit_only: shown for inspection; not a qualified forecast.</p>}
            <p className="muted">
              {obs.data ? `${obs.data.admitted.length} admitted · ${obs.data.pending.length} recorded but not yet available · ${obs.data.hidden_after_cutoff} after the cutoff (hidden)` : ""}
            </p>
            <button onClick={() => setShowTable((s) => !s)} aria-expanded={showTable}>
              {showTable ? "Hide" : "Show"} data table
            </button>
            {showTable && <DataTable observed={observed} bands={bands} revealed={revealedPts} unit={d.unit} />}
          </section>

          <div className="side">
            {forecastRefusal && <RefusalBox refusal={forecastRefusal} />}
            {art && !fc.loading && <SupportPanel artifact={art} />}
            {art && !fc.loading && <EventStack artifact={art} />}

            {art && !fc.loading && (
              <section className="card" aria-labelledby="acq-h">
                <h3 id="acq-h">Next observation</h3>
                <p className="muted">Candidates are recorded before the cutoff but not yet admitted. Listing them reveals no values.</p>
                <button onClick={doAcq}>List candidates</button>
                {acqRes && "error" in acqRes && acqRes.error && <RefusalBox refusal={acqRes.error} />}
                {acqRes && "candidates" in acqRes && (
                  <>
                    <p className="muted">{acqRes.cost_note}</p>
                    {acqRes.scoring.status === "refused" && <RefusalBox refusal={acqRes.scoring.refusal} title="Scoring refused" />}
                    {acqRes.scoring.status === "no_candidates" && <p>{acqRes.scoring.reason}</p>}
                    <table className="data">
                      <thead>
                        <tr>
                          <th>observation</th>
                          <th>t [s]</th>
                          <th>cost</th>
                          <th>utility</th>
                        </tr>
                      </thead>
                      <tbody>
                        {acqRes.candidates.map((c) => {
                          const sc = acqRes.scoring.status === "scored" ? acqRes.scoring.trace.candidates.find((x) => x.action_id === c.id) : undefined;
                          return (
                            <tr key={c.id}>
                              <td>
                                <code>{c.observation_id}</code>
                              </td>
                              <td>{c.time_s.toPrecision(4)}</td>
                              <td>{c.cost}</td>
                              <td>{sc ? sc.utility.toPrecision(3) : "—"}</td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </>
                )}
              </section>
            )}

            {art && !fc.loading && (
              <section className="card" aria-labelledby="rev-h">
                <h3 id="rev-h">Seal and reveal</h3>
                <p className="muted">The continuation is opened only after this artifact is hash-sealed.</p>
                <div className="row">
                  <button onClick={doSeal} disabled={!!sealed}>
                    {sealed ? "Sealed" : "Seal prediction"}
                  </button>
                  <button onClick={doReveal} disabled={!sealed || !!revealed}>
                    {revealed ? "Revealed" : "Reveal continuation"}
                  </button>
                </div>
                {sealed && (
                  <p className="muted">
                    sealed {sealed.sealed.sealed_at} · {sealed.sealed.split_partition} · <code>{sealed.sealed.artifact_hash.slice(0, 19)}…</code>
                  </p>
                )}
                {sealErr && <RefusalBox refusal={sealErr} />}
                {revealErr && <RefusalBox refusal={revealErr} />}
                {revealed && (
                  <p>
                    Revealed {revealed.continuation.length} observations
                    {revealed.endpoint ? `; endpoint: ${revealed.endpoint.censoring}${revealed.endpoint.first_terminal_s != null ? ` at ${revealed.endpoint.first_terminal_s.toPrecision(4)} s` : ""}` : "; no endpoint record"}.
                  </p>
                )}
              </section>
            )}
            {art && fc.data && !fc.loading && (
              <button className="drawer-toggle" onClick={() => setDrawer(true)}>
                Provenance…
              </button>
            )}
          </div>
        </div>
      )}
      {art && fc.data && <Provenance ctx={fc.data.context} artifact={art} cached={fc.data.cached} open={drawer} onClose={() => setDrawer(false)} />}
    </div>
  );
}

function DataTable({ observed, bands, revealed, unit }: { observed: ChartPoint[]; bands: Bands | null; revealed: ChartPoint[] | null; unit: string }) {
  const every = bands ? Math.max(1, Math.floor(bands.times.length / 12)) : 1;
  return (
    <div className="table-wrap">
      <table className="data">
        <caption>Observed (admitted) [{unit}]</caption>
        <thead>
          <tr>
            <th>t [s]</th>
            <th>value</th>
          </tr>
        </thead>
        <tbody>
          {observed.map((p) => (
            <tr key={p.t}>
              <td>{p.t.toPrecision(4)}</td>
              <td>{p.v.toPrecision(4)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {bands && (
        <table className="data">
          <caption>Model output quantiles [{unit}] (every {every}th time)</caption>
          <thead>
            <tr>
              <th>t [s]</th>
              {bands.levels.map((l) => (
                <th key={l}>q{Math.round(l * 100)}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {bands.times
              .map((t, j) => [t, j] as const)
              .filter(([, j]) => j % every === 0)
              .map(([t, j]) => (
                <tr key={t}>
                  <td>{t.toPrecision(4)}</td>
                  {bands.q.map((row, l) => (
                    <td key={l}>{row[j].toPrecision(4)}</td>
                  ))}
                </tr>
              ))}
          </tbody>
        </table>
      )}
      {revealed && (
        <table className="data">
          <caption>Revealed continuation [{unit}]</caption>
          <tbody>
            {revealed.map((p) => (
              <tr key={p.t}>
                <td>{p.t.toPrecision(4)}</td>
                <td>{p.v.toPrecision(4)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
