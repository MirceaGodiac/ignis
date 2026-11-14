// TEMPORARY MIRRORS of /api/v1 request and response types that are defined only in the server
// (crates/ignis-server/src/v1/mod.rs), not in ignis-core contracts, so they are absent from
// generated.ts. AGENTS.md rule 3 wants cross-lane types in ignis-core/src/contracts/; until they move
// there (or a committed openapi.json with a drift test exists), these are the smallest shapes the UI
// sends or reads. Every field with a generated alias uses it. Do not add fields here that the UI
// does not use. Asked of Mircea (M2 owner) in the T3 PR.

import type { ApprovalDecision, ClientRequestId, EpisodeId, EventId, ReleaseId, ToolEffect } from "../contracts/generated";

/** `CreateEpisode` (v1/mod.rs). */
export interface CreateEpisode {
  goal: string;
  client_request_id: ClientRequestId;
  model_releases?: ReleaseId[];
}

/** `PostMessage` (v1/mod.rs). */
export interface PostMessage {
  text: string;
  client_request_id: ClientRequestId;
}

/** `MessageAccepted` (v1/mod.rs). */
export interface MessageAccepted {
  episode_id: EpisodeId;
  event_id: EventId;
  seq: number;
  duplicate: boolean;
}

/** `HostDecision` (v1/mod.rs): the subset of `ApprovalDecision` a host may send. */
export type HostDecision = Extract<ApprovalDecision, "approved" | "denied">;

/** `ServiceStatus` (v1/mod.rs). */
export interface ServiceStatus {
  available: boolean;
  detail: string;
}

/** `ToolStatus` (v1/mod.rs). */
export interface ToolStatus {
  name: string;
  effect: ToolEffect;
  available: boolean;
  reason?: string;
}

/** `Capabilities` (v1/mod.rs). `runtime_build` is not read by the UI and is left out. */
export interface Capabilities {
  agent_runtime: ServiceStatus;
  knowledge: ServiceStatus;
  datasets: ServiceStatus;
  models: ServiceStatus;
  artifact_store: ServiceStatus;
  mcp: ServiceStatus;
  tools: ToolStatus[];
}

/** `ActionName` (v1/mod.rs): `POST /episodes/{id}/{action}`. */
export type EpisodeAction = "pause" | "resume" | "cancel";
