#!/usr/bin/env python3
"""Harness benchmark: the same model alone and inside IGNIS, graded blind by a committee of jurors.

Steps (each writes JSONL under --work, default benchmarks/harness/work/; nothing else is touched):

    python benchmarks/harness/bench.py raw     --label "GPT-6 Luna" --runtime codex  --model gpt-6-luna
    python benchmarks/harness/bench.py raw     --label "Claude Fable 5.2" --runtime claude --model claude-fable-5-2
    python benchmarks/harness/bench.py collect --label "IGNIS v2 · GPT-6 Luna" --run <ignis harness run --out dir>
    python benchmarks/harness/bench.py judge   --jurors claude:claude-opus-5-5 claude:claude-fable-5-2 codex:gpt-6.1-sol
    python benchmarks/harness/bench.py score   [--prices configs/pricing/llm_prices.toml] --out results.md

`raw`     asks every question of tasks.json once, no tools, with prompts/raw.md.
`collect` reads the answers, tokens and cost of an `ignis harness run` output directory.
`judge`   shuffles every (question, answer) pair with a fixed seed, hides the system, and asks each
          juror with prompts/juror.md. Labels: correct 1, partly_correct 0.5, incorrect 0, abstained 0.
`score`   majority vote per answer (a three-way split is left out), score over all questions, 95 %
          bootstrap interval over questions, score per kind, precision over answered questions,
          abstention share, cost and latency per question, answers per dollar, unanimity.

Juror and raw prompts here were rewritten from METHOD.md for reruns; see README.md.
"""
import argparse
import json
import random
import re
import sqlite3
import statistics
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
VALUE = {"correct": 1.0, "partly_correct": 0.5, "incorrect": 0.0, "abstained": 0.0}
# OpenAI standard tier, short context, USD per million tokens (input, cached input, output), read
# 2026-11-15 (results/COSTS.md). Used for raw Codex runs, whose CLI reports tokens but no cost.
PRICES = {"gpt-6-astra": (10.00, 1.00, 50.00), "gpt-6.1-sol": (2.00, 0.10, 10.00),
          "gpt-6.1-terra": (1.00, 0.05, 5.00), "gpt-6-sol": (2.00, 0.20, 10.00), "gpt-6-luna": (0.10, 0.01, 0.50),
          "gpt-5.6-sol": (4.00, 0.40, 20.00), "gpt-5.6-terra": (2.00, 0.20, 12.00), "gpt-5.6-luna": (0.20, 0.02, 1.20)}


def list_cost(model, usage):
    if model not in PRICES or not usage:
        return None
    pi, pc, po = PRICES[model]
    cached = usage.get("cached_input_tokens", 0) or 0
    fresh = (usage.get("input_tokens", 0) or 0) - cached
    return (fresh * pi + cached * pc + (usage.get("output_tokens", 0) or 0) * po) / 1e6


def jl_read(p):
    p = Path(p)
    return [json.loads(x) for x in p.read_text(encoding="utf-8").splitlines() if x.strip()] if p.exists() else []


def jl_append(p, row):
    p.parent.mkdir(parents=True, exist_ok=True)
    with open(p, "a", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(row, ensure_ascii=False) + "\n")


def tasks():
    return json.loads((HERE / "tasks.json").read_text(encoding="utf-8"))["tasks"]


def refs():
    return {r["task_id"]: r for r in json.loads((HERE / "references.json").read_text(encoding="utf-8"))["references"]}


def ask(runtime, model, prompt, timeout=600):
    """One headless call. Returns (text, usage dict)."""
    t0 = time.time()
    if runtime == "claude":
        r = subprocess.run(["claude", "-p", prompt, "--model", model, "--output-format", "json"],
                           capture_output=True, text=True, encoding="utf-8", timeout=timeout)
        out = json.loads(r.stdout)
        return out.get("result", ""), {"latency_s": time.time() - t0, "cost_usd": out.get("total_cost_usd"),
                                       "cost_basis": "reported", "usage": out.get("usage")}
    if runtime == "codex":
        r = subprocess.run(["codex", "exec", "-m", model, "--json", prompt],
                           capture_output=True, text=True, encoding="utf-8", timeout=timeout)
        text, usage = "", None
        for line in r.stdout.splitlines():
            try:
                ev = json.loads(line)
            except ValueError:
                continue
            item = ev.get("item") or {}
            if item.get("type") in ("agent_message", "assistant_message") and item.get("text"):
                text = item["text"]
            if ev.get("usage"):
                usage = ev["usage"]
        return text, {"latency_s": time.time() - t0, "usage": usage, "cost_basis": "list"}
    raise SystemExit(f"unknown runtime {runtime}")


def cmd_raw(a):
    out = Path(a.work) / "answers.jsonl"
    done = {(r["system"], r["task_id"]) for r in jl_read(out)}
    tmpl = (HERE / "prompts/raw.md").read_text(encoding="utf-8")
    for t in tasks():
        if (a.label, t["task_id"]) in done:
            continue
        text, meta = ask(a.runtime, a.model, tmpl.replace("{question}", t["question"]))
        if meta.get("cost_usd") is None:
            meta["cost_usd"] = list_cost(a.model, meta.get("usage"))
        jl_append(out, {"system": a.label, "type": "raw model", "model": a.model, "task_id": t["task_id"],
                        "answer": text, **meta})
        print(a.label, t["task_id"], f"{meta['latency_s']:.1f} s")


def _answer_texts(state_dir):
    """episode_id -> answer text, from any JSON value in the run's state that is a ResearchAnswer."""
    found = {}

    def take(s):
        if '"blocks"' not in s or '"answer_id"' not in s:
            return
        try:
            v = json.loads(s)
        except ValueError:
            return
        if isinstance(v, dict) and "blocks" in v and "episode_id" in v:
            text = "\n\n".join(b.get("text", "") for b in v["blocks"] if isinstance(b, dict) and b.get("text"))
            found[v["episode_id"]] = text
    for db in list(Path(state_dir).rglob("*.sqlite*")) + list(Path(state_dir).rglob("*.db")):
        try:
            con = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
            for (tbl,) in con.execute("select name from sqlite_master where type='table'"):
                for row in con.execute(f'select * from "{tbl}"'):
                    for c in row:
                        if isinstance(c, (str, bytes)):
                            take(c.decode("utf-8", "ignore") if isinstance(c, bytes) else c)
            con.close()
        except sqlite3.Error:
            continue
    for p in Path(state_dir).rglob("*.json"):
        take(p.read_text(encoding="utf-8", errors="ignore"))
    return found


def cmd_collect(a):
    run = Path(a.run)
    summary = json.loads((run / "summary.json").read_text(encoding="utf-8"))
    texts = _answer_texts(run / "state")
    out = Path(a.work) / "answers.jsonl"
    for r in summary["runs"]:
        trace = json.loads((run / r["trace_file"]).read_text(encoding="utf-8")) if r.get("trace_file") else {}
        cost = (r.get("cost") or {}).get("cost") or {}
        tok = r.get("tokens") or {}
        jl_append(out, {"system": a.label, "type": "IGNIS", "model": a.model, "task_id": r["task_id"],
                        "answer": texts.get(r["episode_id"], ""), "latency_s": trace.get("wall_time_s"),
                        "cost_usd": cost.get("amount"), "cost_basis": cost.get("basis"),
                        "output_tokens": tok.get("output_tokens"), "tool_calls": r.get("tool_calls"),
                        "harness": (summary.get("harness") or {}), "outcome": r.get("outcome")})
        if r["episode_id"] not in texts:
            print("no answer text found for", r["task_id"], r["episode_id"], file=sys.stderr)
    print(len(summary["runs"]), "answers of", a.label)


def cmd_judge(a):
    answers, rf = jl_read(Path(a.work) / "answers.jsonl"), refs()
    rng = random.Random(a.seed)
    order = list(range(len(answers)))
    rng.shuffle(order)
    tmpl = (HERE / "prompts/juror.md").read_text(encoding="utf-8")
    out = Path(a.work) / "labels.jsonl"
    done = {(r["juror"], r["system"], r["task_id"]) for r in jl_read(out)}
    for spec in a.jurors:
        runtime, model = spec.split(":", 1)
        for i in order:
            x = answers[i]
            if (model, x["system"], x["task_id"]) in done:
                continue
            ref = rf[x["task_id"]]
            prompt = (tmpl.replace("{question}", ref["question"]).replace("{reference}", ref["reference"])
                      .replace("{answer}", x["answer"] or "(no answer)"))
            text, _ = ask(runtime, model, prompt)
            m = re.search(r"\{.*\}", text, re.S)
            label = json.loads(m.group(0)).get("label") if m else None
            if label not in VALUE:
                print("unparsed juror reply", model, x["task_id"], file=sys.stderr)
                continue
            jl_append(out, {"juror": model, "system": x["system"], "task_id": x["task_id"], "label": label})


def majority(labels):
    for lab in set(labels):
        if labels.count(lab) * 2 > len(labels):
            return lab
    return None


def boot(vals, n=2000, seed=0):
    rng = random.Random(seed)
    ms = sorted(statistics.mean(rng.choice(vals) for _ in vals) for _ in range(n))
    return ms[int(0.025 * n)], ms[int(0.975 * n) - 1]


def cmd_score(a):
    answers, labels, rf = jl_read(Path(a.work) / "answers.jsonl"), jl_read(Path(a.work) / "labels.jsonl"), refs()
    by = {}
    for l in labels:
        by.setdefault((l["system"], l["task_id"]), []).append(l["label"])
    unanimous = sum(len(set(v)) == 1 for v in by.values()) / max(1, len(by))
    systems = sorted({x["system"] for x in answers})
    rows = []
    for s in systems:
        xs = [x for x in answers if x["system"] == s]
        per, kinds = {}, {}
        for x in xs:
            m = majority(by.get((s, x["task_id"]), []))
            if m is not None:
                per[x["task_id"]] = m
        vals = [VALUE[m] for m in per.values()]
        answered = [VALUE[m] for m in per.values() if m != "abstained"]
        for t, m in per.items():
            kinds.setdefault(rf[t]["kind"], []).append(VALUE[m])
        costs = [x["cost_usd"] for x in xs if x.get("cost_usd") is not None]
        lat = [x["latency_s"] for x in xs if x.get("latency_s") is not None]
        cpq = statistics.mean(costs) if costs else None
        lo, hi = boot(vals) if vals else (0, 0)
        rows.append({"system": s, "type": xs[0].get("type"), "score": 100 * statistics.mean(vals) if vals else 0,
                     "ci": (100 * lo, 100 * hi),
                     "kinds": {k: 100 * statistics.mean(v) for k, v in kinds.items()},
                     "precision": 100 * statistics.mean(answered) if answered else 0,
                     "abstained": 100 * sum(m == "abstained" for m in per.values()) / max(1, len(per)),
                     "cpq": cpq, "per_dollar": (1 / cpq) if cpq else None,
                     "p50": statistics.median(lat) if lat else None})
    rows.sort(key=lambda r: -r["score"])
    md = [f"Committee unanimous on {100 * unanimous:.0f}% of answers.", "",
          "| # | System | Type | Score | 95% CI | Concepts | NASA records | Traps | Precision | Abstained | Cost / question | Answers per $ | Latency p50 |",
          "|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
    for i, r in enumerate(rows, 1):
        k = r["kinds"]
        md.append(f"| {i} | {r['system']} | {r['type']} | **{r['score']:.0f}** | {r['ci'][0]:.0f}–{r['ci'][1]:.0f} | "
                  f"{k.get('concept', 0):.0f} | {k.get('record', 0):.0f} | {k.get('trap', 0):.0f} | {r['precision']:.0f} | "
                  f"{r['abstained']:.0f}% | " + (f"${r['cpq']:.4f}" if r["cpq"] else "n/a") + " | "
                  + (f"{r['per_dollar']:.0f}" if r["per_dollar"] else "n/a") + " | "
                  + (f"{r['p50']:.1f} s" if r["p50"] else "n/a") + " |")
    text = "\n".join(md) + "\n"
    if a.out:
        Path(a.out).write_text(text, encoding="utf-8", newline="\n")
    print(text)


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", default=str(HERE / "work"))
    sp = ap.add_subparsers(dest="cmd", required=True)
    p = sp.add_parser("raw"); p.add_argument("--label", required=True); p.add_argument("--runtime", required=True,
                                                                                       choices=["codex", "claude"])
    p.add_argument("--model", required=True)
    p = sp.add_parser("collect"); p.add_argument("--label", required=True); p.add_argument("--run", required=True)
    p.add_argument("--model")
    p = sp.add_parser("judge"); p.add_argument("--jurors", nargs="+", required=True); p.add_argument("--seed", type=int,
                                                                                                     default=20261115)
    p = sp.add_parser("score"); p.add_argument("--out")
    a = ap.parse_args()
    {"raw": cmd_raw, "collect": cmd_collect, "judge": cmd_judge, "score": cmd_score}[a.cmd](a)


if __name__ == "__main__":
    main()
