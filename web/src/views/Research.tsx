// Questions. A question reads as a conversation: the researcher's messages, the agent's steps
// (collapsed), its answers with citation chips, and a composer at the bottom. Sources, a source's
// original and a step's detail open in the side panel. State comes only from the ordered event log
// (research/eventLog + stream, tested).
import { useEffect, useMemo, useRef, useState, type FormEvent, type ReactNode } from "react";
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
import { IconArrow, IconChevron, IconCopy, IconMore, IconSources } from "../shell/icons";
import { ErrorNote, Gap, Synthetic, words } from "../ui/common";
import { AnswerView } from "./Answer";
import { AskBox } from "./AskBox";
import { elapsed, toTurns, TOOL_LABEL, type Stop, type Turn } from "./turns";

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

  return (
    <div className="view" style={{ maxWidth: 920 }}>
      <div className="view-head">
        <h1>{wantNew ? "New question" : "Questions"}</h1>
      </div>
      <AskBox onAsk={create} busy={busy} initial={question ?? ""} autoFocus={wantNew} />
      {error != null && <ErrorNote error={error} title="The question was not sent" />}
      <div className="tbl-wrap">
        <table className="tbl">
          <thead>
            <tr>
              <th>Question</th>
              <th>Status</th>
              <th className="r">Events</th>
              <th>Updated</th>
            </tr>
          </thead>
          <tbody>
            {episodes.data?.map((e) => (
              <tr key={e.episode_id} className="clickable" onClick={() => go("research", e.episode_id)}>
                <td className="wrap">
                  <a href={href("research", e.episode_id)} onClick={(ev) => ev.stopPropagation()}>
                    {e.goal}
                  </a>
                </td>
                <td>{words(e.status)}</td>
                <td className="r num">{e.last_seq}</td>
                <td className="dim num">{e.updated_at.slice(0, 16).replace("T", " ")}</td>
              </tr>
            ))}
            {episodes.data?.length === 0 && (
              <tr>
                <td colSpan={4} className="dim">
                  No questions yet.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
      {episodes.error && <ErrorNote error={episodes.error} title="Questions could not be loaded" />}
    </div>
  );
}

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
  const turns = useMemo(() => toTurns(state), [state]);
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
          <TurnView key={i} turn={t} live={live && i === turns.length - 1} />
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
        {working && (
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

function TurnView({ turn, live }: { turn: Turn; live: boolean }) {
  const { synthetic } = useData();
  const replies = turn.replies.filter((m) => m.text.trim());
  return (
    <div className="turn">
      {turn.user && <div className="msg-user">{turn.user.text}</div>}
      {turn.stops.length > 0 && <RouteSummary stops={turn.stops} turn={turn} open={live} />}
      {replies.map((m) => (
        <p key={m.message_id} className="msg-agent">
          {m.text}
          {!m.complete && <span className="muted"> …</span>}
        </p>
      ))}
      {turn.answers.map((a) => (
        <div key={a.answer_id} id={`a-${a.answer_id}`}>
          <AnswerLoader a={a} pageSynthetic={synthetic} />
        </div>
      ))}
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

function AnswerLoader({ a, pageSynthetic }: { a: AnswerState; pageSynthetic: boolean }) {
  const { client } = useData();
  const q = useAnswer(a);
  if (a.artifact.contract !== "ResearchAnswer") return <Gap title="The answer is not rendered">Its artifact has contract {a.artifact.contract ?? "none"}.</Gap>;
  if (q.isPending) return <p className="muted">Loading the answer…</p>;
  if (q.error) return <ErrorNote error={q.error} title="The answer could not be loaded" />;
  return <AnswerView client={client} answer={q.data} pageSynthetic={pageSynthetic} />;
}

// ---------------------------------------------------------------- the route

function stopTone(s: Stop): string {
  if (s.kind === "answer") return "answer";
  if (s.kind === "note") return "note";
  if (s.kind === "job") return s.status === "succeeded" ? "ok" : s.status === "failed" || s.status === "cancelled" ? "bad" : "run";
  if (s.kind === "approval") return s.decision === "approved" ? "ok" : s.decision ? "bad" : "warn";
  const r = s.call.result;
  if (!r) return "run";
  if (r.execution_status === "failed" || r.execution_status === "refused" || r.execution_status === "cancelled") return "bad";
  if (r.execution_status === "accepted") return "run";
  return r.scientific_status === "conclusive" || r.scientific_status === "not_assessed" ? "ok" : "warn";
}

function stopText(s: Stop): { title: string; sub: string } {
  switch (s.kind) {
    case "tool": {
      const r = s.call.result;
      const title = TOOL_LABEL[s.call.tool] ?? words(s.call.tool);
      if (!r) return { title, sub: "running" };
      const sci = r.scientific_status === "not_assessed" ? "" : ` · ${words(r.scientific_status)}`;
      const q = argQuery(s.call);
      return { title, sub: `${words(r.execution_status)}${sci}${r.result?.partial ? " · preview" : ""}${q ? ` · “${q}”` : ""}` };
    }
    case "job":
      return { title: `Job: ${words(s.jobKind)}`, sub: words(s.status) };
    case "approval":
      return { title: `Approval: ${s.action}`, sub: s.decision ? words(s.decision) : "waiting" };
    case "answer":
      return { title: "Answered", sub: words(s.answer.status) };
    case "note":
      return { title: "Agent's note (its own explanation)", sub: s.text };
  }
}

/** The search query a call was given, when its recorded arguments carry one (K1.1). */
function argQuery(call: Extract<Stop, { kind: "tool" }>["call"]): string | null {
  const v = call.arguments?.value.value;
  const q = v && typeof v === "object" && !Array.isArray(v) ? (v as Record<string, unknown>).query : undefined;
  return typeof q === "string" && q ? (q.length > 60 ? `${q.slice(0, 57)}…` : q) : null;
}

function Stops({ stops, t0 }: { stops: Stop[]; t0: string | null }) {
  const panel = usePanel();
  return (
    <div className="stops" role="list" style={{ paddingTop: 6 }}>
      {stops.map((s, i) => {
        const { title, sub } = stopText(s);
        const on = s.kind === "tool" && panel.current?.kind === "step" && panel.current.call.call_id === s.call.call_id;
        if (s.kind === "note")
          return (
            <div key={`note-${s.seq}`} role="listitem" className="stop stop-note">
              <span className={`node ${stopTone(s)}`} aria-hidden="true" />
              <span className="what">
                {title}
                <span>{sub}</span>
              </span>
              <span className="when">+{elapsed(t0, s.at) ?? "0:00"}</span>
            </div>
          );
        return (
          <button
            key={`${s.kind}-${s.seq}`}
            role="listitem"
            className={`stop${on ? " on" : ""}`}
            onClick={() => {
              if (s.kind === "tool") panel.open({ kind: "step", call: s.call, index: i + 1 });
              else if (s.kind === "answer") document.getElementById(`a-${s.answer.answer_id}`)?.scrollIntoView({ behavior: "smooth" });
            }}
          >
            <span className={`node ${stopTone(s)}`} aria-hidden="true" />
            <span className="what">
              {title}
              <span>{sub}</span>
            </span>
            <span className="when">+{elapsed(t0, s.at) ?? "0:00"}</span>
          </button>
        );
      })}
    </div>
  );
}

function RouteSummary({ stops, turn, open }: { stops: Stop[]; turn: Turn; open: boolean }) {
  const tools = stops.filter((s) => s.kind === "tool").length;
  const issues = stops.filter((s) => ["bad", "warn"].includes(stopTone(s))).length;
  return (
    <details className="route" open={open || undefined}>
      <summary>
        <b>{tools} step{tools === 1 ? "" : "s"}</b>
        <span>
          {issues > 0 && ` · ${issues} to check`}
          {elapsed(turn.startedAt, turn.endedAt) && ` · ${elapsed(turn.startedAt, turn.endedAt)}`}
        </span>
        <IconChevron className="chev" />
      </summary>
      <Stops stops={stops} t0={turn.startedAt} />
    </details>
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
