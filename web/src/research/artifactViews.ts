// T7: which view shows each contract, and the pure helpers the typed artifact viewers use.
// `CONTRACT_COVERAGE` is the T7 checklist: every contract in `IgnisContracts` either has a view
// (where it renders) or an explicit reason it is not shown. The `satisfies` clause makes a new
// contract a type error until it is classified here.

import type { BuildCheck, DataExposure, DatasetBundle, EvaluationResult, ExecutionRecord, IgnisContracts, RunnerLimits, UncertaintyKind } from "../contracts/generated";

export type Coverage = { view: string } | { notShown: string };

export const CONTRACT_COVERAGE = {
  // ---- research and evidence: what a researcher meets in an investigation
  ResearchEpisode: { view: "Research page: title, properties, tabs" },
  ResearchEvent: { view: "Thread and Route tabs" },
  ResearchAnswer: { view: "Answer view with margin claim keys" },
  ToolResult: { view: "Side panel: route step" },
  ToolDescriptor: { view: "Sidebar services and capabilities" },
  JobRef: { view: "Side panel: route step and artifact producer" },
  ArtifactRef: { view: "Side panel: artifact" },
  InlinePayload: { view: "Side panel: route step payload" },
  ErrorEnvelope: { view: "Error and refusal notes" },
  EvidenceRef: { view: "Side panel: evidence, with page text and bbox" },
  SearchPage: { view: "Evidence tab" },
  ReadResult: { view: "Side panel: evidence page text" },
  Source: { view: "Sources page" },
  ExecutionRecord: { view: "Side panel: typed artifact (T7)" },
  AnalysisArtifact: { view: "Side panel: typed artifact (T7); Lab chart" },
  DatasetBundle: { view: "Side panel: typed artifact (T7)" },
  EvaluationResult: { view: "Side panel: typed artifact (T7)" },
  ModelRelease: { view: "Side panel: release, construction and checks (T7); Models page" },
  // ---- legacy run routes (T6 moves them to v1)
  ExperimentRecord: { view: "Runs table and side panel run" },
  Observation: { view: "Run chart, Compare, Lab" },
  SealedPrediction: { view: "Lab seal and reveal" },
  AcquisitionAction: { view: "Lab next test ranking" },
  // ---- not shown yet, with the reason
  Investigation: { notShown: "Waits for the PSI catalog (R6) and the graph (R9)" },
  Campaign: { notShown: "Waits for the PSI catalog (R6)" },
  Session: { notShown: "Waits for the PSI catalog (R6)" },
  Specimen: { notShown: "Waits for the conditions service (R9, T9)" },
  ControlHistory: { notShown: "Waits for v1 run routes (M11, T6)" },
  ContourObservation: { notShown: "Metric contours: waits for a calibrated bundle (R5); not drawn in image space" },
  VisualObservationBundle: { view: "Side panel: typed artifact, video with contour (T7)" },
  ImageContourObservation: { view: "Contour layer over the video, image pixels (T7)" },
  Calibration: { notShown: "Shown only through observations that cite it" },
  EndpointRecord: { notShown: "Waits for v1 run routes (M11, T6)" },
  Cohort: { notShown: "Evaluator internal: named by id in results" },
  Split: { notShown: "Evaluator internal: named by id in results" },
  PrefixBundle: { notShown: "Evaluator internal: forecast input" },
  DecisionTrace: { notShown: "Waits for v1 acquisition route (M11)" },
  ProvenanceLink: { notShown: "Waits for the graph producer (R9)" },
  WikiPatch: { notShown: "Waits for the wiki (R4)" },
  WikiRevision: { notShown: "Waits for the wiki (R4)" },
  ModelSpec: { notShown: "Waits for the WSP lane (I3); release view shows the spec hash" },
  HandwrittenWeightPatch: { notShown: "Waits for the WSP lane (I3); release view shows the patch hash" },
  ModelBuildManifest: { view: "Side panel: release construction (T7)" },
  ExecutionScope: { notShown: "Host-derived; never sent by the user" },
  // ---- K1-evidence and K1-dataset (PR 26): contracts landed, producers and views not yet
  SourceLabel: { notShown: "Waits for the classifier (R8); T7 source panel will show labels" },
  LabelSet: { notShown: "Waits for the classifier (R8); T7 source panel will show labels" },
  SourceAccess: { view: "Side panel: evidence original, PDF at the bbox or video at the time (T7)" },
  SourceOutline: { notShown: "Waits for outlines (R7); T7 source panel section tree" },
  ConditionsTable: { view: "Conditions page: habitat relevance and conditions per run (T9)" },
  GraphNeighborsPage: { notShown: "Waits for the graph service (R9); T7 related navigation" },
  WikiReadResult: { notShown: "Waits for the wiki (R4); T7 wiki view" },
  ExternalSourceCandidate: { notShown: "Waits for web discovery (R4); shown with the wiki" },
  WebSearchPage: { notShown: "Waits for web discovery (R4); shown as a route step" },
  // ---- K1-research (PR 31)
  Capabilities: { view: "Sidebar services and tool availability" },
  AnswerRefusal: { notShown: "Waits for answer_submit validation (M7); T8 shows it on the refused step" },
  TraceBundle: { notShown: "Waits for trace export (M10); T11 scorecard" },
  MessageAccepted: { notShown: "Response: shown as the message in the thread" },
  ApprovalAccepted: { notShown: "Response: shown as the approval's new state in the thread" },
  // ---- requests and references: shown inside the object they produce or name
  AnswerSubmission: { notShown: "Request: shown through the ResearchAnswer it becomes" },
  CreateEpisode: { notShown: "Request: shown through the ResearchEpisode it creates" },
  PostMessage: { notShown: "Request: shown as the message in the thread" },
  ResolveApproval: { notShown: "Request: shown as the approval's new state" },
  EpisodeAction: { notShown: "Request: shown as the episode's new status" },
  ConditionsQuery: { notShown: "Request: shown through its ConditionsTable" },
  GraphNeighborsRequest: { notShown: "Request: shown through its GraphNeighborsPage" },
  OutlineRequest: { notShown: "Request: shown through its SourceOutline" },
  WikiReadRequest: { notShown: "Request: shown through its WikiReadResult" },
  WebSearchRequest: { notShown: "Request: shown through its WebSearchPage" },
  WebFetchRequest: { notShown: "Request: shown through the fetched source" },
  ArtifactReadRequest: { notShown: "Request: shown through the artifact it reads" },
  SearchRequest: { notShown: "Request: shown as step arguments once P1 lands" },
  ReadRequest: { notShown: "Request: shown as step arguments once P1 lands" },
  AnalysisExecuteRequest: { notShown: "Request: shown through its ExecutionRecord" },
  ForecastRequest: { notShown: "Request: shown through its AnalysisArtifact" },
  EvaluationRequest: { notShown: "Request: shown inside its EvaluationResult" },
  JobLookup: { notShown: "Request: shown through the job it names" },
  DatasetRef: { notShown: "Reference: shown through its DatasetBundle" },
  ReleaseRef: { notShown: "Reference: shown through its ModelRelease" },
} satisfies Record<keyof IgnisContracts, Coverage>;

/** The contracts that get a typed body in the artifact side panel. */
export const TYPED_ARTIFACTS = ["ExecutionRecord", "DatasetBundle", "EvaluationResult", "AnalysisArtifact", "VisualObservationBundle"] as const;
export type TypedArtifact = (typeof TYPED_ARTIFACTS)[number];
export const isTypedArtifact = (c: string | null | undefined): c is TypedArtifact => !!c && (TYPED_ARTIFACTS as readonly string[]).includes(c);

// ---------------------------------------------------------------- ExecutionRecord

/** One line for how the sandboxed program ended. Exit 0 is only "ran"; it says nothing about the science. */
export function outcomeText(o: ExecutionRecord["outcome"]): { text: string; ok: boolean } {
  switch (o.kind) {
    case "exited":
      return { text: o.exit_code === 0 ? "Exited with code 0" : `Exited with code ${o.exit_code}`, ok: o.exit_code === 0 };
    case "timed_out":
      return { text: "Stopped: wall-time limit", ok: false };
    case "out_of_memory":
      return { text: "Stopped: memory limit", ok: false };
  }
}

export function durationS(start: string, end: string): number | null {
  const d = (Date.parse(end) - Date.parse(start)) / 1000;
  return Number.isFinite(d) && d >= 0 ? d : null;
}

const bytes = (n: number) => (n >= 1 << 30 ? `${+(n / (1 << 30)).toFixed(1)} GiB` : n >= 1 << 20 ? `${+(n / (1 << 20)).toFixed(1)} MiB` : n >= 1024 ? `${+(n / 1024).toFixed(1)} KiB` : `${n} B`);

export function limitRows(l: RunnerLimits): [string, string][] {
  return [
    ["wall time", `${l.wall_seconds} s`],
    ["CPU", `${l.cpu_millis / 1000} cores`],
    ["memory", bytes(l.memory_bytes)],
    ["processes", String(l.pids)],
    ["scratch", bytes(l.tmp_bytes)],
    ["outputs", bytes(l.output_bytes)],
    ["each stream", bytes(l.stream_bytes)],
  ];
}

// ---------------------------------------------------------------- DatasetBundle

/** A column's role and unit in words. A value column without a unit is flagged, never assumed to be SI. */
export function columnUnit(c: DatasetBundle["views"][number]["columns"][number]): { text: string; missing: boolean } {
  if (c.unit) return { text: c.unit, missing: false };
  const needsUnit = c.role === "value" || c.role === "uncertainty" || c.role === "time" || c.role === "available_at";
  return { text: needsUnit ? "no unit" : "—", missing: needsUnit };
}

// ---------------------------------------------------------------- EvaluationResult

/**
 * Coverage and quality side by side. A metric over `scored_cases` describes only the cases that
 * were scored, so it is always shown with how many cases that is out of the total.
 */
export function evaluationSummary(r: EvaluationResult): { coveragePct: string; scoredOf: string; denominatorKept: boolean } {
  const { total, scored, refused, failed } = r.cases;
  return {
    coveragePct: `${Math.round(r.coverage * 1000) / 10} %`,
    scoredOf: `${scored} of ${total} cases scored`,
    // The refused and failed cases must stay in the denominator (the contract's accounting rule).
    denominatorKept: scored + refused + failed === total,
  };
}

export function fmtMetric(v: number | null | undefined, unit: string): string {
  if (v == null || !Number.isFinite(v)) return "not computed";
  const a = Math.abs(v);
  const s = a !== 0 && (a < 1e-3 || a >= 1e5) ? v.toExponential(2) : String(+v.toPrecision(4));
  return unit && unit !== "1" ? `${s} ${unit}` : s;
}

// ---------------------------------------------------------------- ModelRelease (construction and qualification)

/** What a release's uncertainty means. Literal (WSP) releases cannot claim calibrated predictive intervals. */
export const UNCERTAINTY_TEXT = {
  none: "No uncertainty band",
  assumed_noise: "Assumed noise, not calibrated from data",
  scenario_spread: "Scenario spread, not a predictive interval",
  calibrated_prediction: "Calibrated predictive interval",
} satisfies Record<UncertaintyKind, string>;

export function checkSummary(checks: readonly BuildCheck[] | undefined): { passed: number; failed: number; notRun: number; text: string } {
  const c = checks ?? [];
  const passed = c.filter((x) => x.status === "passed").length;
  const failed = c.filter((x) => x.status === "failed").length;
  const notRun = c.length - passed - failed;
  const parts = [`${passed} passed`, failed ? `${failed} failed` : "", notRun ? `${notRun} not run` : ""].filter(Boolean);
  return { passed, failed, notRun, text: c.length ? parts.join(", ") : "no build checks recorded" };
}

/** What the author saw before it wrote the weights. A literal release must not have seen test data. */
export function exposureRows(e: DataExposure): [string, number][] {
  return [
    ["sources", e.sources?.length ?? 0],
    ["datasets", e.datasets?.length ?? 0],
    ["dev feedback", e.development_feedback?.length ?? 0],
    ["prior candidates", e.prior_candidates?.length ?? 0],
  ];
}
