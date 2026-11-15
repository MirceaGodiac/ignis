# Brief for a data and scoring engineer: one NASA investigation, made forecastable

Repository: D:\dev\SpaceApps-Ardealu, branch feat/handwritten-science. Read AGENTS.md first.

## The goal of the whole effort
The owner wants a hand-written model with a scored forecast for EVERY one of NASA's 19 flight combustion
investigations. In this project a model's parameters are numbers an author model types, with a physical reason;
nothing is fitted (docs/wsp/README.md). Nine investigations have a scored forecast; the others have none. A
physicist is writing a zero-shot law for your investigation at the same time, without seeing any outcome
(`docs/wsp/research/phys_<slug>_w0.md`, when it appears). Your job is everything that law needs in order to be
run and scored blind: what can be forecast, from which inputs, against which recorded outcome, on which tests,
and how it is scored.

## Read
- `docs/wsp/PROTOCOL.md` sections 11.1 to 11.3 and 12.1 to 12.3: how a test on an investigation outside the
  project's partitions is prepared (conditions first; the outcomes of held days are not read by authors).
- For your investigation(s): its section in `docs/wsp/atlas/INVESTIGATIONS.md` and in `docs/wsp/atlas/FILES.md`
  (which file kinds exist), its card(s) under `docs/wsp/atlas/cards/`, its frozen day groups
  `docs/wsp/atlas/splits/<PSI>.json` (train, validation, test days), its row in
  `docs/wsp/dossier/ASTRA_COVERAGE_19.md` (what an earlier author thought the safety observable is).
- The data: every text, table and document file is on disk under `runs/dev/data_all/raw/<PSI>/` (layout in
  `runs/dev/data_all/README.md`; file catalogue `runs/dev/data_all/work/atlas/files.jsonl`; test catalogue with
  conditions `runs/dev/data_all/work/atlas/tests.jsonl`). Images and video are NOT on disk.
- How it was done for a sibling: `docs/wsp/feedback/flame_design_transfer_feasibility.md`,
  `tools/wsp/ext/flame_design.py` (endpoints from raw channels), `tools/wsp/ext/acme_script2.py` (the planned
  stop and the burner limit from the script file), `tools/wsp/ext/transfer_acme.py` (the scoring chain).

## Deliver
1. `docs/wsp/feedback/<slug>_feasibility.md`:
   - One or two **forecast targets** that matter for fire safety and are recorded for most tests. For each: its
     definition, unit, where the outcome is recorded (table column, timeline stamp, sensor channel, analysed
     file, or only in images), what a forecast may know (pre-test conditions; optionally the first seconds of a
     sensor record up to a cutoff), the cutoffs and horizons, and the score (CRPS for a number, integrated Brier
     for an event in time, Brier for a yes or no).
   - The **inputs**: every condition a law could use, with unit, source file and how far it can be trusted;
     which are known before the test starts.
   - **Counts by partition** (train, validation, test days) of tests that have the inputs and the outcome from
     the text files alone. Test days: counts from the catalogue only; no file of a test day is opened.
   - **Baselines** computed on train days only: a constant or incidence baseline, persistence where it applies,
     and the classical correlation of the field if one exists (name it and its source; use it at its published
     constants).
   - What cannot be done without images or video, with the GB it would take.
2. `tools/wsp/ext/<slug>_cases.py`: builds, under `runs/dev/data_all/work/transfer_<slug>/`,
   `cases_train.jsonl` (train days: inputs and outcome), `cases_validation_inputs.jsonl` (validation days:
   inputs only) and `cases_validation_outcomes.jsonl` (kept apart, never printed), and has a `score` function:
   given forecasts for a set of cases it returns the aggregate score and the per-day scores. Show that the
   baselines run through it on train days.
3. One paragraph for the physicist at the top of the feasibility file: "a forecast is asked for THIS quantity,
   from THESE inputs (names and units as they are in the case files), in THIS form (quantiles at 5, 25, 50, 75,
   95 %; or event probabilities at these times)".

## Rules
- Train days: you may read everything. Validation days: inputs only; their outcomes go to the separate file by
  script and are never printed, plotted or summarised. Test days: nothing is opened.
- Never open `holdout/` or `runs/dev/holdout_ext/`; never pass `--final-test`.
- Nothing is fitted except the constants of the named baselines on train days.
- Other agents work in this checkout: no git commands; new files only, under `tools/wsp/ext/`,
  `docs/wsp/feedback/` and `runs/dev/data_all/work/transfer_<slug>/`; no sub-agents.

## Final message (concise)
The target(s) and why; counts by partition; the baseline scores on train days; the inputs; what is missing or
untrustworthy; and whether the outcome needs images.
