# Endpoint audit: CFI-G TRAIN, 14 November 2026

This is a non-modelling audit of all **154 frozen CFI-G TRAIN identities** (81 normal,
73 inverse). It does not qualify a new endpoint method or change a model, table record,
release, original measurement, or score. No Flame Design outcome file or validation/test
payload was opened. The previously disclosed 18/176 Flame Design disagreement aggregate
motivates a future investigation; this audit cannot attribute those 18 cases.

The original 154-row check was admitted only after both split hashes, its producer code,
its entire identity set and its file hash were verified. Its nine >5-second tails are
preserved. Exact raw routes were checked for every identity: **152 old measurements replayed
within their three-decimal receipt precision**, one source is absent, and one internal
identity conflict prevents an authorized replay. All 154 retain their original receipt
and table reference. Seven of the 152 replayable files have non-increasing sample clocks;
the new diagnostics refuse these. Therefore the sample-supported diagnostic subset is
**145**, not 153 or 154. Counts must retain those denominators.

## Artifacts and reproduction

Implementation: `tools/wsp/astra/endpoint_train_audit.py`; tests:
`tools/wsp/astra/test_endpoint_train_audit.py`. Generated private receipts:
`runs/dev/data_astra/work/endpoint_train_audit/`:

- `protocol_frozen.json`: exact code hash, split/producer hashes and all 154 identities;
  written **before** running the new sample-confirmation/bracket diagnostic.
- `summary.json`: coverage, old replay, diagnostic flags and historical timing tails.
- `train/<family>_<test>.json`: original receipt, table endpoint with original source row,
  raw file paths and SHA256, TP stamps, script parameters, unchanged-method replay and
  the separate diagnostic. All files in this directory are TRAIN-only.

Commands actually run:

```powershell
python -m unittest discover -s tools/wsp/astra -p test_endpoint_train_audit.py -v
python tools/wsp/astra/endpoint_train_audit.py --freeze
python tools/wsp/astra/endpoint_train_audit.py --execute
```

Final test run: **11 passed**. The first test run exposed an overly narrow identity syntax
that rejected the already-frozen `21196M2_1`; accepting the literal pinned suffix fixed
that parser before freeze. Freeze succeeded without raw reads. Execution completed with
154 receipts; no network or model execution occurred. No Rust/frontend checks were run:
their implementation was not changed.

Frozen split SHA256:

- normal `eab7dd3d514d70c3a1a8fd2ea6c65d3625e43f5abce8f77e601551af4f045f02`;
- inverse `f849dc580170ede7b2816bf23509a8dac0f94c2b335278437cba41444acb19d1`.

Old check SHA256: `63c69b4bfdfeddc57afda7431b519587ed8860d5692c6035dab08fce9ac42e88`.
Frozen audit receipt SHA256: `d2b7c5bf3124e7628de57d8d74dc87dda52f796e088f21e28449caf33a94c133`.
The old producer is `tools/wsp/ext/transfer_acme.py::cmd_cfig_endpoints` (TRAIN loop);
measurement is `tools/wsp/ext/flame_design.py::measure`. Both are pinned in the audit,
imported without invoking their CLI, and unchanged.

## What the nine timing tails actually compare

Times below are seconds since the unchanged radiometer onset; the difference is the old
sensor endpoint minus the original table midpoint. Table rows are the original NASA
normal/inverse experimental-table row locators retained in the per-test receipts.

| Test | Difference | Concrete source evidence | Classification and remaining uncertainty |
|---|---:|---|---|
| 21270A1 | +5.334 | Inverse row132 says 60; script `constantFlowTime1Sec=60`, `stabilizationTime2Sec=5`; flow command 65.334, PMT 66.040 | Reference-duration/clock-origin mismatch. Table equals the constant stage; sensor follows the full 65s schedule. No globally justified five-second correction. |
| 21270A2 | +5.364 | Inverse row133 says 90; stage 90+5; command 95.364, PMT 96.320 | Same concrete stage-duration mismatch. |
| 21270A4 | +5.297 | Inverse row135 says 90; stage 90+5; command 95.297, PMT 95.871 | Same concrete stage-duration mismatch. |
| 21270A3 | −5.749 | Inverse row134 says 35; TP explicitly aborts for burner-temperature limit; command 29.251, PMT 29.616; recorded data end 31.249 | Actual intervention is documented. Table duration does not agree with active-phase sensor time; its intended origin/end definition is unresolved. The scheduled 95s stop is not the realized intervention. |
| 21270H2 | +12.237 | Inverse row138 says 43; command 55.237, PMT 55.656; TP has no `MFCs off` or extinction stamp | Reference-versus-sensor duration conflict with incomplete log. No source supports an automatic replacement of 43. |
| 21270H3 | +50.481 | Inverse row139 says 43/FT; PMT 93.481, steep radiometer fall 93.147, command 95.277; TP has no extinction stamp | Reference time conflict and physical-cause disagreement. The time ordering is observable, but terminal cause still needs independent adjudication. |
| 21153C4 | −34.208 | Normal row136 says 276/FT; PMT threshold 0.05 gives 241.792, existing 0.02 cross-check gives 251.585; flow 275.233; radiometer drop 275.938; PMT rises again | Instrument/threshold sensitivity plus nonterminal dark episode. A confirmed first dark episode is not demonstrated terminal extinction. Do not tune the threshold until it reaches 276. |
| 21291A1 | −126.328 | Normal row169 says 167/FT; first PMT dark 40.672, later signal rises; TP extinction 162.914 then flow 166.006 after its 3s programmed delay | Recurrent signal and endpoint-definition conflict. The original binary label agrees with the script while timing is wrong for the script's later extinction. No argument for choosing 40.672 as final burn end. |
| 21291C1 | −156.912 | Normal row170 says 246/FT and explicitly records five hot-to-cool transitions, six hot and five cool flames; first PMT dark 89.088, later signal rises; flow 245.351 | Proven estimand mismatch: first dark/hot-to-cool transition versus terminal burn. The sequence must be represented; neither a first-dark rule nor a table-wins rule resolves the physical endpoint. |

None of the nine is explained by adding a universal onset offset. The three approximately
five-second discrepancies have a concrete five-second stage in their own scripts;
other tails involve transitions, missing logs, interventions or conflicting references.
In particular, table `resolution=resolved` means the deterministic table parser resolved
its columns. Its derivation explicitly says the time origin was **not adjudicated against
sensor data/video**; it is not an independent physical gold label.

## Source-proven logical defects and the separate adapter

1. **Abort is a completion state, not a competing physical cause.** The old TP parser gives
   `reason or Test Point Failure` precedence over `Extinction detected`; the subsequent
   agreement check treats every abort as non-extinction. TRAIN `21106F3` has
   `Reason: EXTINCTION DETECTED`, both failure and extinction stamps, but is counted as
   disagreement against sensor extinction. `stamp_account` preserves abort, reason,
   extinction stamp and MFC stamp separately and leaves physical cause unresolved. This
   proves a bookkeeping defect; it does not prove how many of the 18 Flame Design cases
   are affected. In this same train test, the constant stage was never reached and the
   flow reference was zero, so its causal classification needs separate review too.

2. **First sustained does not mean sustained through unobserved time.** The old
   `first_sustained` uses the next false timestamp, accepts a short true suffix at EOF,
   and can cross a recording gap. `observed_sustained` requires the full observed 0.5s hold,
   rejects non-increasing time, splits gaps above two nominal sampling intervals and
   returns a preceding-sample/first-true bracket plus confirmation time. Two nominal
   intervals is an acquisition-continuity guard, not an intensity threshold fitted to
   labels. All intensity, smoothing and hold constants remain the old constants.
   This is a sample-supported threshold episode, not proof of continuous physical
   extinction. `21153C4` changes first confirmed threshold transition by 1.201s, yet
   remains scientifically unresolved. The code does not claim the nine tails are fixed.

3. **An event bracket is not an exact time.** The old transfer adapter emits
   `last_confirmed_active_s == first_terminal_s == end_s` for measured sensor ends. The
   new diagnostics retain sample brackets and temporal support from the old centred
  0.2s PMT and 0.1s radiometer smoothers, sample quantization, and millisecond setpoint
   clock. These are processing-support intervals, **not calibrated confidence bounds**:
   unknown sensor response, gain/noise and onset bias can be larger. They are deliberately
   not installed as qualified physical endpoint records.

4. **Near-coincident darkening and command changes do not identify cause.** The old rule
   forces operator termination unless PMT darkening is at least 0.5s earlier. The new
   interval-order helper preserves that existing extinction margin, requires ordered
   brackets for a definite temporal order and otherwise returns unresolved. One of 145
   diagnostic tests (`21153C5`) has unresolved ordering. The legacy-compatible helper
   string `extinction_precedes_flow` only names ordering of the candidate darkening;
   `physical_extinction_qualified=false` remains mandatory. Recurrent darkening, a
   table conflict or insufficient instrument calibration still prevents a cause claim.

5. **At-risk must use recorded interval semantics.** `at_risk_interval` distinguishes
   `at_risk`, `not_at_risk`, and `unknown`: upper endpoint at/before cutoff is over;
   confirmed-active lower endpoint at/after cutoff is at risk; a straddling interval is
   unknown. Right censoring only establishes activity through its observed lower bound.
   No fabricated midpoint−0.5 is used. This also handles decimal table times correctly:
   `acme.rs` only applies ±0.5 to integer textual burn times. At 1s, an integer table
   burn-time 1 means [0.5,1.5], not proven dead and not confirmed active at 1s.

These helpers are a new **diagnostic/measurement adapter**, not an altered model or a
replacement production scorer. Tests cover unknown identity before outcome decoding,
nested/duplicate identities, EOF, gaps, invalid times, time scaling, event/interval/right
censoring, decimal timestamps, ambiguous cause order and abort semantics.

## All-cohort checks and limitations

| Cutoff(s) | Table confirmed at risk, all 154 | Table unknown, all 154 | Table known ended, all 154 | Old point-sensor at risk, original 153 comparable |
|---:|---:|---:|---:|---:|
| 1 | 142 | 12 | 0 | 153 |
| 3 | 139 | 3 | 12 | 143 |
| 5 | 133 | 5 | 16 | 133 |

The original 153 comparable table confirmed counts are also 142/139/133. Thus the
historical 1s difference cannot be presented as 11 corrected table errors: interval
uncertainty alone changes membership. `summary.json` intentionally reports its separate
**145-row** diagnostic subset: table 134/131/125 versus point-sensor 145/135/125.
It must not be substituted for the all-cohort table above.

Additional issues, found across the complete audit rather than selecting the nine tails:

- `21196M2_1`: TP names `21196M2`, as-run script names `21196M3`. Internal identity
  conflict is retained; no guessed rename/alias is permitted. `21130A9` lacks its exact
  raw data source. Neither is assigned a replacement outcome.
- Seven raw files have one non-increasing timestamp each, with **different** channel
  values across the affected pair: `21088N1`(−2ms), `21125K2`(0ms), `21291G2`(−6ms),
  `21196M2`(−5ms), `21209H1`(−3ms), `21256B1`(−5ms), `21256J1`(0ms). Sorting or deleting
  these samples without a clock-repair rule would hide ambiguity. No nonfinite channel
  values were found in these seven files.
- Two additional files contain sampling gaps above two nominal intervals: `21106F1`
  (four gaps, largest 48ms) and `21130M1`(one 18ms gap). The diagnostic never bridges them.
- In the145 eligible diagnostic rows, 38 have the existing PMT threshold-sensitivity
  flag. Four have at least one later sample above the original lit threshold after dark
  confirmation: `21153C4`, `21153G1`, `21291A1`, `21291C1`. A single later sample may be
  noise; no new relapse/relighting threshold is inferred. `21291C1` independently has
  explicit transition documentation. These are review flags, not 38 or 4 newly proven errors.
- Raw header test names agree with the authorized identity for all 152 replayed files.
  The original parser still silently skips malformed rows; this audit does not certify
  a lossless acquisition parser. The two strict timestamp and threshold checks cannot
  establish gain calibration, PMT visibility of cool flames or a final physical endpoint.

## Next concrete fixes, before any new accuracy claim

The measurement owner can adopt the tested abort/cause separation and interval-aware
at-risk accounting in a **new version**, retaining old receipts. A new lossless parser
should report every skipped row and qualify clock chronology, and an identity custodian
must adjudicate the `21196M2_1` alias using source metadata rather than outcome similarity.
The flow stage must remain unresolved when it never began; stage-duration semantics must
be explicit instead of correcting every table duration by the same offset.

The next endpoint protocol must distinguish hot-flame loss, re-ignition/transition,
terminal loss and commanded termination. Freeze that definition and its instrument/noise
rationale before independent blind labelers inspect additional outcomes. In particular,
sensor gains, the PMT dark-level estimator and whether a low-light interval is a cool
flame need independent video/engineering evidence. Missing timestamps and reference
conflicts in A3/H2/H3 need that adjudication, not a forced table/sensor winner.

Only after such a freeze should an authorized evaluator prepare an independently labelled
blind cohort and compare the old/new versions with abstentions and denominators reported.
No extra Flame Design validation look, new scoring, model revision or qualified-accuracy
claim is authorized or performed by this audit.
