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
| `acme_normal.wsp12` F2 | ACME CFI-G (PSI-159), Flame Design | probability over time of extinction or flow stop (event CDF) | data-informed (ethylene row zero-shot, revised once) | conditions only (fuel_species, injected_mass_flow, pressure, tad, x_fuel_role, x_o2_role, zst); stored runs answer without the planned stop |
| `acme_inverse.wsp12` F2 | same, inverse flames | same | data-informed (ethylene row zero-shot) | same |
| `bass_front.wsp` D1 | BASS (PSI-25), thin fabric | flame tip position after the cutoff (quantiles) | data-informed | x_o2, width in metres, flow history, tip positions; **stored runs are refused** (no stored run has them) |

### The other investigations: generic law releases (law.rs)

Nine releases run on one generic strict forward over declared features (typed conditions only; no
stored run carries the features, so a stored-run forecast is refused). Pick a target with
`settings.target`; event targets also need `settings.times_s`. Validation or held-whole scores,
lower is better, beside the registered baseline (`docs/wsp/COVERAGE_19.md`):

| Model name | Investigation | Authorship | Score against baseline |
|---|---|---|---|
| `bre.wsp` W0 | BRE (PSI-20) | zero-shot | extinction 0.2755 vs incidence 0.2274: **loses**; termination 0.1383 vs 0.1799 |
| `cld.wsp` D1 | CLD Flame (PSI-21) | train aggregates seen + one step from train days | flame lost 0.514 vs 0.478: **loses**; radiometer 0.100 vs 0.174 |
| `efield.wsp` W0 | E-FIELD (PSI-22) | train aggregates seen | ion current 0.546 µA vs constant 0.620 (space-charge 0.374); flame out 0.0753 vs 0.0884 |
| `bass1.wsp` D1 | BASS / BASS-II plastics (PSI-26, PSI-25) | zero-shot law + one step from train days | quench speed 0.507 vs 0.230: **loses**; kept burning 0.086 vs 0.120 |
| `daft.wsp` W0 | DAFT (PSI-47) | zero-shot | number concentration 0.777 vs 2.678 (held whole) |
| `samer.wsp` W0 | SAME-R (PSI-101) | zero-shot | diameter 0.167 vs 0.339; detector at full scale 0.228 vs 0.247 |
| `same.wsp` W0 | SAME (PSI-102) | zero-shot | diameter 0.184 vs 0.192; detector 0.059 vs per-material 0.055 (held whole) |
| `slice.wsp` W0 | SLICE (PSI-106) | zero-shot | blow-off 0.155 vs 0.161 (by-tube 0.149); lifting 0.089 vs 0.056: **loses** (one day) |
| `spice.wsp` W0 | SPICE (PSI-107) | zero-shot | smoke-point length 5.86 mm vs constant 6.10 (per-fuel 5.04); 7 points |

Transfers of the family releases: the n-heptane FLEX model covers FLEX-2 (PSI-68), 20.9 µm vs 32.6
on 8 tests; the normal burner-flame F2 covers Flame Design (PSI-10) and s-Flame (PSI-23), where it is
**undecided** and refuses helium, hydrogen and blends; the BASS front model covers Saffire I to III
(PSI-98 to 100) as blind transfers on two sheets. A zero-shot n-dodecane model for CFI (PSI-39)
exists but is not served. A target marked **loses** must be reported as losing to its baseline.

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
