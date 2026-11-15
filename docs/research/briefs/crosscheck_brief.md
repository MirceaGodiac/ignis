# Brief: cross-check NASA's tables against NASA's own sensor files (conditions only)

Repository `D:\dev\SpaceApps-Ardealu`, branch `feat/handwritten-science`. You write and run a Python
script; no language model reads data. Commit only your own files by explicit path, never `git add -A`,
never push.

## Goal

For the 19 NASA PSI flight combustion investigations, find every place where a NASA **condition**
in a table disagrees with NASA's own sensor or settings text files for the same test, and decide the
cause of each with the checks below. Output: a discrepancy list with both values, the method, the
check that decided the cause, and the verdict.

## Scope: conditions only

Compare, wherever both sides exist:
- tabulated flows (fuel, oxidiser, diluent mass or volume flow) against commanded setpoints and readbacks
  at steady state, with each unit converted explicitly;
- tabulated composition (fuel fraction, O2 fraction, diluent) against the bottle or mixture files and the
  commanded flow ratios;
- pressure and chamber atmosphere against the chamber readings or settings files before ignition;
- test ids and dates against file and folder names;
- each table against itself: repeated rows, fractions that do not sum to one, mixed units, impossible values.

Out of scope: any **outcome** (burn time, extinction, how a test ended, diameters, spread rates, any
measured flame property). Do not compute or record those, for any test.

## Hard rules (protected data)

- Never open: `holdout/`, `runs/dev/holdout_ext/`, `runs/dev/bass_front/val_sealed/`, the validation
  series of `runs/dev/data_ext/`, `runs/dev/data_all/work/atlas/tests_outcomes.jsonl`,
  `runs/dev/data_all/work/flame_design/`, `runs/dev/data_all/work/transfer_acme/`, anything under
  `runs/dev/data_astra/work/labels/`, `runs/dev/data_astra/work/discrepancies/`, `runs/dev/data_docs/cards/`,
  `runs/dev/data_docs/final/`, `runs/dev/data_docs/protect/`, `runs/dev/data_docs/gold/`, `runs/dev/data_docs/visual/`.
- Never read photomultiplier, radiometer, camera or flame-measurement channels. Read only flow, setpoint,
  readback, pressure, temperature-before-ignition and composition channels, and only up to ignition where
  the file marks it.
- Print aggregates and your own discrepancy records only; never dump sensor rows.

## Inputs

- Test catalogue (conditions and group ids, no outcomes): `runs/dev/data_all/work/atlas/tests.jsonl`,
  schema in `docs/wsp/atlas/SCHEMA.md`; per-investigation table defects in `docs/wsp/atlas/INVESTIGATIONS.md`;
  file naming schemes in `docs/wsp/atlas/FILES.md`.
- The NASA files on disk: `D:\Codex\Worktrees\ignis-science-transfer\SpaceApps-Ardealu\runs\dev\data_astra\logs\upload_verified.jsonl`
  maps each NASA `file_name` and investigation to `source_path` (hash-named files) and `sha256`.
- A working reader of ACME sensor and settings text files: `tools/wsp/ext/flame_design.py` (read it for the
  file formats; take only the condition-channel parts).
- Known cases to reproduce first, as the proof that the method is sound: `docs/wsp/dossier/02_data_classification.md`
  section 2.7 (CFI-G train tests: 21125K1-K3 fuel fraction 0.031 vs 0.30, 21106G6-G7 0.8 vs 0.5 mg/s, 21106G9,
  21196M2_1 oxygen 5.59 vs 2.79 mg/s, 21125D2 0.045 vs 0.037, the unsettled 21228C4, 21228K2, 21256C2), and the
  constant readback/table flow factor of 1.083 (5th-95th percentile 1.058-1.089) across CFI-G train. Skip the two
  burn-time rows of 2.7 (outcomes).

## Method for each discrepancy

One record per discrepancy: investigation, test id, group id, field, table value with file, column and unit,
sensor value with file, channel, method and unit, the size of the difference against the method's own error,
and a status (confirmed by a second, independent reading; seen once; unsettled). NASA's value is never replaced.

Then decide the cause, in this order, and record which check decided it:
1. Is it ours? Run the same method on the tests where table and sensor agree. A difference by one factor on every
   test is a calibration or a unit, not a table defect.
2. A third NASA source: setpoint, readback, the as-run file, the script, the operator comment. A file copied from the
   table counts with the table.
3. Physics that cannot be otherwise: bottle composition and diluent flow, component flows against their total,
   pressure rise against gas injected.
4. The shape of the difference: a factor of 2 or 10, a value equal to the row above, a value equal to the planned one.
5. Is it the same quantity (two definitions)?

Verdicts: `table_value_wrong`, `table_records_plan_not_execution`, `two_definitions`, `our_reading_wrong`
(fix the reading; the record goes), `unresolved`. Never use a test's outcome as the argument.

## Outputs

- `tools/wsp/docs/crosscheck.py` (re-runnable; no arguments needed).
- `runs/dev/data_docs/discrepancies/discrepancies.jsonl` (schema `ignis.docs.discrepancy/1`).
- `docs/wsp/measure/DISCREPANCIES.md`: method, how the known 2.7 cases were reproduced (or not), counts by
  investigation, field and verdict, the statement "of N discrepancies, the cause of M is established", and the
  limits (what could not be compared and why). Counts and short descriptions of conditions only.
- Commit the script and the markdown file with message prefix "Document lane:" and a line
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

Reply with: N, M, counts by verdict, whether each known 2.7 case was reproduced, and the commit id.
