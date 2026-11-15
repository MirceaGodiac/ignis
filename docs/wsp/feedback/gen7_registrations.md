# Seventh generation, W0 law releases: registration of the nine blind looks (PROTOCOL 13.2)

> Committed with PROTOCOL 13.2 before any score. The blocks are the assembly engineer's draft, unchanged; the
> coordinator's decisions on the open points G1 to G12 and the rule for reading each result are in PROTOCOL 13.2.

Drafted by the assembly engineer (claude-opus-5-5), 15 November 2026, Phase 1. Facts only, from the dossiers
(`docs/wsp/research/phys_<slug>_w0.md`), the feasibility notes (`docs/wsp/feedback/<slug>_feasibility.md`) and the
input files. No release has been scored on any data. The decision rule is the coordinator's.

Common to every block:

- Forward: the generic strict law forward (`crates/ignis-wsp/src/law.rs`), binaries `target-merged/release/`.
  Each release is a revision of its slug's neutral root (`configs/models/law/<slug>/root`), author
  `claude-fable-5-2`, typed numbers transcribed verbatim (checked element by element against
  `ignis wsp inspect`, `tools/wsp/ext/law_transcription_check.py`: no difference, no number inherited from the root).
- Transcription check at the dossiers' registered expectations: `runs/dev/gen7/transcription_checks.md`
  (forward beside dossier; every difference is within the rounding of the dossier's tables, listed per slug below).
- Inputs: `tools/wsp/ext/law_inputs.py` (conditions only) writes `runs/dev/data_all/work/transfer_<slug>/law_<partition>.jsonl`.
- Admission: `tools/wsp/ext/law_admission.py` -> `runs/dev/gen7/admission_counts.json` (refusal reasons per target).
- Scorer: `tools/wsp/ext/law_score.py score --slug <slug> --release <id> --partition validation --look <section>`
  (the lane's own `score()`; aggregates to `docs/wsp/feedback/<slug>_w0_result.md`; on train also a per-case
  study file `runs/dev/data_all/work/transfer_<slug>/law_study_<id16>.jsonl`). Tested on TRAIN with baselines
  only: every baseline number below that is marked "reproduced" came out of this command equal to the
  feasibility note (outputs in `runs/dev/gen7/scorer_test/`). The release path was run once per slug on train
  with the slug's neutral ROOT (no claim, not a candidate) only to check that it runs; its numbers were not read.
- A refused case gets no forecast; the lane's scorers count it as missing (they do not score it).

---

## bre (PSI-20, ACME Burning Rate Emulator)

- **Release:** `sha256:aa0eef5bc7653922c7a7838796dab6946294e89f79c0b6c500124d9a4b212b4d` (`configs/models/law/bre/w0-fable`).
- **Exposure:** zero-shot; no file under runs/ (except the brief), data/ or holdout/; card PSI-20 aggregate
  findings and two abstracts of this experiment read (comparison only, dossier 8.3).
- **Lane target scored:** T1 flame end and cause (extinction / operator_termination), cases (test, cutoff 1, 3,
  5 s after onset) at risk, horizons 6 ... 210 s. **Score:** IPCW integrated Brier per cause, and the session
  mean over days (PROTOCOL 12.4).
- **How the law gives T1:** `flame_end` (one cause, extinction) run with `cutoff_s` = the case cutoff and times =
  the horizons plus the stop; the extinction incidence is cut at the stop and the rest of the mass is
  operator termination at the stop. The stop is the one the lane's planned-stop baseline uses:
  `planned_stop_s + 0.569 - 0.796` s (train medians from `baselines_train.json`). **Open (G1, G2 below).**
- **Baselines (feasibility section 6, train constants):** incidence per cutoff (train IBS extinction 0.2293,
  operator termination 0.0766; reproduced) and planned stop (0.2311 / 0.0688; reproduced).
- **Not scored by the lane:** T2 (heat flux to the burner) has no law target. `lasts` and `height_at_end` have
  no lane definition (height needs images).
- **Validation inputs:** 35 tests on 4 days (19051: 8, 19063: 6, 20346: 12, 20352: 9); 29 with complete inputs.
- **Refusals expected (admission, per test):** train 93 admitted / 23 refused; validation 32 / 3. Reasons:
  burner_mass_flux or x_fuel missing (no flow data), x_fuel below the declared 0.3 (7 train tests at 0.26 to
  0.28), burner_mass_flux outside [0.0008, 0.015] kg/m2/s (6 train), pressure or x_o2 missing.
- **Threats (named by the dossier and the note):** one log-logistic shape for all conditions (spread 0.5 to >2
  in the law); no held class in `flame_end` (late survival understated near the boundary); the clock is from
  ignition in the dossier, from the measured onset in the lane (about 0.8 s apart); chamber O2 is calculator
  bookkeeping and pressure reads 2.8 % below PTA3; 2019 detector lag; three train labels rest on the script's
  detector; stepwise flows not covered (first constant segment only).
- **Transcription check:** 50 condition points, tau within 0.3 %, P10 to P120 within 0.005, height within 2 %;
  "P_inf" compared at the domain end 3000 s (largest gap 0.015 at 50 mm, 40 %, 6 g/m2/s).

## spice (PSI-107, SPICE) - two candidates, kept apart

- **`w0-fable` = variant B:** `sha256:93e750242725a5a1fb9d38f4178b23605cac636f210bf9f3f8bc8e8150e2f2a8`. The 7.4
  block with the eight `smoke_point_length` lines of section 5.1 substituted in place. **What it has seen:** its
  author had read Dotson et al. 2011 (the paper on these smoke points); variant B takes no number from it for
  `smoke_point_length`. Its other targets keep the main law's lines, which contain the paper's exponents and
  A constants (`-0.91`, `0.414`, `-0.423`, `-0.224`, `1.342`): T2 (`flame_length`) is the same network in both.
- **`lit-fable` = main law:** `sha256:40c345b0453da40b38e85bd7c7441e9dc1e272ce097cf204f7bb3f4c3e35d3bf`. **What it
  has seen:** T1's centre is the investigators' published fit of the same 55 smoke points the lane scores;
  not blind for T1 (dossier section 2 and 8.3).
- (A release `w0-fable` of the main law assembled before the variants were announced is superseded and should
  not be registered; the directory was renamed to `lit-fable`.)
- **Lane targets:** T1 `L_sp_mm` from the pre-test conditions; T2 the same length given the smoke-point flow.
  **Score:** five-level quantile score x 2 (mm), mean of day means (pooled mean beside).
- **How the law gives them:** T1 = `smoke_point_length` quantiles x 1000; the row's `fuel_mass_flow` is set to
  the declared centre 1e-6 kg/s (weight zero; checked: identical forecasts at 5e-6). T2 = `flame_length` at
  `fuel_mass_flow` = `mdot_jet_mg_s` x 1e-6 (dossier 7.1). **Open (G3):** the dossier says "For T2 alone, set
  `flame_length.median.w2[2]` and `w2[3]` to 0"; no such candidate was assembled.
- **Baselines (feasibility 5):** B0 pooled climatology (in-sample 11.32 (10.58); reproduced), B1 per-fuel (7.08
  (7.94); reproduced); leave-one-day-out B0 12.72, B1 10.21 (not recomputed by the scorer). T2: Lin and Faeth
  coflow law, point 3.13, with train sigma 2.01 (not recomputed by the scorer).
- **Validation inputs:** 7 smoke points on 2 days (136: 4, 173: 3).
- **Refusals expected:** none (train 42/42, validation 7/7, both candidates).
- **Threats:** the duct limit of 115 mm (a predicted smoke point beyond it is "none observable", a scorer rule
  not applied: every lane case is an observed smoke point); one ruler calibration for all days; coflow +-10 %;
  smoke point judged by eye; 7 validation cases on 2 days.
- **Transcription check:** main law, 75 grid points within 2.6 % (length) and 3.8 % (flow); soot and length
  statements equal to rounding; variant B, three points within 0.2 %, width 0.45.

## efield (PSI-22, ACME E-FIELD Flames)

- **Release:** `sha256:73f91b55c81dc724fe2441c2254e109c65f8e08b253fcbe4c00a841f7a0986a9`.
- **Exposure (dossier 8.3, typed with train aggregates seen):** the note's 25 flame-end events among about 121
  train tests and the 9.3 uA range were seen; section 1.4 (blow-off only below saturation), `V_ext` and the
  second clock were chosen knowing the first; T2 is not blind to the train base rate. A validation video
  name "extinguished early" was in the note's header; nothing drawn from it.
- **Lane targets:** T1 `ion_current_ua` per planned level (CRPS from 5 quantiles, uA, case mean and mean of
  day means); T2 flame out under the field by 2 ... 150 s (integrated Brier over horizons with at least 5 tests)
  and `p_end_in_window` (Brier).
- **How the law gives them:** T1 = `ion_current` quantiles x 1e6 at voltage = |v_cmd| of the level. T2 = sum of
  the `blowoff` and `other` rows of `flame_end` (cutoff 0 = field on, voltage = sweep maximum) at the horizons;
  `p_end_in_window` = that sum at `window_s` (dossier 7.2).
- **Baselines (feasibility 4):** constant (T1 0.530 / 0.499 day mean; T2 IBS 0.140; end-in-window 0.165;
  reproduced); space-charge law up to saturation 0.448; stratified incidence T2 0.054 / 0.118; persistence
  0.068 (a cutoff variant, not comparable). Only the constant is built by the scorer.
- **Validation inputs:** 36 tests on 4 days (18073: 8, 18078: 9, 18285: 10, 18298: 9), 1289 planned levels.
- **Refusals expected:** validation none; train 25 levels below 100 V (outside the domain; the dossier's
  statement there is "below 0.002 uA") and 1 test.
- **Threats:** a validation stratum (gas jet, negative, ethylene, day 18285) has no train counterpart; corona
  above about 8 kV and the burner type are not in the law; planned vs measured fuel flow (+8 % median); one clock
  per cause sets the time profile crudely; ADC clipping removes the most ionised levels.
- **Transcription check:** saturation table within 3 %, T1 tables within 1.7 % (ln-sd within 0.005), T2 table
  and time series within 0.005.

## samer (PSI-101, SAME-R)

- **Release:** `sha256:05f77ae2f7a666e0da69d04fa29d9ad8b0ac15790645abe1adc0149292327fb0`.
- **Exposure (dossier 8.3):** card findings used as class L for `T_c`, `t_c` and the `sigma_0` grouping; the
  note's train facts (full-scale incidence by material, mass concentrations 2 to 350 mg/m3) read after the
  constants were typed, no number changed; pretraining may include the published SAME tables.
- **Lane targets:** `d10` at `net_aging_time` (quantile CRPS of ln d10, aggregate over cases, day-equal
  beside); `iss_full_scale` (Brier). The law's `d30` is not scored by the lane.
- **How the law gives them:** `d10` quantiles (m); `iss_full_scale` probability. The optional aged-row law of
  7.5 (`samer_aged`, needs its own slug and the `prior` the evaluator hands over) is not assembled.
- **Baselines (feasibility 6):** B0 constant in-sample 0.3229 (day-equal 0.2807; reproduced), LODO 0.3437; B1
  per material 0.2454 (0.2369; reproduced), LODO 0.3627; aged rows: P persistence 0.3849, C coagulation 0.2121.
  ISS: I0 0.2066 (reproduced), LODO 0.2165; I1 0.0312 (reproduced), LODO 0.0534.
- **Validation inputs:** 24 rows on 3 days (2010-07-21: 2, 2010-09-07: 10, 2010-09-08: 12).
- **Refusals expected:** validation 1 (net_aging_time missing); train 2 (net aging time "n/a" on runs 52 and
  57; 57 also no sample temperature).
- **Threats:** most of the error is common to all tests of one material; P-Trak undercounting of large smoke;
  run 69 d30 < d10; one validation run heated 120 s; the agent that built the cases saw condition cells of four
  validation rows.
- **Transcription check:** all section 5 tables within 0.4 % (sizes) and 0.002 (ln-sd). Probabilities: the
  rows "2 or 1 cm/s" are the 2 cm/s values; at 1 cm/s the law's linear flow term gives 0.86, 0.83, 0.81 for
  lamp wick, silicone, Pyrell against the table's 0.85, 0.81, 0.80 (largest 0.016; the dossier's section 3
  text says "+0.10 per cm/s below 8.1", which is what the network does).

## same (PSI-102, SAME, held whole)

- **Release:** `sha256:e1362b55270238360ec6744cfea6225db42d0fe019f6ff1c77dd96cede5f2062`.
- **Exposure:** as SAME-R 8.3; the SAME note holds no outcome; the card's 50 % and 300 nm were known when the
  flow exponent and `D_ref` were typed; the DBP row owes nothing to any SAME result (dossier 8).
- **Lane targets and score:** as SAME-R (`samer_cases.score` on `cases_held_outcomes.jsonl`); one held group,
  so the per-day score is the aggregate.
- **Baselines (same_feasibility 5, SAME-R train constants, not computed on SAME):** B0; B1 per material with B0
  for DBP; C coagulation from the primary row (evaluator passes the prior); I0, I1 (I0 for DBP). The scorer
  builds B0/I0 and B1/I1; its plumbing was tested on SAME-R train rows (B1 0.2454 / I1 0.0312, as samer).
- **Held inputs:** 45 rows, 1 group (`PSI-102:matrix`); the note counts 25 scored for d10 and 45 for ISS.
- **Refusals expected:** none (45/45). The law does not use the sample temperature (4 rows lack it).
- **Threats:** d10 missing on 20 of 45 rows; two DustTraks and two ionisation chambers in this campaign; DBP is
  a liquid aerosol with no SAME-R counterpart; condition cells of SAME rows 18, 19, 24 seen by the lane agent.
- **Transcription check:** within 0.4 % (sizes), 0.005 (P); both 7.2 check values reproduced.

## daft (PSI-47, DAFT / DAFT-2, held whole)

- **Release:** `sha256:061f93de68e66b27606d99cae054c5bd03ddb136c03decc81489cad8b66f5e5d`.
- **Exposure:** card findings told the sign of two forecasts, no number; the note holds no outcome in the
  sections read; no web source opened (dossier 8).
- **Lane target:** number concentration N read by the P-Trak; quantile CRPS of ln(1 + N / 1e6 m^-3);
  aggregate (dominated by the 42 bag readings) with the group-equal mean beside.
- **How the law gives it:** `number_concentration` quantiles (1/m3), with `dusttrak_mass_floored` =
  max(reading, 1e-9 kg/m3) as the dossier asks.
- **Baseline (daft_feasibility 5):** smoke number per mass from SAME-R train, `N_q = k_q * M`
  (`baseline_smoke_number_per_mass.json`); not scored on anything yet. No train partition exists, so the scorer's
  daft path was not run on data in Phase 1 (its forecast assembly is the one the admission run exercised).
- **Held inputs:** 53 cases in 2 groups (session D2: 11, bags 2006-08-21: 42).
- **Refusals expected:** none (53/53).
- **Threats:** cabin readings at the photometer's floor; bag readings voice-downlinked; dust grade not stated;
  two groups only.
- **Transcription check:** medians within 0.4 %; core quantiles within 6 % (the dossier's two-digit entries; the
  forward's own quantiles include the 15 % tail, listed beside).

## bass1 (PSI-26 BASS and PSI-25 BASS-II non-fabric solids)

- **Release:** `sha256:97c0daf9de84482ed7b125b3d41302521f5dd7281c0667241807d93387e239a2`.
- **Exposure (dossier 8.3):** blind to the project's files; not blind to the literature on its experiment
  (two numbers, `x_b` and the working volume, set from published summaries of the same burns).
- **Lane targets:** `quench_u` (cm/s, interval outcome; interval CRPS from quantiles at 5, 25, 50, 75, 95 %;
  mean of day means primary, case mean beside); `sustain` (Brier).
- **How the law gives `quench_u` (settled by dossier 7.1/7.2):** rods, spheres and spherical sections whose nose
  faces the flow (configuration opposed, stagnation, forward stagnation): `quench_speed` quantiles x 100 at
  x_o2 = `x_o2_initial`, nose_radius = `diameter_m` / 2, heat_time = 200 s, is_concurrent = 0. Films and slabs
  with a stated direction: `flat_quench_speed` quantiles x 100 at nose_radius 0.01, is_concurrent from the
  configuration. Both targets give `flow_speed` weight zero; it is filled with `u_cm_s_at_ignition` / 100.
- **`sustain`: not mapped (G5).** The scorer holds the rule "holds = the smaller of the low-flow and high-flow
  targets of the geometry" (`law_score.holds`) but does not score `sustain`.
- **heat_time default:** no case records it; 200 s supplied for every covered case: 72 train, 28 validation.
- **Baselines (feasibility, train constants):** quench constant quantiles 0.66, 1.2, 1.44, 1.66, 9.06 cm/s
  (day mean 0.5518, case mean 0.7714; reproduced); Bhattacharjee et al. 2016 critical velocity with one fitted
  log width 0.40 (0.6134 / 0.8947; reproduced); sustain incidence p = 0.888 (0.0881 / 0.0995), always lit.
- **Validation inputs:** 41 burns on 12 days; the note counts 11 with `quench_u` and 40 with `sustain`.
- **Refusals expected:** train 59 of 131, validation 13 of 41, all "not covered": candles, wax, fabrics,
  Ultem, spheres in the wake, rods in a concurrent or unstated configuration, flat samples with no stated
  direction (counts by kind in `admission_counts.json`, `by_coverage`). On the 55 train cases the lane scores
  for `quench_u`, 39 are covered (the neutral-root plumbing run).
- **Threats:** O2 on PSI-25 is offset to the ISS MCA (+-0.015); PSI-26 uses the 2014 fan calibration; the flow
  history is part of the outcome; low settings read 0 to 4 cm/s with a one-minute lag; re-lit warm samples.
- **Transcription check:** E1 to E6 within 0.5 % (oxygen limit), 0.005 (probabilities), 4 % on one-digit entries
  (quench and blow-off speeds), 0.6 % (flat quench).

## slice (PSI-106, SLICE)

- **Release:** `sha256:d10c94bc6ea0bcb957f89734393b0cbabbd44fdf0ac9c2e440f051a33cd3006e`.
- **Exposure (dossier 8.3):** Cao et al. 2015 (this experiment) read: of 18 microgravity flames on 3.2 mm at low
  coflow none lifted; the mechanism and threshold were written before reading it; no blow-off or lifting limit
  of the flight seen; from the note the stepping procedure, tubes and input names.
- **Lane targets:** `blow_off` per logged line (Brier, mean of day means, pooled beside); `lifted` (Brier).
- **What one case is (checked, as asked):** the note: "A case is one logged line: one (fuel flow, fan) setting
  of one flame", stepped up line by line until blow-off; ineligible lines not scored. The law: one line, with a
  fixed 10 % step built into the per-line level ("the forward sees one line"). They agree on the unit. They
  differ on one thing the dossier states: the law cannot see the previous line, so a large jump is given too
  low a probability and a repeated setting too high.
- **How the law gives them:** `blow_off` and `lifted` probabilities directly.
- **Baselines (feasibility 5):** incidence pooled 0.0535 (0.0504) / lifted 0.0258 (reproduced); leave one day
  out 0.0544 / 0.0263; by nominal tube 0.0516 / 0.0249 (reproduced); "never" 0.0567 / 0.0265.
- **Validation inputs:** 75 lines on 2 days (065: 34, 075: 41).
- **Refusals expected: all 75 validation lines (G6).** No validation line has a cabin oxygen value
  (`x_o2_cabin` null on both days), and day 075 is 20 % ethylene (41 lines), a fuel the law does not cover
  (dossier 8.2) and whose fraction is below the declared 0.3. Train: 273 of 289 admitted (14 without oxygen,
  2 without coflow speed).
- **Threats:** blow-offs concentrated on three train days; fuel and fan calibrations +-10 to 15 %; the log is
  read by position from a PDF, partly unaudited.
- **Transcription check:** both tables and the oxygen line within 0.005; length 2 % (check-point jet speed from
  a stated density).

## cld (PSI-21, ACME CLD Flame)

- **Release:** `sha256:0995df666bdabfa546607fcba565da166a28839ed984f2f1226bc62274eefe4e`.
- **Exposure (dossier 8.3, typed with train aggregates seen; not a blind zero-shot):** the note's train facts
  were read: 20 of 207 flames lost and in which stage, 9 of 11 dilution tests lost against 11 of 196, loss times
  clustering at 0.9 to 2.4 s and 10 to 38 s, radiometer excess ranges (methane 0.14 to 1.8 V, ethylene 0.16 to
  12.4 V). Changed: `Phi` = 2.5 chosen in its range knowing few flames are lost; the small clock's level (4 % in
  120 s) set knowing 11 of 196; `c_V` = 0.5 V/W set from the stated ranges (the radiometer level is taken from
  train data). The mechanism was carried from SLICE before the note was read.
- **Lane targets:** `loss` (`p_loss_by` at 1 ... 120 s after onset, integrated Brier over the horizons at risk;
  `p_loss_before_fuel_off`, Brier); `radiometer` (quantiles of log10 V, CRPS).
- **How the law gives them:** `loss` cause rows (blowoff + other) at the horizons, with the planned fuel-off
  declared as the scheduled stop (condition `planned_fuel_off_from_onset` = `at_cutoff.planned_fuel_off_from_onset_s`;
  see G7); `p_loss_before_fuel_off` = the sum at the fuel-off time. `radiometer` = log10 of the forward's volt
  quantiles. Features 0 and 1 are the lowest fraction and highest coflow of the schedule (dossier 7.1).
- **Baselines (feasibility, train):** incidence (KM) IBS 0.1506, Brier-any 0.0926 (day means 0.1290 / 0.1272;
  reproduced); by fuel, by dilution ramp, persistence, classical LFL (in `baselines_train.json`); radiometer
  constant CRPS 0.2328 (reproduced), by fuel 0.2096, persistence 0.1324, Roper length + fuel offset 0.1166.
- **Validation inputs:** 72 test points on 6 days (18003: 10, 18008: 11, 18032: 13, 18177: 11, 18249: 12,
  18253: 15); 69 usable.
- **Refusals expected:** validation 11 of 72 (no planned fuel-off from onset, i.e. no onset record; 1 also
  without a schedule); train 20 of 218 (19 schedules without a constant-flow stage: x_fuel_stage missing; the
  loss target gives those features weight zero but the forward needs them; 11 without onset record).
- **Threats:** the loss clock reads the most severe condition and the window, not the schedule; ignition-
  retraction losses folded into the small clock; the radiometer has no type, distance or gain documented.
- **Transcription check:** loss table and time series within 0.005; radiometer table within 8 % on two-digit
  entries (0.07 vs 0.067, 0.06 vs 0.055).

---

## Open points for the physicists (the engineer stopped on these; nothing was chosen)

- **G1 (bre, clock origin):** the law's time origin is ignition; the lane's T1 clock is the measured onset
  (0.3 to 1.7 s later, median 0.80 s). The scorer uses the forward's times as onset times unchanged.
- **G2 (bre, operator termination):** the law has one cause; T1 scores two. The scorer cuts the law's
  extinction incidence at the stop the lane's planned-stop baseline uses (`planned_stop_s + 0.569 - 0.796`, two
  train medians of the data lane). That stop is the data lane's, not the dossier's; to be confirmed or replaced.
  Tests with no planned stop (21 train cases) get no operator-termination mass.
- **G3 (spice, T2):** "For T2 alone, set `flame_length.median.w2[2]` and `w2[3]` to 0": a third candidate
  (main law with those two zeros) was not assembled. As registered, T2 is scored on `flame_length` as typed in
  both candidates (identical in both).
- **G4 (spice, variant B coverage):** variant B changes only `smoke_point_length`; its `smoke_point_fuel_flow`
  and `emits_soot` still carry the paper's constants (they are not scored by the lane).
- **G5 (bass1, `sustain`):** not determined by the dossier. Candidate reading: min(`holds_low_flow`,
  `holds_high_flow`) (or the flat pair) at the ignition setting; but at which `heat_time` (the step is the
  igniter period, not a 60 s hold), and the dossier's "holds" requires a step held at least 60 s while
  `sustain` is "kept burning after the igniter". The igniter time `t_min` is outside the forward (7.5).
- **G6 (slice, validation oxygen and ethylene):** no validation line has cabin oxygen; the dossier allows the
  day's value or 0.21 only as "an assumption to record". Without a decision all 75 validation lines are refused.
  Day 075 is 20 % ethylene, outside the law (8.2) and below x_fuel 0.3: refused whatever the oxygen.
- **G7 (cld, stop declaration):** section 7.2 makes the stop conditional ("If the planned fuel-off is declared
  as the scheduled stop ..."); the typed block of 7.4 contains the two stop lines, which exist only with the
  declared stop. The declaration was written with the stop (condition `planned_fuel_off_from_onset`) so that the
  block compiles verbatim. A declaration is frozen once a release exists: confirm before committing.
- **G8 (cld, no constant-flow stage):** 19 train schedules have no `constant_flow` segment, so `x_fuel_stage` and
  `v_fuel_stage` are undefined and the forward refuses the case for both targets (the loss target gives them
  weight zero). A rule for these schedules is the physicist's.
- **G9 (bass1, rods in concurrent flow and configuration null):** treated as not covered (not a nose facing the
  flow). 6 + 2 train rods; confirm.
- **G10 (samer, "2 or 1 cm/s" rows):** the table merges 1 and 2 cm/s; the network gives a slightly higher
  probability at 1 cm/s (up to 0.016). Nothing to change in the numbers; note for the record.
- **G11 (tests of the forward):** `crates/ignis-wsp/tests/law_forward.rs` hard-codes `bre` (and `bass1`) as
  placeholder slugs in `the_demo_spec_has_the_documented_tensors` (line 193) and
  `a_placeholder_cannot_be_assembled` (line 205). Both fail now that these slugs are declared; every other
  suite of `cargo test -p ignis-wsp` passes. The engineer may not edit the tests. The only placeholder left is
  `sflame_event`; the fix is to use it alone in both tests (in the first, `excludes` of an all-lane-local
  placeholder), or a permanent placeholder slug for the tests (one line in `law_slugs.rs` and a placeholder
  `decl.json`).
- **G12 (reassemble):** the new lineages are not in `tools/wsp/reassemble.py` CHAINS (outside the engineer's
  files): add `("<slug>", "configs/models/law/<slug>", "binding.json", ["root", "w0-fable"])` for the nine slugs
  and `["root", "lit-fable"]` for spice.
