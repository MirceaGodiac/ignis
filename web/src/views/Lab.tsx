// Forecast lab: give a model only the beginning of a recording, read its forecast with uncertainty,
// seal it, then open the real continuation. The steps are numbered because their order matters.
import { useMemo, useState, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import type { AnalysisArtifact, Observation, Quantity } from "../contracts/generated";
import { call, type AcquisitionResult, type ExperimentDetail, type ExperimentList, type ForecastOutcome, type ObservationsAtCutoff, type Refusal, type RevealResult, type SealResult } from "../api";
import { asRefusal, sleep, useLatest } from "../hooks";
import { ForecastChart, type Bands, type ChartPoint } from "../components/ForecastChart";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { FamilyMark, IconChevron } from "../shell/icons";
import { go, href } from "../shell/router";
import { Caution, Gap, RefusalNote, ReleaseState, shortId, words } from "../ui/common";
import { LabModels } from "./LabModels";

const display = (unit: string | null | undefined) => (unit === "m" ? { k: 1e3, unit: "mm" } : { k: 1, unit: unit ?? "" });
type Result<T> = T | { error: Refusal | null };
const isErr = <T,>(x: Result<T> | undefined): x is { error: Refusal | null } => !!x && typeof x === "object" && "error" in x;

export function Lab({ runId }: { runId: string | null }) {
  const { origin, experiments } = useData();
  const panel = usePanel();
  const detail = useQuery({ queryKey: [origin, "experiment", runId], queryFn: () => call<ExperimentDetail>(origin, `/experiments/${encodeURIComponent(runId!)}`), enabled: !!runId, retry: false });
  const releases = detail.data?.releases ?? [];
  const [releasePick, setReleasePick] = useState("");
  const release = releases.find((r) => r.id === releasePick) ?? releases[0] ?? null;
  const releaseId = release?.id ?? null;

  // Which partition the run is in for the release's split: shown before any click.
  const part = (p: "train" | "validation" | "test") =>
    useQuery({
      queryKey: [origin, "partition", release?.split_id, p],
      queryFn: () => call<ExperimentList>(origin, `/experiments?split=${encodeURIComponent(release!.split_id)}&partition=${p}`),
      enabled: !!release?.split_id,
      retry: false,
      staleTime: 300_000,
    });
  const train = part("train");
  const validation = part("validation");
  const test = part("test");
  const partition = !runId ? null : train.data?.experiments.some((x) => x.id === runId) ? "train" : test.data?.experiments.some((x) => x.id === runId) ? "test" : validation.data?.experiments.some((x) => x.id === runId) ? "validation" : null;
  const suggestion = validation.data?.experiments.find((x) => x.id !== runId)?.id;

  const extent = useMemo(() => Math.max(1e-6, ...(detail.data?.summary.quantities.map((q) => q.archive_t_max_s) ?? [1])), [detail.data]);
  const [cutoffByRun, setCutoffByRun] = useState<Record<string, number>>({});
  const cutoff = runId ? cutoffByRun[runId] ?? +(extent * 0.4).toPrecision(3) : 0;
  const setCutoff = (c: number) => runId && setCutoffByRun((m) => ({ ...m, [runId]: Math.max(0, Math.min(c, extent)) }));
  const step = +(extent / 200).toPrecision(2);

  const obs = useLatest<ObservationsAtCutoff>(runId ? `${origin}|${runId}|${cutoff}` : null, (signal) => call(origin, `/experiments/${encodeURIComponent(runId!)}/observations?cutoff_s=${cutoff}`, { signal }));
  const fc = useLatest<ForecastOutcome & { outcome: "artifact" }>(
    runId && releaseId ? `${origin}|${runId}|${releaseId}|${cutoff}` : null,
    async (signal) => {
      let r = await call<ForecastOutcome>(origin, "/forecast", { json: { run_id: runId, cutoff_s: cutoff, model_release_id: releaseId, wait_ms: 1500 }, signal });
      while (r.outcome === "job") {
        await sleep(600, signal);
        r = await call<ForecastOutcome>(origin, `/jobs/${r.job_id}`, { signal });
      }
      return r;
    },
    300,
  );
  const art = fc.data?.artifact;
  const quantity: Quantity | undefined = art?.quantity ?? detail.data?.summary.quantities[0]?.quantity;
  const unit = art?.unit ?? detail.data?.summary.quantities.find((q) => q.quantity === quantity)?.unit_si;
  const d = display(unit);
  const observed: ChartPoint[] = useMemo(() => (obs.data?.admitted ?? []).filter((o) => o.quantity === quantity && o.value_si != null).map((o) => ({ t: o.time_s, v: o.value_si! * d.k })), [obs.data, quantity, d.k]);
  const showBands = !!art && art.support.admitted && !fc.data!.refused_by_expert && (art.quantiles?.length ?? 0) > 0;
  const bands: Bands | null = useMemo(() => (showBands && art ? { times: art.times_s ?? [], levels: art.quantile_levels ?? [], q: (art.quantiles ?? []).map((row) => row.map((v) => v * d.k)) } : null), [showBands, art, d.k]);

  const [seal, setSeal] = useState<Record<string, Result<SealResult>>>({});
  const [reveal, setReveal] = useState<Record<string, Result<RevealResult>>>({});
  const [acq, setAcq] = useState<Record<string, Result<AcquisitionResult>>>({});
  // A reveal belongs to the run, not to one artifact: after it, no later seal on that run is blind.
  const [revealedRun, setRevealedRun] = useState<Record<string, { cutoff: number; analysis: string }>>({});
  const aid = art?.id;
  const sealed = aid && seal[aid] && !isErr(seal[aid]) ? (seal[aid] as SealResult) : null;
  const revealed = aid && reveal[aid] && !isErr(reveal[aid]) ? (reveal[aid] as RevealResult) : null;
  const priorReveal = runId ? revealedRun[runId] : undefined;
  const staleReveal = !!priorReveal && priorReveal.analysis !== aid;
  const revealedPts = useMemo(() => (revealed ? revealed.continuation.filter((o: Observation) => o.quantity === quantity && o.value_si != null).map((o) => ({ t: o.time_s, v: o.value_si! * d.k })) : null), [revealed, quantity, d.k]);
  const post = async <T,>(path: string, set: (f: (m: Record<string, Result<T>>) => Record<string, Result<T>>) => void, after?: (r: T) => void) => {
    if (!aid) return;
    try {
      const r = await call<T>(origin, path, { json: { analysis_id: aid } });
      set((m) => ({ ...m, [aid]: r }));
      after?.(r);
    } catch (e) {
      set((m) => ({ ...m, [aid]: { error: asRefusal(e) } }));
    }
  };
  const acqRes = aid ? acq[aid] : undefined;
  const forecastRefusal = asRefusal(fc.error);
  const fitOnly = art?.status === "fit_only";
  const [table, setTable] = useState(false);

  const runs = experiments.data?.experiments ?? [];

  return (
    <div className="view">
      <div className="view-head">
        <h1>{runId ? `Forecast from the first ${cutoff.toPrecision(3)} s` : "Forecast lab"}</h1>
        <div className="toolbar">
          <label className="sr-only" htmlFor="lab-run">
            Run
          </label>
          <select id="lab-run" className="field" value={runId ?? ""} onChange={(e) => go("lab", e.target.value)}>
            <option value="" disabled>
              Choose a run
            </option>
            {runs.map((x) => (
              <option key={x.id} value={x.id}>
                {x.id} · {x.family.replace("_", " ")}
              </option>
            ))}
          </select>
          <label className="sr-only" htmlFor="lab-rel">
            Model release
          </label>
          <select id="lab-rel" className="field" value={releaseId ?? ""} onChange={(e) => setReleasePick(e.target.value)} disabled={!releases.length}>
            {!releases.length && <option value="">No release for this family</option>}
            {releases.map((r) => (
              <option key={r.id} value={r.id}>
                {r.model_name} · {words(r.state)}
              </option>
            ))}
          </select>
        </div>
      </div>

      {!runId && (
        <div className="empty">
          <b>Choose a run to forecast</b>
          <p>Runs in a validation partition can be sealed and revealed. FLEX droplet runs have a model release in this data source.</p>
          <div className="toolbar">
            {runs
              .filter((r) => r.family === "flex")
              .slice(0, 6)
              .map((r) => (
                <a key={r.id} className="btn btn-s" href={href("lab", r.id)}>
                  <FamilyMark family={r.family} size={9} /> {r.id}
                </a>
              ))}
          </div>
        </div>
      )}

      {runId && detail.error && <RefusalNote refusal={asRefusal(detail.error)!} next="Choose another run, or switch the data source in the sidebar." />}

      {runId && detail.data && (
        <div className="lab">
          <section className="sheet" aria-labelledby="chart-h">
            <div className="sheet-head">
              <h2 id="chart-h">
                {quantity ? words(quantity) : "observations"} <span className="muted">[{d.unit}]</span>
              </h2>
              <span className="muted" style={{ fontSize: 12 }} aria-live="polite">
                {fc.loading ? "Updating the forecast…" : obs.loading ? "Updating observations…" : art ? (fc.data!.cached ? "Cached artifact" : "Computed now") : ""}
              </span>
            </div>
            <ul className="legend legend-inline" aria-label="Chart legend">
              <li>
                <span className="glyph g-observed" style={{ color: "var(--observed)" }} /> Observed before the cutoff
              </li>
              <li>
                <span className="glyph g-predicted" /> Model output: quantile bands, dashed median
              </li>
              <li>
                <span className="glyph g-revealed" /> Revealed continuation {revealedPts ? "" : "(sealed)"}
              </li>
              <li>
                <span className="glyph g-cutoff" /> Cutoff
              </li>
            </ul>
            <ForecastChart observed={observed} revealed={revealedPts} bands={bands} cutoff={cutoff} xMax={extent} yLabel={`${quantity ? words(quantity) : ""} [${d.unit}]`} />
            <div className="cutoff-row">
              <label htmlFor="cutoff">Cutoff, s</label>
              <input id="cutoff" type="range" min={0} max={extent} step={step} value={cutoff} onChange={(e) => setCutoff(+e.target.value)} aria-valuetext={`${cutoff} seconds of ${extent.toPrecision(3)}`} />
              <input className="field num" type="number" aria-label="Cutoff in seconds" min={0} max={extent} step={step} value={cutoff} onChange={(e) => setCutoff(+e.target.value)} />
            </div>
            {obs.data && (
              <p className="muted num" style={{ fontSize: 12 }}>
                {obs.data.admitted.length} admitted · {obs.data.pending.length} recorded but not yet available · {obs.data.hidden_after_cutoff} after the cutoff, {revealedPts ? "revealed" : "sealed"} ·
                archive extent {extent.toPrecision(3)} s (display only, never a model input)
              </p>
            )}
            {!bands && !fc.loading && (
              <Gap title="No model output is drawn">
                {forecastRefusal ? `${words(forecastRefusal.kind)}: ${forecastRefusal.reason}` : art ? "The expert refused this prefix; see step 2." : releaseId ? "Waiting for the forecast." : "There is no model release for this family."}
              </Gap>
            )}
            <button className="btn btn-s btn-quiet" style={{ justifySelf: "start" }} onClick={() => setTable((t) => !t)} aria-expanded={table}>
              {table ? "Hide" : "Show"} the data as a table
            </button>
            {table && <DataTable observed={observed} bands={bands} revealed={revealedPts} unit={d.unit} />}
          </section>

          <div className="steps">
            {partition === "train" && (
              <Caution title="This run is in the release's training partition">
                The model has seen this run, so a sealed reveal is not a test and the server refuses it.{" "}
                {suggestion && (
                  <a href={href("lab", suggestion)}>
                    Try {suggestion}, a validation run
                  </a>
                )}
              </Caution>
            )}
            {partition === "test" && (
              <Caution title="This run is in the test partition">It is reserved for the final evaluation. Forecast it here only to inspect; do not use it to tune anything.</Caution>
            )}

            <Step n={1} done title="Run and cutoff">
              <p>
                <button className="linkish" onClick={() => panel.open({ kind: "run", id: runId })}>
                  {runId}
                </button>{" "}
                · cutoff {cutoff.toPrecision(3)} s{partition && ` · ${partition} partition`}
              </p>
            </Step>

            <Step n={2} done={!!art} title="Forecast" tag={fitOnly ? <span className="tag tag-warn">fit only</span> : art ? <ReleaseState state={art.status} /> : undefined}>
              {forecastRefusal && <RefusalNote refusal={forecastRefusal} />}
              {art && !fc.loading && <SupportNote artifact={art} />}
              {art && !fc.loading && <EventStack artifact={art} />}
              {release && (
                <p className="muted" style={{ fontSize: 12 }}>
                  {release.model_name} · trained on {words(release.trained_on)} · <code>{shortId(release.id, 20)}</code>
                </p>
              )}
            </Step>

            <Step n={3} done={!!acqRes && !isErr(acqRes)} title="Next observation">
              <p>Candidates were recorded before the cutoff but are not admitted yet. Listing them shows no values.</p>
              <button className="btn btn-s" disabled={!art || fc.loading} onClick={() => post<AcquisitionResult>("/acquisition", setAcq)}>
                List candidates
              </button>
              {isErr(acqRes) && acqRes.error && <RefusalNote refusal={acqRes.error} />}
              {acqRes && !isErr(acqRes) && (
                <>
                  {acqRes.scoring.status === "refused" && <RefusalNote refusal={acqRes.scoring.refusal} title="Scoring refused" />}
                  {acqRes.candidates.length === 0 ? (
                    <Gap title="No candidates">{acqRes.scoring.status === "no_candidates" ? acqRes.scoring.reason : "Every observation recorded before the cutoff is already admitted."}</Gap>
                  ) : (
                    <table className="tbl">
                      <thead>
                        <tr>
                          <th>Observation</th>
                          <th className="r">t, s</th>
                          <th className="r">Utility</th>
                        </tr>
                      </thead>
                      <tbody>
                        {acqRes.candidates.map((c) => {
                          const sc = acqRes.scoring.status === "scored" ? acqRes.scoring.trace.candidates.find((x) => x.action_id === c.id) : undefined;
                          return (
                            <tr key={c.id}>
                              <td>
                                <code>{shortId(c.observation_id, 18)}</code>
                              </td>
                              <td className="r num">{c.time_s.toPrecision(4)}</td>
                              <td className="r num">{sc ? sc.utility.toPrecision(3) : "—"}</td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  )}
                  <p className="muted" style={{ fontSize: 12 }}>{acqRes.cost_note}</p>
                </>
              )}
            </Step>

            <Step n={4} done={!!sealed} title="Seal the prediction">
              <p>The artifact is hashed before anything after the cutoff is opened.</p>
              {staleReveal && (
                <Caution title="The continuation of this run is already revealed">
                  It was opened at the {priorReveal!.cutoff.toPrecision(3)} s cutoff. A new seal on this run would not be blind, so it is not offered.
                </Caution>
              )}
              <button className="btn btn-s" disabled={!art || fc.loading || !!sealed || staleReveal} onClick={() => post<SealResult>("/seal", setSeal)}>
                {sealed ? "Sealed" : "Seal prediction"}
              </button>
              {sealed && (
                <p className="muted num" style={{ fontSize: 12 }}>
                  {sealed.sealed.sealed_at} · {sealed.sealed.split_partition} · <code>{sealed.sealed.artifact_hash.slice(0, 19)}…</code>
                </p>
              )}
              {aid && isErr(seal[aid]) && (seal[aid] as { error: Refusal | null }).error && <RefusalNote refusal={(seal[aid] as { error: Refusal }).error} />}
            </Step>

            <Step n={5} done={!!revealed} title="Reveal the continuation">
              <button
                className="btn btn-s btn-primary"
                disabled={!sealed || !!revealed}
                onClick={() => post<RevealResult>("/reveal", setReveal, () => runId && aid && setRevealedRun((m) => ({ ...m, [runId]: { cutoff, analysis: aid } })))}
              >
                {revealed ? "Revealed" : "Reveal"}
              </button>
              {aid && isErr(reveal[aid]) && (reveal[aid] as { error: Refusal | null }).error && <RefusalNote refusal={(reveal[aid] as { error: Refusal }).error} />}
              {revealed && (
                <p>
                  {revealed.continuation.length} observations revealed
                  {revealed.endpoint ? `; endpoint ${words(revealed.endpoint.censoring)}${revealed.endpoint.first_terminal_s != null ? ` at ${revealed.endpoint.first_terminal_s.toPrecision(4)} s` : ""}` : "; no endpoint record"}. Compare the rings with the
                  bands on the chart.
                </p>
              )}
            </Step>

            {art && fc.data && (
              <details className="route">
                <summary>
                  <b>Provenance</b> <span>release, prefix and settings hashes</span>
                  <IconChevron className="chev" />
                </summary>
                <dl className="kv" style={{ padding: "4px 14px 14px" }}>
                  <dt>analysis</dt>
                  <dd>
                    <code>{art.id}</code>
                  </dd>
                  <dt>served</dt>
                  <dd>{fc.data.cached ? "cached (same release, prefix and settings)" : `computed ${new Date(fc.data.context.computed_at_unix_s * 1000).toISOString()}`}</dd>
                  <dt>time origin</dt>
                  <dd>{fc.data.context.time_origin}</dd>
                  <dt>cohort / split</dt>
                  <dd>
                    <code>{fc.data.context.cohort_id}</code> / <code>{fc.data.context.split_id}</code>
                  </dd>
                  <dt>prefix hash</dt>
                  <dd>
                    <code>{shortId(fc.data.context.prefix_hash, 26)}</code>
                  </dd>
                  <dt>settings hash</dt>
                  <dd>
                    <code>{shortId(art.settings_hash, 26)}</code>
                  </dd>
                  <dt>admitted</dt>
                  <dd>
                    {fc.data.context.admitted_observation_ids.length} observations, {fc.data.context.admitted_contour_ids.length} contours
                  </dd>
                  <dt>conditions used</dt>
                  <dd>{fc.data.context.known_condition_keys.map(words).join(", ") || "none"}</dd>
                  <dt>engine</dt>
                  <dd>
                    <code>{fc.data.context.engine}</code>
                  </dd>
                </dl>
              </details>
            )}
          </div>
        </div>
      )}

      <LabModels />
    </div>
  );
}

export function Step({ n, title, done, tag, children }: { n: number; title: string; done?: boolean; tag?: ReactNode; children: ReactNode }) {
  return (
    <section className="stepcard" aria-label={`Step ${n}: ${title}`}>
      <h3>
        <span className={`n${done ? " done" : ""}`} aria-hidden="true">
          {done ? (
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
              <path d="M2 5.2l2 2L8 3" />
            </svg>
          ) : (
            n
          )}
        </span>
        {title}
        {tag && <span style={{ marginLeft: "auto" }}>{tag}</span>}
      </h3>
      {children}
    </section>
  );
}

const STATE_NOTE: Record<string, string> = {
  fit_only: "Shown for inspection only: this is not a qualified forecast.",
  forecast_qualified: "Qualified by its release gates.",
  event_qualified: "Event probabilities qualified.",
  unsupported: "Outside the model's support: no number is promoted.",
  insufficient_inference: "Conditioning is not reliable: no number is promoted.",
  failed: "The forecast failed.",
};

/** The support gate and the release state, in one note whose tone follows the weaker of the two. */
export function SupportNote({ artifact }: { artifact: AnalysisArtifact }) {
  const s = artifact.support;
  const tone = !s.admitted ? "note-refusal" : artifact.status === "fit_only" || !["forecast_qualified", "event_qualified", "measurement_qualified"].includes(artifact.status) ? "note-caution" : "note-ok";
  return (
    <div className={`note ${tone}`} role="status">
      <b>
        {s.admitted ? "Inside the model's support" : "Refused by the support gate"} · {words(artifact.status)}
      </b>
      <p>
        {STATE_NOTE[artifact.status] ?? ""} Gate <code>{s.gate}</code>.
      </p>
      {s.reasons?.length ? (
        <ul style={{ paddingLeft: 18 }}>
          {s.reasons.map((r) => (
            <li key={r}>{r}</li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}

export function EventStack({ artifact }: { artifact: AnalysisArtifact }) {
  const cats = artifact.event_categories ?? [];
  const cdf = artifact.event_cdf ?? [];
  const t = artifact.times_s ?? [];
  if (!cats.length || !t.length) return <p className="muted" style={{ fontSize: 12 }}>This artifact carries no event probabilities.</p>;
  const W = 600;
  const H = 110;
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
    <div className="panel-sec">
      <svg viewBox={`0 0 ${W} ${H}`} className="evstack" preserveAspectRatio="none" role="img" aria-label="Stacked cumulative event probabilities (model output)">
        {paths.map((p, k) => (
          <polygon key={cats[k]} points={p} fill={colors[k % colors.length]} />
        ))}
      </svg>
      <table className="tbl">
        <caption>Cumulative probability at t = {t[last].toPrecision(4)} s (model output)</caption>
        <tbody>
          {cats.map((c, k) => (
            <tr key={c}>
              <th scope="row" style={{ fontWeight: 500 }}>
                <span className="swatch" style={{ background: colors[k % colors.length] }} aria-hidden="true" /> {words(c)}
              </th>
              <td className="r num">{(cdf[k][last] * 100).toFixed(1)} %</td>
            </tr>
          ))}
          {!cats.some((c) => c.startsWith("active")) && (
          <tr>
            <th scope="row" style={{ fontWeight: 500 }}>
              still burning
            </th>
            <td className="r num">{((1 - cdf.reduce((s, r) => s + r[last], 0)) * 100).toFixed(1)} %</td>
          </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

function DataTable({ observed, bands, revealed, unit }: { observed: ChartPoint[]; bands: Bands | null; revealed: ChartPoint[] | null; unit: string }) {
  const every = bands ? Math.max(1, Math.floor(bands.times.length / 12)) : 1;
  return (
    <div style={{ display: "grid", gap: 12, gridTemplateColumns: "repeat(auto-fit, minmax(220px, 1fr))", alignItems: "start" }}>
      <table className="tbl">
        <caption>Observed, admitted [{unit}]</caption>
        <thead>
          <tr>
            <th className="r">t, s</th>
            <th className="r">value</th>
          </tr>
        </thead>
        <tbody>
          {observed.map((p) => (
            <tr key={p.t}>
              <td className="r num">{p.t.toPrecision(4)}</td>
              <td className="r num">{p.v.toPrecision(4)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {bands && (
        <table className="tbl">
          <caption>Model output quantiles [{unit}], every {every}th time</caption>
          <thead>
            <tr>
              <th className="r">t, s</th>
              {bands.levels.map((l) => (
                <th key={l} className="r">
                  q{Math.round(l * 100)}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {bands.times
              .map((tt, j) => [tt, j] as const)
              .filter(([, j]) => j % every === 0)
              .map(([tt, j]) => (
                <tr key={tt}>
                  <td className="r num">{tt.toPrecision(4)}</td>
                  {bands.q.map((row, l) => (
                    <td key={l} className="r num">
                      {row[j].toPrecision(4)}
                    </td>
                  ))}
                </tr>
              ))}
          </tbody>
        </table>
      )}
      {revealed && (
        <table className="tbl">
          <caption>Revealed continuation [{unit}]</caption>
          <tbody>
            {revealed.map((p) => (
              <tr key={p.t}>
                <td className="r num">{p.t.toPrecision(4)}</td>
                <td className="r num">{p.v.toPrecision(4)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
