# bre.wsp — self-extinction of a flame over a porous burner (ACME BRE)

`bre.wsp` forecasts whether a flame over the porous-disc burner of the ACME Burning Rate Emulator
(BRE, PSI-20), in a quiescent microgravity chamber, is still burning a stated time after ignition at an
unchanged flow, and through that the chance it has gone out by itself by any horizon. On the 95 unread
test-day cases it scored 0.139 on extinction (integrated Brier) against 0.222 for the incidence
baseline and 0.245 for its zero-shot parent read as registered
(SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md). It is data-informed: the third law `d3`, a revision
of the zero-shot W0 written by `claude-fable-5-2` after studying the development tests (train and
validation days). Release `sha256:27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955`.

| | |
|---|---|
| Model | `bre.wsp` |
| Investigation | ACME BRE (PSI-20), porous-disc burner of 25 or 50 mm in a quiescent microgravity chamber |
| Family | `other` (generic law slug `bre`) |
| Candidate | D3 (`configs/models/law/bre/d3-fable`) |
| Generation | seventh (15 November 2026), PROTOCOL 13.6 (study of the development data) |
| Author | `claude-fable-5-2`; transcribed by the assembly engineer `claude-opus-5-5` |
| Authorship | data-informed (train and validation days) |
| Release | `sha256:27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955` |
| Forward | `ignis_wsp::law::forecast`, `crates/ignis-wsp/src/law.rs` (version 1, profile `strict_no_fitting`) |
| Parent release | `sha256:aa0eef5bc7653922c7a7838796dab6946294e89f79c0b6c500124d9a4b212b4d` (bre W0, zero-shot) |

## 1. Task and targets

Inputs (all typed conditions, all required; `configs/models/served/served.json`, declaration
`configs/models/law/bre/decl.json`):

| Name | Kind | SI unit | Transform | Note |
|---|---|---|---|---|
| `burner_diameter` | condition | m | ln | diameter of the porous face (0.025 or 0.050) |
| `burner_mass_flux` | condition | kg/m2/s | ln | mass flow of the burner gas (fuel plus its nitrogen) at the first setpoint, per unit face area |
| `x_o2` | condition | 1 | ln | oxygen mole fraction of the chamber gas just before ignition |
| `pressure` | condition | Pa | ln | chamber pressure just before ignition |
| `x_fuel` | condition | 1 | ln | fuel mole fraction of the burner gas (1 for a pure fuel, 0.5 for the 50 % bottle) |
| `is_methane` | condition | 1 | identity | 1 for methane, 0 for ethylene |
| `stated_time` | condition | s | ln | the time asked about by `lasts` |

Targets declared (`release.json`, `observation_model.targets`); only `lasts` is served
(`served.json`, `targets: ["lasts"]`):

| Target | Kind | Unit | Meaning | Served |
|---|---|---|---|---|
| `lasts` | binary | none | the flame is still burning `stated_time` seconds after ignition, flow unchanged | yes |
| `flame_end` | event (cause `extinction`) | s | the flame goes out by itself while the fuel flow is still at its first setpoint | no |
| `height_at_end` | positive | m | flame height above the burner centre in the last second before self-extinction (lane-local) | no |

Time origin: ignition. `lasts` is returned at `times_s = [cutoff_s]` as `event_probabilities` with
categories `yes`/`no` (SpaceApps-Ardealu:docs/wsp/research/law_forward.md, section 2.4); the time it
asks about is the input `stated_time`, not the cutoff. No stored run carries the declared features
(`stored_runs: false`), so every call is a typed scenario that gives all seven conditions; a value
outside the declared domain is refused with its name.

## 2. Physical mechanism

A porous disc of radius $R$ feeds gas at a mass flux $m$ into still gas. Without buoyancy the flame is
a dome that grows by diffusion towards a steady size, and its heat release is spread over an ever
larger sheet; radiation puts it out once the release per unit sheet area falls below a critical value.
Two heights, in burner radii, decide the outcome
(SpaceApps-Ardealu:docs/wsp/research/phys_bre_w0.md, section 0):

```math
\xi_{ss} = \cot(L/Pe),\qquad Pe = \frac{mR}{2\rho D},\qquad L = \ln\!\left(1 + \frac{Y_O}{S\,Y_F}\right)
```

```math
\xi_e = \sqrt{\Lambda - 2/3},\qquad \Lambda = \frac{m\,Y_F\,\Delta h_c\,\big(1 - \phi_b/(1-\theta_c)\big)}{2\,q_{c0}\,p^a}\quad (\text{W0: } a = 0.5)
```

The flame is held (burns as long as it is fed) if it reaches its steady size first, $b = \ln(\xi_{ss}/\xi_e) \lt0$;
otherwise it goes out at a time that shortens steeply as $b$ grows from 0 to 0.5. $q_{c0}$ = 9.4 kW/m2
is the published critical release of a non-buoyant ethylene flame.

The third law (SpaceApps-Ardealu:docs/wsp/research/phys_bre_study.md, section 2) keeps W0's steady
height, extinction height, growth curve and constants, and removes two things W0 had added. The
burner's share of the heat $\phi_b$ is charged at the extinction temperature, not the adiabatic one, and
pressure has no exponent (the development data give 0 within about 0.2):

```math
\Lambda_0 = \frac{m_{fuel}\,\Delta h_c}{2\,q_{c0}},\qquad A = \frac{1-\theta_c}{1-\theta_c(1+\phi_b)},\qquad \xi_e = \sqrt{\Lambda_0/A - 2/3}
```

```math
t_{ext} = 0.42\,\frac{R^2\,\xi_e^2}{D_0}\,e^{G(b)}\quad (b \ge 0),\qquad \text{held if } b < 0,\qquad \text{no flame stands if } \Lambda_0\,d/25\,\mathrm{mm} \lesssim 2.2
```

The factor 0.42 is the clock read off the development tests ($D_{eff} = 1.2\times10^{-4}$ m2/s, the
diffusivity of gas at 1000 K and more). In the network's features
$z = (z_d, z_m, z_o, z_p, z_f, z_c, z_t)$, all ln except the methane indicator $z_c$ (centre: 25 mm,
4 g/m2/s, 26.5 % oxygen, 1 atm, pure ethylene, 30 s), the law is three linear forms
(`configs/models/law/bre/d3-fable/edits.wsp`):

```math
b = 0.514 + 1.01 z_d + 0.47 z_m - 0.93 z_o + 0.43 z_f + 0.11 z_c
```

```math
\mathrm{base} = 2.451 + 2.05 z_d + 1.12 z_m - 0.03 z_o + 1.12 z_f + 0.08 z_c \quad (\ln \mathrm{s})
```

```math
s = 2.307 + z_d + z_m + z_f = \ln(\Lambda_0\,d/25\,\mathrm{mm})
```

and the served target is

```math
\mathrm{logit}\;\texttt{lasts} = 5.5\,\big(H(v) - b\big) - 9\,\mathrm{relu}(0.90 - s),\qquad v = 0.950 + z_t - (\mathrm{base} - 2.451)
```

$H = G^{-1}$ is the $b$ whose life is $`t^*`$, a falling curve in six pieces (hinge units 0 to 5 of
`lasts.logit`, knots 3.46, 1.21, 0.60, 0.09, -0.35, -0.77); hinge unit 6 is the stand-off penalty. The
slope 5.5 is $1.7/0.31$, the spread of $b$ (W0 had 4.0 for 0.425). For long times $H \to 0$ and the
probability tends to that of a held flame, so the held class is in the target by construction
(SpaceApps-Ardealu:docs/wsp/research/phys_bre2.md, section 2).

**How the extinction probability is read** (scoring-time rule, phys_bre2.md section 2): for a horizon
$h$ and a cutoff $c$ (seconds after ignition, at least 1 s), with the planned stop at `stop`,

```math
P(\text{out by } h \mid \text{burning at } c) = 1 - \frac{\texttt{lasts}(\min(h, \mathrm{stop}))}{\texttt{lasts}(c)},\qquad P(\text{operator termination}) = \frac{\texttt{lasts}(\mathrm{stop})}{\texttt{lasts}(c)}\ \ (h \ge \mathrm{stop})
```

`lasts` is non-increasing in `stated_time`, so the ratios lie in [0, 1].

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section bre.wsp.

## 3. Authoring

- **Who.** Author `claude-fable-5-2` (`release.json`, `construction.author`). The record reads
  "runs/dev/gen7/study/bre_d3.wsp verbatim, a revision of W0. Transcribed by the assembly engineer
  (claude-opus-5-5)" (`configs/models/law/bre/d3-fable/meta.json`). The result file checks that the
  file's assignment lines are identical to the dossier's block and that d3 equals W0 except exactly the
  144 elements it writes (SpaceApps-Ardealu:docs/wsp/feedback/bre_d3_result.md).
- **When and which step.** Seventh generation, 15 November 2026; PROTOCOL 13.6, "A proper study of the
  development data": the author studied the development tests one by one (111 answered tests of 14
  days), wrote the expected effect of each candidate in the log before running it once with the logged
  what-if tool, and stopped at five candidates (C1 to C4 and the d3 file).
- **What was read.** Exposure (`release.json`, `construction.exposure`): datasets
  `bre-train-and-validation-cases`, partitions `train` and `validation`; development feedback
  `PROTOCOL-13.2-to-13.6-development-scores`; sources `docs/wsp/research/phys_bre_study.md` (sha256
  `dbdced679233e2aee34d82094c1dc35da558d67cce75da205fe5ade28577d10a`) and
  `runs/dev/gen7/whatif_log.jsonl`; prior candidate W0. The study states that no test day was opened.
- **Brief.** The physics brief and the author prompts are collected in
  [docs/research/PROMPTS.md](../../research/PROMPTS.md). The author worked from
  SpaceApps-Ardealu:docs/wsp/research/phys_bre_w0.md (W0), phys_bre_d1.md, phys_bre2.md and
  SpaceApps-Ardealu:docs/wsp/feedback/gen7_failure_analysis.md, and wrote
  SpaceApps-Ardealu:docs/wsp/research/phys_bre_study.md.
- **Decisions** (phys_bre_study.md, sections 0 to 4). One number orders the outcomes, the size ratio
  `b`, once the adiabatic burner share and the pressure exponent are taken out: of 32 ethylene flames
  on the 25 mm burner with $b \lt0$ all 32 were burning at the planned stop, and of 24 with $b \gt0.2$ all
  24 went out. The candidates, cumulative:
  1. C1: `b` with the burner share cut and no pressure, a pressure-free clock, one stand-off hinge
     (0.1955 to 0.1444 on extinction);
  2. C2: the clock at 0.42 of W0's diffusion time (0.1271);
  3. C3: the spread of `b` from 0.425 to 0.31, slope 4.0 to 5.5 (0.1121, 14 of 14 days below W0 read
     through `lasts`): these are the `lasts` lines of d3;
  4. C4: C3 plus a methane critical flux 1.7 times ethylene's (0.0980; one day changes): the
     comparator d3m.

  d3 also brings `flame_end` and `height_at_end` to the same law, so the release does not carry two
  physics, and widens the domain (`x_fuel` from 0.2, flux from 0.3 g/m2/s); those two targets are not
  used by the score.

## 4. Weights

Every number written by d3 is in `configs/models/law/bre/d3-fable/edits.wsp`; the rest are W0's
(`configs/models/law/bre/w0-fable/edits.wsp`). The full list is printed by
`ignis wsp inspect --release sha256:27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955 --edits`.
Feature columns are `(z_d, z_m, z_o, z_p, z_f, z_c, z_t)`.

| Group | Shape (nonzero) | Values | Physical meaning | Written by |
|---|---|---|---|---|
| `x.burner_diameter.*` | 4 scalars (4) | centre 0.025, scale 1.0, lo 0.02, hi 0.06 | 25 mm centre; 20 to 60 mm | W0 |
| `x.burner_mass_flux.*` | 4 (4) | 0.004, 1.0, 0.0003, 0.015 | 4 g/m2/s centre; 0.3 to 15 g/m2/s | W0; lo widened by d3 (W0 0.0008) |
| `x.x_o2.*` | 4 (4) | 0.265, 1.0, 0.18, 0.45 | oxygen 18 to 45 % | W0 |
| `x.pressure.*` | 4 (4) | 101325, 1.0, 50000, 105000 | 50 to 105 kPa | W0 |
| `x.x_fuel.*` | 4 (4) | 1.0, 1.0, 0.2, 1.0 | fuel fraction 0.2 to 1 | W0; lo widened by d3 (W0 0.3) |
| `x.is_methane.*` | 4 (2) | centre 0, scale 1, lo 0, hi 1 | fuel indicator (ethylene reference) | W0 |
| `x.stated_time.*` | 4 (4) | 30.0, 1.0, 1.0, 3000.0 | 30 s centre; 1 to 3000 s | W0 |
| `lasts.logit.b0` | [] (1) | -2.83 | $-5.5 \times 0.514$: $`-5.5\,b`$ at the centre | d3 |
| `lasts.logit.skip` | [7] (5) | [-5.56, -2.59, 5.12, 0.0, -2.37, -0.61, 0.0] | $-5.5$ times the coefficients of `b`; pressure 0 (W0 had -1.08) | d3 |
| `lasts.logit.W1` rows 0-5 | [8, 7] (36 of 45) | each [2.05, 1.12, -0.03, 0.0, 1.12, 0.08, -1.0] | $\mathrm{knot} - v$: the clock `base` and $-z_t$ | d3 (columns 0-4), W0 (columns 5-6) |
| `lasts.logit.b1[0:6]` | (6) | [2.51, 0.26, -0.35, -0.86, -1.30, -1.72] | knots 3.46, 1.21, 0.60, 0.09, -0.35, -0.77 minus 0.950 | d3 |
| `lasts.logit.w2[0:6]` | (6) | [0.97, 0.56, 0.67, 0.87, 1.14, 1.62] | 5.5 times the slope increments of $H$ | d3 |
| `lasts.logit.W1[6]`, `b1[6]`, `w2[6]` | (3), (1), (1) | [-1, -1, 0, 0, -1, 0, 0]; -1.41; -9.0 | stand-off penalty $`-9\,\mathrm{relu}(0.90 - s)`$ | d3 |
| `lasts.logit.W1[7]`, `b1[7]`, `w2[7]` | (6), (1), (0) | [-1.17, -2.17, 0.65, 0.5, -1.97, -0.18, 0]; -2.8; 0.0 | W0's steep small-flame hinge, switched off by d3 | W0 (d3 zeroed w2) |
| `flame_end.extinction.median.b0` | [] (1) | 2.09 | $2.451 - 0.03 - 0.65 \times 0.514$ | d3 |
| `flame_end.extinction.median.skip` | [7] (5) | [1.39, 0.81, 0.57, 0.0, 0.84, 0.01, 0.0] | `base` plus the chord of $G$ | d3 |
| `flame_end.extinction.median.W1`, `b1`, `w2` | (38), (8), (8) | rows 0-6 [-1.01, -0.47, 0.93, 0, -0.43, -0.11, 0]; b1 [1.286 ... -0.264, -1.41]; w2 [0.29, 0.36, 0.49, 0.70, 1.09, 2.09, 5.03, -6.0] | $G(b)$ on knots 1.8 to 0.25; row 7 the stand-off penalty | d3 |
| `flame_end.extinction.shape` | [] (1) | 3.0 | log-logistic clock shape (W0 2.2) | d3 |
| `height_at_end.median.b0`, `skip` | (1), (5) | -3.28; [1.03, 0.56, -0.02, 0.0, 0.56, 0.04, 0.0] | $`\ln(R\,\xi_e / 1\,\mathrm{m})`$, $-3.28 = \ln(0.0125 \times 3.01)$ | d3 |
| `height_at_end.spread.b0`, `tail_weight`, `tail_width` | 3 (3) | 0.30, 0.1, 3.0 | ln-sd and tail | W0 |
| other `height_at_end` tensors | (0) | zero | no hinge | |
| `num.substeps` | constant | 512 | integration steps for the event target (numerical, not physics) | spec |

Totals (`release.json`, `architecture.parameter_counts`): 351 authored elements, 162 nonzero,
1 constant.

## 5. Forward and assembly

| Field | Value |
|---|---|
| Forward | `crates/ignis-wsp/src/law.rs`, entry point `ignis_wsp::law::forecast`, version 1, profile `strict_no_fitting` |
| `forward_hash` | `sha256:be70d6a2dbebdab11c1aa190d9cdefd5460bd7599b445aa64c83fa61e20992ca` (the slug's declaration is one of its forward sources) |
| `code_version` | `ignis-wsp@0.1.0+fwd.be70d6a2dbebdab1` |
| Spec | `law-bre-v1`, `sha256:a27a3dafa38c9824a7335b7fc7581011e275e01713abfa171e926df8b63d9248` |
| Patch | `law-bre-v1-d3-fable`, `sha256:c58e56066811b25a69622ca16a09eafddb8460bed723aba51e9f08d4c096a56e` |
| `weights_hash` | `sha256:1a9bfa3431f2131659a56498ba407351941ba8991790f725410d2445fba9b104` |
| Binding | cohort `atlas:PSI-20:catalogue`, split `sha256:bb87c94d1cfd31b7ef0d5eee49be63721abaea6a34745f5ca6e982fa66ec7c4c`, source manifest `sha256:2264f4414c6ba585f264555cec5499a5b2eede40448a4a953bf130df6fad1695` (`configs/models/law/bre/binding.json`) |
| Candidate directory | `configs/models/law/bre/d3-fable` (`edits.wsp`, `meta.json`, `patch.json`, `release.json`) |
| Served record | `configs/models/served/models/27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955/` |
| Checks | `spec_valid`, `patch_valid`, `tensors_complete`, `strict_forward_audit`, `finite_forward_on_fixture`: passed |

Rebuild (the `--root` registry must hold the parent W0 as
`models/aa0eef5bc7653922c7a7838796dab6946294e89f79c0b6c500124d9a4b212b4d/`; its candidate directory is
`configs/models/law/bre/w0-fable`):

```
cargo build --release -p ignis-cli
ignis wsp compile  --family bre --edits configs/models/law/bre/d3-fable/edits.wsp --meta configs/models/law/bre/d3-fable/meta.json --out configs/models/law/bre/d3-fable/patch.json
ignis wsp assemble --family bre --patch configs/models/law/bre/d3-fable/patch.json --binding configs/models/law/bre/binding.json --root <registry holding models/aa0eef5b…2b4d/> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the record; `served_ids` pins the 14
served ids. `cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at
ignis-web a58c7c3: the served id is the content hash of its record and loads through the registry
checks.

## 6. Evaluation

Metric: IPCW integrated Brier score of the two end causes (extinction, operator termination at the
planned stop), lower is better; d3 is read through the `lasts` rule of section 2.

**Headline: test days.** The final run (PROTOCOL 13.7, run once on the frozen list sha256
`f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`; 3 days, 56 tests; outcome coding
v1; the author had not seen them) (SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md, section PSI-20):

| Release | Role | Rule | Cases | Extinction IBS | Mean of the two causes | Operator termination IBS |
|---|---|---|---|---|---|---|
| **bre d3** | model | `lasts` | 95 | **0.1389** | **0.0962** | **0.0535** |
| bre W0 (registered zero-shot reading) | comparator 1 | `flame_end` | 83 | 0.2448 | 0.1759 | 0.1070 |
| bre W0 | comparator 2 | `lasts` | 83 | 0.2047 | 0.1473 | 0.0899 |
| bre D1 | comparator 3 | `flame_end` | 83 | 0.2223 | 0.1603 | 0.0983 |
| bre d2 | comparator 4 | `lasts` | 83 | 0.1984 | 0.1436 | 0.0887 |
| bre d3m | comparator 5 | `lasts` | 95 | 0.0982 | 0.0701 | 0.0419 |
| incidence (simple) | baseline, on the model's cases | | | 0.2220 | 0.1626 | 0.1031 |
| planned stop | baseline, on the model's cases | | | 0.2250 | 0.1625 | 0.0999 |

d3 holds on both causes by the rule registered before the run (lower than every baseline), and the
improvement over the zero-shot parent holds (0.2448 to 0.1389) (PROTOCOL 13.7, "Outcome of 13.7";
COVERAGE_19.md). The comparators of W0, D1 and d2 answer 83 cases; d3 and d3m answer 95 through their
wider domain. **The comparator d3m scored better than d3 on the test days**: 0.0982 against 0.1389 on
extinction and 0.0419 against 0.0535 on operator termination. d3m differs from d3 only on methane
flames (section 7).

**Development, in-sample** (train and validation days together, after both had been read;
SpaceApps-Ardealu:docs/wsp/feedback/bre_d3_result.md; on the 327 cases W0 answers):

| | W0 by `flame_end` | W0 by `lasts` | **d3 by `lasts`** | d3m by `lasts` | planned stop | incidence |
|---|---|---|---|---|---|---|
| extinction, pooled | 0.2576 | 0.1955 | **0.1121** | 0.0980 | 0.2269 | 0.2293 |
| extinction, train (240) | 0.2511 | 0.1909 | 0.1226 | 0.1034 | 0.2320 | 0.2300 |
| extinction, validation (87) | 0.2755 | 0.2083 | 0.0833 | 0.0833 | 0.2129 | 0.2274 |
| operator termination, pooled | 0.0730 | 0.0577 | 0.0398 | 0.0358 | 0.0699 | 0.0952 |

d3 is lower than W0 by `flame_end` on 13 of 14 days and than W0 by `lasts` on 14 of 14. On the 33 cases
only d3 admits (its wider domain), extinction 0.1288 against 0.2414 (planned stop) and 0.2253
(incidence).

**Blind look of the zero-shot parent** (validation days, 87 cases, PROTOCOL 13.2, before any validation
case was read; SpaceApps-Ardealu:docs/wsp/feedback/bre_w0_result.md): W0 by `flame_end` extinction
0.2755 against 0.2274 (incidence) and 0.2129 (planned stop); operator termination 0.1383 against 0.1799
and 0.0924. d3 has no blind validation number: it was typed after the validation days became study
data.

## 7. Adoption decision and lineage

**Why d3 is served.** By PROTOCOL 13.6 a candidate is kept if it is lower than the current model pooled
and on at least two thirds of the days whose forecasts change; where fewer than six days change, it is
carried as a reported comparator, not as the model. d3 scored 0.1121 against 0.2576, lower on 13 of 14
days: "kept: the model". d3m changes the forecast of one day only (the methane day 20339, 0.1921 to
0.0386), so by the same rule it is a comparator (bre_d3_result.md; PROTOCOL 13.6, "further outcomes").
The author proposed it as such: "One day of evidence; the test days hold a methane day, which will say
whether it carries" (phys_bre_study.md, section 4). The frozen release list of the final run named d3,
read through `lasts`, as the BRE model and d3m as comparator 5 (PROTOCOL 13.7), and after the run
PROTOCOL 13.7 lists BRE d3 among what is served and closes the generation ("no number of any release is
changed from this run"). No source read for this page records a decision about d3m after the run; a
switch chosen after seeing the test days would make their number no longer an unbiased estimate for
that choice, as PROTOCOL 13.7 labels the two choices it did make after the run (CLD Flame D1, plastics
bass1 D1).

**Lineage.**

| Step | Release | Candidate | What it did |
|---|---|---|---|
| root | `sha256:2d0fae57312f2aa953cb4e218cba6bd84948282174ca90d21ebd9034a6378948` | `configs/models/law/bre/root` | neutral root of slug `bre` (no claim) |
| W0 | `sha256:aa0eef5bc7653922c7a7838796dab6946294e89f79c0b6c500124d9a4b212b4d` | `bre/w0-fable` | zero-shot law, section 7.4 of `phys_bre_w0.md`; registered reading through `flame_end` |
| D1 (not adopted) | `sha256:3cffa05ad9dcd29e1fc5b422474e30e2c07c6d5909ea944b77a12d9c695c0f76` | `bre/d1-fable` | PROTOCOL 13.3: pressure exponent of the critical flux 0.5 to 0.8; validation extinction 0.2866, not lower than W0's 0.2755; W0 stayed (`bre_d1_result.md`) |
| d2 (not adopted) | `sha256:429f47431773d02e9608a2fdfd9e24ec4af171ffe9bea98400beb44ea4f18340` | `bre/d2-fable` | PROTOCOL 13.4: the same pressure step, scored through `lasts`; 0.1800 pooled, lower on 9 of 14 days: not kept (`bre_second_law_result.md`) |
| **d3 (served)** | `sha256:27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955` | `bre/d3-fable` | PROTOCOL 13.6: burner share at the extinction temperature, no pressure term, clock 0.42, spread 0.31; read through `lasts` |
| d3m (comparator) | `sha256:4b7d155ce91823425c6bf52c62f8d96318601ec60db13be1e3818664b55c2a9f` | `bre/d3m-fable` | d3 plus the methane critical flux 1.7 times ethylene's (16 elements); a revision of W0 |

W0 read through `lasts` alone (0.1955, 9 of 14 days) was also scored in PROTOCOL 13.4 and not kept by
the rule.

**Scope.**

- Only the `lasts` target is the model: its `flame_end` target disagrees with it (by up to 0.95 in
  probability, `bre_d3_result.md`) and is not served; `height_at_end` is not served either.
- Targets served: `lasts`; event targets would need `settings.times_s` or `horizon_s`.
- The probability that the flame has gone out by time $h$ is $1 - \texttt{lasts}(h)/\texttt{lasts}(\mathrm{cutoff})$
  (phys_bre2.md, section 2): forecast `lasts` at the cutoff and at $h$ through `stated_time`.
- The law is for the first constant-flow segment; a flow changed during the test is outside it.
- The 50 mm burner lives 1.3 to 2.3 times longer than the law on two development days; methane rests
  on one development day; whether a small flame stands at the limit is close to a coin toss
  (phys_bre_study.md, section 6).

## 8. How to call it in IGNIS

- **Forecast lab** (web, route `#/lab`): the Models screen `#/models` lists the served releases and
  opens the model card; `web/src/views/LabModels.tsx` runs a typed scenario (stored runs are refused for
  this release).
- **Model card**: `GET /api/v1/model-cards` (fields from `configs/models/served/served.json`, including
  the cite string).
- **MCP tools**: `models_inspect`, `models_forecast`.
- **API.** Admit a typed scenario, then forecast it:

`POST /api/v1/models/sha256:27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955/scenarios`

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955",
  "cutoff_s": 10.0,
  "phase_established_at_s": 0.0,
  "conditions": {
    "burner_diameter": 0.025, "burner_mass_flux": 0.004, "x_o2": 0.265,
    "pressure": 101325.0, "x_fuel": 1.0, "is_methane": 0.0, "stated_time": 60.0
  },
  "controls": [],
  "observations": []
}
```

These are typed values (the law's centre from `edits.wsp`: pure ethylene on the 25 mm burner at
4 g/m2/s, 26.5 % oxygen, 1 atm), not a measurement. The response carries `input_hash`. Then:

`POST /api/v1/analyses`

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955",
  "input": {"kind": "prefix", "input_hash": "<input_hash>"},
  "settings": {"target": "lasts"}
}
```

It returns a job: poll `GET /api/v1/jobs/<job_id>` and read `GET /api/v1/artifacts/<artifact_id>`, an
`AnalysisArtifact` with evidence class `model_output`; `P(yes)` is the probability that the flame is
still burning 60 s after ignition. The only `settings.target` is `lasts`. For the chance that a flame
burning at 10 s has gone out by 60 s, admit a second scenario identical but for `stated_time: 10.0`,
forecast it, and combine: $1 - \texttt{lasts}(60)/\texttt{lasts}(10)$ (with a planned stop before the
horizon, use the stop as $h$).

Cite: `bre.wsp D3, hand-written (literal WSP) release sha256:27e2af51…b955, experimental, not trained; its output is model output, not a measurement.`

## Sources

- `configs/models/served/served.json`; `configs/models/served/models/27e2af51340e7ba2a8ba73894abcec1c1342e8116834eda1aa7a0a5454d7b955/release.json` and `params.npz`
- `configs/models/law/bre/{d3-fable,d3m-fable,w0-fable}/{edits.wsp,meta.json,release.json}`; `configs/models/law/bre/{root,d1-fable,d2-fable}/release.json`; `configs/models/law/bre/binding.json`
- `crates/ignis-wsp/src/law.rs`, `crates/ignis-wsp/src/law_slugs.rs`, `crates/ignis-server/src/v1/wsp.rs`, `crates/ignis-core/src/contracts/research.rs` (MCP tool descriptions), `tools/wsp/verify_served.py`
- SpaceApps-Ardealu:docs/wsp/research/law_forward.md, law_forward_targets.md (lane branch feat/handwritten-science, 542dad8)
- SpaceApps-Ardealu:docs/wsp/research/phys_bre_w0.md, phys_bre_d1.md, phys_bre2.md, phys_bre_study.md
- SpaceApps-Ardealu:docs/wsp/feedback/gen7_failure_analysis.md, bre_w0_result.md, bre_d1_result.md, bre_second_law_result.md, bre_d3_result.md
- SpaceApps-Ardealu:docs/wsp/PROTOCOL.md (sections 13.3, 13.4, 13.6, 13.7), RESULTS_FINAL_GEN7.md, COVERAGE_19.md
