"""Freeze FLEX-2 image batch 2 (train days only) from the catalogue's conditions; nothing is downloaded.

    python -I tools/wsp/media/freeze_flex2_batch2.py   # writes docs/wsp/atlas/splits/PSI-68-batch2.json

Rule (conditions and file sizes only; no frame and no outcome is read):
  - a rack test point of the per-test catalogue on a TRAIN day of PSI-68-days.json, not in batch 1, with a
    backlit archive, a pure fuel, and the configuration "single free droplet";
  - every such test of the fuels with a hand-written droplet model (n-decane, n-octane) is taken, and every
    such test of the pure fuels without a model (propylbenzene, toluene); n-heptane and iso-octane have no
    such test left on train days (batch 1 took them all);
  - tiers, which order the download and are kept in the file: A inside the forward's domain with a measured
    ambient; C a measured ambient outside the domain (helium diluent); B ambient not measured (no gas reading
    of the day: x_o2, x_co2, x_he unknown); D pure fuel without a model;
  - excluded: mixtures, the 26 points whose fuel is "iso-octane | n-octane" (not resolved), droplet arrays,
    imposed flow, fibre-supported points, validation and test days.
"""
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
DAYS = REPO / "docs/wsp/atlas/splits/PSI-68-days.json"
B1 = REPO / "docs/wsp/atlas/splits/PSI-68-batch1.json"
CAT = REPO / "runs/dev/data_all/work/atlas/flex2_tests.jsonl"
OUT = REPO / "docs/wsp/atlas/splits/PSI-68-batch2.json"
FUELS = ["n-decane", "n-octane", "propylbenzene", "toluene"]
TIER_ORDER = {"A": 0, "C": 1, "B": 2, "D": 3}


def tier(r):
    if not r.get("candidate"):
        return "D"
    if r.get("unknown_features"):
        return "B"
    if r.get("outside_domain"):
        return "C"
    return "A" if r.get("inside_domain") else "B"


def expected(r):
    """The loss expected from batch 1 (by conditions only), as a note."""
    a = r.get("ambient") or {}
    p = (r.get("pressure_pa") or 0) / 101325
    notes = []
    if r["fuel"] == "n-decane" and p > 1.2:
        notes.append("elevated pressure: 0 of 9 batch-1 n-decane burns above 1.2 atm were usable (soot took the track)")
    if r["fuel"] == "n-decane" and (a.get("O2") or 0) >= 0.2 and p <= 1.2:
        notes.append("air at 1 atm: 3 of 11 batch-1 n-decane burns usable")
    if r["fuel"] == "n-octane":
        notes.append("1 of 12 batch-1 n-octane burns in air usable (soot)")
    if r["fuel"] in ("propylbenzene", "toluene"):
        notes.append("aromatic: heavy soot expected, most burns likely lost")
    if r["test_id"].startswith("A307"):
        notes.append("same day as A307D01/02, which showed a fibre on the frames although the settings say none")
    if tier(r) == "B":
        notes.append("ambient unknown: a series is measurable but the releases refuse an unknown condition")
    return notes


def main():
    days = json.loads(DAYS.read_text(encoding="utf-8"))
    train = set(days["groups"]["train"])
    b1 = {t["test_id"] for t in json.loads(B1.read_text(encoding="utf-8"))["tests"]}
    tests = []
    for line in CAT.open(encoding="utf-8"):
        r = json.loads(line)
        if (r.get("group") in train and r["test_id"] not in b1 and r.get("backlit_archive_bytes")
                and r.get("fuel") in FUELS and r.get("fuel_pure")
                and str(r.get("configuration", "")).startswith("single free droplet")):
            a = r.get("ambient") or {}
            tests.append(dict(
                test_id=r["test_id"], date=r["group"], partition="train", fuel=r["fuel"], candidate=r.get("candidate"),
                tier=tier(r), archive=f"{r['test_id']}A.zip", bytes=r["backlit_archive_bytes"],
                conditions=dict(x_o2=a.get("O2"), x_co2=a.get("CO2"), x_he=a.get("He"), x_n2=a.get("N2"),
                                pressure_pa=r.get("pressure_pa"), ambient_how=r.get("ambient_how")),
                expected=expected(r)))
    tests.sort(key=lambda t: (TIER_ORDER[t["tier"]], FUELS.index(t["fuel"]), t["date"], t["test_id"]))
    total = sum(t["bytes"] for t in tests)
    doc = dict(
        schema="ignis.atlas.batch/1", investigation_id="PSI-68", frozen_on="2026-11-15", batch="batch2",
        rule=(__doc__.split("Rule")[1].split('"""')[0]).strip(),
        catalogue=str(CAT.relative_to(REPO)), days=str(DAYS.relative_to(REPO)),
        bytes=total, expected_download="%.1f GB" % (total / 1e9),
        counts={f: {t_: sum(1 for t in tests if t["fuel"] == f and t["tier"] == t_) for t_ in "ACBD"} for f in FUELS},
        tests=tests)
    OUT.write_text(json.dumps(doc, indent=1) + "\n", encoding="utf-8")
    print(json.dumps({k: doc[k] for k in ("bytes", "expected_download", "counts")}, indent=1), len(tests))


if __name__ == "__main__":
    main()
