# spice.wsp — smoke points and flame lengths of microgravity jet flames (SPICE)

`spice.wsp` forecasts, for a laminar gas-jet diffusion flame in coflowing air aboard the ISS (SPICE,
PSI-107), the length at which the flame starts to emit soot (its smoke point), the fuel flow at that
point, the luminous length at a given flow and whether soot leaves the tip. On the six unread test
smoke points it scored 4.18 mm on smoke-point length against 10.3 mm for the per-fuel baseline, and
1.18 against 2.56 for the Lin and Faeth coflow law on flame length given the flow
(SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md). It is data-informed: candidate P3, written by
`claude-fable-5-2` after studying the 49 development smoke points (train and validation days). Release
`sha256:000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0`.

| | |
|---|---|
| Model | `spice.wsp` |
| Investigation | SPICE (PSI-107), smoke points of laminar jet flames in the Microgravity Science Glovebox |
| Family | `other` (generic law slug `spice`) |
| Candidate | P3 (`configs/models/law/spice/p3-fable`) |
| Generation | seventh (15 November 2026), PROTOCOL 13.6 (study of the development data) |
| Author | `claude-fable-5-2`; transcribed by the assembly engineer `claude-opus-5-5` |
| Authorship | data-informed (train and validation days; the published fit had been read, so not blind) |
| Release | `sha256:000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0` |
| Forward | `ignis_wsp::law::forecast`, `crates/ignis-wsp/src/law.rs` (version 1, profile `strict_no_fitting`) |
| Parent release | `sha256:f1fe69f0956eddbbc4333187b1668d0e3b8e6ddc8af649872408e99581e3258a` (spice P2) |

## 1. Task and targets

Inputs (all typed conditions, all required; `configs/models/served/served.json`, declaration
`configs/models/law/spice/decl.json`):

| Name | Kind | SI unit | Transform | Note |
|---|---|---|---|---|
| `nozzle_diameter` | condition | m | ln | inner diameter of the fuel nozzle |
| `coflow_speed` | condition | m/s | ln | duct air speed (anemometer) |
| `fuel_mass_flow` | condition | kg/s | ln | mass flow of all gas through the nozzle; the two smoke-point targets give it weight zero |
| `x_fuel` | condition | 1 | ln | fuel mole fraction of the nozzle gas (balance nitrogen) |
| `is_propane` | condition | 1 | identity | indicator |
| `is_propylene` | condition | 1 | identity | indicator |
| `is_methane` | condition | 1 | identity | indicator (ethylene is the reference fuel) |

Targets (`release.json`, `observation_model.targets`):

| Target | Kind | Unit | Meaning |
|---|---|---|---|
| `smoke_point_length` | positive | m | luminous length, nozzle to tip on the axis, at incipient soot emission (lane-local, label `spice:smoke_point_length`) |
| `smoke_point_fuel_flow` | positive | kg/s | nozzle gas mass flow at the smoke point (shared quantity `mass_flow`) |
| `flame_length` | positive | m | luminous length of the flame at a setting (lane-local) |
| `emits_soot` | binary | none | at this setting soot leaves the tip (at or past the smoke point) |

The law has no time dependence: each answer is returned at `times_s = [cutoff_s]` (quantiles for the
positive targets, `yes`/`no` for `emits_soot`;
SpaceApps-Ardealu:docs/wsp/research/law_forward.md, section 2.4). No stored run carries the declared
features (`stored_runs: false`), so every call is a typed scenario that gives all seven conditions; a
value outside the declared domain is refused with its name. The scored "flame length given the flow"
(T2) is `flame_length` with `fuel_mass_flow` set to the flow at the smoke point.

## 2. Physical mechanism

**Length.** A laminar jet flame ends where enough air has diffused to the axis; its length is
proportional to the fuel flow and independent of the nozzle (Burke and Schumann, Roper, Lin and Faeth):

```math
L = \frac{C_f\,Sc}{4\pi\mu}\,\frac{\dot m}{Z_{st}}\,F(r) = K\,\frac{\dot m}{Z_{st}}\,F(r),\qquad K = 1.30\ \mathrm{mm\ per\ mg/s},\qquad r = \frac{u_{air}}{u_{fuel}}
```

with $F$ passing from 3/2 (a momentum jet in still air) to 1 (strong coflow)
(SpaceApps-Ardealu:docs/wsp/research/phys_spice_w0.md, section 1.1). P3 keeps this law and changes how
it is applied (SpaceApps-Ardealu:docs/wsp/research/phys_spice_study.md, section 5):

```math
L = c_f[\mathrm{fuel}]\;K\,\frac{\dot m}{Z_{st}}\left(\frac{r}{0.3}\right)^{-0.08} + 1.3\,d
```

- $c_f$ = 1.079 (ethylene), 1.169 (propane), $`1.324\,x_{fuel}^{0.22}`$ (propylene), relative to the W0
  constant: the luminous tip of a sootier fuel lies further beyond the flame sheet, because the soot
  needs length to burn out;
- the passage from the jet limit to the coflow limit is spread over the whole range of velocity ratio
  flown, 0.08 per e-fold of $r$, in place of a step between $r$ = 0.02 and 0.5;
- the jet core adds 1.3 nozzle diameters (P2 had 2).

This is carried by `flame_length.median`: `skip` and `b0` hold the linear part, hinge units 2 to 4 hold
$`\ln(1 + 1.3\,d/L_{far})`$ in three pieces.

**Smoke point.** Soot formed in the fuel-rich core must burn out before the tip, cooled by its own
radiation, drops below about 1300 K. With a coflow the residence time is about $L/u_{air}$, so the smoke
point rises with the coflow; a small nozzle shortens the jet's own residence time until, below a
critical diameter, the fuel no longer accumulates its soot time; and in still air radiative quenching
sets a floor (phys_spice_w0.md, sections 1.2 and 1.3). The P2 law, kept unchanged in P3
(phys_spice_study.md, section 2):

```math
L_{sp} = \max\!\left(L_0,\; c\,u^{0.55}\left(\frac{d}{1.6\,\mathrm{mm}}\right)^{-0.33}\Big/x_{fuel}\right)\left(\frac{d_c}{d}\right)^{3}\ \text{for}\ d < d_c
```

with $L_0$ = 21, 28, 14 mm, $c$ = 8.08, 10.3, 2.87 mm at 1 cm/s and $d_c$ = 0.96, 0.80, 0.56 mm
(times $x_{fuel}^{-1.1}$) for ethylene, propane and propylene. `smoke_point_length.median.b0` and `skip`
hold the floor, hinge unit 0 the coflow term above it, hinge unit 1 the nozzle factor.

**Smoke-point flow and soot.** $`\dot m_{sp} = L_{sp}\,Z_{st}/(K F)`$, and
$`\mathrm{logit}\,P(\text{soot}) = 6.1(\ln L - \ln L_{sp})`$ (slope $1.7/0.28$). These two targets are
the W0 lines (phys_spice_w0.md, sections 3 and 7.4) and were not revised.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section spice.wsp.

## 3. Authoring

- **Who.** Author `claude-fable-5-2` (`release.json`, `construction.author`). The record reads
  "runs/dev/gen7/study/spice_p3.wsp verbatim as a revision of spice p2-fable ... Transcribed by the
  assembly engineer (claude-opus-5-5); no number chosen" (`configs/models/law/spice/p3-fable/meta.json`).
- **When and which step.** Seventh generation, 15 November 2026; PROTOCOL 13.6, "A proper study of the
  development data": the author studies the development cases one by one, writes down the expected
  effect of a candidate before it is run once with the logged what-if tool, and keeps it only by the
  by-day rule (SpaceApps-Ardealu:docs/wsp/PROTOCOL.md). P3 was the third of at most five logged
  candidates on `spice`.
- **What was read.** Exposure (`release.json`, `construction.exposure`): datasets
  `spice-train-and-validation-cases`, partitions `train` and `validation`; development feedback
  `PROTOCOL-13.2-13.3-13.4-development-scores` and the what-if run of P3; sources
  `docs/wsp/research/phys_spice_study.md` (sha256
  `33f9808be1fe798355b9d5df29dd0695b63467050240dfe6f8ee92473fd4bb05`),
  `runs/dev/gen7/study/spice_p3.wsp` (sha256
  `4338268390c3d6c3c6fa6c55cf6eb2f839ce871dfcc1dfe2b6ad37e6a4a41b51`), `runs/dev/gen7/whatif_log.jsonl`;
  prior candidate P2. The study states the six test points were not opened. The served note adds that
  P3 was "built after the published fit was read (not blind)": the W0 dossier read Dotson et al. 2011,
  the investigators' fit of these smoke points.
- **Brief.** The physics brief and the author prompts are collected in
  [docs/research/PROMPTS.md](../../research/PROMPTS.md). The author worked from
  SpaceApps-Ardealu:docs/wsp/research/phys_spice_w0.md and wrote
  SpaceApps-Ardealu:docs/wsp/research/phys_spice_study.md (P3 is its section 5).
- **Decisions.** After P2 became the model it was still 0.07 to 0.10 short on flame length and just
  behind the Lin and Faeth baseline (1.94 against 1.84). Reading the 49 development flames against that
  baseline, the author found three things P2 had wrong: the fuel (propylene about 1.2 times longer than
  ethylene at the same velocity ratio), the velocity ratio (acting slowly across the whole range, not
  as a step) and the offset (nearer 1.3 nozzle diameters than 2). P3 writes 35 elements of
  `flame_length`, all different from P2 (`b0`, `skip`, hinge rows 2 to 4 and their knots, `w2[0]` and
  `w2[1]` set to 0 to switch off the old velocity-ratio step, and the spread 0.14 to 0.10;
  `spice_p3_result.md`, element check); every other number is P2's. Predicted before the
  run: T2 from 1.94 to 1.3 to 1.6; realised 1.25 (phys_spice_study.md, 5.2).

## 4. Weights

Every number of P3 is in `configs/models/law/spice/p3-fable/edits.wsp` (changed lines), on top of
`configs/models/law/spice/p2-fable/edits.wsp` and `configs/models/law/spice/w0-fable/edits.wsp`; the
full list is printed by
`ignis wsp inspect --release sha256:000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0 --edits`.
Feature columns are `(z_d, z_u, z_m, z_x, f_propane, f_propylene, f_methane)`, with
`z_d = ln(d / 1 mm)`, `z_u = ln(u / 25 cm/s)`, `z_m = ln(mdot / 1 mg/s)`, `z_x = ln x_fuel`.

| Group | Shape (nonzero) | Values | Physical meaning | Written by |
|---|---|---|---|---|
| `x.nozzle_diameter.*` | 4 scalars (4) | centre 0.001, scale 1.0, lo 0.0003, hi 0.004 | nozzle 0.3 to 4 mm | W0 |
| `x.coflow_speed.*` | 4 (4) | 0.25, 1.0, 0.03, 0.9 | coflow 3 to 90 cm/s | W0 |
| `x.fuel_mass_flow.*` | 4 (4) | 1e-6, 1.0, 5e-8, 2e-5 | 0.05 to 20 mg/s | W0 |
| `x.x_fuel.*` | 4 (4) | 1.0, 1.0, 0.4, 1.0 | fuel fraction 0.4 to 1 | W0 |
| `x.is_propane.*`, `x.is_propylene.*`, `x.is_methane.*` | 12 (6) | centre 0, scale 1, lo 0, hi 1 | fuel indicators | W0 |
| `flame_length.median.b0` | [] (1) | -3.797 | ln length (m) of 1 mg/s of ethylene at the centre, with $c_f$ and $(r/0.3)^{-0.08}$ | P3 |
| `flame_length.median.skip` | [7] (7) | [-0.16, -0.08, 1.08, 0.858, 0.099, 0.173, 0.188] | $\dot m$ and $Z_{st}$ terms plus $-0.08\ln(r/0.3)$ and the fuel factors $c_f$ | P3 |
| `flame_length.median.W1` rows 2-4 | [8, 7] (21 of 35) | each [1.16, 0.08, -1.08, -0.858, -0.099, -0.173, -0.188] | $`\ln(1.3\,d/L_{far})`$ | P3 |
| `flame_length.median.b1[2:5]`, `w2[2:5]` | (3), (3) | [-0.648, -2.848, -5.048]; [0.31, 0.38, 0.31] | $\ln(1 + e^t)$ in three pieces; $`-2.848 = \ln(1.3\,\mathrm{mm}) + 3.797`$ | P3 (b1), P2 (w2) |
| `flame_length.median.W1` rows 0-1, `b1[0:2]` | (14), (2) | rows [-2.0, -1.0, 1.0, -0.27, -0.452, -0.405, 0.559]; b1 [0.784, -2.435] | W0's velocity-ratio step, inert since `w2[0]` = `w2[1]` = 0 | W0 (P3 zeroed w2) |
| `flame_length.spread.b0` | [] (1) | 0.10 | ln-sd of the length (other spread tensors zero) | P3 |
| `flame_length.tail_weight`, `tail_width` | 2 (2) | 0.1, 3.0 | 10 % of cases three times wider | W0 |
| `smoke_point_length.median.b0` | [] (1) | -3.863 | $\ln(0.021\ \mathrm{m})$, the still-air floor of ethylene | W0 (P2 rewrote the same value) |
| `smoke_point_length.median.skip` | [7] (3) | [0, 0, 0, 0, 0.288, -0.405, 2.0] | floors of propane (28 mm), propylene (14 mm), methane, relative to ethylene | P2 |
| `smoke_point_length.median.W1`, `b1`, `w2` | row 0 [-0.33, 0.55, 0, -1.0, -0.045, -0.630, 0], b1 0.970, w2 1.0; row 1 [-1.0, 0, 0, -1.1, -0.182, -0.539, 0], b1 -0.041, w2 3.0 | (9), (2), (2) | unit 0: the coflow term $`c\,u^{0.55}d^{-0.33}/x`$ above the floor; unit 1: the nozzle factor $(d_c/d)^3$ | P2 |
| `smoke_point_length.spread.b0` | [] (1) | 0.18 | ln-sd of the smoke point | P2 |
| `smoke_point_length.spread.W1`, `b1` | (8), (2) | rows 0-1 [-0.91, 0.414, 0, 0, -0.423, -0.224, 0]; b1 [1.342, 0.842] | W0's spread hinges, inert (`w2` = 0) | W0 |
| `smoke_point_length.tail_weight`, `tail_width` | 2 (2) | 0.1, 3.0 | | W0 |
| `smoke_point_fuel_flow.median.b0` | [] (1) | -13.784 | $-3.863 + \ln(0.0637) - \ln(1296)$ | W0 |
| `smoke_point_fuel_flow.median.skip` | [7] (4) | [0, 0, 0, -1.48, 0.232, -1.030, 1.857] | dilution of $L_{sp}$ and $Z_{st}$; $Z_{st}$ of propane and methane | W0 |
| `smoke_point_fuel_flow.median.W1`, `b1`, `w2` | (17), (4), (4) | b1 [1.342, -0.288, 2.158, -1.537]; w2 [1.0, 0.65, -0.1289, 0.1289] | coflow term with the published exponents (-0.91, 0.414), dilution hinge, jet factor $F$ at the smoke point | W0 |
| `smoke_point_fuel_flow.spread.b0`, tails | 3 (3) | 0.30; 0.1, 3.0 | ln-sd 0.30 | W0 |
| `emits_soot.logit.b0`, `skip` | (1), (5) | -0.195; [0, 0, 6.1, 9.03, -1.415, 6.283, -11.33] | $6.1(\ln L - \ln L_{sp})$ at the centre | W0 |
| `emits_soot.logit.W1`, `b1`, `w2` | (19), (4), (4) | b1 [0.784, -2.435, 1.342, -0.288]; w2 [0.903, -0.903, -6.1, -3.965] | the $F(r)$ step of $L$ and the coflow and dilution terms of $L_{sp}$, times 6.1 | W0 |
| `num.substeps` | constant | 512 | integration steps (numerical; unused by these targets) | spec |

Totals (`release.json`, `architecture.parameter_counts`): 594 authored elements, 172 nonzero,
1 constant.

## 5. Forward and assembly

| Field | Value |
|---|---|
| Forward | `crates/ignis-wsp/src/law.rs`, entry point `ignis_wsp::law::forecast`, version 1, profile `strict_no_fitting` |
| `forward_hash` | `sha256:22429520f846e8e83adfb268412312fff1d904e6d6ebaac7f661349336e967f4` (the slug's declaration is one of its forward sources) |
| `code_version` | `ignis-wsp@0.1.0+fwd.22429520f846e8e8` |
| Spec | `law-spice-v1`, `sha256:0dce43b67260ae6d91fd96c7156db8f98fe83167cb4630f8af0f294564143a92` |
| Patch | `law-spice-v1-p3-fable`, `sha256:cc853bc047030f62b4016305a39ab848901af4113497127b81c6df210b6c0998` |
| `weights_hash` | `sha256:a8a7d9cc0a56364770feeca14aeabc5791b1053aace3fa2dcf10101469a4a548` |
| Binding | cohort `atlas:PSI-107:catalogue`, split `sha256:376e677fb08b2bcb02b70400f059fcb8ce314e8ac566f754af65b02aa5a0e212`, source manifest `sha256:6b2489c3e226f394afd4b213a4665460a4ab3f514c36de34ce818b29e97af535` (`configs/models/law/spice/binding.json`) |
| Candidate directory | `configs/models/law/spice/p3-fable` (`edits.wsp`, `meta.json`, `patch.json`, `release.json`) |
| Served record | `configs/models/served/models/000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0/` |
| Checks | `spec_valid`, `patch_valid`, `tensors_complete`, `strict_forward_audit`, `finite_forward_on_fixture`: passed |

Rebuild (the `--root` registry must hold the parent P2 as
`models/f1fe69f0956eddbbc4333187b1668d0e3b8e6ddc8af649872408e99581e3258a/`; its candidate directory is
`configs/models/law/spice/p2-fable`):

```
cargo build --release -p ignis-cli
ignis wsp compile  --family spice --edits configs/models/law/spice/p3-fable/edits.wsp --meta configs/models/law/spice/p3-fable/meta.json --out configs/models/law/spice/p3-fable/patch.json
ignis wsp assemble --family spice --patch configs/models/law/spice/p3-fable/patch.json --binding configs/models/law/spice/binding.json --root <registry holding models/f1fe69f0…258a/> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the record; `served_ids` pins the 14
served ids. `cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at
ignis-web a58c7c3: the served id is the content hash of its record and loads through the registry
checks.

## 6. Evaluation

Metric: quantile score times two, in mm, mean of day means (lower is better). T1 is the smoke-point
length; T2 the flame length given the flow at the smoke point.

**Headline: test days.** The final run (PROTOCOL 13.7, run once on the frozen list sha256
`f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`; 2 days, 6 smoke points; the author
had not seen them) (SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md, section PSI-107):

| Release | Role | T1 smoke-point length, mm | T2 flame length given the flow, mm |
|---|---|---|---|
| **spice P3** | model | **4.1757** | **1.1804** |
| spice P2 | comparator | 4.1757 | 2.4458 |
| spice W0 (variant B) | comparator (zero-shot parent) | 6.9358 | 2.9823 |
| spice lit (published fit; not blind) | comparator | 3.1598 | 2.9823 |
| B0 pooled climatology (simple) | baseline | 10.5782 | 10.5782 |
| B1 per-fuel climatology | baseline | 10.3261 | 10.3261 |
| Lin and Faeth coflow law, train sigma | baseline (T2 only) | - | 2.5627 |

P3 holds on both targets by the rule registered before the run (lower than every baseline), and the
improvement over the zero-shot parent holds (6.94 to 4.18; 2.98 to 1.18) (PROTOCOL 13.7, "Outcome of
13.7"; COVERAGE_19.md). On T2 it is less than half the Lin and Faeth law. On T1 the candidate built on the
investigators' published fit of these same smoke points scores lower (3.16); it is not blind and is
reported beside.

**Development, in-sample** (train and validation days together, after both had been read;
SpaceApps-Ardealu:docs/wsp/feedback/spice_p3_result.md; 9 days):

| Target | Partition | spice P2 | **spice P3** | W0 (variant B) | lit | B0 | B1 | Lin and Faeth |
|---|---|---|---|---|---|---|---|---|
| T2 | pooled | 1.9370 | **1.2519** | 3.0715 | 3.0715 | 10.1567 | 6.6258 | 1.8416 |
| T2 | train | 2.0864 | 1.3723 | 3.2254 | 3.2254 | 11.3160 | 7.0783 | 2.0076 |
| T2 | validation | 1.4139 | 0.8305 | 2.5329 | 2.5329 | 6.0990 | 5.0419 | 1.2606 |
| T1 | pooled | 2.1292 | **2.1292** | 6.5136 | 3.1075 | 10.1567 | 6.6258 | - |

P3 is lower than P2 on T2 on 8 of 9 days (higher on day 119, 0.704 to 0.820) and below the Lin and Faeth
baseline pooled and on both partitions; T1 is identical to P2. P2 itself had been lower than W0 on
both targets on 9 of 9 days (`spice_p2_result.md`).

**Blind look of the zero-shot parent** (validation days, 7 smoke points, PROTOCOL 13.2, before any
validation case was read; SpaceApps-Ardealu:docs/wsp/feedback/spice_w0_result.md): W0 variant B T1
5.8621 against 6.0990 (B0) and 5.0419 (B1); T2 2.5329 against 1.2606 (Lin and Faeth). P2 and P3 have no
blind validation number: they were typed after the validation days became study data.

## 7. Adoption decision and lineage

**Why this release is served.** By PROTOCOL 13.6 a candidate is kept if it is lower than the current
model pooled and on at least two thirds of the days whose forecasts change, for the target that loses,
and not worse by more than 5 % on the others. P3 scored 1.2519 against P2's 1.9370 on T2, lower on 8 of
9 days, with T1 unchanged: "kept. P3 replaces P2 as the spice model for the final run"
(`spice_p3_result.md`; PROTOCOL 13.6, "further outcomes"). It was the SPICE model of the frozen final
list, held on both targets, and is listed among what is served after the run (PROTOCOL 13.7).

**Lineage.**

| Step | Release | Candidate | What it did |
|---|---|---|---|
| root | `sha256:d990bc1e17cbd73dbf050ffe378d3c310e14bc06ddee21165c5523e71a3d1a92` | `configs/models/law/spice/root` | neutral root of slug `spice` (no claim) |
| lit (comparator) | `sha256:40c345b0453da40b38e85bd7c7441e9dc1e272ce097cf204f7bb3f4c3e35d3bf` | `spice/lit-fable` | main W0 law, section 7.4 of `phys_spice_w0.md`; its smoke-point length is the published fit of the same smoke points (not blind for T1) |
| W0 (variant B) | `sha256:93e750242725a5a1fb9d38f4178b23605cac636f210bf9f3f8bc8e8150e2f2a8` | `spice/w0-fable` | the same block with the eight smoke-point lines of section 5.1, which take no number from the paper; the registered zero-shot model |
| P2 | `sha256:f1fe69f0956eddbbc4333187b1668d0e3b8e6ddc8af649872408e99581e3258a` | `spice/p2-fable` | PROTOCOL 13.6: a new smoke-point law (floor, coflow term $u^{0.55}$, nozzle factor below $d_c$) and the flame length plus 2 nozzle diameters; 55 elements written |
| **P3 (served)** | `sha256:000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0` | `spice/p3-fable` | flame length: fuel-dependent luminous tip, gradual passage from jet to coflow, offset 1.3 d |

P1 (the smoke-point length alone) was run as a logged candidate and folded into P2; it was not
assembled (phys_spice_study.md, section 2).

**Scope.**

- Six test points on two days.
- Targets served: `smoke_point_length`, `smoke_point_fuel_flow`, `flame_length`, `emits_soot`; one
  per call, chosen with `settings.target` (default the first).
- Only T1 and T2 are scored. `smoke_point_fuel_flow` and `emits_soot` are W0's lines, which use the
  published coflow and nozzle exponents (-0.91, 0.414) and not P2's smoke-point law (the W0
  record: "smoke_point_fuel_flow, flame_length and emits_soot are those of the main law",
  `configs/models/law/spice/w0-fable/meta.json`); they have no score.
- The 0.41 mm nozzle (four development points with no order in the coflow) gets a level, not a trend;
  propane rests on three points on one nozzle (phys_spice_study.md, section 4).
- Air at 1 atm; cabin oxygen is not an input. Above about 115 mm the flame meets the outlet plate, so
  a longer forecast smoke point means none observable (phys_spice_w0.md, section 3).

## 8. How to call it in IGNIS

- **Forecast lab** (web, route `#/lab`): the Models screen `#/models` lists the served releases and
  opens the model card; `web/src/views/LabModels.tsx` runs a typed scenario (stored runs are refused for
  this release).
- **Model card**: `GET /api/v1/model-cards` (fields from `configs/models/served/served.json`, including
  the cite string).
- **MCP tools**: `models_inspect`, `models_forecast`.
- **API.** Admit a typed scenario, then forecast it:

`POST /api/v1/models/sha256:000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0/scenarios`

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0",
  "cutoff_s": 1.0,
  "phase_established_at_s": 0.0,
  "conditions": {
    "nozzle_diameter": 0.001, "coflow_speed": 0.25, "fuel_mass_flow": 0.000001,
    "x_fuel": 1.0, "is_propane": 0.0, "is_propylene": 0.0, "is_methane": 0.0
  },
  "controls": [],
  "observations": []
}
```

These are typed values (the law's centre from `edits.wsp`: pure ethylene, 1 mm nozzle, 25 cm/s coflow,
1 mg/s), not a measurement. The response carries `input_hash`. Then:

`POST /api/v1/analyses`

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0",
  "input": {"kind": "prefix", "input_hash": "<input_hash>"},
  "settings": {"target": "smoke_point_length", "quantile_levels": [0.05, 0.25, 0.5, 0.75, 0.95]}
}
```

It returns a job: poll `GET /api/v1/jobs/<job_id>` and read `GET /api/v1/artifacts/<artifact_id>`, an
`AnalysisArtifact` with evidence class `model_output`. `settings.target` takes `smoke_point_length`,
`smoke_point_fuel_flow`, `flame_length` or `emits_soot`; none needs `times_s`. The smoke-point targets
give `fuel_mass_flow` no weight, but it must still be typed inside its domain. For the flame length at
the smoke point (T2), type the smoke-point flow as `fuel_mass_flow` and ask for `flame_length`.

Cite: `spice.wsp P3, hand-written (literal WSP) release sha256:000b88b8…98a0, experimental, not trained; its output is model output, not a measurement.`

## Sources

- `configs/models/served/served.json`; `configs/models/served/models/000b88b8232a4c6b7ec04e403b192e2aedc6488aa6e11b3b4be6bb6ae0b398a0/release.json` and `params.npz`
- `configs/models/law/spice/{p3-fable,p2-fable,w0-fable}/{edits.wsp,meta.json,release.json}`; `configs/models/law/spice/{root,lit-fable}/release.json`; `configs/models/law/spice/binding.json`
- `crates/ignis-wsp/src/law.rs`, `crates/ignis-wsp/src/law_slugs.rs`, `crates/ignis-server/src/v1/wsp.rs`, `crates/ignis-core/src/contracts/research.rs` (MCP tool descriptions), `tools/wsp/verify_served.py`
- SpaceApps-Ardealu:docs/wsp/research/law_forward.md, law_forward_targets.md (lane branch feat/handwritten-science, 542dad8)
- SpaceApps-Ardealu:docs/wsp/research/phys_spice_w0.md, phys_spice_study.md
- SpaceApps-Ardealu:docs/wsp/feedback/spice_w0_result.md, spice_p2_result.md, spice_p3_result.md
- SpaceApps-Ardealu:docs/wsp/PROTOCOL.md (sections 13.6, 13.7), RESULTS_FINAL_GEN7.md, COVERAGE_19.md
