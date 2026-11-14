// Research workspace (T1): the researcher follows an episode — conversation, tool calls, jobs,
// artifacts, the cited answer — opens sources at their locators and sees model releases with their
// qualification state. Mock mode runs on the C0 fixtures without any server; live mode uses /api/v1.

import { useEffect, useMemo, useState, type FormEvent } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ArtifactRef, EvidenceRef, Origin, ResearchAnswer } from "../contracts/generated";
import { DEFAULT_FAULTS, liveClient, mockClient, type MockStreamFaults, type Mode, type ResearchClient } from "../research/client";
import {
  AnswerView,
  ArtifactCard,
  ArtifactPayload,
  ErrorBox,
  EvidenceViewer,
  eventSummary,
  JobLine,
  OriginBadge,
  PageOrigin,
  ReleaseCard,
  SearchResults,
  ToolResultCard,
  VisualBundleCard,
} from "../research/components";
import { emptyState, type EpisodeState } from "../research/eventLog";
import { FIXTURES } from "../research/fixtures";
import { followEpisode, type StreamNotice } from "../research/stream";

type Notice = StreamNotice & { at: number };

export function Research() {
  const [mode, setMode] = useState<Mode>("mock");
  const [faults, setFaults] = useState<MockStreamFaults>(DEFAULT_FAULTS);
  const [episodeId, setEpisodeId] = useState(FIXTURES.episode.episode_id);
  const [run, setRun] = useState(0);
  const [state, setState] = useState<EpisodeState>(() => emptyState(episodeId));
  const [notices, setNotices] = useState<Notice[]>([]);
  const [following, setFollowing] = useState(false);
  const [evidence, setEvidence] = useState<EvidenceRef | null>(null);
  const [artifact, setArtifact] = useState<ArtifactRef | null>(null);

  // A new client per (mode, run): mock faults fire once per run.
  const client = useMemo(() => (mode === "mock" ? mockClient(faults) : liveClient()), [mode, run]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const ac = new AbortController(); // StrictMode mounts twice: the first stream is aborted here
    setState(emptyState(episodeId));
    setNotices([]);
    setEvidence(null);
    setArtifact(null);
    setFollowing(true);
    followEpisode(client, emptyState(episodeId), {
      signal: ac.signal,
      onState: (s) => !ac.signal.aborted && setState(s),
      onNotice: (n) => !ac.signal.aborted && setNotices((xs) => [...xs.slice(-49), { ...n, at: Date.now() }]),
    }).finally(() => !ac.signal.aborted && setFollowing(false));
    return () => ac.abort();
  }, [client, episodeId]);

  const episode = useQuery({ queryKey: [mode, run, "episode", episodeId], queryFn: ({ signal }) => client.getEpisode(episodeId, signal), retry: false });
  const latest = state.answers[state.answers.length - 1] ?? null;
  const answer = useQuery({
    queryKey: [mode, run, "answer", latest?.artifact.artifact_id],
    queryFn: () => client.getArtifactJson(latest!.artifact.artifact_id) as Promise<ResearchAnswer>,
    enabled: latest !== null && latest.artifact.contract === "ResearchAnswer",
    retry: false,
  });
  const models = useQuery({ queryKey: [mode, run, "models"], queryFn: () => client.listModels(), retry: false });

  const openEvidence = (e: EvidenceRef) => {
    setEvidence(e);
    requestAnimationFrame(() => document.getElementById("evidence-viewer")?.focus());
  };
  const openArtifact = (a: ArtifactRef) => {
    setArtifact(a);
    requestAnimationFrame(() => document.getElementById("artifact-inspector")?.focus());
  };

  const pageOrigin: Origin = mode === "mock" ? "synthetic_fixture" : "system";
  const statusTone = state.status === "completed" ? "ok" : state.status === "failed" || state.status === "cancelled" ? "bad" : "";

  return (
    <PageOrigin.Provider value={pageOrigin}>
      <div className="research">
        {mode === "mock" && (
          <div className="fixture-banner" role="note">
            Mock mode — C0 contract fixtures (synthetic). {FIXTURES.notice}
          </div>
        )}

        <div className="rs-toolbar" role="group" aria-label="Connection">
          <div className="rs-seg" role="group" aria-label="Data source">
            <button aria-pressed={mode === "mock"} onClick={() => setMode("mock")}>
              Mock
            </button>
            <button aria-pressed={mode === "live"} onClick={() => setMode("live")}>
              Live
            </button>
          </div>
          <label className="rs-episode">
            <span>Episode</span>
            <input value={episodeId} onChange={(e) => setEpisodeId(e.target.value.trim())} aria-describedby="ep-help" spellCheck={false} />
          </label>
          <button onClick={() => setRun((r) => r + 1)}>Replay from start</button>
          <span className="rs-stream-state" aria-live="polite">
            {following ? "Streaming" : "Stream closed"} · seq <span className="num">{state.cursor}</span>
          </span>
          {mode === "mock" && (
            <details className="rs-faults">
              <summary>Stream faults</summary>
              <div className="rs-faults-body">
                <label>
                  Drop connection after seq{" "}
                  <select
                    value={faults.dropAfterSeq ?? ""}
                    onChange={(e) => setFaults((f) => ({ ...f, dropAfterSeq: e.target.value === "" ? null : Number(e.target.value) }))}
                  >
                    <option value="">never</option>
                    {FIXTURES.events.slice(0, -1).map((e) => (
                      <option key={e.seq} value={e.seq}>
                        {e.seq}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  <input type="checkbox" checked={faults.goneOnResume} onChange={(e) => setFaults((f) => ({ ...f, goneOnResume: e.target.checked }))} /> 410 Gone on resume
                </label>
                <label>
                  Overlap on resume{" "}
                  <input
                    type="number"
                    min={0}
                    max={10}
                    value={faults.resumeOverlap}
                    onChange={(e) => setFaults((f) => ({ ...f, resumeOverlap: Math.max(0, Number(e.target.value) || 0) }))}
                  />
                </label>
                <label>
                  Pacing (ms){" "}
                  <input
                    type="number"
                    min={0}
                    max={3000}
                    step={50}
                    value={faults.pacingMs}
                    onChange={(e) => setFaults((f) => ({ ...f, pacingMs: Math.max(0, Number(e.target.value) || 0) }))}
                  />
                </label>
                <p className="muted">Faults apply when you replay from start.</p>
              </div>
            </details>
          )}
        </div>
        <p id="ep-help" className="sr-only">
          {mode === "mock" ? "Mock mode serves only the fixture episode; other ids return not_found." : "Live mode needs the /api/v1 server (task M2)."}
        </p>

        <div className="rs-grid">
          <main className="rs-main">
            <header className="rs-goal">
              <p className="rs-goal-meta">
                <span className={`rs-status ${statusTone}`}>{state.status ? state.status.replace(/_/g, " ") : "connecting"}</span>
                {episode.data && (
                  <>
                    {" "}
                    · corpus <code>{episode.data.pins.corpus_snapshot ?? "not pinned"}</code> · runtime {episode.data.runtime?.runtime ?? "not attached"}{" "}
                    <OriginBadge origin={episode.data.origin} />
                  </>
                )}
              </p>
              <h2 className="rs-goal-text">{episode.data?.goal ?? (episode.isPending ? "Loading the episode…" : "Episode not loaded")}</h2>
              {episode.error && <ErrorBox error={episode.error} title="Episode not loaded" />}
              {state.missing.map((m) => (
                <p key={m.from} className="warn">
                  Events {m.from}–{m.to} were not received (log truncated, 410). The state continues from the snapshot; that history is not shown.
                </p>
              ))}
            </header>

            <section className="rs-section" aria-labelledby="ans-h">
              <h2 id="ans-h">
                Answer {latest && <span className={`rs-status ${latest.status === "complete" ? "ok" : "warn"}`}>{latest.status.replace(/_/g, " ")}</span>}
              </h2>
              {!latest && <p className="muted">{following ? "The agent is still working. The answer appears here when it is complete." : "This episode has no answer."}</p>}
              {latest && latest.artifact.contract !== "ResearchAnswer" && (
                <p className="warn">The answer artifact has contract {latest.artifact.contract ?? "none"}; it is not rendered.</p>
              )}
              {answer.isPending && latest && <p className="muted">Loading the answer…</p>}
              {answer.error && <ErrorBox error={answer.error} title="Answer not loaded" />}
              {answer.data && <AnswerView client={client} answer={answer.data} onEvidence={openEvidence} onArtifact={openArtifact} />}
            </section>

            <section className="rs-section" aria-labelledby="conv-h">
              <h2 id="conv-h">Conversation</h2>
              {state.messageOrder.length === 0 && <p className="muted">No messages yet.</p>}
              {state.messageOrder.map((id) => {
                const m = state.messages[id];
                return (
                  <div key={id} className={`msg msg-${m.role}`}>
                    <span className="msg-role">{m.role === "user" ? "You" : "Assistant"}</span>
                    <p>
                      {m.text}
                      {!m.complete && <span className="muted"> (streaming…)</span>}
                    </p>
                  </div>
                );
              })}
              <Steering send={(text, requestId) => client.postMessage(episodeId, text, requestId)} />
            </section>

            <section className="rs-section" aria-labelledby="search-h">
              <h2 id="search-h">Search sources</h2>
              <SearchBox client={client} onEvidence={openEvidence} mode={mode} run={run} />
            </section>
          </main>

          <aside className="rs-rail" aria-label="Sources and activity">
            {evidence && (
              <section className="rs-section rs-focus" aria-labelledby="ev-h" id="evidence-viewer" tabIndex={-1}>
                <div className="rs-head">
                  <h2 id="ev-h">Source</h2>
                  <button onClick={() => setEvidence(null)} aria-label="Close the source">
                    Close
                  </button>
                </div>
                <EvidenceViewer client={client} evidence={evidence} />
              </section>
            )}
            {artifact && (
              <section className="rs-section rs-focus" aria-labelledby="art-h" id="artifact-inspector" tabIndex={-1}>
                <div className="rs-head">
                  <h2 id="art-h">Artifact</h2>
                  <button onClick={() => setArtifact(null)} aria-label="Close the artifact">
                    Close
                  </button>
                </div>
                <ArtifactCard artifact={artifact} />
                <ArtifactPayload client={client} artifact={artifact} />
              </section>
            )}

            <section className="rs-section" aria-labelledby="tl-h">
              <h2 id="tl-h">Activity</h2>
              {state.events.length === 0 && <p className="muted">No events yet.</p>}
              <ol className="timeline">
                {state.events.map((e) => (
                  <li key={e.seq} className={`ev ev-${e.body.kind.replace(".", "-")}`}>
                    <div className="ev-line">
                      <span className="seq num">#{e.seq}</span>
                      <span className="ev-time num">{e.occurred_at.slice(11, 19)}</span>
                      <span className="ev-text">{eventSummary(e)}</span>
                      {e.body.kind === "artifact.created" && (
                        <button className="link" onClick={() => openArtifact((e.body as { payload: { artifact: ArtifactRef } }).payload.artifact)}>
                          inspect
                        </button>
                      )}
                    </div>
                    {e.body.kind === "tool.finished" && <ToolResultCard tool={e.body.payload.result.tool} result={e.body.payload.result} onArtifact={openArtifact} />}
                    {e.body.kind === "job.updated" && <JobLine job={e.body.payload.job} />}
                  </li>
                ))}
              </ol>
              {Object.values(state.approvals).map((a) => (
                <p key={a.approval_id} className={a.decision ? "muted" : "warn"}>
                  Approval {a.approval_id}: {a.action} [{a.effect}] — {a.decision ?? "waiting for a decision"}
                </p>
              ))}
            </section>

            <section className="rs-section" aria-labelledby="models-h">
              <h2 id="models-h">Models</h2>
              {models.isPending && <p className="muted">Loading…</p>}
              {models.error && <ErrorBox error={models.error} title="Models not loaded" />}
              {models.data?.length === 0 && <p className="muted">No releases.</p>}
              {models.data?.map((r) => <ReleaseCard key={r.id} release={r} />)}
            </section>
          </aside>
        </div>

        <details className="rs-diagnostics">
          <summary>Diagnostics</summary>
          <section aria-labelledby="st-h">
            <h3 id="st-h">Stream log</h3>
            <ul className="notices">
              {notices.map((n, i) => (
                <li key={i} className={n.kind === "error" || n.kind === "gap" || n.kind === "gave_up" ? "warn" : "muted"}>
                  {noticeText(n)}
                </li>
              ))}
            </ul>
          </section>
          {mode === "mock" && (
            <section aria-labelledby="gal-h">
              <h3 id="gal-h">Reference states (C0 fixtures)</h3>
              <p className="muted">Each state that the workspace must show, rendered from its fixture.</p>
              <div className="rs-tiles">
                <div className="rs-tile">
                  <ToolResultCard tool="models_forecast" result={FIXTURES.tools.accepted} onArtifact={openArtifact} />
                </div>
                <div className="rs-tile">
                  <ToolResultCard tool="sources_read" result={FIXTURES.tools.failed} onArtifact={openArtifact} />
                </div>
                <div className="rs-tile">
                  <ToolResultCard tool="models_forecast" result={FIXTURES.tools.insufficient} onArtifact={openArtifact} />
                </div>
                <div className="rs-tile">
                  <VisualBundleCard bundle={FIXTURES.visualBundle} />
                </div>
              </div>
            </section>
          )}
        </details>
      </div>
    </PageOrigin.Provider>
  );
}

function noticeText(n: Notice): string {
  switch (n.kind) {
    case "connected":
      return `connected, resuming after seq ${n.after}`;
    case "ended":
      return `stream closed at seq ${n.cursor}`;
    case "duplicate":
      return `seq ${n.seq} received again: ignored (already applied)`;
    case "gap":
      return `gap: expected seq ${n.expected}, got ${n.got} — not applied, resuming`;
    case "foreign_event":
      return `event of episode ${n.episode_id} ignored`;
    case "resync":
      return `410: reloaded the episode snapshot; seq ${n.from}–${n.to} not received`;
    case "error":
      return `error ${n.error.code}${n.http_status ? ` (HTTP ${n.http_status})` : ""}: ${n.error.message}`;
    case "gave_up":
      return `stopped: ${n.reason}`;
  }
}

function Steering({ send }: { send: (text: string, clientRequestId: string) => Promise<void> }) {
  const [text, setText] = useState("");
  // Idempotency key of the current draft (CONTRACTS.md): a retry of the same draft reuses it, so a
  // POST that reached the server before a timeout is not applied twice. New key after success or edit.
  const [requestId, setRequestId] = useState(() => crypto.randomUUID());
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const submit = async (e: FormEvent) => {
    e.preventDefault();
    if (!text.trim()) return;
    setBusy(true);
    setError(null);
    try {
      await send(text.trim(), requestId);
      setText("");
      setRequestId(crypto.randomUUID());
    } catch (err) {
      setError(err);
    } finally {
      setBusy(false);
    }
  };
  return (
    <form onSubmit={submit} className="row steering">
      <label className="grow">
        <span className="sr-only">Message to the agent</span>
        <input value={text} onChange={(e) => (setText(e.target.value), setRequestId(crypto.randomUUID()))} placeholder="Steer the investigation…" disabled={busy} />
      </label>
      <button type="submit" disabled={busy || !text.trim()}>
        Send
      </button>
      {error != null && <ErrorBox error={error} title="Message not sent" />}
    </form>
  );
}

function SearchBox({ client, onEvidence, mode, run }: { client: ResearchClient; onEvidence: (e: EvidenceRef) => void; mode: Mode; run: number }) {
  const [query, setQuery] = useState("");
  const [submitted, setSubmitted] = useState<string | null>(null);
  const q = useQuery({
    queryKey: [mode, run, "search", submitted],
    queryFn: () => client.search({ schema_version: "evidence.v1", query: submitted!, mode: "hybrid" }),
    enabled: submitted !== null,
    retry: false,
  });
  return (
    <>
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          if (query.trim()) setSubmitted(query.trim());
        }}
      >
        <label className="grow">
          <span className="sr-only">Search query</span>
          <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="e.g. droplet extinction diameter" />
        </label>
        <button type="submit" disabled={!query.trim()}>
          Search
        </button>
      </form>
      {q.isFetching && <p className="muted">Searching…</p>}
      {q.error && <ErrorBox error={q.error} title="Search failed" />}
      {q.data && <SearchResults page={q.data} onEvidence={onEvidence} />}
    </>
  );
}
