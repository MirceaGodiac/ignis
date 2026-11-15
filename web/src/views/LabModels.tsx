// Hand-written models in the forecast lab: the releases the research server serves (/api/v1), each
// with what it is (hand-written, experimental, zero-shot or data-informed), what it reads and what
// its authors reported. A forecast runs the release's strict forward on a stored run of the train
// partition or on conditions typed here; a typed scenario is a synthetic input, never a NASA run.
import { useEffect, useMemo, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { AnalysisArtifact, JobRef, ModelCard, ModelInput, ScenarioAdmitted, ToolResult } from "../contracts/generated";
import { call, type ExperimentList, type ExperimentSummary, type ObservationsAtCutoff } from "../api";
import { ForecastChart, type Bands, type ChartPoint } from "../components/ForecastChart";
import { useData } from "../shell/data";
import { Caution, Gap, words } from "../ui/common";
import { EventStack, Step, SupportNote } from "./Lab";
import { familyLabel, hasSeries, runLabel } from "./RunPicker";

interface Envelope {
  code: string;
  message: string;
}

class V1Error extends Error {
  constructor(public envelope: Envelope) {
    super(`${envelope.code}: ${envelope.message}`);
  }
}

async function v1<T>(base: string, path: string, body?: unknown, signal?: AbortSignal): Promise<T> {
  const res = await fetch(base + path, {
    method: body === undefined ? "GET" : "POST",
    headers: body === undefined ? undefined : { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
    signal,
  });
  let j: unknown = null;
  try {
    j = await res.json();
  } catch {
    /* not JSON */
  }
  if (!res.ok && res.status !== 202) {
    const e = j as Partial<Envelope> | null;
    throw new V1Error(e && typeof e.code === "string" ? (e as Envelope) : { code: `http_${res.status}`, message: "the server answered without an error envelope" });
  }
  return j as T;
}

const authorshipLabel = (c: ModelCard) => (c.authorship === "zero_shot" ? "zero-shot" : "data-informed");
const display = (unit: string) => (unit === "m" ? { k: 1e3, unit: "mm" } : { k: 1, unit });
const fmtExample = (v: unknown) => (v === undefined || v === null ? "" : Array.isArray(v) ? v.map((row) => (Array.isArray(row) ? row.map((x) => (x === null ? "" : String(x))).join(", ") : String(row))).join("\n") : String(v));

/** "t, v" (or "start, end, value") per line; empty cells are null. */
function rows(text: string): (number | null)[][] {
  return text
    .split(/\n/)
    .map((l) => l.trim())
    .filter(Boolean)
    .map((l) =>
      (l.includes(",") ? l.split(",") : l.split(/[;\t ]+/)).map((cell) => {
        const c = cell.trim();
        if (c === "" || c === "-") return null;
        const x = Number(c);
        if (!Number.isFinite(x)) throw new Error(`"${c}" is not a number`);
        return x;
      }),
    );
}

type Mode = "stored" | "typed";

const norm = (x: string) => x.toLowerCase().replace(/^n-/, "").replace(/[^a-z0-9]+/g, " ").trim();

/** How well a served release fits a run: family first, then the fuel/material it was written for. */
export function fit(card: ModelCard, run: ExperimentSummary): { score: number; why: string[] } {
  const why: string[] = [];
  let score = 0;
  const inv = card.investigation.toLowerCase();
  if (card.family === run.family) {
    score += 4;
    why.push(`same family (${familyLabel(run.family)})`);
  } else if (card.family === "other" && run.investigation_id && inv.includes(run.investigation_id.toLowerCase())) {
    score += 3;
    why.push(`written for ${run.investigation_id}`);
  } else return { score: -1, why: [] };
  const fuel = run.fuel_or_material ? norm(run.fuel_or_material) : "";
  if (fuel && norm(card.investigation + " " + card.model_name).includes(fuel.split(" ")[0])) {
    score += 2;
    why.push(`written for ${words(run.fuel_or_material!)}`);
  } else if (fuel && /droplets|fabric|plastics/.test(inv)) score -= 1;
  if (card.stored_runs) {
    score += 1;
    why.push("reads the stored run directly");
  }
  return { score, why };
}

export function LabModels({ run, pickRun, runs }: { run: ExperimentSummary | null; pickRun: (id: string) => void; runs: ExperimentSummary[] }) {
  const { researchMode, origin, caps } = useData();
  const base = researchMode === "live" ? "/api/v1" : researchMode === "fixtures" ? "/api/fixtures/v1" : null;
  // Only the live server says whether the model families are switched on; fixtures always serve theirs.
  const modelsCap = researchMode === "live" ? caps.data?.models : undefined;
  const cards = useQuery({ queryKey: [researchMode, "model-cards"], queryFn: () => v1<ModelCard[]>(base!, "/model-cards"), enabled: !!base && modelsCap?.available !== false, retry: false });
  const [pickByRun, setPickByRun] = useState<Record<string, string>>({});
  const ranked = useMemo(() => {
    const all = cards.data ?? [];
    if (!run) return all.map((c) => ({ c, score: 0, why: [] as string[] }));
    return all.map((c) => ({ c, ...fit(c, run) })).sort((x, y) => y.score - x.score);
  }, [cards.data, run]);
  const fitting = ranked.filter((x) => x.score >= 0 && run);
  const auto = run ? fitting[0] ?? null : ranked[0] ?? null;
  const key = run?.id ?? "";
  const manual = pickByRun[key];
  const chosen = ranked.find((x) => x.c.release_id === manual) ?? auto;
  const card = chosen?.c ?? null;

  if (!base) return null;
  return (
    <section className="sheet" aria-labelledby="hw-h" style={{ marginTop: 16 }}>
      <div className="sheet-head">
        <h2 id="hw-h">
          Model and forecast
        </h2>
        {ranked.length > 0 && (
          <>
            <label className="sr-only" htmlFor="hw-model">
              Model
            </label>
            <select id="hw-model" className="field" value={manual && ranked.some((x) => x.c.release_id === manual) ? manual : ""} onChange={(e) => setPickByRun((m) => ({ ...m, [key]: e.target.value }))}>
              <option value="">{auto ? `Automatic: ${auto.c.model_name} ${auto.c.candidate}` : "Choose a model"}</option>
              {run && fitting.length > 0 && (
                <optgroup label="Fits this run">
                  {fitting.map(({ c }) => (
                    <option key={c.release_id} value={c.release_id}>
                      {c.model_name} {c.candidate} · {c.investigation}
                    </option>
                  ))}
                </optgroup>
              )}
              <optgroup label={run && fitting.length ? "Other models (typed conditions)" : "All served models"}>
                {ranked
                  .filter((x) => !(run && x.score >= 0))
                  .map(({ c }) => (
                    <option key={c.release_id} value={c.release_id}>
                      {c.model_name} {c.candidate} · {c.investigation}
                    </option>
                  ))}
              </optgroup>
            </select>
          </>
        )}
      </div>
      {modelsCap && !modelsCap.available && (
        <Gap title="The prediction models are switched off on this server">
          {modelsCap.detail}. The hand-written releases exist; the deployment has to start with <code>IGNIS_ENABLE_MODELS=true</code> to serve them.
        </Gap>
      )}
      {cards.error && <Gap title="Hand-written models are not served here">{cards.error instanceof V1Error ? cards.error.envelope.message : String(cards.error)}</Gap>}
      {cards.isLoading && <p className="muted">Loading the served models…</p>}
      {cards.data && cards.data.length === 0 && <Gap title="No model is served">This deployment lists no hand-written release.</Gap>}
      {!run && ranked.length > 0 && (
        <p className="muted lab-why">Pick a run above and the model that fits it is chosen for you, or choose any model in the menu and type its conditions.</p>
      )}
      {chosen && (
        <p className="muted lab-why">
          {!manual && run ? (
            <>
              <b>Chosen automatically</b>: {chosen.why.join(", ")}.
              {fitting.length > 1 && ` ${fitting.length - 1} other model${fitting.length > 2 ? "s" : ""} also fit${fitting.length > 2 ? "" : "s"}; switch in the menu.`}
            </>
          ) : run && chosen.score < 0 ? (
            <>This model was not written for {familyLabel(run.family)} runs; it takes typed conditions only.</>
          ) : null}
        </p>
      )}
      {run && cards.data && cards.data.length > 0 && !fitting.length && !manual && (
        <Gap title={`No served model is written for ${familyLabel(run.family)} runs`}>Choose another run, or pick a model in the menu and type its conditions.</Gap>
      )}
      {card && (!run || chosen!.score >= 0 || manual) && (
        <ModelPanel key={`${card.release_id}|${run?.id ?? ""}`} card={card} base={base} legacy={origin} run={run && chosen!.score >= 0 ? run : null} pickRun={pickRun} allRuns={runs} />
      )}
    </section>
  );
}

function ModelPanel({ card, base, legacy, run: fixed, pickRun, allRuns }: { card: ModelCard; base: string; legacy: "registry" | "fixtures"; run: ExperimentSummary | null; pickRun: (id: string) => void; allRuns: ExperimentSummary[] }) {
  const obsInput = card.inputs.find((i) => i.kind === "observation");
  const unit = obsInput?.unit_si ?? "";
  const d = display(unit);
  const [mode, setMode] = useState<Mode>(card.stored_runs ? "stored" : "typed");
  const [values, setValues] = useState<Record<string, string>>({});
  const set = (k: string, v: string) => setValues((m) => ({ ...m, [k]: v }));
  // A run picked above fixes the input; the cutoff starts at 40 % of its recorded series.
  const series = fixed?.quantities.find((q) => q.quantity === obsInput?.name);
  const extent = series ? series.archive_t_max_s : null;
  const [cutoff, setCutoff] = useState(() => (extent != null && extent > 0 ? String(+(extent * 0.4).toPrecision(3)) : fixed ? "3" : ""));
  const [run, setRun] = useState(fixed?.id ?? "");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [art, setArt] = useState<AnalysisArtifact | null>(null);
  const [observed, setObserved] = useState<ChartPoint[]>([]);
  const seq = useRef(0);

  // Stored runs offered: the train partition of the release's split (the research scope reads train only).
  const split = useQuery({
    queryKey: [legacy, "hw-split", card.release_id],
    queryFn: async () => {
      const r = await v1<{ split_id: string }>(base, `/models/${encodeURIComponent(card.release_id)}`);
      return call<ExperimentList>(legacy, `/experiments?split=${encodeURIComponent(r.split_id)}&partition=train`);
    },
    enabled: card.stored_runs && mode === "stored",
    retry: false,
    staleTime: 300_000,
  });
  const runs = split.data?.experiments ?? [];

  const fillExamples = () => {
    const v: Record<string, string> = {};
    for (const i of card.inputs) if (i.example !== undefined) v[i.name] = fmtExample(i.example);
    setValues(v);
    const obs = card.inputs.find((i) => i.kind === "observation")?.example;
    if (Array.isArray(obs) && obs.length) setCutoff(String((obs[obs.length - 1] as number[])[0]));
    else if (!cutoff) setCutoff("3");
  };

  const forecast = async () => {
    const me = ++seq.current;
    const live = () => me === seq.current;
    setBusy(true);
    setError(null);
    try {
      const c = Number(cutoff);
      if (!(cutoff.trim() !== "" && Number.isFinite(c) && c >= 0)) throw new Error("give a cutoff in seconds (≥ 0)");
      let input: unknown;
      let points: ChartPoint[] = [];
      if (mode === "stored") {
        if (!run) throw new Error("choose a run");
        input = { kind: "stored_run", run_id: run, cutoff_s: c };
        try {
          const o = await call<ObservationsAtCutoff>(legacy, `/experiments/${encodeURIComponent(run)}/observations?cutoff_s=${c}`);
          points = o.admitted.filter((x) => x.quantity === obsInput?.name && x.value_si != null).map((x) => ({ t: x.time_s, v: x.value_si! * d.k }));
        } catch {
          points = [];
        }
      } else {
        const conditions: Record<string, number | string> = {};
        const controls: { control: string; t_start_s: number; t_end_s: number | null; value_si: number | null }[] = [];
        let observations: { time_s: number; value_si: number }[] = [];
        let test_date: string | null = null;
        let phase: number | null = null;
        for (const i of card.inputs) {
          const raw = (values[i.name] ?? "").trim();
          if (!raw) {
            if (i.required) throw new Error(`${i.name} is required`);
            continue;
          }
          if (i.kind === "condition") {
            if (i.text) conditions[i.name] = raw;
            else {
              const x = Number(raw);
              if (!Number.isFinite(x)) throw new Error(`${i.name}: "${raw}" is not a number`);
              conditions[i.name] = x;
            }
          } else if (i.kind === "control") {
            for (const r of rows(raw)) {
              if (r[0] == null) throw new Error(`${i.name}: each stage needs a start time`);
              controls.push({ control: i.name, t_start_s: r[0], t_end_s: r[1] ?? null, value_si: r[2] ?? null });
            }
          } else if (i.kind === "observation") {
            observations = rows(raw).map((r) => {
              if (r[0] == null || r[1] == null) throw new Error(`${i.name}: each line is "t, value"`);
              return { time_s: r[0], value_si: r[1] };
            });
          } else if (i.kind === "test_date") test_date = raw;
          else if (i.kind === "phase_start") {
            phase = Number(raw);
            if (!Number.isFinite(phase)) throw new Error(`${i.name}: "${raw}" is not a number`);
          }
        }
        const admitted = await v1<ScenarioAdmitted>(base, `/models/${encodeURIComponent(card.release_id)}/scenarios`, {
          schema_version: "wsp.v1",
          release_id: card.release_id,
          cutoff_s: c,
          conditions,
          controls,
          observations,
          test_date,
          phase_established_at_s: phase,
        });
        input = { kind: "prefix", input_hash: admitted.input_hash };
        points = observations.map((p) => ({ t: p.time_s, v: p.value_si * d.k }));
      }
      const tr = await v1<ToolResult>(base, "/analyses", { schema_version: "wsp.v1", release_id: card.release_id, input });
      let job = tr.job as JobRef;
      for (let n = 0; !["succeeded", "failed", "cancelled"].includes(job.status); n++) {
        if (n > 600) throw new Error("the forecast job did not finish");
        await new Promise((r) => setTimeout(r, 400));
        job = await v1<JobRef>(base, `/jobs/${encodeURIComponent(job.job_id)}`);
      }
      if (job.status !== "succeeded") throw new V1Error({ code: job.error?.code ?? job.status, message: job.error?.message ?? "the forecast did not run" });
      const out = job.outputs?.[0];
      if (!out) throw new Error("the forecast job stored no artifact");
      const a = await v1<AnalysisArtifact>(base, `/artifacts/${encodeURIComponent(out.artifact_id)}`);
      if (!live()) return;
      setArt(a);
      setObserved(points);
    } catch (e) {
      if (!live()) return;
      setArt(null);
      setError(e instanceof V1Error ? `${words(e.envelope.code)}: ${e.envelope.message}` : String((e as Error).message ?? e));
    } finally {
      if (live()) setBusy(false);
    }
  };

  // With a run fixed above, the forecast runs by itself and follows the cutoff (debounced).
  const inTrain = !!fixed && runs.some((r) => r.id === fixed.id);
  const auto = !!fixed && mode === "stored" && inTrain;
  useEffect(() => {
    if (!auto) return;
    const t = setTimeout(() => void forecast(), 350);
    return () => clearTimeout(t);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auto, cutoff]);
  const slide = auto && extent != null && extent > 0;
  const trainWithData = useMemo(() => {
    const ids = new Set(runs.map((r) => r.id));
    return allRuns.filter((r) => ids.has(r.id) && (!obsInput || hasSeries(r))).slice(0, 6);
  }, [runs, allRuns, obsInput]);

  const bands: Bands | null = useMemo(
    () => (art && art.support.admitted && (art.quantiles?.length ?? 0) > 0 ? { times: art.times_s ?? [], levels: art.quantile_levels ?? [], q: (art.quantiles ?? []).map((r) => r.map((v) => v * d.k)) } : null),
    [art, d.k],
  );
  const c = Number(cutoff) || 0;
  const xMax = Math.max(c, ...(bands?.times ?? [0]), ...observed.map((p) => p.t), 1e-6);

  return (
    <div className={obsInput ? "lab" : "lab lab-one"}>
      {obsInput && (
        <section className="sheet" aria-label="Forecast chart">
          {bands || observed.length > 0 ? (
            <>
          <ul className="legend legend-inline" aria-label="Chart legend">
            <li>
              <span className="glyph g-observed" style={{ color: "var(--observed)" }} /> {mode === "typed" ? "Typed before the cutoff (synthetic)" : "Observed before the cutoff"}
            </li>
            <li>
              <span className="glyph g-predicted" /> Model output: quantile bands, dashed median
            </li>
            <li>
              <span className="glyph g-cutoff" /> Cutoff
            </li>
          </ul>
          <ForecastChart observed={observed} revealed={null} bands={bands} cutoff={c} xMax={xMax} yLabel={`${obsInput ? words(obsInput.name) : ""} [${d.unit}]`} />
            </>
          ) : (
            <p className="muted lab-chart-empty">{busy ? "Running the forecast…" : error ? "No forecast to draw." : "The forecast is drawn here once it has run."}</p>
          )}
          {slide && (
            <div className="cutoff-row">
              <label htmlFor="hw-cutoff">Cutoff, s</label>
              <input id="hw-cutoff" type="range" min={0} max={extent!} step={+(extent! / 200).toPrecision(2)} value={Number(cutoff) || 0} onChange={(e) => setCutoff(e.target.value)} aria-valuetext={`${cutoff} seconds of ${extent!.toPrecision(3)}`} />
              <input className="field num" type="number" aria-label="Cutoff in seconds" min={0} step="any" value={cutoff} onChange={(e) => setCutoff(e.target.value)} />
            </div>
          )}
        </section>
      )}
      <div className="steps">
        <Step n={1} done title={`${card.model_name} ${card.candidate}`} tag={<span className="tag tag-warn">experimental</span>}>
          <p>
            <b>Hand-written</b> (literal weights an author model typed, nothing fitted) · <b>{authorshipLabel(card)}</b> · trained on {words(card.trained_on)} · {words(card.state)}
          </p>
          <p>{card.forecasts}</p>
          <p className="muted" style={{ fontSize: 12 }}>
            {card.authorship_note}
          </p>
          <details className="route">
            <summary>
              <b>Reported scores and limits</b> <span>development numbers, with their source</span>
            </summary>
            <ul style={{ paddingLeft: 18 }}>
              {card.scores.map((s) => (
                <li key={s.metric}>
                  {s.metric}: <b className="num">{s.value}</b>
                  {s.comparison ? ` (${s.comparison})` : ""} <span className="muted">· {s.source}</span>
                </li>
              ))}
            </ul>
            <ul style={{ paddingLeft: 18 }}>
              {card.limits.map((l) => (
                <li key={l}>{l}</li>
              ))}
            </ul>
            <p className="muted" style={{ fontSize: 12 }}>
              Cite as: {card.cite}
              {card.card_doc ? ` Model card: ${card.card_doc}.` : ""}
            </p>
          </details>
        </Step>

        <Step n={2} done={false} title="Input">
          <p className="muted" style={{ fontSize: 12 }}>
            {card.inputs_note}
          </p>
          <div className="toolbar" role="radiogroup" aria-label="Input">
            <button className={`btn btn-s${mode === "stored" ? " btn-primary" : ""}`} disabled={!card.stored_runs} onClick={() => setMode("stored")} aria-pressed={mode === "stored"}>
              Stored run (train partition)
            </button>
            <button className={`btn btn-s${mode === "typed" ? " btn-primary" : ""}`} onClick={() => setMode("typed")} aria-pressed={mode === "typed"}>
              Typed conditions
            </button>
          </div>
          {mode === "stored" ? (
            fixed ? (
              <>
                <p>
                  <b>{runLabel(fixed)}</b> <code>{fixed.id}</code>
                </p>
                {split.isLoading && <p className="muted">Checking the model's train partition…</p>}
                {split.error && <Gap title="The model's runs could not be listed">{String((split.error as Error).message ?? split.error)}</Gap>}
                {split.data && !inTrain && (
                  <Caution title="This run is outside the model's train partition">
                    This deployment builds a stored prefix only from train runs, so this run's own data is not fed to this model here (validation and test runs stay sealed for evaluation).
                    {trainWithData.length > 0
                      ? " Runs this model can read:"
                      : " No run of its train partition has a recorded series in IGNIS yet; switch to Typed conditions to forecast from values you enter."}
                  </Caution>
                )}
                {split.data && !inTrain && trainWithData.length > 0 && (
                  <div className="toolbar">
                    {trainWithData.map((r) => (
                      <button key={r.id} className="btn btn-s" onClick={() => pickRun(r.id)}>
                        {runLabel(r)}
                      </button>
                    ))}
                  </div>
                )}
                {inTrain && card.authorship === "data_informed" && (
                  <p className="muted" style={{ fontSize: 12 }}>
                    The authors read this train run while writing the model, so this is inspection, not a test.
                  </p>
                )}
              </>
            ) : (
            <>
              <p className="muted" style={{ fontSize: 12 }}>
                Runs of the release's train partition only; the authors of a data-informed release read these runs, so this is inspection, not a test.
              </p>
              <select className="field" value={run} onChange={(e) => setRun(e.target.value)} aria-label="Stored run">
                <option value="">{split.isLoading ? "Loading runs…" : runs.length ? "Choose a run" : "No stored runs in this data source"}</option>
                {runs.map((r) => (
                  <option key={r.id} value={r.id}>
                    {r.id}
                  </option>
                ))}
              </select>
            </>
            )
          ) : (
            <>
              <Caution title="A typed scenario is not a NASA run">
                What you type is admitted as a synthetic input (evidence class synthetic); the forecast is model output for those conditions.
              </Caution>
              <button className="btn btn-s btn-quiet" style={{ justifySelf: "start" }} onClick={fillExamples}>
                Fill example values (not a measurement)
              </button>
              <dl className="kv">
                {card.inputs.map((i) => (
                  <InputField key={i.name} input={i} value={values[i.name] ?? ""} onChange={(v) => set(i.name, v)} />
                ))}
              </dl>
            </>
          )}
          {slide ? null : (
            <div className="cutoff-row">
              <label htmlFor="hw-cutoff">Cutoff, s</label>
              <input id="hw-cutoff" className="field num" type="number" min={0} step="any" value={cutoff} onChange={(e) => setCutoff(e.target.value)} onKeyDown={(e) => e.key === "Enter" && void forecast()} />
              <button className="btn btn-s btn-primary" disabled={busy || (mode === "stored" && !!fixed && !inTrain)} onClick={forecast}>
                {busy ? "Running the forward…" : "Forecast"}
              </button>
            </div>
          )}
          {auto && (
            <p className="muted" style={{ fontSize: 12 }} aria-live="polite">
              {busy ? "Updating the forecast…" : "The forecast follows the cutoff."}
            </p>
          )}
        </Step>

        <Step n={3} done={!!art} title="Model output">
          {error && <Gap title="No forecast">{error}</Gap>}
          {art && <SupportNote artifact={art} />}
          {art && !art.support.admitted && <p>The model refuses this input and gives no number for it; the reasons are above.</p>}
          {art && art.support.admitted && <EventStack artifact={art} />}
          {art && (
            <p className="muted" style={{ fontSize: 12 }}>
              model output, not a measurement · prefix <code>{art.prefix_hash.slice(0, 19)}…</code>
            </p>
          )}
        </Step>
      </div>
    </div>
  );
}

function InputField({ input, value, onChange }: { input: ModelInput; value: string; onChange: (v: string) => void }) {
  const multi = input.kind === "observation" || input.kind === "control";
  const id = `hw-in-${input.name}`;
  return (
    <>
      <dt>
        <label htmlFor={id}>
          {input.name}
          {input.required ? "" : " (optional)"}
        </label>
      </dt>
      <dd>
        {multi ? (
          <textarea id={id} className="field num" rows={3} value={value} onChange={(e) => onChange(e.target.value)} placeholder={input.kind === "control" ? "start s, end s, value per line" : `t s, value [${input.unit_si}] per line`} />
        ) : (
          <input id={id} className="field" value={value} onChange={(e) => onChange(e.target.value)} placeholder={input.text ? "text" : input.kind === "test_date" ? "YYYY-MM-DD" : input.unit_si} />
        )}
        <span className="muted" style={{ fontSize: 12, display: "block" }}>
          {input.note}
          {!input.text && input.unit_si !== "1" ? ` [${input.unit_si}]` : ""}
        </span>
      </dd>
    </>
  );
}
