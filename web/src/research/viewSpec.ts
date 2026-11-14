// T8: the closed answer-view catalog (K1.4 `ViewSpec`), checked on the client before anything renders.
// The agent names a component and references; it never sends markup. A view renders only what the
// answer itself cites (its evidence, artifacts and releases); anything else is refused with a reason.

import type { ResearchAnswer, SeriesRef, ViewSpec } from "../contracts/generated";
import { resolvePointer } from "./units.ts";

export type ViewKind = ViewSpec["kind"];

/** The contract each artifact a view reads must carry (mirrors the Rust `ViewSpec` requirement). */
export const VIEW_ARTIFACT_CONTRACT: Partial<Record<ViewKind, string>> = {
  conditions_matrix: "ConditionsTable",
  atmosphere_map: "ConditionsTable",
  video_overlay: "VisualObservationBundle",
};

/** Every reason this view must not render for this answer; empty when it may. */
export function viewProblems(view: ViewSpec, answer: ResearchAnswer): string[] {
  const evidence = new Set((answer.evidence ?? []).map((e) => e.evidence_id));
  const artifacts = new Map((answer.artifacts ?? []).map((a) => [a.artifact_id, a]));
  const releases = new Set(answer.model_releases ?? []);
  const out: string[] = [];
  const needArtifact = (id: string, contract?: string) => {
    const a = artifacts.get(id);
    if (!a) out.push(`artifact ${id} is not cited by the answer`);
    // Fail closed like the Rust check: an artifact that names no contract does not satisfy one.
    else if (contract && a.contract !== contract) out.push(`artifact ${id} has contract ${a.contract ?? "none"}, not ${contract}`);
  };
  switch (view.kind) {
    case "source_excerpt":
      if (!evidence.has(view.evidence_id)) out.push(`evidence ${view.evidence_id} is not cited by the answer`);
      break;
    case "series_comparison":
      if (view.series.length === 0) out.push("no series");
      for (const s of view.series) needArtifact(s.artifact_id, s.role === "predicted" ? "AnalysisArtifact" : undefined);
      for (const s of view.series) {
        const c = artifacts.get(s.artifact_id)?.contract;
        if (s.role !== "predicted" && c === "AnalysisArtifact") out.push(`series from ${s.artifact_id} is model output but is labelled ${s.role}`);
      }
      break;
    case "conditions_matrix":
    case "atmosphere_map":
      needArtifact(view.table, "ConditionsTable");
      break;
    case "video_overlay":
      needArtifact(view.bundle, "VisualObservationBundle");
      break;
    case "release_card":
      if (!releases.has(view.release_id)) out.push(`release ${view.release_id} is not listed by the answer`);
      break;
    case "artifact_table":
      needArtifact(view.artifact_id);
      break;
  }
  return out;
}

export interface Series {
  ref: SeriesRef;
  times: number[];
  values: (number | null)[];
  unit: string;
  timeOrigin: string;
  band: { lower: (number | null)[]; upper: (number | null)[] } | null;
}

const numArray = (v: unknown): (number | null)[] | null => (Array.isArray(v) && v.every((x) => x === null || (typeof x === "number" && Number.isFinite(x))) ? (v as (number | null)[]) : null);

/** One series read from its artifact by the view's pointers, or the reason it cannot be read. */
export function readSeries(doc: unknown, ref: SeriesRef): Series | string {
  const times = numArray(resolvePointer(doc, ref.time_pointer));
  const values = numArray(resolvePointer(doc, ref.value_pointer));
  const unit = resolvePointer(doc, ref.unit_pointer);
  const origin = resolvePointer(doc, ref.time_origin_pointer);
  if (!times || times.some((t) => t === null)) return `${ref.time_pointer} is not an array of times`;
  if (!values) return `${ref.value_pointer} is not an array of numbers`;
  if (values.length !== times.length) return `${values.length} values for ${times.length} times`;
  if (typeof unit !== "string" || !unit) return `${ref.unit_pointer} does not name a unit`;
  if (typeof origin !== "string" || !origin) return `${ref.time_origin_pointer} does not name a time origin`;
  let band: Series["band"] = null;
  if (ref.band) {
    const lower = numArray(resolvePointer(doc, ref.band.lower_pointer));
    const upper = numArray(resolvePointer(doc, ref.band.upper_pointer));
    if (!lower || !upper || lower.length !== times.length || upper.length !== times.length) return "the band does not match the times";
    band = { lower, upper };
  }
  return { ref, times: times as number[], values, unit, timeOrigin: origin, band };
}

/** Series share one chart only on one time origin and one unit; otherwise the view says why it will not draw them. */
export function seriesProblems(series: Series[]): string[] {
  const out: string[] = [];
  const origins = new Set(series.map((s) => s.timeOrigin));
  const units = new Set(series.map((s) => s.unit));
  if (origins.size > 1) out.push(`the series start their clocks at different origins (${[...origins].join(", ")})`);
  if (units.size > 1) out.push(`the series are in different units (${[...units].join(", ")})`);
  return out;
}

/** The fields shown for an `artifact_table`: an array of objects becomes rows, an object becomes key/value rows. */
export function tableOf(v: unknown): { columns: string[]; rows: unknown[][] } | null {
  if (Array.isArray(v) && v.length > 0 && v.every((r) => r && typeof r === "object" && !Array.isArray(r))) {
    const columns = [...new Set(v.flatMap((r) => Object.keys(r as object)))];
    return { columns, rows: v.map((r) => columns.map((c) => (r as Record<string, unknown>)[c])) };
  }
  if (v && typeof v === "object" && !Array.isArray(v)) return { columns: ["field", "value"], rows: Object.entries(v as object) };
  if (Array.isArray(v)) return { columns: ["value"], rows: v.map((x) => [x]) };
  return null;
}
