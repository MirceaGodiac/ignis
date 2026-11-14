// Minimal SSE wire format (WHATWG "server-sent events" field rules) for the episode stream.
// CONTRACTS.md §Streaming: `id: <seq>`, `event: <body.kind>`, `data: <ResearchEvent JSON>`,
// heartbeats as `:` comments. EventSource is not used: it hides HTTP status (410) and cannot
// resume with a manual `?after=` cursor. Pure module: no runtime imports (tested with node --test).

export interface SseFrame {
  id: string | null;
  event: string;
  data: string;
}

/** Incremental parser: feed decoded text chunks, get complete frames. Comments are dropped. */
export class SseParser {
  private buf = "";
  private id: string | null = null;
  private event = "";
  private data: string[] = [];

  push(chunk: string): SseFrame[] {
    this.buf += chunk;
    const out: SseFrame[] = [];
    let nl: number;
    while ((nl = this.buf.search(/\r\n|\r|\n/)) >= 0) {
      // A chunk that ends on "\r" may be the first half of "\r\n": wait for the next chunk, or the
      // "\n" would read as an empty line and dispatch the frame early.
      if (nl === this.buf.length - 1 && this.buf.endsWith("\r")) break;
      const line = this.buf.slice(0, nl);
      const sepLen = this.buf.startsWith("\r\n", nl) ? 2 : 1;
      this.buf = this.buf.slice(nl + sepLen);
      if (line === "") {
        if (this.data.length) out.push({ id: this.id, event: this.event || "message", data: this.data.join("\n") });
        this.id = null;
        this.event = "";
        this.data = [];
        continue;
      }
      if (line.startsWith(":")) continue; // heartbeat / comment
      const colon = line.indexOf(":");
      const field = colon < 0 ? line : line.slice(0, colon);
      let value = colon < 0 ? "" : line.slice(colon + 1);
      if (value.startsWith(" ")) value = value.slice(1);
      if (field === "id") this.id = value;
      else if (field === "event") this.event = value;
      else if (field === "data") this.data.push(value);
      // `retry` and unknown fields are ignored.
    }
    return out;
  }
}

/** Encode one event as an SSE frame (used by the mock transport so the parser runs on real wire text). */
export function encodeFrame(seq: number, kind: string, json: string): string {
  return `id: ${seq}\nevent: ${kind}\ndata: ${json}\n\n`;
}

export const HEARTBEAT = ": heartbeat\n\n";
