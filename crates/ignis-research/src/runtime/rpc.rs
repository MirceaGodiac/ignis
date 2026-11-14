//! Private stdio JSON-RPC connection to one `codex app-server` child process.
//!
//! - stdout carries the protocol only; the child's stderr is drained to the log on its own thread.
//! - A reader thread assembles messages incrementally ([`LineAssembler`]) and routes them:
//!   responses to the waiting caller, server-initiated requests and notifications, in order, to the
//!   inbound channel of the bridge's dispatcher.
//! - Writes are serialized under one lock, one complete line each.
//! - Nothing listens on a network socket: the transport is `--listen stdio://`.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};

use super::framing::{encode, LineAssembler};

/// A message the server sent that is not a response to one of our requests.
#[derive(Debug, Clone)]
pub enum Inbound {
    Notification {
        method: String,
        params: Value,
    },
    /// Server-initiated request (approvals and the like): it must be answered explicitly.
    Request {
        id: Value,
        method: String,
        params: Value,
    },
    /// stdout ended (the process exited or closed it).
    Closed {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallError {
    /// The server answered with a JSON-RPC error.
    Rpc { code: i64, message: String },
    /// No answer in time: whether the request took effect is unknown.
    Timeout,
    /// The connection is gone: whether the request took effect is unknown.
    Closed(String),
}

impl CallError {
    /// The request may have taken effect (timeout, lost connection): reconcile, never resend blindly.
    pub fn is_ambiguous(&self) -> bool {
        !matches!(self, CallError::Rpc { .. })
    }
}

impl std::fmt::Display for CallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallError::Rpc { code, message } => write!(f, "app-server error {code}: {message}"),
            CallError::Timeout => f.write_str("app-server did not answer in time"),
            CallError::Closed(r) => write!(f, "app-server connection closed: {r}"),
        }
    }
}

type Waiter = mpsc::Sender<Result<Value, CallError>>;

pub struct Connection {
    label: String,
    stdin: Mutex<Option<ChildStdin>>,
    child: Mutex<Child>,
    pending: Arc<Mutex<HashMap<i64, Waiter>>>,
    next_id: AtomicI64,
    closed: Arc<AtomicBool>,
}

impl Connection {
    /// Spawn `cmd` with piped stdio and start the reader and stderr threads. Inbound messages go to
    /// `inbound` in wire order.
    pub fn spawn(
        mut cmd: Command,
        label: impl Into<String>,
        inbound: mpsc::Sender<Inbound>,
    ) -> std::io::Result<Arc<Self>> {
        let label = label.into();
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().expect("piped stdout");
        let stderr = child.stderr.take().expect("piped stderr");
        let conn = Arc::new(Connection {
            label: label.clone(),
            stdin: Mutex::new(stdin),
            child: Mutex::new(child),
            pending: Arc::new(Mutex::new(HashMap::new())),
            next_id: AtomicI64::new(1),
            closed: Arc::new(AtomicBool::new(false)),
        });
        let (pending, closed, l) = (conn.pending.clone(), conn.closed.clone(), label.clone());
        std::thread::Builder::new()
            .name(format!("codex-read-{label}"))
            .spawn(move || read_loop(stdout, pending, closed, inbound, l))?;
        let l = label;
        std::thread::Builder::new()
            .name(format!("codex-stderr-{l}"))
            .spawn(move || {
                for line in BufReader::new(stderr).lines() {
                    match line {
                        Ok(line) => tracing::debug!(target: "codex", runtime = %l, "{line}"),
                        Err(_) => break,
                    }
                }
            })?;
        Ok(conn)
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    fn write(&self, message: &Value) -> Result<(), CallError> {
        let mut guard = self.stdin.lock().unwrap_or_else(|p| p.into_inner());
        let Some(stdin) = guard.as_mut() else {
            return Err(CallError::Closed("stdin closed".into()));
        };
        stdin
            .write_all(&encode(message))
            .and_then(|_| stdin.flush())
            .map_err(|e| CallError::Closed(format!("write: {e}")))
    }

    /// Send a request and wait for its response.
    pub fn request(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, CallError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel();
        self.pending
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(id, tx);
        if self.is_closed() {
            self.pending
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(&id);
            return Err(CallError::Closed("connection closed".into()));
        }
        if let Err(e) = self.write(&json!({"id": id, "method": method, "params": params})) {
            self.pending
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(&id);
            return Err(e);
        }
        match rx.recv_timeout(timeout) {
            Ok(r) => r,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                self.pending
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .remove(&id);
                Err(CallError::Timeout)
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err(CallError::Closed("connection closed".into()))
            }
        }
    }

    pub fn notify(&self, method: &str, params: Option<Value>) -> Result<(), CallError> {
        let msg = match params {
            Some(p) => json!({"method": method, "params": p}),
            None => json!({"method": method}),
        };
        self.write(&msg)
    }

    /// Answer a server-initiated request.
    pub fn respond(&self, id: &Value, result: Value) -> Result<(), CallError> {
        self.write(&json!({"id": id, "result": result}))
    }

    pub fn respond_error(&self, id: &Value, code: i64, message: &str) -> Result<(), CallError> {
        self.write(&json!({"id": id, "error": {"code": code, "message": message}}))
    }

    /// `initialize` then `initialized` (the order the protocol requires). Returns the
    /// `InitializeResponse`.
    pub fn handshake(&self, opt_out: &[&str], timeout: Duration) -> Result<Value, CallError> {
        let resp = self.request(
            "initialize",
            json!({
                "clientInfo": {
                    "name": "ignis",
                    "title": "IGNIS research runtime",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "capabilities": {
                    "experimentalApi": false,
                    "optOutNotificationMethods": opt_out,
                },
            }),
            timeout,
        )?;
        if resp.get("userAgent").and_then(Value::as_str).is_none() {
            return Err(CallError::Rpc {
                code: -32603,
                message: "initialize response has no userAgent (protocol mismatch)".into(),
            });
        }
        self.notify("initialized", None)?;
        Ok(resp)
    }

    /// Stop the child process (closing stdin first, then killing it).
    pub fn kill(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.stdin.lock().unwrap_or_else(|p| p.into_inner()).take();
        let mut child = self.child.lock().unwrap_or_else(|p| p.into_inner());
        let _ = child.kill();
        let _ = child.wait();
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.kill();
    }
}

fn read_loop(
    mut stdout: impl Read,
    pending: Arc<Mutex<HashMap<i64, Waiter>>>,
    closed: Arc<AtomicBool>,
    inbound: mpsc::Sender<Inbound>,
    label: String,
) {
    let mut asm = LineAssembler::new();
    let mut buf = vec![0u8; 64 * 1024];
    let reason = loop {
        match stdout.read(&mut buf) {
            Ok(0) => break "end of stream".to_string(),
            Ok(n) => {
                asm.push(&buf[..n]);
                while let Some(line) = asm.next_line() {
                    route(&line, &pending, &inbound, &label);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => break format!("read: {e}"),
        }
    };
    if asm.pending() > 0 {
        tracing::warn!(runtime = %label, bytes = asm.pending(), "app-server stream ended inside a message");
    }
    closed.store(true, Ordering::SeqCst);
    for (_, w) in pending.lock().unwrap_or_else(|p| p.into_inner()).drain() {
        let _ = w.send(Err(CallError::Closed(reason.clone())));
    }
    let _ = inbound.send(Inbound::Closed { reason });
}

fn route(
    line: &[u8],
    pending: &Mutex<HashMap<i64, Waiter>>,
    inbound: &mpsc::Sender<Inbound>,
    label: &str,
) {
    let msg: Value = match serde_json::from_slice(line) {
        Ok(Value::Object(m)) => Value::Object(m),
        Ok(_) | Err(_) => {
            // Never guessed at: a line that is not a JSON object is a protocol violation.
            tracing::warn!(runtime = %label, bytes = line.len(), "app-server wrote a line that is not a JSON-RPC message");
            return;
        }
    };
    let method = msg
        .get("method")
        .and_then(Value::as_str)
        .map(str::to_string);
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    match (method, msg.get("id")) {
        (Some(method), Some(id)) => {
            let _ = inbound.send(Inbound::Request {
                id: id.clone(),
                method,
                params,
            });
        }
        (Some(method), None) => {
            let _ = inbound.send(Inbound::Notification { method, params });
        }
        (None, Some(id)) => {
            let waiter = id.as_i64().and_then(|id| {
                pending
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .remove(&id)
            });
            let Some(w) = waiter else {
                tracing::warn!(runtime = %label, %id, "response to an unknown or expired request");
                return;
            };
            let result = match msg.get("error") {
                Some(e) => Err(CallError::Rpc {
                    code: e.get("code").and_then(Value::as_i64).unwrap_or(0),
                    message: e
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                }),
                None => Ok(msg.get("result").cloned().unwrap_or(Value::Null)),
            };
            let _ = w.send(result);
        }
        (None, None) => {
            tracing::warn!(runtime = %label, "app-server message with neither method nor id");
        }
    }
}
