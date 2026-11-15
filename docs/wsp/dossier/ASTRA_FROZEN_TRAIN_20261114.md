# Frozen Astra versus adopted Fable TRAIN comparison

Registered 14 November 2026, before this execution. Owner explicitly requested
all Astra candidates against Fable, protocol committed before scoring and results
committed afterwards. This is a development comparison, not a validation look.
Earlier TRAIN feedback exists: this is not a claim of first exposure or a new
independent experiment. All ten registered Astra candidates are **data-informed**.

## Frozen identities and scope

`../feedback/astra-frozen-train-20261114/freeze.json` pins the exact release IDs,
source hashes, split hashes, TRAIN track hashes and mixed-record file metadata.
Each of these five comparisons includes the baseline, `r1-astra`, `joint-astra`:

| Family | Adopted Fable | Astra |
|---|---|---|
| FLEX methanol | `flex_methanol.wsp2` D3 | `flex_methanol.wsp6` |
| FLEX heptane | `flex_heptane.wsp2` D2 | `flex_heptane.wsp6` |
| ACME normal | `acme_normal.wsp5` P1 | `acme_normal.wsp6` |
| ACME inverse | `acme_inverse.wsp5` P1 | `acme_inverse.wsp6` |
| BASS front | `bass_front.wsp` D1 | `bass_front.wsp3` |

No candidate selection, parameter revision, fitting or additional validation
look is permitted in this execution. Newer clean-room property kernels are
outside this registered-release comparison: they lack complete harness releases
and admitted inputs. They must be reported as not compared, never as winners.

## Data and execution

Use only the normal `data` root and `artifacts` registry. No private extended
root, holdout or sealed validation files. The frozen release split selects TRAIN
identities before records are materialized by `train_source.rs`. Shared JSONL
files are streamed for identity; non-TRAIN payloads are discarded, not exposed
to the author. This is not a claim of byte-level separation of the shared files.
File metadata must match before and after execution; mixed outcomes are not
opened to create the manifest. TRAIN BASS uses only committed `T*.json` tracks.

Use the existing evaluator-backed `wsp-dev` feedback code, with a byte-verified
executable snapshot. The wrapper records commands, timestamps, binary digest,
protocol commit and output hashes, refuses overwrites and verifies frozen inputs
before and after. Source build is identified by the frozen source commit plus
the file hashes; the executable digest is recorded on execution.

- FLEX: every TRAIN run in each frozen fuel split; cutoffs 1, 3, 5 s; 10 s horizon.
- ACME: every TRAIN run in each frozen geometry split; cutoffs 1, 3, 5 s;
  absolute event times 6, 8, 10, 12, 15, 20, 25, 30, 40, 50, 60, 80, 100, 120 s.
- BASS: existing `front.rs` protocol: cutoffs 5, 10, 20, 40 s after igniter-off;
  60 s target window; at least six flame stills, two prefix stills and three
  target stills. Event leads 10, 20, 30, 45, 60 s. Ignition at 20 s is reported
  by the existing scorer as a secondary descriptive diagnostic.

All releases in a family use the same eligible cases, supplied observations,
conditions, controls and scoring definition. At-risk exclusions, refusals,
missing targets and unscoreable tracks are retained in reports. The existing
at-risk filter can use TRAIN endpoint information to define the risk set; that
does not supply post-cutoff observations to the forward. The existing BASS
future-flow convention is retained for both authors, not silently corrected.

## Metrics, pairing and decision rule

Primary: FLEX diameter CRPS (reported in micrometres); BASS tip CRPS (sample
widths); ACME integrated IPCW Brier for extinction and flow termination, both
co-primary. Secondary: median absolute error and 50/90 percent coverage for
continuous forecasts, BASS track-ended Brier and ignition diagnostic.
Track-ended includes leaving view and must not be called pure extinction.

Pair each candidate with the baseline by `(run_id, cutoff_s)`, never model name.
Compute each metric only on their common finite scored pairs, and report lost
baseline cases, gained cases and refusals. Average paired case scores within
independent clusters, then average clusters equally. FLEX/ACME use the scorer's
union of split group and session; BASS uses `conditions.gmt_day`, not individual
tracks as independent replicates. Report the baseline and candidate values on
the identical common set alongside their difference (candidate minus baseline).

Bootstrap 10,000 samples of those cluster means with Python `random.Random`
seed 20261114. Diagnostic 95 percent interval: sorted samples 249 and 9749
(zero-based). With one cluster, no sampling interval is identifiable. These
intervals are descriptive, not multiplicity-adjusted significance claims.
IPCW terms use the evaluator's common observed censoring distribution; it is
not re-estimated inside this diagnostic paired bootstrap. If admission differs,
no overall win is declared and censoring-distribution differences are disclosed.

The report labels, fixed before scores:

1. Missing primary score: unscoreable.
2. Lost baseline scored cases or more refusals: coverage loss; no overall win.
3. Any worse primary mean: worse primary mean, or primary trade-off if another
   primary improves. Numerical equality tolerance is 1e-12 in reported units.
4. All primary means equal: level primary means.
5. Improved primary but worsened BASS track-ended Brier: secondary trade-off.
6. Otherwise lower primary means: paired interval supports improvement only
   when every primary upper bound is at most the equality tolerance; otherwise
   lower TRAIN mean, improvement uncertain.

Coverage and calibration diagnostics remain visible even if the primary score
improves. No result triggers adoption automatically. TRAIN performance of these
data-informed models cannot establish better unseen-session generalisation.

## Reproduce

Build `cargo build -p ignis-wsp-dev -j 2 --target-dir runs/tooling/astra-train-comparison-target`.
Commit this protocol, the freeze manifest and wrapper first. Then:

```powershell
python tools/wsp/astra/frozen_train_comparison.py --execute --binary runs/tooling/astra-train-comparison-target/debug/wsp-dev.exe
```

Output is the new `docs/wsp/feedback/astra-frozen-train-20261114/results/` directory.
Existing output is never overwritten. An execution/parsing failure is recorded;
a necessary mechanical correction must be documented without changing cases,
weights, metrics or this decision rule. Commit full TRAIN results and a readable
summary afterwards. No push or PR.
