---
name: model-qualification
description: Answer predictive questions with the served hand-written model releases (FLEX droplets, ACME burner flames, BASS flame front) through models_inspect and models_forecast, stating each release's qualification, authorship and limits, and reporting refusals instead of numbers.
---

# Hand-written model releases

## 1. What is served

The model tools serve hand-written (literal weight-space) releases: every weight was typed by an
author model, nothing was fitted, the forward cannot fit at request time. Every one is
`trained_on = not_trained`, `state = experimental`. Their output is `model_output`, never a
measurement. The exact list, ids and inputs are in the descriptions of `models_inspect` and
`models_forecast` (the deployment's served list); use those ids, never one from memory.

The names in the tables below (`flex_methanol.wsp2`, ...) are model names, **not** release ids:
every `release_id` argument is the `sha256:...` id the served list gives for that name. A
`not_found` that names a model name tells you its id: call again with it; it never means the
model is not served.

| Model name | Investigation | Forecasts | Authorship | Inputs |
|---|---|---|---|---|
| `flex_methanol.wsp2` D3 | FLEX (PSI-69), methanol | droplet diameter after the cutoff (quantiles) through hot flame, cool flame, no flame | data-informed | conditions (fuel, pressure, x_o2, x_co2, x_he) and diameters before the cutoff; stored runs work |
| `flex_heptane.wsp2` D2 | FLEX (PSI-69), n-heptane | same, for n-heptane | data-informed | same; stored runs of the heptane cohort work |
| `acme_normal.wsp13` F3 stage 1 | ACME CFI-G (PSI-159), Flame Design, s-Flame nitrogen flames | same | data-informed | conditions; optional burner temperature limit of the script (`burner_limit_c`) and diluent; refuses helium |
| `acme_inverse.wsp12` F2 | same, inverse flames | same | data-informed (ethylene row zero-shot) | same |
| `bass_front.wsp` D1 | BASS (PSI-25), thin fabric | flame tip position after the cutoff (quantiles) only; not its quench or ignition outcome | data-informed | x_o2, width in metres, flow history, tip positions; **stored runs are refused** (no stored run has them) |

### The other investigations: generic law releases (law.rs)

Typed conditions only (stored runs are refused). Pick a target with `settings.target`; event targets
need `settings.times_s`. Scores below are the lane's test days of these investigations, scored once
on a frozen list, lower is better, beside the baseline:

| Model name | Investigation | Use it for | Test-day score against baseline |
|---|---|---|---|
| `bre.wsp` D3 | BRE (PSI-20) | target `lasts` only (`flame_end` is refused: it is not the model). Extinction by time h = 1 - lasts(h) / lasts(cutoff) | extinction 0.139 vs incidence 0.222 (95 cases) |
| `cld.wsp` D1 | CLD Flame (PSI-21) | flame lost, radiometer level | 0.073 vs 0.102; 0.115 vs 0.144 (63 cases). D1 was chosen after the test run, where the newer law lost on timing |
| `efield3.wsp` S1 | E-FIELD (PSI-22) | ion current, flame out in window | 0.319 µA vs space-charge 0.575 (788 levels); 0.055 vs 0.133 |
| `bass4.wsp` S1 | BASS / BASS-II plastics and fabrics | does the sample keep burning (targets `flat_holds_*` for films, slabs, fabrics; `holds_*` for rods, spheres) | 0.048 vs 0.071 (42 burns, fabrics included) |
| `slice2.wsp` S1 | SLICE (PSI-106) | blow-off, lifting = lift_noted x (past_lift - past_blow) / (1 - past_blow) | 31 test lines with no blow-off or lifting event: the test says little |
| `spice.wsp` P3 | SPICE (PSI-107) | smoke-point length, flame length | 4.18 mm vs 10.3; 1.18 vs 2.56 (6 points); not blind to the published fit |
| `samer.wsp` S1 | SAME-R (PSI-101) | particle diameter, detector at full scale | 0.167 vs 0.256; 0.014 vs 0.019 (33 rows) |
| `same.wsp` W0 | SAME (PSI-102) | particle diameter, detector | held whole: 0.184 vs 0.192; 0.059 vs per-material 0.055 |
| `daft.wsp` W0 | DAFT (PSI-47) | number concentration | held whole: 0.777 vs 2.678 |

Transfers of the family releases: the n-heptane FLEX model covers FLEX-2 (PSI-68); the normal
burner-flame F3 covers s-Flame nitrogen flames (validation 0.205 vs incidence 0.279; its test days are
helium, which it refuses); the BASS front model covers Saffire I to III as blind transfers. A zero-shot
n-dodecane model for CFI (PSI-39) exists but is not served. Report a target that did not beat its
baseline as such.

**Not forecast, refuse with this reason:** the flow speed at which a sample goes out (quench speed):
the flight record gives a fan setting, not a calibrated speed, below about 4 cm/s, and no model beat
a constant on held data. Do not use `bass_front.wsp` for whether a fabric keeps burning, its quench
or its ignition outcome (worse than a constant: 0.170 against 0.125); use it for the flame-tip
position only, and `bass4.wsp` for kept burning.

## 2. Procedure

1. `models_inspect` the release: read `applicability` (domain), `lineage_note` and the construction
   manifest. Check the run's conditions lie in the domain before forecasting.
2. `models_forecast` with `input.kind = stored_run`, the run id and `cutoff_s` (seconds since the
   release's time origin). Use `input.kind = prefix` only with an `input_hash` a person admitted in
   the forecast lab; never assemble a prefix yourself.
3. Poll `jobs_get`; read the `AnalysisArtifact` with `artifacts_read`.
4. List the release id in `model_releases` of `answer_submit`; claims from it are `predicted`.
   A release your own successful `models_forecast` of this episode ran is accepted; you cannot
   pin one yourself, and a release you did not run here must be pinned by the person.

## 3. Refusals

A forward refuses outside its domain and names the reason: `support.admitted = false` with
`support.reasons`, or status `unsupported` / `insufficient_inference`, or a failed job (missing
input, unknown prefix, `scope_denied` for a test-partition run). Report the refusal and its reason
as the finding. Never substitute a number, another release's output, or an estimate of your own.

## 4. How to state it

- Name the release (model name, candidate, release id) and say: hand-written, experimental, not
  trained, zero-shot or data-informed as listed.
- Development scores, as the authoring lane reported them (validation partitions read as aggregates,
  so they are development numbers, not a test): methanol D3 CRPS 40.0 µm; heptane D2 46.1 µm; normal
  flames F2 integrated Brier 0.154 extinction / 0.096 flow termination on CFI-G validation without
  the planned stop; inverse flames F2 0.129 / 0.088; BASS D1 CRPS 0.117 sample widths, with a quench
  boundary that fails (0.170 against 0.125 for a constant). Source: `docs/wsp/README.md`.
- Limits to carry when relevant: droplet tracker artefacts are scored as data; burner-flame
  validation days are few and ethylene below 1900 K is refused; BASS scores are in sample widths and
  its quench law must not be read as an ignition forecast.
- A forecast on a train run is inspection, not a test: a data-informed release's author read it.
