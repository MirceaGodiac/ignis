# same.wsp — SAME smoke size and ISS detector, first flight

`same.wsp` forecasts the smoke of overheated spacecraft materials and of dibutyl phthalate on SAME, the
first Smoke Aerosol Measurement Experiment (PSI-102): the count mean diameter `d10` and the diameter of
average mass `d30` at the age the smoke is measured, and whether the ISS detector's scatter channel is
at full scale. Typed zero-shot and scored once on the whole investigation, which was held: particle
diameter **0.184** (CRPS of ln d10) against 0.220 for the constant and 0.192 for the per-material
baseline, so it beats every baseline; detector at full scale 0.059 against 0.205 for the constant
(`docs/wsp/feedback/same_w0_result.md`). Release
`sha256:e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062`.

| | |
|---|---|
| Model | `same.wsp` |
| Investigation | SAME (PSI-102), smoke aerosol, first flight (9 September to 5 October 2007) |
| Family | `other` (generic law slug `same`) |
| Candidate | W0 (`configs/models/law/same/w0-fable`) |
| Generation | seventh (PROTOCOL 13.2, one blind look) |
| Author | `claude-fable-5-2`; transcribed by the assembly engineer `claude-opus-5-5` |
| Authorship | zero-shot |
| Release | `sha256:e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062` |
| Forward | `ignis_wsp::law::forecast`, `crates/ignis-wsp/src/law.rs` (profile `strict_no_fitting`, version 1) |
| Parent release | `sha256:070a2a61038b73cd06698df7443760da7055d3376af65bca81595b5b24b62193` (the neutral root of slug `same`, `configs/models/law/same/root`) |

## 1. Task and targets

Every input is a typed condition (`configs/models/served/served.json`, entry `same.wsp`):

| Input | Kind | SI unit | Note |
|---|---|---|---|
| `is_kapton` | condition | 1 | material indicator (0 or 1) |
| `is_teflon` | condition | 1 | material indicator |
| `is_silicone` | condition | 1 | material indicator |
| `is_dbp` | condition | 1 | dibutyl phthalate; all four at 0 is lamp wick, the reference |
| `inlet_velocity` | condition | m/s | nominal duct air speed during heating; domain 0 to 0.12 |
| `net_aging_time` | condition | s | the horizon: age of the smoke when the row was measured; domain 5 to 5000 |

| Target | Kind | Unit | Meaning |
|---|---|---|---|
| `d10` | positive | m | count mean diameter M1/M0 at `net_aging_time` (lane-local, label `same:d10`) |
| `d30` | positive | m | diameter of average mass at `net_aging_time` (lane-local, `same:d30`; the author's own, not scored) |
| `iss_full_scale` | binary | none | ISS detector scatter channel at full scale, \|V\| >= 6.0 V |

There is no time series: the age of the smoke is an input, and a positive target is returned at
`times_s = [cutoff_s]` (`docs/wsp/research/law_forward.md`, section 2.4). The binary
target is an `event_probabilities` artifact with categories `yes` and `no`. `stored_runs` is `false`:
no stored run of this investigation carries the declared features, so a forecast is always a typed
scenario that gives all six conditions; a value outside the release's domain is refused with its name.

## 2. Physical mechanism

A wire heats a few milligrams of material below ignition for about a minute. The vapour condenses
within the first millimetre into particles of a condensation size `D_n` set by the material and by
the flow that cools and dilutes the vapour; a piston draws the smoke into an aging chamber, where
Brownian coagulation at conserved mass makes the mean particle volume grow linearly in time and
narrows the distribution (`docs/wsp/research/phys_same_w0.md`, section 1.1, which
restates `phys_samer_w0.md`, sections 1.3 to 1.5). SAME adds one material that SAME-R lacks: dibutyl
phthalate, a liquid that only evaporates and recondenses, so nearly all its vapour returns as large,
uniform, slowly coagulating drops; the author makes it the nearly inert reference of the flight
(section 1.2).

Coagulation, with `K` = 6.0e-16 m3/s:

```math
v(t) = v_0 + K\,\phi\,t,\qquad D_{30}(t) = D_n\,(1 + t/t_c)^{1/3},\qquad t_c = \frac{1}{K N_0}
```

The law the network carries (section 3), tied to its tensor groups:

```math
f(U) = 0.25\,\ln\frac{8.1\ \mathrm{cm/s}}{\max(U,\ 2\ \mathrm{cm/s})}
```

```math
\ln d_{10} = \ln D_{\mathrm{ref}}[m] - \ln^2\sigma_0[m] + f(U) + 0.413\,\mathrm{softplus}\!\left(\ln\frac{t}{t_c[m]}\right)
```

```math
\ln D_{30} = \ln D_{\mathrm{ref}}[m] + f(U) + \tfrac{1}{3}\,\mathrm{softplus}\!\left(\ln\frac{t}{t_c[m]}\right)
```

- Material levels `ln D_ref - ln^2 sigma_0` sit in `d10.median.b0` (lamp wick) and the first four
  entries of `d10.median.skip`; `d30` uses `ln D_ref` alone.
- `f(U)` is the skip slope on `inlet_velocity` plus hinge units 0 and 1 (steeper below 5 cm/s, flat
  below 2 cm/s).
- The soft-plus of `y = ln(t/t_c[m])` is four hinge units (2 to 5), with the material shifts of `t_c`
  in their `W1` rows; the output weights are 0.413 (d10) or 1/3 (d30) times 0.2, 0.3, 0.3, 0.2.
- Spread: ln-sd 0.42 (0.40 for `d30`), +0.10 for dibutyl phthalate, +0.03 per e-fold of age beyond
  270 s; 10 % of the weight in a component 2.5 times wider (`*.spread.*`, `*.tail_weight`,
  `*.tail_width`).
- The detector: per unit mass the forward-scatter signal goes as the cube of the size below about
  0.7 um and is flat above, so Kapton and Teflon gain with low flow and aging while the large smokes
  sit on the flat top (`phys_samer_w0.md`, section 1.7). Typed as material logits in
  `iss_full_scale.logit.skip`, +0.10 per cm/s below 8.1, and hinge units 0 and 1 for Kapton and
  Teflon at low flow and beyond 270 s.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section same.wsp.

## 3. Authoring

- **Author**: `claude-fable-5-2`, physicist-author of the seventh generation, 15 November 2026. The
  author wrote every number in the dossier `docs/wsp/research/phys_same_w0.md`,
  section 7.2. "Transcribed by the assembly engineer (claude-opus-5-5); no number chosen, rounded or
  changed" (`configs/models/law/same/w0-fable/meta.json`, rationale); the edits file names its source
  lines 201 to 290.
- **Step**: PROTOCOL 13.2, one blind look per investigation without a model
  (`docs/wsp/PROTOCOL.md`, section 13.2). The release is a revision of the slug's
  neutral root.
- **What was read** (the release's exposure): the dossier itself; the exposure of `phys_samer_w0.md`
  section 8.3, which applies (the cards' findings, the abstracts, and SAME-R train facts read after the
  constants were typed and not used); `feedback/same_feasibility.md` sections "For the physicist", 2
  and 3, which hold no outcome; and the author's pretraining knowledge (unverifiable). Datasets: none.
  Development feedback: none. Prior candidates: none. No SAME outcome was opened
  (`phys_same_w0.md`, header and section 8).
- **Decisions**: the network is the SAME-R W0 network without the sample-temperature, no-flow and
  heating-time terms (section 7.1): four of the 45 rows lack a sample temperature and there is no
  reference temperature for the DBP wick, so the law is evaluated at each material's reference
  condition and the spread is widened to 0.42 (section 2). The velocity is read with the identity
  transform so that a row at zero flow would be forecast as the 2 cm/s value instead of refused. DBP:
  `D_ref` 400 nm, `sigma_0` 1.5, `t_c` 2500 s, P(full scale) 0.90, from the evaporation and
  recondensation argument of section 1.2. Of the card's SAME findings, the author used the number
  falling 1.7 to 2.7 times in 800 s to temper `t_c` and the narrow/broad grouping for `sigma_0`; the
  50 % ground ratio and the 300 nm were known when the flow exponent and `D_ref` were typed, and the
  dossier says agreement with them is therefore not a test (section 8, Exposure).

## 4. Weights

Feature order: 0 `is_kapton`, 1 `is_teflon`, 2 `is_silicone`, 3 `is_dbp`, 4 `inlet_velocity`,
5 `net_aging_time`. `z4` is cm/s above 8.1; `z5 = ln(t / 100 s)`. Values from the release's params
(pack of `configs/models/served/models/e1362b55…/`) and the comments of
`configs/models/law/same/w0-fable/edits.wsp`.

| Group | Shape (nonzero) | Value(s) | Meaning |
|---|---|---|---|
| `x.is_*.{centre,scale,lo,hi}` (4 indicators) | 4 x 4 scalars | centre 0, scale 1, domain 0 to 1 | 0/1 indicators, lamp wick = all 0 |
| `x.inlet_velocity.*` | 4 scalars | centre 0.081, scale 0.01, lo 0, hi 0.12 | `z4` in cm/s about 8.1 cm/s |
| `x.net_aging_time.*` | 4 scalars | centre 100, scale 1, lo 5, hi 5000 (ln) | the horizon in e-folds of 100 s |
| `d10.median.b0` | [] | -15.614 | lamp wick, `D_ref` 250 nm, `sigma_0` 1.9: 166 nm at 8.1 cm/s, short aging |
| `d10.median.skip` | [6] (5) | -0.586, -0.463, 0.215, 0.718, -0.039, 0 | material levels (Kapton, Teflon, silicone, DBP); -0.039 per cm/s of flow |
| `d10.median.W1` units 0, 1 | (2) | `W1[0,4]` = `W1[1,4]` = -1 | flow hinges |
| `d10.median.b1` units 0, 1 | (2) | -3.1, -6.1 | hinges at 5 and 2 cm/s |
| `d10.median.w2` units 0, 1 | (2) | 0.0373, -0.0763 | steeper below 5 cm/s, flat below 2 cm/s |
| `d10.median.W1` units 2 to 5 | [4, 6] (20) | each row 0.917, 0.693, -0.336, -0.916, 0, 1.0 | `y = ln(t/t_c)`: material shifts ln(1000/400), ln(1000/500), ln(1000/1400), ln(1000/2500) |
| `d10.median.b1` units 2 to 5 | (4) | 0.197, -1.403, -3.203, -4.803 | ln(100/1000) + 2.5, 0.9, -0.9, -2.5: the soft-plus in four pieces |
| `d10.median.w2` units 2 to 5 | (4) | 0.0826, 0.124, 0.124, 0.0826 | 0.413 x (0.2, 0.3, 0.3, 0.2) |
| `d10.spread.b0` | [] | 0.42 | core ln-sd (a factor 1.5) |
| `d10.spread.skip` | [6] (1) | 0.10 on `is_dbp` | DBP wider |
| `d10.spread.W1/b1/w2` unit 0 | (3) | `W1[0,5]` 1, `b1[0]` -1, `w2[0]` 0.03 | +0.03 per e-fold beyond 272 s |
| `d10.tail_weight`, `d10.tail_width` | [] | 0.10, 2.5 | 10 % in a component 2.5 times wider |
| `d30.median.b0` | [] | -15.202 | lamp wick `D_ref` 250 nm |
| `d30.median.skip` | [6] (5) | -0.777, -0.654, 0.215, 0.470, -0.039, 0 | `ln D_ref` levels; flow slope as `d10` |
| `d30.median.W1/b1` | (22 + 6) | as `d10` | same flow and aging hinges |
| `d30.median.w2` | (6) | 0.0373, -0.0763, 0.0667, 0.100, 0.100, 0.0667 | aging exponent 1/3 |
| `d30.spread.*`, `d30.tail_*` | | b0 0.40; skip 0.10 on DBP; unit 0 as `d10`; tail 0.10, 2.5 | |
| `iss_full_scale.logit.b0` | [] | 1.10 | lamp wick, P = 0.75 |
| `iss_full_scale.logit.skip` | [6] (5) | -2.83, -3.30, -0.25, 1.10, -0.10, 0 | logits 1.10 + skip: Kapton -1.73, Teflon -2.20, silicone 0.85, DBP 2.20; +0.10 per cm/s below 8.1 |
| `iss_full_scale.logit` unit 0 | (5) | `W1[0,0]` 2, `W1[0,1]` 2, `W1[0,4]` -0.164, `b1` -2, `w2` 0.9 | Kapton and Teflon gain the cube of the size at low flow (+0.9 at 2 cm/s) |
| `iss_full_scale.logit` unit 1 | (5) | `W1[1,0]` 2, `W1[1,1]` 2, `W1[1,5]` 1, `b1` -3, `w2` 0.3 | and +0.3 per e-fold of age beyond 270 s |
| `num.substeps` | [] | 512 | constant: integration steps per interval (unused by these targets) |

Totals (release `architecture.parameter_counts`): 383 authored elements, 125 nonzero, 1 constant.
Every number back as edits: `ignis wsp inspect --release sha256:e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062 --edits`,
or `configs/models/law/same/w0-fable/edits.wsp`.

## 5. Forward and assembly

| Field | Value |
|---|---|
| Forward | `crates/ignis-wsp/src/law.rs`, `ignis_wsp::law::forecast`, profile `strict_no_fitting` |
| forward_hash | `sha256:5a0c852e230d33e6399ee483172f932c672b94047d78e2891445afc1c8e1e2ae` |
| code_version | `ignis-wsp@0.1.0+fwd.5a0c852e230d33e6` |
| Spec | `law-same-v1`, `sha256:abf8c3e3acb1444d2f01860e37f5319539df452c198087908bb3e6e0f6bfff3b` (declaration `configs/models/law/same/decl.json`) |
| Patch | `law-same-v1-w0-fable`, `sha256:8344896c1fc84cf7fc4264a19c9c03f50ceec1b0d9c3bb947b75f872b7f9c2b5` |
| weights_hash | `sha256:774275522086c6dbb050cdb14db276f643cf4c98dbdf9907eb67e636b2c18beb` |
| Binding | cohort `atlas:PSI-102:catalogue`, split `sha256:1cf3fb0f69e3c2ee010ca5d0b28234bc079d86f2f85b3e1e4a8156028a212d15` (`configs/models/law/same/binding.json`) |
| Construction | `literal_revision`; uncertainty `assumed_noise`; checks spec_valid, patch_valid, tensors_complete, strict_forward_audit, finite_forward_on_fixture all passed |
| Candidate directory | `configs/models/law/same/w0-fable` |

Rebuild (the parent is the neutral root `configs/models/law/same/root`, which must be assembled into
the registry first, as `tools/wsp/reassemble.py` does):

```
cargo build --release -p ignis-cli
ignis wsp compile  --family same --edits configs/models/law/same/w0-fable/edits.wsp --meta configs/models/law/same/w0-fable/meta.json --out configs/models/law/same/w0-fable/patch.json
ignis wsp assemble --family same --patch configs/models/law/same/w0-fable/patch.json --binding configs/models/law/same/binding.json --root <registry holding sha256:070a2a61…2b62193> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the record; `served_ids` pins the 14
served ids. `cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at
ignis-web a58c7c3: the served id is the content hash of its record and loads through the registry
checks.

## 6. Evaluation

SAME is held whole (`docs/wsp/atlas/splits/PSI-102.json`, one group `PSI-102:matrix`): there is no
train, validation or test partition, and its one use is this blind look. The author had seen no SAME
outcome when the numbers were typed. Baselines were built from SAME-R train days only and applied
unchanged; DBP, absent from SAME-R, takes the constant (`feedback/same_feasibility.md`, section 5).
Lower is better. Source: `docs/wsp/feedback/same_w0_result.md` (PROTOCOL 13.2).

| Target, metric | Rows scored | same W0 | Constant (B0 / I0, SAME-R train) | Per material (B1 / I1, SAME-R train) | Reading |
|---|---|---|---|---|---|
| `d10`, CRPS of ln d10 | 25 | **0.1844** | 0.2197 | 0.1921 | beats the best baseline |
| `iss_full_scale`, Brier | 45 | **0.0591** | 0.2054 | 0.0554 | beats the simple baseline |

All 45 rows were asked and answered, none refused; `d10` is present on 25 of them (17 primary and 8
aged rows; `feedback/same_feasibility.md`, section 4), the detector on all 45. With one held group the
per-day score is the aggregate. The same numbers are in `PROTOCOL.md` 13.2 (outcome table) and
`COVERAGE_19.md` (row SAME).

## 7. Adoption decision and lineage

- **Why it is served.** It is the first and only hand-written model of SAME. PROTOCOL 13.3: "SAME and
  DAFT have no data left to judge a revision and keep their zero-shot releases"; 13.4: their
  zero-shot scores of 13.2 "stand as their only held result, and any later revision of them is
  unjudged". SAME has no test days, so it was not in the final run of 13.7.
- **Lineage.** neutral root `sha256:070a2a61…2b62193` (`configs/models/law/same/root`, generated
  mechanically from the spec, no claim) → W0 `sha256:e1362b55…2062`, which writes every number of the
  law. The W0 restates the SAME-R W0 law (`samer` w0-fable, `sha256:05f77ae2…7fb0`) for the first
  flight; it is not a revision of that release.
- **Tried and not adopted.** Nothing: no revision of SAME was typed. The SAME-R study
  (`phys_samer_study.md`) read the 45 SAME rows as study data for shared physics and states that SAME
  is not revised; it found SAME's starting number concentration about half SAME-R's.

**Scope** (served `limits`, condensed):

- Scored whole: no held data remains to judge a revision.
- On the detector target the per-material baseline (0.0554) is slightly lower than the release
  (0.0591).
- Targets `d10`, `d30`, `iss_full_scale`; one per request through `settings.target` (default `d10`).

## 8. How to call it in IGNIS

- **Forecast lab** (web, route `#/lab`): the Models screen `#/models` lists the served releases and
  opens the model card; `web/src/views/LabModels.tsx` runs a typed scenario (stored runs are not
  available for this release).
- **Model card**: `GET /api/v1/model-cards` (fields from `configs/models/served/served.json`, with the
  cite string).
- **API**: `POST /api/v1/models/sha256:e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062/scenarios`
  returns an `input_hash`; then `POST /api/v1/analyses` with
  `{schema_version:"wsp.v1", release_id, input:{kind:"prefix", input_hash}, settings:{target, quantile_levels}}`
  returns a job; poll `GET /api/v1/jobs/<job_id>` and read `GET /api/v1/artifacts/<artifact_id>` (an
  `AnalysisArtifact`, evidence class `model_output`).
- **MCP**: `models_inspect`, then `models_forecast` with the `input_hash` of a scenario a person
  admitted in the lab.

The served list has no `example` for this release; the values below are typed, not measurements:
dibutyl phthalate at 5 cm/s, smoke aged 1800 s, the check case of `phys_same_w0.md` section 7.2 (where
the author computes `d10` = 492 nm and P = 0.92).

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062",
  "cutoff_s": 0,
  "conditions": {
    "is_kapton": 0, "is_teflon": 0, "is_silicone": 0, "is_dbp": 1,
    "inlet_velocity": 0.05,
    "net_aging_time": 1800
  },
  "controls": [],
  "observations": []
}
```

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062",
  "input": {"kind": "prefix", "input_hash": "<from the scenario>"},
  "settings": {"target": "d10", "quantile_levels": [0.05, 0.25, 0.5, 0.75, 0.95]}
}
```

`settings.target` is `d10`, `d30` or `iss_full_scale`; `times_s` and `horizon_s` are not needed (no
event target). Stored runs: no.

Cite: `same.wsp W0, hand-written (literal WSP) release sha256:e1362b55…2062, experimental, not trained; its output is model output, not a measurement.`

## Sources

- `configs/models/served/served.json` (entry `same.wsp`);
  `configs/models/served/models/e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062/release.json` and its params (data pack)
- `configs/models/law/same/w0-fable/{edits.wsp,meta.json,release.json}`, `configs/models/law/same/root/release.json`,
  `configs/models/law/same/binding.json`, `configs/models/law/same/decl.json`
- `crates/ignis-wsp/src/law.rs`; `crates/ignis-wsp-serve/src/lib.rs`; `crates/ignis-core/src/contracts/wsp.rs`;
  `crates/ignis-core/src/contracts/research.rs`; `tools/wsp/verify_served.py`
- `docs/wsp/research/law_forward.md`, `law_forward_targets.md` (feat/handwritten-science, 542dad8)
- `docs/wsp/research/phys_same_w0.md`, `phys_samer_w0.md`, `phys_samer_study.md`
- `docs/wsp/feedback/same_w0_result.md`, `same_feasibility.md`, `same_train_anatomy.md`
- `docs/wsp/PROTOCOL.md`, sections 13, 13.2 to 13.4, 13.7; `docs/wsp/COVERAGE_19.md`
