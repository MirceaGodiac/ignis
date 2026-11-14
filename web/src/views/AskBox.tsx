// The prompt: one field, send on Enter. Nothing else competes with the question.
import { useState, type FormEvent } from "react";
import { IconArrow } from "../shell/icons";

export function AskBox({ onAsk, busy = false, initial = "", autoFocus = false, synthetic = false }: { onAsk: (q: string) => void; busy?: boolean; initial?: string; autoFocus?: boolean; synthetic?: boolean }) {
  const [q, setQ] = useState(initial);
  const submit = (e?: FormEvent) => {
    e?.preventDefault();
    if (q.trim() && !busy) onAsk(q.trim());
  };
  return (
    <form className="prompt" onSubmit={submit} aria-label="Ask a research question">
      <label htmlFor="ask-q" className="sr-only">
        Research question
      </label>
      <textarea
        id="ask-q"
        rows={2}
        value={q}
        autoFocus={autoFocus}
        onChange={(e) => setQ(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            submit();
          }
        }}
        placeholder="Ask about an investigation, a test condition or a fire-safety question"
      />
      <div className="prompt-foot">
        <span className="muted small">The agent searches the sources and data, then answers with citations.</span>
        {synthetic && (
          <span className="pill-synth" style={{ marginLeft: "auto" }} title="Development fixtures: generated data, not NASA results">
            Synthetic data
          </span>
        )}
        <button className="btn btn-primary send" type="submit" disabled={!q.trim() || busy} aria-label="Ask">
          <IconArrow />
        </button>
      </div>
    </form>
  );
}

export const EXAMPLES = [
  "Which investigations are closest to a 30 % O₂, 10.2 psi Moon-base atmosphere?",
  "How does ambient oxygen change droplet burning rate in FLEX?",
  "What do BASS results say about flame spread at low airflow?",
  "Which data could support a predictive flammability model?",
];
