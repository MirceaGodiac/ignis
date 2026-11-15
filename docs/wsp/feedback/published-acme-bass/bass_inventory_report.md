# Agent report (Opus 5.5 medium, 15 Nov 2026, extracted from transcript): unscored BASS burns inventory

Almost nothing usable is left. The only NASA-published SIBAL concurrent burns that D1 can forecast and no release has scored are two BASS-II burns, T13 and T20. Neither is an independent test, so the draft batch is a 2-burn tip-only check of about 1.25 GB, not a fresh test partition. Nothing was downloaded, committed or pushed.

**What exists and where it stands**

| Source | Burns | Status for D1 |
|---|---|---|
| PSI-25 fabric split | 25 SIBAL (T1, T2, T4–T6, T9–T12, T14–T19, T21–T29) | All scored. T11 and T14 were already scored in `RESULTS_FINAL.md` Step 4; validation is validation |
| PSI-25 T13 (GMT 100) | 2 cm SIBAL, concurrent, 17.5 % O2 | **Never scored**, but on the train day of T5/T6/T16. Left out because its video name is spelled "test T13". The blind measurer's duct track failed QC because the video's overlay could not be read anywhere after igniter-off, so it has no fan history |
| PSI-25 T20 (GMT 190) | 1 cm SIBAL, concurrent, 17.2 % O2 | **Never scored**, but on a validation day (T19/T21/T22). It has no main-view video, only a second view, an original-videos zip and stills |
| PSI-25 other materials | 93 burns in `PSI-25-solids-days` (PMMA films, slabs and rods; Nomex F1/F3; candle F4) plus T3 and T7 (4 cm spherical sections) | D1 refuses any sample label that is not SIBAL and has no law for spheres. `front5` gives only event probabilities for a PMMA sheet, with no tip law |
| PSI-25 B17, O10, O17, P8, P10 | non-SIBAL, stills but no main-view video | Refused. F2 and O20–O22 have no files at all |
| PSI-26 2012, SIBAL concurrent with stills (as-run tests 2, 4, 5, 10, 11) | 5 | Already tracked and compared with D1 (`bass1_psi26_tracks.md`). Published rates for 4, 10 and 11 are transcribed |
| PSI-26 2012 tests 1, 7, 8 (GMT 090/096) | 3 | No stills or video in the PSI listing, so they cannot be measured |
| PSI-26 opposed (3, 5b, 6, 9), Nomex (12–15), and PMMA, spheres, candles, wax, Ultem | rest of the 122 as-run rows; all 114 videos from 2013 | Refused by D1. The 2013 videos were measured blind and all failed QC |
| Saffire I and III S1 (SIBAL sheets, 40.6 × 94 cm), Saffire II 2-5 and 2-6 (5 cm strips) | 4 | **Already spent** by the registered transfer test (`docs/wsp/SAFFIRE_TRANSFER.md`; results in `feedback/saffire-transfer/results/`). Their opposed and non-SIBAL samples are refused |

**What T13 and T20 can be scored on**
- **Tip forecast only.** The investigators' Table 7.1 already gives an end-state comment for both burns, and it is transcribed in `docs/wsp/research/bass_published_burns.json`. So "track ended" and "ignition" cannot be scored fresh.
- **Spread rates are still unseen.** Neither burn is in the published Fig. 7.4, so no rate for either has been read.
- **Fan calibration exists** for both GMT 100 and GMT 190.

**Blockers**
- **T13 has no fan history.** The main video's overlay could not be read. The original-videos zip may hold a capture with a readable overlay; if it doesn't, the scorer refuses the track. The tip needs a new top-view track from the stills, measured the same way as the train tracks.
- **T20 depends on the overlay too.** Either the second view or the zip has to carry it, or the burn cannot be scored. It is a 1 cm sample, so it rests on D1's narrow-sample rule, which no train burn supports. The duct view's 2 cm width conversion fails for 1 cm samples, so the band width must be measured in the top-view stills.
- **No mm scale is needed**, because D1 scores in sample widths.

**Proposed batch** (`docs\wsp\atlas\splits\BASS-batch-fresh.draft.json`, untracked)
- **Groups:** GMT 100 (train day) holds T13; GMT 190 (validation day) holds T20. Both are marked held and scored on the tip only.
- **Downloads:** 1,245,150,437 bytes in total.
  - T13: stills zip 696.6 MB, main mp4 65.7 MB, original-videos zip 24.5 MB, proof sheet, post-burn clip.
  - T20: stills zip 437.2 MB, original-videos zip 14.7 MB, second view 4.7 MB, stills movie 1.0 MB, proof sheet.
- **Time:** about 80 minutes on one stream at 0.25 MB/s, or a few minutes with ranged parallel downloads. Nothing else in D1's domain is left to spend the 150 GB budget on.
- **Do not open before scoring:** the T13 post-burn clip, the last stills of each sequence, the TM Table 7.1 comments and Figs. 7.4–7.7, `bass_published_burns.json`, and every field of T13's existing track except its QC flags.

**What I read along the way**
- **Test aggregate scores:** I saw D1's 2-burn aggregate test numbers in `RESULTS_FINAL.md` Step 4 while checking which burns were scored.
- **T13 and T20 end states:** I read both end-state comments in `bass_published_burns.json`. That is why the batch is tip-only.
- **T13 track:** of the measurer's T13 track I read only its QC flags.

I opened no holdout folder or evaluator vault, used no `--final-test`, and opened no frames or videos.