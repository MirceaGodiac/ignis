// Shared marks and notes. Refusals and errors are typed values shown in full, never swallowed.
import type { ReactNode } from "react";
import type { Claim, ErrorEnvelope, EvidenceClass, Origin } from "../contracts/generated";
import type { Refusal } from "../api";
import { ApiError } from "../research/client";

export const words = (s: string) => s.replace(/_/g, " ");
export const shortId = (id: string, n = 22) => (id.length > n ? `${id.slice(0, n - 6)}…${id.slice(-4)}` : id);
export const fmtBytes = (n: number) => (n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(1)} KiB` : `${(n / 1048576).toFixed(1)} MiB`);

export function Synthetic({ label = "Synthetic" }: { label?: string }) {
  return (
    <span className="tag tag-synthetic" title="Generated for development: not NASA data, not a scientific result">
      {label}
    </span>
  );
}

/** Marks an item synthetic only when the page itself is not already marked. */
export function OriginMark({ origin, pageSynthetic }: { origin: Origin; pageSynthetic: boolean }) {
  return origin === "synthetic_fixture" && !pageSynthetic ? <Synthetic /> : null;
}

const REFUSAL_TITLE: Record<string, string> = {
  unsupported: "Refused: unsupported",
  insufficient_inference: "Refused: insufficient inference",
  unavailable: "Unavailable",
  precondition_failed: "Not allowed yet",
  invalid_request: "Invalid request",
  not_found: "Not found",
  failed: "Failed",
};

export function RefusalNote({ refusal, title, next }: { refusal: Refusal; title?: string; next?: ReactNode }) {
  return (
    <div className="note note-refusal" role="status">
      <b>
        {title ?? REFUSAL_TITLE[refusal.kind] ?? "Refused"} <code>{refusal.code}</code>
      </b>
      <p>{refusal.reason}</p>
      {next && <p>{next}</p>}
    </div>
  );
}

function retryText(e: ErrorEnvelope): string {
  switch (e.retry.kind) {
    case "never":
      return "Do not retry.";
    case "after_delay":
      return `Retry after ${e.retry.seconds} s.`;
    case "after_user_action":
      return "Retry after you change the request.";
    case "reconcile":
      return "The effect is uncertain: check the state before you retry.";
  }
}

export function ErrorNote({ error, title }: { error: unknown; title?: string }) {
  const env = error instanceof ApiError ? error.envelope : null;
  return (
    <div className="note note-refusal" role="status">
      <b>
        {title ?? "Request failed"} {env && <code>{env.code}</code>}
        {error instanceof ApiError && error.httpStatus != null && <span className="muted">HTTP {error.httpStatus}</span>}
      </b>
      <p>{env ? env.message : String(error)}</p>
      {env && <p className="muted">{retryText(env)}</p>}
    </div>
  );
}

export function Caution({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <div className="note note-caution" role="note">
      <b>{title}</b>
      {children && <p>{children}</p>}
    </div>
  );
}

export function Gap({ title, children }: { title: string; children?: ReactNode }) {
  return (
    <div className="note note-gap" role="note">
      <b>{title}</b>
      {children && <p>{children}</p>}
    </div>
  );
}

// ------------------------------------------------------------------ claim kinds and evidence classes

export const CLAIM_LABEL: Record<Claim["kind"], string> = {
  source_reports: "Source reports",
  observed: "Observed",
  computed: "Computed by IGNIS",
  predicted: "Model prediction",
  interpretation: "Interpretation",
};

const CLAIM_GLYPH: Record<Claim["kind"], string> = {
  source_reports: "g-source",
  observed: "g-observed",
  computed: "g-computed",
  predicted: "g-predicted",
  interpretation: "g-interpretation",
};

export function ClaimGlyph({ kind }: { kind: Claim["kind"] }) {
  return <span className={`glyph ${CLAIM_GLYPH[kind]}`} aria-hidden="true" />;
}

const EVIDENCE_LABEL: Record<EvidenceClass, string> = {
  original_observation: "Original observation",
  source_derived: "Derived in the source",
  ignis_derived: "Derived by IGNIS",
  model_output: "Model output",
  synthetic: "Synthetic",
};

export function EvidenceClassTag({ cls }: { cls: EvidenceClass }) {
  if (cls === "synthetic") return <Synthetic />;
  const glyph = cls === "original_observation" ? "g-observed" : cls === "model_output" ? "g-predicted" : cls === "ignis_derived" ? "g-computed" : "g-source";
  return (
    <span className={`tag ${cls === "model_output" ? "tag-flame" : ""}`}>
      <span className={`glyph ${glyph}`} aria-hidden="true" /> {EVIDENCE_LABEL[cls]}
    </span>
  );
}

const QUALIFIED = ["measurement_qualified", "forecast_qualified", "event_qualified"];
export const isQualified = (state: string | null | undefined) => !!state && QUALIFIED.includes(state);

export function ReleaseState({ state }: { state: string | null | undefined }) {
  const s = state ?? "unknown";
  return <span className={`tag ${isQualified(s) ? "tag-ok" : "tag-warn"}`}>{isQualified(s) ? words(s) : `${words(s)} · not qualified`}</span>;
}
