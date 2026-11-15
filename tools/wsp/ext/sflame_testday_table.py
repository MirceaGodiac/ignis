#!/usr/bin/env python3
"""Per-test table of the helium-diluted s-Flame flames for the revision of PROTOCOL 13.8: the test days 20142, 20170,
20176 (study data since the final run; endpoints version 2 as the final step built them) and, for reference, the
eight ethylene-in-helium flames of validation day 20164 the helium row of F4 was typed from.

    python tools/wsp/ext/sflame_testday_table.py [--release sha256:...] [--out docs/wsp/feedback/sflame_testday_helium.md]

Per test: fuel flow, fuel fraction, the oxygen candidates with their adiabatic temperature, the script's burner
limit, the observed end (version 2: cause, time, the rule's branch; and version 1's label and time), and what the
release forecasts at cutoff 1 s without the planned stop, once per oxygen candidate (the prefix with that candidate
as the only oxygen value): the median end time (the first time the probability of a burning flame falls to 0.5),
the probability of extinction and of operator termination by 120 s, and the central times of the two clocks of the
members that are run (the hinge member's radiative clock, the growth member's growth-clock median, the burner
heating law's time to the shifted limit). With two candidates the release answers the mixture (weights 0.5); its
median is given beside them. Inputs, outcomes and forecasts of single tests; for the author of the revision.
"""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import sflame_cases as sc  # noqa: E402
import transfer_acme as ta  # noqa: E402

REPO = ta.REPO
FINAL = REPO / "runs/final/gen7/final/transfer_sflame"
F4_HELIUM = "sha256:7fb9f31fbe14988149553118adb05423537511cdedf0bc3485b0e25f6b07fbef"
TIMES = [round(1.0 + 0.25 * i, 2) for i in range(477)]  # 1 to 120 s
WORK = REPO / "runs/dev/gen7/testday_table"


def default_binary(name):
    merged = REPO / "target-merged/release" / name
    return merged if merged.exists() else REPO / "target/release" / name


def median_end(a):
    t, act = a["times_s"], a["event_cdf"][0]
    for i, p in enumerate(act):
        if p <= 0.5:
            if i == 0:
                return t[0]
            return t[i - 1] + (t[i] - t[i - 1]) * (act[i - 1] - 0.5) / (act[i - 1] - p)
    return None


def clocks(a):
    out = {}
    for m in (a.get("diagnostics") or {}).get("members") or []:
        cl = m.get("clocks") or {}
        cl = cl.get("clocks", cl)
        slot = m["slot"].rstrip(".")
        lim = [t for (_l, w, t) in cl.get("burner_limits") or [] if w > 0 and t is not None]
        out[slot] = dict(radiative=cl.get("radiative_tau_s"), weak=cl.get("p_weak"), phi=cl.get("phi"),
                         limit=min(lim) if lim else None, t1=(cl.get("inputs") or {}).get("t1_k"))
    return out


def forecast(rel, c, src_rel, row, tag, cand):
    p = sc.f3_prefix(c, 1.0, False, "table", row, src_rel)
    kc = p["known_conditions"]
    if cand not in (None, "mix"):  # one candidate alone
        for k in ("x_o2_role_b", "tad_b", "zst_b"):
            kc.pop(k, None)
        for key, v in (("x_o2_role", float(cand[0])), ("tad", cand[1]["tad"]), ("zst", cand[1]["zst"])):
            m = dict(kc.get(key) or kc["pressure"])
            m.update(raw_value=v, value_si=v, raw_unit="1" if key != "tad" else "K", unit_si="1" if key != "tad" else "K")
            kc[key] = m
    p = ta.finish_prefix(p)
    pp = WORK / tag / "prefix" / f"{c['test']}_{'mix' if cand == 'mix' else cand[0] if cand else 'one'}.json"
    ta.write_json(pp, p)
    return ta.run_forward(rel, pp, WORK / tag / "artifact" / rel[7:19] / pp.name)


def fmt(x, nd=1):
    return "–" if x is None else f"{x:.{nd}f}"


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--release", default=F4_HELIUM)
    ap.add_argument("--ignis", default=str(default_binary("ignis.exe")))
    ap.add_argument("--out", default=str(REPO / "docs/wsp/feedback/sflame_testday_helium.md"))
    a = ap.parse_args()
    ta.EXE = Path(a.ignis).resolve()
    ta.SETTINGS = dict(ta.SETTINGS, times_s=TIMES)
    sets = []
    # the test days, as the final step built them; version 2 endpoints and version 1 beside
    v2 = {r["test"]: r for r in sc.read_jsonl(FINAL / "endpoints_v2.jsonl")}
    v1 = {r["test"]: r for r in sc.read_jsonl(FINAL / "cases_validation_outcomes.jsonl")}
    src = (FINAL / "cases_validation_inputs.jsonl").relative_to(REPO).as_posix()
    rows = sc.read_jsonl(FINAL / "cases_validation_inputs.jsonl")
    sets.append(("test days (20142, 20170, 20176)", src, rows, v2, v1, "test"))
    # the validation day the row was typed from
    vsrc = "runs/dev/data_all/work/transfer_sflame/cases_validation_inputs.jsonl"
    vrows = [c for c in sc.read_jsonl(REPO / vsrc) if c["day"] == "20164"]
    vv2 = {r["test"]: r for r in sc.read_jsonl(sc.OUT / "cases_validation_outcomes_v2.jsonl")}
    vv1 = {r["test"]: r for r in sc.read_jsonl(sc.OUT / "cases_validation_outcomes.jsonl")}
    sets.append(("validation day 20164 (what the helium row was typed from)", vsrc, vrows, vv2, vv1, "val"))
    L = ["# Helium-diluted s-Flame flames, test by test (PROTOCOL 13.8: every partition seen)", "",
         f"Release `{a.release}` (F4 stage 2, helium). Forecasts at cutoff 1 s, without the planned stop; per oxygen "
         "candidate the release is run with that candidate alone; \"mix\" is what it answers (weights 0.5). Columns: "
         "median end (s), P(extinction by 120 s) / P(operator termination by 120 s), and the central clocks of the "
         "members run (hinge c0: radiative median after the floor; growth b0: growth median, `held` when none; "
         "the heating law's time to the shifted limit). Observed: version 2 (the final step's coding), version 1 "
         "beside it. On a \"mix\" row the median and the probabilities are the mixture's; the clocks are the first "
         "candidate's (the diagnostics carry the first candidate's members). Hydrogen-bearing blends are refused by "
         "this release (hydrogen not admitted). Generated by `tools/wsp/ext/sflame_testday_table.py`.", ""]
    for title, src_rel, rows, e2, e1, tag in sets:
        L += [f"## {title}", "",
              "| test | fuel | flow mg/s | x_fuel | O2 candidates (tad K) | limit °C | observed v2 (s) | v1 | candidate | median end s | P ext / P term | c0 radiative s | b0 growth s | limit at s | T_1 K |",
              "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"]
        for i, c in enumerate(rows):
            if not (c["in_target_set"] and sc.is_helium(c)):
                continue
            inp = c["inputs"]
            cands = sorted((c.get("inputs_by_o2_candidate") or {}).items(), key=lambda kv: float(kv[0]))
            cand_txt = ", ".join(f"{float(x):.2f} ({v['tad']:.0f})" for x, v in cands)
            r2, r1 = e2.get(c["test"]) or {}, e1.get(c["test"]) or {}
            obs2 = f"{(r2.get('label') or '–').replace('operator_termination', 'stop')} {fmt(r2.get('end_s'))}"
            obs1 = f"{(r1.get('label_sensor') or r1.get('label') or '–').replace('operator_termination', 'stop')} {fmt(r1.get('end_s'))}"
            runs = [(f"{float(x):.2f}", (x, v)) for x, v in cands] if len(cands) > 1 else [(f"{float(cands[0][0]):.2f}" if cands else "–", None)]
            if len(cands) > 1:
                runs.append(("mix", "mix"))
            first = True
            for label, cand in runs:
                art = forecast(a.release, c, src_rel, i + 1, tag, cand)
                if ta.admitted(art):
                    ext, term = art["event_cdf"][1][-1], art["event_cdf"][2][-1]
                    ck = clocks(art)
                    c0, b0 = ck.get("c0", {}), ck.get("b0", {})
                    pred = (f"{fmt(median_end(art))} | {ext:.2f} / {term:.2f} | {fmt(c0.get('radiative'))} | "
                            f"{'held' if b0 and b0.get('radiative') is None else fmt(b0.get('radiative'))} | "
                            f"{fmt(b0.get('limit') or c0.get('limit'))} | {fmt(b0.get('t1') or c0.get('t1'), 0)}")
                else:
                    pred = "refused: " + "; ".join(art["support"].get("reasons", []))[:90] + " | | | | |"
                head = (f"| {c['test']} | {inp['fuel_species']} | {inp['injected_mass_flow'] * 1e6:.2f} | {inp['x_fuel_role']:.2f} | "
                        f"{cand_txt} | {fmt(c.get('burner_limit_c'), 0)} | {obs2} | {obs1} |") if first else "| | | | | | | | |"
                L.append(f"{head} {label} | {pred} |")
                first = False
        L.append("")
    Path(a.out).write_text("\n".join(L) + "\n", encoding="utf-8", newline="\n")
    print(f"wrote {Path(a.out).relative_to(REPO).as_posix()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
