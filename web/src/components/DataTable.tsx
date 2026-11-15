// A readable data table for answer views: column headers from the keys with their units, numbers
// right-aligned in tabular figures and shown in at most 4 significant digits (the full value in the
// cell's tooltip), click a header to sort, sticky header inside a scrolling frame, paged rows.
import { useMemo, useState } from "react";
import { fmtNum, fullNum, sortRows, type TableColumn } from "../research/chartRows";

const PAGE = 100;

function Cell({ v, numeric }: { v: unknown; numeric: boolean }) {
  if (v == null || v === "") return <td className={`dim${numeric ? " r" : ""}`}>—</td>;
  if (typeof v === "number") return <td className="r num" title={fullNum(v)}>{fmtNum(v)}</td>;
  if (typeof v === "boolean") return <td className={numeric ? "r" : undefined}>{v ? "yes" : "no"}</td>;
  if (typeof v === "object") {
    const s = JSON.stringify(v);
    return (
      <td className="mono-cell" title={s.length > 60 ? s : undefined}>
        {s.length > 60 ? `${s.slice(0, 57)}…` : s}
      </td>
    );
  }
  const s = String(v);
  return (
    <td className={s.length > 48 ? "wrap" : undefined} title={s.length > 120 ? s : undefined}>
      {s.length > 120 ? `${s.slice(0, 117)}…` : s}
    </td>
  );
}

export function DataTable({ columns, rows, caption, maxHeight = 420 }: { columns: TableColumn[]; rows: unknown[][]; caption?: string; maxHeight?: number }) {
  const [sort, setSort] = useState<{ col: number; dir: "asc" | "desc" } | null>(null);
  const [shown, setShown] = useState(PAGE);
  const sorted = useMemo(() => (sort ? sortRows(rows, sort.col, sort.dir) : rows), [rows, sort]);
  const toggle = (col: number) =>
    setSort((s) => (s?.col !== col ? { col, dir: columns[col].numeric ? "desc" : "asc" } : s.dir === (columns[col].numeric ? "desc" : "asc") ? { col, dir: s.dir === "asc" ? "desc" : "asc" } : null));
  return (
    <div className="dt">
      <div className="tbl-wrap dt-scroll" style={{ maxHeight }}>
        <table className="tbl tbl-data">
          {caption && <caption>{caption}</caption>}
          <thead>
            <tr>
              {columns.map((c, j) => {
                const on = sort?.col === j;
                return (
                  <th key={c.key} scope="col" className={c.numeric ? "r" : undefined} aria-sort={on ? (sort.dir === "asc" ? "ascending" : "descending") : undefined}>
                    <button className={`sort${on ? " on" : ""}`} onClick={() => toggle(j)} title={`Sort by ${c.key}`}>
                      <span>{c.title}</span>
                      {c.unit && <span className="unit">{c.unit}</span>}
                      <span className="sort-mark" aria-hidden="true">
                        {on ? (sort.dir === "asc" ? "↑" : "↓") : "↕"}
                      </span>
                    </button>
                  </th>
                );
              })}
            </tr>
          </thead>
          <tbody>
            {sorted.slice(0, shown).map((r, i) => (
              <tr key={i}>
                {r.map((v, j) => (
                  <Cell key={j} v={v} numeric={columns[j]?.numeric ?? false} />
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="dt-foot">
        <span>
          {rows.length} {rows.length === 1 ? "row" : "rows"}
          {sort && ` · sorted by ${columns[sort.col].title} ${sort.dir === "asc" ? "ascending" : "descending"}`}
        </span>
        {rows.length > shown && (
          <button className="btn btn-s btn-quiet" onClick={() => setShown((n) => n + PAGE)}>
            Show {Math.min(PAGE, rows.length - shown)} more of {rows.length - shown}
          </button>
        )}
      </div>
    </div>
  );
}
