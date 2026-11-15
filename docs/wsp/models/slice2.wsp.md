# slice2.wsp — lifting and blow-off of a microgravity jet flame (SLICE)

`slice2.wsp` forecasts, for one logged setting of a laminar jet flame in coflow aboard the ISS (SLICE,
PSI-106), whether the flame blows off on that line, whether its base is lifted, and its luminous
length. Its contribution is the lifting forecast: lifting is a window between two limits that the crew
writes down only when they note it, and that one change cut the lifting Brier score from 0.1474 (the
zero-shot W0) to 0.0333 on the development lines, lower on 8 of 8 days, and from 0.0466 to 0.0041 on the
unread test days, whose 31 lines hold no lifting or blow-off event, so that test says little (SpaceApps-Ardealu:docs/wsp/feedback/slice2_second_law_result.md;
SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md). It is data-informed: written by `claude-fable-5-2`
from the train and validation days. Release `sha256:d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61`.

| | |
|---|---|
| Model | `slice2.wsp` |
| Investigation | SLICE (PSI-106), lifted and blown-off jet flames in the Microgravity Science Glovebox |
| Family | `other` (generic law slug `slice2`) |
| Candidate | S1 (`configs/models/law/slice2/s1-fable`) |
| Generation | seventh (15 November 2026), PROTOCOL 13.4 (second law) |
| Author | `claude-fable-5-2`; transcribed by the assembly engineer `claude-opus-5-5` |
| Authorship | data-informed (train and validation days) |
| Release | `sha256:d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61` |
| Forward | `ignis_wsp::law::forecast`, `crates/ignis-wsp/src/law.rs` (version 1, profile `strict_no_fitting`) |
| Parent release | `sha256:e3d6454b1d5762f8634555458ce5bfb30e83c2289120bd956986050f09e23b3d` (neutral root of slug `slice2`); the numbers are those of slice W0 `sha256:d10c94bc6ea0bcb957f89734393b0cbabbd44fdf0ac9c2e440f051a33cd3006e` plus the new lines |

## 1. Task and targets

Inputs (all typed conditions, all required; `configs/models/served/served.json`, declaration
`configs/models/law/slice2/decl.json`):

| Name | Kind | SI unit | Transform | Note |
|---|---|---|---|---|
| `u_coflow` | condition | m/s | ln | duct air velocity (`u_coflow_cm_s / 100`) |
| `u_jet` | condition | m/s | ln | mean jet exit velocity |
| `burner_id` | condition | m | ln | burner tube inner diameter (`burner_id_mm / 1000`) |
| `x_fuel` | condition | 1 | ln | hydrocarbon mole fraction of the jet gas |
| `x_o2` | condition | 1 | ln | cabin oxygen mole fraction |
| `is_ethane` | condition | 1 | identity | 1 for ethane |
| `is_propane` | condition | 1 | identity | 1 for propane (methane is the reference) |
| `mdot_jet` | condition | kg/s | ln | jet gas mass flow (`mdot_jet_mg_s / 1e6`); read by `flame_length` only |

Targets (`release.json`, `observation_model.targets`):

| Target | Kind | Unit | Meaning |
|---|---|---|---|
| `blow_off` | binary | none | the flame blows off at this logged setting (per-line hazard, as in `slice`) |
| `past_lift` | binary | none | the condition is past the lifting limit (what W0 called `lifted`) |
| `past_blow` | binary | none | the condition is past the blow-off limit, cumulative |
| `lift_noted` | binary | none | a lifted flame is written down as lifted; no feature dependence |
| `flame_length` | positive | m | luminous length (lane-local: `quantity = null`, label `slice2:flame_length`) |

The law has no time dependence. Each answer is returned at `times_s = [cutoff_s]`: binary targets as
`event_probabilities` with categories `yes`/`no`, the length as quantiles
(SpaceApps-Ardealu:docs/wsp/research/law_forward.md, section 2.4). There is no stored run of this
investigation carrying the declared features (`stored_runs: false`), so every call is a typed
scenario that gives all eight conditions; a value outside the declared domain is refused with its
name.

## 2. Physical mechanism

The base of a jet diffusion flame is a small premixed kernel about a millimetre off the rim. It holds
while the gas speed it feels, `v_k`, is not much more than the speed at which a flame edge burns
upstream, about the laminar burning speed `S_L`. Two things raise `v_k`: the coflow (sheltered by the
rim boundary layer, factor 0.66) and, on small tubes, the wall shear of the fuel jet. Nitrogen in the
fuel lowers `S_L`; cabin oxygen raises it roughly as the cube
(SpaceApps-Ardealu:docs/wsp/research/phys_slice_w0.md, sections 0 and 1):

```math
v_k = 0.66\,U_{co} + 8\,l_j\,\frac{Z_{st}}{Z_{st,\mathrm{CH_4}}}\,\frac{U_j}{d},\qquad l_j = 3.3\ \mu\mathrm{m},\qquad \Phi = \frac{v_k}{1.4\,S_L}
```

Lifted when $\Phi \gt1$, blown off when $\Phi \gt1.3$, with $\ln\Phi$ known to 0.5 (one standard
deviation). In the network's features ($z_{co}=\ln(U_{co}/0.3)$, $z_j=\ln U_j$, $`z_d=\ln(d/1.6\,\mathrm{mm})`$,
$z_x=\ln x_{fuel}$, $z_o=\ln(x_{O_2}/0.21)$, $e_2$, $e_3$ the fuel indicators):

```math
\ln\Phi = -0.987 + z_{co} - 0.55 z_x - 3.0 z_o - 0.076(e_2+e_3) + \mathrm{sp}(t) + 0.5\,\mathrm{relu}(-0.693 - z_x)
```

```math
t = -2.485 - z_{co} + z_j - z_d - 1.3 z_x + 0.065 e_2 + 0.088 e_3,\qquad \mathrm{sp}(t) = 0.31\,\mathrm{relu}(t+2.2) + 0.38\,\mathrm{relu}(t) + 0.31\,\mathrm{relu}(t-2.2)
```

`sp` is $\ln(1+e^t)$ in three pieces (the jet part added to the coflow part); it is carried by hinge
units 0 to 2 of every logit network, and the steeper fall of `S_L` below 50 % fuel by hinge unit 3.

- `past_lift.logit` $= 3.4\ln\Phi$ (slope $1.7/0.5$: a logistic as wide as the 0.5 uncertainty).
- `past_blow.logit` $= 3.4(\ln\Phi - \ln 1.3)$: the same network 0.892 lower.
- `blow_off.logit` $= -2.184 + 2.0\ln\Phi$: the crew stepped the flow up line by line, so the chance of
  blow-off on a line is the chance that the limit lies inside one step of about 10 % (section 1.6 of
  the W0 dossier).
- `flame_length`: $`\ln L = \ln(K\dot m / Z_{st}) + 0.148\,\mathrm{clip}(\ln(0.5\,U_j/U_{co}), 0, 3.22)`$,
  $K$ about 1.30 mm per mg/s.

The second law (SpaceApps-Ardealu:docs/wsp/research/phys_slice2.md) adds two facts about the log. A
flame past the blow-off limit is not there to be lifted, so lifting is the window $1 \lt \Phi \lt 1.3$
conditional on the flame being present; and the crew wrote "lifted" only when they noticed it, typed as
a chance of 0.5:

```math
p_{\mathrm{lifted}} = p_{\mathrm{lift\_noted}}\;\frac{p_{\mathrm{past\_lift}} - p_{\mathrm{past\_blow}}}{1 - p_{\mathrm{past\_blow}}}
```

Since `past_blow` is the `past_lift` network with a lower offset, $`p_{past\_lift} \ge p_{past\_blow}`$
and the result lies in [0, 1]. This combination is applied at scoring time, outside the forward.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section slice2.wsp.

## 3. Authoring

- **Who.** Author `claude-fable-5-2` (`release.json`, `construction.author`). The release record says
  it is "the parent's block with the dossier's lines, verbatim. Transcribed by the assembly engineer
  (claude-opus-5-5); no number chosen" (`configs/models/law/slice2/s1-fable/meta.json`, `rationale`).
- **When and which step.** Seventh generation, 15 November 2026; PROTOCOL 13.4, "What went wrong, and
  second laws" (SpaceApps-Ardealu:docs/wsp/PROTOCOL.md). A declaration is frozen with its release, so
  a second law is a new slug, `slice2`.
- **What was read.** Exposure (`release.json`, `construction.exposure`): datasets
  `slice2-train-and-validation-cases`, partitions `train` and `validation`; development feedback
  `PROTOCOL-13.2-and-13.3-validation-aggregates` and `docs/wsp/feedback/gen7_failure_analysis.md`;
  source `docs/wsp/research/phys_slice2.md` (sha256
  `333af88eb8ebed9988827578829fc7f74b53f71e1fe0807ffa01e36e1e343906`). The dossier states it was written
  from the train and validation days "read through" the failure analysis, with no test day and no
  per-case file.
- **Brief.** The physics brief and the author prompts are collected in
  [docs/research/PROMPTS.md](../../research/PROMPTS.md). The author worked from
  SpaceApps-Ardealu:docs/wsp/research/phys_slice_w0.md (the W0 law) and wrote
  SpaceApps-Ardealu:docs/wsp/research/phys_slice2.md.
- **Decisions.** The failure analysis showed W0 over-forecasting lifting by a factor of about 7
  everywhere (train: forecast 0.277 against 0.030 observed; 0.4 mm tube: 0.637 against 0.064), because
  "lifted" is rare and written only when noted
  (SpaceApps-Ardealu:docs/wsp/feedback/gen7_failure_analysis.md, "slice lifted"). Its one proposed
  change was a window between the lifting and blow-off limits plus a label prior, expressible as two
  targets combined at scoring time. The author therefore:
  1. kept every W0 line, renaming each `lifted.logit.*` line to `past_lift.logit.*` with the same value;
  2. added `past_blow.logit.*`, the same network with `b0` lowered by $3.4\ln 1.3 = 0.892$;
  3. added `lift_noted.logit.b0 = 0.0` (probability 0.5), going "half-way in ln" on the residual
     over-forecast of about 3.5;
  4. left `blow_off` and `flame_length` untouched, and did not add the line's position or step size
     (the window already uses the fact that the flame survived).

## 4. Weights

Every number is in `configs/models/law/slice2/s1-fable/edits.wsp`; the full list is printed by
`ignis wsp inspect --release sha256:d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61 --edits`.
Feature columns are `(z_co, z_j, z_d, z_x, z_o, e2, e3, z_m)`.

| Group | Shape (nonzero) | Values | Physical meaning |
|---|---|---|---|
| `x.u_coflow.{centre,scale,lo,hi}` | 4 scalars (4) | 0.3, 1.0, 0.02, 1.5 | coflow centre 0.3 m/s; domain 2 to 150 cm/s |
| `x.u_jet.*` | 4 (4) | 1.0, 1.0, 0.02, 100.0 | jet speed centre 1 m/s; domain 0.02 to 100 m/s |
| `x.burner_id.*` | 4 (4) | 0.0016, 1.0, 0.0003, 0.004 | tube centre 1.6 mm; domain 0.3 to 4 mm |
| `x.x_fuel.*` | 4 (4) | 1.0, 1.0, 0.3, 1.0 | fuel fraction 0.3 to 1 |
| `x.x_o2.*` | 4 (4) | 0.21, 1.0, 0.18, 0.25 | cabin oxygen 18 to 25 % |
| `x.is_ethane.*`, `x.is_propane.*` | 8 (4) | centre 0, scale 1, lo 0, hi 1 | fuel indicators (methane reference) |
| `x.mdot_jet.*` | 4 (4) | 1e-6, 1.0, 2e-8, 5e-5 | jet mass flow centre 1 mg/s; domain 0.02 to 50 mg/s |
| `past_lift.logit.b0` | [] (1) | -3.356 | $3.4\ln\Phi$ at the centre ($\ln\Phi = -0.987$: pure methane, 0.3 m/s coflow) |
| `past_lift.logit.skip` | [8] (5) | [3.4, 0, 0, -1.87, -10.2, -0.258, -0.258, 0] | 3.4 times the linear part of $\ln\Phi$: coflow +1, dilution -0.55, oxygen -3.0, ethane/propane -0.076 |
| `past_lift.logit.W1` rows 0-2 | [8, 8] (19 with row 3) | each [-1.0, 1.0, -1.0, -1.3, 0, 0.065, 0.088, 0] | the jet-to-coflow ratio $t$ |
| `past_lift.logit.W1[3, 3]` | | -1.0 | dilution below 50 % fuel |
| `past_lift.logit.b1` | [8] (4) | [-0.285, -2.485, -4.685, -0.693, 0...] | hinge knots: $t$ = -2.2, 0, 2.2 (offset -2.485), $z_x = \ln 0.5$ |
| `past_lift.logit.w2` | [8] (4) | [1.054, 1.292, 1.054, 1.7, 0...] | $3.4 \times (0.31, 0.38, 0.31, 0.5)$ |
| `past_blow.logit.*` | 89 elements (33) | as `past_lift`, `b0` = -4.248 | the blow-off limit $\Phi = 1.3$: `past_lift` minus 0.892 |
| `blow_off.logit.b0` | [] (1) | -4.158 | $-2.184 + 2.0 \times (-0.987)$ |
| `blow_off.logit.skip` | [8] (5) | [2.0, 0, 0, -1.10, -6.0, -0.152, -0.152, 0] | 2.0 times the linear part of $\ln\Phi$ |
| `blow_off.logit.W1`, `b1` | (19), (4) | same rows and knots as `past_lift` | the jet term and the dilution hinge |
| `blow_off.logit.w2` | [8] (4) | [0.62, 0.76, 0.62, 1.0, 0...] | $2.0 \times (0.31, 0.38, 0.31, 0.5)$ |
| `lift_noted.logit.*` | 89 elements (0) | all zero (`b0` = 0.0) | a lifted flame is noted half of the time ($p = 0.5$) |
| `flame_length.median.b0` | [] (1) | -3.752 | $\ln(1296) + \ln(10^{-6}) - \ln(0.0552)$: 23.5 mm per mg/s of methane in strong coflow |
| `flame_length.median.skip` | [8] (4) | [0, 0, 0, 1.3, 0, -0.065, -0.088, 1.0] | length proportional to $\dot m$; $Z_{st}$ rises with dilution (1.3 per e-fold), ethane, propane |
| `flame_length.median.W1`, `b1`, `w2` | rows 0, 1: [-1, 1, 0...]; b1 [0.511, -2.708]; w2 [0.148, -0.148] | (4), (2), (2) | $`\ln F = 0.148\,\mathrm{clip}(\ln(0.5U_j/U_{co}), 0, 3.219)`$, 0.511 at the centre |
| `flame_length.spread.b0` | [] (1) | 0.22 | ln-sd of the length (other spread tensors zero) |
| `flame_length.tail_weight`, `tail_width` | 2 scalars (2) | 0.1, 3.0 | 10 % of cases three times wider |
| `num.substeps` | constant | 512 | integration steps (numerical, not physics; unused by these targets) |

Totals (`release.json`, `architecture.parameter_counts`): 568 authored elements, 143 nonzero,
1 constant.

## 5. Forward and assembly

| Field | Value |
|---|---|
| Forward | `crates/ignis-wsp/src/law.rs`, entry point `ignis_wsp::law::forecast`, version 1, profile `strict_no_fitting` |
| `forward_hash` | `sha256:dbf4c795d70c8c28ac5be9686bd98297890fef33237666f50d1415063f4a01d4` (the slug's declaration is one of its forward sources) |
| `code_version` | `ignis-wsp@0.1.0+fwd.dbf4c795d70c8c28` |
| Spec | `law-slice2-v1`, `sha256:0aeda7655f9b7839e195771b13e007b648e0e2ccadf4d28f991b9c5b09d43a94` |
| Patch | `law-slice2-v1-s1-fable`, `sha256:230965dbad5d87b2af298b013f145e1288b6780a799a8dd7b8cbb97eefeeb9a6` |
| `weights_hash` | `sha256:45e749c2a129a61894aac4399381f6aeb1df9c8e3ba078810954d726d17ee5ea` |
| Binding | cohort `atlas:PSI-106:catalogue`, split `sha256:ea8c6391851b2412fc4b58179c272845a635049024cbcda6676569c0e10fdc65`, source manifest `sha256:a163c5db364a3f3e5172e5b45541a0ad424c242233c068de335d2b613bfee78c` (`configs/models/law/slice2/binding.json`) |
| Candidate directory | `configs/models/law/slice2/s1-fable` (`edits.wsp`, `meta.json`, `patch.json`, `release.json`) |
| Served record | `configs/models/served/models/d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61/` |
| Checks | `spec_valid`, `patch_valid`, `tensors_complete`, `strict_forward_audit`, `finite_forward_on_fixture`: passed |

Rebuild (the `--root` registry must hold the parent, the `slice2` neutral root, as
`models/e3d6454b1d5762f8634555458ce5bfb30e83c2289120bd956986050f09e23b3d/`; its record is
`configs/models/law/slice2/root/release.json`):

```
cargo build --release -p ignis-cli
ignis wsp compile  --family slice2 --edits configs/models/law/slice2/s1-fable/edits.wsp --meta configs/models/law/slice2/s1-fable/meta.json --out configs/models/law/slice2/s1-fable/patch.json
ignis wsp assemble --family slice2 --patch configs/models/law/slice2/s1-fable/patch.json --binding configs/models/law/slice2/binding.json --root <registry holding models/e3d6454b…3b3d/> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the record; `served_ids` pins the 14
served ids. `cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at
ignis-web a58c7c3: the served id is the content hash of its record and loads through the registry
checks.

## 6. Evaluation

Metric: Brier score per logged line, lower is better. "Lifted" is scored through the rule of section 2.

**Headline: lifting.** On the development lines the window cut the lifting Brier score by a factor of
4.4 against its W0 parent, lower on every day; on the test days the false-alarm level fell by a factor
of 11.

**Development, in-sample** (train and validation days together, after both had been read;
SpaceApps-Ardealu:docs/wsp/feedback/slice2_second_law_result.md; mean of day means, pooled line mean in
brackets; 302 lines; the 41 lines of the ethylene day 075 are refused by every release):

| Target | Partition | slice W0 (parent block) | **slice2 S1** | incidence pooled | incidence by tube | never |
|---|---|---|---|---|---|---|
| lifted | pooled | 0.1474 (0.1695) | **0.0333 (0.0368)** | 0.0297 | 0.0293 | 0.0306 |
| lifted | train | 0.1559 (0.1798) | 0.0305 (0.0346) | 0.0258 | 0.0249 | 0.0265 |
| lifted | validation | 0.0886 (0.0886) | 0.0536 (0.0536) | 0.0563 | 0.0596 | 0.0588 |
| blow_off | pooled | 0.1089 (0.1151) | 0.1089 (0.1151) | 0.0509 | 0.0480 | 0.0538 |

Lifting is lower than W0 on 8 of 8 days (day 061: 0.7116 to 0.1087; day 069: 0.1090 to 0.0095). On the
validation day alone S1 (0.0536) is below all three baselines. `blow_off` is identical to W0 on every
line, by construction.

**Second case version** (tube entries corrected from the investigators' picture table, no new release;
SpaceApps-Ardealu:docs/wsp/feedback/slice_cases_v2_result.md; development, in-sample): S1 blow-off
0.0456 against 0.0473 for the by-tube incidence and 0.0509 for the pooled incidence, now below every
baseline; lifted 0.0280.

**Blind looks of the parent** (validation day 065, 34 lines, PROTOCOL 13.2, before any validation case
was read; SpaceApps-Ardealu:docs/wsp/feedback/slice_w0_result.md, COVERAGE_19.md): W0 blow-off 0.1548
against 0.1605 (pooled incidence) and 0.1493 (by tube); lifting 0.0886 against 0.0563. S1 itself has no
blind validation number: it was typed after the validation days became study data.

**Final run on the unread test days** (PROTOCOL 13.7, run once on the frozen release list sha256
`f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2`, outcome coding v2; 31 lines;
SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md, section PSI-106):

| Release | Role | blow_off: Brier, mean of day means | lifted: Brier, mean of day means |
|---|---|---|---|
| **slice2 s1 (second law)** | model | **0.0021** | **0.0041** |
| slice W0 | comparator | 0.0021 | 0.0466 |
| slice D1 | comparator | 0.0021 | 0.0464 |
| slice2 S2 | comparator | 0.0565 | 0.0041 |
| incidence pooled (simple) | baseline | 0.0028 | 0.0008 |
| incidence by nominal tube | baseline | 0.0000 | 0.0000 |
| never (persistence) | baseline | 0.0000 | 0.0000 |

The 31 test lines hold no blow-off and no lifting event, so a baseline that never forecasts the event
scores zero and the test says little about either target (PROTOCOL 13.7, "Outcome of 13.7"). What it
does show is the false-alarm level: S1 forecasts lifting ten times less often than W0 where nothing
lifted, and its blow-off forecast (W0's) is below the pooled incidence.

## 7. Adoption decision and lineage

**Why this release is served.** PROTOCOL 13.4 keeps a second law for the final run if it is lower than
the first law on the pooled development cases and on at least two thirds of the days. `slice2` scored
0.0333 against 0.1474 on lifting, lower on 8 of 8 days, with blow-off unchanged: "kept, and now the
SLICE model" (SpaceApps-Ardealu:docs/wsp/PROTOCOL.md, 13.6, "Outcome of 13.4";
COVERAGE_19.md, "Second laws"). It went into the frozen release list of the final run as the SLICE model
(PROTOCOL 13.7) and is listed among what is served after the run.

**Lineage.**

| Step | Release | Candidate | What it did |
|---|---|---|---|
| root | `sha256:a8a09553667e1d47ea850c4506a3078039a280a5060d6aed48235de88bece484` | `configs/models/law/slice/root` | neutral root of slug `slice` (no claim) |
| W0 | `sha256:d10c94bc6ea0bcb957f89734393b0cbabbd44fdf0ac9c2e440f051a33cd3006e` | `slice/w0-fable` | zero-shot law, section 7.4 of `phys_slice_w0.md` |
| D1 (not adopted) | `sha256:c40cd91b76476c153c7b677adf9adca3feb244ae582f4b361307212d23f6b0d8` | `slice/d1-fable` | PROTOCOL 13.3: removed the $Z_{st}$ factor of the jet term (six `W1[.,3]` elements, -1.3 to 0); blow-off on validation 0.1631, worse than W0's 0.1548 (`slice_d1_result.md`); W0 stayed |
| root | `sha256:e3d6454b1d5762f8634555458ce5bfb30e83c2289120bd956986050f09e23b3d` | `slice2/root` | neutral root of the new slug `slice2` |
| **S1 (served)** | `sha256:d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61` | `slice2/s1-fable` | W0's block under the new target names, plus `past_blow` and `lift_noted` |
| S2 (comparator) | `sha256:0f5763f5749bb669b6ed9b70ac2629c8923838b3308f6094b2efcd7d8313388a` | `slice2/s2-fable` | PROTOCOL 13.6: a blow-off boundary re-read from the development lines, with a saturating jet part; 0.0583 against 0.1089 pooled on blow-off but lower on 5 of 8 days, short of two thirds: a comparator; on the second case version it is worse than S1 (0.0511 against 0.0456) and was dropped (`slice2_s2_result.md`, `slice_cases_v2_result.md`) |

In the record the parent of S1 is the `slice2` root (a new slug cannot inherit a release of another
slug); its numbers are W0's, as `edits.wsp` and `phys_slice2.md` state.

**Scope.**

- Judged on very few events: the 31 test lines hold no blow-off or lifting event.
- Targets served: `blow_off`, `past_lift`, `past_blow`, `lift_noted`, `flame_length`; one per call,
  chosen with `settings.target` (default the first).
- The probability of a logged "lifted" is not a target: compute
  `lift_noted x (past_lift - past_blow) / (1 - past_blow)` from three forecasts.
- `blow_off` is W0's blind forecast; on the first case version it scores above its baselines on the
  development lines (0.1089 against 0.0480).
- `flame_length` is not scorable from the log (lengths exist only in images) and has no score.
- Methane, ethane and propane only; the 20 % ethylene lines of day 075 are refused (no ethylene
  indicator in the declaration).

## 8. How to call it in IGNIS

- **Forecast lab** (web, route `#/lab`): the Models screen `#/models` lists the served releases and
  opens the model card; `web/src/views/LabModels.tsx` runs a typed scenario (stored runs are refused for
  this release).
- **Model card**: `GET /api/v1/model-cards` (fields from `configs/models/served/served.json`, including
  the cite string).
- **MCP tools**: `models_inspect`, `models_forecast`.
- **API.** Admit a typed scenario, then forecast it:

`POST /api/v1/models/sha256:d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61/scenarios`

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61",
  "cutoff_s": 1.0,
  "phase_established_at_s": 0.0,
  "conditions": {
    "u_coflow": 0.3, "u_jet": 1.0, "burner_id": 0.0016, "x_fuel": 1.0,
    "x_o2": 0.21, "is_ethane": 0.0, "is_propane": 0.0, "mdot_jet": 0.000001
  },
  "controls": [],
  "observations": []
}
```

These are typed values (the law's centre from `edits.wsp`: pure methane on the 1.6 mm tube), not a
measurement. The response carries `input_hash`. Then:

`POST /api/v1/analyses`

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61",
  "input": {"kind": "prefix", "input_hash": "<input_hash>"},
  "settings": {"target": "past_lift"}
}
```

It returns a job: poll `GET /api/v1/jobs/<job_id>` and read `GET /api/v1/artifacts/<artifact_id>`, an
`AnalysisArtifact` with evidence class `model_output`. `settings.target` takes `blow_off`, `past_lift`,
`past_blow`, `lift_noted` or `flame_length` (with optional `quantile_levels`, default
`[0.05, 0.25, 0.5, 0.75, 0.95]`); no target needs `times_s`. For the probability that the line is logged
as lifted, run `past_lift`, `past_blow` and `lift_noted` on the same `input_hash` and combine them with
the rule above.

Cite: `slice2.wsp S1, hand-written (literal WSP) release sha256:d586d0d3…0b61, experimental, not trained; its output is model output, not a measurement.`

## Sources

- `configs/models/served/served.json`; `configs/models/served/models/d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61/release.json` and `params.npz`
- `configs/models/law/slice2/s1-fable/{edits.wsp,meta.json,release.json}`; `configs/models/law/slice2/{root,s2-fable}/release.json`; `configs/models/law/slice2/binding.json`; `configs/models/law/slice/{root,w0-fable,d1-fable}/release.json`
- `crates/ignis-wsp/src/law.rs`, `crates/ignis-wsp/src/law_slugs.rs`, `crates/ignis-server/src/v1/wsp.rs`, `crates/ignis-core/src/contracts/research.rs` (MCP tool descriptions), `tools/wsp/verify_served.py`
- SpaceApps-Ardealu:docs/wsp/research/law_forward.md, law_forward_targets.md (lane branch feat/handwritten-science, 542dad8)
- SpaceApps-Ardealu:docs/wsp/research/phys_slice_w0.md, phys_slice_d1.md, phys_slice2.md, phys_slice_study.md
- SpaceApps-Ardealu:docs/wsp/feedback/gen7_failure_analysis.md, slice_w0_result.md, slice_d1_result.md, slice2_second_law_result.md, slice2_s2_result.md, slice_cases_v2_result.md
- SpaceApps-Ardealu:docs/wsp/PROTOCOL.md (sections 13.4, 13.6, 13.7), RESULTS_FINAL_GEN7.md, COVERAGE_19.md
