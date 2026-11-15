# flex_heptane.wsp2 — n-heptane droplet burning forecast

`flex_heptane.wsp2` forecasts the liquid diameter of a free-floating n-heptane droplet burning in microgravity
(NASA FLEX, PSI-69) after a cutoff, as quantiles. It works from the chamber atmosphere and the diameters
measured before the cutoff, and it follows the droplet through a hot flame, a cool flame that can follow the
radiative loss of the hot one, and evaporation without a flame. On the FLEX n-heptane validation partition its
CRPS is **46.1 µm**, against 60.3 µm for the zero-shot W0 and 105 µm for the D²-law. On the sealed test
partition, which no author read, it scores **32.2 µm** against 54.0 for the D²-law
(`docs/wsp/RESULTS.md`; `docs/wsp/RESULTS_FINAL.md`). Unchanged, it carries over to FLEX-2
(PSI-68): 20.9 µm against 32.6 for the D²-law. It is data-informed: the author read the n-heptane train
partition before typing it. Release `sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef`.

| | |
|---|---|
| Model | `flex_heptane.wsp2` |
| Investigation | FLEX (PSI-69), n-heptane droplets; scored transfer to FLEX-2 (PSI-68) |
| Family | `flex` (assemble key `flex_heptane2`) |
| Candidate | D2 (`configs/models/flex/wsp2/heptane/d2-fable`) |
| Author | `claude-fable-5-2` (Anthropic). Every number was typed by the author; there was no transcriber |
| Authorship | data-informed (train) |
| Release | `sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef` |
| Forward | `ignis_wsp::droplet3::forecast`, `crates/ignis-wsp/src/droplet3.rs` (version 3, profile `strict_no_fitting`) |
| Parent release | `sha256:8e40db9dceb898e336f69422d3e2cff2df52b6050188ef84af1a9d0a7ba4ebb1` (the D²-law baseline of the spec, the lineage root) |

State `experimental`, `trained_on = not_trained`, uncertainty kind `assumed_noise`
(`configs/models/served/models/43f7871f…/release.json`).

## 1. Task and targets

**Inputs** (served entry in `configs/models/served/served.json`):

| Name | Kind | SI unit | Note |
|---|---|---|---|
| `fuel` | condition (text) | 1 | fuel species; this release answers n-heptane only (example `Heptane`) |
| `pressure` | condition | Pa | chamber pressure (example 101325.0) |
| `x_o2` | condition | 1 | oxygen mole fraction (example 0.21) |
| `x_co2` | condition | 1 | carbon dioxide mole fraction (example 0.0) |
| `x_he` | condition | 1 | helium mole fraction (example 0.0) |
| `phase_established_at_s` | phase start | s | time the burning droplet was established; the release reads diameters from it (example 0.0) |
| `liquid_diameter` | observation | m | droplet diameters measured up to the cutoff |

The features are those of the methanol release on the same forward (`configs/models/flex/wsp2/heptane/spec.json`):
$c_{O2} = (x_{O2} - 0.21)/0.05$, $c_{CO2} = x_{CO2}/0.20$, $c_{He} = x_{He}/0.20$, $`c_{\log p} = \ln(p/10^5\,\mathrm{Pa})`$
and $`c_{\log D0} = \ln(D_0/2\,\mathrm{mm})`$. They are joined by the burn progress $w = 1 - D^2/D_0^2$, the clock
$`s = t \cdot 1\,\mathrm{mm^2/s}/D_0^2`$, the supercriticality $x_{exc} = \ln(D_0/D_{crit})$, and the prefix encoder
features (`e_late`, `e_early`, `r_late`, `r_end`, `g_span`, `g_count`, `g_progress`).

**Target.** One quantity, `liquid_diameter` (m): the measured liquid diameter at times after the cutoff, as
predictive quantiles. These are an unconditional distribution over the droplet's state (hot flame, cool
flame, no flame). The forward also reports the state probabilities and the competing-risk rows of the end of
the hot phase (`active_supported`, `hot_phase_end`, `disruption`, `unresolved`; `droplet3.rs`).

**Time origin.** The NASA DropletData processed time base (phase entry not adjudicated). The model horizon is
45 s (`num.t_valid_s`), and the support floor is 0.25 mm (`num.d_min_m`).

**Stored run or typed scenario.** A stored FLEX run of the n-heptane cohort gives everything. A typed scenario
must give the conditions and a few diameters before the cutoff, at least two (`enc.min_points`). The declared
domain (`applicability`) is the same as methanol's: $c_{O2} \in [-2.2, 4.8]$, $c_{CO2} \in [0, 4]$,
$c_{He} \in [0, 3]$, $c_{\log p} \in [-1, 1.5]$, $c_{\log D0} \in [-0.9, 1.26]$. It is the declared physical domain of
the law, not a training-support region.

## 2. Physical mechanism

The hot flame burns by the quasi-steady $D^2$ law. A droplet smaller than the critical diameter burns down until
it is too small to hold its flame, which depends on burn progress. A larger droplet loses the hot flame to
radiation at a fixed fraction of a diffusion time, which depends on the clock. For an alkane in nitrogen or
CO2 that radiative loss is usually followed by a **cool flame**: partial burning at 600–800 K, at a rate that
published theory makes independent of oxygen and pressure. The cool flame ends at a diameter set by the
chemistry, and the droplet then evaporates on its own stored heat
(`docs/wsp/research/phys_flex_heptane.md` sections 1.2, 1.6, 1.7, 1.8; `droplet3.rs`).

The laws are hand-typed hinge networks $`N(x) = b_0 + \mathrm{skip}\cdot x + \sum_j w_2[j]\,\mathrm{relu}(b_1[j] + W_1[j,:]\cdot x)`$:

- **Hot flame** (`rate.*`): $dD^2/dt = -K_h$, with $`\ln(K_h/1\,\mathrm{mm^2/s}) = N_{rate}(c, w, s) + b + s_z z`$ and
  $`s_z = \sqrt{s_{floor}^2 + (1-\mathrm{gain})\,s_{pop}^2}`$ (`unc.*`).
- **Critical diameter and class** (`crit.*`, `class.*`): $`\ln(D_{crit}/2\,\mathrm{mm}) = N_{crit}(c_{O2}, c_{CO2}, c_{He}, c_{\log p})`$
  and $`P(\text{radiative}) = \sigma(\kappa\,x_{exc} + \text{bias})`$. The physical critical diameter of the dossier is
  $`D_{crit} = D_{crit,air}\,[(K/K_{air})(\chi_c/\chi_{c,air}) / ((FSR/FSR_{air})^2 (\varepsilon/\varepsilon_{air}))]^{1/(1+n)}`$,
  and it reproduces the typed `crit` law within about 10 % in N2 and in CO2 up to 30 % (`phys_flex_heptane.md` 1.6).
- **Burn-down** (`diff.*`): $`\Lambda_D(w) = \ln 2\,(e^{a_D(w - w_D)} - e^{-a_D w_D})`$, with $w_D = \sigma(N_{diff}(c, x_{exc}))$.
- **Radiative loss of the visible flame** (`rad.*`): $`\Lambda_R(s) = \ln 2\,(e^{a_R(s/s_R - 1)} - e^{-a_R})`$, with $s_R = \exp(N_{rad}(c, x_{exc}))$.
- **Cool flame** (`cool.*`). After a radiative extinction it starts with probability $\sigma(N_{cool,p}(c, x_{exc}))$.
  It burns at $dD^2/dt = -K_c$, with $`\ln(K_c/1\,\mathrm{mm^2/s}) = N_{cool,k}(c) + b + s_z z`$, until $D = D_{ce}$, where
  $`\ln(D_{ce}/2\,\mathrm{mm}) = N_{cool,d}(c) + sd_{ce}\,u`$ (the `num.ce_*` nodes). The published cool-flame rate is
  $K_c = 8(\lambda/c_p)/\rho_l \cdot \ln[(2T_f - T_0 - T_\infty)/T_L]$ (`phys_flex_heptane.md` 1.7).
- **No flame** (`post.*`). Over a step $\Delta t$, $D^2$ falls by $k_{inf} r^m \Delta t + \text{scale}\cdot k_{hot} r^h \tau_e (1 - e^{-\Delta t/\tau_e})$,
  where $\tau_e = \tau_0 r^n$ and $`r = (D_e/2\,\mathrm{mm})^2`$. Scale is `post.cool_scale`.
- **Prefix encoder and state filter** (`enc.*`, `filt.*`, `gate.*`). The level comes from the end window and the end
  rate. The hot run offset is gain times the clamped innovations. The cool run offset is
  `enc.cool_gain` × $g$ × clamped $\ln(\text{end rate}/K_c)$. The state probabilities at the cutoff are the prior
  multiplied by $(N(r_{end}; \mu_{state}, sd_{state}) + l_{floor})^g$ (spec `notes`).
- **Disruption** (`haz.log_disruption`): a constant hazard $e^{-3.9}$ per second for a burning droplet, reported only.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section flex_heptane.wsp2.

## 3. Authoring

**Who.** The author model `claude-fable-5-2` typed every number in
`configs/models/flex/wsp2/heptane/d2-fable/edits.wsp` (78 statements). There was no fit, no search and no
transcriber. A delegated analysis agent (`claude-opus-5-5`) audited D1 on train only, and the author
rechecked its claims (`meta.json`). The record also states that this agent "listed the run ids of validation
forecast artifacts and opened none" (`meta.json`; `docs/wsp/PROTOCOL.md` section 8).

**When and under which step.** Third generation, PROTOCOL section 8. The candidate was committed
before any validation score of it existed.

**What was read** (exposure in `meta.json` and `release.json`):
- the n-heptane **train** partition of split `flex-c43bbe3580ffce2a`: conditions, series, test sheets and narratives;
- the per-case train feedback of W0 and D1 (`runs/dev/fb_train_flex_heptane.json`);
- the train-only audit of D1 (`docs/wsp/feedback/flex_heptane_d1_train_audit.md`);
- the **aggregate validation table** of W0 and D1 (CRPS and coverage only), with no per-run validation outcome or score. The edits header says nothing was chosen from it;
- the outcomes of the methanol train partition (split `flex-660b7d9ede629b30`);
- `docs/wsp/research/exp_flex.md` (published FLEX literature) and the author's pretraining knowledge, which cannot be checked.

`development_feedback` is empty. Prior candidates seen: the wsp2 baseline `8e40db9d…`, W0 `6ea680d1…` and D1
`74b3a0f1…`.

**Decisions** (D2 against D1; `meta.json` rationale, `[D2]` comments, `docs/wsp/cards/flex_droplet3.md`):

| Law | D1 | D2 | Reason on train |
|---|---|---|---|
| Early rate | a fast heat-up deficit (−0.25, fading by s = 0.15) on top of the flame-development deficit | removed; the first 0.3 clock units lifted by 0.11, so the early rate is 0.19 to 0.15 below plateau | the measured rate is 16 % above D1 at 0.3–1 s and 7 % above at 1–2 s, in 13 and 12 of 16 sessions; this also removes a spurious positive run offset (24 of 28 train cases at 1 s) |
| Radiative clock | −2.0 per unit of $x_{exc}$, $s_R$ = 0.85 at $D_{crit}$ | the same, plus −1.0 per e-fold of D0 above 2.3 mm; b0 re-anchored so a 3.4 mm droplet keeps its clock | six same-atmosphere pairs in five sessions; D1's residual is ordered by size, not by $x_{exc}$ |
| Helium in the clock | none | none, on purpose | the two radiative helium runs (35 % He, one session) lose the visible flame at 0.66 of the law, but what follows has no law here; correcting the clock alone made those train forecasts worse |
| After the cool flame | 0.06 mm²/s, decay 4 s · r | $k_{hot}$ 0.12 mm²/s · $r^{0.3}$, decay 3.7 s · $r^{0.2}$ | 13 runs in 8 sessions over four time windows |
| Hot run-offset gain at 1 / 3 / 5 s | 0.12 / 0.92 / 0.98 | about 0.09 / 0.23 / 0.31 | the law alone predicts the next seconds better than the law with D1's offset (median error 0.038 / 0.028 / 0.035 against 0.048 / 0.056 / 0.064) |
| Cool-flame run offset | the mid-to-tail innovation | half of the end-rate innovation (`enc.cool_gain` 0.5) | the rate D1 used missed the cool rate by 21 % where the law alone missed by 6 % (17 cases, 7 sessions) |
| Rate spread before / after a prefix | 0.20 / 0.07 | 0.10 / 0.06 | D1's 50 % band held 0.92 of the burning train cases at 1 s; twice the train residual for unseen sessions |

Unchanged: the critical diameter (3.7 mm in air), the class, the burn-down, and the cool-flame onset, rate and
plateau diameter. The audit found no run on the wrong side of the class, the cool rate within 6 % and the
plateau within about 10 %.

## 4. Weights

Values come from `params.npz` (data pack) and `edits.wsp`. The input order of `rate` is
$(c_{O2}, c_{CO2}, c_{He}, c_{\log p}, c_{\log D0}, w, s)$; of `diff`, `rad` and `cool.p` it is $(c, x_{exc})$;
of `crit` it is $(c_{O2}, c_{CO2}, c_{He}, c_{\log p})$; of `cool.k` and `cool.d` it is $c$.

| Group | Shape (elements / nonzero) | Value(s) | Physical meaning |
|---|---|---|---|
| `rate.b0` | [] (1/1) | −0.48 | ln of the hot rate in air, 1 atm, 2 mm |
| `rate.skip` | [7] (7/5) | [0.13, −0.08, 0.15, 0.05, −0.15, 0, 0] | oxygen, CO2, helium, pressure and size terms |
| `rate.W1` | [16,7] (112/7) | (0,0) −1; (1,0) −1; (2,2) 1; (3,4) 1; (3,5) −0.65; (4,6) −3.33; (5,6) −1.667 | hinge inputs of units 0–5 |
| `rate.b1` | [16] (16/5) | 1: −0.7; 2: −1.5; 3: −0.4; 4: 1.0; 5: 1.0 | hinge offsets |
| `rate.w2` | [16] (16/6) | 0: −0.12; 1: −0.05; 2: −0.05; 3: −0.3; 4: 0.11; 5: −0.3 | 0: below air; 1: below 17.5 % O2; 2: above 30 % He; 3: large droplet early; 4: early lift (D2); 5: flame development over 0.6 clock units |
| `crit.b0`, `crit.skip` | [], [4] (5/4) | 0.615; [0.62, −0.45, −0.08, 0] | $D_{crit}$ 3.7 mm in air at 1 atm, falling with CO2 |
| `crit.W1`, `crit.b1`, `crit.w2` | [8,4], [8], [8] (48/0) | all zero | no hinge |
| `class.kappa`, `class.bias` | [] (2/2) | 14.0, 0.5 | class boundary; no train run is on the wrong side of it |
| `diff.b0`, `diff.skip` | [], [6] (7/4) | 2.44; [0.5, −0.3, 0, 0, 0.5, 0] | burn-down: the flame goes out at D/D0 of 0.27–0.39 |
| `diff.W1`, `diff.b1`, `diff.w2` | [8,6], [8], [8] (64/5) | W1 (0,0) 1, (1,5) 1; b1 [0, 0.3]; w2 [−0.5, −1.0] | hinge terms in oxygen and in $x_{exc}$ |
| `diff.sharp` | [] (1/1) | 10.0 | Gompertz sharpness $a_D$ |
| `rad.b0`, `rad.skip` | [], [6] (7/1) | 0.227; all zero | clock level; no helium term, on purpose |
| `rad.W1`, `rad.b1`, `rad.w2` | [8,6], [8], [8] (64/5) | W1 (0,5) 1, (1,4) 1; b1 [0, −0.14]; w2 [−2.0, −1.0] | −2.0 per unit of $x_{exc}$; −1.0 per e-fold of D0 above 2.3 mm |
| `rad.sharp` | [] (1/1) | 4.0 | Gompertz sharpness $a_R$ |
| `cool.p.b0`, `cool.p.skip` | [], [6] (7/1) | 3.0; all zero | a cool flame almost always follows a radiative loss in N2 and N2/CO2 at 15–21 % O2 |
| `cool.p.W1`, `cool.p.b1`, `cool.p.w2` | [8,6], [8], [8] (64/6) | W1 (0,0) −1, (1,2) 1; b1 [−1.0, −0.75]; w2 [−7.5, −7.0] | weak at 14 % O2; none with 30 % helium |
| `cool.k.b0`, `cool.k.skip` | [], [5] (6/6) | −0.97; [0.2, −0.05, 0.14, 0.1, 0.2] | cool-flame rate 0.32–0.37 mm²/s (ten train runs within 6 %) |
| `cool.k.W1`, `cool.k.b1`, `cool.k.w2` | [8,5], [8], [8] (56/0) | all zero | no hinge |
| `cool.d.b0`, `cool.d.skip`, `cool.d.sd` | [], [5], [] (7/7) | −0.77; [−0.25, 0.05, 0.2, −1.0, 0.8]; 0.12 | cool-flame end (plateau) diameter, about $D_0^{0.8}$; ten train runs within about 10 % |
| `cool.d.W1`, `cool.d.b1`, `cool.d.w2` | [8,5], [8], [8] (56/0) | all zero | no hinge |
| `post.cool_scale` | [] (1/1) | 1.0 | scale of the transient after a cool flame |
| `post.k_inf`, `post.size_exp` | [] (2/2) | 3.0e−8 m²/s, 0.3 | long-time evaporation |
| `post.k_hot`, `post.hot_exp` | [] (2/2) | 1.2e−7 m²/s, 0.3 | transient amplitude (0.11 mm²/s at 1.7 mm in the edits note) |
| `post.tau0_s`, `post.tau_exp` | [] (2/2) | 3.7 s, 0.2 | transient decay time |
| `enc.gain_w`, `enc.gain_b` | [3], [] (4/2) | [0.8, 0, 0], −1.8 | hot run-offset gain sigmoid(−1.8 + 0.8 ln span) |
| `enc.mix`, `enc.e_clip` | [2], [2] (4/4) | [0.85, 0.15]; [0.3, 0.6] | innovation weights; clamp below / above |
| `enc.cool_gain` | [] (1/1) | 0.5 | half of the cool-flame end-rate innovation is kept |
| `filt.sd_hot`, `filt.sd_evap`, `filt.l_floor` | [] (3/3) | 0.2, 0.15, 0.05 | state filter spreads, kept from D1 (right in 82 of 82 hot and 11 of 12 cool train cases) |
| `filt.gain_w`, `filt.gain_b` | [3], [] (4/4) | [2.0, 0.5, 2.0], −1.5 | exponent of the end-rate evidence |
| `unc.s_pop`, `unc.s_floor` | [] (2/2) | 0.10, 0.06 | rate spread before any prefix and the part no prefix removes |
| `unc.sigma_d`, `unc.sigma_frame` | [] (2/2) | 2.5e−5 m, 8.0e−6 m | measurement sd; frame-to-frame sd |
| `haz.log_disruption` | [] (1/1) | −3.9 | ln of the disruption hazard (1/s) |
| constants `norm.*` | 4 | d_ref 0.002 m, p_ref 1e5 Pa, k_ref 1e−6 m²/s, n_ref 32 | references (spec, fixed) |
| constants `enc.*` | 5 | window_frac 0.25, end_frac 0.15, k_floor 2e−8 m²/s, w_ref 0.08, min_points 2 | prefix windows |
| constants `gate.*` | 3 | growth_tol 0.05, min_later 3, k_max 2e−6 m²/s | validity gate |
| constants `num.*` | 8 | dt 0.05 s, bin 0.5 s, d_min 2.5e−4 m, t_valid 45 s, z_step 0.5, z_count 15, ce_step 1, ce_count 5 | numerics |

**Totals** (`release.json`, `architecture.parameters`): 575 authored elements, 93 nonzero, 20 constants.
The full list is printed by `ignis wsp inspect --release sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef --edits`,
or can be read in `configs/models/flex/wsp2/heptane/d2-fable/edits.wsp`.

## 5. Forward and assembly

| | |
|---|---|
| Forward | `crates/ignis-wsp/src/droplet3.rs`, entry `ignis_wsp::droplet3::forecast`, version 3, `strict_no_fitting` |
| forward_hash | `sha256:71ea3173cd8150cde64bcaf5bd069a4d8519584377051c5372b75413f41b950a` |
| code_version | `ignis-wsp@0.1.0+fwd.71ea3173cd8150cd` |
| Spec | `flex_heptane-wsp-v2`, `sha256:a74d5300893a9b9310f36e8530f4905656a96280b5ddd5490820c2c11c5d1d1f` |
| Patch | `flex_heptane-wsp-v2-d2-fable`, `sha256:502e1c6bf7075959368915212fd55065fcdf725e3d6074f00a59392db6d4ac84` |
| weights_hash | `sha256:e8e283be560360a49742a6c09cada94d91bfd583473050eb2060fe23b18acd15` |
| Binding | cohort `sha256:8ffa1875d0c963fe3b3ce6886ba7197a7b0c8675ffd837bbf5139049c1655797`, split `sha256:c43bbe3580ffce2a33c400d5aea73b3330cf721133f925c2963579dee6d32001`, source manifest `sha256:74dedf7e…6a382` (`configs/models/flex/wsp2/heptane/binding.json`) |
| Candidate directory | `configs/models/flex/wsp2/heptane/d2-fable` |
| Assembly checks | spec_valid, patch_valid, tensors_complete, strict_forward_audit, finite_forward_on_fixture: all passed |

Rebuild, from the ignis-web root. The parent is assembled first into a scratch registry:

```
cargo build --release -p ignis-cli
ignis wsp compile  --family flex_heptane2 --edits configs/models/flex/wsp2/heptane/baseline-d2/edits.wsp --meta configs/models/flex/wsp2/heptane/baseline-d2/meta.json --out configs/models/flex/wsp2/heptane/baseline-d2/patch.json
ignis wsp assemble --family flex_heptane2 --patch configs/models/flex/wsp2/heptane/baseline-d2/patch.json --binding configs/models/flex/wsp2/heptane/binding.json --root <registry>
ignis wsp compile  --family flex_heptane2 --edits configs/models/flex/wsp2/heptane/d2-fable/edits.wsp --meta configs/models/flex/wsp2/heptane/d2-fable/meta.json --out configs/models/flex/wsp2/heptane/d2-fable/patch.json
ignis wsp assemble --family flex_heptane2 --patch configs/models/flex/wsp2/heptane/d2-fable/patch.json --binding configs/models/flex/wsp2/heptane/binding.json --root <registry holding models/8e40db9dceb898e336f69422d3e2cff2df52b6050188ef84af1a9d0a7ba4ebb1/> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the id and the record; `served_ids` pins the
14 served ids. `cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at
ignis-web a58c7c3: the served id is the content hash of its record and loads through the registry checks.

## 6. Evaluation

CRPS of the droplet diameter in µm, lower is better; lpd is higher-better. Cutoffs are 1, 3 and 5 s.

**Headline: the sealed test partition.** This partition holds 43 cases of 16 runs in 7 sessions. It is
**blind**: no author read it, and it was scored once on 15 November (`docs/wsp/RESULTS_FINAL.md`,
step 1). There is no fitted heptane baseline.

| Release | Cases | CRPS | MAE of median | Cov. 50 % | Cov. 90 % |
|---|---|---|---|---|---|
| **D2 (served)** | 43 | **32.2** | 43.9 | 0.68 | 0.91 |
| D²-law of the spec | 43 | 54.0 | 72.4 | 0.56 | 0.77 |
| D2, on the 42 cases W0 answers | 42 | 33.3 | 46.0 | 0.68 | 0.92 |

Paired CRPS, D2 minus comparator (7 sessions, 95 %): D²-law −21.8 [−47.1, −4.9], W0 −47.0 [−124.9, −3.2].
Post-hoc point metrics on the same cases (`docs/wsp/RESULTS_POINT_METRICS.md`): MAE of the
median 43.9 µm and nRMSE 5.41 %, against 72.4 µm and 6.78 % for the D²-law. The paired MAE difference is
−28.5 µm [−60.1, −5.2].

**Transfer to FLEX-2 (PSI-68), unchanged.** 8 n-heptane tests, 23 cases, on 2 train-split days of FLEX-2,
measured from NASA's frames by the lane on a pinned scale and scored once by the delegated scoring agent
(`docs/wsp/feedback/flex2_transfer_result_v2.md`; `COVERAGE_19.md`):

| Release | CRPS | MAE of median | Cov. 50 % | Cov. 90 % |
|---|---|---|---|---|
| **D2** | **20.9** | 25.0 | 0.88 | 1.00 |
| D²-law of the spec | 32.6 | 45.8 | 0.63 | 0.99 |
| classical D² law (chain) | 124.7 | 124.7 | | |
| persistence | 444.8 | 444.8 | | |

D2 is lower than both baselines on each of the two days (25.5 and 15.9 µm against 35.5 and 29.5 for the
D²-law). The pooled difference to the D²-law is −11.7 µm, with a day bootstrap of [−13.6, −10.0]. The
registered rule asked for three days, and the third n-heptane day has no usable series.

**Validation partition.** 39 common cases in 7 clusters (`docs/wsp/RESULTS.md`, FLEX n-heptane). The author had
seen the aggregate table of W0 and D1, but no score of D2 itself. D2 was committed before its validation score
existed, and that score was read once.

| Release | CRPS | MAE of median | Cov. 50 % | Cov. 90 % | lpd |
|---|---|---|---|---|---|
| **`flex_heptane.wsp2` D2** | **46.1** | 63.0 | 0.71 | 0.97 | 8.25 |
| `flex_heptane.wsp2` D²-law | 105.1 | 134.3 | 0.61 | 0.79 | 7.69 |

Paired CRPS, other minus D2: D1 +5.8 [−3.9, +17.2], W0 +14.2 [+1.7, +25.6], D²-law of the spec +59.0
[+10.4, +134.2].

**Train partition** (in-sample). 125 cases, 9 refused (`docs/wsp/RESULTS.md`):

| Release | CRPS mean | median | mean, worst 5 % dropped | MAE of median | Cov. 50 % / 90 % |
|---|---|---|---|---|---|
| **D2** | **46.2** | **21.8** | 22.6 | 54.1 | 0.76 / 0.93 |
| D1 | 49.3 | 28.2 | 28.3 | 59.2 | 0.71 / 0.90 |
| W0 | 58.3 | 33.0 | 37.5 | 71.7 | 0.74 / 0.93 |
| D²-law of the spec | 75.4 | 34.9 | 46.9 | 97.2 | 0.57 / 0.82 |

D2 is better in 17 of 20 train sessions against D1 (`docs/wsp/PROTOCOL.md` section 8). The post-hoc anatomy
(`docs/wsp/feedback/flex_heptane_d2_train_anatomy.md`) finds that the problem runs carry 58 %
of the train CRPS, and that the 90 % band is 2.5–4 times wider than the realised errors on clean cases.

## 7. Adoption decision and lineage

**Why this release is served.** It passed the droplet selection rule written before validation: the median
train CRPS over common cases not worse, and better or level in at least 60 % of train sessions. D2 had 21.8
against 28.2 µm and was better in 17 of 20 sessions. Validation then acted only as a veto, and D2 improved it
from 51.9 to 46.1 µm (`docs/wsp/PROTOCOL.md` section 8; `docs/wsp/dossier/04_decisions.md`
D-14, D-40). Every later candidate was measured against it and none replaced it.

**Lineage.**
- `flex_heptane.wsp2` D²-law baseline `sha256:8e40db9dceb898e336f69422d3e2cff2df52b6050188ef84af1a9d0a7ba4ebb1`.
  The lineage root: the classical D²-law at a population rate of 0.55 mm²/s, with no extinction, no cool flame
  and no evaporation (`baseline-d2/edits.wsp`).
- → **D2** `sha256:edbad4d6…edef`: a literal revision of that root. It carries the laws of D1 with the changes in section 3.

**Tried and not adopted:**
- fourth generation, `flex_heptane.wsp3` D3 `sha256:e032ba3f…cc55`: 57.6 against 46.1 on validation (D-15).
- fifth generation, `flex_heptane.wsp5` P0 `sha256:9a108f1a…ac0b` (47.2) and P1 `sha256:57fa63b2…78ce` (48.4, flame cameras), against 46.1 (`docs/wsp/RESULTS_GEN5.md`; D-43).
- `flex_heptane.wsp2` D3 `sha256:b879b3da39a591b27e33926e4f03265879930b39a9f503ae2655be1e361d5b52` (15 November, PROTOCOL 13.7). One law changed: the cool-flame rate. On 36 validation cases in 6 sessions it scored a CRPS at 10 s of 51.3 against 49.9 µm, and a MAE of the median at 39 s of 101.6 against 98.6. All three criteria failed (`docs/wsp/RESULTS_HEPTANE_D3.md` section 7; D-50).
- `flex_heptane.wsp10` D4 `sha256:f3ccc122172b9d680cd17af5183bc640bf643d1175ed576b684ec9b6f324b164` (PROTOCOL 13.9), with a transition stage added on a copy of the forward. In stage A of a blind test on fresh tests, its MAE at 10 s was 18.9 against 20.7 µm, but its nRMSE over the burn was 3.84 % against 3.68 %. Criterion 2 failed, so D2 stays (`docs/wsp/PROTOCOL.md` 13.9).

**Scope** (served `limits`, condensed):
- Tracker artefacts are scored as data. About two fifths of the train error is glare, soot, bubbles and dropouts.
- The stage after the visible flame in 30–35 % helium has no law (one train session), and the clock is deliberately not corrected.
- A flame that went out less than a second before the cutoff still reads as burning.
- It covers FLEX-2 (PSI-68) as a scored transfer, unchanged (above). Zero-shot candidates for n-decane, n-octane, iso-octane and n-dodecane (CFI, PSI-39) exist but are not served.

## 8. How to call it in IGNIS

**Forecast lab** (web, route `#/lab`). The Models screen (`#/models`) lists the releases and opens the model
card. `web/src/views/LabModels.tsx` runs a stored run or a typed scenario.

**Model card:** `GET /api/v1/model-cards`. The fields come from `served.json`; `card_doc` is
`docs/wsp/cards/flex_droplet3.md`.

**API.** Stored runs of the n-heptane cohort work (`stored_runs: true`). `settings.target` has one value,
`liquid_diameter` (the default), so it may be omitted. There are no controls and no combination rule. Forecast
times must stay within 45 s of the time origin.

1. Typed scenario: `POST /api/v1/models/sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef/scenarios`.
   The values below are the served example values, typed and not measurements:

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef",
  "cutoff_s": 3.0,
  "conditions": {"fuel": "Heptane", "pressure": 101325.0, "x_o2": 0.21, "x_co2": 0.0, "x_he": 0.0},
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
  "release_id": "sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef",
  "input": {"kind": "prefix", "input_hash": "<input_hash>"},
  "settings": {"target": "liquid_diameter", "times_s": [4, 6, 8, 10, 13], "horizon_s": 10, "quantile_levels": [0.05, 0.25, 0.5, 0.75, 0.95]}
}
```

   For a stored run, use `"input": {"kind": "stored_run", "run_id": "<FLEX n-heptane run id>", "cutoff_s": 3}`.
   The call returns a job. Poll `GET /api/v1/jobs/<job_id>`, then read `GET /api/v1/artifacts/<artifact_id>`:
   an `AnalysisArtifact` with `evidence_class` `model_output`.

**MCP tools:** `models_inspect` and `models_forecast`.

**Cite:** flex_heptane.wsp2 D2, hand-written (literal WSP) release sha256:edbad4d6…edef, experimental, not trained; its output is model output, not a measurement.

## Sources

- `configs/models/served/served.json`; `configs/models/served/models/edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef/release.json` (and `params.npz`, through the data pack)
- `configs/models/flex/wsp2/heptane/d2-fable/{edits.wsp, meta.json, patch.json, release.json}`; `configs/models/flex/wsp2/heptane/baseline-d2/{edits.wsp, release.json}`; `configs/models/flex/wsp2/heptane/d3-fable/release.json`; `configs/models/flex/wsp2/heptane/{spec.json, binding.json}`
- `configs/models/flex/wsp/heptane/{baseline-d2, w0-fable, d1-fable}/{edits.wsp, release.json}`
- `crates/ignis-wsp/src/droplet3.rs`; `crates/ignis-wsp/src/cli.rs`; `crates/ignis-wsp-serve/src/lib.rs`; `crates/ignis-core/src/contracts/wsp.rs`; `tools/wsp/verify_served.py`; `tools/wsp/reassemble.py`; `web/src/views/LabModels.tsx`
- `docs/wsp/README.md`; `docs/wsp/RESULTS.md`; `docs/wsp/RESULTS_GEN5.md`; `docs/wsp/PROTOCOL.md` section 8; `docs/wsp/cards/flex_droplet3.md`
- `docs/wsp/RESULTS_FINAL.md`; `RESULTS_POINT_METRICS.md`; `RESULTS_HEPTANE_D3.md`; `RESULTS_GEN5.md`; `COVERAGE_19.md`; `PROTOCOL.md` sections 8, 11, 13.7, 13.9 (lane branch feat/handwritten-science, 542dad8)
- `docs/wsp/dossier/04_decisions.md` (D-10, D-14, D-15, D-40, D-43, D-50); `dossier/06_models.md` sections 6.1, 6.3
- `docs/wsp/research/phys_flex_heptane.md`; `research/exp_flex.md`
- `docs/wsp/feedback/flex_heptane_d1_train_audit.md`; `feedback/flex_heptane_d2_train_anatomy.md`; `feedback/flex2_transfer_result_v2.md`
- Data pack `flex_heptane.wsp2.md` (session scratchpad)
