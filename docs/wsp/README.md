# Literal weight-space programming in IGNIS

Scientific models whose every parameter is a number an author model typed, assembled without any
estimation and run by a forward that cannot fit anything. Branch `feat/handwritten-science`.

| Read | For |
|---|---|
| [EXPERIMENTAL_SCIENCE.md](EXPERIMENTAL_SCIENCE.md) | The three NASA experiments, what their train data show, the design decisions that follow, what each generation added and what validation said of it |
| [RESULTS.md](RESULTS.md) | Development scores of every candidate against the baselines, generations one to four |
| [RESULTS_GEN5.md](RESULTS_GEN5.md) | The one validation look of the fifth generation: outcome by the rules written beforehand, and the tables |
| [RESULTS_GEN6.md](RESULTS_GEN6.md) | The sixth generation (15 November): the look on CFI-G validation and Flame Design that adopted `acme_normal.wsp7` P3 |
| [overview.html](overview.html) | Generations one to four in pictures: forecasts on train runs, state probabilities, validation scores, the refused candidates |
| [cards/](cards/) | One model card per model: computation, what each number says and where it comes from, provenance, limits |
| [PROTOCOL.md](PROTOCOL.md) | Spec, edit language, assembler, release record, zero-shot and data-informed candidates, selection rules, commands |
| [RESEARCH.md](RESEARCH.md) | What MIT and Legora showed, the four kinds of "LLM-made weights" |
| [feedback/](feedback/) | The development feedback each revision was written from: train audits and train anatomies |
| [research/](research/) | Sourced dossiers: weight-space programming, the FLEX, ACME and BASS experiments, three reviews of the published combustion science, and the physics dossiers of the fifth generation |
| [dossier/](dossier/README.md) | **For a second author or reviewer (Astra):** what was read and how each document and data file was classified, every measurement with how far it was checked, the decision log, the claims with the cheapest way to break each, every known exposure, the work in progress, and how to file a finding or write a candidate of your own |

## Six generations

**Phase 1** wrote one model per family from theory alone (W0), then revised it against development
feedback (W1, W2). **Phase 2** studied the experiments and the train data first and rebuilt the
models around what the experiments do: a droplet is in a hot flame, a cool flame or no flame; a
burner flame ends on physical clocks. **The third generation** (14 November) audited those models
against their train partitions, rechecked the findings, and revised the laws and the way a prefix
is read. **The fourth generation** (14 November) read the published science of each experiment,
wrote the burner flames as the physics of a growing spherical flame with a heating law for the
burner, measured the BASS burns from the public videos and wrote the first BASS model that met real
data, and added the model service. Its droplet candidates were refused by validation. **The fifth
generation** (14 November) rebuilt the laws from the physical mechanism of each experiment: five
physics dossiers evaluated against train at literature constants, four independent re-derivations,
a look-up of every quoted property value, two new forwards and a committee of member laws for the
burner flames, which can also read the planned stop of a test from NASA's script file. One
validation look per family, on candidates and rules committed beforehand: the two burner-flame
candidates replaced the carried models; the three droplet candidates and the fabric candidate did
not ([RESULTS_GEN5.md](RESULTS_GEN5.md)).

**The sixth generation** (the night of 15 November) changed how a model is tested and revised, and
took the models to data they had never met ([PROTOCOL.md](PROTOCOL.md) section 12,
[RESULTS_GEN6.md](RESULTS_GEN6.md), the ledger [dossier/11_overnight_20261115.md](dossier/11_overnight_20261115.md)).
New data is scored blind first, once, and only then studied; a model is revised in one step per batch
from the errors of its train groups (the loss is decomposed, a script reports which numbers can move
which errors, the author types a damped step with a physical reason), and the validation groups judge
that step once. What it produced: the normal burner flames were rebuilt on two new forwards and
adopted on CFI-G validation and Flame Design together; the burner-flame laws now read the fuel from a
typed table and answer for ethylene and methane (zero-shot, then one revision after a blind run on
Flame Design); the planned stop of a test is computed from NASA's script file alone and lands within
a second of the recorded stop; six zero-shot droplet candidates for the fuels of CFI and FLEX-2 on a
new droplet forward; a zero-shot law for whether a fabric sample ignites and stays lit, tested and
not adopted. The text, tables and documents of all 19 investigations (41,004 files, 22 GB) are on
disk with hashes, and three measurers turn NASA's frames and video into series.

Every candidate, release and feedback file of every generation is preserved and its release id is
unchanged: a forward is frozen with its releases, and a better forward is a new file.

| Family | Carried model | Candidate | How written | Validation (development) |
|---|---|---|---|---|
| FLEX methanol | `flex_methanol.wsp2` | D3 `sha256:55e0cb12…43ce` | data-informed (train); third generation | **40.0 µm**; best fitted model 77.7; zero-shot `flex.wsp` W0 50.4. Not replaced by the fourth generation (44.8 and 40.1 against 39.5 on their common cases) nor by the fifth (42.5 against 39.5) |
| FLEX n-heptane | `flex_heptane.wsp2` | D2 `sha256:43f7871f…588d` | data-informed (train); third generation | **46.1 µm**; zero-shot W0 60.3; D²-law 105. Not replaced by the fourth generation (57.6) nor by the fifth (47.2 and 48.4) |
| ACME normal | `acme_normal.wsp12` | F2 `sha256:55fcf4cb…971d` | data-informed (CFI-G train; train days of Flame Design); sixth generation (15 November): `acme_normal.wsp7` P3 (burner limit by the date of the test, three statements corrected, the hinge member answering through the burner-heating law) reading the fuel from a typed table, so that it answers for ethylene and methane; the ethylene row revised once after a blind run; the planned stop computed from the script file | on ethane, propane and butane without the planned stop it is P3: extinction Brier **0.154**, flow termination 0.096 on CFI-G validation (`acme_normal.wsp5` P1: 0.155 / 0.097). With the planned stop 0.147 / 0.080 (P3 on the old planned stop: 0.144 / 0.075). On the Flame Design validation days, ethylene as itself: 0.032 / 0.393 without the planned stop and 0.007 / 0.072 with it ([RESULTS_GEN6.md](RESULTS_GEN6.md)) |
| ACME inverse | `acme_inverse.wsp12` | F2 `sha256:dff7da0d…594e` | data-informed (train; published theory); `acme_inverse.wsp5` P1 (the carried law with weight 2 and a scenario that holds a marginal flame, weight 1) reading the fuel from a typed table, zero-shot for ethylene and confirmed by a blind run; the planned stop computed from the script file | on ethane, propane and butane it is P1: **0.129** / 0.088 on the shared records; 0.135 / 0.048 with the planned stop. On Flame Design, ethylene as itself against ethylene borrowed as ethane: extinction 0.108 against 0.147 over 7 sessions, interval excludes zero; on its validation days with the planned stop 0.092 / 0.002 |
| BASS | `bass_front.wsp` | D1 `sha256:ac15e49f…855c` | data-informed (four train burns, published theory); the validation burns were measured blind | CRPS of the flame tip **0.117 sample widths**; persistence 0.234; paired difference +0.117 [+0.051, +0.182] over 12 burns. Its quench boundary fails (0.170 against 0.125 for a constant). The fifth-generation candidate (narrow sample corrected: 0.114, track-ended score worse) was not adopted |

The planned stop of a burner test is read from NASA's script parameter file into a private copy of
the data root; it is not in the shared records or contracts, and whether a forecast may know it is
for the data and evaluation lanes to decide. Without it the two burner-flame releases answer as in
the "shared records" numbers.

Zero-shot references that stay in the tables: `flex_heptane.wsp` W0 `sha256:6ea680d1…6d8f` (frozen
before any heptane outcome was opened, 60.3 µm), the phase-1 W0 of every family, and the published
theory of spherical flames with no number from this campaign (0.206 on normal-flame extinction,
where a constant scores 0.214).

Cards: [droplets, third generation](cards/flex_droplet3.md) (carried),
[droplets, fourth generation](cards/flex_droplet4.md) (not adopted),
[burner-flame clocks](cards/acme_clock.md), [growth clock](cards/acme_growth.md),
[two laws together](cards/acme_blend.md), [committee of laws and the planned stop](cards/acme_committee.md)
(carried), [droplets, fifth generation](cards/flex_droplet6.md) (not adopted),
[BASS flame front](cards/bass_front.md).

Full ids: the cards and `configs/models/**/release.json`. Releases live in the local, git-ignored
registry (`artifacts/models/`); `python tools/wsp/reassemble.py` rebuilds all of them from the
committed edits with identical ids.

**Which numbers are zero-shot.** Only W0 candidates and the reference baselines. Every candidate
named D1 to D5, P0 or P1, and every phase-1 W1 and W2, was typed after its author read development data.
Analysis of the train partition and the measurement of the BASS videos were delegated to other
model instances; no script and no delegate wrote a weight. Each candidate was committed before its
validation score existed, and that score decided only whether it replaced the model before it. The
validation partitions have by now been read as aggregate tables several times, so their numbers are
development numbers. The test partition has not been read by anything on this branch.

**What five generations taught.** A mechanism written in quantities that mean the same thing in
another session carried to unseen data (the three droplet states, the reduced flow, the burner
heating law, the band-shaped flame). Anything made sharper or narrower because the train runs looked
that way did not, and neither did a boundary typed as a line from one burn on each side of it.
The fifth generation added three things. Knowing the plan of a test is worth more than refining the
law of the flame: the script's planned stop halved the flow-termination error of inverse flames. A
second scenario carried beside the first with a stated probability transferred (inverse flames,
extinction 0.161 to 0.129). And laws rebuilt from mechanism with constants from property tables
matched the laws typed from train on train and did not beat them on validation: with nine validation
days the difference between the two is inside the noise, so what such a rebuild buys today is fewer
numbers read from train, not a better score.

**What the sixth generation added.** A zero-shot statement that was physics carried to a new fuel
(lean inverse ethylene flames last about twice the clock: 1.7 observed); one that was an analogy did
not (ethylene is "half-way to butane because both soot": wrong in direction in eight of eight
sessions), and a blind run found it at once. The revision that repaired it was a change of the
variable a law reads (ethylene burns as ethane does at about 0.58 of its mass flow), not a new
constant on the old variable. Knowing the plan of a test again beat refining the law: the planned
stop from the script file moved the flow-termination score of inverse flames on unseen days from
0.024 to 0.002. And a measurement is a model too: the first scoring of FLEX-2 was void because a
pixel scale had been assumed, and the fabric burns that were expected in a second investigation were
not there. Both were found by the checks and are recorded as they happened.

## Layout

```text
crates/ignis-wsp/            edit language, assembler, strict forwards, model service; depends on ignis-core only
  flex.rs acme.rs bass.rs      first-generation forwards (identity unchanged)
  droplet.rs                   hot flame / cool flame / no flame (flex_methanol.wsp, flex_heptane.wsp)
  droplet3.rs                  the same physics, third generation (flex_*.wsp2): the carried droplet models
  droplet4.rs                  tracker flags, helium law (flex_*.wsp3): not adopted
  flame.rs                     hinge-law clocks of the burner flames (acme_*.wsp2)
  flame3.rs                    growth clock and burner-heating law (acme_*.wsp3)
  blend.rs                     two member laws and two weights (acme_*.wsp4)
  front.rs, front_tools.rs     flame front of a thin fuel in concurrent flow (bass_front.wsp); track reader
  service.rs                   WspModelService: the ModelService of the literal models
  models.rs                    literal models by key; a release finds its forward by model_name
crates/ignis-wsp-dev/        wsp-dev: feedback, tables, bench, heptane-cohort, front-feedback, front-forecast (train/validation only)
configs/models/flex/         wsp/ (phase 1, and methanol/ heptane/ of phase 2), wsp2/ (third), wsp3/ (fourth)
configs/models/acme/         wsp/ (phase 1), wsp2/ (clocks), wsp3/ (growth), wsp4/ (blend)
configs/models/bass/         wsp/ (phase 1), front/ (flame front)
  <candidate>/                 edits.wsp, meta.json, patch.json, release.json
tools/wsp/                   reassemble.py, dev_eval.py, overview.py, blend_edits.py, analysis/ (train-data measurement),
                             bass_optical/ (measurement method, train tracks)
docs/wsp/                    this folder
```

Model keys for `ignis wsp … --family <key>`: `flex`, `acme_normal`, `acme_inverse`, `bass`,
`flex_methanol`, `flex_heptane`, `acme_normal2`, `acme_inverse2`, `flex_methanol2`, `flex_heptane2`,
`acme_normal3`, `acme_inverse3`, `flex_methanol3`, `flex_heptane3`, `bass_front`, `acme_normal4`,
`acme_inverse4`.

## Integration notes

- `ignis-cli` registers `ignis_wsp::experts()` and the `wsp` subcommand.
- **Model service.** `ignis_wsp::WspModelService` implements `ignis_core::services::ModelService`
  for every literal release: routing by `model_name`, one verified load per release id, the
  partition of a stored run checked against the host scope before its prefix is built, scientific
  refusals returned as artifacts. Construction:

  ```rust
  let models: Arc<dyn ModelService> = Arc::new(ignis_wsp::WspModelService::new(
      ignis_core::registry::default_root(),
      ignis_wsp::ViewSourcePrefixes::new(ignis_data::views::DataStore::new()),
  ));
  ```

  A forecast by prefix hash is refused (`capability_unavailable`): the core crate has no store of
  admitted prefixes by hash, and none was added here.
- Contract (phase 1): `Quantity::PhaseEndTime` (seconds) as the target of event models; `schemas/`
  and `web/src/contracts/generated.ts` were regenerated then. No later generation changed a contract.
- New frozen split (phase 2): `data/splits/flex-c43bbe3580ffce2a.json`, the n-heptane cohort, built
  with the existing `Cohort` and `Split` types and the shared split rule by `wsp-dev heptane-cohort`.
- `ignis-flex`: `--set fit.skip_invalid_condition_runs=true` lets the fitted baselines skip a train
  run with a stored pressure of 0 Pa.
- BASS front tracks are files of this lane (`wsp-bass-front-track/1`), not shared observations: the
  train tracks are in `tools/wsp/bass_optical/tracks/train/`, the validation tracks are local files
  outside git. `bass_front.wsp` also runs on a shared prefix that carries `x_o2`, a `width` in
  metres and the fan history as control stages; no stored BASS run has them, so on stored runs it
  refuses.
- The fourth-generation droplet forward reads one more prospective condition (`initial_diameter`),
  so its prefix hash differs from the older droplet releases; the carry releases hold the
  third-generation numbers on it for paired comparisons.

## Release ids

A release is its weights and its forward, so its id follows the forward's identity. Ids quoted in
older feedback files and their current values:

| Candidate | In feedback files | Current |
|---|---|---|
| FLEX W0 | `34e29dba…`, `9a830c7d…` | `fa8eb18c…` |
| FLEX W1 | `2b168d0a…` | `a3fbfef9…` |
| FLEX W2 | `fbd3d6d4…` | `2ce097c5…` |
| ACME normal W0 / W1 / W2 | `8b8d2feb…` / `ddc8cd5c…` / `255914be…` | `7ef0c7ab…` / `a75ba790…` / `fa1ab73e…` |
| ACME inverse W0 / W1 | `dbd50673…` / `26947885…` | `843eef56…` / `785b6b9d…` |
| BASS W0 | `a4f6c037…` | `9ed0451d…` |
| Heptane W0 | `ceb57c47…`, `80b50326…`, `5a8395c0…`, `c08e513d…` (forward corrections on 13 November; edits unchanged, sha256 `3b49e69c…`) | `6ea680d1…` |
