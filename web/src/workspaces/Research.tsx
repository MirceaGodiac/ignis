// Research workspace (T1 + T3): the researcher starts or opens an episode and follows it —
// conversation, tool calls, jobs, artifacts, the cited answer — opens sources at their locators and
// sees model releases with their qualification state. Three origins share one client: Mock (C0
// fixtures in the browser), Server fixtures (`ignis serve --dev`, /api/fixtures/v1) and Live
// (/api/v1). The server's capabilities decide what the UI offers; "not configured" is never shown
// as a scientific result.

import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import type { ArtifactRef, EvidenceRef, ResearchAnswer, ResearchEpisode } from "../contracts/generated";
import { ApiError, DEFAULT_FAULTS, httpClient, mockClient, MODE_LABEL, modeOrigin, type MockStreamFaults, type Mode, type ResearchClient } from "../research/client";
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
import { availableActions, DraftKey, TERMINAL_STATUSES } from "../research/episodes";
import { emptyState, type EpisodeState } from "../research/eventLog";
import { FIXTURES } from "../research/fixtures";
import type { Capabilities, EpisodeAction, HostDecision, ServiceStatus } from "../research/serverTypes";
import { followEpisode, type StreamNotice } from "../research/stream";

type Notice = StreamNotice & { at: number };

const MODES: Mode[] = ["mock", "fixtures", "live"];
const ACTION_LABEL: Record<EpisodeAction, string> = { pause: "Pause", resume: "Resume", cancel: "Cancel" };

export function Research() {
  const qc = useQueryClient();
  const [mode, setMode] = useState<Mode>("mock");
  const [faults, setFaults] = useState<MockStreamFaults>(DEFAULT_FAULTS);
  const [episodeId, setEpisodeId] = useState<string>(FIXTURES.episode.episode_id);
  const [run, setRun] = useState(0); // replay from start
  const [follow, setFollow] = useState(0); // continue from the cursor (after a message or resume)
  const [state, setState] = useState<EpisodeState>(() => emptyState(episodeId));
  const stateRef = useRef(state);
  stateRef.current = state;
  const [notices, setNotices] = useState<Notice[]>([]);
  const [following, setFollowing] = useState(false);
  const [evidence, setEvidence] = useState<EvidenceRef | null>(null);
  const [artifact, setArtifact] = useState<ArtifactRef | null>(null);
  const [actionError, setActionError] = useState<unknown>(null);

  // A new client per (mode, run): mock faults fire once per run.
  const client = useMemo(() => (mode === "mock" ? mockClient(faults) : httpClient(mode)), [mode, run]); // eslint-disable-line react-hooks/exhaustive-deps

  const caps = useQuery({ queryKey: [mode, run, "capabilities"], queryFn: () => client.capabilities(), retry: false });
  const episodes = useQuery({ queryKey: [mode, run, "episodes"], queryFn: () => client.listEpisodes(), retry: false });

  // Open the first listed episode when the current id is not in this origin's list.
  useEffect(() => {
    const list = episodes.data;
    if (!list) return;
    if (!list.some((e) => e.episode_id === episodeId)) setEpisodeId(list[0]?.episode_id ?? "");
  }, [episodes.data]); // eslint-disable-line react-hooks/exhaustive-deps

  // A new episode or origin starts from an empty state; `follow` continues from the cursor.
  const lastKey = useRef("");
  useEffect(() => {
    if (!episodeId) {
      setState(emptyState(""));
      setFollowing(false);
      return;
    }
    const key = `${mode}|${run}|${episodeId}`;
    const fresh = key !== lastKey.current;
    lastKey.current = key;
    const initial = fresh ? emptyState(episodeId) : stateRef.current;
    const ac = new AbortController(); // StrictMode mounts twice: the first stream is aborted here
    if (fresh) {
      setState(initial);
      setNotices([]);
      setEvidence(null);
      setArtifact(null);
      setActionError(null);
    }
    setFollowing(true);
    followEpisode(client, initial, {
      signal: ac.signal,
      onState: (s) => !ac.signal.aborted && setState(s),
      onNotice: (n) => !ac.signal.aborted && setNotices((xs) => [...xs.slice(-49), { ...n, at: Date.now() }]),
    }).finally(() => !ac.signal.aborted && setFollowing(false));
    return () => ac.abort();
  }, [client, episodeId, follow]); // eslint-disable-line react-hooks/exhaustive-deps

  const episode = useQuery({
    queryKey: [mode, run, "episode", episodeId],
    queryFn: ({ signal }) => client.getEpisode(episodeId, signal),
    enabled: !!episodeId,
    retry: false,
  });
  const latest = state.answers[state.answers.length - 1] ?? null;
  const answer = useQuery({
    queryKey: [mode, run, "answer", latest?.artifact.artifact_id],
    queryFn: () => client.getArtifactJson(latest!.artifact.artifact_id) as Promise<ResearchAnswer>,
    enabled: latest !== null && latest.artifact.contract === "ResearchAnswer",
    retry: false,
  });
  const models = useQuery({ queryKey: [mode, run, "models"], queryFn: () => client.listModels(), retry: false, enabled: caps.data?.models.available !== false });

  // Before the first status event (a new episode has none), the snapshot says where it stands.
  const status = state.status ?? episode.data?.status ?? null;
  const runtime = caps.data?.agent_runtime;
  const knowledge = caps.data?.knowledge;

  const openEvidence = (e: EvidenceRef) => {
    setEvidence(e);
    requestAnimationFrame(() => document.getElementById("evidence-viewer")?.focus());
  };
  const openArtifact = (a: ArtifactRef) => {
    setArtifact(a);
    requestAnimationFrame(() => document.getElementById("artifact-inspector")?.focus());
  };
  // Always (re)start following from the cursor after a write: `following` read here can be stale
  // (the stream may close while the POST is in flight). The effect aborts the old stream, and
  // dedup by seq makes the overlap harmless.
  const continueFollowing = () => setFollow((f) => f + 1);
  const putEpisode = (ep: ResearchEpisode) => {
    qc.setQueryData([mode, run, "episode", ep.episode_id], ep);
    qc.setQueryData<ResearchEpisode[]>([mode, run, "episodes"], (xs) => (xs ? xs.map((x) => (x.episode_id === ep.episode_id ? ep : x)) : xs));
  };
  const act = async (a: EpisodeAction) => {
    setActionError(null);
    try {
      putEpisode(await client.act(episodeId, a));
      continueFollowing();
    } catch (e) {
      setActionError(e);
    }
  };
  const decide = async (approvalId: string, d: HostDecision) => {
    setActionError(null);
    try {
      await client.resolveApproval(episodeId, approvalId, d);
      continueFollowing();
    } catch (e) {
      setActionError(e);
    }
  };

  const statusTone = status === "completed" ? "ok" : status === "failed" || status === "cancelled" ? "bad" : "";
  const pending = Object.values(state.approvals).filter((a) => !a.decision);

  return (
    <PageOrigin.Provider value={modeOrigin(mode)}>
      <div className="research">
        {mode !== "live" && (
          // Short, because it stays on screen while scrolling; the full notice is its tooltip.
          <div className="fixture-banner" role="note" title={FIXTURES.notice}>
            Synthetic C0 fixtures ({mode === "mock" ? "in the browser" : "/api/fixtures/v1"}) — not NASA data or results; nothing here qualifies a model.
          </div>
        )}

        <div className="rs-toolbar" role="group" aria-label="Connection">
          <div className="rs-seg" role="group" aria-label="Data origin">
            {MODES.map((m) => (
              <button key={m} aria-pressed={mode === m} onClick={() => setMode(m)}>
                {MODE_LABEL[m]}
              </button>
            ))}
          </div>
          <label className="rs-episode">
            <span>Episode</span>
            <select value={episodeId} onChange={(e) => setEpisodeId(e.target.value)} disabled={!episodes.data?.length}>
              {!episodes.data?.length && <option value="">{episodes.isPending ? "Loading…" : "No episodes"}</option>}
              {episodes.data?.map((e) => (
                <option key={e.episode_id} value={e.episode_id}>
                  {e.goal.length > 60 ? `${e.goal.slice(0, 57)}…` : e.goal} · {(e.episode_id === episodeId && status ? status : e.status).replace(/_/g, " ")}
                </option>
              ))}
            </select>
          </label>
          <button onClick={() => setRun((r) => r + 1)}>Replay from start</button>
          <span className="rs-stream-state" aria-live="polite">
            {following ? "Streaming" : "Stream closed"} · seq <span className="num">{state.cursor}</span>
          </span>
          {mode === "mock" && <FaultControls faults={faults} setFaults={setFaults} />}
        </div>
        {episodes.error && <ErrorBox error={episodes.error} title="Episodes not loaded" />}
        <CapabilityStrip caps={caps.data} error={caps.error} />

        <div className="rs-grid">
          <main className="rs-main">
            {mode !== "mock" && (
              <NewInvestigation
                create={async (goal, key) => {
                  const ep = await client.createEpisode(goal, key);
                  qc.setQueryData<ResearchEpisode[]>([mode, run, "episodes"], (xs) => (xs?.some((x) => x.episode_id === ep.episode_id) ? xs : [ep, ...(xs ?? [])]));
                  setEpisodeId(ep.episode_id);
                }}
              />
            )}

            {episodeId ? (
              <header className="rs-goal">
                <p className="rs-goal-meta">
                  <span className={`rs-status ${statusTone}`}>{status ? status.replace(/_/g, " ") : "connecting"}</span>
                  {episode.data && (
                    <>
                      {" "}
                      · corpus <code>{episode.data.pins.corpus_snapshot ?? "not pinned"}</code> · runtime {episode.data.runtime?.runtime ?? "not attached"}{" "}
                      <OriginBadge origin={episode.data.origin} />
                    </>
                  )}
                </p>
                <h2 className="rs-goal-text">{episode.data?.goal ?? (episode.isPending ? "Loading the episode…" : "Episode not loaded")}</h2>
                {mode !== "mock" && (
                  <div className="rs-controls" role="group" aria-label="Episode controls">
                    {(["pause", "resume", "cancel"] as EpisodeAction[]).map((a) => (
                      <button key={a} onClick={() => act(a)} disabled={!availableActions(status).includes(a)}>
                        {ACTION_LABEL[a]}
                      </button>
                    ))}
                  </div>
                )}
                {episode.error && <ErrorBox error={episode.error} title="Episode not loaded" />}
                {actionError != null && <ErrorBox error={actionError} title="The server refused the action" />}
                {state.missing.map((m) => (
                  <p key={m.from} className="warn">
                    Events {m.from}–{m.to} were not received (log truncated, 410). The state continues from the snapshot; that history is not shown.
                  </p>
                ))}
              </header>
            ) : (
              <p className="muted rs-goal">No episode in this origin yet. Start an investigation above.</p>
            )}

            {pending.length > 0 && (
              <section className="rs-section" aria-labelledby="apr-h">
                <h2 id="apr-h">Waiting for your approval</h2>
                {pending.map((a) => (
                  <div key={a.approval_id} className="rs-approval">
                    <p>
                      {a.action} <span className="muted">({a.effect.replace(/_/g, " ")})</span>
                    </p>
                    <div className="row">
                      <button onClick={() => decide(a.approval_id, "approved")} disabled={!runtime?.available}>
                        Approve
                      </button>
                      <button onClick={() => decide(a.approval_id, "denied")} disabled={!runtime?.available}>
                        Deny
                      </button>
                    </div>
                    {!runtime?.available && <p className="muted">No agent runtime here, so a decision cannot be sent: {runtime?.detail ?? "capabilities not loaded"}.</p>}
                  </div>
                ))}
              </section>
            )}

            {episodeId && (
              <>
            <section className="rs-section" aria-labelledby="ans-h">
              <h2 id="ans-h">
                Answer {latest && <span className={`rs-status ${latest.status === "complete" ? "ok" : "warn"}`}>{latest.status.replace(/_/g, " ")}</span>}
              </h2>
              {!latest && (
                <p className="muted">
                  {status && TERMINAL_STATUSES.includes(status)
                    ? "This episode has no answer."
                    : runtime?.available === false && mode !== "mock"
                      ? "No agent runtime is configured, so no answer will come. Messages are still recorded."
                      : "The agent is still working. The answer appears here when it is complete."}
                </p>
              )}
              {latest && latest.artifact.contract !== "ResearchAnswer" && (
                <p className="warn">The answer artifact has contract {latest.artifact.contract ?? "none"}; it is not rendered.</p>
              )}
              {answer.isFetching && <p className="muted">Loading the answer…</p>}
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
              {episodeId && (
                <Steering
                  key={`${mode}|${episodeId}`} // a draft and its request key belong to one episode
                  disabledReason={mode === "mock" ? "Mock mode replays a fixed log. Use Server fixtures or Live to send messages." : status === "cancelled" || status === "failed" ? `The episode is ${status}.` : null}
                  note={runtime?.available === false ? "No agent runtime is configured: the message is recorded, but no agent will answer." : null}
                  send={async (text, key) => {
                    await client.postMessage(episodeId, text, key);
                    continueFollowing();
                  }}
                />
              )}
            </section>
              </>
            )}

            <section className="rs-section" aria-labelledby="search-h">
              <h2 id="search-h">Search sources</h2>
              <SearchBox client={client} onEvidence={openEvidence} mode={mode} run={run} knowledge={knowledge} />
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
              <h2 id="tl-h">
                Activity <span className="rs-head-note">times in UTC</span>
              </h2>
              {state.events.length === 0 && <p className="muted">No events yet.</p>}
              <ol className="timeline">
                {/* Partial message text is shown in the conversation; one row per delta is noise. */}
                {state.events
                  .filter((e) => e.body.kind !== "message.delta")
                  .map((e) => (
                  <li key={e.seq} className={`ev ev-${e.body.kind.replace(".", "-")}`} title={`seq ${e.seq}`}>
                    <div className="ev-line">
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
            </section>

            <section className="rs-section" aria-labelledby="models-h">
              <h2 id="models-h">Models</h2>
              {caps.data?.models.available === false ? (
                <p className="muted">Not configured here: {caps.data.models.detail}.</p>
              ) : (
                <>
                  {models.isPending && <p className="muted">Loading…</p>}
                  {models.error && <ErrorBox error={models.error} title="Models not loaded" />}
                  {models.data?.length === 0 && <p className="muted">No releases.</p>}
                  {models.data?.map((r) => <ReleaseCard key={r.id} release={r} />)}
                </>
              )}
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
          {caps.data && (
            <section aria-labelledby="tools-h">
              <h3 id="tools-h">Tools in this deployment</h3>
              <ul className="notices">
                {caps.data.tools.map((t) => (
                  <li key={t.name} className={t.available ? "muted" : "warn"}>
                    <code>{t.name}</code> ({t.effect.replace(/_/g, " ")}): {t.available ? "available" : `unavailable${t.reason ? ` — ${t.reason}` : ""}`}
                  </li>
                ))}
              </ul>
            </section>
          )}
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

function FaultControls({ faults, setFaults }: { faults: MockStreamFaults; setFaults: (f: (x: MockStreamFaults) => MockStreamFaults) => void }) {
  return (
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
          <input type="number" min={0} max={10} value={faults.resumeOverlap} onChange={(e) => setFaults((f) => ({ ...f, resumeOverlap: Math.max(0, Number(e.target.value) || 0) }))} />
        </label>
        <label>
          Pacing (ms){" "}
          <input type="number" min={0} max={3000} step={50} value={faults.pacingMs} onChange={(e) => setFaults((f) => ({ ...f, pacingMs: Math.max(0, Number(e.target.value) || 0) }))} />
        </label>
        <p className="muted">Faults apply when you replay from start.</p>
      </div>
    </details>
  );
}

/** What this deployment can do. "Not configured" is a deployment state, never a scientific result. */
function CapabilityStrip({ caps, error }: { caps: Capabilities | undefined; error: unknown }) {
  if (error) return <ErrorBox error={error} title="Capabilities not loaded" />;
  if (!caps) return null;
  const items: [string, ServiceStatus][] = [
    ["Agent", caps.agent_runtime],
    ["Knowledge", caps.knowledge],
    ["Models", caps.models],
    ["Datasets", caps.datasets],
  ];
  return (
    <ul className="rs-caps" aria-label="Services in this deployment">
      {items.map(([name, s]) => (
        <li key={name} title={s.detail}>
          <span className={`rs-status ${s.available ? "ok" : ""}`}>{s.available ? "●" : "○"}</span> {name}{" "}
          <span className="muted">{s.available ? "available" : "not configured"}</span>
        </li>
      ))}
    </ul>
  );
}

function NewInvestigation({ create }: { create: (goal: string, clientRequestId: string) => Promise<void> }) {
  const [goal, setGoal] = useState("");
  const key = useRef(new DraftKey());
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const submit = async (e: FormEvent) => {
    e.preventDefault();
    if (!goal.trim()) return;
    setBusy(true);
    setError(null);
    try {
      await create(goal.trim(), key.current.current);
      key.current.succeeded();
      setGoal("");
    } catch (err) {
      setError(err);
    } finally {
      setBusy(false);
    }
  };
  return (
    <form onSubmit={submit} className="rs-new" aria-labelledby="new-h">
      <label htmlFor="new-goal" id="new-h" className="rs-new-label">
        New investigation
      </label>
      <div className="row">
        <input
          id="new-goal"
          className="grow"
          value={goal}
          onChange={(e) => {
            setGoal(e.target.value);
            key.current.edited();
          }}
          placeholder="What do you want to find out? e.g. How does ambient pressure change the burning rate of methanol droplets?"
          disabled={busy}
        />
        <button type="submit" disabled={busy || !goal.trim()}>
          Start
        </button>
      </div>
      {error != null && <ErrorBox error={error} title="The investigation was not created" />}
    </form>
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
    case "restart":
      return `410: replaying from the start (${n.reason})`;
    case "error":
      return `error ${n.error.code}${n.http_status ? ` (HTTP ${n.http_status})` : ""}: ${n.error.message}`;
    case "gave_up":
      return `stopped: ${n.reason}`;
  }
}

function Steering({ send, disabledReason, note }: { send: (text: string, clientRequestId: string) => Promise<void>; disabledReason: string | null; note: string | null }) {
  const [text, setText] = useState("");
  const key = useRef(new DraftKey());
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const submit = async (e: FormEvent) => {
    e.preventDefault();
    if (!text.trim() || disabledReason) return;
    setBusy(true);
    setError(null);
    try {
      await send(text.trim(), key.current.current);
      key.current.succeeded();
      setText("");
    } catch (err) {
      setError(err);
    } finally {
      setBusy(false);
    }
  };
  return (
    <form onSubmit={submit} className="steering">
      <div className="row">
        <label className="grow">
          <span className="sr-only">Message to the agent</span>
          <input
            value={text}
            onChange={(e) => {
              setText(e.target.value);
              key.current.edited();
            }}
            placeholder="Steer the investigation…"
            disabled={busy || !!disabledReason}
          />
        </label>
        <button type="submit" disabled={busy || !text.trim() || !!disabledReason}>
          Send
        </button>
      </div>
      {disabledReason && <p className="muted">{disabledReason}</p>}
      {!disabledReason && note && <p className="muted">{note}</p>}
      {error != null && <ErrorBox error={error} title="Message not sent" />}
    </form>
  );
}

function SearchBox({
  client,
  onEvidence,
  mode,
  run,
  knowledge,
}: {
  client: ResearchClient;
  onEvidence: (e: EvidenceRef) => void;
  mode: Mode;
  run: number;
  knowledge: ServiceStatus | undefined;
}) {
  const [query, setQuery] = useState("");
  const [submitted, setSubmitted] = useState<string | null>(null);
  const unavailable = knowledge?.available === false;
  const q = useQuery({
    queryKey: [mode, run, "search", submitted],
    queryFn: () => client.search({ schema_version: "evidence.v1", query: submitted!, mode: "hybrid" }),
    enabled: submitted !== null && !unavailable,
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
          <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="e.g. droplet extinction diameter" disabled={unavailable} />
        </label>
        <button type="submit" disabled={!query.trim() || unavailable}>
          Search
        </button>
      </form>
      {unavailable && <p className="muted">Search is not configured in this deployment: {knowledge?.detail}.</p>}
      {q.isFetching && <p className="muted">Searching…</p>}
      {q.error && <ErrorBox error={q.error} title={q.error instanceof ApiError && q.error.envelope.code === "capability_unavailable" ? "Search is not configured" : "Search failed"} />}
      {q.data && <SearchResults page={q.data} onEvidence={onEvidence} />}
    </>
  );
}
