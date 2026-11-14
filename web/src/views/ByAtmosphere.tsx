// Conditions, by crew atmosphere: which investigations tested inside it (counted from actual test
// points), which came nearest and by how much, and which NASA's tables cannot place at all. Each card
// joins the conditions table (numbers) with the atlas card (what was burned, the fire-safety need in
// NASA's words, the narrative).
import { useState } from "react";
import { firstSentence, INVESTIGATIONS, invName } from "../data/investigations";
import { HABITAT_LIST, type HabitatId } from "../research/habitats";
import { offsetText, rankForAtmosphere, UNDECIDABLE_TEXT, type Standing } from "../research/testPoints";
import { usePanel } from "../shell/inspector";
import { href } from "../shell/router";
import { IconChevron } from "../shell/icons";
import { Gap } from "../ui/common";
import { ShapeSwatch } from "./ConditionsMap";
import { useConditionsView } from "./MapView";

const pct = (r: [number, number]) => (Math.abs(r[1] - r[0]) < 0.0005 ? `${(r[0] * 100).toFixed(0)} %` : `${(r[0] * 100).toFixed(0)}–${(r[1] * 100).toFixed(0)} %`);
const psi = (r: [number, number]) => {
  const a = r[0] / 6.894757;
  const b = r[1] / 6.894757;
  return Math.abs(b - a) < 0.05 ? `${a.toFixed(1)} psi` : `${a.toFixed(1)}–${b.toFixed(1)} psi`;
};

export function ByAtmosphere({ initial }: { initial?: string | null }) {
  const q = useConditionsView();
  const [hid, setHid] = useState<HabitatId>(() => (initial === "suit" || initial === "iss" ? initial : "moon"));
  const [showUndecidable, setShowUndecidable] = useState(false);
  if (q.isPending) return <p className="muted">Loading…</p>;
  if (q.error || !q.data) return <Gap title="Conditions are not available on this server">{(q.error as Error)?.message}</Gap>;
  const h = HABITAT_LIST.find((x) => x.id === hid)!;
  const ranked = rankForAtmosphere(q.data, h);
  const inside = ranked.filter((s) => s.runsInside > 0);
  const nearest = ranked.filter((s) => s.runsInside === 0 && s.nearest);
  const undecidable = ranked.filter((s) => !s.nearest);

  return (
    <div className="atm">
      <div className="atm-head">
        <div className="seg seg-l" role="group" aria-label="Crew atmosphere">
          {HABITAT_LIST.map((x) => (
            <button key={x.id} aria-pressed={hid === x.id} onClick={() => setHid(x.id)}>
              {x.label}
            </button>
          ))}
        </div>
        <span className="atm-spec">
          {h.sub}
          {hid === "moon" && " · partial gravity, of which PSI has no tests yet"}
        </span>
      </div>

      <section className="atm-sec" aria-labelledby="h-inside">
        <h2 id="h-inside">
          Tested in this atmosphere <span className="n">{inside.length}</span>
        </h2>
        {inside.length === 0 ? <p className="muted">No NASA test in these tables was run in this atmosphere.</p> : <div className="inv-grid">{inside.map((s) => <InvCard key={s.investigation} s={s} hid={hid} />)}</div>}
      </section>

      {nearest.length > 0 && (
        <section className="atm-sec" aria-labelledby="h-near">
          <h2 id="h-near">
            Nearest, not tested inside <span className="n">{nearest.length}</span>
          </h2>
          <div className="inv-grid">
            {nearest.map((s) => (
              <InvCard key={s.investigation} s={s} hid={hid} />
            ))}
          </div>
        </section>
      )}

      {undecidable.length > 0 && (
        <section className="atm-sec" aria-labelledby="h-und">
          <button className="atm-fold" onClick={() => setShowUndecidable((o) => !o)} aria-expanded={showUndecidable}>
            <IconChevron className="chev" />
            <h2 id="h-und">
              Not placeable from NASA's tables <span className="n">{undecidable.length}</span>
            </h2>
          </button>
          {showUndecidable && (
            <ul className="und-list">
              {undecidable.map((s) => (
                <UndecidableRow key={s.investigation} s={s} />
              ))}
            </ul>
          )}
        </section>
      )}
    </div>
  );
}

const NEED_LABEL: Record<string, string> = { prevention: "Prevention", detection: "Detection", suppression: "Suppression" };

function InvCard({ s, hid }: { s: Standing; hid: HabitatId }) {
  const panel = usePanel();
  const inv = INVESTIGATIONS.get(s.investigation);
  const n = s.nearest!;
  return (
    <article className="inv-card">
      <header>
        <ShapeSwatch id={s.investigation} size={14} />
        <span className="t">
          <b>{invName(s.investigation)}</b>
          <span>
            {s.investigation}
            {inv?.hardware ? ` · ${inv.hardware}` : ""}
          </span>
        </span>
      </header>
      {s.runsInside > 0 ? (
        <p className="inv-stat">
          <span className="big">{s.runsInside}</span> runs inside, at {s.pointsInside} test point{s.pointsInside === 1 ? "" : "s"}
        </p>
      ) : (
        <p className="inv-stat">
          Nearest test <span className="big-s">{offsetText(n.offset)}</span> away
        </p>
      )}
      <dl className="inv-kv">
        <dt>Tested</dt>
        <dd className="num">
          {s.o2 && pct(s.o2)} O₂ · {s.kpa && psi([s.kpa[0], s.kpa[1]])}
        </dd>
        <dt>Burned</dt>
        <dd>{(inv?.fuels.length ? inv.fuels : [n.point.fuel ?? "not stated"]).slice(0, 3).join(", ")}</dd>
      </dl>
      {inv && inv.needs.length > 0 && (
        <div className="needs">
          {inv.needs.map((x) => (
            <span key={x.need} className={`need${x.inference ? " inferred" : ""}`} title={x.inference ? `${x.basis ?? ""}\n(${x.inference})` : (x.basis ?? undefined)}>
              {NEED_LABEL[x.need] ?? x.need}
            </span>
          ))}
        </div>
      )}
      {inv?.relevance && <p className="inv-rel">{firstSentence(inv.relevance)}</p>}
      <footer>
        <a className="btn btn-s" href={href("conditions", null, { tab: "map", inv: s.investigation })}>
          On the map
        </a>
        {s.runsInside > 0 && (
          <a className="btn btn-s" href={href("conditions", null, { tab: "runs", habitat: hid })}>
            Runs here
          </a>
        )}
        {inv && (
          <button className="btn btn-quiet btn-s" onClick={() => panel.open({ kind: "investigation", id: s.investigation })}>
            About
          </button>
        )}
      </footer>
    </article>
  );
}

function UndecidableRow({ s }: { s: Standing }) {
  const panel = usePanel();
  const inv = INVESTIGATIONS.get(s.investigation);
  return (
    <li>
      <ShapeSwatch id={s.investigation} />
      <b>{invName(s.investigation)}</b>
      <span className="muted">{s.runs} runs · {s.undecidable ? UNDECIDABLE_TEXT[s.undecidable] : ""}</span>
      {inv && (
        <button className="btn btn-quiet btn-s" onClick={() => panel.open({ kind: "investigation", id: s.investigation })}>
          About
        </button>
      )}
    </li>
  );
}
