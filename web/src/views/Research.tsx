// Investigations. An investigation is an object page (Harvey): title, tabs (Thread, Evidence,
// Route) and a properties column; a selected claim, source or step replaces the properties in the
// side panel. State comes only from the ordered event log (research/eventLog + stream, tested).
import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import type { EvidenceRef, ResearchAnswer, ResearchEpisode } from "../contracts/generated";
import { FIXTURES } from "../research/fixtures";
import { availableActions, DraftKey, TERMINAL_STATUSES } from "../research/episodes";
import { emptyState, type AnswerState, type EpisodeState } from "../research/eventLog";
import type { EpisodeAction, HostDecision } from "../research/serverTypes";
import { followEpisode, type StreamNotice } from "../research/stream";
import { useData } from "../shell/data";
import { usePanel } from "../shell/inspector";
import { useEpisodes } from "../shell/queries";
import { go, href } from "../shell/router";
import { IconArrow, IconChevron } from "../shell/icons";
import { CLAIM_LABEL, ClaimGlyph, ErrorNote, EvidenceClassTag, Gap, Synthetic, shortId, words } from "../ui/common";
import { AnswerView, locatorText } from "./Answer";
import { AskBox } from "./AskBox";
import { elapsed, toTurns, TOOL_LABEL, type Stop, type Turn } from "./turns";

type Notice = StreamNotice & { at: number };
type Tab = "thread" | "evidence" | "route";

export function Research({ episodeId, newQuestion, wantNew, tab }: { episodeId: string | null; newQuestion: string | null; wantNew: boolean; tab: string | null }) {
  const { ready } = useData();
  if (!ready) return <div className="view muted">Connecting…</div>;
  if (episodeId) return <Episode key={episodeId} episodeId={episodeId} tab={tab === "evidence" || tab === "route" ? tab : "thread"} />;
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
        <h1>{wantNew ? "New investigation" : "Investigations"}</h1>
      </div>
      <AskBox onAsk={create} busy={busy} initial={question ?? ""} autoFocus={wantNew} />
      {researchMode === "mock" && (
        <Gap title="Read-only replay">No server is reachable, so the recorded fixture investigation is replayed in the browser. Start <code>ignis serve --dev</code> to ask questions.</Gap>
      )}
      {error != null && <ErrorNote error={error} title="The investigation was not started" />}
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
                  No investigations yet.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
      {episodes.error && <ErrorNote error={episodes.error} title="Investigations could not be loaded" />}
    </div>
  );
}

// ---------------------------------------------------------------- one investigation

const ACTION_LABEL: Record<EpisodeAction, string> = { pause: "Pause", resume: "Resume", cancel: "Cancel" };

function Episode({ episodeId, tab }: { episodeId: string; tab: Tab }) {
  const { client, researchMode, replay, caps, synthetic, faults, setFaults, bumpReplay } = useData();
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
  const allStops = useMemo(() => turns.flatMap((t) => t.stops), [turns]);
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
        <ErrorNote error={episode.error} title="This investigation is not in the current data source" />
        <div className="toolbar">
          <a className="btn" href={href("research")}>
            All investigations
          </a>
        </div>
      </div>
    );

  const statusTag = status === "completed" ? "tag-ok" : status === "failed" || status === "cancelled" ? "tag-bad" : "tag-flame";
  const live = following && !(status && TERMINAL_STATUSES.includes(status));
  const tabHref = (t: Tab) => href("research", episodeId, t === "thread" ? undefined : { tab: t });

  return (
    <div className="page">
      <div className="page-main">
        <header className="page-head">
          <h1 title={episode.data?.goal}>{episode.data?.goal ?? (episode.isPending ? "Loading…" : "Investigation")}</h1>
          <nav className="tabs" aria-label="Investigation views">
            <a href={tabHref("thread")} aria-current={tab === "thread" ? "page" : undefined}>
              Thread
            </a>
            <a href={tabHref("evidence")} aria-current={tab === "evidence" ? "page" : undefined}>
              Evidence <span className="count">{state.answers.length ? "" : "0"}</span>
            </a>
            <a href={tabHref("route")} aria-current={tab === "route" ? "page" : undefined}>
              Route <span className="count">{allStops.length}</span>
            </a>
          </nav>
        </header>

        {actionError != null && <ErrorNote error={actionError} title="The server refused the action" />}
        {state.missing.map((m) => (
          <Gap key={m.from} title={`Events ${m.from}–${m.to} were not received`}>
            The log no longer covered this point (HTTP 410); the state continues from the snapshot.
          </Gap>
        ))}
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

        {tab === "thread" && (
          <div className="thread">
            {turns.length === 0 && <p className="muted">{following ? "Waiting for the first event…" : "No events yet."}</p>}
            {turns.map((t, i) => (
              <TurnView key={i} turn={t} live={live && i === turns.length - 1} />
            ))}
            {status && TERMINAL_STATUSES.includes(status) && state.answers.length === 0 && turns.length > 0 && <Gap title="No answer was produced">The investigation ended ({words(status)}).</Gap>}
            <Composer
              key={episodeId}
              disabledReason={!canWrite ? "Read-only replay: start the server to ask follow-ups" : status === "cancelled" || status === "failed" ? `The investigation is ${words(status)}` : null}
              send={async (text, k) => {
                await client.postMessage(episodeId, text, k);
                setFollow((f) => f + 1);
              }}
            />
          </div>
        )}

        {tab === "evidence" && (
          <div style={{ display: "grid", gap: 14, paddingBottom: 32 }}>
            {state.answers.length === 0 && <Gap title="No evidence yet">Claims and their sources appear here when an answer is ready.</Gap>}
            {state.answers.map((a, i) => (
              <EvidenceGrid key={a.answer_id} a={a} n={i + 1} />
            ))}
          </div>
        )}

        {tab === "route" && (
          <div style={{ display: "grid", gap: 14, paddingBottom: 32 }}>
            {turns.map((t, i) => (
              <section key={i} className="card" aria-label={`Route of turn ${i + 1}`}>
                <div className="card-head">
                  <h2>{t.user ? t.user.text : `Turn ${i + 1}`}</h2>
                  <span className="muted small">
                    {t.stops.length} stops{elapsed(t.startedAt, t.endedAt) && ` · ${elapsed(t.startedAt, t.endedAt)}`}
                  </span>
                </div>
                <Stops stops={t.stops} t0={t.startedAt} />
              </section>
            ))}
            {turns.length === 0 && <Gap title="No route yet">Tool calls, jobs and approvals appear here as the agent works.</Gap>}
            <details className="route">
              <summary>
                <b>Stream log</b> <span>{notices.length} notices</span>
                <IconChevron className="chev" />
              </summary>
              <div style={{ padding: "0 12px 12px", display: "grid", gap: 8 }}>
                <ul style={{ display: "grid", gap: 2 }} className="small">
                  {notices.map((n, k) => (
                    <li key={k} style={{ listStyle: "none" }} className={n.kind === "error" || n.kind === "gap" || n.kind === "gave_up" ? "" : "muted"}>
                      {noticeText(n)}
                    </li>
                  ))}
                </ul>
                {researchMode === "mock" && (
                  <div className="toolbar small">
                    <label>
                      Drop after event{" "}
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
              </div>
            </details>
          </div>
        )}
      </div>

      {!panel.current && (
        <aside className="props" aria-label="Properties">
          <div className="panel-sec">
            <h2>Properties</h2>
            <dl className="kv">
              <dt>Status</dt>
              <dd>
                <span className={`tag ${statusTag}`}>{status ? words(status) : "connecting"}</span>
              </dd>
              {episode.data && (
                <>
                  <dt>Created</dt>
                  <dd className="num">{episode.data.created_at.slice(0, 16).replace("T", " ")}</dd>
                  <dt>Corpus</dt>
                  <dd>
                    <code>{episode.data.pins.corpus_snapshot ?? "not pinned"}</code>
                  </dd>
                  <dt>Origin</dt>
                  <dd>{episode.data.origin === "synthetic_fixture" ? <Synthetic /> : "system"}</dd>
                </>
              )}
            </dl>
          </div>
          <details>
            <summary className="muted small" style={{ cursor: "pointer" }}>
              Details
            </summary>
            <dl className="kv" style={{ marginTop: 8 }}>
              <dt>Stream</dt>
              <dd>{following ? "live" : "closed"}</dd>
              <dt>Events</dt>
              <dd className="num">{state.cursor}</dd>
              <dt>Answers</dt>
              <dd className="num">{state.answers.length}</dd>
              <dt>Runtime</dt>
              <dd>{episode.data?.runtime ? words(episode.data.runtime.runtime) : "not attached"}</dd>
              <dt>Releases</dt>
              <dd>{episode.data?.pins.model_releases?.length ? episode.data.pins.model_releases.map((r) => <code key={r}>{shortId(r, 16)} </code>) : "none pinned"}</dd>
              <dt>Harness</dt>
              <dd>
                {episode.data?.pins.harness ? (
                  <span title={episode.data.pins.harness.digest}>
                    <b>{episode.data.pins.harness.label}</b> · {episode.data.pins.harness.model}
                    {episode.data.pins.harness.effort ? ` · ${episode.data.pins.harness.effort}` : ""} · <code>{shortId(episode.data.pins.harness.digest, 16)}</code>
                    <div className="dim">runtime {episode.data.pins.harness.runtime_build}</div>
                  </span>
                ) : (
                  "not pinned"
                )}
              </dd>
            </dl>
          </details>
          {canWrite && availableActions(status).length > 0 && (
            <div className="toolbar">
              {(["pause", "resume", "cancel"] as EpisodeAction[])
                .filter((a) => availableActions(status).includes(a))
                .map((a) => (
                  <button key={a} className="btn btn-s" onClick={() => act(a)}>
                    {ACTION_LABEL[a]}
                  </button>
                ))}
            </div>
          )}
          {runtime?.available === false && researchMode !== "mock" && <p className="muted small">No agent runtime is configured, so no new answer will come.</p>}
          {!synthetic && episode.data?.origin === "synthetic_fixture" && <Synthetic />}
        </aside>
      )}
    </div>
  );
}

function TurnView({ turn, live }: { turn: Turn; live: boolean }) {
  const { synthetic } = useData();
  const replies = turn.replies.filter((m) => m.text.trim());
  return (
    <>
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
    </>
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

// ---------------------------------------------------------------- evidence grid

function EvidenceGrid({ a, n }: { a: AnswerState; n: number }) {
  const q = useAnswer(a);
  const panel = usePanel();
  if (q.isPending) return <p className="muted">Loading answer {n}…</p>;
  if (q.error) return <ErrorNote error={q.error} title={`Answer ${n} could not be loaded`} />;
  const ans = q.data;
  const ev = new Map((ans.evidence ?? []).map((e) => [e.evidence_id, e]));
  return (
    <div className="tbl-wrap">
      <table className="tbl grid-ev">
        <caption>
          Answer {n} · {words(ans.status)} · {ans.claims?.length ?? 0} claims
        </caption>
        <thead>
          <tr>
            <th className="r">#</th>
            <th>Claim</th>
            <th>Kind</th>
            <th>Evidence</th>
            <th>Where</th>
            <th>Quote</th>
          </tr>
        </thead>
        <tbody>
          {(ans.claims ?? []).map((c, i) => {
            const sup = c.support ?? [];
            const first = sup[0];
            const e: EvidenceRef | undefined = first?.kind === "evidence" ? ev.get(first.evidence_id) : undefined;
            const open = () =>
              panel.open({ kind: "claim", claim: c, keyLabel: String(i + 1), evidence: sup.flatMap((s) => (s.kind === "evidence" && ev.get(s.evidence_id) ? [ev.get(s.evidence_id)!] : [])), artifacts: ans.artifacts ?? [] });
            const on = panel.current?.kind === "claim" && panel.current.claim.claim_id === c.claim_id;
            return (
              <tr key={c.claim_id} className="clickable" aria-selected={on} onClick={open}>
                <td className="r dim num">{i + 1}</td>
                <td className="claim">{c.text}</td>
                <td>
                  <span style={{ display: "inline-flex", gap: 6, alignItems: "center" }}>
                    <ClaimGlyph kind={c.kind} /> {CLAIM_LABEL[c.kind]}
                  </span>
                </td>
                <td>{e ? <EvidenceClassTag cls={e.evidence_class} /> : first?.kind === "artifact_value" ? <span className="tag tag-flame">artifact value</span> : <span className="dim">none</span>}</td>
                <td className="dim">{e ? locatorText(e) : first?.kind === "artifact_value" ? <code>{first.pointer}</code> : "—"}</td>
                <td className="quote-cell">{e?.quote ?? <span className="dim">{e ? "not carried" : "—"}</span>}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
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
        <b>Route</b>
        <span>
          {tools} tool call{tools === 1 ? "" : "s"}
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
