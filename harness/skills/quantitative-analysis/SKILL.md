---
name: quantitative-analysis
description: Answer analytical questions by computing from IGNIS datasets and artifacts with analysis_execute, reporting every number as a computed claim bound to the artifact value that holds it.
---

# Quantitative analysis

## 1. Know the data first

- Call `datasets_inspect` before computing: read the views, columns, units, `time_origin` and
  partitions, and use the real `dataset_id` it returns. Experimental times are seconds since that
  origin; say which origin you use. For run conditions use `conditions_query`; it states gaps
  (a missing pressure cell stays missing).
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
- A quick run answers `completed` with its output artifacts; a longer one `accepted` with a job:
  call `jobs_get` (it waits for the job to end) and again only while it is still running. Read the
  output artifact with `artifacts_read` (a JSON Pointer reads one field). Read stderr when a run fails, fix the
  program, and run it again, as often as it takes; do not report results of a failed run.
- Check the output before you report it: row counts against the inputs, units, excluded values,
  and whether the result is plausible. Recompute a surprising value another way (a second program,
  a hand check in the program on one run, a source that states it) before stating it. Run follow-up
  analyses when the first one shows the question needs them (a subgroup, an uncertainty, a check of
  an outlier).

## 3. Report what was computed, and its limits

- Each reported number is a `computed` claim that binds it in `values`, never as a bare numeral:
  `{"value": 1.23, "unit": "mm2/s", "source": {"kind": "artifact_cell", "artifact_id": <output>,
  "sha256": <its digest>, "column": "/burning_rate"}, "derivation": {"computation": <the
  ExecutionRecord artifact>, "inputs": [<every mounted input>]}}`. Pick a row of a table with
  `"rows": [{"table": "/rows", "where": [{"field": "/run_id", "equals": "..."}]}]`, by key fields,
  never by position. The host reads the cell back, records it as `resolved` and compares your value
  in SI; you may omit `value`/`unit` to show the cell as it is. A pointer to a single number in
  `support` is reported as an ungrounded number.
- An `observed` claim (a measured value) must cite evidence: a sources_search hit of class
  original_observation or source_derived. Bind its number either as `"source": {"kind":
  "evidence_quote", "evidence_id": ...}` (the host checks the number and unit are written in that
  hit's own passage) or as an `artifact_cell` of a dataset view (the view's artifact_id and sha256
  from `datasets_inspect`, `"rows": [{"table": "", "where": [{"field": "/run_id", "equals": ...},
  {"field": "/time_s", "equals": ...}]}]`, column such as `/value_si`). `conditions_cell` needs a
  conditions snapshot digest that `conditions_query` does not return yet: do not use it. A value of
  the conditions table or a view that no passage states is reported through an analysis that reads
  it (a `computed` claim whose text says it is the recorded value read from that table). Numbers in
  PDF tables are bound only as text of a hit's passage; never retype them from memory. A number you
  only recall is at most a `background` claim (unverified, no `values`), never a run's measured
  result and never a stand-in for a value the data or a passage holds.
- State the method in one line (fit, average, interval). Report the effect size with its
  uncertainty and sample size, computed by the program; a p-value alone is not a result, and
  significance is not importance.
- Name the interval: SD, SE, or a confidence interval with its level (or coverage factor k). Say
  whether instrument or measurement uncertainty is included; if no source or view states it, write
  "measurement uncertainty not reported". Round values to the digits the uncertainty supports
  (`significant_digits` in `values`).
- Report only quantities the program defines in its output; do not invent a metric in the text.
- "Computed, nothing notable" (no difference, no trend within the uncertainty) is a valid `complete`
  answer bound to the computed values, not an abstention. Abstain only when the data needed is
  missing, conflicting or incoherent, and name what is missing.
- A fit of the data is a computation, not a validated model, and not a prediction outside the
  measured range.
- Say which runs or conditions the result covers; extrapolation beyond them is an `interpretation`
  at most, flagged as such.
