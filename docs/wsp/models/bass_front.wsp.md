# bass_front.wsp — the flame tip on a thin fabric in concurrent flow (BASS-II)

`bass_front.wsp` forecasts where the tip of the visible flame will be on a thin cotton-fibreglass
fabric strip (SIBAL) burning with a slow duct flow aboard the ISS (BASS-II, PSI-25), given the first
seconds of the burn and the fan history as a known control. On the twelve validation burns, measured
from the public videos by agents that knew nothing of the model, its flame-tip CRPS is **0.117
sample widths against 0.234 for persistence** (paired difference +0.117 [+0.051, +0.182] over 12
burns, lower on 10 of 12; SpaceApps-Ardealu:docs/wsp/RESULTS.md, card `docs/wsp/cards/bass_front.md`).
Carried unchanged to the Saffire sheets, twenty times wider, it scores 0.058 against 0.250
(SpaceApps-Ardealu:docs/wsp/COVERAGE_19.md). It is data-informed: `claude-fable-5-2` typed every
number after reading four train burns and the published theory. Release
`sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c`.

| | |
|---|---|
| Model | `bass_front.wsp` |
| Investigation | BASS (PSI-25), thin fabric in concurrent flow |
| Family | `bass` (model key `bass_front`) |
| Candidate | D1 (`configs/models/bass/front/d1-fable`) |
| Generation | fourth (14 November 2026), PROTOCOL section 9 |
| Author | `claude-fable-5-2` (Anthropic); typed by the author, no transcriber |
| Authorship | data-informed (four train burns and published theory; the validation burns were measured blind) |
| Release | `sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c` |
| Forward | `ignis_wsp::front::forecast`, `crates/ignis-wsp/src/front.rs` (track reader and metadata in `crates/ignis-wsp/src/front_tools.rs`) |
| Parent release | `sha256:ce53fe1fb3e6300c5e818dd7c9c5ec8e8f4c2cfd19673449d3936de05527de46` (persistence, `configs/models/bass/front/baseline-persistence`) |

## 1. Task and targets

Inputs (`configs/models/served/served.json`; examples are typed values, not measurements):

| Name | Kind | SI unit | Required | Note | Served example |
|---|---|---|---|---|---|
| `x_o2` | condition | 1 | yes | oxygen mole fraction | 0.19 |
| `width` | condition | m | yes | sample width; converts the law from sample widths to metres | 0.022 |
| `length` | condition | m | no | sample length; when known, the tip is cut at the end of the sample | 0.1 |
| `flow_velocity` | control | m/s | yes | fan (flow) history as stages: start s, end s (empty = open), value | `[0, 22, 0.1]`, `[22, open, 0.03]` |
| `igniter` | control | 1 | no in the list, but the forward refuses without it | igniter stage: start s, end s; its end is the model's clock zero | `[4, 10, null]` |
| `phase_established_at_s` | phase start | s | yes | time the flame was established over the sample (end of the igniter stage) | 10.0 |
| `front_position` | observation | m | yes | flame-tip positions up to the cutoff | 11 s 0.002, 13 s 0.006, 15 s 0.010, 17 s 0.014, 19 s 0.018, 20 s 0.020 |

Target: the **position of the downstream tip of the visible flame** along the strip, measured from
the upstream end of the sample (`front_position`, metres on a prefix with a width; sample widths in
the development tracks). It is given as quantiles at each forecast time after the cutoff, together
with an event table of four categories per time: `active_supported` (a flame on the sample),
`extinction` (quenched), `reached_end` (the tip has passed the end of the sample and the flame has
lingered `end.linger_s`), `unresolved_support` (`crates/ignis-wsp/src/front.rs`, `body`).

Time origin: the prefix's own; the forward shifts it so that the igniter is off at zero
(`applicability.time_origin`). Forecasts are conditional on the flow history up to each forecast
time: the operator's fan steps are an input, not something the model predicts.

What a forecast needs: `x_o2`, a `width` in metres, an igniter stage with an end time, at least one
`flow_velocity` stage, and tip positions before the cutoff. No stored BASS run carries all of these
(`stored_runs = false`), so the release is forecast on a **typed scenario** only.

## 2. Physical mechanism

A flame on a thin fuel in concurrent flow is a band that travels with the flow
(`crates/ignis-wsp/src/front.rs`, module header; SpaceApps-Ardealu:docs/wsp/dossier/04_decisions.md,
D-32). Three places describe it, each answering the flow on its own time scale: the pyrolysis front
`x`, where the fuel starts to burn; the tip, ahead of it by an overhang `h`; and the base, where the
fuel has burned out. Lengths are in sample widths (the sample is its own ruler in the top view),
flows in cm/s.

Features of every law network (`speed`, `over`, `burn`, `quench`):

```math
u = \frac{U}{5\ \mathrm{cm/s}} - 1,\qquad c_{O_2} = \frac{X_{O_2}-0.19}{0.02},\qquad c_w = \ln\frac{W}{W_\mathrm{ref}}
```

```math
N(c) = b_0 + \mathrm{skip}\cdot c + \sum_j w_2[j]\,\mathrm{relu}\big(b_1[j] + W_1[j,:]\cdot c\big)
```

**Spread rate** (`speed.*`). Theory and every numerical model give a steady concurrent spread
linear in flow and close to linear in oxygen (SpaceApps-Ardealu:docs/wsp/research/lit_flame_spread.md,
"Laws a hand-written front model should contain"). The front advances as $`dx/dt = E\,v`$ with

```math
v = \max(N_\mathrm{speed}, v_\mathrm{min})\left(\frac{X_{O_2}}{0.19}\right)^{1.5}\left(\frac{W}{W_\mathrm{ref}}\right)^{-1},\qquad N_\mathrm{speed} = 0.0535 + 0.032\,u = 0.0215 + 0.0064\,U
```

in sample widths per second, and follows a change of flow at once.

**Overhang** (`over.*`). The visible tip is ahead of the front by $h$, which relaxes to its steady
value within `over.tau_s` = 2 s:

```math
h_{ss} = \max(N_\mathrm{over}, 0)\left(\frac{X_{O_2}}{0.19}\right)^{3}\left(\frac{W}{W_\mathrm{ref}}\right)^{-1},\quad N_\mathrm{over} = 0.325 + 0.325\,u = 0.065\,U,\qquad h \leftarrow h_{ss} + (h - h_{ss})\,e^{-\Delta t/\tau}
```

```math
\mathrm{tip}(t) = x(t) + h(t)
```

A step down in flow pulls the tip back while the front goes on, as the three tracked train burns
show (SpaceApps-Ardealu:docs/wsp/feedback/bass_train_measurement.md).

**Base** (`burn.*`). The base repeats the front one burn-through time later, so after a step down
it keeps the old speed for that long and the flame shortens from both ends:

```math
\mathrm{base}(t) = x(t - \tau_b),\qquad \tau_b = e^{N_\mathrm{burn}} = e^{1.84} = 6.3\ \mathrm{s}
```

**Run offset from the prefix** (`enc.*`). The run's own speed is read as the advance seen over the
advance the law gives under the same flow history, from the tip (corrected for the modelled
overhang) and from the base, mixed 0.8 and 0.2. It is trusted by a gain:

```math
E = \exp\big(\mathrm{clamp}(g\,\hat e,\ -0.7,\ 0.7)\big),\qquad g = \sigma\Big(-2.3 + 2.5\ln\frac{\mathrm{span}}{10\ \mathrm{s}} + 3.5\,\mathrm{settled}\Big)
```

where `settled` grows from 0 to 1 over the 10 s after the last change of flow.

**Quench hazard** (`quench.*`). A published boundary for a thin fuel (tissue paper), moved up by 3 %
oxygen to where the one train burn that never spread puts it:

```math
\lambda = \exp\Big(-7 + 2.5\,\mathrm{relu}\big(U_q - U\big) + 12\,\mathrm{relu}\big(-0.9 - c_{O_2}\big)\Big),\qquad U_q = 2.27 - 1.33\,c_{O_2}\ \mathrm{cm/s}
```

```math
P(\text{quenched by } t) = 1 - \exp\Big(-\int_{t_c}^{t}\lambda\,dt\Big)
```

The first relu term is the deficit below the quench velocity in cm/s; the second switches on below
17.2 % oxygen at any flow.

**Spread of the forecast** (`unc.*`, assumed noise). The relative sd of $\ln E$ is
$`\sqrt{g\,s_\mathrm{floor}^2 + (1-g)\,s_\mathrm{pop}^2 + (s_{far,0}\,c_{O_2})^2 + (s_{far,1}\,c_w)^2}`$;
the sd of the tip at a lead is
$`\sqrt{\sigma_w^2 + (s\,E\,\mathrm{advance})^2 + (r_\mathrm{over}\,\Delta h)^2 + d_w^2\,\mathrm{lead}}`$,
with Gaussian quantiles cut at the end of the sample when its length is known (`front.rs`, spec notes).

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section bass_front.wsp.

## 3. Authoring

**Who.** `claude-fable-5-2` typed every number in
`configs/models/bass/front/d1-fable/edits.wsp`, with the reason beside each statement. No number was
fitted or searched; no transcriber was involved (the edits header and `meta.json` say "Every number
typed by the author").

**When.** Fourth generation, 14 November 2026: PROTOCOL section 9 ("published science, three new
forwards, a blind BASS validation"), the first BASS model on real burns
(SpaceApps-Ardealu:docs/wsp/PROTOCOL.md, section 9). Candidate frozen in git before any validation
score of it existed.

**What was read** (`meta.json` exposure, recorded in the release):

- train partition only for outcomes: the front tracks of the four train burns T5, T6, T16 and T10
  (tip, base, fan history, fan calibration; `tools/wsp/bass_optical/tracks/train/`), measured from the
  public PSI-25 videos and proof sheets by delegated analysis agents
  (SpaceApps-Ardealu:docs/wsp/feedback/bass_train_measurement.md, `bass_train_tracks.md`); the author
  rechecked the claims it acted on;
- published theory, numerical models and other experiments of concurrent flame spread
  (SpaceApps-Ardealu:docs/wsp/research/lit_flame_spread.md), which by its own rule left out the
  measured results of the BASS-II concurrent SIBAL tests;
- the inputs only of the seventeen validation track files (oxygen, sample label, band width,
  restrictor, igniter interval, calibration rows, set of fan values), to check the reader accepts
  them; no position, time or outcome;
- pretraining knowledge (unverifiable).

The validation burns were measured by two measurement agents that knew nothing of the model, into
files kept outside git that the author has not opened (`runs/dev/bass_front/val_sealed/`;
SpaceApps-Ardealu:docs/wsp/dossier/04_decisions.md, D-31).

**Brief.** The prompt is in [docs/research/PROMPTS.md](../../research/PROMPTS.md); the author worked
from the literature dossier `lit_flame_spread.md`, the experiment dossier
SpaceApps-Ardealu:docs/wsp/research/exp_bass.md and the train-track tables.

**Decisions** (edits comments; `meta.json` rationale; dossier 06_models.md, section 6.6):

- The flame is a band, not a moving point: three tracked train burns show the tip stalling or
  retreating at a fan step while the base keeps the old speed (D-32).
- Spread rate from the steady phases after the fan steps (0.0374 and 0.0389 w/s at 2.4 cm/s in
  18.8 % O2; 0.0379 at 3.0 cm/s in 17.6 %) and before them (0.043 to 0.044 at 4.7 cm/s; 0.07 to 0.10 at
  10.9 cm/s). A first version ran 1 to 8 % fast on whole 35 s prefixes, so the law was typed 4 % lower:
  `0.0215 + 0.0064 U`.
- Oxygen power 1.5, inside the 1 to 2 the models give; width power -1 (same speed in mm/s), a judgment
  where the published work disagrees on the sign.
- Overhang `0.065 U` from three fan steps (an earlier `0.055 U` left the tip of all three burns behind
  the law across its step); oxygen power 3 from one burn; relaxation 2 s.
- Burn-through time 6.3 s from 6.6, 6.6 and 6.0 s, one number for all conditions.
- Quench: the tissue-paper boundary [L] moved up 3 % oxygen [T, two burns], plus a unit below 17.2 %
  oxygen from T10 (16.4 %, never spread).
- Encoder gains from offsets at 4, 14 and 35 s on three burns; spread 25 % before a prefix, 6 % after.

## 4. Weights

Every tensor of `params.npz` (`configs/models/served/models/ac15e49f…855c/params.npz`); values from
the release, meaning from `edits.wsp`. Totals (`architecture.parameter_counts`): **159 authored
elements, 35 non-zero, 11 constants.**

| Group | Shape / count | Value(s) | Physical meaning |
|---|---|---|---|
| `speed.b0`, `speed.skip` | 1 + 3 | 0.0535; [0.032, 0, 0] | spread rate `0.0215 + 0.0064 U` w/s at 19 % O2, 2.2 cm sample |
| `speed.pow` | 2 | [1.5, -1.0] | powers of `X_O2/0.19` and `W/W_ref` |
| `speed.W1`, `speed.b1`, `speed.w2` | 18 + 6 + 6 | all zero | no hinge in the spread law |
| `over.b0`, `over.skip` | 1 + 3 | 0.325; [0.325, 0, 0] | overhang `h_ss = 0.065 U` sample widths |
| `over.pow` | 2 | [3.0, -1.0] | oxygen power 3 (0.8 of the slope at 17.6 %), width power -1 |
| `over.tau_s` | 1 | 2.0 s | relaxation of the overhang after a change of flow |
| `over.W1`, `over.b1`, `over.w2` | 18 + 6 + 6 | all zero | no hinge |
| `burn.b0` | 1 | 1.84 | `ln` of the burn-through time, 6.3 s |
| `burn.skip`, `burn.W1`, `burn.b1`, `burn.w2` | 3 + 18 + 6 + 6 | all zero | one number for all conditions |
| `quench.b0` | 1 | -7.0 | background hazard, one quench in about 1100 s well above the line |
| `quench.W1[0,:]`, `quench.b1[0]`, `quench.w2[0]` | 3 + 1 + 1 | [-5.0, -1.33, 0]; -2.73; 2.5 | below `U_q = 2.27 - 1.33 c_o2` cm/s, +2.5 in ln rate per cm/s of deficit |
| `quench.W1[1,:]`, `quench.b1[1]`, `quench.w2[1]` | 3 + 1 + 1 | [0, -1.0, 0]; -0.9; 12.0 | below 17.2 % O2 at any flow, +12 per 2 % of oxygen |
| `quench` other elements (`skip`, units 2 to 5) | 3 + 12 + 4 + 4 | all zero | |
| `ign.gain`, `ign.delay_s` | 1 + 1 | 1.0; 0.0 s | the front moves at the law's speed from the igniter while it is on |
| `end.linger_s` | 1 | 5.0 s | the flame is still seen 5 s after the tip reaches the end (T5) |
| `enc.gain_w`, `enc.gain_b` | 2 + 1 | [2.5, 3.5]; -2.3 | trust in the prefix by ln(span/10 s) and settled share |
| `enc.mix` | 2 | [0.8, 0.2] | weights of the tip-based and base-based offsets |
| `enc.e_clip` | 2 | [0.7, 0.7] | the run offset lies in `[e^-0.7, e^0.7]` |
| `unc.s_pop`, `unc.s_floor` | 1 + 1 | 0.25; 0.06 | relative sd of the rate before a prefix, and after a trusted one |
| `unc.s_far` | 2 | [0.06, 0.25] | added per 2 % of oxygen and per e-fold of width |
| `unc.sigma_w`, `unc.sigma_x0_w` | 1 + 1 | 0.04; 0.3 | tip sd at the cutoff; sd when no still exists before the cutoff |
| `unc.rel_over` | 1 | 0.5 | half of a modelled change of overhang is in doubt |
| `unc.drift_w` | 1 | 0.01 | slow drift, sample widths per square root of a second |
| constants `norm.*` | 6 | `u_ref_cm_s` 5.0, `u_min_cm_s` 0.1, `o2_ref` 0.19, `o2_scale` 0.02, `w_ref_m` 0.022, `v_min` 0.001 | feature references and floors |
| constants `enc.*` | 3 | `end_s` 4.0 s, `min_span_s` 3.0 s, `change_cm_s` 0.8 | prefix windows and what counts as a change of flow |
| constants `num.*` | 2 | `dt_s` 0.25 s, `max_lead_s` 600 s | integration step, longest lead |

Full list with the author's comments: `ignis wsp inspect --release sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c --edits`,
or `configs/models/bass/front/d1-fable/edits.wsp`.

## 5. Forward and assembly

| | |
|---|---|
| Forward | `crates/ignis-wsp/src/front.rs`, entry point `ignis_wsp::front::forecast`, version 1, profile `strict_no_fitting` |
| forward_hash | `sha256:39e7e680d7a1415096f338c8cc581f3a1ed3f3ade88514a8b2c908a38b817a72` |
| code_version | `ignis-wsp@0.1.0+fwd.39e7e680d7a14150` |
| Spec | `bass_front-wsp-v1`, `sha256:8f09376336f9b7bbc8ad99817df35818bbfaf0189a365eeecb7c374228e9d2d8` |
| Patch | `bass_front-wsp-v1-d1-fable`, `sha256:4988d085385b3fbba27c01db98d8f7a0ddb7ba4884d7ea00353978dffc98009a` |
| weights_hash | `sha256:f29731bcacc96058a34c80f93e8b5264c62ebf552c5bf451acd48aa93fbe19c8` |
| Binding | cohort `sha256:f66dc826dc830db0efeec395ef4f2f57c6fe3b0b15853c6cc2fb9d2afc06b278`, split `sha256:4083156d5b18dab45889b274812d12fa9c144dec5880fb3a8906e380468b043a`, source manifest `sha256:405c774caa6926ecb5fa6c79969b092e713fffea79df3d724b6d19ac08ca48aa` (`configs/models/bass/front/binding.json`) |
| Candidate directory | `configs/models/bass/front/d1-fable` (`edits.wsp`, `meta.json`, `patch.json`, `release.json`) |
| Numerics | explicit step `num.dt_s` for the front, exact relaxation of the overhang; no request-time fitting |
| Checks | `spec_valid`, `patch_valid`, `tensors_complete`, `strict_forward_audit`, `finite_forward_on_fixture`: passed |
| Created | 2026-11-14T07:45:32Z |

Rebuild (the parent, persistence, is assembled first into the same registry; `assemble` writes the
release into `--root`, default `artifacts/` or `IGNIS_ARTIFACTS`):

```
cargo build --release -p ignis-cli
B=configs/models/bass/front
ignis wsp compile  --family bass_front --edits $B/baseline-persistence/edits.wsp --meta $B/baseline-persistence/meta.json --out $B/baseline-persistence/patch.json
ignis wsp assemble --family bass_front --patch $B/baseline-persistence/patch.json --binding $B/binding.json --root artifacts --out persistence.json
ignis wsp compile  --family bass_front --edits $B/d1-fable/edits.wsp --meta $B/d1-fable/meta.json --out $B/d1-fable/patch.json
ignis wsp assemble --family bass_front --patch $B/d1-fable/patch.json --binding $B/binding.json --root artifacts --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents artifacts
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release from its committed patch and binding and compares
the record; `served_ids` pins the 14 served ids. `cargo test -p ignis-wsp-serve --test served_ids`
passed (2 of 2) on 15 November 2026 at ignis-web a58c7c3: the served id is the content hash of its
record and loads through the registry checks. `tools/wsp/reassemble.py` holds the same chain
(`baseline-persistence`, `d1-fable`, `d2-fable`).

## 6. Evaluation

Metric: CRPS of the flame tip in sample widths (case = mean over its target stills, burn = mean over
its cases, table = mean over burns); lower is better. Scoring rules fixed in PROTOCOL section 9
before the validation score existed. The forecasts are conditional on the fan history.

**Validation, 12 burns, 40 cases, measured blind; read once** (SpaceApps-Ardealu:docs/wsp/RESULTS.md,
BASS section; card `docs/wsp/cards/bass_front.md`):

| | D1 | persistence |
|---|---|---|
| Tip CRPS, mean over burns | **0.117** | 0.234 |
| Absolute error of the median | 0.162 | 0.315 |
| Coverage of the 50 % / 90 % band | 0.66 / 0.90 | 0.34 / 0.62 |
| Has the track ended? (Brier, 10 to 60 s ahead) | **0.178** | 0.277 |
| Spread or no spread, from the conditions alone (Brier, 16 burns) | 0.170 | 0.125 (constant "it spreads") |

Persistence minus D1, paired over 40 cases: +0.117 [+0.051, +0.182] (95 % bootstrap over burns); D1 is
lower in 10 of 12 burns. By cutoff the error falls as the prefix grows (0.182, 0.123, 0.094, 0.040
at 5, 10, 20, 40 s); by lead it grows (0.067, 0.148, 0.167 for 0 to 10, 10 to 30, 30 to 60 s ahead).

**Against published spread laws, 3 validation burns, 11 cases**
(SpaceApps-Ardealu:docs/wsp/RESULTS_PUBLISHED_ACME_BASS.md, section 3): D1 **0.121**; a steady rate
fitted over the 10 s before the cutoff 0.228; Ferkul 2012 with an oxygen-ratio scaling 0.352; Ferkul
2012 as published 0.449. D1 is lower on all 3 burns against each.

**Train, 3 tracked burns, 12 cases** (the burns the numbers were typed from; in-sample;
SpaceApps-Ardealu:docs/wsp/RESULTS.md): tip CRPS 0.076 against 0.242 for persistence (paired +0.166
[+0.035, +0.359]); coverage 0.83 / 0.97.

**Saffire transfer, blind, unchanged weights** (registered in SpaceApps-Ardealu:docs/wsp/SAFFIRE_TRANSFER.md
before any measurement; outcome SpaceApps-Ardealu:docs/wsp/COVERAGE_19.md and dossier/04_decisions.md, D-49):

| Flight | Sample | Cases | D1 | persistence |
|---|---|---|---|---|
| Saffire I and III (PSI-98, PSI-100), deciding | 2 concurrent SIBAL sheets, 40.6 cm wide | 8 | **0.058** | 0.250 (D1 lower on 2 of 2 sheets) |
| Saffire II (PSI-99), reported | 2 strips of 5 cm; the measured tip is unreliable | | 0.654 | 0.852 |

By the registered rule D1 transfers to the wide sheet. The bands over-cover (0.93 at 50 %). No
sheet quenched, so the quench boundary is untested there.

**Test partition, 2 burns, 7 tip cases** (SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL.md, step 4; final
run of 15 November): tip CRPS 0.0839, absolute error of the median 0.1217, coverage 0.62 / 0.95,
track ended 0.2066, ignition outcome 0.0274. With two burns no comparison to persistence was run (it
has no power; RESULTS_PUBLISHED_ACME_BASS.md, section 3).

What the author had seen when each number was made: the train scores are in-sample; the validation,
Saffire and test scores are blind for D1 (no position, time or outcome of those burns was opened
before D1 was frozen). The D2 and later comparisons on the same validation tracks are second looks.

## 7. Adoption decision and lineage

**Why D1 is served.** In the fourth generation there was no carried BASS model; D1 was reported
against persistence on the validation burns with nothing adjusted afterwards (PROTOCOL section 9).
It halved the persistence error on the twelve blind burns and has stayed the carried release since:
every later candidate was held to a rule written before its look, and none met it
(SpaceApps-Ardealu:docs/wsp/README.md, carried models; dossier/04_decisions.md, D-05, D-31 to D-34).

**Lineage.**

| Step | Release | What it is |
|---|---|---|
| root | `sha256:ce53fe1fb3e6300c5e818dd7c9c5ec8e8f4c2cfd19673449d3936de05527de46` | persistence, reference baseline and lineage root: the tip keeps the speed the prefix shows; no law of flow, oxygen or width, no overhang, no quench |
| **D1 (served)** | `sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c` | the band law above; 35 non-zero numbers of 159 |

**Tried and not adopted.**

| Candidate | Release | What changed | Result | Decision |
|---|---|---|---|---|
| D2 (fourth generation) | `sha256:0187941810a89347a9ea3953add9aa6058152bb643b009ca1ffa3e9471f216a7` | D1 with the oxygen boundary of its quench law withdrawn; written after D1's validation table, not blind | tip 0.117 (identical case by case); track ended 0.183 against 0.178; spread or not 0.121 against 0.170 | not adopted: the rule asked for a track-ended score not worse (PROTOCOL section 9) |
| `bass_front.wsp2` P1 (fifth generation, `front2.rs`) | `sha256:d53e2944b62863014b875a8023b57b50ca28095f4d69c86e1a0c28fcf49d429b` | narrow-sample powers corrected from two PSI-26 burns (`speed.pow = [1.5, -0.5]`, `over.pow = [3.0, 1.0]`); D1 number for number on the 2 cm sample | tip 0.1137 against 0.1168 (paired -0.0031 [-0.0096, +0.0032]); track ended 0.1976 against 0.1780 | not adopted: track-ended worse (SpaceApps-Ardealu:docs/wsp/RESULTS_GEN5.md; D-34) |
| `bass_front.wsp5` Q0 and carry of D1 (PROTOCOL 12.9) | `sha256:00a8bc35…5e72`, `sha256:4da9967f…8ed4` | D1's tip law with a zero-shot boundary law, or with the boundary off | tip identical (0.1168); ignition outcome Q0 0.1928, carry 0.1333, D1 0.1703 | Q0 does not carry; D1 stays (PROTOCOL 12.9 outcome) |
| fabric quench revision after the final run (PROTOCOL 13.8) | | fabric oxygen floor 14.8 % to 14.1 % on Q0 | ignition 0.1612 against 0.1333 for the constant | not served; the fabric quench question stays refused (SpaceApps-Ardealu:docs/wsp/research/phys_quench_post_final.md) |

A later physics audit (SpaceApps-Ardealu:docs/wsp/research/phys_bass.md, section 3) supports the
spread line (physics gives 0.029 and 0.0061 at 18.8 %) and explains why the quench line did not carry
to another day: its oxygen slope is about ten times the heat-balance value, and at fan 42 the flow
calibration spans 0 to 3.75 cm/s across days, which the 2.5 per cm/s slope turns into a factor of
about 40 in hazard.

**Scope** (served `limits`, condensed):

- Use it for the flame-tip position only. Its quench boundary scored worse than a constant on
  whether the fabric spreads (0.170 against 0.125); read no quench or ignition forecast from it. For
  whether the fabric keeps burning use `bass5.wsp` S1.
- No run has a length calibration: the scores are in sample widths; the metric route needs a width in
  metres and is tested on a synthetic fixture only.
- Twelve burns of three sessions of one campaign.
- Flows above 11 cm/s, 21 % oxygen and the narrow sample are outside train.
- Not predicted: temperature, flame brightness, the flame base (a base median is in the diagnostics,
  unscored).
- Also covers Saffire I to III (PSI-98 to PSI-100) as blind transfers, unchanged (section 6).

## 8. How to call it in IGNIS

**Forecast lab.** Open `#/lab` in the web app; the Models screen `#/models` lists the served releases
and opens this model card. `web/src/views/LabModels.tsx` runs a stored run or a typed scenario; for
this release only the typed scenario works (`stored_runs = false`: no stored BASS run carries
`x_o2`, a width in metres and the fan history, so it refuses).

**Model card.** `GET /api/v1/model-cards` returns the served entry (inputs, scores, limits, cite).

**API.** First the scenario, which returns an `input_hash`:

```
POST /api/v1/models/sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c/scenarios
```

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c",
  "cutoff_s": 20.0,
  "conditions": { "x_o2": 0.19, "width": 0.022, "length": 0.1 },
  "controls": [
    { "control": "flow_velocity", "t_start_s": 0.0,  "t_end_s": 22.0, "value_si": 0.1 },
    { "control": "flow_velocity", "t_start_s": 22.0, "t_end_s": null, "value_si": 0.03 },
    { "control": "igniter",       "t_start_s": 4.0,  "t_end_s": 10.0, "value_si": null }
  ],
  "observations": [
    { "time_s": 11.0, "value_si": 0.002 }, { "time_s": 13.0, "value_si": 0.006 },
    { "time_s": 15.0, "value_si": 0.010 }, { "time_s": 17.0, "value_si": 0.014 },
    { "time_s": 19.0, "value_si": 0.018 }, { "time_s": 20.0, "value_si": 0.020 }
  ],
  "test_date": null,
  "phase_established_at_s": 10.0
}
```

These are the served example values: typed, not a measurement. The observations are flame-tip
positions in metres (the scenario fills them into the tip channel of the model's template). The flow
drops from 10 to 3 cm/s at 22 s, after the cutoff: a known control step that the forecast is
conditional on.

Then the analysis, which returns a job:

```json
POST /api/v1/analyses
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c",
  "input": { "kind": "prefix", "input_hash": "<input_hash from the scenario>" },
  "settings": { "times_s": [25, 30, 40, 60], "horizon_s": 60, "quantile_levels": [0.05, 0.25, 0.5, 0.75, 0.95] }
}
```

Poll `GET /api/v1/jobs/<job_id>` and read `GET /api/v1/artifacts/<artifact_id>`: an `AnalysisArtifact`
of evidence class `model_output` with the tip quantiles in metres, the four-category event table
(`active_supported`, `extinction`, `reached_end`, `unresolved_support`) and, in the diagnostics, the
quantiles in sample widths and the base median. The front forward has one target, the flame-tip
position; `settings.target` is not needed. A stored-run input (`{"kind": "stored_run", ...}`) is
refused for this release. MCP: `models_inspect` (the card) and `models_forecast` (the same scenario
and analysis).

Combination rule from the limits: read the tip from this release and "does the fabric keep burning"
from `bass5.wsp` S1; do not read the `extinction` row of this release as an ignition or quench
forecast.

Cite: bass_front.wsp D1, hand-written (literal WSP) release sha256:ac15e49f…855c, experimental, not trained; its output is model output, not a measurement.

## Sources

IGNIS (`D:\dev\ignis-web`, branch harness-docs-20261115, a58c7c3):

- `configs/models/served/served.json`; `configs/models/served/models/ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c/release.json`, `params.npz`
- `configs/models/bass/front/d1-fable/edits.wsp`, `meta.json`, `release.json`; `configs/models/bass/front/binding.json`
- `configs/models/bass/front/baseline-persistence/release.json`; `configs/models/bass/front/d2-fable/release.json`
- `crates/ignis-wsp/src/front.rs`, `crates/ignis-wsp/src/front_tools.rs`, `crates/ignis-wsp/src/cli.rs`
- `crates/ignis-wsp-serve/src/lib.rs` (`scenario_prefix`); `crates/ignis-core/src/contracts/wsp.rs` (`ScenarioRequest`)
- `tools/wsp/verify_served.py`, `tools/wsp/reassemble.py`
- `docs/wsp/cards/bass_front.md`, `docs/wsp/README.md`

Research (SpaceApps-Ardealu, lane branch feat/handwritten-science, 542dad8):

- `SpaceApps-Ardealu:docs/wsp/README.md`, `PROTOCOL.md` (sections 9, 12.9, 13.8), `RESULTS.md`, `RESULTS_GEN5.md`, `RESULTS_FINAL.md`, `RESULTS_PUBLISHED_ACME_BASS.md`, `COVERAGE_19.md`, `SAFFIRE_TRANSFER.md`
- `SpaceApps-Ardealu:docs/wsp/research/lit_flame_spread.md`, `exp_bass.md`, `phys_bass.md`, `phys_quench_post_final.md`
- `SpaceApps-Ardealu:docs/wsp/feedback/bass_train_measurement.md`, `bass_train_tracks.md`
- `SpaceApps-Ardealu:docs/wsp/dossier/04_decisions.md` (D-05, D-30 to D-34, D-49), `06_models.md` (section 6.6)
