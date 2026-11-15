# Hand-written scientific models (literal weight-space programming)

**Start with [WSP_GUIDE.md](WSP_GUIDE.md).** It is the technical guide: what WSP is and what it has
achieved; the forecasting tasks; the anatomy of a model (spec, edits, forwards, committees);
assembly and content-addressed release ids; model cards; the serving path; the strictness rules;
evaluation; how to author a new model; and how to rebuild releases and scores.

## In this folder

| Read | For |
|---|---|
| [WSP_GUIDE.md](WSP_GUIDE.md) | **The technical guide**, from what WSP is to the exact rebuild commands |
| [generations 1 to 5](EXPERIMENTS_GEN1-5.md) and [generation 6 onward](EXPERIMENTS_GEN6-ON.md) | The NASA investigations: apparatus, data, and what each model forecasts |
| [PROTOCOL.md](PROTOCOL.md) | Spec, edit language, assembler, release record, zero-shot and data-informed candidates, selection rules, commands, generations one to six (the full text through the seventh generation is docs/wsp/PROTOCOL.md) |
| [RESULTS.md](RESULTS.md) | Development scores of every candidate against the baselines, generations one to four |
| [RESULTS_GEN5.md](RESULTS_GEN5.md) | The registered validation look of the fifth generation |
| [RESULTS_GEN6.md](RESULTS_GEN6.md) | The sixth generation: the look on CFI-G validation and Flame Design that adopted `acme_normal.wsp7` P3 |
| [cards/](cards/) | One model card per model: computation, what each number says and where it comes from, provenance, limits |
| [research/](research/) | [law_forward.md](research/law_forward.md) (the generic law forward of the seventh generation) and [phys_quench_w0.md](research/phys_quench_w0.md) (the fabric ignition and quench law) |
| [atlas/](atlas/) | [SERVER_STORAGE.md](atlas/SERVER_STORAGE.md): what the server would store from NASA's archive for the 19 investigations |

## What is served

The served list is [`configs/models/served/served.json`](../../configs/models/served/served.json), and
the release bytes are in `configs/models/served/models/<hex>/`. Every score below is quoted with its
source in the release's card.

| Investigation | Release | Candidate | Authorship | Headline score |
|---|---|---|---|---|
| FLEX methanol (PSI-69) | `flex_methanol.wsp2` `sha256:c8a90dab…b654` | D3 | data-informed | sealed test CRPS **18.5 µm** against 31.6 for the d-squared law and 42.7 for the best fitted model |
| FLEX n-heptane (PSI-69) | `flex_heptane.wsp2` `sha256:edbad4d6…edef` | D2 | data-informed | sealed test CRPS **32.2 µm** against 54.0; on FLEX-2, unchanged, 20.9 against 32.6 |
| Normal burner flames: CFI-G, Flame Design, s-Flame (PSI-159, 10, 23) | `acme_normal.wsp13` `sha256:54caac19…bda9` | F4 (`f4-hydrogen`) | data-informed; the helium and hydrogen rows were typed after the final run | F2, its ancestor, on the sealed CFI-G test: extinction **0.0414** against 0.0933 for the fitted linear baseline; Flame Design test days 0.2133 against 0.4244 for incidence |
| Inverse burner flames: CFI-G, Flame Design | `acme_inverse.wsp12` `sha256:7404f3ad…a7f7` | F2 | data-informed; the ethylene row zero-shot | sealed CFI-G test, extinction **0.1045** against 0.1645 for the fitted hazard model |
| BASS-II fabric (PSI-25), carried to Saffire | `bass_front.wsp` `sha256:ac15e49f…855c` | D1 | data-informed | tip CRPS **0.117** sample widths against 0.234 for persistence on 12 burns measured blind; 0.121 against 0.449 for Ferkul's published fit |
| BRE (PSI-20) | `bre.wsp` `sha256:27e2af51…b955` | D3 | data-informed | test days: extinction **0.139** against 0.222 |
| CLD Flame (PSI-21) | `cld.wsp` `sha256:a03ec603…2d4f` | D1 | data-informed | test days: flame lost **0.073** against 0.102 |
| E-FIELD (PSI-22) | `efield3.wsp` `sha256:8575550b…6f04` | S1 | data-informed | test days: ion current **0.319 µA** against 0.575 |
| SLICE (PSI-106) | `slice2.wsp` `sha256:d586d0d3…0b61` | S1 | data-informed | test days held no blow-off or lifting event |
| SPICE (PSI-107) | `spice.wsp` `sha256:000b88b8…98a0` | P3 | data-informed | test days: smoke-point length **4.18 mm** against 10.33 |
| SAME-R (PSI-101) | `samer.wsp` `sha256:e33ee766…1b5f` | S1 | data-informed | test days: particle diameter **0.167** against 0.256 |
| SAME (PSI-102) | `same.wsp` `sha256:e1362b55…2062` | W0 | **zero-shot** | whole investigation held: **0.184** against 0.192 for the best baseline |
| DAFT (PSI-47) | `daft.wsp` `sha256:061f93de…5e5d` | W0 | **zero-shot** | whole investigation held: **0.777** against 2.678 |
| Plastics and fabrics: BASS, BASS-II (PSI-26, 25) | `bass5.wsp` `sha256:050a4ceb…b82d` | S1 | data-informed; typed after the final run | kept burning, test days: **0.048** against 0.071 |

**Which numbers are zero-shot.** Only W0 candidates (and Q0, a zero-shot law on a data-informed
forward) and the reference baselines. A candidate named D, P, F or S, and every phase-1 W1 and W2, was
typed after its author read development data. Analysis and video measurement were delegated to other
model instances; no script and no delegate wrote a weight. Each candidate was committed before its
validation score existed, and that score decided only whether it replaced the model before it. The
test partitions were sealed until one final run on frozen release lists.

## Seven generations, in one paragraph each

- **Phase 1** wrote one model per family from theory alone (W0), then revised it against development
  feedback (W1, W2). The zero-shot FLEX W0 beat both fitted d-squared baselines on validation.
- **Phase 2** rebuilt the models around what the experiments do: a droplet is in a hot flame, a cool
  flame or no flame, and a burner flame ends on physical clocks.
- **The third generation** audited those models against their train partitions and revised the laws.
  It produced the carried droplet models.
- **The fourth generation** wrote the burner flames as the physics of a growing spherical flame with a
  burner-heating law, measured the BASS burns from the public videos, wrote the first BASS model that
  met real data, and added the model service.
- **The fifth generation** rebuilt the laws from the physical mechanism of each experiment and added a
  committee of member laws that reads the planned stop of a test from NASA's script file.
- **The sixth generation** (the night of 15 November) introduced blind-first scoring and
  typed back-propagation. It adopted new normal-flame models, a typed fuel table that answers for
  ethylene and methane, and zero-shot candidates for six new droplet fuels.
- **The seventh generation** (15 November, day) gave every remaining investigation a hand-written law
  on one generic strict forward (`law.rs`), scored each once blind, revised from the train days, and
  ran the final test.

Every candidate, release and feedback file is preserved, and its release id is unchanged: a forward is
frozen with its releases, and a better forward is a new file.

**What carried to unseen data.** Mechanisms written in quantities that mean the same thing in another
session: the three droplet states, the reduced flow, the burner-heating law, the band-shaped flame.
Knowing the plan of a test: the planned stop from the script file moved the flow-termination score of
inverse flames on unseen days from 0.024 to 0.002. A second scenario carried beside the first with a
stated probability: inverse-flame extinction went from 0.161 to 0.129. Physics carried to a new fuel:
lean inverse ethylene flames last about twice the clock (1.7 observed).

## Layout

```text
crates/ignis-wsp/              edit language, assembler, strict forwards, model service; depends on ignis-core only
  nn.rs quad.rs common.rs spec.rs   shared kernels (part of every forward identity)
  text.rs assemble.rs rewrite.rs    edit language, assembler, literal rewrite for a new spec
  audit.rs                          forward identity (forward_hash) and the strict-route check
  flex.rs acme.rs bass.rs           first-generation forwards
  droplet*.rs                       droplet forwards (droplet3.rs: the served droplet models)
  flame*.rs fuel.rs fuel2.rs        burner-flame member laws and fuel tables
  blend.rs committee*.rs            committees of member releases (committee3.rs, committee4.rs: the served burner flames)
  front*.rs                         fabric flame-front forwards (front.rs: the served fabric model)
  law.rs law_slugs.rs law_tools.rs  the generic law forward and its slugs (the seventh-generation laws)
  models.rs                         literal models by key; a release finds its forward by model_name
  service.rs                        WspModelService: the ModelService of the literal models
  cli.rs                            ignis wsp spec | compile | check | assemble | rewrite | inspect | forward | forward-hash | law-root
crates/ignis-wsp-serve/        ServedModels: serves the releases of configs/models/served/served.json
configs/models/                <family>/<wspN>/<fuel or flame type>/<candidate>/ and law/<slug>/<candidate>/
  <candidate>/                   edits.wsp, meta.json, patch.json, release.json
configs/models/served/         served.json and models/<hex>/ (release.json, params.npz)
tools/wsp/                     reassemble.py (rebuild every chain), verify_served.py (rebuild the served ids), bass_optical/
docs/wsp/                      this folder
```

## Integration notes

- `ignis-cli` registers `ignis_wsp::experts()` and the `wsp` subcommand. `ignis serve --enable-models`
  (or `IGNIS_ENABLE_MODELS`) serves the releases listed in `IGNIS_WSP_SERVED` (default
  `configs/models/served`) through `ignis_wsp_serve::ServedModels`. It fails closed: a release whose
  record does not hash to its id, or whose forward identity differs from this build's, leaves the
  models unavailable, with the reason.
- **Routes**: `GET /api/v1/model-cards`, `POST /api/v1/models/{id}/scenarios` (a typed scenario,
  admitted as a synthetic prefix), `POST /api/v1/analyses` (a forecast job), `GET /api/v1/jobs/{id}`,
  `GET /api/v1/artifacts/{id}`, `GET /api/v1/models[/{id}]`.
- **Model service.** `ignis_wsp::WspModelService` implements `ignis_core::services::ModelService`. It
  routes by `model_name`, does one verified load per release id, checks the partition of a stored run
  against the host scope before its prefix is built, and returns scientific refusals as artifacts.
- **Contract**: `Quantity::PhaseEndTime` (seconds) is the target of event models.
  `ModelCard`, `ModelInput`, `ReportedScore`, `ScenarioRequest` and `ScenarioAdmitted` are in
  [`ignis-core/src/contracts/wsp.rs`](../../crates/ignis-core/src/contracts/wsp.rs).
- **The planned stop** of a burner test is computed from NASA's script parameter file into a private
  copy of the data root. It is not in the shared records. Without it, the burner-flame releases answer
  on the shared records.
- **BASS front tracks** are files of the authoring lane (`wsp-bass-front-track/1`). The train tracks
  are in `tools/wsp/bass_optical/tracks/train/`. On stored runs `bass_front.wsp` refuses, because no
  stored BASS run carries a `width` in metres and the fan history; it answers typed scenarios.

## Release ids

A release is its weights and its forward, so its id follows the forward's identity
([WSP_GUIDE.md section 8](WSP_GUIDE.md#8-release-assembly-and-content-addressed-ids)). The ids quoted in
older feedback files, and their current values:

| Candidate | In feedback files | Current |
|---|---|---|
| FLEX W0 | `34e29dba…`, `9a830c7d…` | `fa8eb18c…` |
| FLEX W1 | `2b168d0a…` | `a3fbfef9…` |
| FLEX W2 | `fbd3d6d4…` | `2ce097c5…` |
| ACME normal W0 / W1 / W2 | `8b8d2feb…` / `ddc8cd5c…` / `255914be…` | `7ef0c7ab…` / `a75ba790…` / `fa1ab73e…` |
| ACME inverse W0 / W1 | `dbd50673…` / `26947885…` | `843eef56…` / `785b6b9d…` |
| BASS W0 | `a4f6c037…` | `9ed0451d…` |
