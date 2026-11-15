---
name: model-qualification
description: Answer predictive questions with the served hand-written model releases (FLEX droplets, ACME burner flames, BASS flame front, and generic law releases for other PSI investigations) through models_inspect and models_forecast, stating each release's state, authorship and limits, and reporting refusals instead of numbers.
---

# Hand-written model releases

## 1. What is served

If `models_inspect` is not among your tools, this deployment serves no forecasts: say so in one
line (forecasts are not served in this deployment) and answer the rest of the question from the
corpus, the conditions and the data.

The model tools serve hand-written (literal weight-space) releases: every weight was typed by an
author model, nothing was fitted, the forward cannot fit at request time. Every one is
`trained_on = not_trained`, `state = experimental`. Their output is `model_output`, never a
measurement. The exact list, ids and inputs are in the descriptions of `models_inspect` and
`models_forecast` as the MCP tool definitions give them (the deployment's served list), and a
`not_found` refusal lists them again; use those ids, never one from memory.

The names in the tables below (`flex_methanol.wsp2`, ...) are model names, **not** release ids:
every `release_id` argument is the `sha256:...` id the served list gives for that name. A
`not_found` that names a model name tells you its id: call again with it; it never means the
model is not served.

| Model name | Investigation | Forecasts | Authorship | Inputs |
|---|---|---|---|---|
| `flex_methanol.wsp2` D3 | FLEX (PSI-69), methanol | droplet diameter after the cutoff (quantiles) through hot flame, cool flame, no flame | data-informed | conditions (fuel, pressure, x_o2, x_co2, x_he) and diameters before the cutoff; stored runs work |
| `flex_heptane.wsp2` D2 | FLEX (PSI-69), n-heptane | same, for n-heptane | data-informed | same; stored runs of the heptane cohort work |
| `acme_normal.wsp13` F4 (f4-hydrogen) | ACME CFI-G (PSI-159), Flame Design (PSI-10), s-Flame (PSI-23) | probability over time that the flame ends by extinction or by a flow stop | data-informed; methane, helium and hydrogen parts typed after the final run with every partition seen (in-sample on all data; no held-out score) | conditions incl. `diluent_species` (nitrogen or helium); optional `burner_limit_c` (degC), `x_o2_role_b`/`tad_b`/`zst_b` for a day with two oxygen candidates; **blends refused; flow-ramped tests outside the model** |
| `acme_inverse.wsp12` F2 | same, inverse flames | same events, inverse flames | data-informed (ethylene row zero-shot) | same |
| `bass_front.wsp` D1 | BASS (PSI-25), thin fabric | flame tip position after the cutoff (quantiles) only; not its quench or ignition outcome | data-informed | x_o2, width in metres, flow history, tip positions; **stored runs are refused** (no stored run has them): only a scenario a person admitted |

### The other investigations: generic law releases (law.rs)

Typed conditions only: stored runs are refused, and a forecast needs `input.kind = prefix` with the
`input_hash` of a scenario a person admitted in the forecast lab (you cannot admit or assemble one).
With such an `input_hash`, pick a target with `settings.target` (default the first); event targets
need `settings.times_s` or `horizon_s`. Without one, answer from the release itself: name it, its
required inputs (the served list), its scores below, and say a person can type the scenario in the
forecast lab; do not call `models_forecast` on a stored run of these releases.

Scores in this skill are recorded by IGNIS (the served list and the authoring lane's reports), not
returned by a tool in this episode: when you use one, say so ("score recorded by IGNIS") in an
`interpretation` claim or in `limitations`, never as a cited claim. A forecast, a score or a run's
outcome is never a `background` claim from your own knowledge. Lower is better; each is beside
its baseline. Unless the row says otherwise it is from the investigation's test days, scored once on
a frozen list; "held whole" means the whole investigation was kept from the author and scored once.

| Model name | Investigation | Use it for | Recorded score against baseline |
|---|---|---|---|
| `bre.wsp` D3 | BRE (PSI-20) | target `lasts` only (`flame_end` is refused: it is not the model). Extinction by time h = 1 - lasts(h) / lasts(cutoff) | extinction 0.139 vs incidence 0.222 (95 cases) |
| `cld.wsp` D1 | CLD Flame (PSI-21) | flame lost, radiometer level | 0.073 vs 0.102; 0.115 vs 0.144 (63 cases). D1 was chosen after the test run, where the newer law lost on timing |
| `efield3.wsp` S1 | E-FIELD (PSI-22) | ion current, flame out in window | 0.319 µA vs space-charge 0.575 (788 levels); 0.055 vs 0.133 |
| `bass5.wsp` S1 | BASS / BASS-II plastics and fabrics | kept burning (`flat_holds_*`, `holds_*`) and the quench speed (`quench_speed`, `flat_quench_speed`); typed after the final run with every partition seen; in-sample on all data; no held-out score exists for the quench speed | kept burning 0.048 vs 0.071 (42 burns, held, fabrics included); quench speed in-sample 0.114 / 0.108 / 0.126 cm/s vs constant 0.671 / 0.547 / 0.916 (train / validation / test, corrected coding) |
| `slice2.wsp` S1 | SLICE (PSI-106) | blow-off, lifting = lift_noted x (past_lift - past_blow) / (1 - past_blow) | 31 test lines with no blow-off or lifting event: the test says little |
| `spice.wsp` P3 | SPICE (PSI-107) | smoke-point length, flame length | 4.18 mm vs 10.3; 1.18 vs 2.56 (6 points); not blind to the published fit |
| `samer.wsp` S1 | SAME-R (PSI-101) | particle diameter, detector at full scale | 0.167 vs 0.256; 0.014 vs 0.019 (33 rows) |
| `same.wsp` W0 | SAME (PSI-102) | particle diameter, detector | held whole: 0.184 vs 0.192; 0.059 vs per-material 0.055 |
| `daft.wsp` W0 | DAFT (PSI-47) | number concentration | held whole: 0.777 vs 2.678 |

Transfers of the family releases: the n-heptane FLEX model covers FLEX-2 (PSI-68); the normal
burner-flame F4 covers s-Flame: nitrogen flames as F3 (validation 0.205 vs incidence 0.279), helium
and hydrogen answered in-sample (helium validation 0.111 vs 0.330; helium test days without ramped
tests 0.242 vs 0.414, with them 0.708 vs 0.481; hydrogen train 0.314 vs 0.909); a hydrogen and
methane blend is refused (no blend law) and a test whose flow is ramped after ignition is outside
the model; the BASS front model covers Saffire I to III as blind transfers. A zero-shot n-dodecane
model for CFI (PSI-39) exists but is not served. Report a target that did not beat its
baseline as such.

**Quench speed:** answered by `bass5.wsp`, but say it was typed after the final run with every
partition seen (in-sample, no held-out score), and that below about 4 cm/s the recorded outcome is a
fan setting, not a calibrated speed. Do not use `bass_front.wsp` for whether a fabric keeps burning,
its quench or its ignition outcome (worse than a constant: 0.170 against 0.125); use it for the
flame-tip position only, and `bass5.wsp` for kept burning.

## 2. Procedure

1. `models_inspect` the release: read `applicability` (domain), `lineage_note` and the construction
   manifest. Check the run's conditions lie in the domain before forecasting.
2. `models_forecast` with `"schema_version": "wsp.v1"`, the `release_id` and `"input": {"kind":
   "stored_run", "run_id": ..., "cutoff_s": ...}`: `cutoff_s` is seconds after the run's time origin
   (the phase entry of the curated views; `datasets_inspect` states it). Pick a cutoff where the run
   is still burning and has several measurements before it. Use `input.kind = prefix` only with an
   `input_hash` a person admitted in the forecast lab; never assemble a prefix yourself.
3. A quick forecast answers `completed` with its output; otherwise call `jobs_get` once (it waits
   for the job) and again only while it is still running. Read the `AnalysisArtifact` with
   `artifacts_read`. Check it before you state it: its release id, run, cutoff, `support.admitted`, units and time grid. To compare with data,
   read the run's measurements after the cutoff from its dataset view (an `analysis_execute`
   computing the error is a `computed` claim).
4. List the release id in `model_releases` of `answer_submit`; claims from it are `predicted`.
   A release your own successful `models_forecast` of this episode ran is accepted; you cannot
   pin one yourself, and a release you did not run here must be pinned by the person.

## 3. Refusals

A forward refuses outside its domain and names the reason: `support.admitted = false` with
`support.reasons`, or status `unsupported` / `insufficient_inference`, or a failed job (missing
input, unknown prefix, `scope_denied` for a test-partition run). Report the refusal and its reason
as the finding. Never substitute a number, another release's output, or an estimate of your own.

A refusal can belong to the run or the cutoff, not the release. "Modelled phase not established at
the cutoff" means the run was no longer confirmed burning at that cutoff (its flame had ended, or no
measurement reaches it): try an earlier cutoff. A run outside the domain or with too few points
before the cutoff is the run's refusal. When the researcher asked for *a* run or *some* runs rather
than a named one, keep trying other eligible runs of the same release (pick them with
`conditions_query` or the dataset's runs) until forecasts ran or the eligible runs are exhausted, and
report every refusal with its reason next to the forecasts that ran. When they named the run, try a
valid cutoff for it; if the run itself is refused, that refusal is the answer.

## 4. How to state it

- Name the release (model name, candidate, release id) and say: hand-written (its weights were
  written by an AI author model), not trained, zero-shot or data-informed as listed, release state
  `experimental`. `experimental` is an IGNIS release state, not a NASA credibility level: no
  NASA-STD-7009 credibility assessment or independent technical review has been done.
- State the run id (or scenario input_hash), cutoff and input source of every forecast in the claim
  text.
- Report each forecast as its median with the quantile band the artifact gives (e.g. 10 to 90 %),
  never the median alone; if the artifact has no band, say no uncertainty estimate is available for
  this forecast. Say how the band was obtained: the release's hand-written quantile law, not
  calibrated on held-out data unless a test score is listed.
- Relay in `limitations` every warning or `support.reasons` entry of an admitted forecast and every
  listed limit that applies (in-sample, typed after the final run, baseline not beaten), each with
  one line on how it changes the reading.
- Development scores recorded by IGNIS (validation partitions read as aggregates, so development
  numbers, not a test; state them as recorded, see section 1): methanol D3 CRPS 40.0 µm; heptane D2
  46.1 µm; normal flames F4: its nitrogen ethane, propane, butane and ethylene forecasts are those of
  F3 stage 1, integrated Brier 0.154 extinction / 0.096 flow termination on CFI-G validation without
  the planned stop, methane in nitrogen 0.092 (validation days, sum); inverse flames F2 0.129 /
  0.088; BASS D1 CRPS 0.117 sample widths, with a quench boundary that fails (0.170 against 0.125
  for a constant).
- Limits to carry when relevant: droplet tracker artefacts are scored as data; burner-flame
  validation days are few and ethylene below 1900 K is refused; BASS scores are in sample widths and
  its quench law must not be read as an ignition forecast.
- A forecast on a train run is inspection, not a test: a data-informed release's author read it.
