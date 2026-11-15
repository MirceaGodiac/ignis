"""Evaluate four fixed reference-property states; never writes model coefficients.

Dependency (isolated, not a runtime dependency of any forward):
python -m pip install --target runs/dev/astra/property_vendor CoolProp==8.0.0
"""
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "runs/dev/astra/property_vendor"))
import CoolProp
from CoolProp.CoolProp import PropsSI

out = {
    "version": CoolProp.__version__,
    "git_revision": CoolProp.__gitrevision__,
    "purpose": "independent property evaluation only; candidate literals already typed and never modified by this script",
    "states": [],
}
for fluid, temperature in [("Methanol", 300), ("n-Heptane", 300), ("Nitrogen", 300), ("Air", 1250)]:
    row = {key: PropsSI(key, "T", temperature, "P", 101325, fluid)
           for key in ["Dmass", "Cpmass", "VISCOSITY", "CONDUCTIVITY"]}
    row.update(fluid=fluid, temperature_k=temperature, pressure_pa=101325)
    row["nu_m2_s"] = row["VISCOSITY"] / row["Dmass"]
    row["alpha_m2_s"] = row["CONDUCTIVITY"] / (row["Dmass"] * row["Cpmass"])
    out["states"].append(row)
text = json.dumps(out, indent=2) + "\n"
(ROOT / "docs/wsp/feedback/astra_property_crosscheck.json").write_text(text, encoding="utf-8")
print(text)
