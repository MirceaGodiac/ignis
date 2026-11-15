"""Reference-property census and arithmetic audit; never reads experimental outcomes.

Offline default: compare the 374-row reference CSV, replay its frozen producer with
the twenty expressly authorized reference inputs, and write a row ledger/receipt.
--execute additionally downloads the allowlisted fundamental-property references.
No model, parameter patch, optimization, or NASA investigation file is consumed.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys
from datetime import datetime, timezone
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[3]
OLD = ROOT / "runs/dev/research7/properties"
OUT = ROOT / "runs/dev/data_astra/research/property_audit"
INPUT_NAMES = (
    "heptane_satT1.tsv", "heptane_satT2.tsv", "C142825_satP.tsv",
    "heptane_sat204.tsv", "heptane_sat298.tsv", "heptane_isobar.tsv",
    "methanol_satT1.tsv", "C67561_satP.tsv", "methanol_sat298.tsv",
    "methanol_isobar.tsv", "water_satT.tsv", "C7732185_satP.tsv",
    "water_sat298.tsv", "h2o_isobar.tsv", "n2_isobar.tsv", "o2_isobar.tsv",
    "co2_isobar.tsv", "he_isobar.tsv", "type_k.tab.txt", "type_b.tab.txt",
)
URLS = {
    "nist_tn2279_2024.pdf": "https://nvlpubs.nist.gov/nistpubs/TechnicalNotes/NIST.TN.2279.pdf",
    "mcgivern_manion_2012.pdf": "https://tsapps.nist.gov/publication/get_pdf.cfm?pub_id=909700",
    "marrero_mason_1972.pdf": "https://srd.nist.gov/jpcrdreprint/1.3253094.pdf",
    "methanol_conductivity_2013.pdf": "https://www.nist.gov/system/files/documents/2017/01/09/jpcrdv422013_043101.pdf",
    "heptane_conductivity_2013.pdf": "https://srd.nist.gov/jpcrdreprint/1.4794091.pdf",
    "lemmon_jacobsen_2004.pdf": "https://trc.nist.gov/refprop/FAQ/NAO.PDF",
    "psl_slip_kim_2005.pdf": "https://nvlpubs.nist.gov/nistpubs/jres/110/1/j110-1kim.pdf",
    "heptane_phase.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C142825&Mask=4",
    "heptane_cp.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C142825&Mask=2",
    "methanol_phase.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C67561&Mask=4",
    "dodecane_phase.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C112403&Mask=4",
    "ethylene_gas.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C74851&Mask=1",
    "propene_gas.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C115071&Mask=1",
    "farnesane_phase.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C3891983&Mask=4",
    "heptane_cp_1994.pdf": "https://www.nist.gov/system/files/documents/srd/jpcrd469.pdf",
    "methane_gas.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C74828&Mask=1",
    "hydrogen_gas.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C1333740&Mask=1",
    "nitrogen_gas.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C7727379&Mask=1",
    "helium_gas.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C7440597&Mask=1",
    "pmma_cp.html": "https://webbook.nist.gov/cgi/cbook.cgi?ID=C87210320&Mask=2",
    "tnf_radiation.html": "https://tnfworkshop.org/radiation/",
    "vogel_bich_hellmann_2023.html": "https://link.springer.com/article/10.1007/s10765-023-03233-y",
    "vogel_bich_hellmann_2023.pdf": "https://link.springer.com/content/pdf/10.1007/s10765-023-03233-y.pdf",
}


def digest(path: Path) -> dict:
    data = path.read_bytes()
    return {"path": str(path.relative_to(ROOT)), "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest()}


def source_family(row: dict) -> str:
    url = row["url"]
    if "/chemistry/fluid/" in url:
        return "NIST-fluid-EOS/transport"
    if "cbook.cgi" in url:
        return "NIST-WebBook-compilation"
    if "1.3253094" in url:
        return "Marrero-Mason-1972-evaluation"
    if "909700" in url:
        return "McGivern-Manion-2012-measurements/fit"
    if "15/5585" in url:
        return "Tang-2015-evaluation"
    if "14/9233" in url:
        return "Fuller-estimate-via-Tang-2014"
    if "its90" in url:
        return "NIST-ITS90-reference-function"
    if "tnfworkshop" in url:
        return "TNF-RADCAL-fit"
    if "2411.18467" in url:
        return "Smith-secondary-reproduction"
    if "1.555744" in url:
        return "Kadoya-1985-evaluation"
    if not url:
        return "no-source-URL/arithmetic-or-closure"
    return "other-see-original-source"


def phase(row: dict) -> str:
    q, s = row["quantity"].lower(), row["substance"].lower()
    if "thermocouple" in q or s.startswith("type "):
        return "instrument-reference; no fluid phase"
    if "burner" in s or "fabric" in s or "cellulose" in s or "sibal" in s:
        return "solid/apparatus; specimen-specific where stated"
    if "boiling" in q or "vapour pressure" in q or "latent heat" in q or "evaporation" in q or "wet-bulb" in q or "e-fold temperature" in q or "lowers" in q:
        return "liquid/vapour interface or derived closure"
    if "liquid" in q or "rho_l" in q:
        return "liquid (saturation if stated)"
    if "stoichiometric" in q or "heat release" in q or "heat" in q and "combustion" in q or "heating value" in q or "molar mass" in q:
        return "chemical identity/reaction; phases in quantity"
    return "gas or gas-mixture; inferred from property context"


def arithmetic() -> dict:
    def antoine_tb(p_atm: float, a: float, b: float, c: float) -> float:
        return b / (a - math.log10(p_atm * 1.01325)) - c
    def published_d(t: float, a: float, b: float, c: float) -> float:
        return math.exp(a + b/t + c*math.log(t))
    def shomate(t_kelvin: float, a: float, b: float, c: float, d: float, e: float) -> float:
        t=t_kelvin/1000
        return a+b*t+c*t*t+d*t**3+e/t**2
    m_o2 = 31.9988
    stoich = {name: n*m_o2/m for name, n, m in (
        ("heptane", 11, 100.2019), ("methanol", 1.5, 32.0419),
        ("ethane", 3.5, 30.0690), ("propane", 5, 44.0956),
        ("n-butane", 6.5, 58.1222), ("ethylene", 3, 28.0532),
        ("propene", 4.5, 42.0797), ("n-dodecane", 18.5, 170.3348),
        ("farnesane_identity_only", 23, 212.4146),
        ("cellulose_repeat_unit", 6, 162.1406))}
    return {
        "note": "Literal source numbers evaluated for audit only; never model weights.",
        "methanol_Ambrose1970_Tb_K": {str(p): antoine_tb(p,5.20409,1581.341,-33.50) for p in (1,2,3)},
        "methanol_Ambrose1975_Tb_K": {str(p): antoine_tb(p,5.15853,1569.613,-34.846) for p in (2,3)},
        "methanol_Hirata1967_Tb_K": {str(p): antoine_tb(p,5.31301,1676.569,-21.728) for p in (2,3)},
        "heptane_Cp_300K_J_kgK_from_VanMiltenburg":225.14/0.1002019,
        "heptane_Cp_300K_J_kgK_from_Tan":225.33/0.1002019,
        "heptane_Cp_1994_at_300_320_340_360_370K_J_kgK":[v/0.1002019 for v in (225.58,233.38,241.85,250.94,255.71)],
        "heptane_k_1986_at_300_335K_W_mK":[(212.475-89.635*t/298.15)/1000 for t in (300,335)],
        "methanol_k_2013_at_300_320K_W_mK":[(265.67-0.2191*t)/1000 for t in (300,320)],
        "H2_Cp_300K_J_molK":shomate(300,33.066178,-11.363417,11.432816,-2.772874,-0.158558),
        "N2_Cp_300K_J_molK":shomate(300,28.98641,1.853978,-9.647459,16.63537,0.000117),
        "PMMA_Cp_300K_J_kgK_from_Lebedev":131.8/0.1001158,
        "heptane_300K_alpha_from_rounded_rows_m2_s":0.1216/(677.9*2254),
        "methanol_300K_alpha_from_rounded_rows_m2_s":0.1998/(784.5*2546),
        "air_1250K_nu_from_rounded_rows_m2_s":5.021e-5/0.2824,
        "air_1250K_alpha_from_rounded_rows_m2_s":0.08044/(0.2824*1181),
        "stoichiometric_O2_fuel_kg_kg":stoich,
        "ethylene_LHV_298K_MJ_kg_from_Rossini_and_water44_004":(1411.20-2*44.004)/28.0532,
        "propene_LHV_298K_MJ_kg_from_Rossini_and_water44_004":(2057.7-3*44.004)/42.0797,
        "propene_LHV_298K_MJ_kg_from_Wiberg_and_water44_004":(2057.8-3*44.004)/42.0797,
        "N2_CO2_TN2279_298K_cm2_s":published_d(298.15,-7.317,-139.909,1.103),
        "N2_CO2_TN2279_300K_cm2_s":published_d(300,-7.317,-139.909,1.103),
        "N2_CO2_Vogel2023_Table5_298_20K_1atm_cm2_s":6.824e-4*8.314462618*298.20/101325*1e4,
        "N2_CO2_Vogel2023_Table7_rejected_298_20K_1atm_cm2_s":9.283e-4*8.314462618*298.20/101325*1e4,
        "H2O_N2_TN2279_300_1000K_cm2_s":[published_d(t,-10.324,-84.181,1.620) for t in (300,1000)],
        "H2O_CO2_TN2279_300_1000K_cm2_s":[published_d(t,-10.298,-180.060,1.597) for t in (300,1000)],
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--execute", action="store_true", help="download fixed property-only references")
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    replay = OUT / "replay"
    replay.mkdir(exist_ok=True)
    inputs = [OLD / "sources" / name for name in INPUT_NAMES]
    with (OLD / "properties.csv").open(encoding="utf-8", newline="") as stream:
        rows = list(csv.DictReader(stream))
    if len(rows) != 374:
        raise ValueError(f"Reference CSV changed: expected 374 rows, got {len(rows)}")
    result = subprocess.run([sys.executable, "-I", str(OLD / "compute_properties.py"),
                             str(OLD / "sources"), str(replay)], check=True,
                            capture_output=True, text=True, encoding="utf-8")
    (OUT / "replay_stdout.txt").write_text(result.stdout, encoding="utf-8")
    if result.stderr:
        (OUT / "replay_stderr.txt").write_text(result.stderr, encoding="utf-8")
    with (replay / "properties.csv").open(encoding="utf-8", newline="") as stream:
        replay_rows = list(csv.DictReader(stream))
    receipt = {
        "created_utc":datetime.now(timezone.utc).isoformat(),
        "scope":"reference-property-only; not independent empirical validation",
        "auditor_code":digest(Path(__file__)),
        "original_csv":digest(OLD / "properties.csv"),
        "producer":digest(OLD / "compute_properties.py"),
        "inputs":[digest(p) for p in inputs],
        "replay_outputs":[digest(p) for p in sorted(replay.glob("*.csv"))],
        "row_count":len(rows),
        "replay_records_identical":rows == replay_rows,
        "replay_bytes_identical":(OLD / "properties.csv").read_bytes()==(replay / "properties.csv").read_bytes(),
        "arithmetic":arithmetic(),
    }
    unique_keys = {(r["quantity"],r["substance"],r["conditions"],r["value"],r["unit"],r["source"],r["url"]) for r in rows}
    receipt["unique_property_source_tuples"] = len(unique_keys)
    receipt["missing_url_rows"] = [i for i,r in enumerate(rows,1) if not r["url"]]
    ledger = []
    for i, row in enumerate(rows,1):
        ledger.append({"row_id":f"P{i:03d}", **{k:row[k] for k in ("quantity","substance","conditions","value","unit","source","url","how_obtained")},
                       "phase_audit":phase(row), "source_family_audit":source_family(row),
                       "arithmetic_replay":"identical" if rows==replay_rows else "mismatch-see-receipt",
                       "independent_empirical_status":"not established by replay; see scoped comparisons in audit document"})
    with (OUT / "row_ledger.csv").open("w",encoding="utf-8",newline="") as stream:
        writer=csv.DictWriter(stream,fieldnames=ledger[0].keys());writer.writeheader();writer.writerows(ledger)
    receipt["row_ledger"] = digest(OUT / "row_ledger.csv")
    if args.execute:
        sources=OUT / "sources";sources.mkdir(exist_ok=True)
        downloaded=[]
        for name,url in URLS.items():
            path=sources / name
            try:
                with urlopen(Request(url,headers={"User-Agent":"IGNIS-property-audit/1.0"}),timeout=45) as response:
                    data=response.read()
                    content_type=response.headers.get("Content-Type","")
                    if name.endswith(".pdf") and not data.startswith(b"%PDF"):
                        raise ValueError("Response is not a PDF")
                    if b"/_fs-ch-" in data:
                        raise ValueError("Publisher bot-challenge HTML; not reference content")
                    path.write_bytes(data)
                    downloaded.append({"url":url,"final_url":response.url,"content_type":content_type,
                                       "retrieved_utc":datetime.now(timezone.utc).isoformat(),**digest(path)})
            except Exception as exc:
                downloaded.append({"url":url,"error":str(exc)})
        (OUT / "source_manifest.json").write_text(json.dumps(downloaded,indent=2)+"\n",encoding="utf-8")
    (OUT / "audit_receipt.json").write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
    print(json.dumps({k:receipt[k] for k in ("row_count","unique_property_source_tuples","replay_records_identical","replay_bytes_identical")},indent=2))
    print(json.dumps(receipt["arithmetic"],indent=2))


if __name__ == "__main__":
    main()
