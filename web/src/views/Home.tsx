// Home: ask, then the ways in: suggested questions, NASA's runs by investigation family (from the run
// registry this deployment serves), and the questions already asked.
import { useMemo } from "react";
import { useData } from "../shell/data";
import { FAMILY_LABEL, FamilyMark, IconArrow } from "../shell/icons";
import { useEpisodes } from "../shell/queries";
import { go, href } from "../shell/router";
import { AskBox, EXAMPLES } from "./AskBox";

const FAMILY_ORDER = ["flex", "acme_normal", "acme_inverse", "bass", "other"];

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
      const fuels = [...f.fuels.entries()].sort((a, b) => b[1] - a[1]).map(([n]) => n);
      return { family: k, runs: f.runs, fuels };
    });
  }, [experiments.data]);

  return (
    <div className="home">
      <h1>What do you want to find out?</h1>
      <AskBox onAsk={ask} autoFocus />

      <section className="home-sec" aria-labelledby="h-start">
        <h2 id="h-start">Start from</h2>
        <div className="card-grid">
          {EXAMPLES.map((x) => (
            <button key={x} className="q-card" onClick={() => ask(x)}>
              <span>{x}</span>
              <IconArrow />
            </button>
          ))}
        </div>
      </section>

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
                <span>{e.goal}</span>
                <span className="meta">{e.status.replace(/_/g, " ")}</span>
              </a>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
