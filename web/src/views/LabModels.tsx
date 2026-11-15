// Hand-written models in the forecast lab: the releases the research server serves (/api/v1), each
// with what it is (hand-written, experimental, zero-shot or data-informed), what it reads and what
// its authors reported. A forecast runs the release's strict forward on a stored run of the train
// partition or on conditions typed here; a typed scenario is a synthetic input, never a NASA run.
import { useEffect, useMemo, useRef, useState } from "react";
import { useQueries, useQuery } from "@tanstack/react-query";
import type { AnalysisArtifact, EndpointRecord, JobRef, ModelCard, ModelInput, ScenarioAdmitted, ToolResult } from "../contracts/generated";
import { call, type ExperimentDetail, type ExperimentList, type ExperimentSummary, type ObservationsAtCutoff } from "../api";
import { ForecastChart, type Bands, type ChartPoint } from "../components/ForecastChart";
import { useData } from "../shell/data";
import { Caution, Gap, words } from "../ui/common";
import { EventStack, SupportNote } from "./Lab";
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
  if (card.stored_runs) score += 1;
  return { score, why };
}

/** The served hand-written releases, and the v1 base they are served from (null when none is). */
export function useModelCards() {
  const { researchMode, caps } = useData();
  const base = researchMode === "live" ? "/api/v1" : researchMode === "fixtures" ? "/api/fixtures/v1" : null;
  // Only the live server says whether the model families are switched on; fixtures always serve theirs.
  const modelsCap = researchMode === "live" ? caps.data?.models : undefined;
  const cards = useQuery({ queryKey: [researchMode, "model-cards"], queryFn: () => v1<ModelCard[]>(base!, "/model-cards"), enabled: !!base && modelsCap?.available !== false, retry: false });
  return { base, modelsCap, cards };
}

/** The runs a release may read here: the train partition of its split (validation and test stay sealed). */
const trainOf = async (base: string, legacy: "registry" | "fixtures", releaseId: string) => {
  const r = await v1<{ split_id: string }>(base, `/models/${encodeURIComponent(releaseId)}`);
  return call<ExperimentList>(legacy, `/experiments?split=${encodeURIComponent(r.split_id)}&partition=train`);
};
const trainQuery = (base: string | null, legacy: "registry" | "fixtures", releaseId: string, enabled = true) => ({
  queryKey: [legacy, "hw-split", releaseId],
  queryFn: () => trainOf(base!, legacy, releaseId),
  enabled: !!base && enabled,
  retry: false,
  staleTime: 300_000,
});

/**
 * What a served model can do with each run: forecast it from the stored record ("ready"), forecast it
 * from its conditions filled in for the user ("conditions"), forecast it only with values the user
 * types because the record lacks some ("typed"), nothing because the run is sealed for evaluation
 * ("sealed"), or nothing because no model is written for its family ("none").
 */
export type Reach = "ready" | "conditions" | "typed" | "sealed" | "none";
export function useReach(runs: ExperimentSummary[]): Map<string, Reach> | null {
  const { origin } = useData();
  const { base, cards } = useModelCards();
  const stored = (cards.data ?? []).filter((c) => c.stored_runs);
  const trains = useQueries({ queries: stored.map((c) => trainQuery(base, origin, c.release_id)) });
  if (!base || cards.isError) return null;
  if (!cards.data || trains.some((t) => t.isLoading)) return null;
  const readable = stored.map((c, k) => ({ c, ids: new Set((trains[k].data?.experiments ?? []).map((x) => x.id)), needsSeries: c.inputs.some((i) => i.kind === "observation") }));
  const typed = cards.data.filter((c) => !c.stored_runs);
  const storedFamilies = new Set(stored.map((c) => c.family));
  const m = new Map<string, Reach>();
  for (const r of runs) {
    const ready = readable.some(({ c, ids, needsSeries }) => c.family === r.family && ids.has(r.id) && (!needsSeries || hasSeries(r)));
    const mine = typed.filter((c) => c.family === r.family);
    // "conditions" only when the record fills every required input; otherwise the user types the rest.
    const complete = mine.some((c) => {
      const v = fromRun(c, r);
      return c.inputs.every((i) => !i.required || v[i.name] !== undefined);
    });
    m.set(r.id, ready ? "ready" : complete ? "conditions" : mine.length ? "typed" : storedFamilies.has(r.family) ? "sealed" : "none");
  }
  return m;
}

/** A model's inputs filled from what the NASA record holds for a run; anything it lacks stays empty. */
function fromRun(card: ModelCard, r: ExperimentSummary): Record<string, string> {
  const v: Record<string, string> = {};
  const num = (x: number) => String(+x.toPrecision(6));
  for (const i of card.inputs) {
    const cell = r.conditions?.[i.name];
    if (i.kind === "condition") {
      if (i.text) {
        if (/fuel|species|material/.test(i.name) && r.fuel_or_material) v[i.name] = r.fuel_or_material;
      } else if (cell?.value_si != null) v[i.name] = num(cell.value_si);
      else if (i.name === "width" && r.fuel_or_material) {
        // BASS names the sample by its width ("100 micron PMMA film   2 cm wide").
        const w = r.fuel_or_material.match(/([\d.]+)\s*(mm|cm)\s+wide/i);
        if (w) v[i.name] = num(Number(w[1]) * (w[2].toLowerCase() === "cm" ? 0.01 : 0.001));
      }
    } else if (i.kind === "control" && cell?.value_si != null) v[i.name] = `0, , ${num(cell.value_si)}`;
    else if (i.kind === "test_date") {
      const d = r.session_id?.match(/\d{4}-\d{2}-\d{2}/);
      if (d) v[i.name] = d[0];
    }
  }
  return v;
}

export function LabModels({ run, pickRun, runs }: { run: ExperimentSummary | null; pickRun: (id: string) => void; runs: ExperimentSummary[] }) {
  const { origin } = useData();
  const { base, modelsCap, cards } = useModelCards();
  const [pickByRun, setPickByRun] = useState<Record<string, string>>({});
  // With no run picked, the panel stays closed until the user asks to type conditions.
  const [typing, setTyping] = useState(false);
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
  const open = !!run || typing;
  const label = (c: ModelCard) => `${c.investigation} (${c.model_name} ${c.candidate})`;
  return (
    <section className="sheet lab-models" aria-labelledby="hw-h">
      <h2 id="hw-h">Model and forecast</h2>
      {modelsCap && !modelsCap.available && (
        <Gap title="The prediction models are switched off on this server">
          {modelsCap.detail}. The hand-written releases exist; the deployment has to start with <code>IGNIS_ENABLE_MODELS=true</code> to serve them.
        </Gap>
      )}
      {cards.error && <Gap title="Hand-written models are not served here">{cards.error instanceof V1Error ? cards.error.envelope.message : String(cards.error)}</Gap>}
      {cards.isLoading && <p className="muted">Loading the served models…</p>}
      {cards.data && cards.data.length === 0 && <Gap title="No model is served">This deployment lists no hand-written release.</Gap>}

      {!open && ranked.length > 0 && (
        <div className="lab-start">
          <p className="muted">Pick a run above and the model written for it is chosen for you.</p>
          <button className="btn btn-s" onClick={() => setTyping(true)}>
            Forecast from conditions you type instead
          </button>
        </div>
      )}

      {open && ranked.length > 0 && (
        <div className="lab-model-row">
          <label htmlFor="hw-model">Model</label>
          <select id="hw-model" className="field" value={manual && ranked.some((x) => x.c.release_id === manual) ? manual : ""} onChange={(e) => setPickByRun((m) => ({ ...m, [key]: e.target.value }))}>
            <option value="">{auto ? (run ? `Best fit: ${label(auto.c)}` : label(auto.c)) : "Choose a model"}</option>
            {run && fitting.length > 0 && (
              <optgroup label="Written for this run's family">
                {fitting.map(({ c }) => (
                  <option key={c.release_id} value={c.release_id}>
                    {label(c)}
                  </option>
                ))}
              </optgroup>
            )}
            <optgroup label={run && fitting.length ? "Other models (you type the conditions)" : "All served models"}>
              {ranked
                .filter((x) => !(run && x.score >= 0) && x !== (run ? null : auto))
                .map(({ c }) => (
                  <option key={c.release_id} value={c.release_id}>
                    {label(c)}
                  </option>
                ))}
            </optgroup>
          </select>
          {!run && (
            <button className="btn btn-s btn-quiet" onClick={() => setTyping(false)}>
              Cancel
            </button>
          )}
        </div>
      )}
      {open && chosen && run && (
        <p className="muted lab-why">
          {!manual ? (
            <>
              Chosen automatically: {chosen.why.join(", ")}.
              {fitting.length > 1 && ` ${fitting.length - 1} other model${fitting.length > 2 ? "s" : ""} also fit${fitting.length > 2 ? "" : "s"}.`}
            </>
          ) : chosen.score < 0 ? (
            <>This model was not written for {familyLabel(run.family)} runs: you type its conditions.</>
          ) : null}
        </p>
      )}
      {run && cards.data && cards.data.length > 0 && !fitting.length && !manual && (
        <Gap title={`No served model is written for ${familyLabel(run.family)} runs`}>Choose another run, or pick a model in the menu and type its conditions.</Gap>
      )}
      {open && card && (!run || chosen!.score >= 0 || manual) && (
        <ModelPanel key={`${card.release_id}|${run?.id ?? ""}`} card={card} base={base} legacy={origin} run={run && chosen!.score >= 0 ? run : null} pickRun={pickRun} allRuns={runs} />
      )}
    </section>
  );
}

function ModelPanel({ card, base, legacy, run: fixed, pickRun, allRuns }: { card: ModelCard; base: string; legacy: "registry" | "fixtures"; run: ExperimentSummary | null; pickRun: (id: string) => void; allRuns: ExperimentSummary[] }) {
  const obsInput = card.inputs.find((i) => i.kind === "observation");
  const unit = obsInput?.unit_si ?? "";
  const d = display(unit);
  // Without a run the panel only opens to type conditions; with one, a model that reads stored runs reads it.
  const [mode, setMode] = useState<Mode>(card.stored_runs && fixed ? "stored" : "typed");
  // Typed conditions start from what the NASA record holds for the run picked above.
  const recorded = useMemo(() => (fixed ? fromRun(card, fixed) : {}), [card, fixed]);
  const [values, setValues] = useState<Record<string, string>>(recorded);
  const set = (k: string, v: string) => setValues((m) => ({ ...m, [k]: v }));
  // A run picked above fixes the input; the cutoff starts at 40 % of its recorded series.
  const series = fixed?.quantities.find((q) => q.quantity === obsInput?.name);
  const extent = series ? series.archive_t_max_s : null;
  const [cutoff, setCutoff] = useState(() => (extent != null && extent > 0 ? String(+(extent * 0.4).toPrecision(3)) : fixed ? "3" : ""));
  const run = fixed?.id ?? "";
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [art, setArt] = useState<AnalysisArtifact | null>(null);
  const [observed, setObserved] = useState<ChartPoint[]>([]);
  const seq = useRef(0);

  // Stored runs offered: the train partition of the release's split (the research scope reads train only).
  const split = useQuery(trainQuery(base, legacy, card.release_id, card.stored_runs && !!fixed));
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
  // An event model (no observed series) is read against how the run actually ended: the server
  // serves the endpoint only for train runs, and the model never receives it.
  const ended = useQuery({
    queryKey: [legacy, "run-endpoint", fixed?.id],
    queryFn: () => call<ExperimentDetail>(legacy, `/experiments/${encodeURIComponent(fixed!.id)}`),
    enabled: auto && !obsInput,
    retry: false,
    staleTime: 300_000,
  });
  const endpoint = auto && !obsInput ? ended.data?.endpoint ?? null : null;
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

  const extentMax = extent != null && extent > 0 ? extent : null;
  const moveCutoff = (t: number) => setCutoff(String(+Math.max(0, extentMax != null ? Math.min(t, extentMax) : t).toPrecision(3)));
  const summary = bands ? quantileSummary(bands, c) : null;
  const quantityLabel = obsInput ? words(obsInput.name) : "";
  const inspectionNote = inTrain && card.authorship === "data_informed" ? "The authors read this run while writing the model, so this is inspection, not a test." : null;

  return (
    <div className="fc">
      <div className="fc-model">
        <div className="fc-model-name">
          <b>{card.investigation}</b>
          <span className="fc-model-meta">
            <span className="muted">
              {card.model_name} {card.candidate}
            </span>
            <span className="tag tag-warn" title="Not qualified by evaluation: read it as an experimental model output">
              experimental
            </span>
            <span className="tag" title="Literal weights an author model typed; nothing was fitted">
              hand-written · {authorshipLabel(card)}
            </span>
          </span>
        </div>
        {fixed && card.stored_runs && (
          <span className="fc-model-actions seg" role="group" aria-label="Model input">
            <button aria-pressed={mode === "stored"} onClick={() => setMode("stored")}>
              The NASA run
            </button>
            <button aria-pressed={mode === "typed"} onClick={() => setMode("typed")}>
              Conditions I type
            </button>
          </span>
        )}
      </div>
      <details className="fc-about">
        <summary>What this model forecasts, its scores and limits</summary>
        <p>{card.forecasts}</p>
        <p className="muted">{card.authorship_note}</p>
        {card.scores.length > 0 && (
          <>
            <h4>Reported development scores</h4>
            <ul>
              {card.scores.map((s) => (
                <li key={s.metric}>
                  {s.metric}: <b className="num">{s.value}</b>
                  {s.comparison ? ` (${s.comparison})` : ""} <span className="muted">· {s.source}</span>
                </li>
              ))}
            </ul>
          </>
        )}
        {card.limits.length > 0 && (
          <>
            <h4>Limits</h4>
            <ul>
              {card.limits.map((l) => (
                <li key={l}>{l}</li>
              ))}
            </ul>
          </>
        )}
        <p className="muted">
          Inputs: {card.inputs_note} Cite as: {card.cite}
          {card.card_doc ? ` Model card: ${card.card_doc}.` : ""}
        </p>
      </details>

      {mode === "stored" && fixed && split.isLoading && <p className="muted">Checking which runs this model may read…</p>}
      {mode === "stored" && fixed && split.error && <Gap title="The model's runs could not be listed">{String((split.error as Error).message ?? split.error)}</Gap>}
      {mode === "stored" && fixed && split.data && !inTrain && (
        <Caution title="This run is sealed for evaluation">
          The model reads only its training runs here, so it cannot be fed this one. {trainWithData.length > 0 ? "Pick one it can read:" : "None of its training runs has a recorded series in IGNIS yet."}
        </Caution>
      )}
      {mode === "stored" && fixed && split.data && !inTrain && trainWithData.length > 0 && (
        <div className="toolbar">
          {trainWithData.map((r) => (
            <button key={r.id} className="btn btn-s" onClick={() => pickRun(r.id)}>
              {runLabel(r)}
            </button>
          ))}
        </div>
      )}

      {mode === "typed" && (
        <div className="fc-typed">
          <Caution title={fixed ? `Conditions from ${runLabel(fixed)}, entered as your own input` : "Your conditions are a synthetic input, not a NASA run"}>
            {fixed ? "Values the NASA record holds are filled in; complete the rest. The forecast is model output for these values, not a reading of the run." : "The forecast is model output for the values you enter."}
          </Caution>
          <dl className="fc-inputs">
            {card.inputs.map((i) => (
              <InputField
                key={i.name}
                input={i}
                value={values[i.name] ?? ""}
                onChange={(v) => set(i.name, v)}
                source={fixed ? (recorded[i.name] !== undefined && recorded[i.name] === values[i.name] ? "record" : !(values[i.name] ?? "").trim() && i.required ? "missing" : null) : null}
              />
            ))}
          </dl>
          <button className="btn btn-s btn-quiet" style={{ justifySelf: "start" }} onClick={fillExamples}>
            {fixed ? "Replace with example values" : "Fill example values"}
          </button>
        </div>
      )}

      {!slide && !(mode === "stored" && fixed && !inTrain) && (
        <div className="fc-row">
          <label htmlFor="hw-cutoff">{obsInput ? "The model sees the first" : "Start the forecast at"}</label>
          <input id="hw-cutoff" className="field num fc-num" type="number" min={0} step="any" value={cutoff} onChange={(e) => setCutoff(e.target.value)} onKeyDown={(e) => e.key === "Enter" && void forecast()} />
          <span>s</span>
          {auto ? (
            busy && <span className="muted fc-busy">Forecasting…</span>
          ) : (
            <button className="btn btn-s btn-primary" disabled={busy} onClick={forecast}>
              {busy ? "Forecasting…" : "Forecast"}
            </button>
          )}
        </div>
      )}

      {error && <Gap title="No forecast">{error}</Gap>}
      {art && !art.support.admitted && (
        <>
          <SupportNote artifact={art} />
          <p>The model refuses this input and gives no number for it.</p>
        </>
      )}

      {(art?.support.admitted || slide || (obsInput && observed.length > 0)) && (
        <section className="sheet fc-main" aria-label="Forecast">
          <div className="fc-answer" aria-live="polite">
            {summary ? (
              <p>
                Seeing the first <b className="num">{fmtNum(c)} s</b>, the model expects the {quantityLabel} to be{" "}
                <b className="num">
                  {fmtNum(summary.median)} {d.unit}
                </b>{" "}
                at <span className="num">{fmtNum(summary.t)} s</span>{" "}
                <span className="muted num">
                  ({summary.pct}% range {fmtNum(summary.lo)}–{fmtNum(summary.hi)} {d.unit})
                </span>
                .
              </p>
            ) : art?.support.admitted && !obsInput ? (
              <p>
                Probability of each outcome by <b className="num">{fmtNum((art.times_s ?? [0]).slice(-1)[0])} s</b> (model output):
              </p>
            ) : (
              <p className="muted">{busy ? "Forecasting…" : "The forecast appears here."}</p>
            )}
            {busy && <span className="fc-busy muted">updating…</span>}
          </div>
          {art?.support.admitted && <OutcomeBar artifact={art} />}
          {art?.support.admitted && endpoint && <RecordedEnd endpoint={endpoint} />}
          {obsInput && (
            <>
              <ul className="legend legend-inline fc-legend" aria-label="Chart legend">
                <li>
                  <span className="glyph g-observed" style={{ color: "var(--observed)" }} /> {mode === "typed" ? "Your values (synthetic)" : "Measured, given to the model"}
                </li>
                <li>
                  <span className="glyph g-predicted" /> Model: median (dashed) and uncertainty band
                </li>
              </ul>
              <ForecastChart
                observed={observed}
                revealed={null}
                bands={bands}
                cutoff={c}
                xMax={xMax}
                unit={d.unit}
                yLabel={`${quantityLabel} [${d.unit}]`}
                onCutoff={slide ? moveCutoff : undefined}
              />
            </>
          )}
          {slide && (
            <div className="fc-cutoff">
              <div className="fc-cutoff-head">
                <label htmlFor="hw-cutoff">
                  The model sees the first{" "}
                  <input id="hw-cutoff" className="field num fc-num" type="number" min={0} max={extentMax!} step="any" value={cutoff} onChange={(e) => setCutoff(e.target.value)} /> s of the{" "}
                  <span className="num">{fmtNum(extentMax!)} s</span> recorded
                </label>
                <span className="fc-presets" role="group" aria-label="Cutoff presets">
                  {[0.1, 0.25, 0.5, 0.75].map((f) => (
                    <button key={f} className="btn btn-s btn-quiet" onClick={() => moveCutoff(extentMax! * f)}>
                      {f * 100}%
                    </button>
                  ))}
                </span>
              </div>
              <input
                type="range"
                className="fc-range"
                min={0}
                max={extentMax!}
                step={+(extentMax! / 400).toPrecision(2)}
                value={Number(cutoff) || 0}
                onChange={(e) => moveCutoff(+e.target.value)}
                aria-label="Cutoff"
                aria-valuetext={`${cutoff} seconds of ${fmtNum(extentMax!)}`}
              />
              <p className="muted fc-hint">Drag the line on the chart or the slider: the forecast follows. Hover the chart to read values.</p>
            </div>
          )}
          <p className="muted fc-foot">
            Model output, not a measurement.{inspectionNote ? ` ${inspectionNote}` : ""}
          </p>
          {art && (
            <details className="fc-tech">
              <summary>Technical details</summary>
              <SupportNote artifact={art} />
              {art.support.admitted && <EventStack artifact={art} />}
              <p className="muted">
                release <code>{card.release_id.slice(0, 19)}…</code> · prefix <code>{art.prefix_hash.slice(0, 19)}…</code> · artifact <code>{art.id}</code>
              </p>
            </details>
          )}
        </section>
      )}
    </div>
  );
}

const fmtNum = (x: number) => (!Number.isFinite(x) ? "—" : Math.abs(x) >= 100 ? x.toFixed(0) : Math.abs(x) >= 10 ? x.toFixed(1) : +x.toPrecision(3) + "");

/** Median and outer band at the last forecast time. */
function quantileSummary(b: Bands, cutoff: number): { t: number; median: number; lo: number; hi: number; pct: number } | null {
  const n = b.times.length;
  if (!n || !b.levels.length) return null;
  const j = n - 1;
  const mi = b.levels.findIndex((x) => Math.abs(x - 0.5) < 1e-9);
  const last = b.levels.length - 1;
  if (b.times[j] <= cutoff && n > 1) return null;
  return {
    t: b.times[j],
    median: (mi >= 0 ? b.q[mi] : b.q[Math.floor(last / 2)])[j],
    lo: b.q[0][j],
    hi: b.q[last][j],
    pct: Math.round((b.levels[last] - b.levels[0]) * 100),
  };
}

const OUTCOME: Record<string, string> = {
  active_supported: "still burning",
  active: "still burning",
  hot_phase_end: "hot flame ends",
  cool_flame: "cool flame",
  disruption: "droplet disrupted",
  extinction: "extinguished",
  operator_termination: "stopped by the test script",
  flow_stop: "flow stopped",
  unresolved: "unresolved",
  unresolved_support: "unresolved",
};
const OUTCOME_COLORS = ["var(--ev1)", "var(--ev2)", "var(--ev3)", "var(--ev4)"];
/** Fixed colours by meaning, so "still burning" never looks like an ending. */
const outcomeColor = (c: string, k: number) =>
  c.startsWith("active")
    ? "color-mix(in srgb, var(--ink) 22%, transparent)"
    : c.startsWith("unresolved")
      ? "var(--wash)"
      : c === "disruption" || c === "operator_termination" || c === "flow_stop"
        ? "var(--revealed)"
        : c === "extinction" || c === "hot_phase_end"
          ? "var(--model)"
          : OUTCOME_COLORS[k % OUTCOME_COLORS.length];

/** Outcome probabilities at the last forecast time, as one bar plus a legend. */
function OutcomeBar({ artifact }: { artifact: AnalysisArtifact }) {
  const cats = artifact.event_categories ?? [];
  const cdf = artifact.event_cdf ?? [];
  const n = artifact.times_s?.length ?? 0;
  if (!cats.length || !n) return null;
  const last = n - 1;
  const parts = cats.map((c, k) => ({ c, p: cdf[k]?.[last] ?? 0, color: outcomeColor(c, k) }));
  const rest = 1 - parts.reduce((s, x) => s + x.p, 0);
  if (!cats.some((c) => c.startsWith("active")) && rest > 0.0005) parts.push({ c: "active", p: rest, color: outcomeColor("active", 0) });
  const shown = parts.filter((x) => x.p > 0.0005);
  return (
    <div className="fc-outcomes">
      <div className="fc-obar" role="img" aria-label={shown.map((x) => `${OUTCOME[x.c] ?? words(x.c)} ${(x.p * 100).toFixed(0)}%`).join(", ")}>
        {shown.map((x) => (
          <span key={x.c} style={{ width: `${x.p * 100}%`, background: x.color }} />
        ))}
      </div>
      <ul className="fc-olegend">
        {shown.map((x) => (
          <li key={x.c}>
            <span className="swatch" style={{ background: x.color }} aria-hidden="true" /> {OUTCOME[x.c] ?? words(x.c)} <b className="num">{x.p >= 0.995 ? ">99" : x.p < 0.01 ? "<1" : (x.p * 100).toFixed(0)}%</b>
          </li>
        ))}
        <li className="muted">by {fmtNum(artifact.times_s![last])} s</li>
      </ul>
    </div>
  );
}

/** How the run ended in NASA's record, next to the outcome probabilities. Never a model input. */
function RecordedEnd({ endpoint: e }: { endpoint: EndpointRecord }) {
  const causes = (e.allowed_causes ?? []).map((c) => OUTCOME[c] ?? words(c));
  const what = causes.length ? causes.join(" or ") : "ended";
  const at = (t: number | null | undefined) => (t != null ? <b className="num">{fmtNum(t)} s</b> : null);
  const text =
    e.censoring === "event" ? (
      <>
        {what} at {at(e.first_terminal_s) ?? "an unrecorded time"}
      </>
    ) : e.censoring === "interval" ? (
      <>
        {what} between {at(e.last_confirmed_active_s) ?? "?"} and {at(e.first_terminal_s) ?? "?"}
      </>
    ) : e.censoring === "scheduled_stop" ? (
      <>stopped by the test script{e.intervention_s != null ? <> at {at(e.intervention_s)}</> : null}, before any other ending</>
    ) : e.censoring === "right" ? (
      <>still burning when the record ends{e.last_confirmed_active_s != null ? <> at {at(e.last_confirmed_active_s)}</> : null}</>
    ) : (
      <>how it ended is not recorded</>
    );
  return (
    <p className="fc-recorded" title={e.original_comment ?? e.original_label}>
      <span className="glyph g-revealed" aria-hidden="true" /> <b>Recorded</b>: {text}
      {e.resolution !== "resolved" ? ` (${words(e.resolution)})` : ""}.{" "}
      <span className="muted">From the NASA record; the model did not see it.</span>
    </p>
  );
}

/** A readable label from the input's note ("oxygen mole fraction"), the raw name kept as a hint. */
function inputLabel(input: ModelInput): { label: string; rest: string } {
  const note = (input.note ?? "").trim();
  // The label is the note up to its first colon or semicolon; a long parenthetical moves to the hint.
  const stop = note.search(/[:;]\s/);
  let head = (stop > 0 ? note.slice(0, stop) : note).trim();
  const rest: string[] = [];
  head = head.replace(/\s*\(([^)]{11,})\)/g, (_, inner: string) => (rest.push(inner), "")).trim();
  if (stop > 0) rest.push(note.slice(stop + 1).trim());
  if (!head || head.length > 60) return { label: words(input.name), rest: note };
  return { label: head[0].toUpperCase() + head.slice(1), rest: rest.filter(Boolean).join("; ") };
}

function InputField({ input, value, onChange, source }: { input: ModelInput; value: string; onChange: (v: string) => void; source: "record" | "missing" | null }) {
  const multi = input.kind === "observation" || input.kind === "control";
  const id = `hw-in-${input.name}`;
  const { label, rest } = inputLabel(input);
  const unit = !input.text && input.unit_si && input.unit_si !== "1" ? input.unit_si : null;
  const example = input.example !== undefined && input.example !== null && !multi ? fmtExample(input.example) : "";
  return (
    <div className={`fc-in${source ? ` fc-in-${source}` : ""}`}>
      <dt>
        <label htmlFor={id}>
          {label}
          {unit && <span className="muted"> [{unit}]</span>}
          {!input.required && <span className="muted"> · optional</span>}
        </label>
        <code className="fc-in-name">{input.name}</code>
      </dt>
      <dd>
        {multi ? (
          <textarea
            id={id}
            className="field num"
            rows={3}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder={input.kind === "control" ? "one stage per line: start s, end s (empty = open), value" : `one point per line: t s, value${unit ? ` [${unit}]` : ""}`}
          />
        ) : (
          <input id={id} className="field num" value={value} onChange={(e) => onChange(e.target.value)} placeholder={input.kind === "test_date" ? "YYYY-MM-DD" : example ? `e.g. ${example}` : input.text ? "text" : ""} />
        )}
        {(rest || source) && (
          <span className="muted fc-in-note">
            {source === "record" && <span className="fc-in-src">from the NASA record</span>}
            {source === "missing" && <span className="fc-in-src">not in the NASA record: enter it</span>}
            {source && rest ? " · " : ""}
            {rest}
          </span>
        )}
      </dd>
    </div>
  );
}
