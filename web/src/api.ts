// Client for ignis-server. Contract types come from schemas/*.json (src/contracts/generated.ts);
// the envelopes below are the server's own response shapes (crates/ignis-server/src/api.rs).
import type {
  AcquisitionAction,
  AnalysisArtifact,
  ContourObservation,
  DecisionTrace,
  EndpointRecord,
  ExperimentRecord,
  Family,
  Observation,
  Quantity,
  SealedPrediction,
} from "./contracts/generated";

export type Origin = "registry" | "fixtures";

export const base = (o: Origin) => (o === "fixtures" ? "/api/fixtures" : "/api");

export type RefusalKind =
  | "unsupported"
  | "insufficient_inference"
  | "invalid_request"
  | "not_found"
  | "precondition_failed"
  | "unavailable"
  | "failed";

export interface Refusal {
  kind: RefusalKind;
  code: string;
  reason: string;
}

/** A typed refusal is a value, not an exception the UI swallows. */
export class RefusalError extends Error {
  constructor(public refusal: Refusal, public status: number) {
    super(`${refusal.kind}: ${refusal.reason}`);
  }
}

export async function call<T>(origin: Origin, path: string, init?: RequestInit & { json?: unknown }): Promise<T> {
  const { json, ...rest } = init ?? {};
  const res = await fetch(base(origin) + path, {
    ...rest,
    method: json !== undefined ? "POST" : rest.method ?? "GET",
    headers: json !== undefined ? { "content-type": "application/json" } : rest.headers,
    body: json !== undefined ? JSON.stringify(json) : rest.body,
  });
  let body: unknown;
  try {
    body = await res.json();
  } catch {
    throw new RefusalError({ kind: "unavailable", code: "no_json", reason: `server answered ${res.status} without JSON (is ignis-server running?)` }, res.status);
  }
  const b = body as { outcome?: string; refusal?: Refusal };
  if (b?.outcome === "refusal" && b.refusal) throw new RefusalError(b.refusal, res.status);
  if (!res.ok && res.status !== 202) throw new RefusalError({ kind: "failed", code: `http_${res.status}`, reason: JSON.stringify(body) }, res.status);
  return body as T;
}

export interface DataOriginInfo {
  kind: "registry" | "development_fixture";
  label: string;
}

export interface Health {
  data_origin: DataOriginInfo;
  schema_version: string;
  registry: { dir: string; experiments: number; error_count: number; errors: string[] };
  evidence_catalog: { available: boolean; detail: string; path: string } | null;
  experts: { model_name: string; family: Family; is_baseline: boolean }[];
  execution: { mode: "in_process"; compute_threads: number; acquisition_scorer: boolean };
}

export interface ConditionCell {
  value_si: number | null;
  unit_si: string | null;
  resolution: string;
  evidence_class: string;
}

export interface QuantitySummary {
  quantity: Quantity;
  unit_si: string;
  count: number;
  archive_t_min_s: number;
  archive_t_max_s: number;
}

export interface ExperimentSummary {
  id: string;
  investigation_id: string;
  family: Family;
  fuel_or_material: string | null;
  configuration: string | null;
  session_id: string | null;
  gravity: { value_si: number | null; unit_si: string | null } | null;
  conditions: Record<string, ConditionCell>;
  quantities: QuantitySummary[];
  n_observations: number;
  n_contours: number;
  has_endpoint: boolean;
  identity_resolution: string;
  quality_flags: string[];
  synthetic: boolean;
  gaps: string[];
  /** NASA's own names for the run (test sheet, table test, as-run number); absent from older servers. */
  aliases?: { scheme: string; name: string }[];
}

export interface ExperimentList {
  data_origin: DataOriginInfo;
  count: number;
  experiments: ExperimentSummary[];
  registry_errors: number;
}

export interface ReleaseSummary {
  id: string;
  family: Family;
  model_name: string;
  is_baseline: boolean;
  state: string;
  trained_on: string;
  created_at: string;
  cohort_id: string;
  split_id: string;
  code_version: string;
  feature_order: string[];
}

export interface ExperimentDetail {
  data_origin: DataOriginInfo;
  record: ExperimentRecord;
  summary: ExperimentSummary;
  control_history: { time_origin: string } | null;
  /** How the run ended: served only for runs whose outcomes the scope reads (train); absent from older servers. */
  endpoint?: EndpointRecord | null;
  releases: ReleaseSummary[];
}

export interface ObservationsAtCutoff {
  run_id: string;
  cutoff_s: number;
  admitted: Observation[];
  contours: ContourObservation[];
  pending: { id: string; time_s: number; quantity: Quantity; available_at_s: number }[];
  hidden_after_cutoff: number;
}

export interface AnalysisContext {
  analysis_id: string;
  run_id: string;
  cutoff_s: number;
  model_release_id: string;
  model_name: string;
  release_state: string;
  trained_on: string;
  release_created_at: string;
  release_code_version: string;
  cohort_id: string;
  split_id: string;
  source_manifest_hash: string;
  prefix_hash: string;
  time_origin: string;
  admitted_observation_ids: string[];
  admitted_contour_ids: string[];
  known_condition_keys: string[];
  settings: Record<string, unknown>;
  settings_key: string;
  cache_key: string;
  computed_at_unix_s: number;
  data_origin: string;
  /** What produced the artifact (in-process expert). */
  engine: string;
}

export type ForecastOutcome =
  | { outcome: "artifact"; cached: boolean; refused_by_expert: boolean; artifact: AnalysisArtifact; context: AnalysisContext }
  | { outcome: "job"; job_id: string; state: "running" };

export interface Hit {
  chunk_id: unknown;
  /** publication_id (catalog v1) or source_id */
  source_id: unknown;
  title: string | null;
  canonical_url: string | null;
  section: string | null;
  locator: Record<string, unknown> | null;
  evidence_class: string | null;
  page: number | null;
  char_start: number | null;
  char_end: number | null;
  snippet: string;
  outcome_bearing: unknown;
  describes_runs: unknown;
  bm25: number;
}

export interface SearchResult {
  query: string;
  ranking: string;
  hits: { hit: Hit; source: { title?: string | null; canonical_url?: string | null } | null }[];
  highlight: { open: string; close: string };
}

export interface CompareRow {
  key: string;
  values: (ConditionCell | null)[];
  comparable: boolean;
  reason: string | null;
}

export interface CompareResult {
  run_ids: string[];
  quantity: Quantity | null;
  comparison_kind: string;
  columns: { id: string; family: Family; investigation_id: string; fuel_or_material: string | null; configuration: string | null; gravity: number | null; synthetic: boolean }[];
  rows: CompareRow[];
  numeric_comparison: { allowed: boolean; refusals: Refusal[]; note: string };
}

export interface AcquisitionResult {
  analysis_id: string;
  candidates: AcquisitionAction[];
  cost_note: string;
  scoring: { status: "scored"; trace: DecisionTrace } | { status: "refused"; refusal: Refusal } | { status: "no_candidates"; reason: string };
}

export interface SealResult {
  sealed: SealedPrediction;
  already_sealed: boolean;
}

export interface RevealResult {
  analysis_id: string;
  sealed: SealedPrediction;
  cutoff_s: number;
  continuation: Observation[];
  contours: ContourObservation[];
  endpoint: EndpointRecord | null;
}
