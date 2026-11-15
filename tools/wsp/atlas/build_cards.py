"""Builds one card per NASA flight combustion investigation and the ranking of the investigations by
crew habitat and fire-safety need.

Everything in a card is one of three things, and says which:
  - NASA's own words: objective, hypothesis, approach and impacts from the PSI metadata record;
  - a count or range computed from the test catalogue and the file catalogue of this folder;
  - a typed judgement of the author, each with the sentence of NASA's it rests on (the table TYPED).
No outcome of any test is read: the catalogue file of outcome values is not opened.

The ranking is a rule, written out below with a reason for each weight. It ranks by what an
investigation covers (need, atmosphere, gravity, flow, data, predictive support), never by text
similarity, and every rank comes with the parts of its score.

    python tools/wsp/atlas/build_cards.py           # writes docs/wsp/atlas/cards/, CARDS.md, habitat_ranking.json, HABITAT_RANKING.md
"""
import io
import json
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
ATLAS = REPO / "runs/dev/data_all/work/atlas"
META = REPO / "runs/dev/data_all/raw/samples/_meta"
OUT = REPO / "docs/wsp/atlas"

ORDER = ["PSI-69", "PSI-68", "PSI-39", "PSI-159", "PSI-10", "PSI-23", "PSI-20", "PSI-21", "PSI-22",
         "PSI-107", "PSI-106", "PSI-25", "PSI-26", "PSI-98", "PSI-99", "PSI-100", "PSI-102", "PSI-101", "PSI-47"]

# ------------------------------------------------------------------ sentences of the challenge brief
BRIEF = "NASA Space Apps 2026, challenge 'Flame in Freefall' (spaceappschallenge.org/2026/challenges/flame-in-freefall)"
Q_PREV = ("Flammability studies focus on solid materials such as fabrics and plastics, as studied in the Burning and "
          "Suppression of Solids (BASS and BASS-II) and Spacecraft Fire Safety Demonstration (Saffire-I, -II, and -III) investigations.")
Q_DET = "Fire detection was tested in microgravity through the Smoke and Aerosol Measurement Experiment studies (SAME and SAME-R)."
Q_SUP = ("Suppression was studied in the Flame Extinguishment Experiment (FLEX) through dilution of the chamber atmosphere "
         "with carbon dioxide and helium, both of which suppress fire.")
Q_DAFT = ("One of the instruments used for smoke detection in the SAME and SAME-R studies was instead used to detect dust in the "
          "Dust and Aerosol Measurement Feasibility Test (DAFT and DAFT-2) studies.")
Q_BRE = ("While it burned gaseous fuels, the Burning Rate Emulator (BRE) experiment of the Advanced Combustion via Microgravity "
         "Experiments (ACME) project was a study of materials flammability in still atmospheres as a function of material properties.")
Q_OTHER = ("Other combustion investigations within the PSI database (but not explicitly referenced above) are still relevant to "
           "fire safety, just not as strongly or directly.")
Q_CFIG = ("Microgravity tests with the highest oxygen concentrations were conducted as part of the ACME's Cool Flames "
          "Investigation with Gases (CFI-G).")
Q_EFIELD = ("Partial gravity was mimicked in ACME's Electric-Field Effects on Laminar Diffusion Flames (E-FIELD Flames) experiment "
            "through the effect of the electric field on the ions that naturally occur in flames.")
Q_DUCT = ("BASS, BASS-II, the Smoke Point In Co-flow Experiment (SPICE), and the Structure & Liftoff In Combustion Experiment "
          "(SLICE) ... Each of these investigations was conducted within the Microgravity Science Glovebox (MSG) using the modular "
          "fan-driven flow duct built for SPICE.")
Q_ACME = ("The six experiments in the ACME project featured flames of gaseous fuels and were all conducted with the same modular "
          "set of ACME hardware in the Combustion Integrated Rack (CIR).")
Q_MDCA = ("FLEX, FLEX-2, and the Cool Flames Investigation (CFI) each featured burning droplets of liquid fuel and were conducted "
          "with the Multi-user Droplet Combustion Apparatus (MDCA) in the CIR.")
Q_NOPG = "there is currently no partial-gravity combustion data available in PSI"

NAMED, STATED, FOUNDATIONAL = "named in the challenge brief", "stated in NASA's objective for the investigation", "foundational"
TIER = {NAMED: 3, STATED: 2, FOUNDATIONAL: 1}


def need(n, tier, quote, source=None):
    return {"need": n, "tier": tier, "basis": quote,
            "basis_source": source or (BRIEF if tier == NAMED else "PSI metadata record, field objective")}


# ------------------------------------------------------------------ typed judgements, each with its basis
# family: the hardware family as the brief groups the investigations.
# needs: which fire-safety need the investigation serves, and how directly. `foundational` entries quote the
#        phenomenon in NASA's objective; the brief's sentence on other investigations (Q_OTHER) is their warrant.
# stated_atmospheres: a crew atmosphere that NASA's objective says the investigation examines.
# normal_gravity / partial_gravity_proxy: what the investigation offers towards partial gravity.
# flow: an imposed flow is part of the apparatus, where the test catalogue has no flow speed per test.
# predictive: what this project's hand-written models can do with the investigation today.
TYPED = {
    "PSI-69": {
        "family": ("droplets in the MDCA", Q_MDCA),
        "needs": [need("suppression", NAMED, Q_SUP),
                  need("prevention", STATED, "The FLEX (Flame Extinguishment Experiment) investigation aims to map the flammability limits of liquid fuels in microgravity environments")],
        "predictive": {"status": "scored models", "detail": "flex_methanol.wsp2 D3 and flex_heptane.wsp2 D2: diameter forecasts scored on validation days (CRPS 40.0 and 46.1 µm; development numbers)", "points": 2},
    },
    "PSI-68": {
        "family": ("droplets in the MDCA", Q_MDCA),
        "needs": [need("suppression", FOUNDATIONAL, "Study flame extinction mechanisms and their boundaries."),
                  need("prevention", FOUNDATIONAL, "Determine the influence of droplet interactions on the flammability limits and extinction behavior of droplet")],
        "predictive": {"status": "not usable yet", "detail": "the droplet models take the fuel by name; n-heptane tests exist but their diameters are only in images", "points": 0},
    },
    "PSI-39": {
        "family": ("droplets in the MDCA", Q_MDCA),
        "needs": [need("suppression", FOUNDATIONAL, "Analyze the effects of helium dilution on cool flame behavior."),
                  need("prevention", FOUNDATIONAL, "Investigate the relationship between droplet size, pressure, and flame extinction")],
        "predictive": {"status": "refused", "detail": "n-dodecane, its mixtures and farnesane are outside the droplet models' fuels", "points": 0},
    },
    "PSI-159": {
        "family": ("ACME burner flames in the CIR", Q_ACME),
        # 15 Nov: kept as an inference of the atlas: the documents cite engine efficiency and cool-flame kinetics; the
        # observed cool-flame persistence and reignition bear on prevention (narrative need_check, Opus 5.5 xHigh)
        "needs": [dict(need("prevention", FOUNDATIONAL, "Determine the conditions for which cool diffusion flames occur for propane, n-butane, and n-pentane; normal and inverse flames"), inference="the atlas's, not NASA's: the documents do not link CFI-G to fire safety")],
        "brief_note": Q_CFIG,
        "predictive": {"status": "scored models", "detail": "acme_normal.wsp5 P1 and acme_inverse.wsp5 P1: when the flame ends and why, scored on validation days (extinction Brier 0.155 and 0.129; development numbers)", "points": 2},
    },
    "PSI-10": {
        "family": ("ACME burner flames in the CIR", Q_ACME),
        # 15 Nov: was detection (soot-inception limits); the documents tie soot inception to terrestrial combustion,
        # and NASA's own fire-safety link is objective C (narrative need_check, Opus 5.5 xHigh)
        "needs": [need("prevention", FOUNDATIONAL, "Obtain extinction limits as functions of Zst, amount of N2, and pressure for normal and inverse spherical C2H4 and CH4 flames. [...] Evaluate whether pseudo-flammability limits can be obtained for these flames.")],
        "predictive": {"status": "transfer tested", "detail": "the CFI-G models, unchanged, refuse ethylene; with ethane as a surrogate the normal-flame model beats the incidence baseline (0.120 against 0.247), the inverse model is undecided (PROTOCOL 11.3)", "points": 1},
    },
    "PSI-23": {
        "family": ("ACME burner flames in the CIR", Q_ACME),
        "needs": [need("prevention", FOUNDATIONAL, "Determine the quasi-steady convective/chemical-kinetic extinction limits (low system Damköhler numbers) and radiative/chemical-kinetic extinction limits (high system Damköhler numbers)"),
                  need("suppression", FOUNDATIONAL, "extinction limits ... for mixtures of different H2/CH4/diluent ratios (soot-free) and different C2H4/diluent ratios (sooty)")],
        "predictive": {"status": "none", "detail": "no model; same burner as CFI-G, other fuels", "points": 0},
    },
    "PSI-20": {
        "family": ("ACME burner flames in the CIR", Q_ACME),
        "needs": [need("prevention", NAMED, Q_BRE)],
        "stated_atmospheres": {"earth_iss": "NASA - considered atmosphere for human space missions will be examined: 14.7 psia, 21 % oxygen",
                               "moon_base_rover": "NASA - considered atmosphere for human space missions will be examined: ... and 8.2 psia, 34 % oxygen"},
        "predictive": {"status": "none", "detail": "no model", "points": 0},
    },
    "PSI-21": {
        "family": ("ACME burner flames in the CIR", Q_ACME),
        # 15 Nov: was prevention + detection; the documents link the work to extinction (suppression) and tie soot to
        # modelling, not detection (narrative need_check, Opus 5.5 xHigh)
        "needs": [need("suppression", FOUNDATIONAL, "Characterize the behavior of weak coflow diffusion flames of methane and ethylene, as a function of velocity and fuel dilution, through measurement of flame shape, lift-off heights, temperatures and extinction limits.")],
        "flow": "Characterize the behavior of weak coflow diffusion flames of methane and ethylene, as a function of velocity and fuel dilution",
        "predictive": {"status": "none", "detail": "no model", "points": 0},
    },
    "PSI-22": {
        "family": ("ACME burner flames in the CIR", Q_ACME),
        # 15 Nov: was prevention; NASA's documents say the work serves no space application and treat near-limit
        # flames as something the field stabilizes (narrative need_check, Opus 5.5 xHigh): no need is typed
        "needs": [],
        "partial_gravity_proxy": Q_EFIELD,
        "predictive": {"status": "none", "detail": "no model", "points": 0},
    },
    "PSI-107": {
        "family": ("flow duct in the MSG", Q_DUCT),
        # 15 Nov: kept as an inference of the atlas: the smoke point is the onset of soot emission, but NASA's documents
        # never mention smoke detection (narrative need_check, Opus 5.5 xHigh)
        "needs": [dict(need("detection", FOUNDATIONAL, "Identify the laminar smoke-point properties of non-buoyant jet diffusion flames under various co-flow conditions in microgravity."), inference="the atlas's, not NASA's: the documents do not mention smoke detection")],
        "normal_gravity": "by observing flames in a microgravity environment aboard the International Space Station (ISS) and analogous ground experiments",
        "predictive": {"status": "none", "detail": "no model", "points": 0},
    },
    "PSI-106": {
        "family": ("flow duct in the MSG", Q_DUCT),
        # 15 Nov: suppression dropped; nitrogen dilution is studied for soot and stability, not as extinguishment
        # (narrative need_check, Opus 5.5 xHigh)
        "needs": [need("prevention", FOUNDATIONAL, "Determine the liftoff velocity limits for different fuels and burner diameters.")],
        "flow": Q_DUCT,
        "normal_gravity": "Quantify the effects of microgravity on flame length and radiative heat loss compared to normal gravity conditions.",
        "predictive": {"status": "none", "detail": "no model", "points": 0},
    },
    "PSI-25": {
        "family": ("flow duct in the MSG", Q_DUCT),
        "needs": [need("prevention", NAMED, Q_PREV),
                  need("suppression", STATED, "BASS-II results contribute to the combustion computational models used in the design of fire detection and suppression systems in microgravity and on Earth."),
                  need("detection", STATED, "BASS-II results contribute to the combustion computational models used in the design of fire detection and suppression systems in microgravity and on Earth.")],
        "normal_gravity": "Begin to bridge the gap between the normal gravity NASA-STD-6001 Test #1 method, ground-based microgravity tests, and actual material flammability in microgravity.",
        "predictive": {"status": "scored model", "detail": "bass_front.wsp D1: position of the flame tip on the fabric, scored on burns measured blind (CRPS 0.117 sample widths against 0.234 for persistence; development number). Its spread-or-not boundary fails", "points": 2},
    },
    "PSI-26": {
        "family": ("flow duct in the MSG", Q_DUCT),
        "needs": [need("prevention", NAMED, Q_PREV),
                  need("suppression", STATED, "Assess the effectiveness of an inert, gaseous extinguishing agent (similar to that used on ISS) in putting out flames over different materials, geometries, preheating, and flow.")],
        "flow": Q_DUCT,
        "normal_gravity": "Begin to bridge the gap between the normal gravity NASA-STD-6001 Test#1 method, ground-based microgravity tests, and actual material flammability in microgravity.",
        "predictive": {"status": "used as outside data", "detail": "ten burns were measured from the public stills and used once, for a candidate that was not adopted", "points": 0},
    },
    "PSI-98": {
        "family": ("large samples in the Cygnus vehicle", "PSI metadata record, field experimentHardware"),
        "needs": [need("prevention", NAMED, Q_PREV)],
        "normal_gravity": "investigate the low-g flammability limits compared to those obtained in NASA's normal gravity material flammability screening test",
        "predictive": {"status": "none", "detail": "no model; the same fabric as BASS-II at forty times the width", "points": 0},
    },
    "PSI-99": {
        "family": ("large samples in the Cygnus vehicle", "PSI metadata record, field experimentHardware"),
        "needs": [need("prevention", NAMED, Q_PREV)],
        "normal_gravity": "investigate the low-g flammability limits compared to those obtained in NASA's normal gravity material flammability screening test",
        "predictive": {"status": "none", "detail": "no model", "points": 0},
    },
    "PSI-100": {
        "family": ("large samples in the Cygnus vehicle", "PSI metadata record, field experimentHardware"),
        "needs": [need("prevention", NAMED, Q_PREV)],
        "normal_gravity": "investigate the low-g flammability limits compared to those obtained in NASA's normal gravity material flammability screening test",
        "predictive": {"status": "none", "detail": "no model", "points": 0},
    },
    "PSI-102": {
        "family": ("smoke and aerosol instruments", "PSI metadata record, field experimentHardware"),
        "needs": [need("detection", NAMED, Q_DET)],
        "predictive": {"status": "none", "detail": "no model", "points": 0},
    },
    "PSI-101": {
        "family": ("smoke and aerosol instruments", "PSI metadata record, field experimentHardware"),
        "needs": [need("detection", NAMED, Q_DET)],
        "predictive": {"status": "none", "detail": "no model", "points": 0},
    },
    "PSI-47": {
        "family": ("smoke and aerosol instruments", "PSI metadata record, field experimentHardware"),
        "needs": [need("detection", STATED, "The focus of DAFT / DAFT-2 was to mitigate the risk for the Smoke Aerosol Measurement Experiment (SAME).")],
        "brief_note": Q_DAFT,
        "predictive": {"status": "none", "detail": "no model", "points": 0},
    },
}

# For four investigations the catalogue rows are the project's own cohort (train and validation runs of the
# tests the models were written on), not every test of the investigation.
ROW_SCOPE = {
    "PSI-69": "the project's cohort: free-floating methanol and n-heptane droplets, train and validation runs; the investigation has more tests and fuels",
    "PSI-159": "the project's cohort: train and validation tests of the normal and inverse flames",
    "PSI-25": "the project's cohort: the cotton-fiberglass fabric burns, train and validation; the investigation burned other samples too",
    "PSI-26": "no row: this investigation is not in the test catalogue yet",
}

# ------------------------------------------------------------------ habitats, as the brief states them
HABITATS = [
    {"id": "earth_iss", "name": "Spacecraft and space stations at an Earth-like atmosphere (the ISS)",
     "atmosphere": "oxygen 0.20 to 0.22, 14.7 ± 0.5 psi (tolerances of the test catalogue)", "gravity": "microgravity",
     "basis": "the Earth's pressure at sea level is 14.7 psi"},
    {"id": "moon_base_rover", "name": "Moon and Mars surface bases and pressurized rovers",
     "atmosphere": "oxygen 0.30 or more, 10.2 psi or less", "gravity": "partial gravity",
     "basis": "The intermediate oxygen concentration used on the Moon will likely be 30% or greater, with a corresponding pressure of 10.2 psi or less.",
     "note": "The brief states this atmosphere for the Moon Base and pressurized rovers; it gives none for a Mars base, which is listed here for its gravity."},
    {"id": "spacesuit", "name": "Spacesuit", "atmosphere": "oxygen 0.99 or more, 4.3 ± 0.5 psi", "gravity": "any",
     "basis": "spacesuits are filled with 100% oxygen to only 4.3 psi"},
]
NEEDS = ["prevention", "detection", "suppression"]

# ------------------------------------------------------------------ the ranking rule
# Order: first by N (how directly NASA ties the investigation to the need), then by the coverage
# score C = 2 A + G + V + D + P. N is not traded against coverage: an investigation NASA names for a
# need is never ranked below one that is only foundational for it.
WEIGHTS = [
    ("N", "ranks first", "need: 3 when the brief names the investigation for the need, 2 when NASA's objective states it, 1 when the "
             "investigation is foundational for it. It is NASA's statement, not this project's, so nothing computed here overrides it."),
    ("A", 2, "atmosphere: 3 when five or more tests lie inside the habitat's oxygen and pressure, 2 when one to four do or NASA's "
             "objective says the atmosphere is examined, 1 when only the oxygen matches, 0 otherwise. Weight 2: flammability depends "
             "strongly on oxygen, and a habitat is defined by its atmosphere."),
    ("G", 1, "gravity: for a microgravity cabin every flight investigation counts 2. For a surface habitat PSI holds no partial-gravity "
             "data, so every investigation counts 0, or 1 when it offers a normal-gravity comparison or mimics partial gravity."),
    ("V", 1, "ventilation: 1 when an imposed flow is part of the tests. Air flow matters in microgravity; crew cabins are ventilated."),
    ("D", 1, "data: 2 when 50 or more tests have a sensor series or processed data, 1 when any has, 0 when only images or video exist."),
    ("P", 1, "predictive support: 2 when a hand-written model of this project is scored on the investigation, 1 when one was tested on it."),
]


def load_jsonl(p):
    return [json.loads(line) for line in io.open(p, encoding="utf-8")]


def val(v):
    if isinstance(v, dict):
        return v.get("value_si") if v.get("status") == "resolved" else None
    return v


def rng(xs, scale=1.0, nd=3):
    xs = [x for x in xs if isinstance(x, (int, float))]
    return {"n": len(xs), "min": round(min(xs) * scale, nd), "max": round(max(xs) * scale, nd)} if xs else {"n": 0}


def clean(text):
    return " ".join((text or "").replace(" ", " ").split())


NARRATIVE_FIELDS = ("how_tests_differ", "sensors_and_measurements", "findings", "fire_safety_relevance", "need_check",
                    "atmospheres_tested", "samples_and_conditions", "atmosphere_and_gravity", "not_found", "documents_read",
                    "written_by", "date")


def narrative(pid):
    """The part of a card that comes from NASA's documents: docs/wsp/atlas/narratives/PSI-<id>.json, schema
    ignis.atlas.narrative/1. Written from page cards by another process; merged here unchanged."""
    path = OUT / "narratives" / f"{pid}.json"
    if not path.exists():
        return {"status": "not written yet", "how_tests_differ": None, "sensors_and_measurements": None, "findings": []}
    n = json.load(io.open(path, encoding="utf-8"))
    assert n.get("schema") == "ignis.atlas.narrative/1" and n.get("investigation_id") == pid, path
    out = {"status": "written", "source_file": f"docs/wsp/atlas/narratives/{pid}.json"}
    out.update({k: n.get(k) for k in NARRATIVE_FIELDS if k in n})
    return out


def loc(src):
    if not src:
        return ""
    where = src.get("file") or ""
    if src.get("page") is not None:
        where += f", p. {src['page']}"
    elif src.get("field"):
        where += f", field {src['field']}"
    return where


def build():
    tests = load_jsonl(ATLAS / "tests.jsonl")
    fsum = json.load(io.open(ATLAS / "files_summary.json", encoding="utf-8"))["investigations"]
    by = defaultdict(list)
    for r in tests:
        by[r["investigation_id"]].append(r)
    cards = {}
    for pid in ORDER:
        m = json.load(io.open(META / f"{pid}.json", encoding="utf-8"))
        t = TYPED[pid]
        rs = by.get(pid, [])
        f = fsum.get(pid, {})
        # ---- computed from the test catalogue (conditions only)
        fuels = Counter(str(val(r["fuel"].get("name"))) for r in rs)
        phases = Counter(str(val(r["fuel"].get("phase"))) for r in rs)
        configs = Counter(str(val(r["geometry"].get("configuration"))) for r in rs)
        sizes = defaultdict(list)
        for r in rs:
            for s in r["geometry"].get("sizes") or []:
                v = val(s.get("value"))
                if isinstance(v, (int, float)):
                    sizes[s.get("dimension")].append(v)
        amb = [r["map"].get("x_o2") for r in rs if r["map"].get("x_o2_role") == "ambient"]
        inj = [r["map"].get("x_o2") for r in rs if r["map"].get("x_o2_role") == "injected"]
        pres = [r["map"].get("pressure_pa") for r in rs]
        speed = [val(r["flow"].get("speed")) for r in rs]
        varied = Counter(v for r in rs for v in (r["varied"] or []))
        outcomes = sorted({o["name"] for r in rs for o in (r["outcome_fields_available"] or [])})
        inside = Counter(a for r in rs for a in (r["map"].get("crew_atmospheres") or []))
        o2only = Counter(a for r in rs for a in (r["map"].get("crew_atmospheres_o2_only") or []))
        o2_30 = sum(1 for x in amb if isinstance(x, (int, float)) and x >= 0.30)
        levels = Counter(r["level"] for r in rs)
        # ---- computed from the file catalogue (names and sizes only)
        schemes = f.get("id_schemes") or {}
        with_kind = Counter()
        for sch in schemes.values():
            for k in ("with sensor time series", "with image archive", "with video", "with processed or analysed data"):
                with_kind[k] = max(with_kind[k], sch.get(k, 0))
        if not schemes:
            for k, v in (f.get("test_days_with") or {}).items():
                with_kind[k] = v
        n_ids = max([s.get("ids", 0) for s in schemes.values()] or [0])
        numeric = max(with_kind["with sensor time series"], with_kind["with processed or analysed data"])
        flow_tests = len([x for x in speed if isinstance(x, (int, float))])
        cards[pid] = {
            "schema": "ignis.atlas.card/1",
            "investigation_id": pid,
            "acronym": m.get("investigationAcronym"),
            "title": m.get("title"),
            "doi": m.get("doi"),
            "licence": m.get("licenseIdentifier") or None,
            "nasa_sub_area": m.get("subResearchArea") or None,
            "flight_program": m.get("flightProgram") or None,
            "hardware": {"nasa_record": m.get("experimentHardware"), "family": t["family"][0], "family_basis": t["family"][1]},
            "project_period_nasa_record": {"start": m.get("investigationStartDate") or None, "end": m.get("investigationEndDate") or None},
            "nasa_text": {"source": f"PSI metadata record of {pid}", "objective": clean(m.get("objective")),
                          "hypothesis": clean(m.get("hypothesis")), "approach": clean(m.get("approach")),
                          "research_impacts": clean(m.get("researchImpacts"))},
            "publications_listed_by_nasa": len(m.get("publications") or []),
            "tests": {
                "source": "test catalogue (docs/wsp/atlas/SCHEMA.md); conditions only",
                "rows": len(rs), "level": dict(levels),
                "row_scope": ROW_SCOPE.get(pid, "every row of NASA's experiment table"),
                "fuels_or_materials": dict(fuels.most_common()),
                "fuel_phase": dict(phases), "configuration": dict(configs),
                "sizes_m": {k: rng(v, nd=5) for k, v in sorted(sizes.items())},
                "ambient_oxygen": rng(amb), "injected_oxidiser_oxygen": rng(inj),
                "pressure_kpa": rng(pres, 1e-3, 1), "flow_speed_m_s": rng(speed),
                "varied_between_tests": [k for k, _ in varied.most_common()],
                "outcome_fields_in_the_table": outcomes,
                "tests_at_30_percent_oxygen_or_more": o2_30,
            },
            "files": {
                "source": "file catalogue (docs/wsp/atlas/FILES.md); names and sizes only",
                "distinct_files": f.get("distinct_files"), "bytes": f.get("distinct_bytes"),
                "by_medium": f.get("by_medium"), "test_ids_in_file_names": n_ids,
                "tests_with": dict(with_kind), "tests_with_numeric_series": numeric,
            },
            "crew_atmospheres": {h["id"]: {"tests_inside": inside.get(h["id"], 0), "tests_with_matching_oxygen_only": o2only.get(h["id"], 0),
                                           "stated_by_nasa": (t.get("stated_atmospheres") or {}).get(h["id"])} for h in HABITATS},
            "gravity": {"level": "microgravity", "normal_gravity_comparison": t.get("normal_gravity"),
                        "partial_gravity_proxy": t.get("partial_gravity_proxy"), "partial_gravity_data_in_psi": Q_NOPG},
            "imposed_flow": {"tests_with_a_flow_speed": flow_tests, "stated": t.get("flow")},
            "fire_safety": t["needs"],
            "brief_note": t.get("brief_note"),
            "predictive": {k: v for k, v in t["predictive"].items() if k != "points"},
            "narrative": narrative(pid),
            "_score_parts": {"numeric": numeric, "flow": 1 if (flow_tests or t.get("flow")) else 0, "P": t["predictive"]["points"]},
        }
    # ---- related investigations, by rule
    for pid, c in cards.items():
        rel = []
        for other, o in cards.items():
            if other == pid:
                continue
            shared = []
            if o["hardware"]["family"] == c["hardware"]["family"]:
                shared.append("same hardware family")
            common = sorted((set(c["tests"]["fuels_or_materials"]) & set(o["tests"]["fuels_or_materials"])) - {"None"})
            if common:
                shared.append("same fuel or material: " + ", ".join(common))
            needs = sorted({n["need"] for n in c["fire_safety"]} & {n["need"] for n in o["fire_safety"]})
            if needs and shared:
                shared.append("same need: " + ", ".join(needs))
            if shared:
                rel.append({"investigation_id": other, "acronym": o["acronym"], "shared": shared})
        rel.sort(key=lambda x: (-len(x["shared"]), ORDER.index(x["investigation_id"])))
        c["related"] = rel[:5]
    return cards


def rank(cards):
    out = {}
    for h in HABITATS:
        per_need = {}
        for nd in NEEDS:
            rows = []
            for pid, c in cards.items():
                hit = [n for n in c["fire_safety"] if n["need"] == nd]
                if not hit:
                    continue
                n = hit[0]
                ca = c["crew_atmospheres"][h["id"]]
                if ca["tests_inside"] >= 5:
                    a, why_a = 3, f"{ca['tests_inside']} tests inside this atmosphere"
                elif ca["tests_inside"] >= 1:
                    a, why_a = 2, f"{ca['tests_inside']} tests inside this atmosphere"
                elif ca["stated_by_nasa"]:
                    a, why_a = 2, "NASA's objective says this atmosphere is examined; the table gives no pressure per test"
                elif ca["tests_with_matching_oxygen_only"] >= 1:
                    a, why_a = 1, f"{ca['tests_with_matching_oxygen_only']} tests match the oxygen; pressure unknown or outside"
                else:
                    a, why_a = 0, "no test at this atmosphere in the catalogue"
                if h["gravity"] == "microgravity":
                    g, why_g = 2, "microgravity data"
                elif h["gravity"] == "partial gravity":
                    if c["gravity"]["partial_gravity_proxy"]:
                        g, why_g = 1, "mimics partial gravity with an electric field"
                    elif c["gravity"]["normal_gravity_comparison"]:
                        g, why_g = 1, "offers a normal-gravity comparison, which with microgravity bounds partial gravity"
                    else:
                        g, why_g = 0, "PSI holds no partial-gravity data"
                else:
                    g, why_g = 0, "gravity is not what defines this habitat"
                v = c["_score_parts"]["flow"]
                num = c["_score_parts"]["numeric"]
                d = 2 if num >= 50 else 1 if num >= 1 else 0
                p = c["_score_parts"]["P"]
                cover = 2 * a + g + v + d + p
                rows.append({
                    "investigation_id": pid, "acronym": c["acronym"], "tier": n["tier"], "coverage": cover,
                    "parts": {"N": TIER[n["tier"]], "A": a, "G": g, "V": v, "D": d, "P": p},
                    "why": {"need": f"{n['tier']}: \"{n['basis']}\"", "atmosphere": why_a, "gravity": why_g,
                            "ventilation": "an imposed flow is part of the tests" if v else "no imposed flow recorded",
                            "data": f"{num} tests with a sensor series or processed data" if num else "images or video only: values have to be measured first",
                            "predictive": c["predictive"]["status"] + ": " + c["predictive"]["detail"]},
                })
            rows.sort(key=lambda r: (-r["parts"]["N"], -r["coverage"], -r["parts"]["A"], ORDER.index(r["investigation_id"])))
            for i, r in enumerate(rows):
                r["rank"] = i + 1
            per_need[nd] = rows
        out[h["id"]] = per_need
    return {"schema": "ignis.atlas.habitat_ranking/1", "habitats": HABITATS, "needs": NEEDS,
            "order": "by N, then by the coverage score C = 2 A + G + V + D + P", "weights": [{"part": k, "weight": w, "meaning": why} for k, w, why in WEIGHTS],
            "ranking": out,
            "limits": ["The ranking orders what PSI holds for a need and a habitat; it does not say that a habitat is safe.",
                       "Need tiers and the flags for gravity and flow are typed judgements, each with the sentence of NASA's it rests on.",
                       "Atmosphere counts use the test catalogue, where many tables give no pressure per test; a count of zero can mean unknown.",
                       "No test in PSI lies at the spacesuit atmosphere, and none at partial gravity."]}


def md_cards(cards):
    L = ["# The 19 flight combustion investigations in NASA's PSI repository", "",
         "One card per investigation, built by `tools/wsp/atlas/build_cards.py`. Objective and approach are NASA's own words from the",
         "PSI metadata record. Counts and ranges come from the test catalogue and the file catalogue of this folder (conditions, names",
         "and sizes; no outcome of a test was read). The fire-safety need of each investigation is a typed judgement, given with the",
         "sentence of NASA's it rests on. Findings from NASA's documents are not in the cards yet.", ""]
    for pid, c in cards.items():
        t, f = c["tests"], c["files"]
        L += [f"## {c['acronym']} ({pid})", "", f"**{c['title']}**. {c['hardware']['nasa_record']}. Family: {c['hardware']['family']}."
              + (f" DOI {c['doi']}." if c["doi"] else "") + (f" Licence {c['licence']}." if c["licence"] else " Licence not stated in NASA's record."), "",
              f"**Objective (NASA).** {c['nasa_text']['objective']}", ""]
        if c["nasa_text"]["approach"]:
            L += [f"**Approach (NASA).** {c['nasa_text']['approach']}", ""]
        if c["nasa_text"]["research_impacts"]:
            L += [f"**Impacts, on Earth and in space (NASA).** {c['nasa_text']['research_impacts']}", ""]
        L += ["| | |", "|---|---|"]
        fuels = ", ".join(f"{k} ({v})" for k, v in list(t["fuels_or_materials"].items())[:8]) or "not in the catalogue"
        L.append(f"| Fuels or materials (rows) | {fuels} |")
        L.append(f"| Phase and configuration | {', '.join(k for k in t['fuel_phase'] if k != 'None') or 'unknown'}; {', '.join(k for k in t['configuration'] if k != 'None') or 'unknown'} |")
        if t["sizes_m"]:
            L.append("| Sizes | " + "; ".join(f"{k.replace('_', ' ')} {v['min'] * 1e3:g} to {v['max'] * 1e3:g} mm" for k, v in t["sizes_m"].items() if v["n"]) + " |")
        ao, io_, pr, fl = t["ambient_oxygen"], t["injected_oxidiser_oxygen"], t["pressure_kpa"], t["flow_speed_m_s"]
        L.append("| Ambient oxygen | " + (f"{ao['min']:.2f} to {ao['max']:.2f} ({ao['n']} rows)" if ao["n"] else "not in the table") + " |")
        if io_["n"]:
            L.append(f"| Oxygen in the injected stream | {io_['min']:.2f} to {io_['max']:.2f} ({io_['n']} rows) |")
        L.append("| Pressure | " + (f"{pr['min']:.0f} to {pr['max']:.0f} kPa ({pr['n']} rows)" if pr["n"] else "not in the table") + " |")
        L.append("| Imposed flow | " + (f"{fl['min'] * 100:.0f} to {fl['max'] * 100:.0f} cm/s ({fl['n']} rows)" if fl["n"] else ("stated by NASA, not per test" if c["imposed_flow"]["stated"] else "none recorded")) + " |")
        L.append(f"| What varied between tests | {', '.join(v.split('.')[-1].replace('_', ' ') for v in t['varied_between_tests'][:8]) or 'not in the catalogue'} |")
        lv = ", ".join(f"{v} {k.replace('_', ' ')}s" for k, v in t["level"].items()) or "no row"
        L.append(f"| Catalogue rows | {lv} ({t['row_scope']}) |")
        w = f["tests_with"]
        L.append(f"| Files at NASA | {f['distinct_files']:,} files, {f['bytes'] / 1e9:,.1f} GB; test ids in file names: {f['test_ids_in_file_names']} |")
        L.append(f"| Measurements per test | sensor series {w.get('with sensor time series', 0)}, images {w.get('with image archive', 0)}, video {w.get('with video', 0)}, processed data {w.get('with processed or analysed data', 0)} |")
        ca = c["crew_atmospheres"]
        L.append("| Crew atmospheres | " + "; ".join(
            f"{h['id'].replace('_', ' ')}: {ca[h['id']]['tests_inside']} inside" + (f", {ca[h['id']]['tests_with_matching_oxygen_only']} oxygen only" if ca[h['id']]['tests_with_matching_oxygen_only'] else "")
            + (", stated by NASA" if ca[h['id']]['stated_by_nasa'] else "") for h in HABITATS) + " |")
        L.append("| Fire-safety need | " + "; ".join(f"{n['need']} ({n['tier']})" for n in c["fire_safety"]) + " |")
        g = c["gravity"]
        L.append("| Towards partial gravity | " + ("mimics it with an electric field" if g["partial_gravity_proxy"] else "normal-gravity comparison" if g["normal_gravity_comparison"] else "nothing") + " |")
        L.append(f"| Hand-written models | {c['predictive']['status']}: {c['predictive']['detail']} |")
        if c["related"]:
            L.append("| Closest investigations | " + "; ".join(f"{r['acronym']} ({', '.join(r['shared'])})" for r in c["related"][:3]) + " |")
        L.append("")
        nv = c["narrative"]
        if nv["status"] == "written":
            if (nv.get("how_tests_differ") or {}).get("text"):
                L += [f"**How the tests differ.** {nv['how_tests_differ']['text']}", ""]
            if nv.get("sensors_and_measurements"):
                L += ["**Instruments.** " + "; ".join(f"{x['instrument']} ({x['measures']})" for x in nv["sensors_and_measurements"]) + ".", ""]
            if nv.get("findings"):
                L.append("**Findings, from NASA's documents.**")
                L.append("")
                for x in nv["findings"]:
                    L.append(f"- {x['statement']} *({x['kind']}; {loc(x.get('source'))})*")
                L.append("")
            if (nv.get("fire_safety_relevance") or {}).get("text"):
                L += [f"**Why it matters for fire safety** ({nv['fire_safety_relevance'].get('kind', 'interpretation')}). {nv['fire_safety_relevance']['text']}", ""]
        for n in c["fire_safety"]:
            L.append(f"- *{n['need']}, {n['tier']}:* \"{n['basis']}\"")
        if c.get("brief_note"):
            L.append(f"- *the brief also says:* \"{c['brief_note']}\"")
        L.append("")
    return "\n".join(L)


def md_rank(r, cards):
    L = ["# Which investigations serve which habitat", "",
         "Built by `tools/wsp/atlas/build_cards.py` from the cards of this folder. The ranking is a rule, not a similarity search:",
         f"investigations are ordered {r['order']}.", "", "| Part | Weight | What it measures, and why it has that weight |", "|---|---|---|"]
    for w in r["weights"]:
        L.append(f"| {w['part']} | {w['weight']} | {w['meaning']} |")
    L += ["", "Limits:", ""] + [f"- {x}" for x in r["limits"]] + [""]
    for h in r["habitats"]:
        L += [f"## {h['name']}", "", f"Atmosphere: {h['atmosphere']}. Gravity: {h['gravity']}. From the brief: \"{h['basis']}\"" + (f" {h['note']}" if h.get("note") else ""), ""]
        for nd in r["needs"]:
            rows = r["ranking"][h["id"]][nd]
            L += [f"### {nd.capitalize()}", "", "| Rank | Investigation | Tie to the need | Coverage | A | G | V | D | P | Why |", "|---|---|---|---|---|---|---|---|---|---|"]
            for x in rows:
                p = x["parts"]
                why = f"{x['why']['atmosphere']}; {x['why']['data']}"
                L.append(f"| {x['rank']} | {x['acronym']} ({x['investigation_id']}) | {x['tier']} | {x['coverage']} | {p['A']} | {p['G']} | {p['V']} | {p['D']} | {p['P']} | {why} |")
            L.append("")
    return "\n".join(L)


def main():
    cards = build()
    ranking = rank(cards)
    (OUT / "cards").mkdir(parents=True, exist_ok=True)
    for pid, c in cards.items():
        pub = {k: v for k, v in c.items() if not k.startswith("_")}
        io.open(OUT / "cards" / f"{pid}.json", "w", encoding="utf-8", newline="\n").write(json.dumps(pub, indent=1, ensure_ascii=False) + "\n")
    io.open(OUT / "habitat_ranking.json", "w", encoding="utf-8", newline="\n").write(json.dumps(ranking, indent=1, ensure_ascii=False) + "\n")
    io.open(OUT / "CARDS.md", "w", encoding="utf-8", newline="\n").write(md_cards(cards) + "\n")
    io.open(OUT / "HABITAT_RANKING.md", "w", encoding="utf-8", newline="\n").write(md_rank(ranking, cards) + "\n")
    for h in HABITATS:
        for nd in NEEDS:
            top = ranking["ranking"][h["id"]][nd][:4]
            print(f"{h['id']:16s} {nd:11s} " + "  ".join(f"{x['acronym']} [{x['parts']['N']}|{x['coverage']}]" for x in top))
    print("cards:", len(cards))


if __name__ == "__main__":
    main()
