# F-002: fifth-generation validation execution and per-case handling

- **About:** X-20 / X-21 / X-22; D-45 / D-46; fifth-generation validation execution
- **Reviewer:** Codex audit agent, 14 November 2026
- **Verdict:** confirmed (X-21 splitter implementation only); cannot tell (X-20 and X-22 chronology)
- **Severity:** integrity
- **Partitions touched while reviewing:** other — source code and aggregate documentation only; no validation inputs, case rows, vaults, or test material opened

## What the dossier says

The fifth-generation validation look used a private holdout vault for private-root droplet continuations (X-20), and `cmp_split.py` divided burner-flame comparisons by data root while printing aggregates (X-21). The report says the author read aggregate validation tables only. X-22 records a comparison of carried forecasts on validation prefixes to explain a score discrepancy, with no outcomes used.

## What I did

Read the aggregate record in `docs/wsp/PROTOCOL.md` §10 and `docs/wsp/RESULTS_GEN5.md`; inspected only the source of `tools/wsp/dev_eval.py` and `tools/wsp/analysis/cmp_split.py`. I did not run either script or any evaluator, and did not open a comparison output, vault, per-case row, validation input, or test artifact.

## Evidence

Static source inspection establishes the following:

1. `dev_eval.py` lines 252–262 have a validation path that calls `ignis infer forecast-cohort ... --partition validation ... --execute --seal`, then `ignis eval compare ... --partition validation`; line 262 separately invokes `wsp-dev feedback ... --partition train`. The source therefore routes validation predictions through the sealed evaluator and its comparison command. It does not itself set or verify `IGNIS_HOLDOUT`, so the recorded use of the private vault cannot be independently authenticated from this file.
2. `cmp_split.py` lines 73–89 load the mixed and private comparison JSON documents, including `cases` arrays with per-case metrics, and build private/shared subsets by `(run_id, prefix_hash)`. Lines 101–112 compute cluster means, per-case integrated Brier aggregates, and paired differences. Its normal output is aggregate-only and its optional output file contains aggregates. The code confirms that the author-owned splitter operates on evaluator case rows; it does not establish that the author viewed or otherwise inspected per-case outcome values. Reading rows internally to compute an aggregate is inherent in aggregation and is consistent with the dossier's stated aggregate-only author read. The code does not print case metrics.
3. `cmp_split.py` lines 95–134 checks that the private-half aggregate matches the evaluator's private comparison aggregate (`worst <= 1e-9`). This verifies numerical agreement of the aggregates, not the provenance or correctness of the case-to-root assignment. The case key set comes from the supplied private comparison's first release; the script contains no independent source-root marker check. This finite paired-average check must not be treated as proof that the split corresponds to the intended data roots.
4. The aggregate protocol/results document the private-root droplet vault and the burner-flame double-root comparison. Without inspecting the vault or run logs (which I did not do), I cannot establish that deposits, seals, and score runs followed the written chronology. The X-22 forecast-only explanation also cannot be independently confirmed from the allowed source code; `cmp_split.py` handles validation outcomes, not the claimed carry-forecast identity comparison.

## What should change

Retain X-20 and X-22 as documented but not independently verified by this review; their chronology verdict is **cannot tell** from the allowed sources. For X-21, preserve the distinction between the aggregate-only author-facing output and the author-owned splitter's internal use of evaluator case rows. This source inspection does not establish human per-case outcome exposure or a violation of the aggregate-only reading rule. It does identify a process-control boundary: an author-owned post-processing script performs the data-root split outside the evaluator, and its numerical check alone does not authenticate the grouping. The data/evaluation owners should audit the split provenance from source-root metadata or an evaluator-produced root label, without exposing per-case outcomes to authors. No numerical result or model decision was recomputed here, and no further validation look is warranted to resolve it.

## Response

(The author answers here.)
