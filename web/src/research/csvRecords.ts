// CSV records for the original-source viewer: quote-aware splitting of a text stream into records
// (a quoted field may span lines), so a locator's `row` (1-based record number, header = 1, as
// `ignis_data::text::Row`) points at the same record the server read. Pure, so node --test runs it.

export interface CsvState {
  /** Characters of the record being read. */
  cur: string;
  inQuotes: boolean;
  /** A `\r` ended the previous chunk; a following `\n` belongs to the same line break. */
  pendingCr: boolean;
}

export const csvStart = (): CsvState => ({ cur: "", inQuotes: false, pendingCr: false });

/** Feed one decoded chunk; returns the records completed in it (raw record text, no line break). */
export function csvFeed(st: CsvState, chunk: string): string[] {
  const out: string[] = [];
  for (let i = 0; i < chunk.length; i++) {
    const c = chunk[i];
    if (st.pendingCr) {
      st.pendingCr = false;
      if (c === "\n") continue;
    }
    if (c === '"') {
      st.inQuotes = !st.inQuotes;
      st.cur += c;
    } else if ((c === "\n" || c === "\r") && !st.inQuotes) {
      out.push(st.cur);
      st.cur = "";
      if (c === "\r") st.pendingCr = true;
    } else {
      st.cur += c;
    }
  }
  return out;
}

/** The last record when the stream ends without a line break. */
export const csvEnd = (st: CsvState): string[] => (st.cur.length ? [st.cur] : []);

/** Split one record into fields (RFC 4180 quoting). */
export function csvFields(record: string): string[] {
  const out: string[] = [];
  let f = "";
  let q = false;
  for (let i = 0; i < record.length; i++) {
    const c = record[i];
    if (q) {
      if (c === '"' && record[i + 1] === '"') {
        f += '"';
        i++;
      } else if (c === '"') q = false;
      else f += c;
    } else if (c === '"') q = true;
    else if (c === ",") {
      out.push(f);
      f = "";
    } else f += c;
  }
  out.push(f);
  return out;
}
