# efield3.wsp — ion current and flame-out of a flame in an electric field

`efield3.wsp` forecasts what a microgravity diffusion flame does when a field is applied between its
burner and a mesh above it (NASA ACME E-FIELD Flames, PSI-22): the ion current at each planned voltage
level, the time at which the field puts the flame out, and the peak current of the sweep with its
voltage. On the unread test days of the seventh generation, scored once on a frozen list, its ion-current
CRPS is **0.319 µA against 0.575 µA for the space-charge baseline** and 0.986 µA for a constant, and its
flame-out Brier is **0.055 against 0.133** (SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md). It is a
data-informed law written by `claude-fable-5-2` from the train and validation days, and the served
release is `sha256:8575550b310950efde7da2f00a1964c9a4d0c393fd3fbcaf4e4f2f1f8f046f04`.

| | |
|---|---|
| Model | `efield3.wsp` |
| Investigation | E-FIELD Flames (PSI-22) |
| Family | `other` (generic law slug `efield3`) |
| Candidate | S1 (`configs/models/law/efield3/s1-fable`) |
| Generation | seventh (15 November 2026), PROTOCOL 13.6 study candidate, scored in the final run of 13.7 |
| Author | `claude-fable-5-2`; transcribed by the assembly engineer `claude-opus-5-5` |
| Authorship | data-informed (development days: train and validation) |
| Release | `sha256:8575550b310950efde7da2f00a1964c9a4d0c393fd3fbcaf4e4f2f1f8f046f04` |
| Forward | `ignis_wsp::law::forecast`, `crates/ignis-wsp/src/law.rs` (profile `strict_no_fitting`, version 1) |
| Parent release | `sha256:f78c2ece35c2e4808e00edb7aca5c797fe668edb11d9f6250081ef1e9c38efd8` (neutral root of slug `efield3`, `configs/models/law/efield3/root`) |

## 1. Task and targets

Every input is a typed condition known before the test; burner, coflow and polarity are part of the
plan (SpaceApps-Ardealu:docs/wsp/feedback/efield_planned_fields.md). Source of the table:
`configs/models/served/served.json` and the declaration `configs/models/law/efield3/decl.json`.

| Name | Kind | SI unit | Transform | Note |
|---|---|---|---|---|
| `carbon_flow` | condition | 1/s | ln | fuel carbon atoms per second (case input `carbon_flow_atoms_s`) |
| `x_fuel` | condition | 1 | ln | fuel mole fraction of the fuel stream |
| `is_ethylene` | condition | 1 | identity | 1 when fuel = ethylene |
| `mesh_positive` | condition | 1 | identity | 1 when polarity = positive |
| `voltage` | condition | V | ln | magnitude of the level's `v_cmd`; for the targets without a level, the highest magnitude of the sweep |
| `sweep_v_max` | condition | V | ln | largest magnitude of `v_cmd` over the programme (the planned maximum) |
| `window` | condition | s | ln | `window_s` of the field programme |
| `is_gas_jet` | condition | 1 | identity | 1 if burner = `gas_jet_2.1mm`, else 0 |
| `still_coflow_burner` | condition | 1 | identity | 1 if burner = `coflow` and `u_coflow_m_s` = 0, else 0 |
| `strong_coflow` | condition | 1 | identity | 1 if `u_coflow_m_s` >= 0.1, else 0 |

All ten are required; a value outside the declared domain is refused with its name.

| Target | Kind | Unit | Meaning |
|---|---|---|---|
| `ion_current` | positive | A | flame current at a planned voltage level, positive in the direction the field drives it, given that the flame still burns (lane-local label `efield3:ion_current`) |
| `flame_end` | event, causes `blowoff`, `press`, `other` | s | the flame goes out under the field; no scheduled stop declared |
| `peak_ion_current` | positive | A | largest current of the sweep (lane-local) |
| `peak_voltage` | positive | V | voltage at which the largest current is reached (lane-local) |

Time origin of `flame_end`: the field switched on. The positive targets are returned at
`times_s = [cutoff_s]`. A stored run cannot be used (`stored_runs: false`: no stored run of this
investigation carries the declared features); a typed scenario must give all ten conditions and no
observations.

## 2. Physical mechanism

A hydrocarbon flame makes roughly one ion pair per million carbon atoms burnt, in the reaction
`CH + O -> CHO+ + e-`. The field pulls the charges apart; at low voltage the current is limited by the
space charge of the ions crossing the gap, at high voltage by how many the flame makes
(SpaceApps-Ardealu:docs/wsp/research/phys_efield_w0.md, sections 1.1 to 1.3). The study of the
development cases (SpaceApps-Ardealu:docs/wsp/research/phys_efield_study.md, sections 1 and 2) kept that
picture and found what the zero-shot law lacked: the polarity of the mesh decides the ceiling, the rise
above 1 kV goes as $V^{1.4}$, and burner and coflow decide when the flame goes out.

With $`z_c = \ln(N_C/10^{19}\,\mathrm{s^{-1}})`$, $z_x = \ln x_\mathrm{fuel}$, $`z_v = \ln(V/3\,\mathrm{kV})`$,
$`z_m = \ln(V_\mathrm{max}/5\,\mathrm{kV})`$, $`z_w = \ln(\mathrm{window}/100\,\mathrm{s})`$, $e$ ethylene,
$p$ mesh positive, $g$ gas-jet, $s$ still coflow burner, $q$ strong coflow:

**Rising branch** (tensors `ion_current.median.b0`, `.skip`, hinge unit 2). Quadratic below 1 kV
($G$ = 0.25 µA/kV², 2.46 times with the mesh positive), $V^{1.4}$ above; the gas-jet tube, closer to the
mesh, 1.4 times higher:

```math
\ln a = \ln(1.164\,\mu\mathrm{A}) + 1.4\,z_v + 0.9\,p + 0.35\,g - 0.6\,\mathrm{relu}\big(\ln(1\,\mathrm{kV}/V)\big)
```

**Ceilings**, each a hinge that subtracts the excess of $\ln a$ over it and is switched on or off by an
indicator with weight ±20, so no hinge sits inside another:

- mesh negative (unit 0): $`\ln A = \ln(1.28\,\mu\mathrm{A}) + z_c + 0.4\,z_x + 1.10\,e`$ (0.8e-6 ion pairs
  per carbon, $x^{0.4}$, ethylene three times);
- mesh positive, coflow burner with weak coflow (unit 1): $`\ln B = \ln(0.5\,\mu\mathrm{A}) + 0.25\,z_c + 0.95\,e`$,
  0.3 times that in still gas ($`+1.2\,s`$), off for the gas-jet and for a strong coflow;
- mesh positive, gas-jet (unit 3): 0.45 µA methane, 2.6 µA ethylene, independent of flow;
- mesh positive with a real coflow (unit 4): the negative-mesh ceiling times 0.52.

```math
\ln I = \ln a - \sum_k \mathrm{relu}(\ln a - \ln C_k \pm 20\,\text{indicators}),\qquad \sigma_{\ln} = 0.45
```

The physics of the positive ceiling (study 1.2): with the mesh positive the ion wind blows against the
fuel jet and presses the flame on to the tube, so the ceiling is a property of the pressed flame, not of
the carbon flow; a real coflow wins against the wind and restores the carbon-limited value, and in still
gas the pressed flame starves.

**Flame-out** (tensors `flame_end.<cause>.*`), three log-logistic clocks
$S_c(t) = 1/(1+(t/\tau_c)^{k_c})$, $S = \prod_c S_c$ (the law forward of
SpaceApps-Ardealu:docs/wsp/research/law_forward.md, section 2.3):

- `blowoff` (gas-jet, methane, mesh negative): $`\tau = \mathrm{window}\cdot 7.3\,\mathrm{kV}/V_\mathrm{max}`$,
  shape 12; every other flame keeps $`\tau = \mathrm{window}\cdot 60\,\mathrm{kV}/V_\mathrm{max}`$;
- `press` (gas-jet, methane, mesh positive): 7.0 kV, shape 4 (the pressed flame fails over a wide range of voltage);
- `other` (a flame on the coflow burner in still gas starves): median 45 s with the mesh positive,
  600 s with the mesh negative, 3000 s for every other flame; shape 1.5.

**Secondary targets** (unchanged from W0, phys_efield_w0.md section 3):
$`\ln I_\mathrm{peak} = \ln(2.40\,\mu\mathrm{A}) + z_c + z_x`$ and
$`\ln V_\mathrm{sat} = \ln(3.90\,\mathrm{kV}) + 0.5\,z_c + 0.5\,z_x - 0.203\,p`$.

Every target goes through the one network of the law forward,
$`N(z) = b_0 + \mathrm{skip}\cdot z + \sum_{j\lt8} w_{2,j}\,\mathrm{relu}(b_{1,j} + W_{1,j}\cdot z)`$, with a
log-normal core plus a fixed wider tail for positive targets.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section efield3.wsp.

## 3. Authoring

- **Who.** The numbers are `claude-fable-5-2`'s, typed in the study dossier
  SpaceApps-Ardealu:docs/wsp/research/phys_efield_study.md (section 4, lines 200-248, sha256
  `573d8dbc…da6e`) and copied to `runs/dev/gen7/study/efield3_block.wsp` (sha256 `3810450a…03ec`). The
  release record says: "Transcribed by the assembly engineer (claude-opus-5-5); no number chosen."
  (`configs/models/law/efield3/s1-fable/meta.json`). The author typed the block without running it: the
  what-if tool could not run a declaration that did not yet exist (study section 4).
- **Generation and step.** Seventh generation, PROTOCOL 13.6 (a study of the development data, then a
  typed candidate), as a new slug because a declaration is frozen with its release (13.4, 13.6
  "structure before numbers").
- **What was read.** The exposure record lists dataset `efield-train-and-validation-cases`, partitions
  `train` and `validation`; development feedback `PROTOCOL-13.2-to-13.6-development-scores` and
  `what-if-run-of-E1`; prior candidates W0 (`sha256:73f91b55…86a9`) and E1 (`sha256:dbb4bd64…73b0`).
  The study states it was written from 3259 scored voltage levels of 152 tests and 156 flame-out
  records, read test by test, with no test day opened.
- **Brief.** See [docs/research/PROMPTS.md](../../research/PROMPTS.md). The dossier the author worked
  from is phys_efield_study.md, on top of the zero-shot dossier phys_efield_w0.md.
- **Decisions.** efield3 = E1's ion current plus a new flame-out law (meta.json rationale):
  - three indicators appended to the declaration of `efield` (indices 7, 8, 9), because the flame-out
    events "are decided by burner, coflow and polarity, none of which the present declaration carries
    except polarity" (study 1.3); `sweep_v_max` confirmed as the planned maximum (efield_planned_fields.md);
  - ion current: the gas-jet rising branch 1.4 times, and three new positive-mesh ceilings (gas-jet,
    still gas, real coflow) to close the three days E1 left worst (18137, 18319, 18106; study 2);
  - flame-out: W0's single blow-off voltage (20 kV times $x^{1.5}$) replaced by the class table of
    study 1.3: no flame out among 65 coflow tests with coflow and 50 ethylene tests; 13 of 26 still-gas
    positive-mesh methane flames out within 0 to 60 s; gas-jet methane out near 7 kV.
  - The parent check (SpaceApps-Ardealu:docs/wsp/feedback/efield3_result.md): the block writes 150
    elements, 58 of them differ from E1, and no other element differs from E1.

## 4. Weights

Features in column order $z$ = (z_c, z_x, e, p, z_v, z_m, z_w, g, s, q), indices 0 to 9. Values from the
pack of the release's params.npz and the comments of `configs/models/law/efield3/s1-fable/edits.wsp`.

| Group | Shape / count | Value(s) | Physical meaning |
|---|---|---|---|
| `x.carbon_flow.*` | 4 scalars | centre 1e19, scale 1, lo 1e18, hi 1e20 | carbon atoms per second; declared domain |
| `x.x_fuel.*` | 4 | centre 1.0, scale 1, lo 0.2, hi 1.0 | fuel fraction |
| `x.voltage.*` | 4 | centre 3000, scale 1, lo 100, hi 10000 | level voltage, V |
| `x.sweep_v_max.*` | 4 | centre 5000, scale 1, lo 100, hi 10000 | planned sweep maximum, V |
| `x.window.*` | 4 | centre 100, scale 1, lo 5, hi 2000 | programme window, s |
| `x.is_ethylene.*`, `x.mesh_positive.*`, `x.is_gas_jet.*`, `x.still_coflow_burner.*`, `x.strong_coflow.*` | 5 × 4 | centre 0, scale 1, lo 0, hi 1 | indicators |
| `ion_current.median.b0` | 1 | -13.664 | ln(1.164 µA), rising branch at 3 kV |
| `ion_current.median.skip` | 10, 3 nonzero | p 0.9, z_v 1.4, g 0.35 | mesh-positive factor 2.46, $V^{1.4}$, gas-jet 1.4 times |
| `ion_current.median` unit 0 | W1 row 0 (6 nonzero), b1 -0.095, w2 -1.0 | W1 = [-1.0, -0.4, -1.1, -20.0, 1.4, 0, 0, 0.35, 0, 0] | negative-mesh ceiling, 0.8e-6 per carbon, $x^{0.4}$, ethylene 3 times |
| `ion_current.median` unit 1 | W1 row 1 (7 nonzero), b1 -18.255, w2 -1.0 | W1 = [-0.25, 0, -0.95, 20.0, 1.4, 0, 0, -20.0, 1.2, -20.0] | positive-mesh ceiling of the coflow burner, 0.5 µA methane, 1.3 µA ethylene; 0.3 times in still gas; off for gas-jet and strong coflow |
| `ion_current.median` unit 2 | W1[2,4] -1.0, b1 -1.0986, w2 -0.6 | | quadratic below 1 kV (ln 3 = 1.0986) |
| `ion_current.median` unit 3 | W1 row 3 (4 nonzero), b1 -37.800, w2 -1.0 | W1 = [0, 0, -1.754, 20.0, 1.4, 0, 0, 20.0, 0, 0] | positive-mesh gas-jet ceiling, 0.45 µA methane, 2.6 µA ethylene |
| `ion_current.median` unit 4 | W1 row 4 (6 nonzero), b1 -38.545, w2 -1.0 | W1 = [-1.0, -0.4, -1.10, 20.0, 1.4, 0, 0, 0, 0, 20.0] | positive mesh with a real coflow: negative ceiling times 0.52 |
| `ion_current.spread.*` | b0 0.45; W1 row 0 [-1.0, -1.0, 0, 0.405, 2.0, …], b1 -0.824, w2 0.0 | ln-sd 0.45 everywhere | W0's widening hinge kept but switched off by E1 (w2 = 0) |
| `ion_current.tail_weight`, `.tail_width` | 2 | 0.1, 3.0 | 10 % of cases three times wider |
| `flame_end.blowoff.*` | b0 7.09; skip z_m -1, z_w 1; W1 row 0 (e -1, p -1, g 1), b1 0.0, w2 -2.106; shape 12.0 | | ln(100 s) + ln(60 kV / 5 kV); gas-jet methane mesh negative at 7.3 kV |
| `flame_end.press.*` | b0 7.090; skip z_m -1, z_w 1; W1 row 0 (e -1, p 1, g 1), b1 -1.0, w2 -2.148; shape 4.0 | | gas-jet methane mesh positive at 7.0 kV |
| `flame_end.other.*` | b0 8.006; skip all 0; W1 row 0 (e -1, p 1, s 1), b1 -1.0, w2 -4.2; W1 row 1 (e -1, p -1, s 1), b1 0.0, w2 -1.6; shape 1.5 | | 3000 s (ln 3000 = 8.006); 45 s still gas mesh positive; 600 s still gas mesh negative |
| `peak_ion_current.*` | median b0 -12.939, skip z_c 1, z_x 1; spread b0 0.55, skip z_x -0.5, p 0.10; tail 0.1, 3.0 | | W0: saturation current ln(2.40 µA) |
| `peak_voltage.*` | median b0 8.268, skip z_c 0.5, z_x 0.5, p -0.203; spread b0 0.45, skip z_x -0.25; tail 0.1, 3.0 | | W0: saturation voltage 3.90 kV |
| `num.substeps` | 1 (constant) | 512 | integration steps per requested interval of the event target |

Totals (release.json `architecture.parameter_counts`): **1012 authored elements, 120 nonzero, 1
constant** (`num.substeps`); 95 tensors. Every element not listed is zero. The full list:
`ignis wsp inspect --release sha256:8575550b310950efde7da2f00a1964c9a4d0c393fd3fbcaf4e4f2f1f8f046f04 --edits`,
or `configs/models/law/efield3/s1-fable/edits.wsp`. The edits file carries the whole chain W0, E1,
efield3 with each later number written in place, so several comments above changed lines are W0's
original comments (for example the "ln(2.403e-6 A)" beside `ion_current.median.b0 = -13.664`); the
meaning of the current values is the one in study sections 2 and 4.

## 5. Forward and assembly

| Field | Value |
|---|---|
| Forward | `crates/ignis-wsp/src/law.rs`, entry `ignis_wsp::law::forecast`; slug registered in `crates/ignis-wsp/src/law_slugs.rs` (`slug!("efield3")`) |
| forward_hash | `sha256:c0a9c1512a2875f407ba629e76c642bf7422be329ed8a3c09030064bb1b37a06` |
| code_version | `ignis-wsp@0.1.0+fwd.c0a9c1512a2875f4` |
| spec | `law-efield3-v1`, `sha256:e97878bc66f11bd8317f00eb3ce386dded48ad5289ce69a5299de1d8844d6f9b` |
| patch | `law-efield3-v1-s1-fable`, `sha256:1b33b13252e9672b31129938f0df4902aaa81061b4e0eb704d4e5738ec5df458` |
| weights_hash | `sha256:827f1bb2a36c104f5d7ef7ccf1dff5a4ec234c8c2621c4bc0afffeb7dab2a524` |
| binding | cohort `atlas:PSI-22:catalogue`, split `sha256:eea9c9f2afb906ca12398701f6a49db62434b242feccc2ef7cd5d22d3c32bdf6`, source manifest `sha256:c34470df1cd271d589911e26a433234f8337bd69ea31a9e54c0e5c3786c2101d` (`configs/models/law/efield3/binding.json`) |
| Candidate directory | `configs/models/law/efield3/s1-fable` (edits.wsp, meta.json, patch.json, release.json) |
| Checks | spec_valid, patch_valid, tensors_complete, strict_forward_audit, finite_forward_on_fixture: passed |
| Numerics | event: `num.substeps` equal steps per interval, exact survival, causes share each step's loss by midpoint hazards; positive quantiles by bisection of the fixed two-component mixture CDF; request-time fitting: none |

Rebuild (the parent is the neutral root `configs/models/law/efield3/root`, which is not in the served
registry; the registry passed to `--root` and `--parents` must hold
`models/f78c2ece35c2e4808e00edb7aca5c797fe668edb11d9f6250081ef1e9c38efd8/`):

```
cargo build --release -p ignis-cli
ignis wsp compile  --family efield3 --edits configs/models/law/efield3/s1-fable/edits.wsp --meta configs/models/law/efield3/s1-fable/meta.json --out configs/models/law/efield3/s1-fable/patch.json
ignis wsp assemble --family efield3 --patch configs/models/law/efield3/s1-fable/patch.json --binding configs/models/law/efield3/binding.json --root <registry holding sha256:f78c2ece…efd8> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the record; `served_ids` pins the 14
served ids. `cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at
ignis-web a58c7c3: the served id is the content hash of its record and loads through the registry checks.

## 6. Evaluation

All scores are the lane metrics of `tools/wsp/ext/law_score.py` (research repository); lower is
better. t1: CRPS of the ion current in µA per voltage level; t2: Brier of a flame end within the
programme window, and the integrated Brier over time.

### Final run, test days (headline)

Scored once on the frozen release list sha256 `f3556071…70e2`, test days of the seventh generation
(PROTOCOL 13.7: 4 days); the author had seen train and validation days, **not** the test days. Source:
SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md, section PSI-22.

| Release | Role | Answered | t1: CRPS µA, case mean | t1: mean of day means | t2: Brier of ends in window | t2: integrated Brier |
|---|---|---|---|---|---|---|
| **efield3 (third law)** | model | t1 788, t2 39 | **0.3190** | **0.3135** | **0.0547** | 0.0348 |
| efield E1 | comparator | 788, 39 | 0.3238 | 0.3115 | 0.1072 | 0.0282 |
| efield W0 (zero-shot parent of 13.2) | comparator | 788, 39 | 0.7039 | 0.7085 | 0.1072 | 0.0282 |
| efield2 (second law) | comparator | 788, 39 | 0.7719 | 0.8066 | 0.1964 | 0.0898 |
| constant (simple) | baseline | | 0.9864 | 0.9847 | 0.1331 | 0.0394 |
| space-charge law up to saturation, train sigma (T1) | baseline | | 0.5746 | 0.5625 | - | - |
| stratified incidence (T2) | baseline | | - | - | 0.1456 | 0.0551 |

Reading by the rule written beforehand (PROTOCOL 13.7 outcome): ion current **holds** (lower than every
baseline) and the generation's improvement holds (0.319 against W0's 0.704); flame out in the window
**holds** (0.0547 against 0.1331); flame out by integrated Brier holds against the baselines (0.0348
against 0.0394) and is not better than the zero-shot parent (0.0282).

### Development days, in-sample

Train and validation days, which the author had studied case by case before typing; scored on the
cases all four releases answer (train 2735 levels / 122 tests, validation 1289 / 36). Source:
SpaceApps-Ardealu:docs/wsp/feedback/efield3_result.md (producer `runs/dev/gen7/third_laws_dev.json`).

| Release | Partition | t1: CRPS µA | t1: mean of day means | t2: integrated Brier | t2: Brier of ends in window |
|---|---|---|---|---|---|
| **efield3** | pooled | **0.2103** | 0.2000 | **0.0630** | 0.0545 |
| efield3 | train | 0.1913 | 0.1772 | 0.0555 | 0.0429 |
| efield3 | validation | 0.2455 | 0.2628 | 0.0850 | 0.0951 |
| E1 | pooled / train / validation | 0.2530 / 0.2681 / 0.2252 | 0.2428 / 0.2487 / 0.2263 | 0.1478 / 0.1761 / 0.0753 | 0.1484 / 0.1689 / 0.0762 |
| W0 | pooled / train / validation | 0.6592 / 0.7202 / 0.5463 | 0.6324 / 0.6690 / 0.5320 | 0.1478 / 0.1761 / 0.0753 | 0.1484 / 0.1689 / 0.0762 |
| efield2 s1 | pooled / train / validation | 0.7206 / 0.7857 / 0.6002 | 0.6733 / 0.7224 / 0.5384 | 0.1001 / 0.1015 / 0.0975 | 0.1303 / 0.1220 / 0.1596 |
| constant (simple) | pooled | 0.5625 | 0.5612 | 0.1252 | 0.1533 |
| space-charge law, train sigma | pooled | 0.4256 | 0.4267 | - | - |
| stratified incidence | pooled | - | - | 0.0697 | 0.1237 |

Against E1, efield3 is lower on t1 on 8 of the 9 days whose scores differ, and on t2 (integrated
Brier) on 13 of 15. The largest gains are where the study predicted them: day 18137 0.796 to 0.387,
18319 0.563 to 0.451, 18106 0.152 to 0.038 on t1. On the four validation days alone efield3 is worse
than E1 on both targets (t1 0.2455 against 0.2252; t2 0.0850 against 0.0753), and three validation
days moved against the author's written expectation (18285 on t1; 18078 and 18298 on t2).

### Blind look of the zero-shot root of the lineage

The W0 law (`sha256:73f91b55…86a9`), typed with train aggregates seen, scored once on the validation
days (36 tests, 1289 levels): ion current 0.5463 µA against 0.6202 for a constant and 0.3742 for the
space-charge baseline; flame out 0.0753 against 0.0884 (SpaceApps-Ardealu:docs/wsp/feedback/efield_w0_result.md;
SpaceApps-Ardealu:docs/wsp/COVERAGE_19.md).

## 7. Adoption decision and lineage

**Why it is served.** Under the 13.6 keep rule (lower than the current model pooled and on at least two
thirds of the days whose forecasts change, for the target that loses, and not worse by more than 5 %
pooled on the others) efield3 passed against E1 on both targets: ion current 0.2103 against 0.2530 (8 of
9 days), flame out 0.0630 against 0.1478 (13 of 15) (PROTOCOL 13.6 further outcomes; efield3_result.md).
Because it was worse than E1 on the validation days, E1 went to the final run beside it (PROTOCOL 13.6,
flags; 13.7 known risks). The final run confirmed it as the model: it holds on every scored target
against the baselines, and PROTOCOL 13.7 lists `efield3` among the releases served after the run, with
no number changed after the run. COVERAGE_19.md records the same: 0.319 against 0.575 on the ion
current, 0.055 against 0.133 on flame out in the window.

**Lineage.**

| Step | Release | Candidate | What it changed | Outcome |
|---|---|---|---|---|
| efield root | `sha256:ba9789c6…3a32` | `configs/models/law/efield/root` | neutral root, no claim | lineage root |
| W0 | `sha256:73f91b55…86a9` | `configs/models/law/efield/w0-fable` | zero-shot law of phys_efield_w0.md section 7.4: $\min(I_\mathrm{sat}, GV^2)$ with a $1/V$ fall, yield 1.5e-6 per carbon, $x^{1.0}$; blow-off when $`V_\mathrm{ext} = 20\,\mathrm{kV}\,x^{1.5}`$ lies below saturation | blind look 13.2; beat the constant, not the space-charge baseline on t1 |
| efield2 s1 | `sha256:cf5459b5…d8c5` | `configs/models/law/efield2/s1-fable` | second law (13.4): `has_coflow` input, four damped numbers | not kept: t1 0.7206 against 0.6592 (efield2_second_law_result.md) |
| E1 | `sha256:dbb4bd64…73b0` | `configs/models/law/efield/e1-fable` | 13.6 study: two ceilings switched by polarity, $V^{1.4}$, no fall, width 0.45; 31 elements written | kept: t1 0.2530, 15 of 15 days; flame out unchanged |
| efield3 root | `sha256:f78c2ece…efd8` | `configs/models/law/efield3/root` | neutral root of the new declaration (three indicators, three causes) | parent of S1 |
| **efield3 S1** | `sha256:8575550b…6f04` | `configs/models/law/efield3/s1-fable` | E1 lines in place plus the block of study section 4 | the model |

What it replaced: E1 as the E-FIELD model. Tried and not adopted: efield2 (second law) and E1 remain
comparators.

**Scope.**

- Targets served: `ion_current`, `flame_end`, `peak_ion_current`, `peak_voltage`; choose one with
  `settings.target` (default the first). Event targets need `settings.times_s` or `horizon_s` (served
  `limits`).
- Typed conditions only; inputs must lie inside the declared domain (section 4).
- On the validation days alone efield3 scored above E1 on both targets; its flame-out integrated Brier on
  the test days (0.0348) is above W0 and E1 (0.0282).
- Not typed (study section 4): the fall of the current when the flame weakens, the strong-coflow rising
  branch, the reason for $V^{1.4}$. Not covered (W0 section 3): corona and arcs above about 8 kV; the
  recording channel clips at 9.3 µA.
- `peak_ion_current` and `peak_voltage` carry W0's numbers and were not scored in the final run.
- A test with no coflow field cannot be answered (`strong_coflow` missing; one train test, 18319U1).

## 8. How to call it in IGNIS

- **Forecast lab** (web, route `#/lab`): the Models screen `#/models` lists the releases and opens the
  model card; `web/src/views/LabModels.tsx` runs a typed scenario (stored runs are not available for this
  model: `stored_runs: false`).
- **Model card**: `GET /api/v1/model-cards` (fields from `configs/models/served/served.json`, with the
  cite string).
- **API**: `POST /api/v1/models/sha256:8575550b310950efde7da2f00a1964c9a4d0c393fd3fbcaf4e4f2f1f8f046f04/scenarios`
  returns an `input_hash`; then `POST /api/v1/analyses` returns a job; poll `GET /api/v1/jobs/<job_id>`
  and read `GET /api/v1/artifacts/<artifact_id>` (an `AnalysisArtifact`, evidence_class `model_output`).
- **MCP tools**: `models_inspect`, `models_forecast`.

The served entry has no `example` block. The values below are typed for illustration (a methane gas-jet
flame with the mesh negative, at the centre of the declared domain), not measurements:

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:8575550b310950efde7da2f00a1964c9a4d0c393fd3fbcaf4e4f2f1f8f046f04",
  "cutoff_s": 0.0,
  "conditions": {
    "carbon_flow": 1e19,
    "x_fuel": 1.0,
    "is_ethylene": 0,
    "mesh_positive": 0,
    "voltage": 3000.0,
    "sweep_v_max": 8000.0,
    "window": 100.0,
    "is_gas_jet": 1,
    "still_coflow_burner": 0,
    "strong_coflow": 0
  },
  "controls": [],
  "observations": [],
  "test_date": null,
  "phase_established_at_s": null
}
```

Then, for the flame-out clock:

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:8575550b310950efde7da2f00a1964c9a4d0c393fd3fbcaf4e4f2f1f8f046f04",
  "input": {"kind": "prefix", "input_hash": "<input_hash>"},
  "settings": {"target": "flame_end", "times_s": [10, 30, 60, 100], "quantile_levels": null}
}
```

`settings.target` is one of `ion_current` (default), `flame_end`, `peak_ion_current`, `peak_voltage`;
the positive targets take `quantile_levels` (default 0.05, 0.25, 0.5, 0.75, 0.95), `flame_end` needs
`times_s` or `horizon_s`. For `ion_current`, `voltage` is the level asked about; for the targets without
a level it is the highest magnitude of the sweep.

Cite: efield3.wsp S1, hand-written (literal WSP) release sha256:8575550b…6f04, experimental, not trained; its output is model output, not a measurement.

## Sources

IGNIS (ignis-web, branch harness-docs-20261115):
- `configs/models/served/served.json`; `configs/models/served/models/8575550b310950efde7da2f00a1964c9a4d0c393fd3fbcaf4e4f2f1f8f046f04/release.json`
- `configs/models/law/efield3/s1-fable/edits.wsp`, `meta.json`, `release.json`; `configs/models/law/efield3/binding.json`; `configs/models/law/efield3/root/meta.json`, `release.json`
- `configs/models/law/efield/root/release.json`, `w0-fable/meta.json`, `w0-fable/release.json`, `e1-fable/meta.json`, `e1-fable/release.json`; `configs/models/law/efield2/s1-fable/meta.json`, `release.json`
- `crates/ignis-wsp/src/law_slugs.rs`; `crates/ignis-wsp/src/law.rs` (entry point); `crates/ignis-wsp-serve/src/lib.rs` (scenario admission); `tools/wsp/verify_served.py`; `tools/wsp/reassemble.py`; `docs/wsp/README.md`
- data pack of the release (params.npz values)

Research (SpaceApps-Ardealu, lane branch feat/handwritten-science, 542dad8):
- `docs/wsp/research/law_forward.md`, `law_forward_targets.md`
- `docs/wsp/research/phys_efield_w0.md`, `phys_efield2.md`, `phys_efield_study.md`
- `docs/wsp/feedback/efield_w0_result.md`, `efield_e1_result.md`, `efield2_second_law_result.md`, `efield3_result.md`, `efield_planned_fields.md`
- `docs/wsp/RESULTS_FINAL_GEN7.md`, `docs/wsp/COVERAGE_19.md`, `docs/wsp/PROTOCOL.md` (sections 12, 13.2 to 13.7)
