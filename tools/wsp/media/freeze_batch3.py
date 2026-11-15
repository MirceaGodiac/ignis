"""Freeze droplet batch 3 (FLEX-2 PSI-68 and CFI PSI-39) from conditions and file sizes only; nothing is fetched.

    python -I tools/wsp/media/freeze_batch3.py
    -> docs/wsp/atlas/splits/PSI-68-batch3.json, docs/wsp/atlas/splits/PSI-39-batch3.json

Choice, in the owner's order of priority (no frame and no outcome is read):
 1. FLEX-2 train days, n-decane, measured ambient (batch-2 tier A never fetched), <= 1.2 atm: 5. The batch-1/2
    loss table says the track was kept most often at reduced oxygen (x_O2 ~0.17 at 1 atm: 3 of 4 usable),
    at 0.5 atm (8 of 13 tier-B n-decane usable) and in helium (2 of 3); in air at 1 atm 3 of 11; above
    1.2 atm 0 of 9. No measured-ambient test at reduced pressure is left, so: the three x_O2 0.17 tests of
    two days (A221D04, A118D06, A118D04) and one test of each air day (A113D07, A109D08), smallest archive
    first within a day.
 2. FLEX-2 train days, n-octane, measured ambient, never fetched: all that remain (E053A03, E135F10), and the
    two archives the old downloader corrupted (A184D03, A186D02: n-decane, ambient unknown, kept set).
 3. FLEX-2 TEST days, n-decane/n-octane free droplets with a measured ambient, never fetched: only n-decane in
    helium at 0.5 atm exists (day 2012-11-05, A310D02-08; no n-octane): the 3 smallest archives. Held blind.
 4. CFI train days, n-dodecane free droplets without a processed series, with a backlit archive: 2, spread
    over the diluent (F125A04 air 1 atm, F139A03 He 0.29 1 atm). CFI test days, n-dodecane, never fetched,
    with an archive: both that exist (F158A03 free, 5 atm, outside the pressure domain; F163A03 support not
    stated). Held blind.
"""
import json
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SPLITS = REPO / "docs/wsp/atlas/splits"
CAT68 = REPO / "runs/dev/data_all/work/atlas/flex2_tests.jsonl"
CAT39 = REPO / "runs/dev/data_all/work/transfer_droplet/cfi/conditions.jsonl"
LIST39 = REPO / "runs/dev/data_all/manifests/PSI-39.listing.jsonl"

F68 = [("A221D04", 1, "train"), ("A118D06", 1, "train"), ("A118D04", 1, "train"), ("A113D07", 1, "train"),
       ("A109D08", 1, "train"), ("E053A03", 2, "train"), ("E135F10", 2, "train"), ("A184D03", 2, "train"),
       ("A186D02", 2, "train"), ("A310D04", 3, "test"), ("A310D03", 3, "test"), ("A310D05", 3, "test")]
F39 = [("F125A04", 4, "train"), ("F139A03", 4, "train"), ("F158A03", 4, "test"), ("F163A03", 4, "test")]


def main():
    cat = {json.loads(l)["test_id"]: json.loads(l) for l in CAT68.open(encoding="utf-8")}
    tests = []
    for tp, pri, part in F68:
        r = cat[tp]
        assert r["group"] and (part == "test") == (r["group"] in json.loads(
            (SPLITS / "PSI-68-days.json").read_text(encoding="utf-8"))["groups"]["test"]), tp
        a = r.get("ambient") or {}
        tests.append(dict(test_id=tp, date=r["group"], partition=part, fuel=r["fuel"], candidate=r.get("candidate"),
                          priority=pri, archive=f"{tp}A.zip", bytes=r["backlit_archive_bytes"],
                          conditions=dict(x_o2=a.get("O2"), x_co2=a.get("CO2"), x_he=a.get("He"),
                                          pressure_pa=r.get("pressure_pa"), ambient_how=r.get("ambient_how")),
                          held=part == "test"))
    doc = dict(schema="ignis.atlas.batch/1", investigation_id="PSI-68", frozen_on="2026-11-15", batch="batch3",
               rule=__doc__.split("Choice,")[1].strip(), bytes=sum(t["bytes"] for t in tests), tests=tests)
    (SPLITS / "PSI-68-batch3.json").write_text(json.dumps(doc, indent=1) + "\n", encoding="utf-8")
    cfi = {json.loads(l)["cir"]: json.loads(l) for l in CAT39.open(encoding="utf-8")}
    sizes = {}
    for l in LIST39.open(encoding="utf-8"):
        x = json.loads(l)
        if x["file_name"].endswith("A.zip"):
            sizes[x["file_name"].split("_")[-1]] = x["size_bytes"]
    t39 = []
    for cir, pri, part in F39:
        r = cfi[cir]
        assert r["partition"] == part, cir
        t39.append(dict(test_id=cir, flex=r["flex"], date=r["group"], partition=part, fuel=r["fuel"], priority=pri,
                        support=r["fibre"], archive=f"{cir}A.zip", bytes=sizes[f"{cir}A.zip"],
                        conditions=dict(x_o2=r["x_o2"], x_co2=r["x_co2"], x_he=r["x_he"], pressure_pa=r["pressure_pa"]),
                        gates_dodecane=r["gates"].get("wsp5/dodecane/w0-fable"), held=part == "test"))
    doc39 = dict(schema="ignis.atlas.batch/1", investigation_id="PSI-39", frozen_on="2026-11-15", batch="batch3",
                 rule=__doc__.split(" 4. ")[1].strip(), split="docs/wsp/atlas/splits/PSI-39.json",
                 bytes=sum(t["bytes"] for t in t39), tests=t39)
    (SPLITS / "PSI-39-batch3.json").write_text(json.dumps(doc39, indent=1) + "\n", encoding="utf-8")
    print(round(doc["bytes"] / 1e9, 1), round(doc39["bytes"] / 1e9, 1), len(tests), len(t39))


if __name__ == "__main__":
    main()
