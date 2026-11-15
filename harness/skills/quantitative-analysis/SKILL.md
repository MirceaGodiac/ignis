---
name: quantitative-analysis
description: Answer analytical questions by computing from IGNIS datasets and artifacts with analysis_execute, reporting every number as a computed claim bound to the artifact value that holds it.
---

# Quantitative analysis

## 1. Know the data first

- Call `datasets_inspect` before computing: read the views, columns, units, `time_origin` and
  partitions. Experimental times are seconds since that origin; say which origin you use.
- Note missing, conflicting and not-applicable values. They are explicit statuses, never zeros: drop
  or flag them in the program and report how many were excluded.

## 2. Compute once, reproducibly

- Mount data by reference, never by pasting it into the program: each entry of `inputs` has a `name`
  (the file `/in/<name>`) and exactly one of `artifact_id` (an artifact a tool returned),
  `dataset_view` (`{"dataset_id", "view"}` from `datasets_inspect`; a Parquet file) or `conditions`
  (a `conditions_query` without `cursor`; the complete table, every page, as one JSON document).
  Compute over many runs this way instead of reading tables into the conversation.
- `/job/inputs.json` lists what was mounted: per input its artifact (sha256) and, for a view, the
  columns with units, `time_origin`, partitions and row count. Read units from there, not from memory.
  The execution record keeps the same list, so a result names exactly the data it read.
- Write one short Python program for `analysis_execute`: read inputs from `/in/<name>`, write results
  as JSON (and figures or tables as files) under `/out`. Put every number you will report in the JSON
  output with its unit, e.g. `{"burning_rate": {"value": 1.23, "unit": "mm2/s", "n": 14}}`.
- The runner has no network and only the inputs you mount. Keep raw units in the data and convert in
  the program, naming the conversion.
- The call returns a job: follow it with `jobs_get` until it finishes, then read the output artifact
  with `artifacts_read` (a JSON Pointer reads one field). Read stderr when a run fails, fix the
  program, and run it again; do not report results of a failed run.

## 3. Report what was computed, and its limits

- Each reported number is a `computed` claim whose support is the artifact value: the output
  artifact id plus the JSON Pointer of the field (`/burning_rate/value`). Do not retype numbers into
  text without that binding.
- State the method in one line (fit, average, interval). Report the effect size with its
  uncertainty and sample size, computed by the program; a p-value alone is not a result, and
  significance is not importance.
- Report only quantities the program defines in its output; do not invent a metric in the text.
- "Computed, nothing notable" (no difference, no trend within the uncertainty) is a valid `complete`
  answer bound to the computed values, not an abstention. Abstain only when the data needed is
  missing, conflicting or incoherent, and name what is missing.
- A fit of the data is a computation, not a validated model, and not a prediction outside the
  measured range.
- Say which runs or conditions the result covers; extrapolation beyond them is an `interpretation`
  at most, flagged as such.
