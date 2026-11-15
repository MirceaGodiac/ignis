# cld.wsp — CLD Flame loss and radiometer level

`cld.wsp` forecasts two things for the nitrogen-diluted methane and ethylene coflow flames of ACME CLD
Flame (PSI-21): when the flame is lost while the fuel still flows (an event with two causes, blow-off
and other, in seconds since flame onset) and how much the radiometers read over their pre-ignition
level in the first constant-flow stage. On the unread test days of the seventh generation, scored once
on a frozen list, it beats every baseline on every score: flame lost **0.073** (integrated Brier)
against 0.102 for the incidence, radiometer **0.115** (CRPS of log10 V) against 0.144 for the best
baseline (`SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md`, row "cld D1"). Data-informed: D1 is one
damped step, typed from the train days, on a W0 typed with train aggregates seen; it was chosen as the
served model after that run. Release
`sha256:a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f`.

| | |
|---|---|
| Model | `cld.wsp` |
| Investigation | ACME CLD Flame (PSI-21): methane and ethylene coflow flames diluted with nitrogen, 2.18 mm tube in a 25 mm air annulus |
| Family | `other` (generic law slug `cld`) |
| Candidate | D1 (`configs/models/law/cld/d1-fable`) |
| Generation | seventh (PROTOCOL 13.3, one revision from train days; served after the final run of 13.7) |
| Author | `claude-fable-5-2`; transcribed by the assembly engineer `claude-opus-5-5` |
| Authorship | data-informed (train days; W0 parent typed with train aggregates seen) |
| Release | `sha256:a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f` |
| Forward | `ignis_wsp::law::forecast`, `crates/ignis-wsp/src/law.rs` (profile `strict_no_fitting`, version 1) |
| Parent release | `sha256:0995df666bdabfa546607fcba565da166a28839ed984f2f1226bc62274eefe4e` (`cld` W0, `configs/models/law/cld/w0-fable`) |

## 1. Task and targets

Every input is a typed condition (`configs/models/served/served.json`, entry `cld.wsp`; domains from
the release tensors):

| Input | Kind | SI unit | Note |
|---|---|---|---|
| `x_fuel_final` | condition | 1 | lowest fuel fraction the script reaches; 0.04 to 1 |
| `v_coflow_final` | condition | m/s | highest coflow speed the script reaches; 0.02 to 1.5 |
| `window` | condition | s | planned fuel-off, seconds since flame onset; 5 to 600 |
| `is_ethylene` | condition | 1 | 1 when the fuel is ethylene, 0 for methane |
| `x_fuel_stage` | condition | 1 | fuel fraction of the first constant-flow segment of the schedule; 0.04 to 1 |
| `v_fuel_stage` | condition | m/s | fuel-stream speed of that segment; 0.02 to 1.5 |

| Target | Kind | Unit | Meaning |
|---|---|---|---|
| `loss` | event, causes `blowoff`, `other` | s since flame onset | the flame goes out while the fuel still flows; the planned fuel-off is the declared scheduled stop |
| `radiometer` | positive | V | excess of the radiometer sum over its pre-ignition level in the first constant-flow stage (lane-local, `cld:radiometer`) |

Time origin of `loss`: flame onset. The event artifact gives, at each requested time, the rows
`active`, `blowoff`, `other` and `scheduled_stop`, which sum to one; conditioned on no loss before the
cutoff. `stored_runs` is `false`: a forecast is a typed scenario with all six conditions; a value
outside the domain is refused with its name.

## 2. Physical mechanism

A dilute flame on this burner is held or lost by one comparison: the coflow speed against the laminar
burning speed of the diluted fuel in air. The flame base is a premixed kernel carried at about
0.66 of the coflow speed and leaving the rim at 1.4 times the burning speed
(`SpaceApps-Ardealu:docs/wsp/research/phys_cld_w0.md`, sections 0 and 1.1 to 1.4):

```math
\Phi = \frac{0.66\,U_{co}}{1.4\,S_L(x_{fuel})},\qquad \text{lifted above } 1
```

Dilution lowers `S_L` steeply through the adiabatic temperature (section 1.3):

```math
S_L(x) = S_{L0}\,\exp\!\left(-\frac{T_a}{2}\left(\frac{1}{T_{ad}(x)} - \frac{1}{T_{ad}(1)}\right)\right)
```

with `S_L0` 0.38 m/s (methane), 0.68 m/s (ethylene). The script ramps toward its most severe point;
with the ramp uniform in `ln Phi` over the window, the blow-off clock is log-logistic with median

```math
\ln\tau_b = \ln W + \ln\Phi_c - \ln\Phi_{final},\qquad P(\text{lost by } t) = 1 - \frac{1}{\left(1 + (t/\tau_b)^{3.4}\right)\left(1 + (t/\tau_o)^{0.5}\right)}
```

where W0 set the loss threshold `Phi_c` = 2.5 for both fuels and D1 lowers it to 1.7 for methane (a
methane jet has no stable lifted flame, since methane diffuses faster than momentum), keeping 2.5 for
ethylene (`SpaceApps-Ardealu:docs/wsp/research/phys_cld_d1.md`, step 1). The small clock `tau_o`
(about 70 000 s, shape 0.5) stands for every other loss: about 4 % of flames in 120 s, front-loaded.

The radiometer reads the radiant power of the flame (section 1.5):

```math
V = c_V\,\chi\,Q_{fuel},\qquad \chi = 0.05 + \chi_{soot}\,x^{2.5}\ (\chi_{soot}\ 0.15\ \text{methane},\ 0.35\ \text{ethylene})
```

with D1's volt scale `c_V` = 0.39 V/W (W0: 0.5).

- `loss.blowoff.median`: `b0` = ln of the methane median at the centre; `skip` carries the
  burning-speed slope on dilution (0.55 per e-fold), -1 on coflow, +1 on the window and the ethylene
  shift; hinge units 0 to 3 make `ln(S_L / 0.38)` piecewise (no gain above the pure fuel; faster falls
  below 50, 30 and 20 % methane, shifted by 0.47 in ln for ethylene). `loss.blowoff.shape` is the
  clock shape.
- `loss.other.median.b0`, `loss.other.shape`: the small clock. `loss.stop.offset_w/offset_s`: the stop
  is the planned fuel-off exactly.
- `radiometer.median`: `b0` = ln(`c_V` x gas radiation of pure methane at 0.2 m/s); `skip` 1 on the
  stage fuel fraction and fuel speed (heat release), 0.5 for ethylene (1.65 times the heat per volume);
  hinge units 0 to 2 the soot soft-plus `ln(1 + chi_soot x^2.5 / 0.05)`. `radiometer.spread` and tail:
  ln-sd 0.8, 10 % of cases three times wider.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section cld.wsp.

## 3. Authoring

- **Author**: `claude-fable-5-2`, 15 November 2026, in `SpaceApps-Ardealu:docs/wsp/research/phys_cld_d1.md`
  (sha256 `6144d2fb…c86e`), its Edits block, lines 67 to 69. "Transcribed by the assembly engineer
  (claude-opus-5-5); no number chosen" (`configs/models/law/cld/d1-fable/meta.json`); every other
  number is the parent's (`feedback/cld_d1_result.md`, assembly check).
- **Step**: PROTOCOL 13.3, one damped revision per investigation from its train days, scored once on
  the validation cases of the 13.2 look beside its parent (`SpaceApps-Ardealu:docs/wsp/PROTOCOL.md`,
  13.3).
- **What was read** (the release's exposure): the dossier; `feedback/cld_train_anatomy.md` (train days
  only); the coordinator's summary of the 13.2 blind look (aggregates only, no validation case).
  Dataset `cld-train-cases` (`transfer_cld/cases_train.jsonl`), partition **train**. Development
  feedback: the PROTOCOL 13.2 validation aggregates of the W0 parent. Prior candidate: W0. No
  validation outcome, no test day and no per-case file was opened (`phys_cld_d1.md`, header).
- **The parent's exposure.** W0 is "not a blind zero-shot" (`configs/models/law/cld/w0-fable/meta.json`):
  the author saw train aggregates in the engineer's note (20 of 207 flames lost and in which stage;
  9 of 11 dilution tests lost against 11 of 196; loss-time clusters; radiometer ranges) and set the
  loss threshold inside its theoretical range, the level of the small clock and `c_V` knowing them
  (`phys_cld_w0.md`, section 8.3). The mechanism and every flame number are from outside the
  experiment.
- **Brief**: [docs/research/PROMPTS.md](../../research/PROMPTS.md); the author worked from
  `phys_cld_d1.md`, on the law of `phys_cld_w0.md`.
- **Decisions** (three numbers, each moved about half of what the train strata ask for):
  1. `loss.blowoff.median.b0` 6.403 to **6.03**: the methane loss threshold `Phi` 2.5 to 1.7. Train
     methane at 25 to 40 % fuel was lost six times in ten where W0 said one in ten; ethylene was
     calibrated.
  2. `loss.blowoff.median.skip[3]` (ethylene) 0.84 to **1.21**: the same amount added back, so that
     every ethylene forecast is unchanged.
  3. `radiometer.median.b0` -0.404 to **-0.66**: `c_V` 0.5 to 0.39 V/W, half of the median error of
     +0.2 to +0.3 log10 V on both fuels.

  Not typed, named as structural: a feature saying that the script dilutes during the test, and a
  class of flames that fail at igniter retraction (`phys_cld_d1.md`, "Not typed").

## 4. Weights

Feature order and transforms: 0 `x_fuel_final` (ln, centre 1), 1 `v_coflow_final` (ln, centre
0.2 m/s), 2 `window` (ln, centre 60 s), 3 `is_ethylene` (identity), 4 `x_fuel_stage` (ln, centre 1),
5 `v_fuel_stage` (ln, centre 0.2 m/s). Values from the release params; D1 wrote the three entries in
bold, every other number is W0's (`configs/models/law/cld/w0-fable/edits.wsp`).

| Group | Shape (nonzero) | Value(s) | Meaning |
|---|---|---|---|
| `x.x_fuel_final.*`, `x.x_fuel_stage.*` | 2 x 4 scalars | centre 1, scale 1, lo 0.04, hi 1 | ln fuel fraction |
| `x.v_coflow_final.*`, `x.v_fuel_stage.*` | 2 x 4 | centre 0.2, scale 1, lo 0.02, hi 1.5 | ln of speed over 0.2 m/s |
| `x.window.*` | 4 | centre 60, scale 1, lo 5, hi 600 | ln of window over 60 s |
| `x.is_ethylene.*` | 4 | centre 0, scale 1, domain 0 to 1 | fuel indicator |
| `loss.blowoff.median.b0` | [] | **6.03** | methane median at the centre, threshold 1.7 (W0 6.403 = ln(60 s) + ln 2.5 - ln Phi, 604 s) |
| `loss.blowoff.median.skip` | [6] (4) | 0.55, -1.0, 1.0, **1.21**, 0, 0 | `S_L` slope on dilution; more coflow sooner; longer window later; ethylene shift |
| `loss.blowoff.median` unit 0 | (4) | `W1[0,0]` 1, `W1[0,3]` 0.47, `b1` 0, `w2` -0.55 | no gain above the pure fuel |
| `loss.blowoff.median` units 1 to 3 | (6 + 3 + 3) | `W1[j,0]` -1, `W1[j,3]` -0.47; `b1` -0.693, -1.204, -1.609; `w2` -0.5, -0.6, -1.0 | `S_L` falls faster below 50, 30, 20 % methane (31, 19, 12.5 % ethylene) |
| `loss.blowoff.shape` | [] | 3.4 | log-logistic shape, `1.7 / 0.5` |
| `loss.other.median.b0` | [] | 11.156 | small clock median about 70 000 s |
| `loss.other.median.skip/W1/b1/w2` | (0) | all zero | no dependence on the conditions |
| `loss.other.shape` | [] | 0.5 | hazard falling with time |
| `loss.stop.offset_s` | [3] (0) | 0, 0, 0 | stop = the planned fuel-off |
| `loss.stop.offset_w` | [3] (1) | 1, 0, 0 | one node |
| `radiometer.median.b0` | [] | **-0.66** | ln V of pure methane gas radiation at 0.2 m/s with `c_V` 0.39 V/W (W0 -0.404 = ln 0.668 V) |
| `radiometer.median.skip` | [6] (3) | 0, 0, 0, 0.5, 1.0, 1.0 | ethylene heat per volume; proportional to stage fuel fraction and fuel speed |
| `radiometer.median` units 0 to 2 | (6 + 3 + 3) | `W1[j,3]` 0.847, `W1[j,4]` 2.5; `b1` 3.299, 1.099, -1.101; `w2` 0.31, 0.38, 0.31 | soot term in three pieces; 1.099 = ln(0.15/0.05), ethylene 0.847 = ln(0.35/0.15) more |
| `radiometer.spread.b0` | [] | 0.8 | ln-sd (0.35 in log10) |
| `radiometer.spread.skip/W1/b1/w2` | (0) | all zero | |
| `radiometer.tail_weight`, `tail_width` | [] | 0.1, 3.0 | 10 % of cases three times wider |
| `num.substeps` | [] | 512 | constant: integration steps per requested interval of the event table |

Totals (release `architecture.parameter_counts`): 318 authored elements, 65 nonzero, 1 constant.
Every number back as edits: `ignis wsp inspect --release sha256:a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f --edits`;
the parent's full block is `configs/models/law/cld/w0-fable/edits.wsp`, D1's three lines are
`configs/models/law/cld/d1-fable/edits.wsp`.

## 5. Forward and assembly

| Field | Value |
|---|---|
| Forward | `crates/ignis-wsp/src/law.rs`, `ignis_wsp::law::forecast`, profile `strict_no_fitting` |
| forward_hash | `sha256:8d6e4c99aa7e971105865903b01461fd7a8e63e74b456a3859118f0f0d01c24a` |
| code_version | `ignis-wsp@0.1.0+fwd.8d6e4c99aa7e9711` |
| Spec | `law-cld-v1`, `sha256:bb23a7b334500ac5f2a4800b5232356704fe4b9ddea49780445f74f36674cc9a` (declaration `configs/models/law/cld/decl.json`) |
| Patch | `law-cld-v1-d1-fable`, `sha256:f34c9045d8f32989a1c0fb8d99ce88167678dc0fd3dee01ed8110bc6e58b7dd4` |
| weights_hash | `sha256:f325ddad866116efb395f8409a01e496570b2beb6037cf656ff5833cbbd5d433` |
| Binding | cohort `atlas:PSI-21:catalogue`, split `sha256:fd9aa7d0b03a3039022dd7364a23a915408db1512797eef36274ccdf70c690af` (`configs/models/law/cld/binding.json`) |
| Construction | `literal_revision`; uncertainty `assumed_noise`; checks spec_valid, patch_valid, tensors_complete, strict_forward_audit, finite_forward_on_fixture all passed |
| Candidate directory | `configs/models/law/cld/d1-fable` |

Rebuild (the chain is root `configs/models/law/cld/root` → `w0-fable` → `d1-fable`; the registry must
hold the W0 release, as `tools/wsp/reassemble.py` builds it):

```
cargo build --release -p ignis-cli
ignis wsp compile  --family cld --edits configs/models/law/cld/d1-fable/edits.wsp --meta configs/models/law/cld/d1-fable/meta.json --out configs/models/law/cld/d1-fable/patch.json
ignis wsp assemble --family cld --patch configs/models/law/cld/d1-fable/patch.json --binding configs/models/law/cld/binding.json --root <registry holding sha256:0995df66…fe4e> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the record; `served_ids` pins the 14
served ids. `cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at
ignis-web a58c7c3: the served id is the content hash of its record and loads through the registry
checks.

## 6. Evaluation

Lower is better. Loss: integrated Brier over the horizons, the Brier score of "lost before fuel-off",
and the integrated Brier as a mean of day means. Radiometer: CRPS of log10 V.

**Final run, test days** (PROTOCOL 13.7, run once on the frozen list sha256 `f3556071…c70e2`; 63 test
points answered on 7 days; D1, typed from train days, had never seen them; source
`SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md`, section PSI-21, where D1 is "comparator 2" and the
registered model was `cld3`):

| Release | Role in the run | loss: integrated Brier | loss: Brier, lost before fuel-off | loss: integrated Brier, day means | radiometer: CRPS log10 V | radiometer: day means |
|---|---|---|---|---|---|---|
| **cld D1 (served)** | comparator 2 | **0.0726** | **0.0443** | **0.0320** | **0.1145** | **0.1134** |
| cld3 (third law) | model | 0.1239 | 0.0175 | 0.0732 | 0.1145 | 0.1134 |
| cld W0 | comparator 1 | 0.0786 | 0.0545 | 0.0342 | 0.1411 | 0.1384 |
| cld C1 | comparator 3 | 0.1052 | 0.0173 | 0.0574 | 0.1145 | 0.1134 |
| cld2 (second law) | comparator 4 | 0.1094 | 0.0372 | 0.0476 | 0.1145 | 0.1134 |
| incidence (simple) | baseline | 0.1020 | 0.1022 | 0.0559 | 0.3145 | 0.3128 |
| by fuel | baseline | 0.1199 | 0.1017 | 0.0688 | 0.2547 | 0.2527 |
| loss by dilution ramp | baseline | 0.1166 | 0.0568 | 0.0565 | - | - |
| classical LFL at the stoichiometric surface | baseline | 0.1201 | 0.1111 | 0.0659 | - | - |
| Roper length plus fuel offset | baseline | - | - | - | 0.1439 | 0.1750 |

D1 is lower than every baseline on all five scores, and it has the lowest integrated Brier of every
release in the run. cld3, C1 and cld2 carry D1's radiometer network unchanged, hence the equal
radiometer columns. PROTOCOL 13.7: "the first revision D1 beats every baseline on all three scores"
of the loss.

**Validation days** (PROTOCOL 13.3, the one look of D1; validation outcomes had not been opened by the
author; 61 test points answered, 11 refused, 6 days; source `feedback/cld_d1_result.md`):

| Target | W0 | D1 | Simple baseline | Best baseline | Reading |
|---|---|---|---|---|---|
| loss, integrated Brier | 0.5925 | 0.5139 | 0.4782 (incidence) | 0.3282 (by dilution ramp) | D1 lower than W0; above the simple baseline |
| radiometer, CRPS log10 V | 0.1083 | **0.1003** | 0.2627 | 0.1743 (Roper) | beats the best baseline |

PROTOCOL 13.2 notes that the CLD validation set is harder than train for every forecaster (the
incidence baseline moves from 0.15 to 0.48).

**Train days** (study data; `cld_d1_result.md`, summary): loss 0.1569 (W0) to 0.1253 (D1); radiometer
0.1500 to 0.1234. **Development, in-sample** (train and validation pooled, after validation was opened
as study data; `feedback/cld3_result.md`, `PROTOCOL.md` 13.4): D1 loss 0.3174 against the best baseline
0.2282; radiometer 0.1186.

## 7. Adoption decision and lineage

- **Why it is served.** PROTOCOL 13.3: D1 replaced W0 because it was lower on the target that lost
  (0.5139 against 0.5925) and within 5 % on the radiometer (0.1003 against 0.1083). The later
  candidates were kept for the final run on development numbers (C1, then `cld3` as the model), and
  `cld3` was the registered model of the test run. There `cld3` forecast *whether* a flame is lost
  better than anything else (0.0175) but its time course did not hold (0.1239 against 0.1020 for the
  incidence), while D1 held on every score. PROTOCOL 13.7, "what was chosen after seeing it": "CLD
  Flame is served by D1 ... their test-day numbers are no longer an unbiased estimate for those two
  choices". The served list says the same in its limits.
- **Lineage.** neutral root `sha256:793e793e…6a06` (`configs/models/law/cld/root`, no claim) → W0
  `sha256:0995df66…fe4e` (`w0-fable`, W0 with stated exceptions, `phys_cld_w0.md` section 7.4 typed
  verbatim) → D1 `sha256:a03ec603…2d4f` (methane threshold, ethylene compensation, volt scale).
- **Tried and not adopted** (each a comparator in the final run):
  - C1 `sha256:541e9b2a…9ab6` (`configs/models/law/cld/c1-fable`, a revision of D1, PROTOCOL 13.6,
    `phys_cld_study.md`): the loss decided by a sharp limit on `R = U_co / (k S_L^2)`. In-sample 0.0431
    against D1's 0.3174, lower on 21 of 25 days, so kept for the final run; on the test days 0.1052.
  - `cld2` `sha256:870419c3…8721` (slug `cld2`, `phys_cld2.md`, PROTOCOL 13.4): dilution and ramp
    flags. In-sample 0.143 against 0.317, but lower on 5 of 8 days that change, short of two thirds:
    not kept; test days 0.1094.
  - `cld3` `sha256:b9df5239…9e4b` (slug `cld3`, the C1 chain plus a `t_severe` input: the loss time
    follows the ramp): kept as the model on development numbers (0.0334, 7 of 9 days); on the test
    days its time course did not hold (above).

**Scope** (served `limits`, condensed, and the result files):

- D1 was chosen after the test run, where the newer CLD law lost on timing: its test numbers are not
  blind to that choice.
- On the validation days D1's loss score (0.5139) is above the incidence (0.4782); on the pooled
  development days it is above the best baseline (0.3174 against 0.2282). The author names the missing
  input: a feature saying that the script dilutes until the flame is lost.
- A case whose inputs lack a declared feature is refused with its name: 11 of the 72 validation points
  (all for `window`; schedules without a constant-flow stage are refusals by PROTOCOL 13.2).
- Targets `loss` and `radiometer`; one per request through `settings.target` (default `loss`). The
  event target needs `settings.times_s` or `horizon_s`.

## 8. How to call it in IGNIS

- **Forecast lab** (web, route `#/lab`): the Models screen `#/models` lists the served releases and
  opens the model card; `web/src/views/LabModels.tsx` runs a typed scenario (no stored runs).
- **Model card**: `GET /api/v1/model-cards` (fields from `configs/models/served/served.json`).
- **API**: `POST /api/v1/models/sha256:a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f/scenarios`
  returns an `input_hash`; then `POST /api/v1/analyses` with
  `{schema_version:"wsp.v1", release_id, input:{kind:"prefix", input_hash}, settings:{target, times_s, horizon_s, quantile_levels}}`
  returns a job; poll `GET /api/v1/jobs/<job_id>` and read `GET /api/v1/artifacts/<artifact_id>` (an
  `AnalysisArtifact`, evidence class `model_output`).
- **MCP**: `models_inspect`, then `models_forecast` with the `input_hash` of a scenario a person
  admitted in the lab.

The served list has no `example`; these values are typed, not measurements: a methane flame whose
first constant-flow stage is pure fuel at 0.2 m/s, diluted to 15 % methane with the coflow raised to
0.30 m/s, fuel off 60 s after onset (the case of `phys_cld_w0.md` section 5, there forecast by W0). The
serving policy admits only the six declared features as conditions, so the planned fuel-off is given
as a `scheduled_stop` control at the same time as `window`; the forward takes the earliest such stage
as the stop.

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f",
  "cutoff_s": 0,
  "conditions": {
    "x_fuel_final": 0.15,
    "v_coflow_final": 0.30,
    "window": 60,
    "is_ethylene": 0,
    "x_fuel_stage": 1.0,
    "v_fuel_stage": 0.2
  },
  "controls": [{"control": "scheduled_stop", "t_start_s": 60}],
  "observations": []
}
```

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f",
  "input": {"kind": "prefix", "input_hash": "<from the scenario>"},
  "settings": {"target": "loss", "times_s": [5, 10, 20, 30, 45, 60, 90]}
}
```

`settings.target` is `loss` (event: `times_s`, or `horizon_s` with `n_points`, required) or
`radiometer` (quantiles in V). Stored runs: no. Give the most severe fuel fraction and coflow the
script reaches in `x_fuel_final` and `v_coflow_final`, even when they are not the final ones
(`phys_cld_w0.md`, section 7.1).

Cite: `cld.wsp D1, hand-written (literal WSP) release sha256:a03ec603…2d4f, experimental, not trained; its output is model output, not a measurement.`

## Sources

- `configs/models/served/served.json` (entry `cld.wsp`);
  `configs/models/served/models/a03ec603f0926cf522df5d5a9f09d73f077ba22e6cabd5e8b29970794ddf2d4f/release.json` and its params (data pack)
- `configs/models/law/cld/d1-fable/{edits.wsp,meta.json,release.json}`, `configs/models/law/cld/w0-fable/{edits.wsp,meta.json,release.json}`,
  `configs/models/law/cld/c1-fable/{edits.wsp,meta.json,release.json}`, `configs/models/law/cld/root/release.json`,
  `configs/models/law/cld/binding.json`, `configs/models/law/cld/decl.json`
- `crates/ignis-wsp/src/law.rs`; `crates/ignis-wsp-serve/src/lib.rs`; `crates/ignis-core/src/contracts/wsp.rs`;
  `crates/ignis-core/src/prefix.rs`; `crates/ignis-core/src/contracts/research.rs`; `tools/wsp/verify_served.py`
- `SpaceApps-Ardealu:docs/wsp/research/law_forward.md`, `law_forward_targets.md` (feat/handwritten-science, 542dad8)
- `SpaceApps-Ardealu:docs/wsp/research/phys_cld_w0.md`, `phys_cld_d1.md`, `phys_cld_study.md`, `phys_cld2.md`
- `SpaceApps-Ardealu:docs/wsp/feedback/cld_d1_result.md`, `cld3_result.md`, `cld2_second_law_result.md`
- `SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md`; `SpaceApps-Ardealu:docs/wsp/PROTOCOL.md`, sections 13.2 to 13.7;
  `SpaceApps-Ardealu:docs/wsp/COVERAGE_19.md`
