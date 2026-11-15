# The logged what-if tool for the hand-written laws (PROTOCOL 13.6)

`tools/wsp/ext/law_whatif.py` lets the author of a law check a typed candidate on the development cases (train and
validation days, in-sample since PROTOCOL 13.4) before anything is assembled. It never reads a test day, never
writes a release, a patch or a file under `configs/`, and every evaluation is logged.

## Commands

```bash
# what the law reads and every number of the base, as edits (the column order of every W1 and skip)
python -I tools/wsp/ext/law_whatif.py describe --slug cld --base configs/models/law/cld/d1-fable

# the development cases as one flat CSV: inputs, outcome, base forecast, per-case loss, partition, day
python -I tools/wsp/ext/law_whatif.py cases --slug cld --base configs/models/law/cld/d1-fable --out runs/dev/gen7/whatif_cases/cld_d1.csv

# one logged check of a candidate: the base's numbers plus the changed lines of a file
python -I tools/wsp/ext/law_whatif.py run --slug cld --base configs/models/law/cld/d1-fable \
    --edits my_step.wsp --note "methane at 25-40 % from 0.16 to about 0.4; ethylene unchanged; radiometer unchanged"
```

| slug | base directories | scoring-time rule |
|---|---|---|
| `bre` | `configs/models/law/bre/w0-fable` (also `d1-fable`, `d2-fable`) | `--rule flame_end` (the registered mapping: `flame_end` cut at the lane's planned stop) or `--rule lasts` (phys_bre2.md section 2) |
| `cld` | `configs/models/law/cld/d1-fable` | the declared stop at the planned fuel-off |
| `cld2` | `configs/models/law/cld2/s1-fable` | as cld |
| `cld3` | `configs/models/law/cld3/s1-fable` | as cld; three causes (blowoff, other, crossing), the loss is their sum |
| `slice2` | `configs/models/law/slice2/s1-fable` | `p_lifted = p_lift_noted (p_past_lift - p_past_blow) / (1 - p_past_blow)` |
| `bass2` | `configs/models/law/bass2/s1-fable` | quench_u from `quench_speed` or `flat_quench_speed` by geometry; sustain = the smaller of the two holds targets |
| `efield3` | `configs/models/law/efield3/s1-fable` | as efield; T2 = the sum of the three causes (blowoff, press, other) |
| `bass3` | `configs/models/law/bass3/s1-fable` | as bass2 |
| `bass4` | `configs/models/law/bass4/s1-fable` | as bass2; fabrics and Ultem answer kept burning only |
| `slice` | `configs/models/law/slice/w0-fable` (also `d1-fable`) | as in 13.2 |
| `efield`, `spice`, `samer` | `configs/models/law/<slug>/w0-fable` (spice: also `lit-fable`) | as in 13.2 |

`--coding v1|v2` (alias `--cases`; default v1, the registered version): the plastics outcome coding for bass2, bass3 and bass4 (`law_score.BASS_CODING`) and the SLICE case version for slice and slice2 (`law_score.SLICE_CASES`, the tube from the picture table); refused for any other slug. It is printed in the run's header and logged as `coding` in each line of the log.

The development cases are those of the data lanes' case files, with the inputs of `tools/wsp/ext/law_inputs.py`
(validation lines of SLICE without cabin oxygen get the train median, 0.219122, as in 13.2).

## What `run` does

1. Reads every number of the base release (`ignis wsp inspect --edits`) and applies the lines of `--edits` under
   the compiler's rules. A tensor the slug's spec does not have, an index outside its shape, an element written
   twice in the file, or a value outside the tensor's bounds is refused, and nothing runs. The file holds changed
   lines only, in the edit language of section 7 of the dossiers (single elements, `[i:j]` slices with a list or
   one number, `[i, j:k]` rows).
2. Runs base and candidate on the train and validation cases with the Python copy of `law.rs`
   (`tools/wsp/ext/law_pyforward.py`, equal to the Rust forward to about 1e-14 on every train case of the nine
   slugs, and to 3e-16 on a random sample of 240 BRE `lasts` evaluations), then applies the slug's scoring-time rule.
3. Prints, for each scored target, on the cases both answer:
   - the lane's score (its own `score()`), pooled, by partition, beside every baseline of 13.2;
   - the strata of the failure analysis (`docs/wsp/feedback/gen7_failure_analysis.md`), for base and candidate:
     cases, share of the loss, sign of the error (median error and 50 % and 90 % band coverage for quantile
     targets; mean forecast against observed rate, and calibration by probability bin, for yes/no and event
     targets);
   - by day (the mean per-case loss of the day), with **days lower over days whose forecasts differ**, the count
     the keep rule of 13.4 and 13.6 uses.
4. Appends one line to `runs/dev/gen7/whatif_log.jsonl`: UTC time, slug, rule, base directory and release id,
   the path and sha256 of the edits file, the number of elements changed, the note, the pooled scores of base and
   candidate, and the day counts.

The note is the prediction of 13.6, written before the run: which strata move, in which direction, by about how
much. It is logged with the result so that the prediction and the realised effect sit side by side.

## Worked example

The check of the tool itself: the base's own value rewritten, so nothing should change.

```text
$ echo "loss.blowoff.median.b0 = 6.03" > same.wsp
$ python -I tools/wsp/ext/law_whatif.py run --slug cld --base configs/models/law/cld/d1-fable --edits same.wsp \
      --note "engineer's tool test: the base's own value rewritten, no change expected"
# what-if: cld, base configs/models/law/cld/d1-fable (sha256:a03ec603…4d4f), rule flame_end; 1 elements changed; development, in-sample

## target loss (on the cases base and candidate both answer; lower is better)

| metric | partition | base | candidate | incidence (simple) | by fuel | loss by dilution ramp | loss, classical LFL ... |
|---|---|---|---|---|---|---|---|
| integrated Brier | pooled | 0.317 | 0.317 | 0.328 | 0.274 | 0.228 | 0.455 |
| integrated Brier | train | 0.125 | 0.125 | 0.149 | 0.141 | 0.071 | 0.189 |
| integrated Brier | validation | 0.514 | 0.514 | 0.478 | 0.388 | 0.328 | 0.676 |
...
## loss (per case: mean Brier over the horizons at risk; calibration of lost before fuel-off)
Base: By fuel: | methane | 114 | 0.086 | 73 % | 0.102 | 0.219 | -0.117 | ...
Candidate: (the same)
By day (mean per-case loss): candidate lower on 0 of 0 days whose forecasts differ (25 days).
logged -> runs/dev/gen7/whatif_log.jsonl
```

Read it as follows. A real candidate changes some lines, and the tables then show where its loss moved. On the
pooled line of the target that lost, it must be lower than the base. It must also be lower on at least two thirds of
the days whose forecasts change, and not more than 5 % worse pooled on the other targets (13.4, 13.6). Where fewer
than six days change, it is carried as a comparator, not as the model. Refusals print one line, for example
`refused: element loss.blowoff.shape[] written twice`, and are not logged.

## Limits

- A candidate here is numbers on an existing declaration. A new input or a new target (a structural change) needs a
  new slug from the engineer (13.6, "structure before numbers"); the tool refuses a tensor the spec does not have.
- The per-day table uses the mean per-case loss of each day. For yes/no and quantile targets this is the lane's
  per-day score. For the integrated Brier of event targets (bre, cld, efield T2) the lane's per-day score
  integrates per horizon; the pooled and per-partition lines above it are the lane's own.
- At most five logged candidates per law in this round (13.6); the log shows the count.
- The engineer's tool tests of 15 November are in the log with the note "engineer's tool test", one per slug, each a
  rewrite of a base value, not a candidate.
