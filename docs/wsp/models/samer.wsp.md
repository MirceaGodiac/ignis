# samer.wsp — SAME-R smoke size and ISS detector

`samer.wsp` forecasts the smoke of overheated spacecraft materials on SAME-R (PSI-101): the count mean
diameter `d10` and the diameter of average mass `d30` at the age the smoke is measured, and whether
the ISS detector's scatter channel is at full scale. On the unread test days of the seventh
generation, scored once on a frozen list, it scores **0.167** on particle diameter (CRPS of ln d10)
against 0.256 for the best baseline and 0.260 for its zero-shot parent, and **0.014** on the detector
(Brier) against 0.019 and 0.081: it holds on both targets
(`SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md`). Data-informed revision S1 of the zero-shot W0,
typed from a case-by-case study of the development days. Release
`sha256:e33ee7665b599e7e6c8c978f61ebf3bf106b2b6a94c3a0821dbdf42cf47a1b5f`.

| | |
|---|---|
| Model | `samer.wsp` |
| Investigation | SAME-R (PSI-101), smoke aerosol in the Microgravity Science Glovebox |
| Family | `other` (generic law slug `samer`) |
| Candidate | S1 (`configs/models/law/samer/s1-fable`) |
| Generation | seventh (PROTOCOL 13.6 study candidate; judged in the final run of 13.7) |
| Author | `claude-fable-5-2`; transcribed by the assembly engineer `claude-opus-5-5` |
| Authorship | data-informed (train and validation days) |
| Release | `sha256:e33ee7665b599e7e6c8c978f61ebf3bf106b2b6a94c3a0821dbdf42cf47a1b5f` |
| Forward | `ignis_wsp::law::forecast`, `crates/ignis-wsp/src/law.rs` (profile `strict_no_fitting`, version 1) |
| Parent release | `sha256:05f77ae2f7a666e0da69d04fa29d9ad8b0ac15790645abe1adc0149292327fb0` (`samer` W0, `configs/models/law/samer/w0-fable`) |

## 1. Task and targets

Every input is a typed condition (`configs/models/served/served.json`, entry `samer.wsp`; domains from
the release tensors):

| Input | Kind | SI unit | Note |
|---|---|---|---|
| `is_kapton` | condition | 1 | material indicator (0 or 1) |
| `is_teflon` | condition | 1 | material indicator |
| `is_silicone` | condition | 1 | material indicator |
| `is_pyrell` | condition | 1 | material indicator; all four at 0 is lamp wick, the reference |
| `sample_temperature` | condition | K | sample temperature during heating; 450 to 950 |
| `inlet_velocity` | condition | m/s | nominal duct air speed during heating; 0 to 0.12 |
| `no_flow` | condition | 1 | 1 when the inlet velocity is 0 |
| `net_aging_time` | condition | s | the horizon: age of the smoke when the row was measured; 5 to 5000 |
| `heating_duration` | condition | s | heating time of the sample; 20 to 300 |

| Target | Kind | Unit | Meaning |
|---|---|---|---|
| `d10` | positive | m | count mean diameter M1/M0 at `net_aging_time` (lane-local, `samer:d10`) |
| `d30` | positive | m | diameter of average mass (lane-local, `samer:d30`; not scored) |
| `iss_full_scale` | binary | none | ISS detector scatter channel at full scale, \|V\| >= 6.0 V |

The age of the smoke is an input; a positive target is returned at `times_s = [cutoff_s]`, the binary
target as `yes`/`no` probabilities. `stored_runs` is `false`: a forecast is a typed scenario giving all
nine conditions; a value outside the domain is refused with its name.

## 2. Physical mechanism

The mechanism is W0's: the vapour of the decomposition products condenses near the sample, the smoke
is drawn into an aging chamber, and Brownian coagulation at conserved mass grows the mean particle
volume linearly in time (`SpaceApps-Ardealu:docs/wsp/research/phys_samer_w0.md`, sections 1.3 to
1.5):

```math
D_{30}^3 = D_n^3 + \frac{6K}{\pi}\,\phi\,t,\qquad N(t) = \frac{N_0}{1 + K N_0 t},\qquad d_{10} = D_{30}\,e^{-\ln^2\sigma_g}
```

The study of the development rows confirmed the coagulation law to 10 % (`K` = 5.0 to 5.7e-16 m3/s
measured on the large smokes against 6.0e-16 typed) and changed the amounts
(`SpaceApps-Ardealu:docs/wsp/research/phys_samer_study.md`, sections 0, 1 and 3): every smoke starts
aging with about the same number of particles (3.4e6 per cm3 at 8.1 cm/s), that number falls with the
flow, the size is therefore set by the mass of smoke, the smokes are broader than typed so the count
mean grows faster, and the detector follows the mass of smoke. S1's law:

```math
\ln d_{10} = \mathrm{level}[m] + 0.20\,\frac{T - T_{\mathrm{ref}}}{50\ \mathrm{K}} + f(U) + \text{no-flow terms} + 0.30\,\ln\frac{t_h}{60\ \mathrm{s}} + 0.55\,\mathrm{softplus}(y)
```

```math
y = \ln\frac{t}{450\ \mathrm{s}} + 0.30\,\frac{U - 8.1\ \mathrm{cm/s}}{1\ \mathrm{cm/s}} + 0.30\,[U = 0]
```

with `f(U) = 0.25 ln(8.1 cm/s / max(U, 2 cm/s))` from W0, and `d30` aging with the exponent 1/3.

- Material levels, temperature slope, no-flow and heating-time terms: `d10.median.b0` and
  `d10.median.skip`; `d30.median.b0` and `skip` (W0's).
- `f(U)`: the skip slope on `inlet_velocity` and hinge units 0 and 1.
- No flow for Kapton and Teflon: hinge unit 2 (W0's); Pyrell with no flow: hinge unit 7 (new in S1,
  its count mean does not rise because its distribution becomes very broad, `sigma_g` 3.1 to 3.5).
- The soft-plus of `y`: hinge units 3 to 6, now with no material and no temperature entry (one
  coagulation time for every smoke), output weights 0.55 x (0.2, 0.3, 0.3, 0.2) for `d10`.
- Spread (W0's): ln-sd 0.40 (0.38 for `d30`), +0.06 with no flow, +0.03 per e-fold of age beyond
  270 s; 10 % of the weight 2.5 times wider.
- Detector: `iss_full_scale.logit`, +1.5 for lamp wick, Pyrell and silicone (they are denser smokes),
  no flow -2.6 with silicone given +1.5 back through hinge unit 1, and no low-flow gain for Kapton and
  Teflon (unit 0's output weight set to 0); the aging hinge for Kapton and Teflon and the temperature
  slope are W0's.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section samer.wsp.

## 3. Authoring

- **Author**: `claude-fable-5-2`, 15 November 2026, under PROTOCOL 13.6, in
  `SpaceApps-Ardealu:docs/wsp/research/phys_samer_study.md` (sha256 `e9687b7b…a884f`, block "Edits").
  The logged edits file `runs/dev/gen7/study/samer_s1.wsp` (sha256 `b9a15a81…4872`, 103 elements) was
  taken verbatim as a revision of W0. "Transcribed by the assembly engineer (claude-opus-5-5); no
  number chosen" (`configs/models/law/samer/s1-fable/meta.json`). The element check: the release equals
  its parent except the 103 elements the file writes; 87 differ from the parent, 0 mismatches
  (`SpaceApps-Ardealu:docs/wsp/feedback/samer_s1_result.md`).
- **Step**: PROTOCOL 13.6, "a proper study of the development data": the author studies the
  development cases (train and validation days) case by case, writes the expected effect of each
  candidate before it is run once with the logged what-if tool, drops any candidate whose sign was not
  predicted, and keeps one only if it is lower pooled and on at least two thirds of the days whose
  forecasts change (`SpaceApps-Ardealu:docs/wsp/PROTOCOL.md`, 13.6).
- **What was read** (the release's exposure): the study, the logged edits file and the logged what-if
  run of S1; datasets `samer-train-and-validation-cases`, partitions **train and validation**;
  development feedback: the development scores of PROTOCOL 13.2 to 13.4 and the what-if run of S1;
  prior candidate W0. The study names what it read: the 74 development rows of SAME-R (50 train, 24
  validation; inputs, `d10`, and from the outcome files `d30`, `sigma_g`, number and mass
  concentration, the detector) and the 45 SAME rows as study data for shared physics. The three SAME-R
  test days were not opened.
- **Brief**: [docs/research/PROMPTS.md](../../research/PROMPTS.md); the author worked from
  `phys_samer_study.md`, building on `phys_samer_w0.md` (and having seen `phys_samer_d1.md`).
- **Decisions** (study section 2; three candidates and the proposal, each logged with its prediction):
  - C1, aging: one coagulation time of 450 s for every smoke at 8.1 cm/s, lengthening 1.35 times per
    cm/s below it; `d10` exponent 0.55. Predicted and realised in sign in every stratum: `d10` 0.238 to
    0.198, lower on 11 of 12 days. Kept.
  - C2, levels: lamp wick -0.25 (broader than typed), silicone +0.10, Pyrell +0.35 (run 55 K below
    its reference), Pyrell with no flow -0.5, heating-time exponent 0.30. `d10` to 0.172, lower on 12
    of 12 days. Kept.
  - C3, detector: +1.5 for the large smokes, no flow -2.6 except silicone, no low-flow gain for Kapton
    and Teflon. Brier 0.175 to 0.056, lower on 10 of 10 days. Kept.
  - S1 is C2 and C3 in one file. Spread, tails, flow law, temperature slope, Kapton and Teflon levels
    and `d30` levels stay W0's; no number was changed after a run, and the fifth evaluation was not
    used.

## 4. Weights

Feature order: 0 `is_kapton`, 1 `is_teflon`, 2 `is_silicone`, 3 `is_pyrell`, 4 `sample_temperature`
(`z = (T - 673 K)/50 K`), 5 `inlet_velocity` (`z` = cm/s above 8.1), 6 `no_flow`, 7 `net_aging_time`
(`z = ln(t/100 s)`), 8 `heating_duration` (`z = ln(t_h/60 s)`). Values from the release params; the
group is marked S1 where S1 wrote it, W0 where it is the parent's
(`configs/models/law/samer/w0-fable/edits.wsp`, `configs/models/law/samer/s1-fable/edits.wsp`).

| Group | Shape (nonzero) | Value(s) | Meaning |
|---|---|---|---|
| `x.is_*.*` (4 indicators), `x.no_flow.*` | 5 x 4 scalars | centre 0, scale 1, domain 0 to 1 | W0 |
| `x.sample_temperature.*` | 4 | centre 673, scale 50, lo 450, hi 950 | W0 |
| `x.inlet_velocity.*` | 4 | centre 0.081, scale 0.01, lo 0, hi 0.12 | W0 |
| `x.net_aging_time.*` | 4 | centre 100, scale 1, lo 5, hi 5000 (ln) | W0 |
| `x.heating_duration.*` | 4 | centre 60, scale 1, lo 20, hi 300 (ln) | W0 |
| `d10.median.b0` | [] | -15.412 | S1: lamp wick level 22 % smaller than W0's -15.162 |
| `d10.median.skip` | [9] (8) | -1.416, -1.133, 0.125, 0.704, 0.20, -0.039, 0.236, 0, 0.30 | S1 entries 0 to 3 (Kapton and Teflon unchanged net of b0, silicone +0.10, Pyrell +0.35) and 8 (heating power 0.30); 0.20 per 50 K, -0.039 per cm/s, 0.236 no flow: W0 |
| `d10.median` units 0, 1 | (6) | `W1[*,5]` -1; `b1` -3.1, -6.1; `w2` 0.0373, -0.0763 | W0 flow law, steeper below 5, flat below 2 cm/s |
| `d10.median` unit 2 | (5) | `W1[2,0]` = `W1[2,1]` = `W1[2,6]` = 1; `b1` -1; `w2` 0.149 | W0: Kapton and Teflon with no flow |
| `d10.median` units 3 to 6 | (12 + 4 + 4) | each `W1` row 0.30 on `inlet_velocity`, 0.30 on `no_flow`, 1.0 on `net_aging_time`; `b1` 0.996, -0.604, -2.404, -4.004; `w2` 0.11, 0.165, 0.165, 0.11 | S1: `y = ln(t/450 s) + ...` in four pieces (ln(100/450) + 2.5, 0.9, -0.9, -2.5); exponent 0.55 |
| `d10.median` unit 7 | (4) | `W1[7,3]` = `W1[7,6]` = 1; `b1` -1; `w2` -0.5 | S1: Pyrell with no flow |
| `d10.spread` | b0 + skip (1) + unit 0 (3) | b0 0.40; skip 0.06 on `no_flow`; `W1[0,7]` 1, `b1` -1, `w2` 0.03 | W0 |
| `d10.tail_weight`, `tail_width` | [] | 0.10, 2.5 | W0 |
| `d30.median.b0` | [] | -14.750 | W0 |
| `d30.median.skip` | [9] (8) | -1.857, -1.574, -0.225, 0.037, 0.20, -0.039, 0.19, 0, 0.10 | W0 |
| `d30.median` units 0 to 2 | (11) | as `d10` units 0, 1; unit 2 `w2` 0.12 | W0 |
| `d30.median` units 3 to 6 | (12 + 4 + 4) | `W1` and `b1` as `d10` (S1); `w2` 0.0667, 0.100, 0.100, 0.0667 | aging exponent 1/3 kept |
| `d30.spread`, `d30.tail_*` | | b0 0.38; skip 0.06 on `no_flow`; unit 0 as `d10`; 0.10, 2.5 | W0 |
| `iss_full_scale.logit.b0` | [] | 3.956 | S1: +1.5 for the large smokes (W0 2.456) |
| `iss_full_scale.logit.skip` | [9] (8) | -7.57, -7.56, -1.57, -0.47, 0.6, -0.10, -2.6, 0, 0.5 | S1 entries 0, 1 (Kapton and Teflon held where they were with flow) and 6 (no flow -2.6); 0.6 per 50 K, 0.5 per e-fold of heating: W0 |
| `iss_full_scale.logit` unit 0 | (5) | `W1[0,0]` 2, `W1[0,1]` 2, `W1[0,5]` -0.164, `b1` -2, `w2` **0** | S1 sets the output to 0: no low-flow gain for Kapton and Teflon |
| `iss_full_scale.logit` unit 1 | (4) | `W1[1,2]` 1, `W1[1,6]` 1, `b1` -1, `w2` 1.5 | S1: now silicone with no flow (was Kapton or Teflon with no flow, 0.8) |
| `iss_full_scale.logit` unit 2 | (5) | `W1[2,0]` 2, `W1[2,1]` 2, `W1[2,7]` 1, `b1` -3, `w2` 0.3 | W0: Kapton and Teflon beyond 270 s |
| `num.substeps` | [] | 512 | constant (unused by these targets) |

Totals (release `architecture.parameter_counts`): 530 authored elements, 145 nonzero, 1 constant.
Every number back as edits: `ignis wsp inspect --release sha256:e33ee7665b599e7e6c8c978f61ebf3bf106b2b6a94c3a0821dbdf42cf47a1b5f --edits`;
the parent's full block is `configs/models/law/samer/w0-fable/edits.wsp` and S1's changed lines are
`configs/models/law/samer/s1-fable/edits.wsp`.

## 5. Forward and assembly

| Field | Value |
|---|---|
| Forward | `crates/ignis-wsp/src/law.rs`, `ignis_wsp::law::forecast`, profile `strict_no_fitting` |
| forward_hash | `sha256:c805dec77046018844eabad8935056c628c4ab0abbce63d52d59a7712b9230d1` |
| code_version | `ignis-wsp@0.1.0+fwd.c805dec770460188` |
| Spec | `law-samer-v1`, `sha256:1ac8d550b9f92f76f349dcb69e9cd93f2aad11aaa0bd7b342906563d91232312` (declaration `configs/models/law/samer/decl.json`) |
| Patch | `law-samer-v1-s1-fable`, `sha256:f619208e8d5d663474c35906fdc123cdbed34788db290410bdaa78a6402a40aa` |
| weights_hash | `sha256:57414cc925875c06f87cec540c4b1fc19d23c2c7c459477780ae507fb68fe6eb` |
| Binding | cohort `atlas:PSI-101:catalogue`, split `sha256:66a554b3d85373d84f9cd540d8aba3339ccd9fec9742da4f0e9770760e9d6a1f` (`configs/models/law/samer/binding.json`) |
| Construction | `literal_revision`; uncertainty `assumed_noise`; checks spec_valid, patch_valid, tensors_complete, strict_forward_audit, finite_forward_on_fixture all passed |
| Candidate directory | `configs/models/law/samer/s1-fable` |

Rebuild (the chain is root `configs/models/law/samer/root` → `w0-fable` → `s1-fable`; the registry
must hold the W0 release, as `tools/wsp/reassemble.py` builds it):

```
cargo build --release -p ignis-cli
ignis wsp compile  --family samer --edits configs/models/law/samer/s1-fable/edits.wsp --meta configs/models/law/samer/s1-fable/meta.json --out configs/models/law/samer/s1-fable/patch.json
ignis wsp assemble --family samer --patch configs/models/law/samer/s1-fable/patch.json --binding configs/models/law/samer/binding.json --root <registry holding sha256:05f77ae2…7fb0> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the record; `served_ids` pins the 14
served ids. `cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at
ignis-web a58c7c3: the served id is the content hash of its record and loads through the registry
checks.

## 6. Evaluation

Lower is better. CRPS of ln `d10` from the 5/25/50/75/95 % quantiles; Brier for the detector.

**Final run, test days** (PROTOCOL 13.7, run once on the frozen list sha256 `f3556071…c70e2`; the test
days were unread by every author until this run; source `SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md`,
section PSI-101):

| Release | Role | d10: CRPS, case mean | d10: day-equal | iss: Brier, case mean | iss: day-equal |
|---|---|---|---|---|---|
| **samer S1** | model | **0.1672** | **0.1508** | **0.0144** | **0.0123** |
| samer W0 | comparator 1 (zero-shot parent) | 0.2595 | 0.2050 | 0.0807 | 0.0858 |
| samer D1 | comparator 2 | 0.2165 | 0.1702 | 0.0477 | 0.0590 |
| B0 / I0 constant (SAME-R train) | baseline | 0.3609 | 0.3483 | 0.2997 | 0.2303 |
| B1 / I1 per material (SAME-R train) | baseline | 0.2559 | 0.2532 | 0.0189 | 0.0140 |

33 rows answered on each target (3 test days, 34 rows in the list; `PROTOCOL.md` 13.7). Reading by the
rule written beforehand: S1 **holds** on both targets (lower than every baseline), and the improvement
of the generation holds (lower than the zero-shot parent).

**Development, in-sample** (train and validation days, which the author had studied; PROTOCOL 13.6;
source `SpaceApps-Ardealu:docs/wsp/feedback/samer_s1_result.md`):

| Target, case mean | Partition | W0 | S1 | D1 | Constant | Per material |
|---|---|---|---|---|---|---|
| d10, CRPS | pooled | 0.2377 | **0.1717** | 0.2016 | 0.3458 | 0.2762 |
| d10, CRPS | train | 0.2722 | 0.1907 | 0.2158 | 0.3229 | 0.2454 |
| d10, CRPS | validation | 0.1672 | 0.1327 | 0.1725 | 0.3925 | 0.3392 |
| iss, Brier | pooled | 0.1753 | **0.0562** | 0.1468 | 0.2729 | 0.1012 |
| iss, Brier | train | 0.1501 | 0.0491 | 0.1005 | 0.2066 | 0.0312 |
| iss, Brier | validation | 0.2279 | 0.0711 | 0.2436 | 0.4112 | 0.2473 |

Lower than W0 on 12 of 12 days whose `d10` forecasts change and on 10 of 10 for the detector.

**Earlier looks of the lineage** (blind when made): the zero-shot W0 on the validation days (PROTOCOL
13.2, 23 rows answered, 1 refused, 3 days): `d10` 0.167 against 0.393 and 0.339, detector 0.228 against
0.411 and 0.247, beating every baseline on both (`PROTOCOL.md` 13.2 outcome table, `COVERAGE_19.md`).
D1 on the same validation cases (13.3): `d10` 0.1725 against W0's 0.1672, so W0 stayed.

## 7. Adoption decision and lineage

- **Why it is served.** PROTOCOL 13.6 keeps a candidate for the final run if it is lower than the
  current model pooled and on at least two thirds of the days whose forecasts change, without being
  worse by more than 5 % on another target: S1 passed on both targets (12 of 12, 10 of 10) and became
  the SAME-R model of the frozen list (`samer_s1_result.md`, Reading). In the final run it held on
  both targets, and PROTOCOL 13.7 lists "SAME-R S1" among the releases served after the run, chosen
  before it.
- **Lineage.** neutral root `sha256:f7ce355b…832f` (`configs/models/law/samer/root`, no claim) → W0
  `sha256:05f77ae2…7fb0` (`w0-fable`, zero-shot, `phys_samer_w0.md` section 7.4 typed verbatim; the
  optional aged-row law of its section 7.5 is not part of the release) → S1 `sha256:e33ee766…1b5f`
  (aging, levels, Pyrell with no flow, heating time, detector).
- **Tried and not adopted.** D1 `sha256:f55c4044…ce7a` (`configs/models/law/samer/d1-fable`, PROTOCOL
  13.3, `phys_samer_d1.md`): one damped step from the train anatomy (Pyrell reference temperature
  570 K to 515 K, silicone smoke about 2.5 times denser). It improved train and was worse on
  validation on both targets, so W0 stayed; it ran as comparator 2 in the final run. S1 is a revision
  of W0, not of D1.

**Scope** (served `limits`, condensed, and the result files):

- Targets served: `d10`, `d30`, `iss_full_scale`; one per request through `settings.target` (default
  `d10`). `d30` received the aging change only and is not scored; the study states its level is 0.3 to
  0.65 low in ln (`phys_samer_study.md`, section 3).
- On the train days alone the per-material detector baseline is lower than S1 (0.0312 against 0.0491).
- The study names thin support: the heating exponent rests on one pair of runs, Pyrell with no flow
  and silicone's detector exemption on two runs each (section 4).

## 8. How to call it in IGNIS

- **Forecast lab** (web, route `#/lab`): the Models screen `#/models` lists the served releases and
  opens the model card; `web/src/views/LabModels.tsx` runs a typed scenario (no stored runs).
- **Model card**: `GET /api/v1/model-cards` (fields from `configs/models/served/served.json`).
- **API**: `POST /api/v1/models/sha256:e33ee7665b599e7e6c8c978f61ebf3bf106b2b6a94c3a0821dbdf42cf47a1b5f/scenarios`
  returns an `input_hash`; then `POST /api/v1/analyses` with
  `{schema_version:"wsp.v1", release_id, input:{kind:"prefix", input_hash}, settings:{target, quantile_levels}}`
  returns a job; poll `GET /api/v1/jobs/<job_id>` and read `GET /api/v1/artifacts/<artifact_id>` (an
  `AnalysisArtifact`, evidence class `model_output`).
- **MCP**: `models_inspect`, then `models_forecast` with the `input_hash` of a scenario a person
  admitted in the lab.

The served list has no `example`; these values are typed, not measurements: Kapton heated 60 s at
830 K (its W0 reference temperature) at the standard 8.1 cm/s, smoke aged 40 s.

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:e33ee7665b599e7e6c8c978f61ebf3bf106b2b6a94c3a0821dbdf42cf47a1b5f",
  "cutoff_s": 0,
  "conditions": {
    "is_kapton": 1, "is_teflon": 0, "is_silicone": 0, "is_pyrell": 0,
    "sample_temperature": 830,
    "inlet_velocity": 0.081,
    "no_flow": 0,
    "net_aging_time": 40,
    "heating_duration": 60
  },
  "controls": [],
  "observations": []
}
```

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:e33ee7665b599e7e6c8c978f61ebf3bf106b2b6a94c3a0821dbdf42cf47a1b5f",
  "input": {"kind": "prefix", "input_hash": "<from the scenario>"},
  "settings": {"target": "iss_full_scale"}
}
```

`settings.target` is `d10`, `d30` or `iss_full_scale`; no event target, so no `times_s`. Stored runs:
no. Set `no_flow` = 1 exactly when `inlet_velocity` = 0.

Cite: `samer.wsp S1, hand-written (literal WSP) release sha256:e33ee766…1b5f, experimental, not trained; its output is model output, not a measurement.`

## Sources

- `configs/models/served/served.json` (entry `samer.wsp`);
  `configs/models/served/models/e33ee7665b599e7e6c8c978f61ebf3bf106b2b6a94c3a0821dbdf42cf47a1b5f/release.json` and its params (data pack)
- `configs/models/law/samer/s1-fable/{edits.wsp,meta.json,release.json}`, `configs/models/law/samer/w0-fable/{edits.wsp,meta.json,release.json}`,
  `configs/models/law/samer/d1-fable/{edits.wsp,meta.json,release.json}`, `configs/models/law/samer/root/release.json`,
  `configs/models/law/samer/binding.json`, `configs/models/law/samer/decl.json`
- `crates/ignis-wsp/src/law.rs`; `crates/ignis-wsp-serve/src/lib.rs`; `crates/ignis-core/src/contracts/wsp.rs`;
  `crates/ignis-core/src/contracts/research.rs`; `tools/wsp/verify_served.py`
- `SpaceApps-Ardealu:docs/wsp/research/law_forward.md`, `law_forward_targets.md` (feat/handwritten-science, 542dad8)
- `SpaceApps-Ardealu:docs/wsp/research/phys_samer_w0.md`, `phys_samer_d1.md`, `phys_samer_study.md`
- `SpaceApps-Ardealu:docs/wsp/feedback/samer_s1_result.md`
- `SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md`; `SpaceApps-Ardealu:docs/wsp/PROTOCOL.md`, sections 13.2 to 13.7;
  `SpaceApps-Ardealu:docs/wsp/COVERAGE_19.md`
