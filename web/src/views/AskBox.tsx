// The prompt: one field, send on Enter. Nothing else competes with the question.
import { useState, type FormEvent } from "react";
import { IconArrow } from "../shell/icons";

export function AskBox({ onAsk, busy = false, initial = "", autoFocus = false }: { onAsk: (q: string) => void; busy?: boolean; initial?: string; autoFocus?: boolean; synthetic?: boolean }) {
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
        placeholder="Ask IGNIS"
      />
      <div className="prompt-foot">
        <button className="btn btn-primary send" type="submit" disabled={!q.trim() || busy} aria-label="Ask">
          <IconArrow />
        </button>
      </div>
    </form>
  );
}

export const EXAMPLES = [
  "Which tests come closest to Moon-base air?",
  "How does oxygen change FLEX burning rates?",
  "What did BASS find about flame spread at low airflow?",
  "Which data could feed a flammability model?",
];
