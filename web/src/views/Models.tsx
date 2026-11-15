// The models this server serves, as a researcher reads them: a plain name and what each forecasts,
// what it was scored on and its lead score. Every served model is qualified; ids, construction and
// build checks open in the side panel, under "Technical details".
import type { ModelCard } from "../contracts/generated";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { headlineScore, modelTitle, scoreParts, useModelCards } from "../shell/models";
import { useReleases } from "../shell/queries";
import { ErrorNote, Gap, Qualified } from "../ui/common";

export function Models() {
  const { caps } = useData();
  const panel = usePanel();
  const releases = useReleases();
  const { cards } = useModelCards();
  const byId = new Map<string, ModelCard>((cards.data ?? []).map((c) => [c.release_id, c]));
  return (
    <div className="view">
      <div className="view-head">
        <h1>Models</h1>
      </div>
      {caps.data?.models.available === false ? (
        <Gap title="Models are not available on this server">{caps.data.models.detail}</Gap>
      ) : (
        releases.error && <ErrorNote error={releases.error} title="The models could not be loaded" />
      )}
      {releases.data && (
        <div className="tbl-wrap">
          <table className="tbl models-tbl">
            <caption>Hand-written models served here</caption>
            <thead>
              <tr>
                <th>Model</th>
                <th>Scored on</th>
                <th className="r">Lead score</th>
              </tr>
            </thead>
            <tbody>
              {releases.data.map((r) => {
                const card = byId.get(r.id) ?? null;
                const score = headlineScore(card);
                const parts = score && scoreParts(score);
                return (
                  <tr
                    key={r.id}
                    className="clickable"
                    tabIndex={0}
                    onClick={() => panel.open({ kind: "release", release: r })}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        panel.open({ kind: "release", release: r });
                      }
                    }}
                  >
                    <td>
                      <span className="models-name">
                        <span className="models-title">
                          <b>{modelTitle(r.model_name, card)}</b>
                          <Qualified basis={card?.qualification.basis} />
                        </span>
                        {card && <span className="models-sum">{card.summary}</span>}
                      </span>
                    </td>
                    <td className="models-basis">{card?.qualification.basis ?? ""}</td>
                    <td className="r models-score" title={score?.metric}>
                      {parts && (
                        <>
                          <b className="num">{parts.value}</b>
                          <span className="muted">vs {parts.vs}</span>
                        </>
                      )}
                    </td>
                  </tr>
                );
              })}
              {releases.data.length === 0 && (
                <tr>
                  <td colSpan={3} className="dim">
                    No models are served here.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
