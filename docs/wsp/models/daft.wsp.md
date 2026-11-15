# daft.wsp — DAFT particle counter on cabin air and test dust

`daft.wsp` forecasts what the modified P-Trak condensation counter of DAFT and DAFT-2 (PSI-47) read on
a sample of air in orbit, the number concentration `N`, given the kind of air (filtered, cabin, cabin
while Velcro is torn, road dust in a bag) and what the DustTrak photometer read on the same air. Typed
zero-shot and scored once on the whole investigation, which was held: CRPS **0.777** against 2.678 for
the one baseline, the smoke number per mass of SAME-R
(`docs/wsp/feedback/daft_w0_result.md`). Release
`sha256:061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d`.

| | |
|---|---|
| Model | `daft.wsp` |
| Investigation | DAFT / DAFT-2 (PSI-47), dust and aerosol: a modified P-Trak against a DustTrak in orbit |
| Family | `other` (generic law slug `daft`) |
| Candidate | W0 (`configs/models/law/daft/w0-fable`) |
| Generation | seventh (PROTOCOL 13.2, one blind look) |
| Author | `claude-fable-5-2`; transcribed by the assembly engineer `claude-opus-5-5` |
| Authorship | zero-shot |
| Release | `sha256:061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d` |
| Forward | `ignis_wsp::law::forecast`, `crates/ignis-wsp/src/law.rs` (profile `strict_no_fitting`, version 1) |
| Parent release | `sha256:f6c492064bc897d1448c5edce3ad8d06d649b038fa53f4127cff5a0645d5539d` (the neutral root of slug `daft`, `configs/models/law/daft/root`) |

## 1. Task and targets

Every input is a typed condition (`configs/models/served/served.json`, entry `daft.wsp`):

| Input | Kind | SI unit | Note |
|---|---|---|---|
| `is_cabin` | condition | 1 | cabin air through the hose (0 or 1) |
| `is_velcro` | condition | 1 | cabin air while Velcro is separated |
| `is_bag` | condition | 1 | road dust in a bag; all three at 0 is filtered (HEPA) air, the reference |
| `dusttrak_mass_floored` | condition | kg/m3 | `max(dusttrak_mass_concentration, 1e-9 kg/m3)`, the photometer reading floored at its resolution; domain 1e-9 to 2e-4 |

| Target | Kind | Unit | Meaning |
|---|---|---|---|
| `number_concentration` | positive | 1/m3 | what the modified P-Trak reads on the sample: a window mean (2005) or one reading (2006 bags); lane-local, label `daft:number_concentration` |

There is no time dependence: the forecast is one distribution returned at `times_s = [cutoff_s]`.
`stored_runs` is `false`: no stored run carries the declared features, so a forecast is a typed
scenario with all four conditions; a value outside the domain is refused with its name.

## 2. Physical mechanism

DAFT burned nothing. The counter grows every particle above about 20 nm into an alcohol droplet and
counts it (the zeroth moment); the photometer reads scattered light that, for a fixed size
distribution, is proportional to mass. The two readings are tied by the mean mass of a particle
(`docs/wsp/research/phys_daft_w0.md`, sections 0 and 1.1):

```math
N = \frac{c}{m_p} = \frac{6\,c}{\pi\,\rho\,D_{30}^3}
```

so the forecast is a statement about `D30` for each kind of air. In orbit nothing settles and the
dust does not coagulate (coagulation number about 2e-3), so cabin air is a source against the HEPA
filter loop, `N = S / (eta Q)`, and a dust bag keeps a constant `N/c`. The law (section 3), with
`x = max(DustTrak reading, 1e-9 kg/m3)`:

```math
\ln N = \begin{cases}\ln(5.0\times10^{4}) & \text{filtered air, ln-sd 1.5}\\ \ln(1.5\times10^{6}) + 0.30\,\ln(x / 3\times10^{-9}) & \text{cabin, ln-sd 1.5}\\ \ln(8.0\times10^{6}) + 0.50\,\ln(x / 3\times10^{-8}) & \text{Velcro, ln-sd 1.5}\\ \ln(9.0\times10^{8}) + 0.95\,\ln(x / 10^{-6}) & \text{bag, ln-sd 0.9}\end{cases}
```

with `N` in 1/m3 and 15 % of the weight in a component 2.5 times wider. The bag value is 900 per cm3
for each mg/m3 read: the fine tail of Arizona road dust below the 2.5 um inlet cut, `D30` about
0.85 um at 2650 kg/m3, with the photometer over-reading that fraction by about 1.3 (section 1.4).

- The filtered-air level is `number_concentration.median.b0`; the level of each kind of air,
  extrapolated to a reading of 1 mg/m3, is in `number_concentration.median.skip`.
- The slope on the reading differs by kind of air. A pair of hinges switched by an indicator,
  `s relu(z + 12 (i - 1)) - s relu(-z + 12 (i - 1))`, equals `s z` when `i` = 1 and 0 when `i` = 0
  for `|z|` below 12 (section 7.1): three pairs, six hinge units.
- The spread is `number_concentration.spread` (b0 1.5, -0.6 for the bag), the tail
  `number_concentration.tail_weight` and `tail_width`.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section daft.wsp.

## 3. Authoring

- **Author**: `claude-fable-5-2`, physicist-author of the seventh generation, 15 November 2026, in
  `docs/wsp/research/phys_daft_w0.md`, section 7.2. "Transcribed by the assembly
  engineer (claude-opus-5-5); no number chosen, rounded or changed"
  (`configs/models/law/daft/w0-fable/meta.json`); the edits file names its source lines 239 to 279.
- **Step**: PROTOCOL 13.2, one blind look (`docs/wsp/PROTOCOL.md`, section 13.2); a
  revision of the slug's neutral root.
- **What was read** (the release's exposure): the dossier; `feedback/daft_feasibility.md`, sections
  "For the physicist" and 1 to 3, with no outcome; the card `docs/wsp/atlas/cards/PSI-47.json` (its
  findings, no number); pretraining knowledge (unverifiable). Datasets: none. Development feedback:
  none. Prior candidates: none. The card told the author the sign of two forecasts (a clean cabin, a
  low count per unit mass); the dossier states that no number came from it and that the cabin median
  of 1.5 per cm3 is the source-over-filter estimate, not a remembered value (section 8, Exposure).
- **Decisions**: use the kind of air and the DustTrak reading only; location, bag number, elapsed
  time and window duration carry no term because the author had no physical reason for one
  (section 2). Floor the reading at the photometer's resolution so that the ln transform can read a
  photometer at its floor (section 7.1). Exponents on the reading: 0.30 for cabin air (number and
  mass come from different particles, and `c` is read near the floor), 0.50 for Velcro, 0.95 in the
  bag (no settling, no coagulation; 0.05 for coincidence at the top of the range).

## 4. Weights

Feature order: 0 `is_cabin`, 1 `is_velcro`, 2 `is_bag`, 3 `dusttrak_mass_floored`, with
`z3 = ln(x / 1 mg/m3)` (-6.9 to 5.3 over the domain). Values from the release params and
`configs/models/law/daft/w0-fable/edits.wsp`.

| Group | Shape (nonzero) | Value(s) | Meaning |
|---|---|---|---|
| `x.is_cabin/is_velcro/is_bag.*` | 3 x 4 scalars | centre 0, scale 1, domain 0 to 1 | indicators; filtered air = all 0 |
| `x.dusttrak_mass_floored.*` | 4 scalars | centre 1e-6, scale 1, lo 1e-9, hi 2e-4 (ln) | reading in e-folds of 1 mg/m3 |
| `number_concentration.median.b0` | [] | 10.82 | filtered air, 5.0e4 per m3 (0.05 per cm3) |
| `number_concentration.median.skip` | [4] (3) | 5.144, 6.828, 9.798, 0 | cabin, Velcro, bag at a reading of 1 mg/m3, minus b0 |
| `number_concentration.median.W1` units 0, 1 | (4) | rows (12, 0, 0, 1) and (12, 0, 0, -1) | cabin pair |
| units 2, 3 | (4) | rows (0, 12, 0, 1) and (0, 12, 0, -1) | Velcro pair |
| units 4, 5 | (4) | rows (0, 0, 12, 1) and (0, 0, 12, -1) | bag pair |
| `number_concentration.median.b1` | [8] (6) | -12 for units 0 to 5 | switches each pair off unless its indicator is 1 |
| `number_concentration.median.w2` | [8] (6) | 0.30, -0.30, 0.50, -0.50, 0.95, -0.95 | exponents on the reading: cabin, Velcro, bag |
| `number_concentration.spread.b0` | [] | 1.5 | ln-sd for filtered, cabin and Velcro air (a factor 4.5) |
| `number_concentration.spread.skip` | [4] (1) | -0.6 on `is_bag` | ln-sd 0.9 for the dust |
| `number_concentration.spread.W1/b1/w2` | (0) | all zero | no hinge |
| `number_concentration.tail_weight`, `tail_width` | [] | 0.15, 2.5 | 15 % for a counter that misbehaves |
| `num.substeps` | [] | 512 | constant (unused by this target) |

Totals (release `architecture.parameter_counts`): 124 authored elements, 42 nonzero, 1 constant.
Every number back as edits: `ignis wsp inspect --release sha256:061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d --edits`,
or `configs/models/law/daft/w0-fable/edits.wsp`.

## 5. Forward and assembly

| Field | Value |
|---|---|
| Forward | `crates/ignis-wsp/src/law.rs`, `ignis_wsp::law::forecast`, profile `strict_no_fitting` |
| forward_hash | `sha256:31f5fc46329b626451a6bac85d9e93d4455271f381a9f809dc6c5b6922677056` |
| code_version | `ignis-wsp@0.1.0+fwd.31f5fc46329b6264` |
| Spec | `law-daft-v1`, `sha256:451e59f648b2015ce782bbe0e6ea9373c7e142bbde9ab010ffe1fb515a5f28e7` (declaration `configs/models/law/daft/decl.json`) |
| Patch | `law-daft-v1-w0-fable`, `sha256:87b09d3a343894699cd10eb0885c938ac99c985ce998e3e7930b99012d1cf44c` |
| weights_hash | `sha256:638297f9818637d0214d4f43768181eeb2a9c1f660f82eb3470d80c8e034ce48` |
| Binding | cohort `atlas:PSI-47:catalogue`, split `sha256:a345930846600ecf82c8d1f2350391323e8731bb536fb8c2132f9449e4dea08a` (`configs/models/law/daft/binding.json`) |
| Construction | `literal_revision`; uncertainty `assumed_noise`; checks spec_valid, patch_valid, tensors_complete, strict_forward_audit, finite_forward_on_fixture all passed |
| Candidate directory | `configs/models/law/daft/w0-fable` |

Rebuild (the parent is the neutral root `configs/models/law/daft/root`, assembled into the registry
first, as `tools/wsp/reassemble.py` does):

```
cargo build --release -p ignis-cli
ignis wsp compile  --family daft --edits configs/models/law/daft/w0-fable/edits.wsp --meta configs/models/law/daft/w0-fable/meta.json --out configs/models/law/daft/w0-fable/patch.json
ignis wsp assemble --family daft --patch configs/models/law/daft/w0-fable/patch.json --binding configs/models/law/daft/binding.json --root <registry holding sha256:f6c49206…d5539d> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the record; `served_ids` pins the 14
served ids. `cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at
ignis-web a58c7c3: the served id is the content hash of its record and loads through the registry
checks.

## 6. Evaluation

DAFT is held whole (`docs/wsp/atlas/splits/PSI-47.json`; two groups, too few to split): no train,
validation or test partition, and this blind look is its one use. The author had seen no DAFT
outcome. No DAFT day is train, so the baseline was fixed from SAME-R train days alone: the smoke number
per mass, `N_q = k_q * dusttrak_mass_concentration` (`feedback/daft_feasibility.md`, section 5). Metric:
quantile CRPS of ln(1 + N per cm3); lower is better. Source:
`docs/wsp/feedback/daft_w0_result.md` (PROTOCOL 13.2).

| Score | Cases | daft W0 | Smoke number per mass (SAME-R train) |
|---|---|---|---|
| **Case mean (registered)** | 53 | **0.7772** | 2.6778 |
| Group-equal mean | 2 groups | 1.3725 | 2.2342 |
| Group `PSI-47:2006-08-21:bags` (DAFT-2 road dust) | 42 | 0.3547 | 2.9927 |
| Group `PSI-47:session:D2` (filtered, cabin, Velcro) | 11 | 2.3903 | 1.4756 |

All 53 cases asked were answered, none refused. Reading: beats the only baseline (also in
`PROTOCOL.md` 13.2 and `COVERAGE_19.md`, row DAFT). The 42 bag readings carry the case mean, as the
engineer's note said they would (section 4 there).

## 7. Adoption decision and lineage

- **Why it is served.** It is the first and only hand-written model of DAFT. PROTOCOL 13.3: SAME and
  DAFT "have no data left to judge a revision and keep their zero-shot releases"; 13.4: the zero-shot
  scores of 13.2 stand as their only held result. DAFT has no test days and was not in the final run
  of 13.7.
- **Lineage.** neutral root `sha256:f6c49206…d5539d` (`configs/models/law/daft/root`, mechanical, no
  claim) → W0 `sha256:061f93de…5e5d`, which writes every number of the law.
- **Tried and not adopted.** Nothing; no revision of DAFT was typed.

**Scope** (served `limits`, condensed, and the result file):

- Scored whole on 2 groups: no held data remains to judge a revision.
- On the 11 cases of session D2 (filtered, cabin and Velcro air) the baseline is lower than the
  release (1.476 against 2.390); the win is carried by the bag readings.
- One target, `number_concentration`.

## 8. How to call it in IGNIS

- **Forecast lab** (web, route `#/lab`): the Models screen `#/models` lists the served releases and
  opens the model card; `web/src/views/LabModels.tsx` runs a typed scenario (no stored runs).
- **Model card**: `GET /api/v1/model-cards` (fields from `configs/models/served/served.json`).
- **API**: `POST /api/v1/models/sha256:061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d/scenarios`
  returns an `input_hash`; then `POST /api/v1/analyses` with
  `{schema_version:"wsp.v1", release_id, input:{kind:"prefix", input_hash}, settings:{target, quantile_levels}}`
  returns a job; poll `GET /api/v1/jobs/<job_id>` and read `GET /api/v1/artifacts/<artifact_id>` (an
  `AnalysisArtifact`, evidence class `model_output`).
- **MCP**: `models_inspect`, then `models_forecast` with the `input_hash` of a scenario a person
  admitted in the lab.

The served list has no `example` for this release; the values below are typed, not measurements: road
dust in a bag at a DustTrak reading of 1e-5 kg/m3 (10 mg/m3), the check case of `phys_daft_w0.md`
section 7.2, where the author computes a median of 8.0e9 per m3 with ln-sd 0.9.

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d",
  "cutoff_s": 0,
  "conditions": {
    "is_cabin": 0, "is_velcro": 0, "is_bag": 1,
    "dusttrak_mass_floored": 0.00001
  },
  "controls": [],
  "observations": []
}
```

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d",
  "input": {"kind": "prefix", "input_hash": "<from the scenario>"},
  "settings": {"target": "number_concentration", "quantile_levels": [0.05, 0.25, 0.5, 0.75, 0.95]}
}
```

`settings.target` is `number_concentration` (the only target). Stored runs: no. Floor the reading
yourself: give `max(reading, 1e-9)` kg/m3.

Cite: `daft.wsp W0, hand-written (literal WSP) release sha256:061f93de…5e5d, experimental, not trained; its output is model output, not a measurement.`

## Sources

- `configs/models/served/served.json` (entry `daft.wsp`);
  `configs/models/served/models/061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d/release.json` and its params (data pack)
- `configs/models/law/daft/w0-fable/{edits.wsp,meta.json,release.json}`, `configs/models/law/daft/root/release.json`,
  `configs/models/law/daft/binding.json`, `configs/models/law/daft/decl.json`
- `crates/ignis-wsp/src/law.rs`; `crates/ignis-wsp-serve/src/lib.rs`; `crates/ignis-core/src/contracts/wsp.rs`;
  `crates/ignis-core/src/contracts/research.rs`; `tools/wsp/verify_served.py`
- `docs/wsp/research/law_forward.md`, `law_forward_targets.md` (feat/handwritten-science, 542dad8)
- `docs/wsp/research/phys_daft_w0.md`
- `docs/wsp/feedback/daft_w0_result.md`, `daft_feasibility.md`, `daft_train_anatomy.md`
- `docs/wsp/PROTOCOL.md`, sections 13.2 to 13.4, 13.7; `docs/wsp/COVERAGE_19.md`
