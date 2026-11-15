// Row windowing for long tables: only the rows in view (plus an overscan margin) are in the DOM;
// spacer rows keep the scroll height. CSS `content-visibility` cannot do this for tables, because
// containment does not apply to internal table boxes (`tr`, `tbody`).

export interface RowWindow {
  start: number;
  /** Exclusive. */
  end: number;
  /** Heights of the spacer rows above and below the rendered rows, px. */
  top: number;
  bottom: number;
}

export function rowWindow(scrollTop: number, viewHeight: number, rowHeight: number, rows: number, overscan = 10): RowWindow {
  if (rows <= 0 || rowHeight <= 0) return { start: 0, end: Math.max(0, rows), top: 0, bottom: 0 };
  const first = Math.floor(Math.max(0, scrollTop) / rowHeight);
  const visible = Math.ceil(Math.max(0, viewHeight) / rowHeight) + 1;
  const start = Math.max(0, Math.min(rows - 1, first - overscan));
  const end = Math.min(rows, first + visible + overscan);
  return { start, end, top: start * rowHeight, bottom: (rows - end) * rowHeight };
}
