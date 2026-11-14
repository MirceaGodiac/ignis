// Home: ask, with a few suggested questions under the prompt; then the ways into NASA's data, each
// a strip of real numbers or targets that opens the page already filtered: runs by investigation
// family (run registry), crew atmospheres (the conditions page at that habitat), and the questions
// already asked.
import { useMemo } from "react";
import { HABITAT_LIST } from "../research/habitats";
import { useData } from "../shell/data";
import { FAMILY_LABEL, FamilyMark, IconArrow, IconCompare, IconConditions, IconLab, IconSources, IconThread } from "../shell/icons";
import { useEpisodes } from "../shell/queries";
import { go, href } from "../shell/router";
import { AskBox, EXAMPLES } from "./AskBox";

const FAMILY_ORDER = ["flex", "acme_normal", "acme_inverse", "bass", "other"];
const PILL_ICON = [IconConditions, IconThread, IconSources, IconLab];

export function Home() {
  const { experiments } = useData();
  const episodes = useEpisodes();
  const ask = (q: string) => go("research", null, { new: "1", q });
  const recent = episodes.data?.slice(0, 4) ?? [];

  const families = useMemo(() => {
    const m = new Map<string, { runs: number; fuels: Map<string, number> }>();
    for (const x of experiments.data?.experiments ?? []) {
      const f = m.get(x.family) ?? { runs: 0, fuels: new Map() };
      f.runs += 1;
      if (x.fuel_or_material) f.fuels.set(x.fuel_or_material, (f.fuels.get(x.fuel_or_material) ?? 0) + 1);
      m.set(x.family, f);
    }
    return FAMILY_ORDER.filter((k) => m.has(k)).map((k) => {
      const f = m.get(k)!;
      return { family: k, runs: f.runs, fuels: [...f.fuels.entries()].sort((a, b) => b[1] - a[1]).map(([n]) => n) };
    });
  }, [experiments.data]);

  return (
    <div className="home">
      <h1>What do you want to find out?</h1>
      <div className="ask-wrap">
        <AskBox onAsk={ask} autoFocus />
        <div className="pills">
          {EXAMPLES.map((x, i) => {
            const I = PILL_ICON[i % PILL_ICON.length];
            return (
              <button key={x} className="pill" onClick={() => ask(x)}>
                <I /> {x}
              </button>
            );
          })}
        </div>
      </div>

      {families.length > 0 && (
        <section className="home-sec" aria-labelledby="h-runs">
          <div className="home-sec-head">
            <h2 id="h-runs">NASA runs</h2>
            <a href={href("runs")}>
              All {experiments.data?.count} runs <IconArrow width={14} height={14} />
            </a>
          </div>
          <div className="fam-grid">
            {families.map((f) => (
              <a key={f.family} className="fam-cell" href={href("runs", null, { family: f.family })}>
                <span className="fam-name">
                  <FamilyMark family={f.family} size={9} />
                  {FAMILY_LABEL[f.family] ?? f.family}
                </span>
                <span className="fam-n">{f.runs}</span>
                <span className="fam-fuels">{f.fuels.slice(0, 3).join(", ") || " "}</span>
              </a>
            ))}
          </div>
        </section>
      )}

      <section className="home-sec" aria-labelledby="h-atmos">
        <div className="home-sec-head">
          <h2 id="h-atmos">Crew atmospheres</h2>
          <a href={href("conditions", null, { tab: "map" })}>
            Atmosphere map <IconArrow width={14} height={14} />
          </a>
        </div>
        <div className="fam-grid">
          {HABITAT_LIST.map((h) => (
            <a key={h.id} className="fam-cell" href={href("runs", null, { habitat: h.id })}>
              <span className="fam-name">{h.label}</span>
              <span className="fam-n fam-n-s">{h.sub}</span>
              <span className="fam-fuels">Runs tested here</span>
            </a>
          ))}
        </div>
      </section>

      <section className="home-sec" aria-labelledby="h-go">
        <h2 id="h-go">Go to</h2>
        <div className="go-grid">
          {(
            [
              ["sources", "Search NASA documents", IconSources],
              ["conditions", "Habitat relevance", IconConditions],
              ["lab", "Forecast a run", IconLab],
              ["compare", "Compare runs", IconCompare],
            ] as const
          ).map(([v, label, I]) => (
            <a key={v} className="go-item" href={href(v)}>
              <span className="go-ico">
                <I />
              </span>
              {label}
            </a>
          ))}
        </div>
      </section>

      {recent.length > 0 && (
        <section className="home-sec" aria-labelledby="h-recent">
          <div className="home-sec-head">
            <h2 id="h-recent">Continue</h2>
            <a href={href("research")}>
              All questions <IconArrow width={14} height={14} />
            </a>
          </div>
          <div className="card-grid">
            {recent.map((e) => (
              <a key={e.episode_id} className="q-card" href={href("research", e.episode_id)}>
                <span className="q-ico">
                  <IconThread />
                </span>
                <span className="q-t">
                  {e.goal}
                  <span className="meta">{e.updated_at.slice(0, 16).replace("T", " ")}</span>
                </span>
              </a>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
