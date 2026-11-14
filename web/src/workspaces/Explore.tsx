import { useState, type FormEvent } from "react";
import { useQuery } from "@tanstack/react-query";
import { call, type ExperimentList, type ExperimentSummary, type Origin, type SearchResult } from "../api";
import { asRefusal } from "../hooks";
import { RefusalBox } from "../components/Panels";
import { fmtGravity, fmtMeasured } from "../format";

function Snippet({ text, open, close }: { text: string; open: string; close: string }) {
  const parts: { t: string; hit: boolean }[] = [];
  let rest = text;
  while (rest.length) {
    const i = rest.indexOf(open);
    if (i < 0) {
      parts.push({ t: rest, hit: false });
      break;
    }
    parts.push({ t: rest.slice(0, i), hit: false });
    const j = rest.indexOf(close, i + open.length);
    const end = j < 0 ? rest.length : j;
    parts.push({ t: rest.slice(i + open.length, end), hit: true });
    rest = j < 0 ? "" : rest.slice(j + close.length);
  }
  return <p className="snippet">{parts.map((p, k) => (p.hit ? <mark key={k}>{p.t}</mark> : <span key={k}>{p.t}</span>))}</p>;
}


function ExperimentCard({ x, selected, onToggle, onOpen }: { x: ExperimentSummary; selected: boolean; onToggle: () => void; onOpen: () => void }) {
  return (
    <li className="card exp">
      <div className="exp-head">
        <label>
          <input type="checkbox" checked={selected} onChange={onToggle} /> <b>{x.id}</b>
        </label>
        <span className={`badge fam-${x.family}`}>{x.family}</span>
        {x.synthetic && <span className="badge badge-synthetic">synthetic</span>}
      </div>
      <p className="muted">
        {x.investigation_id} · {x.fuel_or_material ?? "material unknown"} · {x.configuration ?? "configuration unknown"} · gravity {fmtGravity(x.gravity?.value_si)}
      </p>
      <dl className="conds">
        {Object.entries(x.conditions).map(([k, c]) => (
          <div key={k}>
            <dt>{k}</dt>
            <dd>
              {fmtMeasured(c.value_si, c.unit_si)}
              {c.resolution !== "resolved" && <em> ({c.resolution})</em>}
            </dd>
          </div>
        ))}
      </dl>
      <p className="muted">
        {x.n_observations} observations ({x.quantities.map((q) => `${q.quantity} ×${q.count}`).join(", ") || "none"}) · endpoint {x.has_endpoint ? "recorded" : "missing"}
      </p>
      {x.gaps.length > 0 && (
        <details>
          <summary>Evidence gaps ({x.gaps.length})</summary>
          <ul>
            {x.gaps.map((g) => (
              <li key={g}>{g}</li>
            ))}
          </ul>
        </details>
      )}
      <button onClick={onOpen}>Open in Experiment Lab</button>
    </li>
  );
}

export function Explore({ origin, compareSet, toggleCompare, openLab }: { origin: Origin; compareSet: string[]; toggleCompare: (id: string) => void; openLab: (id: string) => void }) {
  const [q, setQ] = useState("");
  const [family, setFamily] = useState("");
  const [question, setQuestion] = useState("");
  const [asked, setAsked] = useState<string | null>(null);
  const exps = useQuery({
    queryKey: [origin, "experiments", family],
    queryFn: () => call<ExperimentList>(origin, `/experiments${family ? `?family=${family}` : ""}`),
  });
  const search = useQuery({
    queryKey: [origin, "search", asked],
    queryFn: ({ signal }) => call<SearchResult>(origin, "/evidence/search", { json: { query: asked, limit: 12 }, signal }),
    enabled: !!asked,
    retry: false,
  });
  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (question.trim()) setAsked(question.trim());
  };
  const shown = (exps.data?.experiments ?? []).filter((x) => !q || `${x.id} ${x.investigation_id} ${x.fuel_or_material ?? ""}`.toLowerCase().includes(q.toLowerCase()));
  const searchRefusal = asRefusal(search.error);
  return (
    <div className="explore">
      <section className="card" aria-labelledby="ask-h">
        <h2 id="ask-h">Findings</h2>
        <form onSubmit={submit} className="row">
          <label className="grow">
            <span className="sr-only">Question</span>
            <input value={question} onChange={(e) => setQuestion(e.target.value)} placeholder="e.g. How does oxygen affect flame persistence?" />
          </label>
          <button type="submit">Search evidence</button>
        </form>
        {search.isFetching && <p className="muted">searching…</p>}
        {searchRefusal && <RefusalBox refusal={searchRefusal} />}
        {search.data && (
          <>
            <p className="muted">
              {search.data.hits.length} passages · ranking: {search.data.ranking}
            </p>
            <ol className="hits">
              {search.data.hits.map(({ hit, source }) => (
                <li key={String(hit.chunk_id)} className="hit">
                  <Snippet text={hit.snippet} open={search.data!.highlight.open} close={search.data!.highlight.close} />
                  <p className="locator">
                    source <code>{String(hit.source_id)}</code>
                    {(hit.title ?? source?.title) ? ` — ${hit.title ?? source?.title}` : ""}
                    {hit.section ? ` · ${hit.section}` : ""} · page {hit.page ?? "—"} · chars {hit.char_start ?? "?"}–{hit.char_end ?? "?"}
                    {(hit.outcome_bearing === 1 || hit.outcome_bearing === true) && <span className="badge badge-outcome">outcome-bearing</span>}
                    {Array.isArray(hit.describes_runs) && hit.describes_runs.length > 0 && <> · runs {(hit.describes_runs as string[]).join(", ")}</>}
                  </p>
                </li>
              ))}
            </ol>
          </>
        )}
      </section>

      <section aria-labelledby="exp-h">
        <div className="row">
          <h2 id="exp-h" className="grow">
            Experiments {exps.data ? `(${shown.length})` : ""}
          </h2>
          <label>
            Family{" "}
            <select value={family} onChange={(e) => setFamily(e.target.value)}>
              <option value="">all</option>
              {["flex", "acme_normal", "acme_inverse", "bass", "other"].map((f) => (
                <option key={f}>{f}</option>
              ))}
            </select>
          </label>
          <label>
            <span className="sr-only">Filter</span>
            <input value={q} onChange={(e) => setQ(e.target.value)} placeholder="filter id / material" />
          </label>
        </div>
        {exps.error && <RefusalBox refusal={asRefusal(exps.error)!} />}
        {exps.data && exps.data.count === 0 && <p className="muted">The registry has no experiments yet ({origin === "registry" ? "run ignis data ingest" : "no fixtures"}).</p>}
        {exps.data && exps.data.registry_errors > 0 && <p className="warn">{exps.data.registry_errors} registry lines failed contract validation (see /health).</p>}
        <p className="muted">Selected for Compare: {compareSet.length ? compareSet.join(", ") : "none"}</p>
        <ul className="exp-list">
          {shown.map((x) => (
            <ExperimentCard key={x.id} x={x} selected={compareSet.includes(x.id)} onToggle={() => toggleCompare(x.id)} onOpen={() => openLab(x.id)} />
          ))}
        </ul>
      </section>
    </div>
  );
}
