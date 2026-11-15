// Hand-written models in the forecast lab: the releases the research server serves (/api/v1), each
// with what it is (hand-written, experimental, zero-shot or data-informed), what it reads and what
// its authors reported. A forecast runs the release's strict forward on a stored run of the train
// partition or on conditions typed here; a typed scenario is a synthetic input, never a NASA run.
import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { AnalysisArtifact, JobRef, ModelCard, ModelInput, ScenarioAdmitted, ToolResult } from "../contracts/generated";
import { call, type ExperimentList, type ObservationsAtCutoff } from "../api";
import { ForecastChart, type Bands, type ChartPoint } from "../components/ForecastChart";
import { useData } from "../shell/data";
import { Caution, Gap, words } from "../ui/common";
import { EventStack, Step, SupportNote } from "./Lab";

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

export function LabModels() {
  const { researchMode, origin } = useData();
  const base = researchMode === "live" ? "/api/v1" : researchMode === "fixtures" ? "/api/fixtures/v1" : null;
  const cards = useQuery({ queryKey: [researchMode, "model-cards"], queryFn: () => v1<ModelCard[]>(base!, "/model-cards"), enabled: !!base, retry: false });
  const [pick, setPick] = useState("");
  const card = cards.data?.find((c) => c.release_id === pick) ?? cards.data?.[0] ?? null;

  if (!base) return null;
  return (
    <section className="sheet" aria-labelledby="hw-h" style={{ marginTop: 16 }}>
      <div className="sheet-head">
        <h2 id="hw-h">Hand-written models</h2>
        {cards.data && cards.data.length > 0 && (
          <>
            <label className="sr-only" htmlFor="hw-model">
              Hand-written model
            </label>
            <select id="hw-model" className="field" value={card?.release_id ?? ""} onChange={(e) => setPick(e.target.value)}>
              {cards.data.map((c) => (
                <option key={c.release_id} value={c.release_id}>
                  {c.model_name} {c.candidate} · {c.investigation}
                </option>
              ))}
            </select>
          </>
        )}
      </div>
      {cards.error && (
        <Gap title="Hand-written models are not served here">{cards.error instanceof V1Error ? cards.error.envelope.message : String(cards.error)}</Gap>
      )}
      {cards.isLoading && <p className="muted">Loading the served models…</p>}
      {card && <ModelPanel key={card.release_id} card={card} base={base} legacy={origin} />}
    </section>
  );
}

function ModelPanel({ card, base, legacy }: { card: ModelCard; base: string; legacy: "registry" | "fixtures" }) {
  const obsInput = card.inputs.find((i) => i.kind === "observation");
  const unit = obsInput?.unit_si ?? "";
  const d = display(unit);
  const [mode, setMode] = useState<Mode>(card.stored_runs ? "stored" : "typed");
  const [values, setValues] = useState<Record<string, string>>({});
  const set = (k: string, v: string) => setValues((m) => ({ ...m, [k]: v }));
  const [cutoff, setCutoff] = useState("");
  const [run, setRun] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [art, setArt] = useState<AnalysisArtifact | null>(null);
  const [observed, setObserved] = useState<ChartPoint[]>([]);

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
    setBusy(true);
    setError(null);
    setArt(null);
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
      setArt(await v1<AnalysisArtifact>(base, `/artifacts/${encodeURIComponent(out.artifact_id)}`));
      setObserved(points);
    } catch (e) {
      setError(e instanceof V1Error ? `${words(e.envelope.code)}: ${e.envelope.message}` : String((e as Error).message ?? e));
    } finally {
      setBusy(false);
    }
  };

  const bands: Bands | null = useMemo(
    () => (art && art.support.admitted && (art.quantiles?.length ?? 0) > 0 ? { times: art.times_s ?? [], levels: art.quantile_levels ?? [], q: (art.quantiles ?? []).map((r) => r.map((v) => v * d.k)) } : null),
    [art, d.k],
  );
  const c = Number(cutoff) || 0;
  const xMax = Math.max(c, ...(bands?.times ?? [0]), ...observed.map((p) => p.t), 1e-6);

  return (
    <div className="lab">
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
          <div className="cutoff-row">
            <label htmlFor="hw-cutoff">Cutoff, s</label>
            <input id="hw-cutoff" className="field num" type="number" min={0} step="any" value={cutoff} onChange={(e) => setCutoff(e.target.value)} />
            <button className="btn btn-s btn-primary" disabled={busy} onClick={forecast}>
              {busy ? "Running the forward…" : "Forecast"}
            </button>
          </div>
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
      {(bands || observed.length > 0) && (
        <section className="sheet" aria-label="Forecast chart">
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
        </section>
      )}
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
