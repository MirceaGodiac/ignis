// Agent text in the thread and in answers, made navigable: `**bold**`, `[label](https://…)` links
// (http(s) only, new tab) and run names. A run is linked when the data source knows it, by its id
// (`PSI-69:193F001:…`) or by NASA's name for it (FLEX-093, FLEX–238, 21088 A1, B1); clicking opens
// the run panel (conditions, NASA originals, forecast). Nothing is rendered as HTML.
import { Fragment, useMemo, type ReactNode } from "react";
import { useData } from "./data";
import { usePanel } from "./inspector";
import { runNameKey, splitRich, type RichPart } from "./richParts";

/** Run names and ids of the current data source, by comparison key. */
export function useRunIndex(): Map<string, string> {
  const { experiments } = useData();
  return useMemo(() => {
    const m = new Map<string, string>();
    for (const r of experiments.data?.experiments ?? []) {
      m.set(runNameKey(r.id), r.id);
      for (const a of r.aliases ?? []) {
        if (a.scheme === "pi" || a.scheme === "campaign_day" || a.scheme === "sample_number") continue;
        const k = runNameKey(a.name);
        // A name two runs share is ambiguous: link neither.
        if (m.has(k) && m.get(k) !== r.id) m.set(k, "");
        else m.set(k, r.id);
      }
    }
    return m;
  }, [experiments.data]);
}

export function RichText({ text }: { text: string }) {
  const runs = useRunIndex();
  const panel = usePanel();
  const parts = useMemo(() => splitRich(text, runs), [text, runs]);
  return <>{parts.map((p, i) => render(p, i, (id) => panel.open({ kind: "run", id })))}</>;
}

function render(p: RichPart, i: number, openRun: (id: string) => void): ReactNode {
  switch (p.kind) {
    case "text":
      return <Fragment key={i}>{p.text}</Fragment>;
    case "break":
      return <br key={i} />;
    case "bold":
      return <b key={i}>{p.parts.map((q, j) => render(q, j, openRun))}</b>;
    case "link":
      return (
        <a key={i} href={p.href} target="_blank" rel="noopener noreferrer">
          {p.label}
        </a>
      );
    case "run":
      return (
        <button key={i} className="linkish run-link" onClick={() => openRun(p.id)} title={`Open run ${p.id}`}>
          {p.text}
        </button>
      );
  }
}
