# flex_methanol.wsp2 — methanol droplet burning forecast

`flex_methanol.wsp2` forecasts the liquid diameter of a free-floating methanol droplet burning in
microgravity (NASA FLEX, PSI-69) after a cutoff, as quantiles. It works from the chamber atmosphere and the
diameters measured before the cutoff, and it follows the droplet through a hot flame, the loss of that flame
(by water uptake or by radiation) and evaporation without a flame. On the FLEX methanol validation partition
its CRPS is **40.0 µm**. The best fitted model scores 77.7 µm, the zero-shot `flex.wsp` W0 50.4 µm and the
D²-law of its own spec 102.1 µm. On the sealed test partition, which no author read, it scores **18.5 µm**
against 31.6 for the D²-law (`docs/wsp/RESULTS.md`; `SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL.md`). It is
data-informed: the author read the methanol train partition before typing it. Release
`sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce`.

| | |
|---|---|
| Model | `flex_methanol.wsp2` |
| Investigation | FLEX (PSI-69), methanol droplets |
| Family | `flex` (assemble key `flex_methanol2`) |
| Candidate | D3 (`configs/models/flex/wsp2/methanol/d3-fable`) |
| Generation | third. The served record dates it 13 November; the release was created 2026-11-13T23:09:20Z; PROTOCOL section 8 heads the generation "14 November" |
| Author | `claude-fable-5-2` (Anthropic). Every number was typed by the author; there was no transcriber |
| Authorship | data-informed (train) |
| Release | `sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce` |
| Forward | `ignis_wsp::droplet3::forecast`, `crates/ignis-wsp/src/droplet3.rs` (version 3, profile `strict_no_fitting`) |
| Parent release | `sha256:abf919b6199864c0a0eb6d27beffb295421c9d080f664d7c9a5929d6f254247b` (the D²-law baseline of the spec, the lineage root) |

State `experimental`, `trained_on = not_trained`, uncertainty kind `assumed_noise`
(`configs/models/served/models/55e0cb12…/release.json`).

## 1. Task and targets

**Inputs** (served entry in `configs/models/served/served.json`):

| Name | Kind | SI unit | Note |
|---|---|---|---|
| `fuel` | condition (text) | 1 | fuel species; this release answers methanol only (example `Methanol`) |
| `pressure` | condition | Pa | chamber pressure (example 101325.0) |
| `x_o2` | condition | 1 | oxygen mole fraction (example 0.21) |
| `x_co2` | condition | 1 | carbon dioxide mole fraction (example 0.0) |
| `x_he` | condition | 1 | helium mole fraction (example 0.0) |
| `phase_established_at_s` | phase start | s | time the burning droplet was established; the release reads diameters from it (example 0.0) |
| `liquid_diameter` | observation | m | droplet diameters measured up to the cutoff, the early prefix the forward reads |

The forward turns these into features (`configs/models/flex/wsp2/methanol/spec.json`):
$c_{O2} = (x_{O2} - 0.21)/0.05$, $c_{CO2} = x_{CO2}/0.20$, $c_{He} = x_{He}/0.20$,
$c_{\log p} = \ln(p / 10^5\,\mathrm{Pa})$ and $c_{\log D0} = \ln(D_0 / 2\,\mathrm{mm})$. Here $D_0$ comes from
the head window of the prefix, carried back to $t = 0$. It also uses burn progress $w = 1 - D^2/D_0^2$ and the
diffusion clock $s = t \cdot 1\,\mathrm{mm^2/s} / D_0^2$. The prefix adds the encoder features `e_late`,
`e_early`, `r_late`, `r_end`, `g_span`, `g_count` and `g_progress`.

**Target.** One quantity, `liquid_diameter` (m): the measured liquid diameter at times after the cutoff, as
predictive quantiles. These are an unconditional distribution over the droplet's state (hot flame or no flame).
The forward also reports the state probabilities and the cumulative incidence of the end of the hot-flame
phase as competing-risk rows (`active_supported`, `hot_phase_end`, `disruption`, `unresolved`; `droplet3.rs`).
The spec excludes the visible or UV flame diameter, flame temperature, soot and emission, fibre-supported
droplets, other fuels, cool-flame burning (not observed for methanol), and any claim of a calibrated
posterior.

**Time origin.** The NASA DropletData processed time base (phase entry not adjudicated). The model horizon is
45 s from that origin (`num.t_valid_s`), and the model stops at a diameter of 0.25 mm (`num.d_min_m`).

**Stored run or typed scenario.** A stored FLEX run gives everything: the conditions and the diameters measured
up to the cutoff. A typed scenario must give the conditions and a few diameters before the cutoff. At least two
valid diameters are needed (`enc.min_points`). Declared domain of the law (`applicability`, in feature units):
$c_{O2} \in [-2.2, 4.8]$, $c_{CO2} \in [0, 4]$, $c_{He} \in [0, 3]$, $c_{\log p} \in [-1, 1.5]$,
$c_{\log D0} \in [-0.9, 1.26]$. In the inputs' own units, that is 10 to 45 % oxygen, up to 80 % CO2 and up to
60 % helium. It is the declared physical domain of the hand-written law, not a training-support region.

## 2. Physical mechanism

The model rests on the classical quasi-steady droplet flame: in a hot flame $D^2$ falls linearly in time at the
burning-rate constant $K$. Methanol adds two ways for the flame to go out. A droplet smaller than the critical
diameter of its atmosphere absorbs water from its own flame until the liquid can no longer sustain it, at
about half its initial diameter. That ending depends on burn progress. A larger droplet loses its flame to
radiation once the flame has grown, a fixed fraction of a diffusion time after ignition. That ending depends
on the clock. After either ending the droplet keeps evaporating on the heat the flame left behind
(`droplet3.rs` header; mechanism and literature in `SpaceApps-Ardealu:docs/wsp/research/phys_flex_methanol.md`
sections 1.1, 1.4, 1.5 and 1.6).

Every law is a small hinge network typed by hand:

$$N(x) = b_0 + \mathrm{skip}\cdot x + \sum_j w_2[j]\,\mathrm{relu}\big(b_1[j] + W_1[j,:]\cdot x\big)$$

- **Hot flame** (`rate.*`): $\dfrac{dD^2}{dt} = -K_h$, with $\ln(K_h / 1\,\mathrm{mm^2/s}) = N_{rate}(c, w, s) + b + s_z z$.
  The $b$ term is the run's own rate offset read from the prefix. The $s_z z$ term is a scenario node
  (`num.z_*`), with $s_z = \sqrt{s_{floor}^2 + (1 - \mathrm{gain})\,s_{pop}^2}$ (`unc.*`, `droplet3.rs`).
- **Critical diameter and class** (`crit.*`, `class.*`): $\ln(D_{crit}/2\,\mathrm{mm}) = N_{crit}(c_{O2}, c_{CO2}, c_{He}, c_{\log p})$.
  The supercriticality is $x_{exc} = \ln(D_0/D_{crit})$, and $P(\text{radiative}) = \sigma(\kappa\,x_{exc} + \text{bias})$.
- **Diffusive extinction** (`diff.*`), the water limit. The flame goes out at progress $W$ with the Gompertz
  hazard $\Lambda_D(w) = \ln 2\,\big(e^{a_D (w - w_D)} - e^{-a_D w_D}\big)$, where $w_D = \sigma(N_{diff}(c, x_{exc}))$
  and $a_D$ = `diff.sharp`. The physics dossier puts the liquid at about 30 % water when $D/D_0 \approx 0.47$–$0.51$,
  which is where flame-out is measured (`phys_flex_methanol.md` 1.4).
- **Radiative extinction** (`rad.*`): $\Lambda_R(s) = \ln 2\,\big(e^{a_R (s/s_R - 1)} - e^{-a_R}\big)$, with
  $s_R = \exp(N_{rad}(c, x_{exc}))$ and $a_R$ = `rad.sharp`. The radiative-loss criterion is
  $\chi > \chi_c = (T_{f0} - T_{ext})/(T_{f0} - T_\infty)$ (`phys_flex_methanol.md` 1.5).
- **No flame** (`post.*`). Over a step $\Delta t$, $D^2$ falls by $k_{inf} r^m \Delta t + k_{hot} r^h \tau_e (1 - e^{-\Delta t/\tau_e})$,
  where $\tau_e = \tau_0 r^n$ and $r = (D_e / 2\,\mathrm{mm})^2$. In D3's edits file the rate form reads
  $d(D^2)/dt = -[k_{inf} r^m + k_{hot} r^h e^{-t/\tau_e}]$.
- **Prefix encoder and state filter** (`enc.*`, `filt.*`, `gate.*`). The level at the cutoff is the mean $D^2$ of
  the last end window, carried to the cutoff along the end rate. The run offset is
  $b = \mathrm{gain}\cdot$(clamped mix of the mid-to-tail and head-to-mid log-rate innovations), with
  $\mathrm{gain} = \sigma(\text{enc.gain\_b} + \text{enc.gain\_w}\cdot(g_{span}, g_{count}, g_{progress}))$. The state
  probabilities at the cutoff are the laws' prior multiplied by $(N(r_{end}; \mu_{state}, sd_{state}) + l_{floor})^g$.
  A physical validity gate first drops diameters that later frames contradict, because a droplet does not grow
  (spec `notes`).
- **Disruption** (`haz.log_disruption`): a constant hazard $\lambda = e^{-4.83}$ per second for a burning droplet.
  It is reported as an event and does not change the diameter.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section flex_methanol.wsp2.

## 3. Authoring

**Who.** The author model `claude-fable-5-2` typed every number in
`configs/models/flex/wsp2/methanol/d3-fable/edits.wsp` (59 statements). There was no fit and no search, and
no transcriber: the edits header states that "every number below was typed by the author". A delegated
analysis agent (`claude-opus-5-5`) audited D2 on train only. It wrote no weight, and the author rechecked the
claims he acted on against the train tables (`meta.json`; `SpaceApps-Ardealu:docs/wsp/dossier/04_decisions.md`
D-04).

**When and under which step.** Third generation, PROTOCOL section 8 (audits, a frozen forward per generation,
the selection rule; `docs/wsp/PROTOCOL.md`). The candidate was committed (`d3e00f4`) before any validation
score of it existed.

**What was read** (exposure in `meta.json` and `release.json`):
- the methanol **train** partition of split `flex-660b7d9ede629b30`: conditions, series, test sheets and narratives;
- the per-case train feedback of D1 and D2 (`runs/dev/fb_train_flex.json`);
- the train-only audit of D2 (`SpaceApps-Ardealu:docs/wsp/feedback/flex_methanol_d2_train_audit.md`);
- the **aggregate validation tables** of D1 and D2 (CRPS and coverage only), with no per-run validation outcome or score. The edits header says nothing was chosen from them;
- the outcomes of the n-heptane train partition (split `flex-c43bbe3580ffce2a`), where the helium coefficient of the radiative clock agrees across the two fuels;
- the published FLEX literature (`docs/wsp/research/exp_flex.md`) and the author's pretraining knowledge, which cannot be checked.

`development_feedback` is empty. Prior candidates seen: the wsp2 baseline `abf919b6…`, D1 `f1f72389…` and
D2 `b2f6930f…`.

**Brief.** The prompts of the campaign are collected in [docs/research/PROMPTS.md](../../research/PROMPTS.md).
The author worked from the D2 audit and the train tables, not from a physics dossier. The methanol physics
dossier (`SpaceApps-Ardealu:docs/wsp/research/phys_flex_methanol.md`) was written later, on 14 November for
the fifth generation, and audits this release in its section 3.

**Decisions** (D3 against D2; `meta.json` rationale, edits comments marked `[D3]`,
`docs/wsp/cards/flex_droplet3.md`):

| Law | D2 | D3 | Reason on train |
|---|---|---|---|
| Radiative clock $\ln s_R$ | 0.90 at $D_{crit}$, −1.2 per unit of $x_{exc}$, with a cliff beyond 0.45 | 1.0 at $D_{crit}$, −3.8 per unit from $x_{exc}$ = 0.05, no second term; −0.22 per 20 % He | 13 radiative runs in 9 sessions lie on one line; four same-atmosphere pairs; three helium runs. D2's late radiative extinctions were 30 % of its non-artefact train error |
| No flame | fixed amplitude, decay time growing like $D_e^2$ | $k_{hot}$ 0.13 mm²/s · $r^{0.45}$, decay 2.8 s · $r^{0.2}$ | 20 runs in 15 sessions, two size classes, four time windows: the unlit droplet sheds the flame's heat about twice as fast in the first two seconds |
| Heat-up | one deficit in all atmospheres | deeper and longer below 16 % O2 (unit 4); plateau 5 % lower at 15 % O2 and below (unit 1, 0.21 → 0.14) | 4 of 5 and 3 of 5 low-oxygen train sessions |
| Run-offset gain at 1 / 3 / 5 s | 0.12 / 0.92 / 0.98 | about 0.08 / 0.36 / 0.55 | only 0.2–0.3 of a prefix's offset carries on at a 3 s cutoff, and 0.5–0.6 at 5 s |
| State at the cutoff | from the mid-to-tail rate | from the end rate of the prefix (forward change) | a droplet that went out one or two seconds before the cutoff was read as burning in five of seven such train cases |
| Rate spread before / after a prefix | 0.18 / 0.08 | 0.10 / 0.06 | train run-level sd 0.06, with a margin for unseen sessions; not chosen from validation |

Unchanged from D2: the rate level and its oxygen, CO2, helium, pressure, size and water-uptake terms, the
critical diameter, the class and the diffusive extinction. The audit found a median rate residual of −0.003
over 263 windows of 58 train runs, no run on the wrong side of $D_{crit}$, and a diffusive progress of 0.81
observed against 0.79.

## 4. Weights

All values come from the release's `params.npz` (data pack) and `edits.wsp`; the meanings come from the edits
comments and `droplet3.rs`. The input order of `rate` is $(c_{O2}, c_{CO2}, c_{He}, c_{\log p}, c_{\log D0}, w, s)$,
of `diff` and `rad` it is $(c, x_{exc})$, and of `crit` it is $(c_{O2}, c_{CO2}, c_{He}, c_{\log p})$.

| Group | Shape (elements / nonzero) | Value(s) | Physical meaning |
|---|---|---|---|
| `rate.b0` | [] (1/1) | −0.53 | ln K in air, 1 atm, 2 mm: 0.59 mm²/s on the plateau |
| `rate.skip` | [7] (7/5) | [0.14, −0.04, 0.18, 0.10, 0, −0.35, 0] | faster with O2, He and pressure, slower with CO2; −0.35 over the whole burn as the droplet takes up water |
| `rate.W1` | [16,7] (112/7) | (0,0) −1; (1,0) −1; (2,4) 1; (2,5) −0.65; (3,6) −5.56; (4,0) −1; (4,6) −2 | hinge inputs of units 0–4 |
| `rate.b1` | [16] (16/4) | 1: −0.6; 2: −0.3; 3: 1.0; 4: −0.9 | hinge offsets (unit 0 at 0) |
| `rate.w2` | [16] (16/5) | 0: −0.19; 1: 0.14; 2: −0.5; 3: −0.6; 4: −0.55 | unit 0: steeper fall below air; 1: flattening below 18 % O2; 2: a large droplet burns slower early; 3: heat-up (starts at 0.55 of plateau, reaches it at s = 0.18); 4: slower heat-up below 16 % O2 |
| `crit.b0` | [] (1/1) | 1.51 | $D_{crit}$ about 9 mm in air at 1 atm |
| `crit.skip` | [4] (4/4) | [0.75, −0.16, −0.18, 0.7] | 3.7 mm at 15 % O2, 3.2 at 14 %, 2.35 at 12 % |
| `crit.W1`, `crit.b1`, `crit.w2` | [8,4], [8], [8] (48/0) | all zero | no hinge |
| `class.kappa`, `class.bias` | [] (2/2) | 10.0, 0.7 | soft class boundary; on train it lies between $x_{exc}$ = −0.10 and −0.04 |
| `diff.b0` | [] (1/1) | 1.35 | flame out at D/D0 ≈ 0.46 (w = 0.79) |
| `diff.skip` | [6] (6/2) | [0.12, 0, 0, −0.6, 0, 0] | later in strong atmospheres and at low pressure |
| `diff.W1` | [8,6] (48/2) | (0,0) 1; (1,5) 1 | |
| `diff.b1` | [8] (8/2) | [−1.0, 0.3, 0…] | |
| `diff.w2` | [8] (8/2) | [0.5, −2.0, 0…] | unit 0: later in strong atmospheres (hinge at $c_{O2}$ = 1); unit 1: earlier as D0 nears $D_{crit}$ |
| `diff.sharp` | [] (1/1) | 14.0 | Gompertz sharpness $a_D$ |
| `rad.b0` | [] (1/0) | 0.0 | $s_R$ = 1.0 at the critical diameter |
| `rad.skip` | [6] (6/1) | [0, 0, −0.22, 0, 0, 0] | −0.22 per 20 % He: helium raises gas diffusivity, so transients run faster on the clock |
| `rad.W1`, `rad.b1`, `rad.w2` | [8,6], [8], [8] (64/3) | W1(0,5) 1; b1[0] −0.05; w2[0] −3.8 | $\ln s_R = -3.8\,(x_{exc} - 0.05)$ above the threshold |
| `rad.sharp` | [] (1/1) | 5.0 | Gompertz sharpness $a_R$: quartiles of $s/s_R$ at 0.83 and 1.15 |
| `post.k_inf`, `post.size_exp` | [] (2/2) | 1.5e−8 m²/s, 0.72 | long-time evaporation $k_{inf} r^m$ |
| `post.k_hot`, `post.hot_exp` | [] (2/2) | 1.3e−7 m²/s, 0.45 | heat shed by the unlit droplet, amplitude ∝ $r^{0.45}$ |
| `post.tau0_s`, `post.tau_exp` | [] (2/2) | 2.8 s, 0.2 | decay time $\tau_0 r^{0.2}$ |
| `enc.gain_w`, `enc.gain_b` | [3], [] (4/2) | [1.4, 0, 0], −1.6 | run-offset gain from the prefix span: sigmoid(−1.6 + 1.4 ln span) |
| `enc.mix` | [2] (2/2) | [0.85, 0.15] | weights of late and early innovations |
| `enc.e_clip` | [2] (2/2) | [0.6, 0.8] | clamp of innovations below / above the law |
| `enc.cool_gain` | [] (1/0) | 0.0 | methanol has no cool flame |
| `filt.sd_hot`, `filt.sd_evap`, `filt.l_floor` | [] (3/3) | 0.18, 0.2, 0.03 | state likelihood of the end rate: burning within about 15 % of the law; unlit at 0.1–0.45 of the hot rate |
| `filt.gain_w`, `filt.gain_b` | [3], [] (4/4) | [2.0, 0.5, 2.0], −1.5 | exponent of the evidence, as in D2 (0.12, 0.92, 0.98 at 1, 3, 5 s) |
| `unc.s_pop`, `unc.s_floor` | [] (2/2) | 0.10, 0.06 | rate spread before any prefix, and the part no prefix removes |
| `unc.sigma_d`, `unc.sigma_frame` | [] (2/2) | 2.0e−5 m, 6.0e−6 m | measurement sd; frame-to-frame sd |
| `haz.log_disruption` | [] (1/1) | −4.83 | ln of the disruption hazard (1/s) |
| constants `norm.*` | 4 | d_ref 0.002 m, p_ref 1e5 Pa, k_ref 1e−6 m²/s, n_ref 32 | references (spec, fixed) |
| constants `enc.*` | 5 | window_frac 0.25, end_frac 0.15, k_floor 2e−8 m²/s, w_ref 0.08, min_points 2 | prefix windows (spec, fixed) |
| constants `gate.*` | 3 | growth_tol 0.05, min_later 3, k_max 2e−6 m²/s | validity gate (twice the fastest measured rate) |
| constants `num.*` | 8 | dt 0.05 s, bin 0.5 s, d_min 2.5e−4 m, t_valid 45 s, z_step 0.5, z_count 15, ce_step 1, ce_count 5 | numerics |

**Totals** (`release.json`, `architecture.parameters`): 378 authored elements, 68 nonzero, 20 constants.
The full list is printed by `ignis wsp inspect --release sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce --edits`,
or can be read in `configs/models/flex/wsp2/methanol/d3-fable/edits.wsp`.

## 5. Forward and assembly

| | |
|---|---|
| Forward | `crates/ignis-wsp/src/droplet3.rs`, entry `ignis_wsp::droplet3::forecast`, version 3, `strict_no_fitting` |
| forward_hash | `sha256:71ea3173cd8150cde64bcaf5bd069a4d8519584377051c5372b75413f41b950a` |
| code_version | `ignis-wsp@0.1.0+fwd.71ea3173cd8150cd` |
| Spec | `flex_methanol-wsp-v3`, `sha256:b842c136587b0b34a34df3861485a869633989a924bc04e1c4f2ac8c65e94c98` |
| Patch | `flex_methanol-wsp-v3-d3-fable`, `sha256:8f27e3537761fefc661c4face2e2829d3cb044140c03fb40358067ce885d9f89` (`construction_method` literal_revision) |
| weights_hash | `sha256:88f05321b949be75d8916ebe3e530eced4e5c81c73bcde7d238fc8208ce117c2` |
| Binding | cohort `sha256:9e31b1aa6b56b2086aac85776d27778bb1680a83fcdfc543dad62e0385da637e`, split `sha256:660b7d9ede629b3009b4b761ea9e5875560719566e3dea21112f607cddd49894`, source manifest `sha256:74dedf7e…6a382` (`configs/models/flex/wsp2/methanol/binding.json`) |
| Candidate directory | `configs/models/flex/wsp2/methanol/d3-fable` |
| Assembly checks | spec_valid, patch_valid, tensors_complete, strict_forward_audit, finite_forward_on_fixture: all passed |

Rebuild, from the ignis-web root. The parent is assembled first from its own patch into a scratch registry:

```
cargo build --release -p ignis-cli
ignis wsp compile  --family flex_methanol2 --edits configs/models/flex/wsp2/methanol/baseline-d2/edits.wsp --meta configs/models/flex/wsp2/methanol/baseline-d2/meta.json --out configs/models/flex/wsp2/methanol/baseline-d2/patch.json
ignis wsp assemble --family flex_methanol2 --patch configs/models/flex/wsp2/methanol/baseline-d2/patch.json --binding configs/models/flex/wsp2/methanol/binding.json --root <registry>
ignis wsp compile  --family flex_methanol2 --edits configs/models/flex/wsp2/methanol/d3-fable/edits.wsp --meta configs/models/flex/wsp2/methanol/d3-fable/meta.json --out configs/models/flex/wsp2/methanol/d3-fable/patch.json
ignis wsp assemble --family flex_methanol2 --patch configs/models/flex/wsp2/methanol/d3-fable/patch.json --binding configs/models/flex/wsp2/methanol/binding.json --root <registry holding models/abf919b6199864c0a0eb6d27beffb295421c9d080f664d7c9a5929d6f254247b/> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

The first assemble writes the parent `abf919b6…247b` into `<registry>`, where the second one looks for it.
`verify_served.py` rebuilds every served release from its committed patch and compares the id and the record;
`served_ids` pins the 14 served ids. `cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on
15 November 2026 at ignis-web a58c7c3: the served id is the content hash of its record and loads through
the registry checks.

## 6. Evaluation

CRPS of the droplet diameter in µm, lower is better. The log predictive density (lpd) is higher-better.
Cutoffs are 1, 3 and 5 s.

**Headline: the sealed test partition.** This partition holds 65 cases of 22 runs in 9 sessions, 64 of them
scored. It is **blind**: no release of this generation was written or chosen from it, and it was scored once
on 15 November (`SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL.md`, step 1).

| Release | CRPS | MAE of median | Cov. 50 % | Cov. 90 % |
|---|---|---|---|---|
| **D3 (served)** | **18.5** | 22.4 | 0.83 | 0.99 |
| D²-law of the spec | 31.6 | 43.8 | 0.65 | 0.93 |
| `flex.wsp` W0 (zero-shot) | 37.2 | 48.4 | 0.66 | 0.89 |
| `flex.v3` (MAP, fitted) | 42.7 | 65.1 | 0.40 | 0.96 |
| `flex.d2_prefix` (fitted) | 55.7 | 89.7 | 0.49 | 0.98 |
| `flex.v1` (MAP, fitted) | 58.9 | 75.7 | 0.68 | 0.97 |
| `flex.d2_population` (fitted) | 75.4 | 82.9 | 0.07 | 0.18 |

Paired CRPS, D3 minus comparator (65 cases, 9 sessions, 95 % interval): D²-law −13.1 [−19.0, −7.7], W0 −18.7
[−35.2, −5.8], v3 −24.2 [−32.7, −15.9], d2_prefix −37.2 [−45.4, −28.8], v1 −40.4 [−56.2, −26.5],
d2_population −56.9 [−86.5, −30.7]. Every interval lies below zero.

Post-hoc point metrics on the same sealed cases (session-weighted; `SpaceApps-Ardealu:docs/wsp/RESULTS_POINT_METRICS.md`):
MAE of the median 22.4 µm and nRMSE 1.52 %, against 43.8 µm and 2.52 % for the D²-law. The paired MAE
difference is −21.4 µm [−29.4, −12.8].

**Validation partition.** 56 common cases in 9 clusters (`docs/wsp/RESULTS.md`, FLEX methanol). The author had
seen the aggregate validation tables of D1 and D2, but no score of D3 itself. D3 was committed before its own
validation score existed, and that score was read once.

| Release | CRPS | MAE of median | Cov. 50 % | Cov. 90 % | lpd |
|---|---|---|---|---|---|
| **`flex_methanol.wsp2` D3** | **40.0** | 53.7 | 0.71 | 0.90 | 8.60 |
| `flex.wsp` W2 (phase 1, data-informed on train + validation) | 41.7 | 59.8 | 0.56 | 0.93 | 8.37 |
| `flex_methanol.wsp` D2 | 45.3 | 61.2 | 0.63 | 0.86 | 8.32 |
| `flex_methanol.wsp` D1 | 46.3 | 62.0 | 0.53 | 0.80 | 8.29 |
| `flex.wsp` W0 (zero-shot) | 50.4 | 68.7 | 0.58 | 0.85 | 8.23 |
| `flex.wsp` W1 | 56.3 | 77.0 | 0.49 | 0.81 | 8.09 |
| `flex.v3` (MAP, fitted on train) | 77.7 | 108.6 | 0.34 | 0.75 | 7.08 |
| `flex.v1` (MAP, fitted on train) | 93.8 | 124.7 | 0.53 | 0.82 | 7.40 |
| `flex_methanol.wsp2` D²-law | 102.1 | 126.8 | 0.53 | 0.72 | 7.57 |
| `flex.d2_prefix` (fitted) | 103.4 | 145.0 | 0.50 | 0.79 | 6.80 |
| `flex.d2_population` (fitted) | 137.5 | 149.9 | 0.05 | 0.11 | −6.09 |

Paired CRPS, other minus D3: W2 +1.7 [−16.2, +15.5], D2 +5.2 [−1.0, +11.5], W0 +10.4 [−0.2, +21.4],
v3 +37.7 [+14.2, +60.8], D²-law of the spec +62.0 [+26.4, +105.2]. On later forwards the same numbers
("carry of D3", on 59 cases) score 39.5 (`docs/wsp/RESULTS.md`, fourth generation; `docs/wsp/RESULTS_GEN5.md`).

**Train partition** (in-sample: the laws were typed from these runs). 182 cases, 3 refused
(`docs/wsp/RESULTS.md`):

| Release | CRPS mean | median | mean, worst 5 % dropped | MAE of median | Cov. 50 % / 90 % |
|---|---|---|---|---|---|
| **D3** | **29.9** | **17.7** | 19.2 | 37.8 | 0.77 / 0.94 |
| D2 | 35.8 | 23.1 | 25.1 | 46.4 | 0.69 / 0.93 |
| D1 | 34.6 | 20.9 | 23.9 | 46.1 | 0.61 / 0.88 |
| W0 (zero-shot) | 62.4 | 31.6 | 42.6 | 85.6 | 0.55 / 0.85 |
| D²-law of the spec | 87.2 | 37.9 | 59.7 | 112.9 | 0.49 / 0.77 |

D3 is better or level in 24 of 26 train sessions against D2 (`docs/wsp/PROTOCOL.md` section 8). The
post-hoc anatomy of D3 (`SpaceApps-Ardealu:docs/wsp/feedback/flex_methanol_d3_train_anatomy.md`) finds that
artefacts carry 39 % of train CRPS, and that the 0–2 s CRPS is the assumed 20 µm measurement noise.

## 7. Adoption decision and lineage

**Why this release is served.** It passed the droplet selection rule, written down before validation was run:
replace the carried candidate when the median train CRPS over common cases is not worse and the candidate is
better or level (within 1 µm) in at least 60 % of train sessions. D3 had a median of 17.7 against 23.1 µm and
was better or level in 24 of 26 sessions. Validation then acted only as a veto, and D3 improved it from 45.3
to 40.0 µm (`docs/wsp/PROTOCOL.md` section 8; `SpaceApps-Ardealu:docs/wsp/dossier/04_decisions.md` D-14, D-40).
No later candidate has replaced it (`docs/wsp/README.md`).

**Lineage.**
- `flex_methanol.wsp2` D²-law baseline `sha256:abf919b6199864c0a0eb6d27beffb295421c9d080f664d7c9a5929d6f254247b`.
  The reference baseline and lineage root: the classical D²-law at the published median rate 0.46 mm²/s, with
  no extinction and no evaporation (`baseline-d2/edits.wsp`).
- → **D3** `sha256:55e0cb12…43ce`: a literal revision of that root. It carries the laws of D2 with the changes in section 3.
- The laws came from the second generation (`flex_methanol.wsp`, forward `droplet.rs`). There the chain is: D²-law
  `sha256:6c6fd99d756e4d2edb550547d1b5f0767a8878cf3a63d73866fc799dbca16962` → D1
  `sha256:f1f723892493030df0525e3410c7fca42e6709a384b224a69e20d188836b28fb` (first multistate candidate,
  data-informed on train) → D2 `sha256:b2f6930fc58ddcc0565f25e3968ffa405a718379584890850794bf5ec34a4e97`
  (no law changes; the spread was widened after D1's aggregate validation coverage of 0.80). Phase 1 before
  that: `flex.wsp` W0 `sha256:fa8eb18cbb2affa850846fc90b82475fbac13f44d2fbca0fd2321024f4a6e28f` (zero-shot),
  then W1 and W2.

**What it replaced:** `flex_methanol.wsp` D2 (45.3 µm on validation).

**Tried and not adopted** (validation CRPS against the carry of D3 on the same cases):
- fourth generation: `flex_methanol.wsp3` D4 `sha256:4e975d3b…2d02`, 44.8 against 39.5;
- fourth generation: `flex_methanol.wsp3` D5 `sha256:0b57dba7…f2c7`, 40.1 against 39.5 (`docs/wsp/RESULTS.md`; D-15);
- fifth generation: `flex_methanol.wsp5` P1 `sha256:7884e61c…141a`, physics-first laws with flame cameras, 42.5 against 39.5; carry of D3 minus P1 −3.1 µm [−12.1, +4.6] (`docs/wsp/RESULTS_GEN5.md`; D-43).

The Astra candidates of the later forwards are data-informed and experimental. None replaces the carried
release (`SpaceApps-Ardealu:docs/wsp/dossier/06_models.md`).

**Scope** (served `limits`, condensed):
- Tracker artefacts (glare, soot, bubbles, dropouts, frozen values) are scored as data, and no model forecasts them. They are about a third of the train error.
- A flame that went out less than a second before the cutoff still reads as burning.
- The helium term of the clock rests on three runs.
- The radiative clock above the critical diameter rests on CO2-diluted pairs. If the critical diameter in CO2 is off, the slope is wrong between atmospheres.

## 8. How to call it in IGNIS

**Forecast lab** (web, route `#/lab`). The Models screen (`#/models`) lists the releases and opens the model
card. `web/src/views/LabModels.tsx` runs a stored run or a typed scenario.

**Model card:** `GET /api/v1/model-cards`. The fields come from `served.json`, including the cite string and
`card_doc` `docs/wsp/cards/flex_droplet3.md`.

**API.** Stored runs work (`stored_runs: true`). `settings.target` has one value, `liquid_diameter` (the spec
target, which is also the default), so it may be omitted. There are no controls and no combination rule.
Forecast times must stay within 45 s of the time origin.

1. Typed scenario: `POST /api/v1/models/sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce/scenarios`.
   The values below are the served example values, typed and not measurements:

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce",
  "cutoff_s": 3.0,
  "conditions": {"fuel": "Methanol", "pressure": 101325.0, "x_o2": 0.21, "x_co2": 0.0, "x_he": 0.0},
  "controls": [],
  "observations": [
    {"time_s": 0.0, "value_si": 0.003}, {"time_s": 0.5, "value_si": 0.002954},
    {"time_s": 1.0, "value_si": 0.002907}, {"time_s": 1.5, "value_si": 0.002859},
    {"time_s": 2.0, "value_si": 0.002811}, {"time_s": 2.5, "value_si": 0.002762},
    {"time_s": 3.0, "value_si": 0.002711}
  ],
  "phase_established_at_s": 0.0
}
```

   The call returns `input_hash`. `test_date` is optional and is not read by this release.

2. `POST /api/v1/analyses`:

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce",
  "input": {"kind": "prefix", "input_hash": "<input_hash>"},
  "settings": {"target": "liquid_diameter", "times_s": [4, 6, 8, 10, 13], "horizon_s": 10, "quantile_levels": [0.05, 0.25, 0.5, 0.75, 0.95]}
}
```

   For a stored FLEX methanol run, use `"input": {"kind": "stored_run", "run_id": "<run id>", "cutoff_s": 3}`.
   The call returns a job. Poll `GET /api/v1/jobs/<job_id>`, then read `GET /api/v1/artifacts/<artifact_id>`:
   an `AnalysisArtifact` with `evidence_class` `model_output`.

**MCP tools:** `models_inspect` (the card and the release) and `models_forecast` (a stored run).

**Cite:** flex_methanol.wsp2 D3, hand-written (literal WSP) release sha256:55e0cb12…43ce, experimental, not trained; its output is model output, not a measurement.

## Sources

- `configs/models/served/served.json`; `configs/models/served/models/55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce/release.json` (and its `params.npz`, through the data pack)
- `configs/models/flex/wsp2/methanol/d3-fable/{edits.wsp, meta.json, patch.json, release.json}`; `configs/models/flex/wsp2/methanol/baseline-d2/{edits.wsp, release.json}`; `configs/models/flex/wsp2/methanol/{spec.json, binding.json}`
- `configs/models/flex/wsp/methanol/{baseline-d2, d1-fable, d2-fable}/{edits.wsp, release.json}`; `configs/models/flex/wsp/w0-fable/release.json`
- `crates/ignis-wsp/src/droplet3.rs`; `crates/ignis-wsp/src/cli.rs`; `crates/ignis-wsp-serve/src/lib.rs`; `crates/ignis-core/src/contracts/wsp.rs`; `tools/wsp/verify_served.py`; `tools/wsp/reassemble.py`; `web/src/views/LabModels.tsx`
- `docs/wsp/README.md`; `docs/wsp/RESULTS.md`; `docs/wsp/RESULTS_GEN5.md`; `docs/wsp/PROTOCOL.md` section 8; `docs/wsp/cards/flex_droplet3.md`
- `SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL.md`; `RESULTS_POINT_METRICS.md`; `RESULTS_GEN5.md`; `PROTOCOL.md` section 8; `cards/flex_droplet3.md` (lane branch feat/handwritten-science, 542dad8)
- `SpaceApps-Ardealu:docs/wsp/dossier/04_decisions.md` (D-04, D-14, D-15, D-40, D-43); `dossier/06_models.md` sections 6.1, 6.2
- `SpaceApps-Ardealu:docs/wsp/research/phys_flex_methanol.md`; `research/exp_flex.md`
- `SpaceApps-Ardealu:docs/wsp/feedback/flex_methanol_d2_train_audit.md`; `feedback/flex_methanol_d3_train_anatomy.md`
- Data pack `flex_methanol.wsp2.md` (session scratchpad)
