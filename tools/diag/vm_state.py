#!/usr/bin/env python3
"""Read-only diagnosis of the IGNIS origin state. Run ON the VM (see tools/diag/vm.sh):
   sudo python3 vm_state.py [episode_id]
Lists episodes that never started, turns still open, tool calls that never finished, and the
tool-call outcome mix per episode. Opens the SQLite state read-only."""
import json, sqlite3, sys, collections
DB = "file:/srv/ignis/state/state.sqlite?mode=ro"
c = sqlite3.connect(DB, uri=True)
only = sys.argv[1] if len(sys.argv) > 1 else None
turns = {r[0]: r[1:] for r in c.execute("select episode_id,state,outcome,through_seq from runtime_turns where state!='finished'")}
for eid, rec in c.execute("select episode_id, record from episodes" + (" where episode_id=?" if only else ""), (only,) if only else ()):
    ep = json.loads(rec)
    ev = [(s, k, json.loads(r)) for s, k, r in c.execute("select seq,kind,record from events where episode_id=? order by seq", (eid,))]
    kinds = collections.Counter(k for _, k, _ in ev)
    started, finished, errs = {}, set(), []
    for s, k, r in ev:
        b = r["body"]["payload"]
        if k == "tool.started": started[b["call_id"]] = (s, b["tool"])
        if k == "tool.finished":
            res = b["result"]; finished.add(res.get("call_id"))
            st = json.dumps({x: res.get(x) for x in ("status", "error", "refusal") if res.get(x)})
            if res.get("error") or "error" in st or "refus" in st: errs.append((s, res.get("tool"), st[:160]))
    flags = []
    if ep["last_seq"] == 0: flags.append("NEVER STARTED (no message.user)")
    if eid in turns: flags.append(f"OPEN TURN {turns[eid]}")
    for cid, (s, t) in started.items():
        if cid not in finished: flags.append(f"tool never finished: {t} seq {s}")
    if ep["status"] in ("running", "ready") and ev and ep["status"] == "running": flags.append("status running: check last event age")
    print(f"{eid} {ep['status']:<10} seq={ep['last_seq']:<4} {ep['updated_at']} {ep['goal'][:50]!r}")
    print(f"   tools={len(started)} finished={len(finished)} errs={len(errs)} kinds={dict(kinds.most_common(5))}")
    for e in errs[:5]: print("   ERR", e)
    for f in flags: print("   !!", f)
