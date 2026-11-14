// The side panel's own navigation: a stack of items, so a scientist can go claim → evidence →
// source → run and come back the same way. Items hold contract objects, not copies.
import { createContext, useCallback, useContext, useState, type ReactNode } from "react";
import type { ArtifactRef, Claim, EvidenceRef, ModelRelease } from "../contracts/generated";
import type { ToolCallState } from "../research/eventLog";

export type PanelItem =
  | { kind: "run"; id: string }
  | { kind: "evidence"; evidence: EvidenceRef; claim?: Claim; keyLabel?: string }
  | { kind: "claim"; claim: Claim; keyLabel: string; evidence: EvidenceRef[]; artifacts: ArtifactRef[] }
  | { kind: "artifact"; artifact: ArtifactRef; caption?: string }
  | { kind: "step"; call: ToolCallState; index: number }
  | { kind: "release"; release: ModelRelease };

export const itemKey = (i: PanelItem): string => {
  switch (i.kind) {
    case "run":
      return `run:${i.id}`;
    case "evidence":
      return `ev:${i.evidence.evidence_id}`;
    case "claim":
      return `claim:${i.claim.claim_id}`;
    case "artifact":
      return `art:${i.artifact.artifact_id}`;
    case "step":
      return `step:${i.call.call_id}`;
    case "release":
      return `rel:${i.release.id}`;
  }
};

interface PanelCtx {
  stack: PanelItem[];
  current: PanelItem | null;
  /** Open as the root of a new trail (from the page). */
  open: (i: PanelItem) => void;
  /** Follow a link inside the panel (keeps the trail). */
  push: (i: PanelItem) => void;
  back: () => void;
  close: () => void;
  isOpen: (key: string) => boolean;
}

const Ctx = createContext<PanelCtx | null>(null);

export function PanelProvider({ children }: { children: ReactNode }) {
  const [stack, setStack] = useState<PanelItem[]>([]);
  const open = useCallback((i: PanelItem) => setStack([i]), []);
  const push = useCallback((i: PanelItem) => setStack((s) => (s.length && itemKey(s[s.length - 1]) === itemKey(i) ? s : [...s, i])), []);
  const back = useCallback(() => setStack((s) => s.slice(0, -1)), []);
  const close = useCallback(() => setStack([]), []);
  const current = stack[stack.length - 1] ?? null;
  const isOpen = (key: string) => !!current && itemKey(current) === key;
  return <Ctx.Provider value={{ stack, current, open, push, back, close, isOpen }}>{children}</Ctx.Provider>;
}

export function usePanel(): PanelCtx {
  const v = useContext(Ctx);
  if (!v) throw new Error("usePanel outside PanelProvider");
  return v;
}
