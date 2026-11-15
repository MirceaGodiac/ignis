# tools/wsp/attrib — where the train loss sits, and which numbers can move it

**Diagnostic only. A perturbation is a question about the model, not a proposed value.**

These are steps 2 and 3 of PROTOCOL.md 12.3 ("decompose", "attribute"). The tool runs the real forwards
through the built binaries (`target/release/wsp-dev.exe`, `target/release/ignis.exe`) on the TRAIN
partition and reports. It never proposes a value, never writes into a model, and never touches
`configs/` or `artifacts/models/`. Moving a number stays the author's job (12.3 step 4).

## Safety

- TRAIN only. `--partition` accepts nothing but `train`. `--final-test` is never passed. Paths with a
  `holdout` or `holdout_ext` component are refused (data root, candidate, blocks, outputs). For
  `bass_front*` the track folder is fixed to `tools/wsp/bass_optical/tracks/train`, and every file in it
  must say `"partition": "train"`.
- Isolated registry. Every forward, the base included, runs with `IGNIS_ARTIFACTS` pointed at a scratch
  copy of the candidate under the output folder. Children are assembled with `--root` set to that
  scratch registry. The scratch registries are deleted at the end (`--keep-scratch` keeps them).
- Each report records the entry count and a listing hash of `artifacts/models/` before and after the
  run. `sensitivity` also records a content hash of the candidate's config tree, and exits with code 2
  if either one changed.
- Outputs go only under `runs/dev/attrib/`.

## `decompose`

```bash
python tools/wsp/attrib/attrib.py decompose --family acme_normal7 --candidate configs/models/acme/wsp7/normal/p3-fable
python tools/wsp/attrib/attrib.py decompose --family flex_methanol2 --candidate configs/models/flex/wsp2/methanol/d3-fable
```

Options: `--data-root <IGNIS_DATA>` (for example `runs/dev/data_ext`; the default is `data`), `--out`
(must be under `runs/dev/attrib/`; the default is `runs/dev/attrib/<family>_<candidate dir>`), and
`--n-boot` (bootstrap draws inside feedback, default 50; it changes only the confidence intervals,
never the case rows or the totals).

The tool runs `wsp-dev feedback --partition train`, as `tools/wsp/dev_eval.py` does: event models get
`--times 6,8,…,120` and droplets keep the default 10 s horizon at cutoffs 1, 3 and 5 s. For
`bass_front*` it runs `wsp-dev front-feedback` on the train tracks with `--detail`. It writes
`decompose.md`, `decompose.json` (every case row is included) and `feedback_train.json`.

**Loss.**
- Droplets: the CRPS of the diameter forecast, as a case mean.
- Burner flames: the sum over causes of each case's integrated-Brier contribution (`ibs_contrib[cause]`,
  IPCW). The mean of each cause's contributions equals the evaluator's integrated Brier for that
  cause. The sum includes every cause the scorer reports. For normal flames `unresolved_support` is 0,
  so the sum equals the 12.6 decision score.
- Fabric: the CRPS in sample widths. A case is the mean over its targets, a run is the mean over its
  cases, and the total is the mean over runs (the front protocol).

The header shows the totals as `wsp-dev feedback` reports them and as recomputed from the case rows,
and says whether the two are equal.

**Groups.**
- Session: the evaluator's cluster (for BASS, the burn).
- Cutoff.
- Regimes for droplets: fuel (from the train records via `tools/wsp/analysis/lib.py`), O₂ mole fraction
  (< 0.16, 0.16–0.20, 0.20–0.25, ≥ 0.25), pressure (< 80, 80–150, ≥ 150 kPa), diluent (He present,
  CO₂ present, N₂ only), and initial diameter (< 2.5, 2.5–3.5, ≥ 3.5 mm).
- Regimes for burner flames: fuel (the records' `fuel_or_material`), pressure (< 75, 75–150, 150–250,
  ≥ 250 kPa), ambient O₂ (< 0.25, 0.25–0.38, ≥ 0.38), and era by the session date. The era bounds are
  those of acme_normal.wsp7 P2: early up to 2021-08-30, late from 2021-09-27.
- Regimes for BASS: initial O₂ and restrictor, from the track files.

**Columns.**

| column | meaning |
|---|---|
| cases | scored cases in the group (refused cases are counted in the header) |
| CRPS / loss | the group's mean loss as defined above |
| share of loss | the group's sum of case losses over the sum over all cases |
| IBS ‹cause› | mean per-case integrated-Brier contribution of that cause (event models) |
| bias of median µm | case mean of the signed error of the median, prediction − observation. It comes from feedback's lead bins (0–2, 2–5, 5–10 s), which are rounded to 1 µm, weighted by their point counts |
| MAE of median | the evaluator's `mae_median`, as a case mean |
| cov 50 % / cov 90 % | the share of continuation points inside the central 50 % and 90 % bands |
| CRPS if centred | the CRPS the same bands would score if their median sat on every observation. For the evaluator's quantile CRPS this is exact: Σ over symmetric level pairs of 2·w_τ·τ·width (with levels 0.05…0.95 it is 0.0125·width₉₀ + 0.1125·width₅₀) |
| centred share | CRPS if centred, over the actual CRPS. Near 100 %, the loss is the width of the band; low, the median is off |

Event models also get the Brier score of each cause at every horizon, and a reliability table. That
table holds the evaluator's own calibration bins: four bins of forecast probability per horizon, with
the mean forecast and the IPCW observed frequency, shown at 10, 20, 30, 60 and 120 s (the JSON has
every horizon). A table of the 15 cases with the largest loss gives run id, session, cutoff, loss
(with its parts for events), regimes and conditions.

## `sensitivity`

```bash
python tools/wsp/attrib/attrib.py sensitivity --family flex_methanol2 --candidate configs/models/flex/wsp2/methanol/d3-fable \
    --blocks tools/wsp/attrib/examples/flex_methanol2_d3.blocks
python tools/wsp/attrib/attrib.py sensitivity --family acme_normal7 --candidate configs/models/acme/wsp7/normal/p3-fable \
    --blocks tools/wsp/attrib/examples/acme_normal7_p3.blocks
```

The blocks file holds one perturbation per line: `<tensor>[selectors] <step>`, with `#` for comments.
The tensor and selectors use the edit language: committees take the slot-prefixed name, for example
`b0.se.diff.skip[7]`. Selectors are integers, `a:b` or `:`, and no selector means the whole tensor.
A step is absolute (`+0.1`, `-0.02`) or relative (`x1.1`). For each line the tool:

1. reads the current values from the candidate's stored tensors and computes the new ones (each
   element of a box gets the same step);
2. writes a child edits file with one literal statement: `set` for one element, `set_box` with a
   nested list for a box. A changed value is written to 12 significant digits; an unchanged value is
   written exactly. It copies the candidate's `meta.json` with `parent_release_id` set to the
   candidate, a probe `patch_id`, and a rationale saying the release is a diagnostic. With the
   candidate as parent, that single statement over the parent's tensors gives exactly the candidate's
   edits plus this statement. The candidate's own edits are not written again: rewriting an element
   the patch already sets would be refused as "written twice";
3. runs `ignis wsp compile` and `ignis wsp assemble --root <scratch>` in the probe's own scratch
   registry, then the same train feedback as the base;
4. pairs cases by (run, cutoff) with the base and reports.

The jobs run in a process pool (`--workers`, default 4). A statement the spec refuses is reported as
`refused:` with the assembler's message and the run goes on. That covers a write to a constant, a
value outside the declared bounds, an unknown tensor and an out-of-range selector.

**Output.** `sensitivity.md` opens with the banner, then one table sorted by absolute effect. Each
perturbation then gets its own section with Δ per regime, Δ per session, and the five cases that
improve most and the five that worsen most. `sensitivity.json` holds everything: old and new values,
statement, child id and weight hash, child totals, per-session, per-regime and per-case deltas,
timings and the isolation check.

| column | meaning |
|---|---|
| perturbation | target and step as written in the blocks file |
| old → new | the value before and after (the element count for a box). "(weights = candidate's)" means the child's weight hash equals the candidate's |
| Δ loss | child loss − base loss, each over its own scored cases. Negative means lower loss on train |
| Δ % | Δ loss over the base loss |
| Δ by cause | event models: Δ of each cause's integrated Brier |
| sessions better / worse / all | sessions whose mean paired Δ is below / above ±10⁻⁹ × base loss |
| cases moved | paired cases whose loss changed by more than that tolerance |
| largest session Δ | the session with the largest absolute mean Δ |

A Δ answers one question: "what does this number do to the train loss at this step?". It does not say
the step is the right one. PROTOCOL 12.3 asks the author for evidence across sessions, the source
range, damping, rounding and a physical reason before any number is moved.

## Runtime (measured 15 November, this workstation, 4 workers)

| model | base feedback | per perturbation (compile + assemble + feedback) |
|---|---|---|
| flex_methanol.wsp2 D3 (droplet, 182 cases) | 2.7 s | 2.7–2.8 s (assemble 0.06–0.10 s) |
| acme_normal.wsp7 P3 (committee of burner flames, 210 cases) | 0.9 s | 1.1 s (assemble 0.12 s) |
| bass_front.wsp D2 (12 cases) | < 0.1 s | 0.1 s |

The perturbations run in parallel, so the wall time for N of them is roughly ⌈N/4⌉ × that, plus the
base.

## Limits

- The bias of the median comes from feedback's lead bins, which are rounded to 1 µm. Feedback does not
  export the forecast quantiles, so no finer number is available without a change to the Rust code.
- The reliability table uses the evaluator's four quantile bins per horizon. Feedback does not export
  per-case CIFs, so the tool cannot rebin them.
- BASS rows carry no band widths, so there is no centre-against-spread split for the fabric model.
  Its "session" is the burn, and the train set has 3 scoreable burns and 12 cases.
- Δ loss compares each release's own scored cases. If a step changes which cases are refused, the
  counts "only base" and "only child" in the section say so; the session and case deltas use paired
  cases only.
- The regime bins are fixed thresholds that describe the train cases. They were chosen from the
  condition values, not from losses.
