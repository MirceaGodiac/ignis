"""Does the flame go out by itself? Irace et al.'s R > 1 against F2's forecast, per CFI-G TRAIN run.

Post-hoc, 15 November 2026. Nothing is adopted or refused. Only train runs: F2's CFI-G forecasts on disk are the
chain-check forecasts of PROTOCOL 12.10 on train (runs/dev/data_all/work/transfer_acme/fd_f2/cfig/artifact);
validation and test forecasts are not on disk in readable form and none is made here.

Exposure, stated plainly: F2 was typed by an author who could read CFI-G train, so this compares a published law
(no CFI-G data in it) with a data-informed release on the data its author saw. It says how the published law
does on these flames; it cannot show that F2 generalises better. The test-day comparison of
posthoc_irace_testdays.py (Flame Design) is the out-of-sample one.

Labels and R: runs/final/20261115/local/posthoc/kim_comparator/<fam>_occurrence_trainval.json
(posthoc_kim_comparator.py: SE = self-extinguished, FT = flow terminated; R of Irace et al. Eq. 6.9).
F2 score: P(extinction comes first) = CIF_ext / (CIF_ext + CIF_term) at 120 s, earliest cutoff (1 s).

    python tools/wsp/final/posthoc_irace_cfig_occurrence.py
"""
import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[3]
ART = ROOT / "runs/dev/data_all/work/transfer_acme/fd_f2/cfig/artifact"
KIM = ROOT / "runs/final/20261115/local/posthoc/kim_comparator"
OUT = ROOT / "runs/final/20261115/posthoc_irace_cfig_occurrence.json"


def auc(pos, neg):
    if not pos or not neg:
        return None
    s = sum((p > n) + 0.5 * (p == n) for p in pos for n in neg)
    return s / (len(pos) * len(neg))


def main():
    res = {"what": __doc__.strip().splitlines()[0], "families": {}}
    for fam in ("acme_normal", "acme_inverse"):
        rows = [r for r in json.loads((KIM / f"{fam}_occurrence_trainval.json").read_text(encoding="utf-8"))
                if r["partition"] == "train"]
        got = []
        for r in rows:
            test = r["run"].split(":")[-1]
            f = ART / fam / "f2" / f"{test}_c1.json"
            if not f.exists():
                continue
            a = json.loads(f.read_text(encoding="utf-8"))
            cat = a["event_categories"]
            ce, ct = a["event_cdf"][cat.index("extinction")][-1], a["event_cdf"][cat.index("operator_termination")][-1]
            p = ce / (ce + ct) if ce + ct > 0 else 0.5
            got.append(dict(run=r["run"], y=r["label"] == "SE", p_f2=p, R=r["R_published"]))
        y = [g["y"] for g in got]
        n = len(got)
        base = sum(y) / n
        b = dict(runs=n, SE=sum(y), FT=n - sum(y), not_matched=len(rows) - n,
                 always_SE_accuracy=max(base, 1 - base),
                 f2=dict(auc=auc([g["p_f2"] for g in got if g["y"]], [g["p_f2"] for g in got if not g["y"]]),
                         accuracy=sum((g["p_f2"] > 0.5) == g["y"] for g in got) / n,
                         brier=sum((g["p_f2"] - g["y"]) ** 2 for g in got) / n),
                 irace_R=dict(auc=auc([g["R"] for g in got if g["y"]], [g["R"] for g in got if not g["y"]]),
                              accuracy=sum((g["R"] > 1) == g["y"] for g in got) / n,
                              brier_hard=sum(((g["R"] > 1) - g["y"]) ** 2 for g in got) / n,
                              frac_R_gt_1=sum(g["R"] > 1 for g in got) / n))
        res["families"][fam] = b
    OUT.write_text(json.dumps(res, indent=1), encoding="utf-8")
    print(json.dumps(res, indent=1))


if __name__ == "__main__":
    main()
