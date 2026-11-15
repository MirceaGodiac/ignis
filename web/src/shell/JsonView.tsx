import { useState } from "react";

/**
 * A readable JSON tree: syntax colours, wrapped strings (no horizontal scroll for long prose),
 * collapsible objects/arrays and a copy button. Purely presentational — the value is shown whole;
 * deep branches start folded so the shape is visible first.
 */
export function JsonView({ value, openDepth = 4 }: { value: unknown; openDepth?: number }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    void navigator.clipboard?.writeText(JSON.stringify(value, null, 2)).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1200);
    });
  };
  return (
    <div className="jv">
      <button type="button" className="jv-copy" onClick={copy} aria-label="Copy JSON">
        {copied ? "Copied" : "Copy"}
      </button>
      <div className="jv-body">
        {/* A top-level object shows its keys directly: the outer braces add nothing to read. */}
        {value !== null && typeof value === "object" && !Array.isArray(value) && Object.keys(value).length > 0 ? (
          Object.entries(value as Record<string, unknown>).map(([k, v]) => <Node key={k} name={k} value={v} depth={1} openDepth={openDepth} />)
        ) : (
          <Node value={value} depth={0} openDepth={openDepth} />
        )}
      </div>
    </div>
  );
}

function Node({ name, value, depth, openDepth, last = true }: { name?: string | number; value: unknown; depth: number; openDepth: number; last?: boolean }) {
  const isArr = Array.isArray(value);
  const isObj = value !== null && typeof value === "object";
  const [open, setOpen] = useState(depth < openDepth);
  const key =
    name === undefined ? null : typeof name === "number" ? null : (
      <>
        <span className="jv-key">{name}</span>
        <span className="jv-punct">: </span>
      </>
    );
  const comma = last ? null : <span className="jv-punct">,</span>;

  if (!isObj) {
    return (
      <div className="jv-row">
        {key}
        <Scalar value={value} />
        {comma}
      </div>
    );
  }

  const entries: [string | number, unknown][] = isArr ? (value as unknown[]).map((v, i) => [i, v]) : Object.entries(value as Record<string, unknown>);
  const [o, c] = isArr ? ["[", "]"] : ["{", "}"];
  if (entries.length === 0) {
    return (
      <div className="jv-row">
        {key}
        <span className="jv-punct">
          {o}
          {c}
        </span>
        {comma}
      </div>
    );
  }
  const count = `${entries.length} ${isArr ? (entries.length === 1 ? "item" : "items") : entries.length === 1 ? "key" : "keys"}`;
  return (
    <div className="jv-node">
      <button type="button" className="jv-row jv-toggle" onClick={() => setOpen(!open)} aria-expanded={open}>
        <span className={`jv-caret${open ? " open" : ""}`} aria-hidden>
          ▸
        </span>
        {key}
        <span className="jv-punct">{o}</span>
        {!open && (
          <>
            <span className="jv-summary">{count}</span>
            <span className="jv-punct">{c}</span>
            {comma}
          </>
        )}
      </button>
      {open && (
        <>
          <div className="jv-children">
            {entries.map(([k, v], i) => (
              <Node key={k} name={k} value={v} depth={depth + 1} openDepth={openDepth} last={i === entries.length - 1} />
            ))}
          </div>
          <div className="jv-row">
            <span className="jv-punct">{c}</span>
            {comma}
          </div>
        </>
      )}
    </div>
  );
}

function Scalar({ value }: { value: unknown }) {
  if (value === null) return <span className="jv-null">null</span>;
  switch (typeof value) {
    case "string":
      return <span className="jv-str">"{value}"</span>;
    case "number":
      return <span className="jv-num">{String(value)}</span>;
    case "boolean":
      return <span className="jv-bool">{String(value)}</span>;
    default:
      return <span className="jv-null">{String(value)}</span>;
  }
}
