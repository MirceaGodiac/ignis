// Episode controls and idempotent drafts. Pure module: type-only imports (tested with node --test).

import type { EpisodeStatus } from "../contracts/generated";
import type { EpisodeAction } from "../contracts/generated";

export const TERMINAL_STATUSES: EpisodeStatus[] = ["completed", "failed", "cancelled"];

/**
 * Actions the server accepts from a status (ignis-research `ResearchStore::act`): pause from any
 * active state; resume only from paused; cancel from anything but failed and cancelled. Completed
 * episodes can be cancelled by the server, but the UI does not offer it: there is nothing to stop.
 */
export function availableActions(status: EpisodeStatus | null | undefined): EpisodeAction[] {
  switch (status) {
    case "ready":
    case "running":
    case "waiting_tools":
    case "waiting_approval":
    case "waiting_user":
      return ["pause", "cancel"];
    case "paused":
      return ["resume", "cancel"];
    default:
      return [];
  }
}

/**
 * The idempotency key of a draft (CONTRACTS.md: `client_request_id`). A retry of the same draft
 * reuses the key, so a request that reached the server before a timeout is not applied twice; a new
 * key is made only after a success or when the text changes.
 */
export class DraftKey {
  private key: string;
  private readonly make: () => string;
  constructor(make: () => string = () => crypto.randomUUID()) {
    this.make = make;
    this.key = make();
  }
  get current(): string {
    return this.key;
  }
  /** The text changed: it is a different request. */
  edited(): string {
    this.key = this.make();
    return this.key;
  }
  /** The server accepted the request: the next one is new. */
  succeeded(): string {
    this.key = this.make();
    return this.key;
  }
}
