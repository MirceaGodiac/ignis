// T8: the component registry for `AnswerBlock::View` (K1.4 closed `ViewSpec` catalog). The agent
// places a component by kind and references; the UI renders it from its own trusted components and
// never renders agent markup. A view that references something the answer does not cite is refused
// with the reason (A2UI's model: the client owns the catalog). A new kind is a contract change.
import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ConditionsTable, ResearchAnswer, ViewSpec, VisualObservationBundle } from "../contracts/generated";
import { SeriesChart } from "../components/SeriesChart";
import { readSeries, seriesProblems, tableOf, viewProblems, type Series } from "../research/viewSpec";
import type { HabitatId } from "../research/habitats";
import { conditionsView } from "../research/testPoints";
import { ConditionsMap, HOME, type Domain } from "./ConditionsMap";
import { resolvePointer } from "../research/units";
import type { ResearchClient } from "../research/client";
import { ContourVideo } from "../shell/ContourVideo";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { useReleases } from "../shell/queries";
import { ErrorNote, EvidenceClassTag, ReleaseState, shortId, words } from "../ui/common";
import { ConditionsMatrix } from "./Conditions";

const KIND_TITLE: Record<ViewSpec["kind"], string> = {
  source_excerpt: "Source excerpt",
  series_comparison: "Series comparison",
  conditions_matrix: "Conditions matrix",
  atmosphere_map: "Habitat atmospheres",
  video_overlay: "Video with contour",
  release_card: "Model release",
  artifact_table: "Table",
};

export function ViewBlock({ view, caption, answer, client, claimKeys }: { view: ViewSpec; caption?: string | null; answer: ResearchAnswer; client: ResearchClient; claimKeys: string[] }) {
  const problems = viewProblems(view, answer);
  return (
    <figure className="view-block" aria-label={KIND_TITLE[view.kind]}>
      <figcaption>
        <b>{KIND_TITLE[view.kind]}.</b> {caption ?? ""}
        {claimKeys.length > 0 && <span className="dim"> · illustrates claim {claimKeys.join(", ")}</span>}
      </figcaption>
      {problems.length > 0 ? <p className="note note-refusal small">Not shown: {problems.join("; ")}.</p> : <ViewBody view={view} answer={answer} client={client} />}
    </figure>
  );
}

function ViewBody({ view, answer, client }: { view: ViewSpec; answer: ResearchAnswer; client: ResearchClient }) {
  switch (view.kind) {
    case "source_excerpt":
      return <SourceExcerpt evidenceId={view.evidence_id} answer={answer} />;
    case "series_comparison":
      return <SeriesView view={view} client={client} />;
    case "conditions_matrix":
      return <TableArtifact id={view.table} client={client} render={(t) => <ConditionsMatrix t={t} />} />;
    case "atmosphere_map":
      return <TableArtifact id={view.table} client={client} render={(t) => <AnswerMap t={t} />} />;
    case "video_overlay":
      return <BundleArtifact id={view.bundle} sourceId={view.source_id} client={client} />;
    case "release_card":
      return <ReleaseCard id={view.release_id} />;
    case "artifact_table":
      return <ArtifactTable id={view.artifact_id} pointer={view.pointer} client={client} />;
  }
}

function useArtifact<T>(client: ResearchClient, id: string) {
  const { researchMode } = useData();
  return useQuery({ queryKey: [researchMode, "artifact", id], queryFn: () => client.getArtifactJson(id) as Promise<T>, retry: false, staleTime: Infinity });
}

// ---------------------------------------------------------------- source_excerpt

function SourceExcerpt({ evidenceId, answer }: { evidenceId: string; answer: ResearchAnswer }) {
  const panel = usePanel();
  const e = (answer.evidence ?? []).find((x) => x.evidence_id === evidenceId)!;
  const loc = e.locator;
  const where = loc?.page != null ? `page ${loc.page}${loc.bbox ? ", marked region" : ""}` : loc?.pts_s != null ? `video at ${loc.pts_s} s` : loc?.path ? loc.path : "metadata only";
  return (
    <div className="panel-sec">
      <div className="toolbar">
        <EvidenceClassTag cls={e.evidence_class} />
        <span className="muted small">
          <code>{e.source_id}</code> · {where}
        </span>
        <button className="btn btn-s" style={{ marginLeft: "auto" }} onClick={() => panel.open({ kind: "evidence", evidence: e })}>
          Open original
        </button>
      </div>
      {e.quote ? <blockquote className="quote">{e.quote}</blockquote> : <p className="muted small">No quote is carried with this citation; open the original to read it at its place.</p>}
    </div>
  );
}

// ---------------------------------------------------------------- series_comparison

function SeriesView({ view, client }: { view: Extract<ViewSpec, { kind: "series_comparison" }>; client: ResearchClient }) {
  const { researchMode } = useData();
  const q = useQuery({
    queryKey: [researchMode, "series", view.series.map((s) => `${s.artifact_id}${s.value_pointer}`).join("|")],
    queryFn: async () => {
      const docs = new Map<string, unknown>();
      for (const id of new Set(view.series.map((s) => s.artifact_id))) docs.set(id, await client.getArtifactJson(id));
      return view.series.map((s) => readSeries(docs.get(s.artifact_id), s));
    },
    retry: false,
    staleTime: Infinity,
  });
  if (q.isPending) return <p className="muted small">Reading the series…</p>;
  if (q.error) return <ErrorNote error={q.error} title="The series could not be read" />;
  const bad = q.data.flatMap((s, i) => (typeof s === "string" ? [`series ${i + 1}: ${s}`] : []));
  if (bad.length) return <p className="note note-refusal small">Not drawn: {bad.join("; ")}.</p>;
  const series = q.data as Series[];
  const mismatch = seriesProblems(series);
  if (mismatch.length) return <p className="note note-refusal small">Not drawn on one axis: {mismatch.join("; ")}.</p>;
  return (
    <>
      <SeriesChart series={series} id={`series:${view.series.map((s) => s.artifact_id).join(",")}`} />
    </>
  );
}

// ---------------------------------------------------------------- tables and bundles

/** The atmosphere map of the table an answer cites: its own test points, zoomable, nothing hidden. */
function AnswerMap({ t }: { t: ConditionsTable }) {
  const v = useMemo(() => conditionsView(t.rows), [t]);
  const [domain, setDomain] = useState<Domain>(HOME);
  const [selected, setSelected] = useState<string | null>(null);
  const maxKpa = Math.max(150, ...v.points.map((p) => p.kpa));
  return <ConditionsMap points={v.points} hidden={NONE} habitats={ALL_HABITATS} selected={selected} onSelect={setSelected} domain={domain} setDomain={setDomain} maxKpa={maxKpa} />;
}
const NONE = new Set<string>();
const ALL_HABITATS = new Set<HabitatId>(["moon", "suit", "iss"]);

function TableArtifact({ id, client, render }: { id: string; client: ResearchClient; render: (t: ConditionsTable) => JSX.Element }) {
  const q = useArtifact<ConditionsTable>(client, id);
  if (q.isPending) return <p className="muted small">Reading the conditions…</p>;
  if (q.error) return <ErrorNote error={q.error} title="The conditions table could not be read" />;
  return render(q.data);
}

function BundleArtifact({ id, sourceId, client }: { id: string; sourceId: string; client: ResearchClient }) {
  const q = useArtifact<VisualObservationBundle>(client, id);
  if (q.isPending) return <p className="muted small">Reading the contour bundle…</p>;
  if (q.error) return <ErrorNote error={q.error} title="The contour bundle could not be read" />;
  if (q.data.video_source_id !== sourceId)
    return (
      <p className="note note-refusal small">
        Not shown: the bundle was extracted from {q.data.video_source_id}, not from {sourceId}.
      </p>
    );
  return <ContourVideo bundle={q.data} />;
}

// ---------------------------------------------------------------- release_card

function ReleaseCard({ id }: { id: string }) {
  const panel = usePanel();
  const releases = useReleases();
  const r = releases.data?.find((x) => x.id === id);
  if (releases.isPending) return <p className="muted small">Reading the model registry…</p>;
  if (!r) return <p className="note note-gap small">Release {shortId(id, 22)} is not in this server's model registry.</p>;
  return (
    <div className="toolbar">
      <ReleaseState state={r.state} />
      <b>{r.model_name}</b>
      <span className="muted small">
        {words(r.trained_on)} · {r.construction ? words(r.construction.construction_method) : "no build manifest"}
      </span>
      <button className="btn btn-s" style={{ marginLeft: "auto" }} onClick={() => panel.open({ kind: "release", release: r })}>
        Construction and checks
      </button>
    </div>
  );
}

// ---------------------------------------------------------------- artifact_table

const PAGE = 50;
const cell = (v: unknown) => (v == null ? "—" : typeof v === "object" ? JSON.stringify(v) : String(v));

function ArtifactTable({ id, pointer, client }: { id: string; pointer: string; client: ResearchClient }) {
  const q = useArtifact<unknown>(client, id);
  const [shown, setShown] = useState(PAGE);
  if (q.isPending) return <p className="muted small">Reading the artifact…</p>;
  if (q.error) return <ErrorNote error={q.error} title="The artifact could not be read" />;
  const t = tableOf(resolvePointer(q.data, pointer));
  if (!t) return <p className="note note-refusal small">Not shown: {pointer || "the document"} is not a table.</p>;
  return (
    <>
      <div className="tbl-wrap">
        <table className="tbl">
          <thead>
            <tr>
              {t.columns.map((c) => (
                <th key={c}>{c}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {t.rows.slice(0, shown).map((r, i) => (
              <tr key={i}>
                {r.map((v, j) => (
                  <td key={j} className="num">
                    {cell(v)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {t.rows.length > shown && (
        <button className="btn btn-s" onClick={() => setShown((n) => n + PAGE)}>
          Show {Math.min(PAGE, t.rows.length - shown)} more of {t.rows.length - shown}
        </button>
      )}
    </>
  );
}
