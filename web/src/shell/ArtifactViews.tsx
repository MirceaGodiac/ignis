// T7: typed bodies for the artifact side panel. Each reads one contract and keeps its scientific
// rules visible: exit code is not a result, units are never assumed, coverage sits beside quality,
// and model output is never presented as a measurement. Raw JSON stays one click away in the panel.
import { Fragment } from "react";
import type { AnalysisArtifact, AnswerRefusal, ArtifactRef, DatasetBundle, EvaluationResult, ExecutionRecord, ModelRelease, VisualObservationBundle } from "../contracts/generated";
import { ContourVideo } from "./ContourVideo";
import { checkSummary, columnUnit, durationS, evaluationSummary, exposureRows, fmtMetric, limitRows, outcomeText, UNCERTAINTY_TEXT, type TypedArtifact } from "../research/artifactViews";
import { Caution, Gap, OutputState, VerdictTag, fmtBytes, shortId, words } from "../ui/common";
import { modelTitle, useServedCard } from "./models";
import { usePanel } from "./inspector";
import { useReleases } from "./queries";

export function TypedArtifactBody({ contract, data }: { contract: TypedArtifact; data: unknown }) {
  switch (contract) {
    case "ExecutionRecord":
      return <ExecutionRecordView r={data as ExecutionRecord} />;
    case "DatasetBundle":
      return <DatasetBundleView b={data as DatasetBundle} />;
    case "EvaluationResult":
      return <EvaluationResultView r={data as EvaluationResult} />;
    case "AnalysisArtifact":
      return <AnalysisArtifactView a={data as AnalysisArtifact} />;
    case "VisualObservationBundle":
      return <VisualBundleView b={data as VisualObservationBundle} />;
    case "AnswerRefusal":
      return <RefusalProblems refusal={data as AnswerRefusal} />;
  }
}

function ArtifactLink({ artifact, label }: { artifact: ArtifactRef; label: string }) {
  const panel = usePanel();
  return (
    <button className="linkish" onClick={() => panel.push({ kind: "artifact", artifact })} title={artifact.artifact_id}>
      {label}
    </button>
  );
}

/** Opens the release when the model registry lists it; otherwise shows the id only. */
function ReleaseLink({ id }: { id: string }) {
  const panel = usePanel();
  const release = useReleases().data?.find((r) => r.id === id);
  const card = useServedCard(id);
  if (!release) return <code title={id}>{shortId(id, 22)}</code>;
  return (
    <button className="linkish" onClick={() => panel.push({ kind: "release", release })} title={`${release.model_name} · ${id}`}>
      {modelTitle(release.model_name, card)}
    </button>
  );
}

// ---------------------------------------------------------------- ExecutionRecord

function ExecutionRecordView({ r }: { r: ExecutionRecord }) {
  const out = outcomeText(r.outcome);
  const dur = durationS(r.started_at, r.finished_at);
  const rejected = r.rejected_outputs ?? [];
  return (
    <>
      <div className="panel-sec">
        <h3>Run</h3>
        <div className="toolbar">
          <span className={`tag ${out.ok ? "" : "tag-bad"}`}>{out.text}</span>
          {dur != null && <span className="muted small num">{dur.toFixed(1)} s</span>}
        </div>
        <dl className="kv">
          <dt>program</dt>
          <dd>
            <ArtifactLink artifact={r.program} label={`${words(r.language)} · ${fmtBytes(r.program.byte_length)}`} />
          </dd>
          <dt>isolation</dt>
          <dd>
            {r.environment.isolation} · {r.environment.oci_runtime} · {r.environment.interpreter}
          </dd>
          <dt>image</dt>
          <dd>
            <code>{shortId(r.environment.image_id, 26)}</code>
          </dd>
        </dl>
      </div>
      <div className="panel-sec">
        <h3>Inputs</h3>
        {(r.inputs ?? []).length === 0 ? (
          <p className="muted small">No mounted inputs.</p>
        ) : (
          (r.inputs ?? []).map((i) => (
            <div key={i.name} className="stat-line">
              <code>{i.name}</code>
              <ArtifactLink artifact={i.artifact} label={fmtBytes(i.artifact.byte_length)} />
            </div>
          ))
        )}
      </div>
      <div className="panel-sec">
        <h3>Outputs</h3>
        {(r.outputs ?? []).length === 0 && <p className="muted small">No kept outputs.</p>}
        {(r.outputs ?? []).map((o) => (
          <div key={o.path} className="stat-line">
            <code>{o.path}</code>
            <ArtifactLink artifact={o.artifact} label={`${o.artifact.media_type} · ${fmtBytes(o.artifact.byte_length)}`} />
          </div>
        ))}
        {rejected.length > 0 && (
          <Caution title={`${rejected.length} output${rejected.length > 1 ? "s" : ""} not kept`}>
            {rejected.map((x) => (
              <span key={x.path} style={{ display: "block" }}>
                <code>{x.path}</code>: {x.reason}
              </span>
            ))}
          </Caution>
        )}
      </div>
      <div className="panel-sec">
        <h3>Streams</h3>
        {(["stdout", "stderr"] as const).map((k) => {
          const s = r[k];
          return (
            <div key={k} className="stat-line">
              <span className="muted">{k}</span>
              {s.artifact ? <ArtifactLink artifact={s.artifact} label={fmtBytes(s.total_bytes)} /> : <span className="muted">{s.total_bytes === 0 ? "empty" : fmtBytes(s.total_bytes)}</span>}
              {s.truncated && <span className="tag tag-warn">truncated</span>}
            </div>
          );
        })}
      </div>
      <details>
        <summary className="muted small" style={{ cursor: "pointer" }}>
          Limits
        </summary>
        <dl className="kv" style={{ marginTop: 6 }}>
          {limitRows(r.limits).map(([k, v]) => (
            <Row key={k} k={k} v={v} />
          ))}
        </dl>
      </details>
    </>
  );
}

const Row = ({ k, v }: { k: string; v: string }) => (
  <>
    <dt>{k}</dt>
    <dd className="num">{v}</dd>
  </>
);

// ---------------------------------------------------------------- DatasetBundle

function DatasetBundleView({ b }: { b: DatasetBundle }) {
  return (
    <>
      <dl className="kv">
        <dt>family</dt>
        <dd>{b.family.toUpperCase()}</dd>
        <dt>audience</dt>
        <dd>{b.audience}</dd>
        <dt>partitions</dt>
        <dd>{b.partitions.join(", ")}</dd>
        <dt>time origin</dt>
        <dd>
          <code>{b.time_origin}</code>
        </dd>
        <dt>split</dt>
        <dd>
          <code>{b.split_id}</code>
        </dd>
        {Object.entries(b.inclusion_criteria ?? {}).map(([k, v]) => (
          <Row key={k} k={words(k)} v={v} />
        ))}
      </dl>
      {b.views.map((v) => (
        <div key={v.name} className="panel-sec">
          <div className="toolbar">
            <h3 style={{ margin: 0, fontSize: 12.5, fontWeight: 550 }}>{v.name}</h3>
            <span className="muted small num">{v.row_count.toLocaleString()} rows</span>
            <span style={{ marginLeft: "auto" }}>
              <ArtifactLink artifact={v.artifact} label={fmtBytes(v.artifact.byte_length)} />
            </span>
          </div>
          <div className="tbl-wrap">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Column</th>
                  <th>Role</th>
                  <th>Unit</th>
                </tr>
              </thead>
              <tbody>
                {v.columns.map((c) => {
                  const u = columnUnit(c);
                  return (
                    <tr key={c.name}>
                      <td>
                        <code>{c.name}</code>
                        {c.nullable && <span className="dim"> · nullable</span>}
                      </td>
                      <td className="dim">{words(c.role)}</td>
                      <td>{u.missing ? <span className="tag tag-warn">{u.text}</span> : u.text}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      ))}
      {b.quality_report ? (
        <div className="stat-line">
          <span className="muted">quality report</span>
          <ArtifactLink artifact={b.quality_report} label={fmtBytes(b.quality_report.byte_length)} />
        </div>
      ) : (
        <p className="muted small">No quality report is attached to this bundle.</p>
      )}
    </>
  );
}

// ---------------------------------------------------------------- EvaluationResult

function EvaluationResultView({ r }: { r: EvaluationResult }) {
  const s = evaluationSummary(r);
  return (
    <>
      <div className="panel-sec">
        <div className="toolbar">
          <VerdictTag verdict={r.verdict} />
          <span className="muted small">{words(r.request.purpose)}</span>
        </div>
        <dl className="kv">
          <dt>coverage</dt>
          <dd className="num">
            {s.coveragePct} · {s.scoredOf}
          </dd>
          <dt>refused / failed</dt>
          <dd className="num">
            {r.cases.refused} / {r.cases.failed}
          </dd>
          {r.request.target && (
            <>
              <dt>target</dt>
              <dd>
                {words(r.request.target)} ({r.request.unit})
              </dd>
            </>
          )}
          <dt>partition</dt>
          <dd>{r.request.partition}</dd>
          {(r.request.cutoffs_s ?? []).length > 0 && (
            <>
              <dt>cutoffs</dt>
              <dd className="num">{r.request.cutoffs_s!.map((c) => `${c} s`).join(", ")}</dd>
            </>
          )}
          {r.request.candidate && (
            <>
              <dt>candidate</dt>
              <dd>
                <ReleaseLink id={r.request.candidate.release_id} />
              </dd>
            </>
          )}
          {r.request.harness && (
            <>
              <dt>arms</dt>
              <dd>
                {r.request.harness.variants.map((v) => v.name).join(", ")} · baseline {r.request.harness.baseline}
              </dd>
              <dt>gold set</dt>
              <dd>
                <code>{r.request.harness.gold_set.set_id}</code> · {r.request.harness.gold_set.split} · {r.request.harness.gold_set.task_count} tasks × {r.request.harness.repetitions} repetitions
              </dd>
            </>
          )}
        </dl>
        {r.harness && <Gap title="Harness comparison">The per-arm scores and paired deltas are in the payload; their view is part of T11.</Gap>}
        {!s.denominatorKept && <Caution title="Case accounting does not add up">Scored, refused and failed cases do not sum to the total.</Caution>}
      </div>
      {!r.harness && (
        <div className="panel-sec">
          <h3>Metrics</h3>
          <div className="tbl-wrap">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Metric</th>
                  <th className="r">Value</th>
                  <th className="r">n</th>
                  <th>Over</th>
                </tr>
              </thead>
              <tbody>
                {r.metrics.map((m, i) => (
                  <tr key={`${m.metric}${i}`}>
                    <td>
                      {m.metric.toUpperCase()} <span className="dim">v{m.version}</span>
                    </td>
                    <td className="r num">{fmtMetric(m.value, m.unit)}</td>
                    <td className="r num">{m.n}</td>
                    <td className="dim">{words(m.population)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="muted small">A metric over scored cases describes only those cases. Read it with the coverage above.</p>
        </div>
      )}
      {(r.failures ?? []).length > 0 && (
        <div className="panel-sec">
          <h3>Cases not scored</h3>
          {r.failures!.map((f) => (
            <div key={f.case_id} className="small">
              <span className={`tag ${f.kind === "failed" ? "tag-bad" : "tag-warn"}`}>{f.kind}</span> <code>{f.case_id}</code>
              <div className="muted">{f.reason}</div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}

// ---------------------------------------------------------------- AnalysisArtifact

function AnalysisArtifactView({ a }: { a: AnalysisArtifact }) {
  const panel = usePanel();
  const n = a.times_s?.length ?? 0;
  return (
    <>
      <Caution title="Model output, not a measurement">This artifact is a computed prediction. Compare it with observations; do not cite it as one.</Caution>
      <dl className="kv">
        <dt>kind</dt>
        <dd>{words(a.kind)}</dd>
        <dt>status</dt>
        <dd>
          <OutputState status={a.status} />
        </dd>
        <dt>support</dt>
        <dd>
          {a.support.admitted ? "admitted" : <span className="tag tag-warn">not admitted</span>} · gate {words(a.support.gate)}
          {(a.support.reasons ?? []).length > 0 && <div className="muted small">{a.support.reasons!.join("; ")}</div>}
        </dd>
        {a.quantity && <Row k="quantity" v={`${words(a.quantity)}${a.unit ? ` (${a.unit})` : ""}`} />}
        <dt>time origin</dt>
        <dd>{a.time_origin ? <code>{a.time_origin}</code> : <span className="tag tag-warn">not stated</span>}</dd>
        {n > 0 && <Row k="points" v={`${n}, ${a.times_s![0]} s to ${a.times_s![n - 1]} s`} />}
        {(a.quantile_levels ?? []).length > 0 && <Row k="bands" v={a.quantile_levels!.map((q) => `q${q}`).join(", ")} />}
        {a.run_id && (
          <>
            <dt>run</dt>
            <dd>
              <button className="linkish" onClick={() => panel.push({ kind: "run", id: a.run_id! })}>
                {a.run_id}
              </button>
            </dd>
          </>
        )}
        <dt>release</dt>
        <dd>
          <ReleaseLink id={a.model_release_id} />
        </dd>
      </dl>
    </>
  );
}

// ---------------------------------------------------------------- ModelRelease construction

/** How a release was built and checked (the technical details of a model). */
export function ReleaseConstruction({ release }: { release: ModelRelease }) {
  const m = release.construction;
  const tensors = release.tensors ?? [];
  const trainable = tensors.filter((t) => t.trainable).length;
  if (!m) {
    return (
      <p className="muted small">
        No build manifest: {tensors.length} tensors, {trainable} trainable. Construction is not recorded for this release.
      </p>
    );
  }
  const checks = checkSummary(m.checks);
  return (
    <>
      <div className="panel-sec">
        <h3>Construction</h3>
        <dl className="kv">
          <dt>method</dt>
          <dd>{words(m.construction_method)}</dd>
          <dt>uncertainty</dt>
          <dd>{UNCERTAINTY_TEXT[m.uncertainty_kind]}</dd>
          {m.author && <Row k="author" v={`${m.author.provider} · ${m.author.model_id}${m.author.effort ? ` · ${m.author.effort}` : ""}`} />}
          <dt>forward</dt>
          <dd>
            <code>{m.forward.entry_point}</code> · {words(m.forward.profile)}
          </dd>
          <Row k="tensors" v={`${tensors.length}, ${trainable} trainable`} />
          {release.parent_id && <Row k="parent" v={shortId(release.parent_id, 22)} />}
        </dl>
      </div>
      <div className="panel-sec">
        <h3>Build checks</h3>
        <p className="small">{checks.text}</p>
        {(m.checks ?? []).map((c) => (
          <div key={c.check} className="stat-line">
            <span className={`tag ${c.status === "passed" ? "tag-ok" : c.status === "failed" ? "tag-bad" : "tag-warn"}`}>{words(c.status)}</span>
            <span>{words(c.check)}</span>
            {c.detail && <span className="muted small">{c.detail}</span>}
          </div>
        ))}
        <p className="muted small">The checks confirm the release is well formed.</p>
      </div>
      <div className="panel-sec">
        <h3>What the author saw</h3>
        <dl className="kv">
          {exposureRows(m.exposure).map(([k, n]) => (
            <Row key={k} k={k} v={String(n)} />
          ))}
        </dl>
      </div>
      <details>
        <summary className="muted small" style={{ cursor: "pointer" }}>
          Hashes
        </summary>
        <dl className="kv" style={{ marginTop: 6 }}>
          {(
            [
              ["spec", m.spec_hash],
              ["patch", m.patch_hash],
              ["weights", m.weights_hash],
              ["forward", m.forward_hash],
            ] as const
          )
            .filter(([, v]) => !!v)
            .map(([k, v]) => (
              <Fragment key={k}>
                <dt>{k}</dt>
                <dd>
                  <code title={v!}>{shortId(v!, 26)}</code>
                </dd>
              </Fragment>
            ))}
        </dl>
      </details>
    </>
  );
}

// ---------------------------------------------------------------- VisualObservationBundle

function VisualBundleView({ b }: { b: VisualObservationBundle }) {
  return (
    <>
      <dl className="kv">
        <dt>units</dt>
        <dd>{b.units_status === "metric" ? "metric" : <span className="tag tag-warn">{words(b.units_status)}: not metres</span>}</dd>
        <dt>camera</dt>
        <dd>
          <code>{b.camera_id}</code>
        </dd>
        <dt>time origin</dt>
        <dd>
          <code>{b.time_origin}</code>
        </dd>
        <dt>time basis</dt>
        <dd>{words(b.timestamp_basis)}</dd>
        <dt>run</dt>
        <dd>{b.run.run_id ? <code>{b.run.run_id}</code> : <span className="tag tag-line">identity {words(b.run.resolution)}</span>}</dd>
        <dt>method</dt>
        <dd>
          <code>{b.method}</code>
        </dd>
        <dt>contours</dt>
        <dd>
          <ArtifactLink artifact={b.contours} label={fmtBytes(b.contours.byte_length)} />
        </dd>
      </dl>
      <ContourVideo bundle={b} />
    </>
  );
}

// ---------------------------------------------------------------- AnswerRefusal

/** Every problem the host found in a refused `answer_submit` (K1.3). */
export function RefusalProblems({ refusal }: { refusal: AnswerRefusal }) {
  return (
    <ul className="small" style={{ margin: 0, paddingLeft: 18 }}>
      {refusal.problems.map((p, i) => (
        <li key={i}>
          <b>{words(p.kind)}</b>
          {p.block != null && <span className="dim"> · block {p.block + 1}</span>}
          {p.claim_id && (
            <span className="dim">
              {" "}
              · claim <code>{p.claim_id}</code>
            </span>
          )}
          <div className="muted">{p.detail}</div>
        </li>
      ))}
    </ul>
  );
}
