# F-001: programmed-stop provenance and train recount

- **About:** C-21 / M-10 / D-27; `programmed_stop` from the ACME "as run" script file
- **Reviewer:** Codex audit agent, 14 November 2026
- **Verdict:** weakened
- **Severity:** integrity
- **Partitions touched while reviewing:** train only; no validation or test inputs opened

## What the dossier says

The dossier says the NASA "as run" file contains stage durations fixed before ignition, and that their sum predicts scripted flow termination at the end of the final constant-flow stage, with an offset of 0.4 s (`02_data_classification.md` §2.4, C-21; M-10 in `03_datasets_and_measurements.md`; D-27 in `04_decisions.md`).

## What I did

Inspected (code only) `tools/wsp/ext/acme_script.py` and `tools/wsp/analysis/lib.py`, plus the section 10 protocol and the aggregate train measurement report. Before opening the two train summary tables, I set this check: filter both tables to the frozen ACME train IDs; confirm whether all rows labelled early extinction/limit have a later programmed stop than observed burn time, and whether scripted-stop rows match the sum of the eight listed stage durations within the claimed tolerance. A material counterexample (an early stop at/after the programmed time, or a scripted stop outside the stated range) weakens the prospective-plan claim; multiple such counterexamples would refute it. No values from validation or test are admissible to this check.

## Evidence

The parser code reads only the eight script duration keys and sums them. The script's own prose says the source file is set before ignition, but that assertion is not itself independent provenance evidence. `lib.partition_ids(name, "train")` is explicitly train-only and asserts that only train or validation can be requested; the independent recount below forms the train-ID allowlist before opening either measurement table. The report states that it measured the 154 ACME train tests and gives the expected summary:

| Subset | n | Programmed stop minus observed burn time / actual stop minus stage sum |
|---|---:|---|
| early extinction or limit | 99 | min 10.2 s; median 79.4 s; max 265.3 s |
| scripted ends | 29 | min −1.3 s; median −0.4 s; max +1.7 s |

**Independent train-only recount:** exact match. The ID allowlist held 154 train tests; 154 timeline rows and 153 burner rows matched it. Of the 99 early extinction/limit rows with both values, every scripted target was later than burn time. The 29 scripted-end offsets match the published range and median. Command, run from the repository root:

```powershell
@'
import csv, sys
from pathlib import Path
sys.path.insert(0, 'tools/wsp/analysis')
import lib
train = {rid.split(':')[-1] for family in ('acme_normal', 'acme_inverse') for rid in lib.partition_ids(family, 'train')}
root = Path('runs/dev/research5/acme_raw')
with (root / 'tp_timelines.csv').open(encoding='utf-8', newline='') as f:
    timelines = [r for r in csv.DictReader(f) if r.get('test') in train]
with (root / 'tests_burner.csv').open(encoding='utf-8', newline='') as f:
    tests = {r['test']: r for r in csv.DictReader(f) if r.get('test') in train}
num = lambda x: float(x) if x not in (None, '') else None
stages = ['ar_stabilizationTime1Sec', 'ar_stabilizationTime2Sec', 'ar_ignitionToTestRampTimeSec', 'ar_constantFlowTime1Sec', 'ar_testFlowRamp1To2TimeSec', 'ar_constantFlowTime2Sec', 'ar_testFlowRamp2To3TimeSec', 'ar_constantFlowTime3Sec']
early = []
for r in timelines:
    if 'extinction' not in r.get('stop_class', '').lower() and 'limit' not in r.get('stop_class', '').lower(): continue
    b = tests.get(r.get('test'))
    if b is None: continue
    p, t = num(r.get('programmed_off_from_onset')), num(b.get('table_burn_time'))
    if p is not None and t is not None: early.append(p-t)
scripted = []
for r in timelines:
    if not r.get('stop_class', '').lower().startswith('scripted'): continue
    actual = num(r.get('actual_off_from_onset'))
    parts = [num(r.get(k)) for k in stages]
    if actual is not None and any(x is not None for x in parts): scripted.append(actual - sum(x or 0 for x in parts))
for name, xs in [('early', sorted(early)), ('scripted', sorted(scripted))]:
    print(name, 'n=', len(xs), 'min=', round(xs[0], 1), 'median=', round(xs[len(xs)//2], 1), 'max=', round(xs[-1], 1))
print('train_ids=', len(train), 'timeline_rows_used=', len(timelines), 'burner_rows_used=', len(tests))
'@ | python -
```

Output: `early n=99 min=10.2 median=79.4 max=265.3`; `scripted n=29 min=-1.3 median=-0.4 max=1.7`; `train_ids=154 timeline_rows_used=154 burner_rows_used=153`.

This confirms the recorded timing relationship on train. It does not authenticate file timestamps or establish that NASA did not rewrite a nominally "as run" script after an abort. The C-21 clause "known before ignition" therefore remains weakened pending source-format/provenance evidence from NASA or the data lane.

## What should change

Keep `programmed_stop` provisional as a script-plan proxy, not an established pre-ignition measurement. M-10's timing comparison can show that recorded early terminations usually precede the script target; it cannot alone establish when that file was authored or rule out a post-test rewrite. Ask the data/evaluation owners to accept the source's temporal semantics before treating D-27's stop knowledge as admissible. The shared-record score remains the supported fallback.

## Response

(The author answers here.)
