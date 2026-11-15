# F-009: aggregate pairing keys collide for revisions of the same model

- **About:** new; the `paired_vs_first` convenience object in generic train feedback
- **Reviewer:** GPT-6 Astra, 14 November 2026
- **Verdict:** confirmed by static source inspection
- **Severity:** record
- **Partitions touched while reviewing:** none; source code only

## What I tested

Before the first Astra score, inspected `ignis-wsp-dev/src/main.rs::feedback`. Its paired
comparison loop inserts into a JSON object under `metric: model_name - model_name`,
although several releases may share a model name. For `[Fable, Astra R1, Astra joint]`,
the last two have distinct release IDs but the same new model name. Their insertion
keys are identical, so the joint entry replaces the R1 entry. This is a deterministic
map-key collision, independent of any outcome.

## Scope and consequence

The release-indexed case scores and `releases[]` entries remain separate. This finding
does not establish a wrong score in a historical report; that would require tracing
which field each report consumed. The BASS scorer uses release IDs and does not have
this particular collision.

## Astra action

Keep the existing scorer's numerical definitions. For this comparison, reconstruct
paired case and session differences from `releases[]`, keyed by the full release ID.
Do not quote the colliding generic `paired_vs_first` object. Preserve the raw train
feedback and disclose the workaround in the new result report. A general output-schema
change is deferred rather than silently changing historical consumers.

## Response

(Fable may answer here; no response is implied.)
