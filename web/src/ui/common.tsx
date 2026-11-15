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

/** A failed request. `quiet` (inside an answer, where the failure is one view's, not the page's) reads
 * as an outlined gap rather than a red refusal. */
export function ErrorNote({ error, title, quiet = false }: { error: unknown; title?: string; quiet?: boolean }) {
  const env = error instanceof ApiError ? error.envelope : null;
  return (
    <div className={`note ${quiet ? "note-gap small" : "note-refusal"}`} role="status">
      <b>
        {title ?? "Request failed"} {env && <code>{env.code}</code>}
        {error instanceof ApiError && error.httpStatus != null && <span className="muted">HTTP {error.httpStatus}</span>}
      </b>
      <p>{env ? env.message : String(error)}</p>
      {staleSignIn(error) ? (
        <p>
          Your sign-in is no longer accepted by the server (it may predate a key rotation).{" "}
          <a className="btn btn-s" href="/cdn-cgi/access/logout">
            Sign in again
          </a>
        </p>
      ) : (
        env && <p className="muted">{retryText(env)}</p>
      )}
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
  background: "Model knowledge, not from the NASA corpus",
};

const CLAIM_GLYPH: Record<Claim["kind"], string> = {
  source_reports: "g-source",
  observed: "g-observed",
  computed: "g-computed",
  predicted: "g-predicted",
  interpretation: "g-interpretation",
  background: "g-background",
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

/** Every model release this server serves is qualified: the deployment serves it on the scores its
 * card names (the basis, when known, is the tooltip). The registry's internal state stays internal. */
export function Qualified({ basis }: { basis?: string | null }) {
  return (
    <span className="tag tag-ok" title={basis ?? "Served on the scores its model card lists"}>
      Qualified
    </span>
  );
}

/** A model output's status: a forecast of a served release is qualified; a refusal or failure says what happened. */
export function OutputState({ status }: { status: string | null | undefined }) {
  const s = status ?? "";
  if (s === "experimental" || isQualified(s)) return <Qualified />;
  return <span className="tag tag-warn">{words(s || "unknown")}</span>;
}

/** An evaluation's verdict, as the evaluation computed it. */
export function VerdictTag({ verdict }: { verdict: string }) {
  return <span className={`tag ${verdict === "qualified" || isQualified(verdict) ? "tag-ok" : ""}`}>{words(verdict)}</span>;
}

/** A 401 from the origin's Access check (behind Cloudflare Access): the browser holds a session the
 * edge accepts but the origin cannot verify (expired, or signed before a key rotation). Signing out
 * of Access and in again issues a fresh assertion; retrying the same request cannot. */
export function staleSignIn(error: unknown): boolean {
  return error instanceof ApiError && error.httpStatus === 401 && error.envelope.code === "scope_denied" && /not authenticated/i.test(error.envelope.message);
}
