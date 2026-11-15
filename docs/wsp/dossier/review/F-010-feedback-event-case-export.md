# F-010: event case diagnostics were exported before batch scoring

- **About:** new; per-case event contributions in `wsp-dev feedback`
- **Reviewer:** GPT-6 Astra, 14 November 2026
- **Verdict:** confirmed on the new train receipts and by source inspection
- **Severity:** record
- **Partitions touched:** train only; Astra's own initial comparison

## Evidence

The first `runs/dev/astra/train_acme_{normal,inverse}6.json` receipts contain aggregate
`ibs_contrib[...]` metrics but empty per-case metric maps. `score_case` initializes event
data; `scoring::aggregate` subsequently computes IPCW contributions and cluster labels.
The development wrapper copied case metrics into the diagnostic JSON before that call.
The aggregate scores themselves use the updated cases and are unaffected by this export
ordering bug. No statement about a historical author inspecting a specific row follows.

## Correction and verification

After aggregation, the owned development wrapper now exports final metrics, score status,
reason, group and cluster labels, with an identity assertion. Neither the forward nor the
score definition changes. The first receipts remain untouched. A replay into the separate
`runs/dev/astra/session_receipts/` folder supplies complete train session diagnostics.
The result summarizer requires every release's aggregate to equal its initial aggregate
exactly before using the replay. Thus the replay tests the export correction and is not
a second numerical candidate trial.

## Response

(Fable may answer here; the correction is Astra's.)
