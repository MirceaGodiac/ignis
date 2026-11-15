# s-Flame validation days: F3 stage 1 against F2 (PROTOCOL 13.5 (a)), run once

Release `acme_normal.wsp13` F3 stage 1 `sha256:a9986fb93275fe264d76ee1b632f59898e51052ba4bfdb89980bf51a0a3b6177`, beside `acme_normal.wsp12` F2 and the two incidence baselines. Scorer: `tools/wsp/ext/sflame_cases.py validation-release` (the `validation-f2` chain with a release argument; F3 receives F2's prefixes plus the script's burner limit of each test, an input). The nitrogen tests of the validation days with an oxygen fraction (16 tests, 2 days), cutoffs 1, 3, 5 s; integrated Brier 6 to 120 s. F2 was first re-scored on the same cases and reproduced 13.1 (0.2619 / 0.3698 on 46 cases) before any number of F3 was read. Aggregates only; output `runs/dev/data_all/work/transfer_sflame/validation_f3_s1.json`.

## Without the planned stop

Answered: F3 46 cases of 16 tests (methane 13, ethylene 3); F2 46; scored on the 46 common cases. F3 refused 1 case(s) of 1 test (methane, flame temperature below its 1900 K bound; F2 refuses it too).

| Group | cases | F3 sum (case / day means) | F2 | CFI-G train incidence | s-Flame train incidence | F3 ext / term |
|---|---|---|---|---|---|---|
| all | 46 | 0.2048 / 0.1425 | 0.2619 / 0.1783 | 0.2793 / 0.2579 | 0.3218 / 0.3100 | 0.1254 / 0.0795 |
| fuel=ethylene | 9 | 0.0400 / 0.0400 | 0.0409 / 0.0409 | 0.2229 / 0.2229 | 0.2907 / 0.2907 | 0.0400 / 0.0000 |
| fuel=methane | 37 | 0.2449 / 0.2449 | 0.3157 / 0.3157 | 0.2930 / 0.2930 | 0.3293 / 0.3293 | 0.1461 / 0.0988 |
| reaches 250 C before the planned stop (F2 heating law) | 19 | 0.4452 / 0.4452 | 0.5386 / 0.5386 | 0.3073 / 0.3073 | 0.3407 / 0.3407 | 0.2528 / 0.1924 |
| does not reach 250 C before the planned stop | 27 | 0.0357 / 0.0368 | 0.0672 / 0.0606 | 0.2596 / 0.2504 | 0.3084 / 0.3040 | 0.0357 / 0.0000 |

## With the planned stop

Answered: F3 46 cases of 16 tests (methane 13, ethylene 3); F2 46; scored on the 46 common cases. F3 refused 1 case(s) of 1 test (methane, flame temperature below its 1900 K bound; F2 refuses it too).

| Group | cases | F3 sum (case / day means) | F2 | CFI-G train incidence | s-Flame train incidence | F3 ext / term |
|---|---|---|---|---|---|---|
| all | 46 | 0.2051 / 0.1427 | 0.3698 / 0.2454 | 0.2793 / 0.2579 | 0.3218 / 0.3100 | 0.1252 / 0.0799 |
| fuel=ethylene | 9 | 0.0401 / 0.0401 | 0.0410 / 0.0410 | 0.2229 / 0.2229 | 0.2907 / 0.2907 | 0.0401 / 0.0000 |
| fuel=methane | 37 | 0.2452 / 0.2452 | 0.4498 / 0.4498 | 0.2930 / 0.2930 | 0.3293 / 0.3293 | 0.1459 / 0.0994 |
| reaches 250 C before the planned stop (F2 heating law) | 19 | 0.4458 / 0.4458 | 0.7981 / 0.7981 | 0.3073 / 0.3073 | 0.3407 / 0.3407 | 0.2524 / 0.1934 |
| does not reach 250 C before the planned stop | 27 | 0.0357 / 0.0368 | 0.0684 / 0.0615 | 0.2596 / 0.2504 | 0.3084 / 0.3040 | 0.0356 / 0.0001 |

## Reading by 13.5

F3's sum is lower than F2's on both paths by the case mean (0.2048 against 0.2619; 0.2051 against 0.3698): **F3 stage 1 replaces F2 on s-Flame**. It is also lower than the CFI-G train incidence on both paths (0.2793): **it carries over to s-Flame**. Whether it replaces F2 as the normal-flame model is decided by (b).

What the parts say: ethylene is level with F2 (0.040 against 0.041); methane falls from 0.316 / 0.450 to 0.245 / 0.245. On the 7 tests that F2's heating law brings to 250 C before the planned stop, F3 scores 0.445 against F2's 0.539 / 0.798, still above both incidence baselines (0.307, 0.341); on the other 9 tests 0.036 against 0.067. The planned stop no longer costs anything (0.2048 / 0.2051). Two validation days; no interval is quoted as stable. These days' aggregates of 13.1 were known to the dossier's author.

## 13.5 (b), Flame Design validation days (normal flames), run once

`tools/wsp/ext/transfer_f2.py score-f3` (the 12.10 chain; F2's forwards are those of 12.10; F3 reads the same
prefixes plus the script's burner limit where the script file gives one: 500 C on 36 cases, none on 50).
Output `runs/dev/data_all/work/transfer_acme/fd_f2/report_f3.json`. 86 cases, all answered by both, case means:

| Path | release | extinction | termination | sum |
|---|---|---|---|---|
| without the planned stop | F2 | 0.0317 | 0.3933 | 0.4250 |
| | F3 stage 1 | 0.0299 | 0.3737 | 0.4035 |
| | incidence baseline | 0.2703 | 0.5340 | 0.8043 |
| with the planned stop | F2 | 0.0068 | 0.0724 | 0.0792 |
| | F3 stage 1 | 0.0066 | 0.0732 | 0.0798 |

F3 is not higher than F2 by more than 0.005 on either score on either path (the largest rise is +0.0008, termination
with the stop). **CFI-G validation (the other half of (b)) has not been run**: the reading of 13.5 on replacing F2
as the normal-flame model is open until it is.

## 13.5 (b), CFI-G validation, run once

The sealed chain of 12.10: `ignis infer forecast-cohort --seal` of F3 on the validation partition (cutoffs 1, 3,
5 s, the 14 event horizons), on the shared records and on `runs/dev/data_ext2` (the planned stop), 84 cases of 28
runs each, no refusal; F2's seals of 12.10; `ignis eval compare --intersect`; the two halves split by
`tools/wsp/analysis/cmp_split.py` (its check against the evaluator: 2.8e-17). CFI-G carries no burner-limit input,
so F3's members keep the limits of the era of the test. Integrated Brier (case means):

| Path | F2 extinction | F3 extinction | F2 termination | F3 termination | F3 - F2, cluster bootstrap (ext; term) |
|---|---|---|---|---|---|
| shared records (no planned stop) | 0.1537 | 0.1536 | 0.0961 | 0.0956 | -0.0001 [-0.0002, -0.0000]; -0.0004 [-0.0006, -0.0002] |
| planned stop (`data_ext2`) | 0.1473 | 0.1472 | 0.0798 | 0.0795 | -0.0001 [-0.0002, +0.0000]; -0.0003 [-0.0004, -0.0002] |

F2's numbers equal those of 12.10 (0.1473 + 0.0798 = 0.2271 with the planned stop).

## Reading of 13.5

(a) F3 stage 1 replaces F2 on s-Flame and carries over to s-Flame. (b) On CFI-G validation and on the Flame Design
validation days neither its extinction nor its termination score is higher than F2's by more than 0.005 (the largest
rise is +0.0008, Flame Design termination with the planned stop; on CFI-G both are lower). **F3 stage 1 replaces F2
as the normal-flame model.**

## Stage 2 (helium, zero-shot Part A), registered in 13.5: release and its one look

**Release** `acme_normal.wsp13` F3 stage 2 `sha256:fb576270f2227e43cf29506d972097465540f61aa61e998bf9dd0190691008d7`, a
revision of stage 1: the helium lines of Part A in the diluent table of every member, verbatim
(`configs/models/acme/fuel_tables/normal-f3s2-diluent.wsp`); every stage-1 line kept; hydrogen and blends refused.
Inputs, from condition fields only (`tools/wsp/ext/sflame_cases.py`, `f3_prefix`): `diluent_species` from the
burner-stream gas codes and the ambient of the experiment table (a text that is not a row when they differ),
`burner_limit_c` from the script, and for the days that name two oxygen fractions the two candidates
(`x_o2_role`, `tad`, `zst` the lower, `_b` the higher; weight 0.5 each, a refusal at either refuses).

**Transcription check** (forward beside the dossier, inputs only, cutoff 1 s; nothing changed):

| Dossier | Forward | |
|---|---|---|
| A6: methane in helium at 21 %, effective temperature 1904 K, 4 K above the bound | T_1 = 1904 K on every 20160 test | agrees |
| A6: the two tests at 5.13 mg/s refused either way | refused (c_logm outside the domain) | agrees |
| A7: ethylene in helium, 7 tests answered, 4.6 mg/s refused | 7 answered, 20164C1 (4.62 mg/s) refused | agrees |
| A7: reduced flow 0.55 to 1.1, no growth clock inside 45 s | phi 0.55 to 1.10; growth median none or 890 s | agrees |
| A7: burner reaches 250 C at 7 to 14 s (Part A heating law) | the 260 C limit at 11.7 to 24.8 s | differs: Part A's figure predates Part B's temperature factor, exponent 0.45 and limit 260 C |
| B7: ethylene in helium, limit at 12 to 16 s below 2.2 mg/s, 25 s at 3.1 mg/s | 11.7 to 16.5 s below 2.2 mg/s, 24.8 s at 3.08 mg/s | agrees (11.7 against 12) |
| A7: methane in helium, limit at 6 to 14 s (1.0 to 2.4 mg/s), about 19 s at 3.4 mg/s | 10.7 to 26.9 s, 46.6 s at 3.42 mg/s | differs (Part A's law, as above) |
| B7: methane in helium flashes, 0.99 at 21 % | weak-class probability 0.99 (growth members), 1.00 (hinge members) at 21 % | agrees |
| B7: survivors reach the limit at 9 to 35 s | 10.7 to 46.6 s | differs at 3.42 mg/s (46.6 s) |
| A6: 2050 K at 30 % | not read out (the diagnostics give the first candidate's members only) | not checked |

**The look, run once** (`sflame_cases.py validation-helium`, output `validation_f3_s2_helium.json`, aggregates only).
17 helium-diluted constant-flow validation tests (ethylene 8 with an oxygen fraction, methane 9 with two
candidates). Answered 35 cases of 14 tests (ethylene 7, methane 7); refused 7 cases of 3 tests (the three flows
above the declared range). Sum of the integrated Brier scores, case mean / mean of day means:

| Cases | F3 stage 2 | CFI-G train incidence | s-Flame train incidence |
|---|---|---|---|
| all, without the planned stop (35) | 0.9990 / 0.8708 | 0.3299 / 0.3213 | 0.3244 / 0.3153 |
| all, with the planned stop (35) | 0.9992 / 0.8708 | 0.3299 / 0.3213 | 0.3244 / 0.3153 |
| ethylene in helium, one oxygen fraction (21) | 1.5120 (ext 0.7986) | 0.3639 | 0.3610 |
| methane in helium, the 21/30 % mixture (14) | 0.2295 (ext 0.1231) | 0.2788 | 0.2696 |

**Reading by 13.5:** the sum is higher than the CFI-G train incidence on the cases it answers: **stage 2 does not
answer helium**. The loss is ethylene in helium: the release forecasts flames that last until the burner limit,
and the extinction score of 0.80 says they ended otherwise; the methane mixture is below both baselines.

**Train days (study data).** No helium-diluted train test is answered by this release: the 35 helium train tests
are hydrogen (19) or hydrogen/methane blends (16), refused in this stage.
