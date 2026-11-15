// Questions. A question reads as a conversation: the researcher's messages, the agent's steps
// (collapsed), its answers with citation chips, and a composer at the bottom. Sources, a source's
// original and a step's detail open in the side panel. State comes only from the ordered event log
// (research/eventLog + stream, tested).
import { RichText } from "../shell/RichText";
import { useEffect, useMemo, useRef, useState, type ComponentType, type FormEvent, type ReactNode, type SVGProps } from "react";
import { useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import type { EpisodeStatus, ResearchAnswer, ResearchEpisode } from "../contracts/generated";
import { FIXTURES } from "../research/fixtures";
import { availableActions, DraftKey, TERMINAL_STATUSES } from "../research/episodes";
import { emptyState, type AnswerState, type EpisodeState } from "../research/eventLog";
import type { EpisodeAction, HostDecision } from "../contracts/generated";
import { followEpisode, type StreamNotice } from "../research/stream";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { useEpisodes } from "../shell/queries";
import { go, href } from "../shell/router";
import { IconArrow, IconCheck, IconChevron, IconConditions, IconCopy, IconEvidence, IconExternal, IconLab, IconModels, IconMore, IconRoute, IconSearch, IconSources, IconThread } from "../shell/icons";
import { ErrorNote, Gap, Synthetic, words } from "../ui/common";
import { AnswerView } from "./Answer";
import { groupChats, useChatPrefs } from "../shell/chatPrefs";
import { describeCall, duration, isBookkeeping, learnSourceTitles, routeSummary, seconds, stepState, toTurns, type StepKind, type StepState, type Stop, type Turn } from "./turns";
import "./trail.css";

type Notice = StreamNotice & { at: number };

export function Research({ episodeId, newQuestion, wantNew }: { episodeId: string | null; newQuestion: string | null; wantNew: boolean }) {
  const { ready } = useData();
  if (!ready) return <div className="view muted">Connecting…</div>;
  if (episodeId) return <Episode key={episodeId} episodeId={episodeId} />;
  return <Index wantNew={wantNew} question={newQuestion} />;
}

// ---------------------------------------------------------------- index + new investigation

function Index({ wantNew, question }: { wantNew: boolean; question: string | null }) {
  const { client, researchMode, replay } = useData();
  const qc = useQueryClient();
  const episodes = useEpisodes();
  const key = useRef(new DraftKey());
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const started = useRef(false);

  const create = async (goal: string) => {
    setBusy(true);
    setError(null);
    try {
      const ep = await client.createEpisode(goal, key.current.current);
      key.current.succeeded();
      // Creating an episode starts nothing: the agent turn begins with the first message.
      // If this fails the investigation still opens and the composer can send it.
      if (ep.status === "ready" && ep.last_seq === 0) {
        try {
          await client.postMessage(ep.episode_id, goal, crypto.randomUUID());
        } catch {
          /* the thread shows no events and the composer is available */
        }
      }
      qc.setQueryData<ResearchEpisode[]>([researchMode, replay, "episodes"], (xs) => (xs?.some((x) => x.episode_id === ep.episode_id) ? xs : [ep, ...(xs ?? [])]));
      go("research", ep.episode_id);
    } catch (e) {
      setError(e);
    } finally {
      setBusy(false);
    }
  };
  useEffect(() => {
    if (question && !started.current && researchMode !== "mock") {
      started.current = true;
      void create(question);
    }
  }, [question, researchMode]); // eslint-disable-line react-hooks/exhaustive-deps

  // New questions are asked on Home; this page is the history.
  useEffect(() => {
    if (wantNew && !question) go("home");
  }, [wantNew, question]);

  const [prefs] = useChatPrefs();
  const [q, setQ] = useState("");
  const [filter, setFilter] = useState<QFilter>("all");
  const groups = useMemo(() => {
    const needle = q.trim().toLowerCase();
    const shown = (episodes.data ?? []).filter((e) => QFILTER[filter].has(e.status) && (!needle || e.goal.toLowerCase().includes(needle)));
    return groupChats(shown, { ...prefs, hidden: [] });
  }, [episodes.data, prefs, q, filter]);
  const counts = useMemo(() => {
    const all = episodes.data ?? [];
    return Object.fromEntries((Object.keys(QFILTER) as QFilter[]).map((k) => [k, all.filter((e) => QFILTER[k].has(e.status)).length])) as Record<QFilter, number>;
  }, [episodes.data]);

  return (
    <div className="view qhist">
      <div className="view-head">
        <div>
          <h1>Questions</h1>
          <p className="muted qhist-sub">Every question you asked, by day.</p>
        </div>
      </div>
      {question && (busy || error == null) && <p className="muted">Starting “{question}”…</p>}
      {error != null && <ErrorNote error={error} title="The question was not sent" />}
      <label className="qhist-search">
        <IconSearch aria-hidden="true" />
        <span className="sr-only">Search questions</span>
        <input type="search" placeholder="Search your questions" value={q} onChange={(e) => setQ(e.target.value)} />
      </label>
      <div className="qhist-chips" role="group" aria-label="Filter">
        {(Object.keys(QFILTER) as QFilter[])
          .filter((k) => k === "all" || counts[k] > 0)
          .map((k) => (
            <button key={k} className="qhist-chip" aria-pressed={filter === k} onClick={() => setFilter(k)}>
              {QFILTER_LABEL[k]} <span className="num muted">{counts[k]}</span>
            </button>
          ))}
      </div>
      {episodes.isLoading && <p className="muted">Loading your questions…</p>}
      {episodes.data && !groups.length && <p className="muted">{episodes.data.length ? "No question matches." : "No questions yet. Ask one on Home."}</p>}
      {groups.map((g) => (
        <section key={g.label} className="qhist-group" aria-label={g.label}>
          <h2>{g.label}</h2>
          <ul>
            {g.items.map((e) => (
              <li key={e.episode_id}>
                <a className="qhist-row" href={href("research", e.episode_id)}>
                  <span className="qhist-icon" aria-hidden="true">
                    <IconThread />
                  </span>
                  <span className="qhist-main">
                    <b>{e.goal}</b>
                    <span className="muted">{STATUS_LABEL[e.status] ?? words(e.status)}</span>
                  </span>
                  <span className="qhist-when muted num">{when(e.updated_at || e.created_at)}</span>
                </a>
              </li>
            ))}
          </ul>
        </section>
      ))}
      {episodes.error && <ErrorNote error={episodes.error} title="Questions could not be loaded" />}
    </div>
  );
}

type QFilter = "all" | "answered" | "running" | "stopped";
const RUNNING: EpisodeStatus[] = ["ready", "running", "waiting_tools", "waiting_approval", "waiting_user", "paused"];
const STOPPED: EpisodeStatus[] = ["failed", "cancelled"];
const QFILTER: Record<QFilter, Set<string>> = {
  all: new Set<string>([...RUNNING, "completed", ...STOPPED]),
  answered: new Set(["completed"]),
  running: new Set<string>(RUNNING),
  stopped: new Set<string>(STOPPED),
};
const QFILTER_LABEL: Record<QFilter, string> = { all: "All", answered: "Answered", running: "In progress", stopped: "Stopped" };
const STATUS_LABEL: Record<string, string> = {
  completed: "Answered",
  running: "Working…",
  ready: "Ready",
  waiting_tools: "Working…",
  waiting_approval: "Needs your approval",
  waiting_user: "Waiting for you",
  paused: "Paused",
  failed: "Failed",
  cancelled: "Cancelled",
};
/** "15 Nov, 10:28" (with the year when it is not this year). */
const when = (iso: string) => {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  const sameYear = d.getFullYear() === new Date().getFullYear();
  return d.toLocaleString("en-GB", { day: "numeric", month: "short", ...(sameYear ? {} : { year: "numeric" }), hour: "2-digit", minute: "2-digit" });
};

// ---------------------------------------------------------------- one investigation

const ACTION_LABEL: Record<EpisodeAction, string> = { pause: "Pause", resume: "Resume", cancel: "Cancel" };

function Episode({ episodeId }: { episodeId: string }) {
  const { client, researchMode, replay, caps, faults, setFaults, bumpReplay } = useData();
  const panel = usePanel();
  const qc = useQueryClient();
  const [state, setState] = useState<EpisodeState>(() => emptyState(episodeId));
  const stateRef = useRef(state);
  stateRef.current = state;
  const [notices, setNotices] = useState<Notice[]>([]);
  const [following, setFollowing] = useState(false);
  const [follow, setFollow] = useState(0);
  const [actionError, setActionError] = useState<unknown>(null);
  const lastKey = useRef("");

  useEffect(() => {
    const k = `${researchMode}|${replay}|${episodeId}`;
    const fresh = k !== lastKey.current;
    lastKey.current = k;
    const initial = fresh ? emptyState(episodeId) : stateRef.current;
    const ac = new AbortController();
    if (fresh) {
      setState(initial);
      setNotices([]);
    }
    setFollowing(true);
    followEpisode(client, initial, {
      signal: ac.signal,
      onState: (s) => !ac.signal.aborted && setState(s),
      onNotice: (n) => !ac.signal.aborted && setNotices((xs) => [...xs.slice(-49), { ...n, at: Date.now() }]),
    }).finally(() => !ac.signal.aborted && setFollowing(false));
    return () => ac.abort();
  }, [client, episodeId, follow]); // eslint-disable-line react-hooks/exhaustive-deps

  const episode = useQuery({ queryKey: [researchMode, replay, "episode", episodeId], queryFn: ({ signal }) => client.getEpisode(episodeId, signal), retry: false });
  const status = state.status ?? episode.data?.status ?? null;
  const runtime = caps.data?.agent_runtime;
  const turns = useMemo(() => {
    learnSourceTitles(state);
    return toTurns(state);
  }, [state]);
  const answerQueries = useQueries({
    queries: state.answers
      .filter((a) => a.artifact.contract === "ResearchAnswer")
      .map((a) => ({ queryKey: [researchMode, replay, "answer", a.artifact.artifact_id], queryFn: () => client.getArtifactJson(a.artifact.artifact_id) as Promise<ResearchAnswer>, retry: false })),
  });
  const pending = Object.values(state.approvals).filter((a) => !a.decision);
  const canWrite = researchMode !== "mock";

  const putEpisode = (ep: ResearchEpisode) => {
    qc.setQueryData([researchMode, replay, "episode", ep.episode_id], ep);
    qc.setQueryData<ResearchEpisode[]>([researchMode, replay, "episodes"], (xs) => (xs ? xs.map((x) => (x.episode_id === ep.episode_id ? ep : x)) : xs));
  };
  const act = async (a: EpisodeAction) => {
    setActionError(null);
    try {
      putEpisode(await client.act(episodeId, a));
      setFollow((f) => f + 1);
    } catch (e) {
      setActionError(e);
    }
  };
  const decide = async (approvalId: string, d: HostDecision) => {
    setActionError(null);
    try {
      await client.resolveApproval(episodeId, approvalId, d);
      setFollow((f) => f + 1);
    } catch (e) {
      setActionError(e);
    }
  };

  if (episode.error && state.cursor === 0)
    return (
      <div className="view" style={{ maxWidth: 720 }}>
        <ErrorNote error={episode.error} title="This question is not on this server" />
        <div className="toolbar">
          <a className="btn" href={href("research")}>
            All questions
          </a>
        </div>
      </div>
    );

  const live = following && !(status && TERMINAL_STATUSES.includes(status));
  const working = live && status !== "waiting_user" && status !== "waiting_approval";
  const goal = episode.data?.goal ?? turns[0]?.user?.text ?? "";
  const answersLoaded = answerQueries.flatMap((q) => (q.data ? [q.data] : []));
  const nSources = new Set(answersLoaded.flatMap((a) => (a.evidence ?? []).map((e) => e.source_id))).size;

  return (
    <div className="chat">
      <header className="topbar chat-bar">
        <nav className="crumbs" aria-label="Breadcrumb">
          <a href={href("research")}>Questions</a>
          <IconChevron width={12} height={12} />
          <span className="here" title={goal}>
            {goal || "…"}
          </span>
        </nav>
        <span className="spacer" />
        {nSources > 0 && (
          <button className={`btn btn-quiet${panel.current?.kind === "sources" ? " on" : ""}`} onClick={() => (panel.current?.kind === "sources" ? panel.close() : panel.open({ kind: "sources", answers: answersLoaded }))}>
            <IconSources /> Sources <span className="count">{nSources}</span>
          </button>
        )}
        <EpisodeMenu
          episode={episode.data ?? null}
          status={status}
          canWrite={canWrite}
          act={act}
          details={
            <>
              <dl className="kv">
                <dt>Status</dt>
                <dd>{status ? words(status) : "connecting"}</dd>
                {episode.data && (
                  <>
                    <dt>Created</dt>
                    <dd className="num">{episode.data.created_at.slice(0, 16).replace("T", " ")}</dd>
                    <dt>Corpus</dt>
                    <dd>
                      <code>{episode.data.pins.corpus_snapshot ?? "not pinned"}</code>
                    </dd>
                    <dt>Runtime</dt>
                    <dd>{episode.data.runtime ? words(episode.data.runtime.runtime) : "not attached"}</dd>
                    <dt>Harness</dt>
                    <dd>{episode.data.pins.harness ? `${episode.data.pins.harness.label} · ${episode.data.pins.harness.model}` : "not pinned"}</dd>
                    {episode.data.origin === "synthetic_fixture" && (
                      <>
                        <dt>Origin</dt>
                        <dd>
                          <Synthetic />
                        </dd>
                      </>
                    )}
                  </>
                )}
                <dt>Events</dt>
                <dd className="num">{state.cursor}</dd>
              </dl>
              {researchMode === "mock" && (
                <div className="toolbar small" style={{ marginTop: 10 }}>
                  <label>
                    Drop after{" "}
                    <select className="field" value={faults.dropAfterSeq ?? ""} onChange={(e) => setFaults((f) => ({ ...f, dropAfterSeq: e.target.value === "" ? null : Number(e.target.value) }))}>
                      <option value="">never</option>
                      {FIXTURES.events.slice(0, -1).map((e) => (
                        <option key={e.seq} value={e.seq}>
                          {e.seq}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
                    <input type="checkbox" checked={faults.goneOnResume} onChange={(e) => setFaults((f) => ({ ...f, goneOnResume: e.target.checked }))} /> 410 on resume
                  </label>
                  <button className="btn btn-s" onClick={bumpReplay}>
                    Replay
                  </button>
                </div>
              )}
              {notices.some((n) => n.kind === "error" || n.kind === "gap" || n.kind === "gave_up") && (
                <ul className="small" style={{ display: "grid", gap: 2, marginTop: 10 }}>
                  {notices
                    .filter((n) => n.kind === "error" || n.kind === "gap" || n.kind === "gave_up")
                    .map((n, k) => (
                      <li key={k} style={{ listStyle: "none" }}>
                        {noticeText(n)}
                      </li>
                    ))}
                </ul>
              )}
            </>
          }
        />
      </header>

      <div className="chat-col">
        {actionError != null && <ErrorNote error={actionError} title="The server refused the action" />}
        {state.missing.map((m) => (
          <Gap key={m.from} title={`Events ${m.from}–${m.to} were not received`}>
            The state continues from the snapshot.
          </Gap>
        ))}
        {turns.map((t, i) => (
          <TurnView key={i} turn={t} working={working && i === turns.length - 1} />
        ))}
        {turns.length === 0 && goal && <div className="msg-user">{goal}</div>}
        {pending.map((a) => (
          <div key={a.approval_id} className="note note-caution" role="alert">
            <b>Approval needed: {a.action}</b>
            <div className="toolbar">
              <button className="btn btn-s btn-primary" onClick={() => decide(a.approval_id, "approved")} disabled={!runtime?.available}>
                Approve
              </button>
              <button className="btn btn-s" onClick={() => decide(a.approval_id, "denied")} disabled={!runtime?.available}>
                Deny
              </button>
            </div>
          </div>
        ))}
        {working && turns.length === 0 && (
          <div className="working" role="status">
            <span className="dot dot-run" aria-hidden="true" /> Working
          </div>
        )}
        {!working && turns.length === 0 && runtime?.available === false && researchMode !== "mock" && <Gap title="No agent is configured on this server">The question is saved; no answer will come until an agent runtime is attached.</Gap>}
        {status && TERMINAL_STATUSES.includes(status) && state.answers.length === 0 && turns.length > 0 && <Gap title="No answer was produced">The question ended ({words(status)}).</Gap>}
      </div>

      <Composer
        key={episodeId}
        disabledReason={!canWrite ? "Replay only" : status === "cancelled" || status === "failed" ? `This question is ${words(status)}` : null}
        send={async (text, k) => {
          await client.postMessage(episodeId, text, k);
          setFollow((f) => f + 1);
        }}
      />
    </div>
  );
}

/** The "…" menu of a question: pause, resume or cancel when the episode allows it, copy its link, details. */
function EpisodeMenu({ episode, status, canWrite, act, details }: { episode: ResearchEpisode | null; status: EpisodeStatus | null; canWrite: boolean; act: (a: EpisodeAction) => void; details: ReactNode }) {
  const [open, setOpen] = useState<"menu" | "details" | null>(null);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!open) return;
    const on = (e: MouseEvent) => !ref.current?.contains(e.target as Node) && setOpen(null);
    const key = (e: KeyboardEvent) => e.key === "Escape" && setOpen(null);
    window.addEventListener("mousedown", on);
    window.addEventListener("keydown", key);
    return () => {
      window.removeEventListener("mousedown", on);
      window.removeEventListener("keydown", key);
    };
  }, [open]);
  const actions = canWrite ? (["pause", "resume", "cancel"] as EpisodeAction[]).filter((a) => availableActions(status).includes(a)) : [];
  return (
    <div className="menu-wrap" ref={ref}>
      <button className="btn btn-quiet btn-icon" aria-label="More" aria-haspopup="menu" aria-expanded={open === "menu"} onClick={() => setOpen((o) => (o ? null : "menu"))}>
        <IconMore />
      </button>
      {open === "menu" && (
        <div className="menu" role="menu">
          {actions.map((a) => (
            <button
              key={a}
              role="menuitem"
              onClick={() => {
                setOpen(null);
                act(a);
              }}
            >
              {ACTION_LABEL[a]}
            </button>
          ))}
          <button
            role="menuitem"
            onClick={() => {
              void navigator.clipboard?.writeText(window.location.href);
              setOpen(null);
            }}
          >
            <IconCopy /> Copy link
          </button>
          <button role="menuitem" onClick={() => setOpen("details")} disabled={!episode}>
            Details
          </button>
        </div>
      )}
      {open === "details" && (
        <div className="menu menu-details" role="dialog" aria-label="Question details">
          {details}
        </div>
      )}
    </div>
  );
}

function TurnView({ turn, working }: { turn: Turn; working: boolean }) {
  const { synthetic } = useData();
  const final = turn.answers[turn.answers.length - 1] ?? null;
  const earlier = turn.answers.slice(0, -1);
  const spoken = (s: Stop) => s.kind === "said" && s.message.text.trim() !== "";
  // Without a stored answer, the agent's last words are the reply; everything said before (and,
  // with an answer, everything said at all) is narration and lives in the trail where it was said.
  const last = [...turn.stops].reverse().find((s) => s.kind !== "said" || spoken(s));
  const body = !final && last?.kind === "said" ? last.message : null;
  const trail = turn.stops.filter((s) => (s.kind === "said" ? spoken(s) && s.message !== body : true));
  const lastSaid = [...turn.stops].reverse().find((s): s is Extract<Stop, { kind: "said" }> => spoken(s));
  const showTrail = working || trail.some((s) => s.kind !== "answer" && !isBookkeeping(s));
  return (
    <div className="turn">
      {turn.user && <div className="msg-user">{turn.user.text}</div>}
      {showTrail && <RouteSummary stops={trail} turn={turn} working={working} />}
      {body && (
        <div className="msg-agent">
          <RichText text={body.text} />
          {!body.complete && <span className="muted"> …</span>}
        </div>
      )}
      {earlier.length > 0 && (
        <details className="trail-earlier">
          <summary className="muted">
            {earlier.length === 1 ? "An earlier answer in this turn" : `${earlier.length} earlier answers in this turn`}
            <IconChevron className="chev" aria-hidden="true" />
          </summary>
          {earlier.map((a) => (
            <div key={a.answer_id} id={`a-${a.answer_id}`}>
              <AnswerLoader a={a} pageSynthetic={synthetic} />
            </div>
          ))}
        </details>
      )}
      {final && (
        <div id={`a-${final.answer_id}`}>
          <AnswerLoader a={final} pageSynthetic={synthetic} fallback={lastSaid?.message.text ?? null} />
        </div>
      )}
    </div>
  );
}

function useAnswer(a: AnswerState) {
  const { client, researchMode, replay } = useData();
  return useQuery({
    queryKey: [researchMode, replay, "answer", a.artifact.artifact_id],
    queryFn: () => client.getArtifactJson(a.artifact.artifact_id) as Promise<ResearchAnswer>,
    enabled: a.artifact.contract === "ResearchAnswer",
    retry: false,
  });
}

/** The stored answer. When it cannot be shown, the agent's own last message stands in, said as such. */
function AnswerLoader({ a, pageSynthetic, fallback = null }: { a: AnswerState; pageSynthetic: boolean; fallback?: string | null }) {
  const { client } = useData();
  const q = useAnswer(a);
  const said = fallback ? (
    <div className="msg-agent">
      <p className="muted small">The agent's last message, unchecked:</p>
      <RichText text={fallback} />
    </div>
  ) : null;
  if (a.artifact.contract !== "ResearchAnswer")
    return (
      <>
        <Gap title="The answer is not rendered">Its artifact has contract {a.artifact.contract ?? "none"}.</Gap>
        {said}
      </>
    );
  if (q.isPending) return <p className="muted">Loading the answer…</p>;
  if (q.error)
    return (
      <>
        <ErrorNote error={q.error} title="The answer could not be loaded" />
        {said}
      </>
    );
  return <AnswerView client={client} answer={q.data} pageSynthetic={pageSynthetic} />;
}

// ---------------------------------------------------------------- the route, as a notebook trail

const KIND_ICON: Record<StepKind, ComponentType<SVGProps<SVGSVGElement>>> = {
  search: IconSearch,
  read: IconSources,
  data: IconConditions,
  model: IconModels,
  compute: IconLab,
  web: IconExternal,
  note: IconEvidence,
  other: IconRoute,
};

const STATE_TEXT: Record<StepState, string | null> = { ok: null, running: "working", failed: "failed", refused: "refused", cancelled: "cancelled", waiting: "waiting for you" };

const ANSWER_TEXT: Record<string, string> = { complete: "Answered", partial: "Gave a partial answer", insufficient_evidence: "Answered: insufficient evidence", failed: "The answer failed" };

/** A tone for the step's state, used by its marker and the summary's count of problems. */
function stopTone(s: Stop): "ok" | "run" | "bad" | "warn" | "quiet" {
  if (s.kind === "note" || s.kind === "said") return "quiet";
  if (s.kind === "answer") return s.answer.status === "complete" ? "ok" : s.answer.status === "failed" ? "bad" : "warn";
  const st = stepState(s);
  if (st === "running") return "run";
  if (st === "failed" || st === "refused" || st === "cancelled") return "bad";
  if (st === "waiting") return "warn";
  if (s.kind === "tool") {
    const sci = s.call.result?.scientific_status;
    if (sci === "inconclusive" || sci === "insufficient_evidence" || sci === "out_of_domain") return "warn";
  }
  return "ok";
}

/** The notebook sentence of a step, a few words on what came back, and its icon kind. */
function stopText(s: Stop): { text: string; detail: string | null; kind: StepKind } {
  switch (s.kind) {
    case "tool": {
      const d = describeCall(s.call);
      const sci = s.call.result?.scientific_status;
      const verdict = sci === "inconclusive" || sci === "insufficient_evidence" || sci === "out_of_domain" ? words(sci) : null;
      return { text: d.text, detail: [d.detail, verdict].filter(Boolean).join(" · ") || null, kind: d.kind };
    }
    case "job": {
      const fc = s.jobKind === "models_forecast";
      const what = fc ? "The forecast" : `The ${words(s.jobKind)} job`;
      const st = stepState(s);
      return { text: st === "ok" ? `${what} finished` : st === "running" ? `${what} is running` : `${what} ${words(s.status)}`, detail: fc ? "model output" : null, kind: fc ? "model" : "compute" };
    }
    case "approval":
      return { text: `Asked you to approve: ${s.action}`, detail: null, kind: "other" };
    case "answer":
      return { text: ANSWER_TEXT[s.answer.status] ?? `Answered (${words(s.answer.status)})`, detail: null, kind: "other" };
    case "note":
      return { text: s.text, detail: null, kind: "note" };
    case "said":
      return { text: s.message.text, detail: null, kind: "note" };
  }
}

/** A step's own duration, when it took a second or more; shorter ones say nothing. */
function stepDuration(s: Stop): string | null {
  if (s.kind !== "tool" && s.kind !== "job") return null;
  if (!s.end || Date.parse(s.end) - Date.parse(s.at) < 1000) return null;
  return duration(s.at, s.end);
}

function useNow(on: boolean): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!on) return;
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, [on]);
  return now;
}

/** The trail as a list: one sentence per step; narration and notes as muted asides where they were said. */
function Stops({ stops, t0 }: { stops: Stop[]; t0: string | null }) {
  const panel = usePanel();
  let n = 0;
  return (
    <ol className="trail-list">
      {stops.map((s) => {
        if (isBookkeeping(s)) return null;
        if (s.kind === "note" || s.kind === "said") return <Aside key={`${s.kind}-${s.seq}`} stop={s} />;
        const { text, detail, kind } = stopText(s);
        const tone = stopTone(s);
        const state = s.kind === "answer" ? null : STATE_TEXT[stepState(s)];
        const Icon = s.kind === "answer" ? IconCheck : KIND_ICON[kind];
        const index = s.kind === "tool" ? ++n : 0;
        const on = s.kind === "tool" && panel.current?.kind === "step" && panel.current.call.call_id === s.call.call_id;
        const right = s.kind === "answer" ? duration(t0, s.at) : stepDuration(s);
        const inner = (
          <>
            <span className={`trail-icon tone-${tone}`} aria-hidden="true">
              {tone === "run" ? <span className="trail-spin" /> : <Icon />}
            </span>
            <span className="trail-text">
              <span className="trail-sentence">{text}</span>
              {detail && <span className="trail-detail"> · {detail}</span>}
              {state && (
                <>
                  {" "}
                  <span className={`trail-state tone-${tone}`}>{state}</span>
                </>
              )}
            </span>
            {right && <span className="trail-dur">{right}</span>}
          </>
        );
        const cls = `trail-step${s.kind === "answer" ? " trail-end" : ""}${on ? " on" : ""}`;
        return (
          <li key={`${s.kind}-${s.seq}`}>
            {s.kind === "tool" || s.kind === "answer" ? (
              <button
                className={cls}
                title={s.kind === "tool" ? text : undefined}
                onClick={() => {
                  if (s.kind === "tool") panel.open({ kind: "step", call: s.call, index });
                  else document.getElementById(`a-${s.answer.answer_id}`)?.scrollIntoView({ behavior: "smooth", block: "start" });
                }}
              >
                {inner}
              </button>
            ) : (
              <div className={cls}>{inner}</div>
            )}
          </li>
        );
      })}
    </ol>
  );
}

/** What the agent said between steps, or a note it left on its route: muted, folded to a few lines. */
function Aside({ stop }: { stop: Extract<Stop, { kind: "note" | "said" }> }) {
  const [open, setOpen] = useState(false);
  const text = stop.kind === "note" ? stop.text : stop.message.text;
  const long = text.length > 240 || text.split("\n").length > 3;
  return (
    <li className={`trail-aside${open ? " open" : ""}`}>
      <span className="trail-icon tone-quiet" aria-hidden="true">
        <IconEvidence />
      </span>
      <div className="trail-aside-body">
        <span className="sr-only">{stop.kind === "note" ? "The agent's note: " : "The agent said: "}</span>
        <div className="trail-aside-text">
          <RichText text={text} />
        </div>
        {long && (
          <button className="trail-more" onClick={() => setOpen((o) => !o)} aria-expanded={open}>
            {open ? "Show less" : "Show all"}
          </button>
        )}
      </div>
    </li>
  );
}

/** The current step, live: its sentence, a quiet spinner and how long it has run. */
function LiveRow({ stop }: { stop: Stop | null }) {
  const now = useNow(stop != null);
  const from = stop?.at ?? null;
  const secs = from ? Math.max(0, Math.round((now - Date.parse(from)) / 1000)) : null;
  return (
    <div className="trail-live" role="status" aria-live="polite">
      <span className="trail-icon tone-run" aria-hidden="true">
        <span className="trail-spin" />
      </span>
      <span className="trail-text">
        <span className="trail-sentence">{stop ? stopText(stop).text : "Working"}</span>
        <span className="trail-dots" aria-hidden="true">…</span>
      </span>
      {stop && secs != null && secs >= 1 && <span className="trail-dur">{seconds(secs)}</span>}
    </div>
  );
}

/** One line when closed ("Searched 3 times, read 2 sources · 32 s"); the trail when open. While the
 * agent works, the current step stays visible under the line. */
function RouteSummary({ stops, turn, working }: { stops: Stop[]; turn: Turn; working: boolean }) {
  const [open, setOpen] = useState(false);
  const visible = stops.filter((s) => !isBookkeeping(s));
  const lastStep = [...visible].reverse().find((s) => s.kind === "tool" || s.kind === "job");
  const current = working && lastStep && stepState(lastStep) === "running" ? lastStep : null;
  const listed = current ? visible.filter((s) => s !== current) : visible;
  const problems = visible.filter((s) => stopTone(s) === "bad").length;
  const asides = visible.filter((s) => s.kind === "note" || s.kind === "said").length;
  const done = listed.filter((s) => s.kind === "tool").length;
  const summary = working
    ? done
      ? `${done} step${done === 1 ? "" : "s"} so far`
      : null
    : routeSummary(stops) ?? (asides ? `${asides} note${asides === 1 ? "" : "s"} on the way` : "No tool steps");
  const total = working ? null : duration(turn.startedAt, turn.endedAt);
  return (
    <section className={`trail${open ? " open" : ""}${working ? " live" : ""}`} aria-label="What the agent did">
      {summary && (
        <button className="trail-head" aria-expanded={open} onClick={() => setOpen((o) => !o)}>
          <span className="trail-sum">
            {summary}
            {total && <span className="trail-total"> · {total}</span>}
          </span>
          {problems > 0 && <span className="trail-state tone-bad">{problems === 1 ? "1 step did not run" : `${problems} steps did not run`}</span>}
          <IconChevron className="chev" aria-hidden="true" />
        </button>
      )}
      {open && <Stops stops={listed} t0={turn.startedAt} />}
      {working && <LiveRow stop={current} />}
    </section>
  );
}

// ---------------------------------------------------------------- composer

function Composer({ send, disabledReason }: { send: (text: string, key: string) => Promise<void>; disabledReason: string | null }) {
  const [text, setText] = useState("");
  const key = useRef(new DraftKey());
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const area = useRef<HTMLTextAreaElement>(null);
  // A next step drafts a follow-up here; the researcher edits and sends it.
  useEffect(() => {
    const on = (e: Event) => {
      setText((e as CustomEvent<string>).detail);
      key.current.edited();
      area.current?.scrollIntoView({ behavior: "smooth", block: "end" });
      requestAnimationFrame(() => area.current?.focus());
    };
    window.addEventListener("ignis:followup", on);
    return () => window.removeEventListener("ignis:followup", on);
  }, []);
  const submit = async (e?: FormEvent) => {
    e?.preventDefault();
    if (!text.trim() || disabledReason || busy) return;
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
    <div className="composer">
      {error != null && <ErrorNote error={error} title="The message was not sent" />}
      <form onSubmit={submit}>
        <label htmlFor="steer" className="sr-only">
          Follow-up
        </label>
        <textarea
          id="steer"
          ref={area}
          rows={text.length > 90 ? 3 : 1}
          value={text}
          onChange={(e) => {
            setText(e.target.value);
            key.current.edited();
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              void submit();
            }
          }}
          placeholder={disabledReason ?? "Ask a follow-up…"}
          disabled={!!disabledReason || busy}
        />
        <button className="btn btn-primary btn-icon" type="submit" disabled={!!disabledReason || busy || !text.trim()} aria-label="Send">
          <IconArrow />
        </button>
      </form>
    </div>
  );
}

function noticeText(n: Notice): string {
  switch (n.kind) {
    case "connected":
      return `Connected after event ${n.after}.`;
    case "ended":
      return `Stream closed at event ${n.cursor}.`;
    case "duplicate":
      return `Event ${n.seq} arrived again; ignored.`;
    case "gap":
      return `Gap: expected ${n.expected}, got ${n.got}; resuming.`;
    case "foreign_event":
      return `Event of another investigation ignored.`;
    case "resync":
      return `HTTP 410: reloaded; events ${n.from}–${n.to} not received.`;
    case "restart":
      return `HTTP 410: replaying from the start.`;
    case "error":
      return `Error ${n.error.code}${n.http_status ? ` (HTTP ${n.http_status})` : ""}: ${n.error.message}`;
    case "gave_up":
      return `Stopped: ${n.reason}.`;
  }
}
