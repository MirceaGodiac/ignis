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

| Release | Investigation | Forecasts | Authorship | Inputs |
|---|---|---|---|---|
| `flex_methanol.wsp2` D3 | FLEX (PSI-69), methanol | droplet diameter after the cutoff (quantiles) through hot flame, cool flame, no flame | data-informed | conditions (fuel, pressure, x_o2, x_co2, x_he) and diameters before the cutoff; stored runs work |
| `flex_heptane.wsp2` D2 | FLEX (PSI-69), n-heptane | same, for n-heptane | data-informed | same; stored runs of the heptane cohort work |
| `acme_normal.wsp12` F2 | ACME CFI-G (PSI-159), Flame Design | probability over time of extinction or flow stop (event CDF) | data-informed (ethylene row zero-shot, revised once) | conditions only (fuel_species, injected_mass_flow, pressure, tad, x_fuel_role, x_o2_role, zst); stored runs answer without the planned stop |
| `acme_inverse.wsp12` F2 | same, inverse flames | same | data-informed (ethylene row zero-shot) | same |
| `bass_front.wsp` D1 | BASS (PSI-25), thin fabric | flame tip position after the cutoff (quantiles) | data-informed | x_o2, width in metres, flow history, tip positions; **stored runs are refused** (no stored run has them) |

## 2. Procedure

1. `models_inspect` the release: read `applicability` (domain), `lineage_note` and the construction
   manifest. Check the run's conditions lie in the domain before forecasting.
2. `models_forecast` with `input.kind = stored_run`, the run id and `cutoff_s` (seconds since the
   release's time origin). Use `input.kind = prefix` only with an `input_hash` a person admitted in
   the forecast lab; never assemble a prefix yourself.
3. Poll `jobs_get`; read the `AnalysisArtifact` with `artifacts_read`.
4. Pin the release in `model_releases` of `answer_submit`; claims from it are `predicted`.

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
