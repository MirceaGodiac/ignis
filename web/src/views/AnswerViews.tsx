// T8: the component registry for `AnswerBlock::View` (K1.4 closed `ViewSpec` catalog). The agent
// places a component by kind and references; the UI renders it from its own trusted components and
// never renders agent markup. A view that references something the answer does not cite is refused
// with the reason (A2UI's model: the client owns the catalog). A new kind is a contract change.
// A reader sees what a view shows, not how it was addressed: artifact ids, pointers and the exact
// reason a view did not draw stay in a "Technical details" disclosure or a tooltip.
import { useMemo, useState, type ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ConditionsTable, ResearchAnswer, ViewSpec, VisualObservationBundle } from "../contracts/generated";
import { SeriesChart } from "../components/SeriesChart";
import { readSeries, seriesProblems, viewProblems, type Series } from "../research/viewSpec";
import { chartRows, rowCount, skippedNote, tableModel, type ChartView } from "../research/chartRows";
import { sourceTitle } from "./turns";
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
  source_excerpt: "Source passage",
  series_comparison: "Over time",
  conditions_matrix: "Test conditions",
  atmosphere_map: "Habitat atmospheres",
  video_overlay: "Video",
  release_card: "Model",
  artifact_table: "Table",
  chart: "Chart",
};

/**
 * A view that cannot be drawn: one plain sentence, and the exact reasons (ids, pointers, contract
 * names) folded under "Technical details" for whoever needs them.
 */
export function NotShown({ children, reasons }: { children: ReactNode; reasons: string[] }) {
  return (
    <div className="note note-gap small view-gap">
      <p>{children}</p>
      {reasons.length > 0 && (
        <details className="view-tech">
          <summary>Technical details</summary>
          <ul>
            {reasons.map((r) => (
              <li key={r}>{r}</li>
            ))}
          </ul>
        </details>
      )}
    </div>
  );
}

export function ViewBlock({ view, caption, answer, client, claimKeys, srcNo }: { view: ViewSpec; caption?: string | null; answer: ResearchAnswer; client: ResearchClient; claimKeys: string[]; srcNo?: Map<string, number> }) {
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
      <div className="view-body">
        {problems.length > 0 ? <NotShown reasons={problems}>This view refers to something the answer does not cite, so it is not drawn.</NotShown> : <ViewBody view={view} answer={answer} client={client} srcNo={srcNo} caption={caption} />}
      </div>
    </figure>
  );
}

function ViewBody({ view, answer, client, srcNo, caption }: { view: ViewSpec; answer: ResearchAnswer; client: ResearchClient; srcNo?: Map<string, number>; caption?: string | null }) {
  switch (view.kind) {
    case "source_excerpt":
      return <SourceExcerpt evidenceId={view.evidence_id} answer={answer} srcNo={srcNo} />;
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
      return <ChartArtifact view={view} client={client} caption={caption} />;
  }
}

function useArtifact<T>(client: ResearchClient, id: string) {
  const { researchMode } = useData();
  return useQuery({ queryKey: [researchMode, "artifact", id], queryFn: () => client.getArtifactJson(id) as Promise<T>, retry: false, staleTime: Infinity });
}

// ---------------------------------------------------------------- source_excerpt

function SourceExcerpt({ evidenceId, answer, srcNo }: { evidenceId: string; answer: ResearchAnswer; srcNo?: Map<string, number> }) {
  const panel = usePanel();
  const e = (answer.evidence ?? []).find((x) => x.evidence_id === evidenceId)!;
  const loc = e.locator;
  const where = loc?.page != null ? `page ${loc.page_label ?? loc.page}${loc.bbox ? ", marked region" : ""}` : loc?.pts_s != null ? `video at ${(+loc.pts_s).toFixed(1)} s` : loc?.path ? "a data table" : "record only";
  // The source by its title when a search named it, else by its number in this answer.
  const n = srcNo?.get(e.source_id);
  const name = sourceTitle(e.source_id) ?? (n != null ? `Source ${n}` : "The source");
  return (
    <div className="excerpt">
      {e.quote ? <blockquote className="excerpt-quote">{e.quote}</blockquote> : <p className="muted small">No quote is carried with this citation; open the original to read it at its place.</p>}
      <div className="view-meta">
        <EvidenceClassTag cls={e.evidence_class} />
        <span className="view-meta-t">
          <span className="view-meta-src" title={e.source_id}>
            {name}
          </span>
          <span title={loc?.path ?? undefined}>{where}</span>
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
  const bad = q.data.flatMap((s, i) => (typeof s === "string" ? [`series ${i + 1} (${view.series[i].artifact_id}): ${s}`] : []));
  if (bad.length) return <NotShown reasons={bad}>{bad.length === 1 && view.series.length > 1 ? "One of the series" : "The series"} could not be read from the results it names, so the chart is not drawn.</NotShown>;
  const series = q.data as Series[];
  const mismatch = seriesProblems(series);
  if (mismatch.length) return <NotShown reasons={mismatch}>These series are not on one time axis or in one unit, so they are not drawn together.</NotShown>;
  return <SeriesChart series={series} id={`series:${view.series.map((s) => s.artifact_id).join(",")}`} />;
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
    return <NotShown reasons={[`the bundle was extracted from ${q.data.video_source_id}, not from ${sourceId}`]}>The contour belongs to a different video, so it is not drawn over this one.</NotShown>;
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
  if (!t) return <NotShown reasons={[`${pointer || "the document"} of ${id} is not a table`]}>This result has no rows to show as a table.</NotShown>;
  return <DataTable columns={t.columns} rows={t.rows} />;
}

// ---------------------------------------------------------------- chart

function ChartArtifact({ view, client, caption }: { view: ChartView; client: ResearchClient; caption?: string | null }) {
  const q = useArtifact<unknown>(client, view.artifact_id);
  const d = useMemo(() => (q.data === undefined ? null : chartRows(q.data, view)), [q.data, view]);
  if (q.isPending) return <p className="muted small view-wait">Reading the rows…</p>;
  if (q.error) return <ErrorNote quiet error={q.error} title="The chart's artifact could not be read" />;
  if (typeof d === "string" || !d) return <NotShown reasons={[`${view.artifact_id}: ${d ?? "no rows"}`]}>The chart's rows could not be read from its result.</NotShown>;
  if (!d.points.length) return <NotShown reasons={[skippedNote(d) ?? `${view.artifact_id}${view.pointer}: ${d.total} rows`]}>None of the rows has a value that can be plotted.</NotShown>;
  const note = skippedNote(d);
  // A count of tests says something; a count of rows of a computed table does not, so it is not shown.
  const count = d.noun === "test" ? rowCount(d.points.length, d.noun) : null;
  return (
    <>
      <AnswerChart view={view} data={d} caption={caption} />
      {(count || note) && (
        <p className="view-foot">
          {count && <span>{count}</span>}
          {note && (
            <span className="view-skip" title={note}>
              {d.skipped} {d.skipped === 1 ? `${d.noun} has` : `${d.noun}s have`} no value to plot
            </span>
          )}
        </p>
      )}
    </>
  );
}
