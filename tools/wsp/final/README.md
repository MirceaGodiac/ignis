# The final run (PROTOCOL 12.5)

`run_final.py` runs the one final score of this generation: the project's sealed test partitions and the
test days of the other investigations, on frozen releases, aggregates only. It was prepared and rehearsed
on 15 November 2026 without touching any test data. Nothing here has been run in final mode.

## Modes

| | Rehearsal (default) | Final |
|---|---|---|
| Call | `run_final.py rehearse <step>\|all` | `run_final.py final <step> --releases <frozen list> --execute-final --i-am-the-final-run` |
| Data | validation partition, validation days, BASS-II train tracks | test partition, test days |
| Seals and deposits | none (`forecast-cohort` runs as a plan, without `--execute` or `--seal`) | `deposit-cohort --partition test`, then `forecast-cohort --final-test --execute --seal` |
| Writes | `runs/final/20261115/rehearsal/<step>.json` | `runs/final/20261115/<step>.json`, `docs/wsp/RESULTS_FINAL.md` |
| Once only | no | marker `runs/final/20261115/markers/<step>.started`, created exclusively before any read |

- **Release list.** Final mode prints the sha256 of the list file and writes it into every result.
- **Per-case rows.** They go to `runs/final/20261115/local/` only and are never printed: comparison files,
  forecast listings and case files.
- **Marker.** The marker is never removed by the script. A step that fails part-way stays failed until the
  lead decides what to do.
- **Preflight.** `run_final.py preflight [--releases LIST]` is safe at any time. It checks the binaries and
  their sha256, that every listed release is in the registry, that no listed release has a test seal, and
  which markers exist. It also checks whether test-day raw files are on disk, by file name and existence
  only.

The proposed list is `releases_proposed.json` (sha256 `de2b688d…6ef6`; PSI-26 is left out, see step 4). The lead
freezes it, or a corrected copy of it, at a commit and names that file. Its format is
`families.<name>.release`, `.input` (`shared`, or `["shared", "data_ext2"]` for the planned stop of
`runs/dev/data_ext2`), and `.baselines` (label → release id, or `{"id", "input"}` for a comparator scored on
only some of the family's inputs).

Step-specific keys:
- `fabric.psi26_partition_map`, with `psi26_map_from_frozen_split: true`. These are absent from the proposed
  list, because PSI-26 has no conditions yet. They are added only if `DONE_PSI-26_CONDITIONS.txt` exists at
  the go. When present, final mode rewrites the map with `tools/wsp/atlas/freeze_psi26_days.py --map` (file
  names only) before the marker.
- `flex2.fuels` (per fuel: candidate, root) and `flex2.test_series`.

**Replaced models.** The burner-flame comparators include the models F2 replaced (`acme_*.wsp5` P1), on the
shared records only. `runs/dev/data_ext` carries the old planned stop for train and validation runs only:
`acme_script.py` touches no test run. So the P1 with-stop path cannot be scored on test.

## Steps, rehearsal, and what final mode does

Binaries: `target-merged/release/ignis.exe` (`b03a35c8…`) and `wsp-dev.exe` (`bd6055e0…`, rebuilt 15 November with the final mode of `front-feedback`).

### 1. `sealed`: FLEX droplets and ACME burner flames through the evaluator

**Rehearsal (validation).** It reproduces every quoted number exactly:

| Release | Expected | Rehearsed |
|---|---|---|
| `flex_methanol.wsp2` D3 | 40.0 µm | 40.03 µm (56 cases) |
| `flex_heptane.wsp2` D2 | 46.1 µm | 46.08 µm (39 cases) |
| `acme_normal.wsp12` F2, without the stop | 0.1537 / 0.0961 | 0.15367 / 0.09611 |
| `acme_normal.wsp12` F2, with the stop | 0.1473 / 0.0798 | 0.14729 / 0.07977 |
| `acme_inverse.wsp12` F2, without the stop | 0.1288 / 0.0880 | 0.12880 / 0.08804 |
| `acme_inverse.wsp12` F2, with the stop | 0.1349 / 0.0480 | 0.13494 / 0.04796 |

- Each number comes from the release set of the registered comparison, read back from the file that look
  wrote.
- The split of a doubled comparison (a release sealed on both data roots) by input reproduces the evaluator
  on the comparison of 14 November, to within 6e-17.
- **No seal was created.** Every rehearsed release already had validation seals. The rehearsal counts seal
  records before it compares, and it never passes `--execute` or `--seal`.
- Each comparison of the evaluator appends its own validation lines to the reveal log, as every
  development look does.

**Final, per family.** `C`/`S` are the cohort and split of the release (all comparators share them). `<s>` is
`{"horizon_s":10}` for droplets and `{"times_s":[6,…,120]}` for burner flames.

```
# 1. continuations of the test runs into the shared vault (holdout/), from the shared records
ignis eval deposit-cohort --cohort C --split S --partition test
# 2. every seal of every family and input BEFORE any comparison (the vault refuses a test seal after a reveal)
[IGNIS_DATA=runs/dev/data_ext2] ignis infer forecast-cohort --release X --split S --partition test --final-test \
    --cutoffs 1,3,5 --settings '<s>' --execute --seal          # X = the release and each comparator, per input
# 3. one comparison per comparator, the comparator as reference
ignis eval compare B R --partition test --final-test --allow-feature-mismatch --allow-unit-mismatch \
    --allow-cohort-mismatch --intersect --out runs/final/20261115/local/final/sealed/<family>/cmp_<B>.json \
    [--horizons 6,8,10,12,15,20,25,30,40,50,60,80,100,120]
```

**The deposit.** Test continuations are deposited only from the shared records, into the default vault.
This is how the validation forecasts on `data_ext2` were scored. A second deposit from `data_ext2` would be
refused by write-once wherever a record differs.

**Reported per release and input.** Cases, runs, sessions and scored cases. Refusals, `not_at_risk` and
`refusal_counts` come from the forecast listing, because a refused forecast is never sealed and the
comparison does not see it.

- **Droplets:** CRPS and MAE of the median in µm, and coverage of the 50 % and 90 % bands. These are the
  evaluator's cluster-equal estimates.
- **Burner flames:** integrated Brier score for extinction and for operator termination, and their sum, as
  case means.
- **Paired differences** to each comparator use the evaluator's cluster bootstrap. Where a comparison holds
  both inputs, the numbers are recomputed per input with the functions of `tools/wsp/analysis/cmp_split.py`.

**What the evaluator requires.**
- `--final-test` on `forecast-cohort` and on `compare`. Without it, both refuse the test partition.
- Deposited test continuations.
- A seal per artifact made before any reveal of its run.
- No active lab session.

Every reveal (granted or refused) is logged by the evaluator itself in `runs/evaluation/reveal_log.jsonl`
(purpose `compare`, `final_test: true`). The script adds no line for this step. It counts those lines
afterwards (`evaluator_reveal_lines_final`).

### 2. `flame-design`: Flame Design test days (normal 19123, 19200, 19206; inverse 21361)

This is the chain of `tools/wsp/ext/transfer_f2.py`, imported unchanged: F2 with ethylene as itself and the
new planned stop, F1, ethylene as ethane, and the incidence baseline of 11.1. Each runs without and with the
planned stop. Forwards and prefixes go to `runs/final/…/local/`.

**Rehearsal (validation days).** The decision-input table of `docs/wsp/feedback/flame_design_f2_result.md`
is reproduced exactly. All four blocks match `fd_f2/report.json` with a largest difference of 0.0 (86 and
75 cases).

**Final.**
1. Copy `conditions.csv` byte for byte.
2. Fetch any missing raw text files of included tests (`transfer_acme.cmd_fd_fetch`). Preflight found none
   missing for the 49 test-day tests.
3. Measure the endpoints of the test-day tests with `flame_design.measure` (thresholds unchanged) into this
   run's own `endpoints.csv`.
4. Build and score with `transfer_f2.build` and `transfer_f2.summarise`.
5. Append one reveal line per scored forecast artifact: purpose `final:flame_design`, `split_id` = sha256 of
   `docs/wsp/atlas/splits/PSI-10.json`.

**Caveat.** The measurement and the score run in one process. No author sees the endpoints, but they are not
"measured by an agent that sees no forecast" in a separate step.

### 3. `cfi`: CFI n-dodecane test days

This is the chain of `transfer_droplet_arms.py` with the inputs of `cfi_cases.py`: candidate
`flex_dodecane.wsp7` W0, its root, persistence and the classical D² law.

**Selection.** The registered rule: n-dodecane, free droplet, a processed series, inside the dodecane
release's domain. On train and validation it gives exactly the seven tests of `cfi_cases.TESTS`; this is
asserted. On test days it gives one test, whose series is on disk and unopened.

**Rehearsal (validation day).** The line of `cfi_transfer_result.md` is reproduced exactly:
- 6 cases.
- Candidate 130.1 / 155.3 / 0.33 / 0.54.
- Root 212.0 / 256.6 / 0.24 / 0.38.
- Persistence 313.6, D² 1315.6.
- Difference −81.9.
- Largest difference to `aggregate.json`: 0.0.

**Final.** The same on the test day. One reveal line per artifact (purpose `final:cfi`, split = sha256 of
`PSI-39.json`). One test (at most 3 cases) gives no interval.

### 4. `fabric`: BASS-II test tracks (and PSI-26 test days)

This step runs `wsp-dev front-feedback --quiet-cases`, with `--out` holding aggregates and `--detail`
local.

**Rehearsal (train tracks).** Exact, for D1 and Q0:
- Tip CRPS 0.0761 for both.
- Ignition-outcome Brier 0.0033 and 0.1469.
- The carry of D1 gives 0.2500.

**The binary's modes** (`crates/ignis-wsp-dev/src/front.rs`, 15 November):
- It reads every `*.json` of the folder that carries `wsp-bass-front-track/1`.
- A track's partition comes from the file's field, where `held_validation` means validation and `held_test`
  means test. When `--partition-map FILE` is given (a JSON object: run id or file stem -> `train` |
  `validation` | `test`), the map's value wins. A track with no partition, or with any other value, makes
  the call refuse.
- Without `--final-test`, a folder holding a test track is refused. With `--only-partition train|validation`,
  only the tracks of that partition are read past their partition field, and the rest are skipped.
- `--final-test` scores test tracks only, and a mixed folder is refused. It never prints a per-case line.
  Before `--out` is written, it appends one line per scored burn to `runs/evaluation/reveal_log.jsonl`
  in the evaluator's format: purpose `final:bass_front`; `artifact_id` is the content key of the aggregate
  report; `split_id` is `--split-id`, or else the sha256 of the map.

**Final.** The step refuses before the marker if the binary has no `--final-test`.
- **BASS-II.** It scores only the held test tracks whose measurer verdict is `quality == "ok"`.
  - `front-feedback` does not read that field. A `failed` track, such as a rod, a film or a 1 cm strip,
    has no trusted outcome.
  - The step copies the `ok` files unchanged from `runs/dev/media/fabric/tracks/psi25/held_test/` into
    `local/final/fabric/bass2_test_quality_ok/`. It decides from that one top-level field.
  - It records how many held tracks it left out, and their `quality` values.
  - It scores that folder with `--final-test --split-id` set to the project split `bass-4083156d5b18dab4`.
- **PSI-26.** Left out of the list for now, so its test days stay unread. Its tracks carry no conditions yet
  (oxygen, flow, material and width are null), so no model can run on them.
  - Add `families.fabric.psi26_partition_map` only if `runs/dev/media/fabric/DONE_PSI-26_CONDITIONS.txt`
    exists when the lead gives the go. Also add `psi26_map_from_frozen_split: true`, which rewrites the map
    from `docs/wsp/atlas/splits/PSI-26-days.json` by file names before the marker.
  - The step skips PSI-26, and says so, whenever that marker is missing, even if the key is present.
  - When it does run, it copies the test-day tracks unchanged into `local/final/fabric/psi26_test_tracks/`
    and scores them with `--final-test --partition-map`.
- **Recorded.** The result records the quality filter's counts, the map's sha256 (when PSI-26 is scored),
  and the count of final reveal lines by purpose.
- **Blocker found in rehearsal: the tracks are in the wrong unit.** The measurer's tracks are written with
  `position_unit` `fw`, a fraction of the frame width. `front_tools` accepts only `sample_width`.
  - The quality filter was rehearsed on the measurer's own train folder `psi25/train/`: 8 tracks, 5 `ok`
    and 3 `failed` left out.
  - `front-feedback` then scored **0 cases and 0 ignition tracks** on the 5 `ok` tracks, all with reason
    `position_unit_not_sample_width`.
  - The held test tracks come from the same measurer, so the final BASS-II tables would be empty.
  - A conversion of the measurer's tracks into the `wsp-bass-front-track/1` sample-width form (stills,
    conditions and fan as in `tools/wsp/bass_optical/tracks/train/`), or a reader in `front_tools`, is
    needed first. Neither exists in this lane.

**PSI-26 validation and train days (12.9, development).** These use the same map:
`front-feedback --tracks runs/dev/media/fabric/tracks/psi26 --partition-map MAP --only-partition validation`
(or `train`). Test-day tracks are skipped unparsed.

### 5. `flex2`: FLEX-2 (PSI-68) test days, arm F

**What it scores.** PROTOCOL 12.7, second and third amendments: the eleven test-day droplets of
`docs/wsp/atlas/splits/PSI-68-batch-test.json` (n-octane 6, n-decane 5), measured blind into
`runs/dev/media/droplet/flex2_series_test/`.
- **Chain.** `transfer_droplet_arms.py`, imported unchanged. For each fuel, the arm-F candidate
  (`flex_<fuel>.wsp7` W0) is scored against its root (`baseline-d2`, "treat the fuel as heptane"),
  persistence and the classical D² law.
- **Series.** Only series the measurer's own rule accepts (`quality = usable`) are scored.
- **Output.** Aggregates, plus paired differences with the 10,000-resample day bootstrap. Per-case files are
  local.

**Final.**
- The step refuses unless `runs/dev/media/droplet/SCALE_PINNED.txt` exists. The third amendment applies the
  pinned scale rule to the held series too.
- It reads only the files named by the batch, under `flex2.test_series` of the list. Confirm that path
  against what the marker says about where the rescaled held series are.
- A series of any partition other than `test` stops the step.
- It appends one reveal line per scored test: purpose `final:flex2`, `artifact_id` = sha256 of the series
  file, `split_id` = sha256 of `PSI-68-days.json`.

**Rehearsal.** It runs on `flex2_series_v2/validation/` when the scale marker and that folder exist.
Otherwise it runs on the first, unpinned `flex2_series/validation/`, as plumbing only. On the unpinned
series it reproduces the validation-day line of n-decane in `docs/wsp/feedback/flex2_transfer_result.md`
exactly:
- 6 cases, 2 tests, 2 days.
- Candidate 113.3, root 56.1, persistence 175.5, D² 84.7 (µm).
- Candidate minus root +57.2.
- Largest difference to `flex2_F_decane/aggregate.json`: 0.0.

No validation-day n-octane series is accepted.

## Known ways the final run could fail or leak

**Step 1.**
- **Two half-sets.** The comparison against the spec's reference baseline holds both inputs, so its tables
  are the split by input. The split was checked exactly on 14 November's doubled comparison.
- **Intersection with comparators.** Pairing with the fitted inverse baselines keeps 49 of F2's 54
  validation cases, and pairing with the FLEX fitted baselines keeps 56 of D3's 59. Each table states its
  own case count. Quote F2 or D3 from the comparison against the reference baseline of the spec. The
  README's 40.0 µm is on the 56 cases of the 12-release group; on the 59 cases against the D² law alone, D3
  is 39.5.
- **The planned stop on test runs.** `runs/dev/data_ext2` was built in every partition. A test run with no
  script file has no planned stop. Its `data_ext2` forecast then equals the shared one and is counted under
  both inputs. Compare the forecast counts per input in the result.
- **Concurrent writers.** Other agents write the reveal log at the same time. One malformed line, two
  objects run together, already exists. The script appends whole lines and does not read the log back,
  except to count.

**Steps 2 and 3.** These read test-day outcomes outside the vault. The protection is the marker, the reveal
lines, and the rule that per-case files stay under `runs/final/…/local/`.

**Printing.** No command whose stdout carries per-case rows is printed. `forecast-cohort` lists cases on
stdout; the script captures that to a local file. `compare --out` prints only `{"written": …}`.
