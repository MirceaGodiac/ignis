"""ACME: full TRAIN table (conditions, endpoint, post facts incl. cool flame and comments, timelines).

usage: p2_acme_table.py <acme_normal|acme_inverse> <out.md>
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import lib as p2lib

fam, out = sys.argv[1], Path(sys.argv[2])
ids = p2lib.partition_ids(fam, "train")
recs = p2lib.experiments(fam, ids)
grp = p2lib.groups(fam)
ends = p2lib.jsonl_for(fam, "endpoints.jsonl", ids)
post = p2lib.jsonl_for(fam, "post.jsonl", ids)
ctl = p2lib.jsonl_for(fam, "extra_control_history.jsonl", ids)
cond_keys = sorted({k for r in recs.values() for k in r["conditions"]})
fact_keys = sorted({k for rows in post.values() for r in rows for k in r["facts"]})
print("condition keys", cond_keys)
print("post fact keys", fact_keys)


def cv(r, k):
    c = r["conditions"].get(k)
    if c is None:
        return ""
    v = c.get("value_si")
    if v is None:
        v = c.get("raw_value")
    if isinstance(v, float):
        return "%.4g" % v
    return str(v)


def fv(rid, k):
    rows = post.get(rid, [])
    if not rows:
        return ""
    f = rows[0]["facts"].get(k)
    if f is None:
        return ""
    v = f.get("raw_value")
    return "" if v is None else str(v)


order = sorted(ids, key=lambda r: (grp[r], r))
md = [f"# {fam} TRAIN ({len(ids)} runs)", ""]
short = [k for k in cond_keys if k not in ("x_fuel_role", "x_o2_role", "fuel_bottle")]
md.append("| run | session | " + " | ".join(short) + " | label | active_until | terminal_from | censoring | " + " | ".join(k for k in fact_keys if k != "comments") + " |")
md.append("|" + "---|" * (len(short) + 6 + len(fact_keys) - 1 + 0))
for rid in order:
    r = recs[rid]
    e = ends.get(rid, [{}])[0]
    md.append("| " + " | ".join([rid.split(":")[-1], grp[rid].split(":")[-1]] + [cv(r, k) for k in short] + [
        str(e.get("original_label")), str(e.get("last_confirmed_active_s")), str(e.get("first_terminal_s")),
        str(e.get("censoring"))] + [fv(rid, k) for k in fact_keys if k != "comments"]) + " |")
md.append("")
md.append("## Comments")
for rid in order:
    c = fv(rid, "comments")
    e = ends.get(rid, [{}])[0]
    extra = ""
    if e.get("original_comment") or e.get("intervention_kind") or e.get("observation_state"):
        extra = f" [endpoint: comment={e.get('original_comment')!r} intervention={e.get('intervention_kind')}@{e.get('intervention_s')} state={e.get('observation_state')}]"
    if c or extra:
        md.append(f"- **{rid.split(':')[-1]}** ({fv(rid, 'burn_time')} s, {e.get('original_label')}): {c}{extra}")
md.append("")
md.append("## Control timelines (runs with a TP file)")
for rid in order:
    for row in ctl.get(rid, []):
        st = "; ".join(f"{s['t_start_s']:g}s {s['control']}" + (" [op]" if s.get("operator_action") else "") for s in row["stages"])
        md.append(f"- **{rid.split(':')[-1]}**: {st}")
out.write_text("\n".join(md), encoding="utf-8")
print("written", out)
