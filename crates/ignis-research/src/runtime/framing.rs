//! Wire framing of the Codex App Server stdio transport: one JSON-RPC message per line (JSONL),
//! without the `"jsonrpc"` member (the generated schema of the pinned binary has none).
//!
//! An OS read is not a message: a read may end inside a message or carry several of them, so bytes
//! are assembled incrementally and a message is released only at its newline.

use serde_json::Value;

/// Incremental line assembler. Feed it whatever a `read` returned; take complete lines out.
#[derive(Debug, Default)]
pub struct LineAssembler {
    buf: Vec<u8>,
    /// Start of the first unreleased line in `buf`.
    start: usize,
    /// Bytes before this index (from `start`) are known not to contain a newline.
    scanned: usize,
}

impl LineAssembler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append bytes from one read.
    pub fn push(&mut self, chunk: &[u8]) {
        // Drop released lines before growing, so the buffer holds only the unfinished tail.
        if self.start > 0 && self.start == self.buf.len() {
            self.buf.clear();
            self.scanned = 0;
            self.start = 0;
        } else if self.start > 0 && self.start >= self.buf.len() / 2 {
            self.buf.drain(..self.start);
            self.scanned -= self.start;
            self.start = 0;
        }
        self.buf.extend_from_slice(chunk);
    }

    /// The next complete line (without `\n` or a trailing `\r`), skipping empty lines.
    pub fn next_line(&mut self) -> Option<Vec<u8>> {
        loop {
            let from = self.scanned.max(self.start);
            let nl = self.buf[from..].iter().position(|&b| b == b'\n');
            let Some(off) = nl else {
                self.scanned = self.buf.len();
                return None;
            };
            let end = from + off;
            let mut line = &self.buf[self.start..end];
            if line.last() == Some(&b'\r') {
                line = &line[..line.len() - 1];
            }
            let out = line.to_vec();
            self.start = end + 1;
            self.scanned = self.start;
            if !out.iter().all(u8::is_ascii_whitespace) {
                return Some(out);
            }
        }
    }

    /// Bytes of an unfinished message (non-zero at end of stream means a truncated message).
    pub fn pending(&self) -> usize {
        self.buf.len() - self.start
    }
}

/// One message as written on the wire: compact JSON (no raw newline can occur) plus `\n`.
pub fn encode(message: &Value) -> Vec<u8> {
    let mut out = serde_json::to_vec(message).expect("a JSON value serializes");
    out.push(b'\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(a: &mut LineAssembler) -> Vec<String> {
        std::iter::from_fn(|| a.next_line())
            .map(|l| String::from_utf8(l).unwrap())
            .collect()
    }

    #[test]
    fn a_message_split_across_reads_is_released_once_complete() {
        let wire = b"{\"id\":1,\"result\":{\"text\":\"a\\nb\"}}\n";
        let mut a = LineAssembler::new();
        let mut got = Vec::new();
        for byte in wire.iter() {
            a.push(std::slice::from_ref(byte));
            got.extend(lines(&mut a));
        }
        assert_eq!(
            got,
            vec![String::from_utf8(wire[..wire.len() - 1].to_vec()).unwrap()]
        );
        assert_eq!(a.pending(), 0);
    }

    #[test]
    fn several_messages_in_one_read_are_separated() {
        let mut a = LineAssembler::new();
        a.push(b"{\"a\":1}\r\n\n{\"b\":2}\n{\"c\":");
        assert_eq!(lines(&mut a), vec!["{\"a\":1}", "{\"b\":2}"]);
        assert!(a.pending() > 0, "the unfinished third message is kept");
        a.push(b"3}\n");
        assert_eq!(lines(&mut a), vec!["{\"c\":3}"]);
        assert_eq!(a.pending(), 0);
    }

    #[test]
    fn encoded_messages_are_single_lines() {
        let v = serde_json::json!({"method": "x", "params": {"text": "line1\nline2"}});
        let wire = encode(&v);
        assert_eq!(wire.iter().filter(|&&b| b == b'\n').count(), 1);
        let mut a = LineAssembler::new();
        a.push(&wire);
        let back: Value = serde_json::from_slice(&a.next_line().unwrap()).unwrap();
        assert_eq!(back, v);
    }
}
