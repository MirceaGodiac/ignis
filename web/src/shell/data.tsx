// One data source for the whole app. "Synthetic" = the dev fixture registry (/api/fixtures) and
// the C0 research fixtures (server /api/fixtures/v1 when it runs with --dev, otherwise replayed in
// the browser). "Live" = the registry (/api) and the research runtime (/api/v1). The two never mix
// on one screen, so a link always resolves in the source it came from.
import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import { call, type ExperimentList, type Health, type Origin as LegacyOrigin } from "../api";
import { DEFAULT_FAULTS, httpClient, mockClient, type MockStreamFaults, type Mode, type ResearchClient } from "../research/client";
import type { Capabilities } from "../contracts/generated";

export type Source = "synthetic" | "live";

interface DataCtx {
  source: Source;
  setSource: (s: Source) => void;
  /** Legacy registry routes. */
  origin: LegacyOrigin;
  /** Research client for this source. */
  client: ResearchClient;
  researchMode: Mode;
  faults: MockStreamFaults;
  setFaults: (f: (x: MockStreamFaults) => MockStreamFaults) => void;
  /** Bumped to replay the research stream from the start (mock faults fire per replay). */
  replay: number;
  bumpReplay: () => void;
  health: UseQueryResult<Health>;
  experiments: UseQueryResult<ExperimentList>;
  caps: UseQueryResult<Capabilities>;
  synthetic: boolean;
  /** False while the app still probes for server fixtures (the research client may change). */
  ready: boolean;
}

const Ctx = createContext<DataCtx | null>(null);

const STORE = "ignis.source";
const readStored = (): Source => {
  try {
    return localStorage.getItem(STORE) === "live" ? "live" : "synthetic";
  } catch {
    return "synthetic";
  }
};

export function DataProvider({ children }: { children: ReactNode }) {
  const [source, setSourceState] = useState<Source>(readStored);
  const [faults, setFaults] = useState<MockStreamFaults>(DEFAULT_FAULTS);
  const [replay, setReplay] = useState(0);
  const setSource = (s: Source) => {
    setSourceState(s);
    try {
      localStorage.setItem(STORE, s);
    } catch {
      /* private mode: the choice lasts for this page only */
    }
  };
  const origin: LegacyOrigin = source === "synthetic" ? "fixtures" : "registry";

  // Server fixtures exist only with `ignis serve --dev`; without them the browser replays C0.
  const serverFixtures = useQuery({
    queryKey: ["probe", "fixtures-v1"],
    queryFn: () => httpClient("fixtures").capabilities(),
    retry: false,
    enabled: source === "synthetic",
    staleTime: 60_000,
  });
  const researchMode: Mode = source === "live" ? "live" : serverFixtures.isSuccess ? "fixtures" : "mock";
  const client = useMemo(() => (researchMode === "mock" ? mockClient(faults) : httpClient(researchMode)), [researchMode, replay]); // eslint-disable-line react-hooks/exhaustive-deps

  const health = useQuery({ queryKey: [origin, "health"], queryFn: () => call<Health>(origin, "/health"), retry: false });
  const experiments = useQuery({ queryKey: [origin, "experiments"], queryFn: () => call<ExperimentList>(origin, "/experiments"), retry: false });
  const caps = useQuery({ queryKey: [researchMode, replay, "capabilities"], queryFn: () => client.capabilities(), retry: false });

  useEffect(() => {
    document.documentElement.dataset.source = source;
  }, [source]);

  const value: DataCtx = {
    source,
    setSource,
    origin,
    client,
    researchMode,
    faults,
    setFaults,
    replay,
    bumpReplay: () => setReplay((r) => r + 1),
    health,
    experiments,
    caps,
    synthetic: source === "synthetic",
    ready: source === "live" || !serverFixtures.isPending,
  };
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useData(): DataCtx {
  const v = useContext(Ctx);
  if (!v) throw new Error("useData outside DataProvider");
  return v;
}
