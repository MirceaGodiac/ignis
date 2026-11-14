//! Test double of `codex app-server` (built as the `ignis-fake-codex` binaries). It speaks the
//! pinned protocol over stdio with no model and no network, so the bridge's conformance tests run
//! offline. It is strict where the real server is: requests before `initialize`/`initialized` fail.
//!
//! - `--version` prints `codex-cli <IGNIS_FAKE_CODEX_VERSION or the pinned version>`.
//! - `app-server generate-json-schema --out DIR` writes the message-variant files the preflight reads
//!   (`IGNIS_FAKE_CODEX_SCHEMA=old` leaves out `turn/interrupt`).
//! - `app-server ...` serves the protocol. `IGNIS_FAKE_CODEX_FRAMING=split` writes every message in
//!   3-byte pieces with flushes; `merged` writes all messages of a burst in one write.
//! - Threads persist in `$CODEX_HOME/fake-threads.json`; on start, turns left in progress by a killed
//!   process become `interrupted` (what a reloaded thread looks like).
//! - The turn scenario is chosen by words in the user input: `tool`, `approval`, `shell`,
//!   `permissions`, `opaque`, `hang` (wait for `turn/interrupt`), `fail`; plain text just answers.
//!   `hang` also persists the items produced so far, so a restart can replay them.
//! - Every request method received is appended to `$CODEX_HOME/fake-requests.log`.

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Duration;

use serde_json::{json, Value};

use super::preflight::PINNED_CODEX_VERSION;

pub fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version") {
        let v = std::env::var("IGNIS_FAKE_CODEX_VERSION").unwrap_or(PINNED_CODEX_VERSION.into());
        println!("codex-cli {v}");
        return;
    }
    if args.first().map(String::as_str) != Some("app-server") {
        eprintln!("ignis-fake-codex: only `app-server` is implemented");
        std::process::exit(2);
    }
    if args.get(1).map(String::as_str) == Some("generate-json-schema") {
        let out = args
            .iter()
            .position(|a| a == "--out")
            .and_then(|i| args.get(i + 1))
            .expect("--out DIR");
        write_schema(&PathBuf::from(out));
        return;
    }
    serve();
}

fn variants(methods: &[&str]) -> Value {
    json!({"oneOf": methods.iter().map(|m| json!({
        "type": "object",
        "properties": {"method": {"type": "string", "enum": [m]}}
    })).collect::<Vec<_>>()})
}

fn write_schema(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).unwrap();
    let mut client = vec!["initialize", "thread/start", "thread/resume", "turn/start"];
    if std::env::var("IGNIS_FAKE_CODEX_SCHEMA").as_deref() != Ok("old") {
        client.push("turn/interrupt");
    }
    let files = [
        ("ClientRequest.json", variants(&client)),
        ("ClientNotification.json", variants(&["initialized"])),
        (
            "ServerNotification.json",
            variants(&[
                "thread/started",
                "turn/started",
                "turn/completed",
                "item/started",
                "item/completed",
                "item/agentMessage/delta",
                "serverRequest/resolved",
                "thread/tokenUsage/updated",
            ]),
        ),
        (
            "ServerRequest.json",
            variants(&[
                "item/commandExecution/requestApproval",
                "item/fileChange/requestApproval",
                "item/permissions/requestApproval",
            ]),
        ),
    ];
    for (name, v) in files {
        std::fs::write(dir.join(name), serde_json::to_vec_pretty(&v).unwrap()).unwrap();
    }
}

// ---------------------------------------------------------------------------- server

#[derive(Clone, Copy, PartialEq)]
enum Framing {
    Plain,
    Split,
    Merged,
}

struct Out {
    framing: Framing,
    lock: Mutex<()>,
    opt_out: Mutex<Vec<String>>,
}

impl Out {
    fn send_all(&self, msgs: &[Value]) {
        let msgs: Vec<&Value> = {
            let opt = self.opt_out.lock().unwrap();
            msgs.iter()
                .filter(|m| {
                    m.get("id").is_some()
                        || !m
                            .get("method")
                            .and_then(Value::as_str)
                            .is_some_and(|x| opt.iter().any(|o| o == x))
                })
                .collect()
        };
        let _g = self.lock.lock().unwrap();
        let mut out = std::io::stdout().lock();
        let wire: Vec<Vec<u8>> = msgs
            .iter()
            .map(|m| {
                let mut b = serde_json::to_vec(m).unwrap();
                b.push(b'\n');
                b
            })
            .collect();
        match self.framing {
            Framing::Merged => {
                let _ = out.write_all(&wire.concat());
                let _ = out.flush();
            }
            Framing::Plain => {
                for w in wire {
                    let _ = out.write_all(&w);
                    let _ = out.flush();
                }
            }
            Framing::Split => {
                for w in wire {
                    for piece in w.chunks(3) {
                        let _ = out.write_all(piece);
                        let _ = out.flush();
                        std::thread::sleep(Duration::from_micros(200));
                    }
                }
            }
        }
    }

    fn send(&self, m: Value) {
        self.send_all(&[m]);
    }
}

#[derive(Default)]
struct Store {
    path: PathBuf,
    threads: HashMap<String, Value>,
}

impl Store {
    fn load(path: PathBuf) -> Self {
        let mut threads: HashMap<String, Value> = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        for t in threads.values_mut() {
            for turn in t["turns"].as_array_mut().into_iter().flatten() {
                if turn["status"] == "inProgress" {
                    turn["status"] = json!("interrupted");
                }
            }
        }
        Store { path, threads }
    }

    fn save(&self) {
        let _ = std::fs::write(&self.path, serde_json::to_vec(&self.threads).unwrap());
    }

    fn turn_mut(&mut self, thread: &str, turn: &str) -> Option<&mut Value> {
        self.threads
            .get_mut(thread)?
            .get_mut("turns")?
            .as_array_mut()?
            .iter_mut()
            .find(|t| t["id"] == turn)
    }
}

struct Shared {
    out: Out,
    store: Mutex<Store>,
    /// Server→client requests waiting for an answer.
    waiting: Mutex<HashMap<String, mpsc::Sender<Value>>>,
    /// Interrupt flags of running turns.
    interrupts: Mutex<HashMap<String, bool>>,
    wake: Condvar,
    ids: AtomicU64,
    home: PathBuf,
}

impl Shared {
    fn id(&self, prefix: &str) -> String {
        format!(
            "{prefix}-{}-{}",
            std::process::id(),
            self.ids.fetch_add(1, Ordering::SeqCst)
        )
    }

    fn interrupted(&self, turn: &str) -> bool {
        *self.interrupts.lock().unwrap().get(turn).unwrap_or(&false)
    }

    fn log_request(&self, method: &str) {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.home.join("fake-requests.log"))
        {
            let _ = writeln!(f, "{method}");
        }
    }
}

fn serve() {
    let home = PathBuf::from(std::env::var("CODEX_HOME").unwrap_or(".".into()));
    let framing = match std::env::var("IGNIS_FAKE_CODEX_FRAMING").as_deref() {
        Ok("split") => Framing::Split,
        Ok("merged") => Framing::Merged,
        _ => Framing::Plain,
    };
    let sh = Arc::new(Shared {
        out: Out {
            framing,
            lock: Mutex::new(()),
            opt_out: Mutex::new(vec![]),
        },
        store: Mutex::new(Store::load(home.join("fake-threads.json"))),
        waiting: Mutex::new(HashMap::new()),
        interrupts: Mutex::new(HashMap::new()),
        wake: Condvar::new(),
        ids: AtomicU64::new(1),
        home,
    });
    let (mut initialized, mut ready) = (false, false);
    for line in std::io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let method = msg
            .get("method")
            .and_then(Value::as_str)
            .map(str::to_string);
        let id = msg.get("id").cloned();
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        match (method, id) {
            // Answer to one of our requests.
            (None, Some(id)) => {
                let key = id.to_string();
                if let Some(tx) = sh.waiting.lock().unwrap().remove(&key) {
                    let _ = tx.send(msg.get("result").cloned().unwrap_or(Value::Null));
                }
            }
            (Some(m), None) => {
                if m == "initialized" && initialized {
                    ready = true;
                }
            }
            (Some(m), Some(id)) => {
                sh.log_request(&m);
                let err = |code: i64, text: &str| json!({"id": id, "error": {"code": code, "message": text}});
                if m == "initialize" {
                    if initialized {
                        sh.out.send(err(-32600, "Already initialized"));
                        continue;
                    }
                    initialized = true;
                    let opt: Vec<String> = params
                        .pointer("/capabilities/optOutNotificationMethods")
                        .and_then(Value::as_array)
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    *sh.out.opt_out.lock().unwrap() = opt;
                    sh.out.send(json!({"id": id, "result": {
                        "userAgent": format!("ignis-fake-codex/{PINNED_CODEX_VERSION}"),
                        "codexHome": sh.home.to_string_lossy(),
                        "platformFamily": std::env::consts::FAMILY,
                        "platformOs": std::env::consts::OS,
                    }}));
                    continue;
                }
                if !ready {
                    sh.out.send(err(-32002, "Not initialized"));
                    continue;
                }
                handle(&sh, &m, id, params);
            }
            (None, None) => {}
        }
    }
}

fn thread_json(t: &Value) -> Value {
    t.clone()
}

fn handle(sh: &Arc<Shared>, method: &str, id: Value, params: Value) {
    let ok = |result: Value| json!({"id": id, "result": result});
    let err = |code: i64, text: &str| json!({"id": id, "error": {"code": code, "message": text}});
    match method {
        "thread/start" => {
            let tid = sh.id("thr");
            let t = json!({"id": tid, "turns": [], "cwd": params["cwd"], "status": "idle",
                           "sandbox": params["sandbox"], "approvalPolicy": params["approvalPolicy"]});
            {
                let mut st = sh.store.lock().unwrap();
                st.threads.insert(tid.clone(), t.clone());
                st.save();
            }
            sh.out.send_all(&[
                ok(json!({"thread": thread_json(&t), "model": "fake", "modelProvider": "fake",
                          "cwd": params["cwd"], "approvalPolicy": params["approvalPolicy"], "sandbox": {"type": "readOnly"}, "approvalsReviewer": "user"})),
                json!({"method": "thread/started", "params": {"thread": thread_json(&t)}}),
            ]);
        }
        "thread/resume" => {
            let tid = params["threadId"].as_str().unwrap_or("");
            let t = sh.store.lock().unwrap().threads.get(tid).cloned();
            match t {
                Some(t) => sh.out.send(ok(json!({"thread": t, "model": "fake", "modelProvider": "fake",
                    "cwd": params["cwd"], "approvalPolicy": "on-request", "sandbox": {"type": "readOnly"}, "approvalsReviewer": "user"}))),
                None => sh.out.send(err(-32600, "no such thread")),
            }
        }
        "turn/start" => {
            let tid = params["threadId"].as_str().unwrap_or("").to_string();
            let text: String = params["input"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|i| i["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let turn_id = sh.id("turn");
            let turn = json!({"id": turn_id, "status": "inProgress", "items": [], "error": null});
            {
                let mut st = sh.store.lock().unwrap();
                let Some(t) = st.threads.get_mut(&tid) else {
                    drop(st);
                    sh.out.send(err(-32600, "no such thread"));
                    return;
                };
                t["turns"].as_array_mut().unwrap().push(turn.clone());
                st.save();
            }
            sh.interrupts.lock().unwrap().insert(turn_id.clone(), false);
            sh.out.send(ok(json!({"turn": turn})));
            let sh = sh.clone();
            std::thread::spawn(move || run_turn(&sh, &tid, &turn_id, &text));
        }
        "turn/interrupt" => {
            let turn = params["turnId"].as_str().unwrap_or("").to_string();
            sh.interrupts.lock().unwrap().insert(turn, true);
            sh.wake.notify_all();
            sh.out.send(ok(json!({})));
        }
        _ => sh.out.send(err(-32601, "method not found")),
    }
}

fn n(method: &str, params: Value) -> Value {
    json!({"method": method, "params": params})
}

/// Ask the client and wait for its answer.
fn ask(sh: &Shared, method: &str, params: Value) -> Option<Value> {
    let id = sh.id("srv");
    let (tx, rx) = mpsc::channel();
    sh.waiting.lock().unwrap().insert(json!(id).to_string(), tx);
    sh.out
        .send(json!({"id": id, "method": method, "params": params}));
    let answer = rx.recv_timeout(Duration::from_secs(60)).ok();
    sh.out.send(n(
        "serverRequest/resolved",
        json!({"requestId": id, "threadId": params["threadId"]}),
    ));
    answer
}

fn run_turn(sh: &Arc<Shared>, tid: &str, turn: &str, text: &str) {
    let has = |w: &str| text.contains(w);
    let base = |extra: Value| {
        let mut p = json!({"threadId": tid, "turnId": turn});
        if let (Some(p), Some(e)) = (p.as_object_mut(), extra.as_object()) {
            p.extend(e.clone());
        }
        p
    };
    let persist_item = |item: &Value| {
        let mut st = sh.store.lock().unwrap();
        if let Some(t) = st.turn_mut(tid, turn) {
            t["items"].as_array_mut().unwrap().push(item.clone());
        }
        st.save();
    };
    let mut burst = vec![n(
        "turn/started",
        json!({"threadId": tid, "turn": {"id": turn, "status": "inProgress", "items": []}}),
    )];
    let user = json!({"type": "userMessage", "id": sh.id("item"), "content": [{"type": "text", "text": text}]});
    persist_item(&user);
    burst.push(n(
        "item/started",
        base(json!({"item": user, "startedAtMs": 0})),
    ));
    burst.push(n(
        "item/completed",
        base(json!({"item": user, "completedAtMs": 0})),
    ));
    sh.out.send_all(&burst);

    let mut reply = String::from("Fixture answer");
    if has("opaque") {
        sh.out.send_all(&[
            n(
                "thread/tokenUsage/updated",
                base(json!({"tokenUsage": {"total": {"inputTokens": 10}}})),
            ),
            n(
                "x/unknownNotification",
                base(json!({"anything": [1, 2, 3]})),
            ),
            n("account/rateLimits/updated", json!({"rateLimits": {}})),
        ]);
    }
    if has("tool") {
        let call = sh.id("call");
        let started = json!({"type": "mcpToolCall", "id": call, "server": "ignis", "tool": "sources_search",
                             "status": "inProgress", "arguments": {"query": "flame spread"}});
        let tool_result = json!({
            "schema_version": "tools.v1", "call_id": call, "tool": "sources_search",
            "execution_status": "completed", "scientific_status": "not_assessed",
            "result": {"value": {"hits": 1}, "partial": false}, "artifact_refs": [], "origin": "synthetic_fixture"
        });
        let mut done = started.clone();
        done["status"] = json!("completed");
        done["result"] = json!({"content": [{"type": "text", "text": "1 hit"}], "structuredContent": tool_result});
        persist_item(&done);
        sh.out.send_all(&[
            n(
                "item/started",
                base(json!({"item": started, "startedAtMs": 0})),
            ),
            n(
                "item/completed",
                base(json!({"item": done, "completedAtMs": 0})),
            ),
        ]);
    }
    for (word, method, extra) in [
        (
            "approval",
            "item/fileChange/requestApproval",
            json!({"reason": "write notes.md"}),
        ),
        (
            "shell",
            "item/commandExecution/requestApproval",
            json!({"command": "ls /"}),
        ),
        (
            "permissions",
            "item/permissions/requestApproval",
            json!({"permissions": {"network": {"enabled": true}}, "cwd": "/"}),
        ),
    ] {
        if has(word) {
            let item = sh.id("item");
            let answer = ask(
                sh,
                method,
                base(json!({"itemId": item, "startedAtMs": 0}))
                    .as_object()
                    .map(|o| {
                        let mut o = o.clone();
                        o.extend(extra.as_object().unwrap().clone());
                        Value::Object(o)
                    })
                    .unwrap(),
            );
            let decision = answer
                .as_ref()
                .and_then(|a| {
                    a.get("decision")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .or_else(|| {
                    answer.as_ref().and_then(|a| a.get("permissions")).map(|p| {
                        if p.as_object().is_some_and(|o| o.is_empty()) {
                            "decline".into()
                        } else {
                            "accept".into()
                        }
                    })
                })
                .unwrap_or_else(|| "none".into());
            reply = format!("{method} -> {decision}");
        }
    }
    if has("hang") {
        sh.out.send(n(
            "item/agentMessage/delta",
            base(json!({"itemId": "msg-hang", "delta": "thinking"})),
        ));
        let mut flags = sh.interrupts.lock().unwrap();
        while !*flags.get(turn).unwrap_or(&false) {
            flags = sh.wake.wait(flags).unwrap();
        }
    }
    if sh.interrupted(turn) {
        finish(sh, tid, turn, "interrupted", None);
        return;
    }
    if has("fail") {
        finish(sh, tid, turn, "failed", Some("model refused the request"));
        return;
    }
    let msg_id = sh.id("msg");
    let item = json!({"type": "agentMessage", "id": msg_id, "text": ""});
    let mut burst = vec![n(
        "item/started",
        base(json!({"item": item, "startedAtMs": 0})),
    )];
    let words: Vec<String> = reply.split(' ').map(|w| format!("{w} ")).collect();
    for w in &words {
        burst.push(n(
            "item/agentMessage/delta",
            base(json!({"itemId": msg_id, "delta": w})),
        ));
    }
    let done = json!({"type": "agentMessage", "id": msg_id, "text": words.concat().trim_end()});
    burst.push(n(
        "item/completed",
        base(json!({"item": done, "completedAtMs": 0})),
    ));
    persist_item(&done);
    sh.out.send_all(&burst);
    finish(sh, tid, turn, "completed", None);
}

fn finish(sh: &Shared, tid: &str, turn: &str, status: &str, error: Option<&str>) {
    let snapshot = {
        let mut st = sh.store.lock().unwrap();
        let snap = st.turn_mut(tid, turn).map(|t| {
            t["status"] = json!(status);
            t["error"] = error.map_or(Value::Null, |e| json!({"message": e}));
            t.clone()
        });
        st.save();
        snap
    };
    if let Some(t) = snapshot {
        sh.out
            .send(n("turn/completed", json!({"threadId": tid, "turn": t})));
    }
}
