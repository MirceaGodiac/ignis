// Hand-written models in the forecast lab: the releases the research server serves (/api/v1), each
// with its plain name, what it forecasts, what it reads and how it scored (every served model is
// qualified). A forecast runs the release's strict forward on a stored run the server lists for
// it (train, plus validation and test runs the deployment opened once their final test was spent:
// held out, the model never read them) or on conditions typed here; a typed scenario is a synthetic
// input, never a NASA run.
import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { useQueries, useQuery } from "@tanstack/react-query";
import type { AnalysisArtifact, EndpointRecord, JobRef, ModelCard, ModelInput, ScenarioAdmitted, StoredRunList, ToolResult } from "../contracts/generated";
import { call, type ExperimentDetail, type ExperimentSummary, type ObservationsAtCutoff } from "../api";
import { ForecastChart, type Bands, type ChartPoint } from "../components/ForecastChart";
import { useData } from "../shell/data";
import { builtParagraph, readerLimits, scoreParts, useModelCards, v1, V1Error, writtenBy } from "../shell/models";
import { useReleases } from "../shell/queries";
import { Caution, Gap, Qualified, words } from "../ui/common";
import { EventStack, SupportNote } from "./Lab";
import { familyLabel, hasSeries, runLabel } from "./RunPicker";

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

/**
 * The stored runs a release may forecast here, each with its partition: train, plus the validation
 * and test runs the deployment opened for it (held out: no author of the model read them).
 */
const runsQuery = (base: string | null, releaseId: string, enabled = true) => ({
  queryKey: [base, "hw-runs", releaseId],
  queryFn: () => v1<StoredRunList>(base!, `/models/${encodeURIComponent(releaseId)}/runs`),
  enabled: !!base && enabled,
  retry: false,
  staleTime: 300_000,
});

/** Does the deployment let this release forecast validation or test runs (their final test spent)? */
export const opensHeldOut = (c: ModelCard) => (c.open_partitions ?? []).some((p) => p !== "train");

/**
 * What a served model can do with each run: forecast it from the stored record ("ready"), forecast it
 * from its conditions filled in for the user ("conditions"), forecast it only with values the user
 * types because the record lacks some ("typed"), nothing because the run is sealed for evaluation
 * ("sealed", only where the deployment kept validation and test closed), or nothing because no model
 * is written for its family ("none"). `heldOut` holds the runs a model forecasts that are validation
 * or test runs: the model never read them.
 */
export type Reach = "ready" | "conditions" | "typed" | "sealed" | "none";
export interface Reaches {
  reach: Map<string, Reach>;
  heldOut: Set<string>;
}
export function useReach(runs: ExperimentSummary[]): Reaches | null {
  const { base, cards } = useModelCards();
  const tables = useTables(cards.data);
  const stored = (cards.data ?? []).filter((c) => c.stored_runs);
  const lists = useQueries({ queries: stored.map((c) => runsQuery(base, c.release_id)) });
  if (!base || cards.isError) return null;
  if (!cards.data || lists.some((t) => t.isLoading)) return null;
  const readable = stored.map((c, k) => ({ c, runs: new Map((lists[k].data?.runs ?? []).map((x) => [x.run_id as string, x])), needsSeries: c.inputs.some((i) => i.kind === "observation") }));
  const typed = cards.data.filter((c) => !c.stored_runs);
  // A family is sealed only where a stored-run release keeps validation and test closed.
  const sealedFamilies = new Set(stored.filter((c) => !opensHeldOut(c)).map((c) => c.family));
  const reach = new Map<string, Reach>();
  const heldOut = new Set<string>();
  for (const r of runs) {
    const listed = readable.filter(({ c, runs }) => c.family === r.family && runs.has(r.id));
    const hit = listed.find(({ needsSeries }) => !needsSeries || hasSeries(r));
    if (hit?.runs.get(r.id)?.held_out) heldOut.add(r.id);
    const mine = typed.filter((c) => fit(c, r).score >= 0);
    // "conditions" only when the record fills every required input; otherwise the user types the rest.
    const filled = mine.some((c) => complete(c, r, tables));
    // A listed run without a recorded series can still be forecast from values the user types.
    reach.set(r.id, hit ? "ready" : filled ? "conditions" : mine.length || listed.length ? "typed" : sealedFamilies.has(r.family) ? "sealed" : "none");
  }
  return { reach, heldOut };
}

/**
 * Inputs the WSP lane derived per run, for models whose features the registry does not hold as conditions
 * (fan curve at pot 1, material class, sides burning...). Built by tools/web/<slug>_inputs.py from the
 * lane's case input files only; keyed by the lane's case id.
 */
interface TableRow {
  coverage: string;
  material?: string;
  inputs: Record<string, number>;
}
interface RecordTable {
  runs: Record<string, TableRow>;
}
export type Tables = Map<string, RecordTable>;
const TABLES: Record<string, string> = { "bass5.wsp": "data/bass5_inputs.json" };

export function useTables(cards: ModelCard[] | undefined): Tables {
  const names = [...new Set((cards ?? []).map((c) => c.model_name).filter((n) => TABLES[n]))];
  const qs = useQueries({
    queries: names.map((n) => ({
      queryKey: ["record-table", n],
      queryFn: async () => {
        const res = await fetch(import.meta.env.BASE_URL + TABLES[n]);
        if (!res.ok) throw new Error(`${TABLES[n]}: ${res.status}`);
        return (await res.json()) as RecordTable;
      },
      staleTime: Infinity,
      retry: false,
    })),
  });
  const m: Tables = new Map();
  names.forEach((n, k) => qs[k].data && m.set(n, qs[k].data!));
  return m;
}

/** A run's row in a lane table: the same id, or (PSI-26) the lane's "<investigation>:<date>:<test>". */
function tableRow(t: RecordTable | undefined, r: ExperimentSummary): TableRow | null {
  if (!t) return null;
  if (t.runs[r.id]) return t.runs[r.id];
  const test = (r.aliases ?? []).find((a) => a.scheme === "test_number")?.name;
  const date = r.session_id?.match(/\d{4}-\d{2}-\d{2}/)?.[0];
  if (!test || !date) return null;
  const exact = t.runs[`${r.investigation_id}:${date}:${test}`];
  if (exact) return exact;
  // A mistyped year in the registry (2103 for 2013): the same test on the same month and day.
  const near = Object.keys(t.runs).filter((k) => k.startsWith(`${r.investigation_id}:`) && k.endsWith(`-${date.slice(5)}:${test}`));
  return near.length === 1 ? t.runs[near[0]] : null;
}

/** Whether the record fills every required input of a model for a run. */
export function complete(card: ModelCard, r: ExperimentSummary, tables: Tables): boolean {
  const v = fromRun(card, r, tables);
  return card.inputs.every((i) => !i.required || v[i.name] !== undefined);
}

/** A model's inputs filled from what the NASA record holds for a run; anything it lacks stays empty. */
function fromRun(card: ModelCard, r: ExperimentSummary, tables: Tables): Record<string, string> {
  const v: Record<string, string> = {};
  const num = (x: number) => String(+x.toPrecision(6));
  const row = tableRow(tables.get(card.model_name), r);
  if (row) {
    for (const i of card.inputs) if (row.inputs[i.name] !== undefined) v[i.name] = num(row.inputs[i.name]);
    return v;
  }
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
  const tables = useTables(cards.data);
  const [pickByRun, setPickByRun] = useState<Record<string, string>>({});
  // With no run picked, the panel stays closed until the user asks to type conditions.
  const [typing, setTyping] = useState(false);
  const ranked = useMemo(() => {
    const all = cards.data ?? [];
    if (!run) return all.map((c) => ({ c, score: 0, why: [] as string[] }));
    return all
      .map((c) => {
        const f = fit(c, run);
        // A model the record fills completely beats one that needs values typed in.
        if (f.score >= 0 && !c.stored_runs && complete(c, run, tables)) return { c, score: f.score + 3, why: [...f.why, "every input from the NASA record"] };
        return { c, ...f };
      })
      .sort((x, y) => y.score - x.score);
  }, [cards.data, run, tables]);
  const fitting = ranked.filter((x) => x.score >= 0 && run);
  // With a run, the best fit is chosen for the user; typing conditions with no run, the user picks.
  const auto = run ? fitting[0] ?? null : null;
  const key = run?.id ?? "";
  const manual = pickByRun[key];
  const chosen = ranked.find((x) => x.c.release_id === manual) ?? auto;
  const card = chosen?.c ?? null;
  // The model list is open while nothing is chosen, or when the user asks to change it (per run).
  const [listFor, setListFor] = useState<string | null>(null);
  const listOpen = !chosen || listFor === key;

  if (!base) return null;
  const open = !!run || typing;
  const pick = (id: string) => setPickByRun((m) => ({ ...m, [key]: id }));
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

      {open && !run && ranked.length > 0 && (
        <div className="lab-start">
          <p className="muted">Forecast from conditions you type: pick the model, then enter its inputs.</p>
          <button className="btn btn-s btn-quiet" onClick={() => setTyping(false)}>
            ← Back to picking a run
          </button>
        </div>
      )}

      {open && ranked.length > 0 &&
        (listOpen ? (
          <ModelChooser items={ranked} run={run} best={auto?.c.release_id ?? null} selected={chosen?.c.release_id ?? null} onPick={(id) => (pick(id), setListFor(key))} onDone={() => setListFor(null)} />
        ) : (
          chosen && (
            <div className="mc-line">
              <span className="muted">Model</span>
              <b title={`${chosen.c.model_name} ${chosen.c.candidate}`}>{chosen.c.title}</b>
              <button className="btn btn-s btn-quiet" aria-expanded={false} onClick={() => setListFor(key)}>
                Change
              </button>
            </div>
          )
        ))}
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
        <Gap title={`No served model is written for ${familyLabel(run.family)} runs`}>Choose another run, or pick a model in the list and type its conditions.</Gap>
      )}
      {open && card && (!run || chosen!.score >= 0 || manual) && (
        <ModelPanel key={`${card.release_id}|${run?.id ?? ""}`} card={card} base={base} legacy={origin} run={run && chosen!.score >= 0 ? run : null} pickRun={pickRun} allRuns={runs} tables={tables} />
      )}
    </section>
  );
}

function ModelPanel({ card, base, legacy, run: fixed, pickRun, allRuns, tables }: { card: ModelCard; base: string; legacy: "registry" | "fixtures"; run: ExperimentSummary | null; pickRun: (id: string) => void; allRuns: ExperimentSummary[]; tables: Tables }) {
  // A law release (conditions only, several declared targets): every target that applies to the sample is forecast.
  const law = card.inputs.every((i) => i.kind === "condition") ? lawTargets(card) : [];
  const release = useReleases().data?.find((r) => r.id === card.release_id) ?? null;
  const limits = readerLimits(card);
  const row = fixed ? tableRow(tables.get(card.model_name), fixed) : null;
  const targets = law.filter((t) => !row || t.applies === null || t.applies === row.coverage);
  const obsInput = card.inputs.find((i) => i.kind === "observation");
  const unit = obsInput?.unit_si ?? "";
  const d = display(unit);
  // Without a run the panel only opens to type conditions; with one, a model that reads stored runs reads it.
  const [mode, setMode] = useState<Mode>(card.stored_runs && fixed ? "stored" : "typed");
  // Typed conditions start from what the NASA record holds for the run picked above.
  const recorded = useMemo(() => (fixed ? fromRun(card, fixed, tables) : {}), [card, fixed, tables]);
  const [values, setValues] = useState<Record<string, string>>(recorded);
  const set = (k: string, v: string) => setValues((m) => ({ ...m, [k]: v }));
  // A run picked above fixes the input; the cutoff starts at 40 % of its recorded series.
  const series = fixed?.quantities.find((q) => q.quantity === obsInput?.name);
  const extent = series ? series.archive_t_max_s : null;
  const [cutoff, setCutoff] = useState(() => (extent != null && extent > 0 ? String(+(extent * 0.4).toPrecision(3)) : law.length ? "0" : fixed ? "3" : ""));
  const run = fixed?.id ?? "";
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [art, setArt] = useState<AnalysisArtifact | null>(null);
  const [answers, setAnswers] = useState<{ t: LawTarget; art: AnalysisArtifact }[] | null>(null);
  const [observed, setObserved] = useState<ChartPoint[]>([]);
  const seq = useRef(0);

  // Stored runs offered: the runs the server lists for this release (train, and validation and
  // test runs when the deployment opened them: held out).
  const split = useQuery(runsQuery(base, card.release_id, card.stored_runs && !!fixed));
  const runs = split.data?.runs ?? [];
  const entry = fixed ? runs.find((r) => r.run_id === fixed.id) : undefined;
  const heldOut = !!entry?.held_out;
  const opened = opensHeldOut(card);

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
      const run1 = async (settings?: { target: string }) => {
        const tr = await v1<ToolResult>(base, "/analyses", { schema_version: "wsp.v1", release_id: card.release_id, input, ...(settings ? { settings } : {}) });
        let job = tr.job as JobRef;
        for (let n = 0; !["succeeded", "failed", "cancelled"].includes(job.status); n++) {
          if (n > 600) throw new Error("the forecast job did not finish");
          await new Promise((r) => setTimeout(r, 400));
          job = await v1<JobRef>(base, `/jobs/${encodeURIComponent(job.job_id)}`);
        }
        if (job.status !== "succeeded") throw new V1Error({ code: job.error?.code ?? job.status, message: job.error?.message ?? "the forecast did not run" });
        const out = job.outputs?.[0];
        if (!out) throw new Error("the forecast job stored no artifact");
        return v1<AnalysisArtifact>(base, `/artifacts/${encodeURIComponent(out.artifact_id)}`);
      };
      if (targets.length) {
        const got = await Promise.all(targets.map(async (t) => ({ t, art: await run1({ target: t.name }) })));
        if (!live()) return;
        setArt(null);
        setAnswers(got);
        return;
      }
      const a = await run1();
      if (!live()) return;
      setAnswers(null);
      setArt(a);
      setObserved(points);
    } catch (e) {
      if (!live()) return;
      setArt(null);
      setAnswers(null);
      setError(e instanceof V1Error ? `${words(e.envelope.code)}: ${e.envelope.message}` : String((e as Error).message ?? e));
    } finally {
      if (live()) setBusy(false);
    }
  };

  // With a run fixed above, the forecast runs by itself and follows the cutoff (debounced).
  const inTrain = !!entry;
  const auto = !!fixed && mode === "stored" && inTrain;
  // A law release whose every input the record fills forecasts the picked run by itself, once.
  const lawAuto = !!fixed && targets.length > 0 && mode === "typed" && card.inputs.every((i) => !i.required || recorded[i.name] !== undefined);
  useEffect(() => {
    if (lawAuto) void forecast();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lawAuto]);
  useEffect(() => {
    if (!auto) return;
    const t = setTimeout(() => void forecast(), 350);
    return () => clearTimeout(t);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auto, cutoff]);
  const slide = auto && extent != null && extent > 0;
  // An event model (no observed series) is read against how the run actually ended: the server
  // serves the endpoint only for runs whose outcomes it serves (train, and held-out runs the
  // deployment opened), and the model never receives it.
  const ended = useQuery({
    queryKey: [legacy, "run-endpoint", fixed?.id],
    queryFn: () => call<ExperimentDetail>(legacy, `/experiments/${encodeURIComponent(fixed!.id)}`),
    enabled: auto && !obsInput,
    retry: false,
    staleTime: 300_000,
  });
  const endpoint = auto && !obsInput ? ended.data?.endpoint ?? null : null;
  const trainWithData = useMemo(() => {
    const ids = new Set(runs.map((r) => r.run_id as string));
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
  const inspectionNote = inTrain && !heldOut && card.authorship === "data_informed" ? "The authors read this run while writing the model, so this is inspection, not a test." : null;

  return (
    <div className="fc">
      <div className="fc-model">
        <div className="fc-model-name">
          <span className="fc-model-full">{card.summary}</span>
          <span className="fc-model-meta">
            <Qualified basis={card.qualification.basis} />
            <span className="muted" title={`${card.model_name} ${card.candidate} · ${authorshipLabel(card)}`}>
              {writtenBy(release)}
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
        <summary>About this model</summary>
        <p>{builtParagraph(card, release)}</p>
        {card.scores.length > 0 && (
          <>
            <h4>How it scored</h4>
            <ul>
              {card.scores.map((s) => (
                <li key={s.metric}>
                  <b className="num">{scoreParts(s).value}</b>
                  {scoreParts(s).vs ? ` vs ${scoreParts(s).vs}` : ""} <span className="muted">· {s.metric}</span>
                </li>
              ))}
            </ul>
          </>
        )}
        {limits.length > 0 && (
          <>
            <h4>Where it stops</h4>
            <ul>
              {limits.map((l) => (
                <li key={l}>{l}</li>
              ))}
            </ul>
          </>
        )}
        <details className="fc-tech">
          <summary>Technical details</summary>
          <p>
            <code>
              {card.model_name} {card.candidate}
            </code>{" "}
            · <code title={card.release_id}>{card.release_id.slice(0, 19)}…</code>
          </p>
          <p>{card.forecasts}</p>
          <p className="muted">{card.authorship_note}</p>
          <p className="muted">
            Inputs: {card.inputs_note} Cite as: {card.cite}
            {card.card_doc ? ` Model card: ${card.card_doc}.` : ""}
          </p>
          {card.limits.length > limits.length && (
            <ul>
              {card.limits
                .filter((l) => !limits.includes(l))
                .map((l) => (
                  <li key={l}>{l}</li>
                ))}
            </ul>
          )}
          <ul>
            {[...new Set(card.scores.map((s) => s.source))].map((src) => (
              <li key={src} className="muted">
                Scores from {src}
              </li>
            ))}
          </ul>
        </details>
      </details>

      {mode === "stored" && fixed && split.isLoading && <p className="muted">Checking which runs this model may read…</p>}
      {mode === "stored" && fixed && split.error && <Gap title="The model's runs could not be listed">{String((split.error as Error).message ?? split.error)}</Gap>}
      {mode === "stored" && fixed && split.data && !inTrain && (
        <Caution title={opened ? "This run is not one of the model's runs" : "This run is sealed for evaluation"}>
          {opened ? "It is outside the split the model was written on, so it cannot be fed this one." : "The model reads only its training runs here, so it cannot be fed this one."}{" "}
          {trainWithData.length > 0 ? "Pick one it can read:" : "None of its runs has a recorded series in IGNIS yet."}
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
          {!lawAuto && (
            <Caution title={fixed ? `Conditions from ${runLabel(fixed)}, entered as your own input` : "Your conditions are a synthetic input, not a NASA run"}>
              {fixed ? "Values the NASA record holds are filled in; complete the rest. The forecast is model output for these values, not a reading of the run." : "The forecast is model output for the values you enter."}
            </Caution>
          )}
          {(() => {
            const fields = (
              <InputGroups
                inputs={card.inputs}
                values={values}
                set={set}
                sourceOf={(i) => (fixed ? (recorded[i.name] !== undefined && recorded[i.name] === values[i.name] ? "record" : !(values[i.name] ?? "").trim() && i.required ? "missing" : null) : null)}
              />
            );
            const hasExamples = card.inputs.some((i) => i.example !== undefined && i.example !== null);
            return lawAuto ? (
              <details className="fc-about">
                <summary>
                  The {card.inputs.length} inputs, all from the NASA record of this {row?.coverage === "nose" ? "rod or sphere" : "sample"}
                </summary>
                {fields}
              </details>
            ) : (
              <>
                <div className="fc-typed-bar">
                  <span className="muted">Every field is needed unless it says optional.</span>
                  {hasExamples && (
                    <button className="btn btn-s" onClick={fillExamples}>
                      {fixed ? "Replace with example values" : "Start from an example"}
                    </button>
                  )}
                </div>
                {fields}
              </>
            );
          })()}
        </div>
      )}

      {!slide && !(mode === "stored" && fixed && !inTrain) && (
        <div className="fc-row">
          {!law.length && (
            <>
              <label htmlFor="hw-cutoff">{obsInput ? "The model sees the first" : "Start the forecast at"}</label>
              <input id="hw-cutoff" className="field num fc-num" type="number" min={0} step="any" value={cutoff} onChange={(e) => setCutoff(e.target.value)} onKeyDown={(e) => e.key === "Enter" && void forecast()} />
              <span>s</span>
            </>
          )}
          {auto ? (
            busy && <span className="muted fc-busy">Forecasting…</span>
          ) : (
            <button className={`btn btn-s ${lawAuto ? "btn-quiet" : "btn-primary"}`} disabled={busy} onClick={forecast}>
              {busy ? "Forecasting…" : lawAuto && answers ? "Forecast again with these inputs" : "Forecast"}
            </button>
          )}
        </div>
      )}

      {answers && <LawAnswers answers={answers} card={card} />}
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
            {mode === "stored" && heldOut && (
              <span className="tag tag-flame fc-heldout" title={`A ${entry!.partition} run: the model was written without reading it. ${card.open_reason ?? ""}`.trim()}>
                Held-out run · the model never read it
              </span>
            )}
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

/** A target a law release declares, parsed from its card ("name (kind, unit): description; ..."). */
interface LawTarget {
  name: string;
  kind: string;
  unit: string | null;
  description: string;
  /** "nose" for rod tips and spheres, "flat" for films and slabs, null when the target names no sample kind. */
  applies: string | null;
}
function lawTargets(card: ModelCard): LawTarget[] {
  const out: LawTarget[] = [];
  for (const m of card.forecasts.matchAll(/(\w+) \((binary|positive|event)(?:, ([^)]*))?\): ([^;]+)/g)) {
    const description = m[4].trim().replace(/\.$/, "");
    const applies = /^rod tip or sphere:/.test(description) ? "nose" : /^film or slab:/.test(description) ? "flat" : null;
    out.push({ name: m[1], kind: m[2], unit: m[3] ?? null, description, applies });
  }
  return out;
}

const lawQuestion = (t: LawTarget) => {
  const q = t.description.replace(/^(rod tip or sphere|film or slab):\s*/, "");
  return q[0].toUpperCase() + q.slice(1);
};

/** A law value in the unit a reader expects: speeds in cm/s, mole fractions in percent. */
const lawValue = (v: number, unit: string | null) => (unit === "m/s" ? `${fmtNum(v * 100)} cm/s` : unit === "1" ? `${fmtNum(v * 100)} %` : `${fmtNum(v)}${unit ? ` ${unit}` : ""}`);

function LawAnswers({ answers, card }: { answers: { t: LawTarget; art: AnalysisArtifact }[]; card: ModelCard }) {
  return (
    <section className="sheet fc-main" aria-label="Forecast" aria-live="polite">
      <ul className="fc-law">
        {answers.map(({ t, art }) => {
          let a: ReactNode;
          if (!art.support.admitted) a = <span className="muted">outside the model&apos;s domain{art.support.reasons?.length ? `: ${art.support.reasons.join("; ")}` : ""}</span>;
          else if (t.kind === "binary") {
            const yes = Math.max(0, (art.event_categories ?? []).indexOf("yes"));
            const p = art.event_cdf?.[yes]?.[0] ?? NaN;
            a = (
              <>
                <span className="fc-law-bar" aria-hidden="true">
                  <span style={{ width: `${Math.round(p * 100)}%` }} />
                </span>
                <b className="num">{p >= 0.995 ? ">99" : p < 0.01 ? "<1" : (p * 100).toFixed(0)}%</b> <span className="muted">likely</span>
              </>
            );
          } else {
            const lv = art.quantile_levels ?? [];
            const q = (art.quantiles ?? []).map((r) => r[0]);
            const mi = lv.findIndex((x) => Math.abs(x - 0.5) < 1e-9);
            const med = mi >= 0 ? q[mi] : art.mean?.[0];
            a =
              med == null ? (
                <span className="muted">no value</span>
              ) : (
                <>
                  <b className="num">{lawValue(med, t.unit)}</b>
                  {q.length > 1 && (
                    <span className="muted num">
                      {" "}
                      ({Math.round((lv[lv.length - 1] - lv[0]) * 100)}% range {lawValue(q[0], t.unit)} to {lawValue(q[q.length - 1], t.unit)})
                    </span>
                  )}
                </>
              );
          }
          return (
            <li key={t.name}>
              <span className="fc-law-q">{lawQuestion(t)}</span>
              <span className="fc-law-a">{a}</span>
            </li>
          );
        })}
      </ul>
      <p className="muted fc-foot">Model output, not a measurement. {card.authorship === "data_informed" ? "Its authors read the NASA runs while writing it, so this is inspection, not a test." : ""}</p>
      <details className="fc-tech">
        <summary>Technical details</summary>
        {answers.map(({ t, art }) => (
          <p key={t.name} className="muted">
            <code>{t.name}</code> · artifact <code>{art.id}</code>
          </p>
        ))}
      </details>
    </section>
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

/** Hint parts that only explain the implementation (the forward, the lane, weights, raw formulas): hidden, kept in the tooltip. */
const DEV_HINT = /\b(release|forward|prefix|law|laws|lane|lane-local|weights?|case input|refused|reads)\b|\b[a-z]+_[a-z0-9_]+\b|^optional$/i;
const readableHint = (rest: string) =>
  rest
    .split(/;\s+/)
    .map((s) => s.trim())
    .filter((s) => s && !DEV_HINT.test(s))
    .join("; ");

type Source = "record" | "missing" | null;

/** The typed inputs grouped by what they are: conditions, control stages, measurements before the cutoff, timing. */
const INPUT_GROUPS: { kinds: ModelInput["kind"][]; label: string }[] = [
  { kinds: ["condition"], label: "Conditions" },
  { kinds: ["control"], label: "Controls over time" },
  { kinds: ["observation"], label: "Measured before the cutoff" },
  { kinds: ["phase_start", "test_date"], label: "Timing" },
];

function InputGroups({ inputs, values, set, sourceOf }: { inputs: ModelInput[]; values: Record<string, string>; set: (k: string, v: string) => void; sourceOf: (i: ModelInput) => Source }) {
  return (
    <div className="fc-inputs">
      {INPUT_GROUPS.map((g) => {
        const mine = inputs.filter((i) => g.kinds.includes(i.kind));
        if (!mine.length) return null;
        return (
          <fieldset key={g.label} className="fc-group">
            <legend>{g.label}</legend>
            <div className="fc-grid">
              {mine.map((i) => (
                <InputField key={i.name} input={i} value={values[i.name] ?? ""} onChange={(v) => set(i.name, v)} source={sourceOf(i)} />
              ))}
            </div>
          </fieldset>
        );
      })}
    </div>
  );
}

function InputField({ input, value, onChange, source }: { input: ModelInput; value: string; onChange: (v: string) => void; source: Source }) {
  const multi = input.kind === "observation" || input.kind === "control";
  const id = `hw-in-${input.name}`;
  const { label, rest } = inputLabel(input);
  const unit = !input.text && input.unit_si && input.unit_si !== "1" ? input.unit_si : null;
  const example = input.example !== undefined && input.example !== null && !multi ? fmtExample(input.example) : "";
  // A series or a stage table explains its own format; a scalar keeps only the hint a reader can use.
  const hint = multi ? (input.kind === "control" ? "Leave End empty for a stage that stays on." : "") : readableHint(rest);
  const tip = `${input.name}${input.note ? ` — ${input.note}` : ""}`;
  const cols = input.kind === "control" ? ["Start [s]", "End [s]", `Value${unit ? ` [${unit}]` : ""}`] : ["t [s]", `Value${unit ? ` [${unit}]` : ""}`];
  return (
    <div className={`fc-in${multi ? " fc-in-wide" : ""}${source ? ` fc-in-${source}` : ""}`}>
      <label htmlFor={id} id={`${id}-l`} className="fc-in-label" title={tip}>
        <span>
          {label}
          {unit && !multi && <span className="muted"> [{unit}]</span>}
        </span>
        {!input.required && <span className="fc-opt">optional</span>}
      </label>
      {multi ? (
        <RowsEditor id={id} labelledBy={`${id}-l`} cols={cols} value={value} onChange={onChange} />
      ) : (
        <input id={id} className="field num" value={value} onChange={(e) => onChange(e.target.value)} placeholder={input.kind === "test_date" ? "YYYY-MM-DD" : example ? `e.g. ${example}` : input.text ? "text" : ""} />
      )}
      {(hint || source) && (
        <span className="muted fc-in-note">
          {source === "record" && <span className="fc-in-src">from the NASA record</span>}
          {source === "missing" && <span className="fc-in-src">not in the NASA record: enter it</span>}
          {source && hint ? " · " : ""}
          {hint}
        </span>
      )}
    </div>
  );
}

/**
 * A small table over the same text the forecast parses: one line per row, cells joined by ", "
 * ("t, v" or "start, end, value"), an empty cell left empty. The table always shows at least one row.
 */
function RowsEditor({ id, labelledBy, cols, value, onChange }: { id: string; labelledBy: string; cols: string[]; value: string; onChange: (v: string) => void }) {
  const split = (l: string) => (l.includes(",") ? l.split(",") : l.trim() ? l.trim().split(/[;\t ]+/) : []);
  const grid = (value === "" ? [""] : value.split("\n")).map((l) => {
    const cells = split(l);
    return cols.map((_, k) => (cells[k] ?? "").trim());
  });
  const write = (g: string[][]) => onChange(g.length === 1 && g[0].every((c) => c === "") ? "" : g.map((r) => (r.every((c) => c === "") ? "" : r.join(", "))).join("\n"));
  const setCell = (k: number, j: number, v: string) => write(grid.map((r, a) => (a === k ? r.map((c, b) => (b === j ? v.replace(/[\s,;]/g, "") : c)) : r)));
  const remove = (k: number) => write(grid.length > 1 ? grid.filter((_, a) => a !== k) : [cols.map(() => "")]);
  const add = () => onChange(value === "" ? "\n" : value + "\n");
  // Pasting several lines (or "t, v" pairs) into a cell fills the rows from there.
  const paste = (k: number, text: string) => {
    const lines = text.split(/\r?\n/).filter((l) => l.trim());
    if (lines.length < 2 && !/[,;\t ]/.test(text.trim())) return false;
    const pasted = lines.map((l) => cols.map((_, j) => (split(l)[j] ?? "").trim()));
    write([...grid.slice(0, k), ...pasted, ...grid.slice(k + pasted.length)]);
    return true;
  };
  return (
    <div className="fc-rows" role="group" aria-labelledby={labelledBy}>
      <table>
        <thead>
          <tr>
            {cols.map((c) => (
              <th key={c} scope="col">
                {c}
              </th>
            ))}
            <th className="fc-rows-x">
              <span className="sr-only">Remove</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {grid.map((r, k) => (
            <tr key={k}>
              {r.map((cell, j) => (
                <td key={j}>
                  <input
                    id={k === 0 && j === 0 ? id : undefined}
                    className="field num"
                    inputMode="decimal"
                    aria-label={`${cols[j]}, row ${k + 1}`}
                    value={cell}
                    onChange={(e) => setCell(k, j, e.target.value)}
                    onPaste={(e) => paste(k, e.clipboardData.getData("text")) && e.preventDefault()}
                  />
                </td>
              ))}
              <td className="fc-rows-x">
                <button type="button" className="btn btn-s btn-quiet btn-icon" aria-label={`Remove row ${k + 1}`} title="Remove row" onClick={() => remove(k)}>
                  ×
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <button type="button" className="btn btn-s btn-quiet fc-rows-add" onClick={add}>
        + Add row
      </button>
    </div>
  );
}

/** The served models grouped by what they forecast. */
const MODEL_GROUPS: { label: string; families: string[] }[] = [
  { label: "FLEX droplets", families: ["flex"] },
  { label: "ACME burner flames", families: ["acme_normal", "acme_inverse"] },
  { label: "BASS solids", families: ["bass"] },
  { label: "Other NASA investigations", families: ["other"] },
];

function ModelChooser({
  items,
  run,
  best,
  selected,
  onPick,
  onDone,
}: {
  items: { c: ModelCard; score: number }[];
  run: ExperimentSummary | null;
  best: string | null;
  selected: string | null;
  onPick: (id: string) => void;
  onDone?: () => void;
}) {
  const groups: { label: string; items: ModelCard[] }[] = [];
  const fits = run ? items.filter((x) => x.score >= 0).map((x) => x.c) : [];
  if (fits.length) groups.push({ label: `Written for ${familyLabel(run!.family)} runs`, items: fits });
  const rest = items.filter((x) => !fits.includes(x.c)).map((x) => x.c);
  for (const g of MODEL_GROUPS) {
    const mine = rest.filter((c) => g.families.includes(c.family));
    if (mine.length) groups.push({ label: fits.length ? `${g.label} (you type the conditions)` : g.label, items: mine });
  }
  const known = new Set(MODEL_GROUPS.flatMap((g) => g.families));
  const unknown = rest.filter((c) => !known.has(c.family));
  if (unknown.length) groups.push({ label: "Other models", items: unknown });
  return (
    <div className="mc">
      <div className="mc-head">
        <span id="hw-model-l" className="mc-title">
          {selected ? "Model" : "Choose the model to forecast with"}
        </span>
        {selected && onDone && (
          <button className="btn btn-s btn-quiet" onClick={onDone}>
            Done
          </button>
        )}
      </div>
      <div className="mc-list" role="radiogroup" aria-labelledby="hw-model-l">
        {groups.map((g, gi) => (
          <div key={g.label} className="mc-group" role="group" aria-labelledby={`hw-mg-${gi}`}>
            <h3 id={`hw-mg-${gi}`} className="mc-group-h">
              {g.label}
            </h3>
            {g.items.map((c) => {
              const on = c.release_id === selected;
              return (
                <label key={c.release_id} className={`mc-opt${on ? " on" : ""}`} title={`${c.investigation} · ${c.model_name} ${c.candidate}`}>
                  <input
                    type="radio"
                    name="hw-model"
                    className="sr-only"
                    checked={on}
                    onChange={() => onPick(c.release_id)}
                    // A pointer click picks and closes the list (after the pick lands); arrow keys only move the choice.
                    onClick={(e) => {
                      if (e.detail > 0 && onDone) setTimeout(onDone, 0);
                    }}
                  />
                  <span className="mc-dot" aria-hidden="true" />
                  <span className="mc-main">
                    <b>{c.title}</b>
                    <span className="mc-what">{c.summary}</span>
                  </span>
                  <span className="mc-side">
                    {c.release_id === best && <span className="tag">best fit</span>}
                  </span>
                </label>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}
