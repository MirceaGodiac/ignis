"""Inventory literal assignments and write Astra cards, never estimates a coefficient.

Counts are tensor positions, not effective statistical degrees of freedom. Source kinds
below are the main author's classification, not inferred from feedback or numeric values.
"""
from collections import Counter
import json

from candidates import ROOT, CASES, cli, read, write_same

THERMAL = {
    "on": "S", "member_weight": "S", "nitrogen_only": "S", "cutoff_on": "S",
    "apply": "SSS", "dt_s": "S", "gas_constant_j_mol_k": "N", "molar_mass_kg_mol": "N",
    "rho_kg_m3": "N", "cp_liquid_j_kg_k": "N", "cp_gas_j_kg_k": "N",
    "k_gas_w_m_k": "NSS", "diffusivity_ref_m2_s": "NSS", "transport_ref_k": "N",
    "transport_ref_pa": "N", "diffusivity_exponent": "L", "antoine": "NNN",
    "antoine_range_k": "NN", "latent": "NNNN", "latent_range_k": "NN",
    "vapor_extension": "L", "latent_extrapolation": "L", "ambient_k": "A",
    "cloud_excess_k": "SSS", "cloud_tau_s": "SSS", "initial_k": "USS",
    "activity": "LSS", "x_infinity": "L", "surface_fraction_max": "U",
}
OXYGEN = {
    "mix": "S", "member_weight": "S", "rho_kg_m3": "A", "nu_m2_s": "N",
    "alpha_m2_s": "N", "molar_o2_kg_mol": "N", "molar_carrier_kg_mol": "N",
    "fuel_areal_kg_m2": "A", "faces": "S", "stoich_kg_o2_kg_fuel": "N",
    "residence_s": "L", "reference_width_m": "A",
}
LIMITS = {
    "flex_methanol6": "Only future hot-radiative endings without He/CO2 use coupled cooling. Water activity, camera view loss, already-unlit thermal history and diffusive endings retain predecessor limits. Pure-liquid, room-environment and 330 K starting-temperature assumptions are uncertain; extrapolations are disclosed in the trace.",
    "flex_heptane6": "Only future hot-radiative endings without He/CO2 use coupled cooling. This does not solve the dominant helium, high-CO2 or cool-flame discrepancies. The 360 K initial state, constant liquid properties and no-cloud-heat assumption are not observations; source extrapolations are explicit.",
    "acme_normal6": "The flow/pressure nodes are sensitivity judgments at cutoff, not sensor calibration or inferred posterior. Composition/Tad/Zst stay fixed. Wrong labels or gross table errors remain. Planned-stop use is conditional on its unresolved prospective provenance; the main train comparison uses shared-record fallback.",
    "acme_inverse6": "Persistent condition scenarios may blur a correct regime prediction. They do not identify oscillatory extinction, repair table errors or add a trustworthy control plan. Supplied Tad/Zst are held conditional; node masses at cutoff are assumed, not posterior probabilities.",
    "bass_front3": "The oxygen supply law uses ambient density and hot-layer transport under a declared steady capture-layer closure. Its 10 s residence time is directly train-informed. Finite width, fan calibration and visible-front transients remain assumptions. Active forecasts use coherent first-event semantics; the joint reinterprets both speed members' events and therefore is not an average of the old artifact's event rows.",
}
INHERITED_TRAIN = {
    "flex_methanol6": "unknown; not zero (one explicitly identified mixed-origin train-sd floor)",
    "flex_heptane6": "unknown; not zero (one explicitly identified mixed-origin train-sd floor)",
    "acme_normal6": "unknown; at least 5 measurement-backed inherited positions",
    "acme_inverse6": "unknown; at least 9 measurement-backed inherited positions",
    "bass_front3": "unknown; at least 2 measurement-backed positions (1 inherited + 1 new)",
}


def statements(text):
    return [line for line in text.splitlines() if line.strip() and not line.lstrip().startswith("#")]


def positions(op):
    if op["op"] == "set":
        return 1
    count = 1
    for n in op["shape"]:
        count *= n
    return count


def main():
    rows = []
    for key, old, new in CASES:
        directory = ROOT / "configs/models" / new
        prefix = "thermal." if key.startswith("flex") else "oxygen." if key == "bass_front3" else "condition."
        origins = THERMAL if prefix == "thermal." else OXYGEN if prefix == "oxygen." else {
            "w": "UUUUU", "flow_multiplier": "UUUUU", "pressure_multiplier": "UUUUU"}
        inventory = Counter("".join(origins.values()))
        original = read(ROOT / "configs/models" / old / "release.json")
        parent_text = statements(cli("inspect", "--release", original["id"], "--edits"))
        patch = read(directory / "r1-astra/patch.json")
        new_ops = [op for op in patch["ops"] if op["tensor"].startswith(prefix)]
        assert sum(positions(op) for op in new_ops) == sum(inventory.values())
        assert {op["tensor"][len(prefix):] for op in new_ops} == set(origins)
        for label in ("r1", "joint"):
            candidate = directory / f"{label}-astra"
            release = read(candidate / "release.json")
            receipt = read(candidate / "assembly.json")
            full = statements(cli("inspect", "--release", release["id"], "--edits"))
            inherited = [line for line in full if not line.startswith(prefix)]
            assert inherited == parent_text, f"copied Fable values changed in {key}/{label}"
            inspection = json.loads(cli("inspect", "--release", release["id"]))
            assert inspection["forward_matches"]
            current_patch = read(candidate / "patch.json")
            authored_now = sum(positions(op) for op in current_patch["ops"] if op["tensor"].startswith(prefix))
            cumulative = inventory.copy()
            if label == "joint" and prefix != "condition.":
                cumulative["S"] -= 1
                cumulative["U"] += 1
            info = {
                "model": key, "candidate": label, "release_id": release["id"],
                "status": "data-informed; experimental; not_trained; no new validation look",
                "authored_tensor_capacity": receipt["counts"]["authored_total"],
                "authored_nonzero": receipt["counts"]["authored_nonzero"],
                "spec_fixed_elements": receipt["counts"]["constant_total"],
                "inherited_literal_positions_from_fable": len(inherited),
                "new_explicit_positions_this_patch": authored_now,
                "cumulative_new_positions_by_origin": dict(sorted(cumulative.items())),
                "new_direct_train_read_positions_cumulative": 1 if key == "bass_front3" else 0,
                "inherited_train_exposure": "all copied Fable positions retain exposure; precise direct-train subset of historical literals not reclassified",
                "exact_inherited_transcription_verified": True,
                "forward_hash_matches": True,
            }
            write_same(candidate / "inventory.json", info)
            lines = [f"# {release['model_name']} — Astra {label}", "",
                     "**Data-informed, not zero-shot. Experimental; not trained; no new validation look.**", "",
                     f"Release: `{release['id']}`. Parent: `{release['parent_id']}`.", "",
                     "Main author: `gpt-6-astra`, effort `max`; Fable is credited for exact inherited literals.", "",
                     LIMITS[key], "", "| Inventory | Count / status |", "|---|---|",
                     f"| Authored tensor capacity (including declared structural zeros) | {info['authored_tensor_capacity']} |",
                     f"| Nonzero authored positions | {info['authored_nonzero']} |",
                     f"| Fixed spec constants | {info['spec_fixed_elements']} |",
                     f"| Explicit literal positions copied unchanged from Fable | {len(inherited)} |",
                     f"| Explicit positions in full R1 patch (including authored zeros) | {len(inherited) + sum(inventory.values())} |",
                     f"| New positions written by Astra in this patch | {authored_now} |",
                     f"| Cumulative new N / A / L / S / U positions | {' / '.join(str(cumulative[k]) for k in 'NALSU')} |",
                     f"| New directly train-read positions, cumulative | {info['new_direct_train_read_positions_cumulative']} |",
                     f"| Full-model measurement-backed train count | {INHERITED_TRAIN[key]} |", "",
                     "N = source property; A = apparatus/environment (may be assumed); L = closure; S = structural/numerical; U = uncertainty judgment. Inactive carrier/branch placeholders count as S. Counts are expanded positions, not unique numbers or effective degrees of freedom.", "",
                     "The inherited block has mixed origins and is conservatively marked **inherited, train-informed**. Its exact directly train-read subset was not recounted, so the full model's count is not zero merely because a new block has zero direct-train entries. Reasons for new values are beside every assignment in `addition.wsp` (R1) or `edits.wsp` (joint).", "",
                     "The subsequent documentary audit `docs/wsp/feedback/astra_inherited_origin_audit.md` establishes the conservative inherited bounds above; exact inherited transcription proves those coordinates survive here. Measurement-backed includes explicitly described empirical summaries/differences, with raw transcriptions and mixed-origin judgments separated in that audit. No bound is an exact census or an effective-freedom count.", "",
                     "R1 uses the explicit new-spec full-literal rewrite route; the parent records ancestry without array inheritance. The joint is an ordinary same-spec literal revision. `inventory.json` verifies exact inherited transcription and the executing forward hash. See dossier `ASTRA_PROPERTIES.md`, `ASTRA_ERROR_BUDGET.md`, and `ASTRA_RESULTS.md` for source limits, priorities, and actual train results. No adoption or generalisation claim follows from a train score.", ""]
            write_same(candidate / "CARD.md", "\n".join(lines))
            rows.append(info)
    write_same(ROOT / "docs/wsp/feedback/astra_literal_inventory.json", rows)
    print(json.dumps(rows, indent=2))


if __name__ == "__main__":
    main()
