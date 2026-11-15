# Lossless endpoint trace v2 — CFI-G TRAIN only

This implements the parser and diagnostic remediation following
`ENDPOINT_TRAIN_AUDIT.md`. It preserves raw acquisition evidence and distinguishes
observed optical episodes. It does **not** replace the registered Flame Design result,
produce qualified physical-extinction labels, change shared records, or modify models.
No Flame Design per-run outcome, validation, held-out or test payload was opened.

The same two hash-pinned splits admit exactly **154 TRAIN identities** before payload
access. The v1 source and receipts, every raw source hash, the old measurement source,
and the new implementation hash are frozen in
`runs/dev/data_astra/work/endpoint_train_audit/trace_v2/protocol_frozen.json` before replay.
Its SHA256 is `10dde0c07d66d7331c91f61713e4e4cb0e6fb63fc0982d2fbc4467966bca85f6`.

## Operational result

| Item | Result |
|---|---:|
| Frozen TRAIN identities inventoried | 154 |
| Sensor files / command files | 153 / 153 |
| Sensor data rows preserved | 1,942,217 |
| Sensor parse failures | 4, all in 21106F1 |
| Sensor duplicate/backward-clock observations | 7 |
| Sensor missing intervals above two nominal periods | 5 |
| Command parse failures | 0 |
| Command rows with a duplicate timestamp | 4,612, preserved in original order |
| TP clock links by exact matching name and unique pre-test start | 133 |
| TP alias linked by unique pre-test start | 1 |
| TP clock unresolved because no TP metadata is present | 19 |
| Eligible optical diagnostic traces | 144 |
| Refused optical estimator inputs | 8: seven clock defects, one corrupted source |
| Unresolved onset/script binding | 1: 21196M2_1 |
| Missing exact sensor source | 1: 21130A9 |
| Traces with confirmed later returns above the lit threshold | 4 |
| Last-observed loss candidates | 143 of 144 eligible traces |

The diagnostic denominator is **144**, not v1's 145. The additional refusal is warranted
by source bytes that the old reader silently discarded. A source refusal remains part
of the all 154 inventory; it is never silently removed from the reported population.

All **306 ledgers were independently decoded after execution**. Concatenating their
preserved row bytes reproduced the pinned source SHA256, byte length, row count and
contiguous offsets for every file: **1,948,244 physical rows, 726,514,735 bytes**.
Receipt: `trace_v2/lossless_verification.json`. This establishes lossless storage of these
files, not scientific accuracy or completeness of the NASA corpus.

## Reader and evidence format

Implementation: `tools/wsp/astra/endpoint_trace_v2.py`. Each exact TRAIN source produces
`train/<family>_<test>/sensor_rows.jsonl.gz` or `command_rows.jsonl.gz`, plus an
`inventory.json`. The gzip ledger includes **every physical line**, including metadata,
schema, malformed records, line endings and a final line without a newline. Each entry
contains the original bytes as Base64, physical line number, byte offset and length,
parsed clock when available, and explicit errors. The source hash is checked at EOF.

The schema requires a unique `Time` column, the named PMT/flow channels for sensor data,
or the exact three voltage setpoint columns for commands. Every numeric column is
checked for parseability and finiteness; unknown auxiliary channels remain raw source
columns and are not assigned invented units. Wrong field counts, malformed clocks,
invalid UTF-8, missing fields and nonfinite values are recorded. A mismatched header
identity refuses the body parser. There is no sorting, deduplication, interpolation,
replacement of damaged bytes, or implicit conversion of missing values to zero.

The sensor and command tables have different temporal semantics: sensors are sampled
traces, whereas commands are irregular recorded events. Tied command timestamps are
retained as distinct source rows; they do not prove simultaneous physical actions or
justify deleting a row. Every changed setpoint vector is retained as a command action.
Its human-versus-automatic actor and physical causal effect remain unidentified.

### Concrete newly exposed corruption

`21106F1` contains four malformed source lines, each with **1,460 NUL bytes**:

| Physical line | Byte offset | Byte length | Parser finding |
|---:|---:|---:|---|
| 4736 | 1,794,128 | 1,823 | Invalid numeric field 16 |
| 5998 | 2,254,001 | 1,814 | Field count mismatch; invalid field 18 |
| 6982 | 2,613,044 | 1,804 | Field count mismatch; invalid field 30 |
| 9578 | 3,554,956 | 1,813 | Field count mismatch; invalid field 9 |

These are retained verbatim in that run's sensor ledger. The v2 global dark estimator
refuses the damaged input rather than pretending those intervals were observed. This
does not establish whether corruption originated in acquisition, publication or a local
copy. Repair requires an independently verified source copy, not numerical interpolation.

## Acquisition clock, script clock and burn-time origin

The acquisition header supplies a full calendar date, day-of-year and millisecond clock.
The TP stage log supplies a full calendar date and whole-second clock. Both are retained;
neither is asserted to be UTC because these source fields do not declare a timezone.
The encoded day in a test name is not substituted for the acquisition date. Thus an
overnight session can keep its frozen identity while its recording date advances.

The TP clock is linked only when its **pre-test `Script start`** agrees with the sensor
header at TP's one-second precision and that start is unique among the admitted TRAIN
headers. Post-test timing, labels, signal similarity or score improvement cannot grant
a link. The 133 exact-name links and one alias link pass this documentary condition.
For 19 files with no TP metadata, the separate TP clock remains unresolved.

For `21196M2_1`, the sensor header explicitly names `21196M2_1` and starts on
**2021-07-15 at 22:05:42.879**. Its exact-routed TP file names `21196M2` but records
**2021-07-15 at 22:05:42**. That unique pre-test start binds the TP calendar clock to
this acquisition. The other TRAIN `21196M2` acquisition starts at 21:45:45.227, so it
cannot satisfy that mapping. This resolves the TP clock/alias using metadata alone.
The as-run parameter file still says `testpointName:21196M3` and has no independent
clock binding. **That script remains refused.** No new onset, planned stop or endpoint
is invented for this partially resolved identity.

Calendar rollover is applied only when a negative half-day wrap has a next-day date
corroborated by an already-bound TP log. Source clock bytes are always retained alongside
the calendar-relative interpretation. An uncorroborated wrap is an explicit failure.
No admitted file required a day rollover in this replay; crossing midnight and starting
acquisition after midnight are covered by synthetic tests. The seven observed small
duplicate/backward sensor steps are retained, not reinterpreted as midnight.

The table's implied hot-ignition burn-time origin is a **third, scientifically unresolved
definition**, not a timezone adjustment. The previously documented five-second
stabilization-stage explanation for 21270A1/A2/A4 remains source-supported, but v2 applies
no universal offset to table durations. TP stage names also remain source labels; a
`Constant flow` stamp is not silently assumed to mean the start rather than completion
of that stage. Original radiometer onsets are reused only from the pinned unchanged v1
receipts tied to the same sensor bytes; they are not newly qualified physical onsets.

## Distinct optical diagnostics

The old PMT dark estimator, 0.2s centred smoothing, 0.05V dark/lit threshold, 0.5s dark
confirmation and 0.2s lit confirmation are unchanged. The lit hold is the old PMT-onset
hold, not a noise threshold selected to match the table. No parameter search or fitting
was performed. Gaps above two median sample periods split observational support.
The global block-based dark estimator is refused for parse failures or non-increasing
time, because sorting or deleting samples would change its temporal meaning.

Each eligible trace separately reports:

- **First observed dark interval:** the first measured below-threshold span, whether
  confirmed or too short to confirm.
- **First confirmed dark interval:** an observed span covering the full old 0.5s hold.
- **Confirmed later relighting:** a later above-threshold span covering the old 0.2s
  hold in the same uninterrupted segment after confirmed darkening. This field means
  a sustained *optical threshold return*, not proven physical re-ignition.
- **Last-observed loss candidate:** a confirmed final dark span that reaches recording
  end. Its future is right-censored; permanence after EOF is never inferred.
- **Command actions and temporal order:** every change in the command log, with separate
  ordering diagnostics against the old selected command. Unknown clock alignment,
  overlapping processing support or insufficient separation produces unresolved order.
  Physical cause is always unresolved, even when temporal order is clear.

A short dark suffix at EOF never satisfies confirmation. Left/right censoring is explicit
on each span, unknown acquisition intervals are retained, and recording-end censoring
always states that subsequent physical state is unobserved. 143 traces have a last-observed
dark candidate; `21242A1` remains above the lit threshold through the final recorded sample.
There were no unconfirmed EOF dark spans among the eligible 144 in this replay.

The four recurrent-signal examples now have the following distinct measurements. Times
are seconds from the pinned old onset; none is substituted for the registered endpoint.

| TRAIN test | First confirmed dark span begins | Confirmed later lit spans | Last-observed dark span begins |
|---|---:|---:|---:|
| 21153C4 | 242.993 | 26 | 275.584 |
| 21153G1 | 126.168 | 2 | 130.112 |
| 21291A1 | 40.672 | 5 | 167.056 |
| 21291C1 | 89.088 | 5 | 246.072 |

This resolves the software conflation of a first dark episode with the last observed
loss. It does not determine whether oscillations are flame transitions, gain/noise
effects or another instrument phenomenon. Physical interpretation still needs independent
instrument/video evidence. The processing-support brackets are not calibrated confidence
intervals and do not account for all sensor-response uncertainty.

## Tests, execution and remaining work

Commands actually run:

```powershell
python -m unittest discover -s tools/wsp/astra -p test_endpoint_trace_v2.py -v
python tools/wsp/astra/endpoint_trace_v2.py --freeze
python tools/wsp/astra/endpoint_trace_v2.py --execute
python -m unittest discover -s tools/wsp/astra -p 'test_endpoint*.py' -v
```

Results: **13 new tests passed; 24 combined v1/v2 tests passed; full replay completed**.
Tests exercise exact byte preservation through malformed/invalid-UTF8 lines, unknown
header identity before body parsing, duplicate/backward/gapped time, documented and
undocumented midnight, millisecond units, schema rejection, unique pre-test alias linking,
EOF confirmation, recurrence versus final loss, and unresolved causal ordering.

The independent post-run ledger verification decoded each gzip JSONL, Base64-decoded
each row, checked sequential line numbers and contiguous offsets, concatenated bytes into
SHA256, and compared row count, byte count and digest with every inventory's source receipt.
All 306 matched; the individual verification receipts are retained. No network or model
execution occurred. No Rust/frontend checks were run because those implementations were
not touched. No old measurement, shared record, score or forward was edited.

Remaining unresolved items are explicit and operational: recover the exact missing
21130A9 source; obtain authoritative script identity for 21196M2_1; obtain 19 missing TP
logs if the primary source provides them; verify an uncorrupted 21106F1 copy; and obtain
documentary clock-repair evidence for the seven non-increasing traces. Until then the
relevant gates continue to refuse scientific inference. The A3/H2/H3 reference-duration
conflicts, instrument calibration, and physical distinction between hot-to-cool transition
and terminal extinction still require independent blind adjudication under a newly frozen
measurement protocol. These diagnostics do not authorize a new Flame Design validation look.
