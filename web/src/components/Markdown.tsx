// Agent text in an answer, rendered from parsed Markdown (research/markdown.ts) with React elements
// only: nothing the agent wrote reaches the DOM as HTML. Run names stay links to the run panel in
// every inline run, table cells included. `tail` (the claim chips of the block) is placed at the end
// of the last paragraph or list item, or on its own line after a closing table or code block.
import { Fragment, useMemo, type ReactNode } from "react";
import { useRunIndex } from "../shell/RichText";
import { usePanel } from "../shell/inspector";
import { isEmptyCell, parseInline, parseMarkdown, type InlinePart, type MdBlock } from "../research/markdown";

type OpenRun = (id: string) => void;

export function Markdown({ text, tail: chips }: { text: string; tail?: ReactNode }) {
  const tail = (Array.isArray(chips) ? chips.some(Boolean) : !!chips) ? chips : null;
  const runs = useRunIndex();
  const panel = usePanel();
  const blocks = useMemo(() => parseMarkdown(text), [text]);
  const openRun: OpenRun = (id) => panel.open({ kind: "run", id });
  const inl = (s: string) => parseInline(s, runs).map((p, i) => renderInline(p, i, openRun));
  const last = blocks.length - 1;
  const tailInline = last >= 0 && (blocks[last].kind === "paragraph" || blocks[last].kind === "list" || blocks[last].kind === "heading");
  return (
    <div className="md">
      {blocks.map((b, i) => renderBlock(b, i, inl, i === last && tailInline ? tail : null))}
      {!tailInline && tail ? <p className="md-tail">{tail}</p> : null}
    </div>
  );
}

function withTail(nodes: ReactNode, tail: ReactNode) {
  return tail ? (
    <>
      {nodes} {tail}
    </>
  ) : (
    nodes
  );
}

function renderBlock(b: MdBlock, key: number, inl: (s: string) => ReactNode[], tail: ReactNode): ReactNode {
  switch (b.kind) {
    case "heading": {
      const H = (b.level <= 2 ? "h3" : "h4") as "h3" | "h4";
      return (
        <H key={key} className={`md-h md-h${Math.min(b.level, 4)}`}>
          {withTail(inl(b.text), tail)}
        </H>
      );
    }
    case "paragraph":
      return <p key={key}>{withTail(inl(b.text), tail)}</p>;
    case "list": {
      const items = b.items.map((it, j) => (
        <li key={j} className={it.level ? `md-l${it.level}` : undefined}>
          {withTail(inl(it.text), j === b.items.length - 1 ? tail : null)}
        </li>
      ));
      return b.ordered ? (
        <ol key={key} start={b.start !== 1 ? b.start : undefined}>
          {items}
        </ol>
      ) : (
        <ul key={key}>{items}</ul>
      );
    }
    case "quote":
      return <blockquote key={key}>{inl(b.text)}</blockquote>;
    case "code":
      return (
        <pre key={key} className="md-code">
          <code>{b.text}</code>
        </pre>
      );
    case "rule":
      return <hr key={key} />;
    case "table":
      return <MdTable key={key} b={b} inl={inl} />;
  }
}

function MdTable({ b, inl }: { b: Extract<MdBlock, { kind: "table" }>; inl: (s: string) => ReactNode[] }) {
  const align = (j: number) => {
    const a = b.align[j] ?? (b.numeric[j] ? "right" : null);
    return a === "right" ? "r" : a === "center" ? "c" : undefined;
  };
  return (
    <div className="tbl-wrap md-table">
      <table className="tbl tbl-data">
        <thead>
          <tr>
            {b.header.map((h, j) => (
              <th key={j} scope="col" className={align(j)}>
                {inl(h)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {b.rows.map((r, i) => (
            <tr key={i}>
              {r.map((c, j) => (
                <td key={j} className={[align(j), b.numeric[j] ? "num" : "", isEmptyCell(c) && c ? "dim" : ""].filter(Boolean).join(" ") || undefined}>
                  {inl(c)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function renderInline(p: InlinePart, i: number, openRun: OpenRun): ReactNode {
  switch (p.kind) {
    case "text":
      return <Fragment key={i}>{p.text}</Fragment>;
    case "break":
      return <br key={i} />;
    case "bold":
      return <b key={i}>{p.parts.map((q, j) => renderInline(q, j, openRun))}</b>;
    case "italic":
      return <em key={i}>{p.parts.map((q, j) => renderInline(q, j, openRun))}</em>;
    case "code":
      return (
        <code key={i} className="md-ic">
          {p.text}
        </code>
      );
    case "link":
      return (
        <a key={i} href={p.href} target="_blank" rel="noopener noreferrer">
          {p.label}
        </a>
      );
    case "run":
      return (
        <button key={i} className="linkish run-link" onClick={() => openRun(p.id)} title={`Open run ${p.id}`}>
          {p.text}
        </button>
      );
  }
}
