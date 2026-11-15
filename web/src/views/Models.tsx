// Model releases and their qualification state. "Not qualified" is stated on every row: an
// experimental release's output is model output, never a result.
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { useReleases } from "../shell/queries";
import { FamilyMark, FAMILY_LABEL } from "../shell/icons";
import { ErrorNote, Gap, ReleaseState, shortId, words } from "../ui/common";

export function Models() {
  const { caps } = useData();
  const panel = usePanel();
  const releases = useReleases();
  return (
    <div className="view">
      <div className="view-head">
        <h1>Models</h1>
      </div>
      {caps.data?.models.available === false ? (
        <Gap title="Models are not available on this server">{caps.data.models.detail}</Gap>
      ) : (
        releases.error && <ErrorNote error={releases.error} title="Releases could not be loaded" />
      )}
      {releases.data && (
        <div className="tbl-wrap">
          <table className="tbl">
            <caption>Releases in the research service</caption>
            <thead>
              <tr>
                <th>Model</th>
                <th>Family</th>
                <th>State</th>
                <th>Weights</th>
                <th>Release</th>
              </tr>
            </thead>
            <tbody>
              {releases.data.map((r) => (
                <tr key={r.id} className="clickable" onClick={() => panel.open({ kind: "release", release: r })}>
                  <td>{r.model_name}</td>
                  <td>
                    <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
                      <FamilyMark family={r.family} size={9} /> {FAMILY_LABEL[r.family] ?? r.family}
                    </span>
                  </td>
                  <td>
                    <ReleaseState state={r.state} />
                  </td>
                  <td>{r.trained_on === "not_trained" ? "hand-written" : words(r.trained_on)}</td>
                  <td>
                    <code>{shortId(r.id, 20)}</code>
                  </td>
                </tr>
              ))}
              {releases.data.length === 0 && (
                <tr>
                  <td colSpan={5} className="dim">
                    No releases.
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
