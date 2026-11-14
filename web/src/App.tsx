import { useRef, useState, type KeyboardEvent } from "react";
import { useQuery } from "@tanstack/react-query";
import { call, type Health, type Origin } from "./api";
import { SCHEMA_VERSION } from "./contracts/generated";
import { Explore } from "./workspaces/Explore";
import { Compare } from "./workspaces/Compare";
import { Lab } from "./workspaces/Lab";

const TABS = [
  ["explore", "Explore"],
  ["compare", "Compare"],
  ["lab", "Experiment Lab"],
] as const;
type Tab = (typeof TABS)[number][0];

export function App() {
  const [origin, setOrigin] = useState<Origin>("registry");
  const [tab, setTab] = useState<Tab>("explore");
  const [compareSet, setCompareSet] = useState<string[]>([]);
  const [runId, setRunId] = useState<string | null>(null);
  const health = useQuery({ queryKey: [origin, "health"], queryFn: () => call<Health>(origin, "/health"), retry: false });
  // The /fixtures origin exists only when the server runs with --dev.
  const fixtures = useQuery({ queryKey: ["fixtures", "health"], queryFn: () => call<Health>("fixtures", "/health"), retry: false });
  const tabRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const onKey = (e: KeyboardEvent, i: number) => {
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
    if (!d) return;
    const j = (i + d + TABS.length) % TABS.length;
    setTab(TABS[j][0]);
    tabRefs.current[j]?.focus();
  };
  const switchOrigin = (o: Origin) => {
    setOrigin(o);
    setCompareSet([]);
    setRunId(null);
  };
  const isFixture = origin === "fixtures";
  return (
    <div className={isFixture ? "app fixture-mode" : "app"}>
      {isFixture && (
        <div className="fixture-banner" role="alert">
          Development fixture — synthetic. Records, release and results below are generated for development; they are not NASA data or scientific results.
        </div>
      )}
      <header className="top">
        <h1>
          IGNIS <span className="muted">microgravity combustion lab</span>
        </h1>
        <div className="origin" role="group" aria-label="Data origin">
          <button aria-pressed={origin === "registry"} onClick={() => switchOrigin("registry")}>
            Registry
          </button>
          {fixtures.data && (
            <button aria-pressed={isFixture} onClick={() => switchOrigin("fixtures")} className="fixture-btn">
              Development fixture — synthetic
            </button>
          )}
        </div>
      </header>
      <nav role="tablist" aria-label="Workspaces" className="tabs">
        {TABS.map(([id, label], i) => (
          <button
            key={id}
            role="tab"
            id={`tab-${id}`}
            aria-selected={tab === id}
            aria-controls={`panel-${id}`}
            tabIndex={tab === id ? 0 : -1}
            ref={(el) => (tabRefs.current[i] = el)}
            onClick={() => setTab(id)}
            onKeyDown={(e) => onKey(e, i)}
          >
            {label}
            {id === "compare" && compareSet.length > 0 ? ` (${compareSet.length})` : ""}
          </button>
        ))}
      </nav>
      <main id={`panel-${tab}`} role="tabpanel" aria-labelledby={`tab-${tab}`}>
        {health.error && (
          <div className="refusal refusal-unavailable" role="status">
            <strong>Server unavailable.</strong> Start it with <code>cargo run -p ignis-cli -- serve --dev</code>.
          </div>
        )}
        {tab === "explore" && (
          <Explore
            origin={origin}
            compareSet={compareSet}
            toggleCompare={(id) => setCompareSet((s) => (s.includes(id) ? s.filter((x) => x !== id) : s.length >= 8 ? s : [...s, id]))}
            openLab={(id) => {
              setRunId(id);
              setTab("lab");
            }}
          />
        )}
        {tab === "compare" && <Compare origin={origin} runIds={compareSet} clear={() => setCompareSet([])} />}
        {tab === "lab" && <Lab origin={origin} runId={runId} setRunId={setRunId} />}
      </main>
      <footer className="muted">
        contracts v{SCHEMA_VERSION}
        {health.data && (
          <>
            {" "}
            · {health.data.data_origin.label} · {health.data.registry.experiments} experiments · evidence catalog{" "}
            {health.data.evidence_catalog?.available ? "available" : "unavailable"} · {health.data.experts.length} expert{health.data.experts.length === 1 ? "" : "s"} in process
          </>
        )}
      </footer>
    </div>
  );
}
