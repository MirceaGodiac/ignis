# Served WSP models

IGNIS serves fourteen forecasting models built by weight-space programming (WSP). An author model,
Claude Fable 5.2 (`claude-fable-5-2`), typed every parameter by hand as a literal assignment in an
`edits.wsp` file. `ignis wsp compile` turns that file into a patch, and `ignis wsp assemble` builds an
immutable release whose id is the sha256 of its record. A frozen Rust forward in
`crates/ignis-wsp/src/` then runs it. At request time nothing is fitted, sampled or searched, and the
uncertainty bands are typed by the author too (`assumed_noise`). Every release has
`trained_on = not_trained` and `state = experimental`, and its output is model output, not a
measurement.

Each served release is listed in [`configs/models/served/served.json`](../../../configs/models/served/served.json),
with its bytes in `configs/models/served/models/<hex>/`. The model card for each one is served at
`GET /api/v1/model-cards`. `cargo test -p ignis-wsp-serve --test served_ids` checks that every
served id is the content hash of its record and loads through the registry checks; it passed, 2 of 2,
on 15 November 2026 at ignis-web a58c7c3.

Each model has one page, and every page follows the same steps of the WSP process: 1 task and
targets, 2 physical mechanism, 3 authoring, 4 weights, 5 forward and assembly, 6 evaluation,
7 adoption and lineage, and 8 how to call the model in IGNIS. The full derivations are in
[docs/research/FORMULAS.md](../../research/FORMULAS.md) and the author briefs in
[docs/research/PROMPTS.md](../../research/PROMPTS.md).

## Index

The headline is the release's best-sourced score against its baseline, where lower is better. The
"test" scores come from data no author of that release had read, scored once:
`SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL.md` (FLEX, CFI-G, BASS-II) and `RESULTS_FINAL_GEN7.md`
(seventh generation, test days on a frozen list). The "validation" scores come from the development
validation partition. An "in-sample" score was made with every partition seen. Research paths are on
the lane branch `feat/handwritten-science` (542dad8).

| Model | Investigation (PSI) | Family | What it forecasts | Generation | Author | Zero-shot / data-informed | Release (sha256) | Headline score |
|---|---|---|---|---|---|---|---|---|
| [flex_methanol.wsp2](flex_methanol.wsp2.md) D3 | FLEX, methanol droplets (PSI-69) | flex | Droplet diameter after the cutoff (quantiles), through hot flame, flame loss and evaporation | third (13 Nov) | claude-fable-5-2 | data-informed (train) | `55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce` | Test CRPS **18.5 µm**, D²-law 31.6, zero-shot W0 37.2, best fitted 42.7 (`RESULTS_FINAL.md`). Validation 40.0 µm, best fitted 77.7 (`docs/wsp/README.md`) |
| [flex_heptane.wsp2](flex_heptane.wsp2.md) D2 | FLEX, n-heptane droplets (PSI-69); transfer to FLEX-2 (PSI-68) | flex | Droplet diameter after the cutoff (quantiles), through hot flame, cool flame and no flame | third (13 Nov) | claude-fable-5-2 | data-informed (train) | `43f7871f778040550291789ca97e319729675a9dd0df9c9529525c1805ad588d` | Test CRPS **32.2 µm**, D²-law 54.0 (`RESULTS_FINAL.md`). Validation 46.1 µm, W0 60.3, D²-law 105. FLEX-2 20.9 against 32.6 |
| [acme_normal.wsp13](acme_normal.wsp13.md) F4 (f4-hydrogen) | ACME CFI-G (PSI-159), Flame Design (PSI-10), s-Flame (PSI-23), normal flames | acme_normal | Probability over time that a normal burner flame ends by extinction or by a flow stop | seventh (typed under PROTOCOL 13.6, served under 13.8) | claude-fable-5-2; transcribed by claude-opus-5-5 | data-informed | `4cc2fa60cc8c40b316594034ab8f299946117705736677e941c99f645fb89436` | Helium flames, validation days: sum **0.111**, incidence 0.330, in-sample. Hydrogen 0.314 against 0.909, in-sample. Nitrogen ethane, propane, butane and ethylene are F3 stage 1: CFI-G validation 0.154 / 0.096 (`feedback/sflame_f4_result.md`, `docs/wsp/README.md`) |
| [acme_inverse.wsp12](acme_inverse.wsp12.md) F2 | ACME CFI-G (PSI-159) and Flame Design (PSI-10), inverse flames | acme_inverse | Probability over time that an inverse burner flame ends by extinction or by a flow stop | sixth (15 Nov) | claude-fable-5-2 | data-informed (ethylene row zero-shot) | `dff7da0da8e77785acecc0ffb5acaf222358d4274b36e2c52f39426aaebd594e` | Test extinction **0.0998** on 93 cases. On the 69 shared cases, 0.1045 against 0.1645 for the fitted hazard and 0.2233 for the constant (`RESULTS_FINAL.md`) |
| [bass_front.wsp](bass_front.wsp.md) D1 | BASS-II (PSI-25), thin fabric in concurrent flow | bass | Flame-tip position after the cutoff (quantiles), with the fan history as a known control | fourth (14 Nov) | claude-fable-5-2 | data-informed (four train burns, theory) | `ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c` | Validation tip CRPS **0.117 sample widths**, persistence 0.234, 12 burns measured blind. Saffire transfer 0.058 against 0.250 (`docs/wsp/README.md`, `COVERAGE_19.md`) |
| [bass5.wsp](bass5.wsp.md) S1 | BASS and BASS-II plastics and fabrics (PSI-26, PSI-25) | other (law) | Whether the sample keeps burning; quench and blow-off speeds; oxygen limit | seventh (PROTOCOL 13.8) | claude-fable-5-2; transcribed by claude-opus-5-5 | data-informed; in-sample on all data | `050a4ceb6c094b9256c5568408f8574508530ce1b31ce45eae43e8785fa4b82d` | Kept burning: test **0.048**, baseline 0.071, 42 burns, identical to bass4. Quench speed 0.114 / 0.108 / 0.126 cm/s against a constant 0.671 / 0.547 / 0.916, in-sample (`RESULTS_FINAL_GEN7.md`, `feedback/bass5_post_final_result.md`) |
| [same.wsp](same.wsp.md) W0 | SAME (PSI-102), smoke aerosol | other (law) | Smoke size d10 and d30; ISS detector at full scale | seventh (PROTOCOL 13.2) | claude-fable-5-2; transcribed by claude-opus-5-5 | zero-shot | `e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062` | Whole investigation held: diameter **0.184**, beating both baselines (0.220 and 0.192). Detector 0.059 against 0.205 for a constant (`COVERAGE_19.md`) |
| [daft.wsp](daft.wsp.md) W0 | DAFT (PSI-47), dust and aerosol | other (law) | Number concentration a condensation counter reads, from the photometer reading | seventh (PROTOCOL 13.2) | claude-fable-5-2; transcribed by claude-opus-5-5 | zero-shot | `061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d` | Whole investigation held: **0.777** against 2.678 for the one baseline (`COVERAGE_19.md`) |
| [cld.wsp](cld.wsp.md) D1 | ACME CLD Flame (PSI-21) | other (law) | When the flame is lost while fuel flows (event); radiometer excess | seventh (PROTOCOL 13.3) | claude-fable-5-2; transcribed by claude-opus-5-5 | data-informed (train) | `a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f` | Test days: flame lost **0.073**, incidence 0.102. Radiometer 0.115 against 0.144 (`RESULTS_FINAL_GEN7.md`) |
| [bre.wsp](bre.wsp.md) D3 | ACME BRE (PSI-20), porous-disc burner | other (law) | Whether the flame still burns a stated time after ignition, hence the chance it has gone out | seventh (PROTOCOL 13.6) | claude-fable-5-2; transcribed by claude-opus-5-5 | data-informed (development days) | `27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955` | Test days, extinction: **0.139**, incidence 0.222, zero-shot W0 0.245 (`RESULTS_FINAL_GEN7.md`) |
| [efield3.wsp](efield3.wsp.md) S1 | E-FIELD Flames (PSI-22) | other (law) | Ion current at a planned voltage; flame-out under the field; peak current and its voltage | seventh (PROTOCOL 13.6) | claude-fable-5-2; transcribed by claude-opus-5-5 | data-informed (development days) | `8575550b310950efde7da2f00a1964c9a4d0c393fd3fbcaf4e4f2f1f8f046f04` | Test days: ion current **0.319 µA**, space-charge baseline 0.575. Flame out in window 0.055 against 0.133 (`RESULTS_FINAL_GEN7.md`) |
| [slice2.wsp](slice2.wsp.md) S1 | SLICE (PSI-106), lifted and blown-off jet flames | other (law) | Blow-off, past the lifting and blow-off limits, lift noted; flame length | seventh (PROTOCOL 13.4) | claude-fable-5-2; transcribed by claude-opus-5-5 | data-informed (development days) | `d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61` | Lifting on development lines: **0.0333**, zero-shot W0 0.1474, lower on 8 of 8 days. The 31 test lines hold no event (`feedback/slice2_second_law_result.md`, `RESULTS_FINAL_GEN7.md`) |
| [spice.wsp](spice.wsp.md) P3 | SPICE (PSI-107), smoke points of jet flames | other (law) | Smoke-point length and fuel flow; flame length; whether soot leaves the tip | seventh (PROTOCOL 13.6) | claude-fable-5-2; transcribed by claude-opus-5-5 | data-informed (published fit read) | `000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0` | Test smoke-point length **4.18 mm**, per-fuel baseline 10.3. Flame length 1.18 against 2.56 for the Lin and Faeth law, 6 points (`RESULTS_FINAL_GEN7.md`) |
| [samer.wsp](samer.wsp.md) S1 | SAME-R (PSI-101), smoke aerosol | other (law) | Smoke size d10 and d30; ISS detector at full scale | seventh (PROTOCOL 13.6) | claude-fable-5-2; transcribed by claude-opus-5-5 | data-informed (development days) | `e33ee7665b599e7e6c8c978f61ebf3bf106b2b6a94c3a0821dbdf42cf47a1b5f` | Test days: diameter **0.167**, best baseline 0.256. Detector 0.014 against 0.019 (`RESULTS_FINAL_GEN7.md`) |

"Transcribed by claude-opus-5-5" means the author wrote the numbers in a research dossier, and the
assembly engineer, an Opus instance, copied them verbatim into `edits.wsp` without choosing,
rounding or changing any number. That is how each release's `meta.json` rationale words it.

## Forwards

| Forward (`crates/ignis-wsp/src/`) | Served models |
|---|---|
| `droplet3.rs` | flex_methanol.wsp2, flex_heptane.wsp2 |
| `committee4.rs` | acme_normal.wsp13 |
| `committee3.rs` | acme_inverse.wsp12 |
| `front.rs` | bass_front.wsp |
| `law.rs`, the generic law forward with one slug per investigation (`law_slugs.rs`, `configs/models/law/<slug>/decl.json`) | bass5, same, daft, cld, bre, efield3, slice2, spice, samer |

## Notes on the sources

Each page gives the value recorded in the release itself. The served card texts were corrected to
match the records on 15 Nov 2026 (fuels admitted by `acme_inverse.wsp12`, the methanol flame states,
the data each revision read, the F3 origin of one `acme_normal.wsp13` score).

- **`bass5.wsp`.** Its 42 burns are the cases answered; 40 of them have an outcome on the corrected
  coding.
- **`lineage_note` fields.** In several release records this field is cut off; the `meta.json`
  rationales hold the full text.
