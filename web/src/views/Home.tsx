// Home: one question field, a few starting questions, and the work in progress. Tools live in the sidebar.
import { useData } from "../shell/data";
import { useEpisodes } from "../shell/queries";
import { go, href } from "../shell/router";
import { IconArrow } from "../shell/icons";
import { AskBox, EXAMPLES } from "./AskBox";

export function Home() {
  const { synthetic } = useData();
  const episodes = useEpisodes();
  const ask = (q: string) => go("research", null, { new: "1", q });
  const recent = episodes.data?.slice(0, 4) ?? [];
  return (
    <div className="home">
      <h1>What do you want to find out?</h1>
      <AskBox onAsk={ask} autoFocus synthetic={synthetic} />
      <section className="list-sec" aria-labelledby="h-start">
        <h2 id="h-start">Start from</h2>
        <div className="suggestions">
          {EXAMPLES.map((x) => (
            <button key={x} onClick={() => ask(x)}>
              <IconArrow width={14} height={14} /> {x}
            </button>
          ))}
        </div>
      </section>
      {recent.length > 0 && (
        <section className="list-sec" aria-labelledby="h-recent">
          <h2 id="h-recent">Continue</h2>
          {recent.map((e) => (
            <a key={e.episode_id} className="row-link" href={href("research", e.episode_id)}>
              <span className={`dot ${e.status === "completed" ? "dot-ok" : "dot-run"}`} aria-hidden="true" />
              <span className="grow">{e.goal}</span>
              <span className="meta">{e.status.replace(/_/g, " ")}</span>
            </a>
          ))}
        </section>
      )}
    </div>
  );
}
