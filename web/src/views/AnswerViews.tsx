// T8: the component registry for `AnswerBlock::View` (K1.4 closed `ViewSpec` catalog). The agent
// places a component by kind and references; the UI renders it from its own trusted components and
// never renders agent markup. A view that references something the answer does not cite is refused
// with the reason (A2UI's model: the client owns the catalog). A new kind is a contract change.
import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ConditionsTable, ResearchAnswer, ViewSpec, VisualObservationBundle } from "../contracts/generated";
import { SeriesChart } from "../components/SeriesChart";
import { readSeries, seriesProblems, viewProblems, type Series } from "../research/viewSpec";
import { chartRows, rowCount, skippedNote, tableModel, type ChartView } from "../research/chartRows";
import { compactKeys } from "../research/noteText";
import { AnswerChart } from "../components/AnswerChart";
import { DataTable } from "../components/DataTable";
import "./answer.css";
import type { HabitatId } from "../research/habitats";
import { conditionsView } from "../research/testPoints";
import { ConditionsMap, HOME, type Domain } from "./ConditionsMap";
import { resolvePointer } from "../research/units";
import type { ResearchClient } from "../research/client";
import { ContourVideo } from "../shell/ContourVideo";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { useReleases } from "../shell/queries";
import { ErrorNote, EvidenceClassTag, Qualified, shortId } from "../ui/common";
import { headlineScore, modelTitle, scoreParts, useServedCard, writtenBy } from "../shell/models";
import { ConditionsMatrix } from "./Conditions";

const KIND_TITLE: Record<ViewSpec["kind"], string> = {
  source_excerpt: "Source excerpt",
  series_comparison: "Series comparison",
  conditions_matrix: "Conditions matrix",
  atmosphere_map: "Habitat atmospheres",
  video_overlay: "Video with contour",
  release_card: "Model release",
  artifact_table: "Table",
  chart: "Chart",
};

export function ViewBlock({ view, caption, answer, client, claimKeys }: { view: ViewSpec; caption?: string | null; answer: ResearchAnswer; client: ResearchClient; claimKeys: string[] }) {
  const problems = viewProblems(view, answer);
  const claims = claimKeys.length ? compactKeys(claimKeys) : null;
  return (
    <figure className={`view-block view-${view.kind}`} aria-label={caption ? `${KIND_TITLE[view.kind]}: ${caption}` : KIND_TITLE[view.kind]}>
      <figcaption className="view-cap">
        <span className="view-kind">{KIND_TITLE[view.kind]}</span>
        {caption && <span className="view-caption">{caption}</span>}
        {claims && (
          <span className="view-claims" title={claims.short === claims.full ? undefined : claims.full}>
            {claims.short}
          </span>
        )}
      </figcaption>
      <div className="view-body">{problems.length > 0 ? <p className="note note-gap small">Not shown: {problems.join("; ")}.</p> : <ViewBody view={view} answer={answer} client={client} />}</div>
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
    case "chart":
      return <ChartArtifact view={view} client={client} />;
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
    <div className="excerpt">
      {e.quote ? <blockquote className="excerpt-quote">{e.quote}</blockquote> : <p className="muted small">No quote is carried with this citation; open the original to read it at its place.</p>}
      <div className="view-meta">
        <EvidenceClassTag cls={e.evidence_class} />
        <span className="view-meta-t">
          <code title={e.source_id}>{shortId(e.source_id, 36)}</code>
          <span>{where}</span>
        </span>
        <button className="btn btn-s" onClick={() => panel.open({ kind: "evidence", evidence: e })}>
          Open original
        </button>
      </div>
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
  if (q.isPending) return <p className="muted small view-wait">Reading the series…</p>;
  if (q.error) return <ErrorNote quiet error={q.error} title="The series could not be read" />;
  const bad = q.data.flatMap((s, i) => (typeof s === "string" ? [`series ${i + 1}: ${s}`] : []));
  if (bad.length) return <p className="note note-gap small">Not drawn: {bad.join("; ")}.</p>;
  const series = q.data as Series[];
  const mismatch = seriesProblems(series);
  if (mismatch.length) return <p className="note note-gap small">Not drawn on one axis: {mismatch.join("; ")}.</p>;
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
  return <ConditionsMap points={v.points} hidden={NONE} habitats={ALL_HABITATS} selected={selected} onSelect={setSelected} domain={domain} setDomain={setDomain} maxKpa={maxKpa} inline />;
}
const NONE = new Set<string>();
const ALL_HABITATS = new Set<HabitatId>(["moon", "suit", "iss"]);

function TableArtifact({ id, client, render }: { id: string; client: ResearchClient; render: (t: ConditionsTable) => JSX.Element }) {
  const q = useArtifact<ConditionsTable>(client, id);
  if (q.isPending) return <p className="muted small view-wait">Reading the conditions…</p>;
  if (q.error) return <ErrorNote quiet error={q.error} title="The conditions table could not be read" />;
  return render(q.data);
}

function BundleArtifact({ id, sourceId, client }: { id: string; sourceId: string; client: ResearchClient }) {
  const q = useArtifact<VisualObservationBundle>(client, id);
  if (q.isPending) return <p className="muted small view-wait">Reading the contour bundle…</p>;
  if (q.error) return <ErrorNote quiet error={q.error} title="The contour bundle could not be read" />;
  if (q.data.video_source_id !== sourceId)
    return (
      <p className="note note-gap small">
        Not shown: the bundle was extracted from {q.data.video_source_id}, not from {sourceId}.
      </p>
    );
  return <ContourVideo bundle={q.data} />;
}

// ---------------------------------------------------------------- release_card

function ReleaseCard({ id }: { id: string }) {
  const panel = usePanel();
  const releases = useReleases();
  const card = useServedCard(id);
  const r = releases.data?.find((x) => x.id === id);
  if (releases.isPending) return <p className="muted small view-wait">Reading the model…</p>;
  if (!r) return <p className="note note-gap small">{card ? card.title : "This model"} is not served here.</p>;
  const score = headlineScore(card);
  const parts = score && scoreParts(score);
  return (
    <div className="release-card">
      <span className="release-glyph" aria-hidden="true">
        <span className="glyph g-predicted" />
      </span>
      <span className="release-t">
        <span className="release-name">
          <b>{modelTitle(r.model_name, card)}</b>
          <Qualified basis={card?.qualification.basis} />
        </span>
        {card && <span className="release-sum">{card.summary}</span>}
        <span className="release-meta">
          <span>{writtenBy(r)}</span>
          {score && parts && (
            <span title={score.metric}>
              <b className="num">{parts.value}</b> vs {parts.vs}
            </span>
          )}
        </span>
      </span>
      <button className="btn btn-s" onClick={() => panel.open({ kind: "release", release: r })}>
        About this model
      </button>
    </div>
  );
}

// ---------------------------------------------------------------- artifact_table

function ArtifactTable({ id, pointer, client }: { id: string; pointer: string; client: ResearchClient }) {
  const q = useArtifact<unknown>(client, id);
  const t = useMemo(() => (q.data === undefined ? null : tableModel(resolvePointer(q.data, pointer))), [q.data, pointer]);
  if (q.isPending) return <p className="muted small view-wait">Reading the artifact…</p>;
  if (q.error) return <ErrorNote quiet error={q.error} title="The artifact could not be read" />;
  if (!t) return <p className="note note-gap small">Not shown: {pointer || "the document"} is not a table.</p>;
  return <DataTable columns={t.columns} rows={t.rows} />;
}

// ---------------------------------------------------------------- chart

function ChartArtifact({ view, client }: { view: ChartView; client: ResearchClient }) {
  const q = useArtifact<unknown>(client, view.artifact_id);
  const d = useMemo(() => (q.data === undefined ? null : chartRows(q.data, view)), [q.data, view]);
  if (q.isPending) return <p className="muted small view-wait">Reading the rows…</p>;
  if (q.error) return <ErrorNote quiet error={q.error} title="The chart's artifact could not be read" />;
  if (typeof d === "string" || !d) return <p className="note note-gap small">Not drawn: {d ?? "no rows"}.</p>;
  if (!d.points.length) return <p className="note note-gap small">Not drawn: none of the {d.total} rows has plottable values{d.skipped ? ` (${skippedNote(d)})` : ""}.</p>;
  const note = skippedNote(d);
  return (
    <>
      <AnswerChart view={view} data={d} />
      <p className="view-foot">
        <span title={`From ${view.artifact_id}${view.pointer ? ` ${view.pointer}` : ""}`}>{rowCount(d.points.length, d.noun)}</span>
        {note && <span className="view-skip">{note}</span>}
      </p>
    </>
  );
}
