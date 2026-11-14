// T8: one time/selection store per answer, so the views it places move together: hovering the chart
// moves the video to that instant, and playing the video moves the chart's cursor. Times are seconds
// since a named `time_origin`; two views link only when they name the same origin.
import { createContext, useContext, useMemo, useState, type ReactNode } from "react";

export interface LinkedTime {
  origin: string;
  t: number;
  /** Which view set it, so a view does not react to its own update. */
  by: string;
}

interface Ctx {
  time: LinkedTime | null;
  setTime: (t: LinkedTime | null) => void;
}

const LinkedCtx = createContext<Ctx | null>(null);

export function LinkedViews({ children }: { children: ReactNode }) {
  const [time, setTime] = useState<LinkedTime | null>(null);
  const v = useMemo(() => ({ time, setTime }), [time]);
  return <LinkedCtx.Provider value={v}>{children}</LinkedCtx.Provider>;
}

/** The answer's linked time, or a no-op store when the view is shown on its own (side panel). */
export function useLinkedTime(): Ctx {
  return useContext(LinkedCtx) ?? NONE;
}
const NONE: Ctx = { time: null, setTime: () => {} };
