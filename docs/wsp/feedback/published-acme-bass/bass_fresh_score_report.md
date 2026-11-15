# Agent report (Opus 5.5 medium, 15 Nov 2026, extracted from transcript): BASS fresh batch T13/T20 measured and scored

I measured both burns and scored them, but only with top-view tracks from a new script I had to add. Each burn has 3 tip cases and covers one run, so these numbers describe two burns. They don't estimate skill.

## Per burn

**T13 (GMT 100, train day, 2 cm, 17.5 % O2): measured.**
- **Duct view (the existing measurer, unchanged):**
  - The main mp4 failed QC again: "R5 overlay registered in 0 %".
  - The main-view `.ts` inside the original-videos zip passed QC with no flags. Its overlay was read in 107 of 107 seconds, so T13 now has a fan history: FAN 46 stepping to 42 at about 15 s, using the GMT 100 calibration.
  - After conversion, the duct track has only 19 points: 1.7 to 17.7 s, then 59.8 and 62.8 s. The duct view loses the dim flame after the fan step, so this gives only 2 cases.
- **Top view (new script, from the NASA proof sheet):**
  - Band 55.29 px, window 4.624 w, clock offset +0.30 ± 0.21 s.
  - 24 usable flame stills from 0.79 to 50.48 s. The stills stop with the flame still burning (`record_end`).
  - This gives 3 cases.

**T20 (GMT 190, validation day, 1 cm, 17.2 % O2): duct track refused, top view measured.**
- **Duct view:** the main `.ts` failed QC rule R6 (1-cm sample), so it was not converted, as the rules require.
  - Its overlay still gave a fan history: FAN 69 stepping to 46 at about 12 s.
  - Every T20 video lasts only about 30 s, so the fan history ends at 27 s. The scorer holds the last reading after that. That matches the table's two-value fan schedule, but it is an assumption.
  - The video starts with the igniter already on, so only the igniter-off bracket timed the stills: clock offset +2.54 ± 0.62 s.
- **Top view:** band 30.2 px. That is below the scorer's 44 px split, so D1 treats it as a narrow sample, on the narrow-sample rule no train burn supports. Window 8.445 w. 19 usable flame stills from 0.62 to 23.2 s, then the flame is gone (extinction at 23.98 s). This gives 3 cases.

## Scores (tip, in sample widths; cutoffs 5 / 10 / 20 s)
These come from `front-feedback`, the same scorer path as RESULTS_FINAL Step 4. Q0 and the carry give exactly the same tip scores as D1 (paired difference 0).

| Track | Cases | D1 CRPS | D1 abs. error of median | Cov. 50 / 90 % | steady_prefix | pos12 | pos12_o2 |
|---|---|---|---|---|---|---|---|
| T13 top view | 3 | 0.1581 | 0.1939 | 0.64 / 0.84 | 0.1837 | 0.2220 | 0.1931 |
| T13 duct (converted) | 2 | 0.1862 | 0.2327 | 0.43 / 0.92 | 0.2109 | 0.2743 | 0.2259 |
| T20 top view | 3 | 0.2699 | 0.3341 | 0.48 / 0.67 | 0.6974 | 0.4665 | 0.3676 |

The comparators are point forecasts, so their score is their absolute error.

**Paired differences, D1 minus comparator (negative means D1 is better; 1 run each):**

| Track | D1 CRPS minus steady_prefix / pos12 / pos12_o2 | D1 median error minus the same |
|---|---|---|
| T13 top view | −0.0257 / −0.0640 / −0.0350 | +0.0102 / −0.0281 / +0.0008 |
| T13 duct | −0.0247 / −0.0881 / −0.0397 | +0.0218 / −0.0416 / +0.0068 |
| T20 top view | −0.4275 / −0.1966 / −0.0978 | −0.3633 / −0.1324 / −0.0335 |

- **By cutoff:** D1 is weakest at the 20 s cutoff. On T13 it scores 0.2506, against pos12 at 0.1594. On T20 it scores 0.5189, and that case has only 3 target stills.
- **No oracle:** no `fig74_oracle` was run, because neither burn is in Fig. 7.4.
- **For context:** D1 scored 0.0761 on the original train top-view tracks and 0.0845 on my re-measurement of them (below).

## The top-view addition
`tools/wsp/media/fabric/topview.py` (new, sha256 `a79d05b9…0766`) re-implements the train top-view method from its written description. The original train scripts were never committed. Like the train tracks, it reads the 324 × 216 thumbnails in NASA's proof sheets. It takes the time origin, overlay clock, fan history and conditions from the measurer's duct track and does not change `measure.py`, `qc.py` or `enrich.py`.

I checked it on the train proof sheets (T5, T6, T16) against the committed train tracks:
- band width is 0.8–1.1 px narrower;
- the igniter-side origin is within 0.8 px;
- the window end is within 1 px, except T6, whose band is not lit to its end;
- the tip median differs by +0.03 to +0.07 w (p90 ≤ 0.10 w).

While building it I changed three of its rules after seeing T5 and T6, and partly the T13 window end (still positions only): which stills count as igniter-lit, how the window end is found, and which columns are used for the band width. I did this before any T13/T20 tip track was built, and the final version was applied identically to all five burns.

## Two things to know
- **The NEF stills were never used.** The stills zips held raw NEF files, and the train method uses the proof-sheet thumbnails. So those 1.13 GB were downloaded and deleted unused.
- **`posthoc_bass_published.py` change:** I added `--tracks DIR` and `--no-reproduce`. Its reproduction step reads `bass_published_burns.json`, so I skipped it to stay blind.

I did not open that file, Table 7.1 or Figs. 7.4–7.7, or the T13 post-burn clip, and I did not download that clip. I read only the QC flags of the old T13 track. I looked by eye only at early stills; nothing was opened in `holdout/` or `val_sealed`, and nothing was committed.

## Bytes
- **Downloaded:** 1,244,983,732 bytes for the batch (all files except the post-burn clip), plus 1,653,377 for the three train proof sheets used in the check.
- **Deleted:** 1,244,434,056 bytes of downloads (both stills zips, both original-videos zips, the three mp4s), plus 41,216,744 bytes of `.ts` files extracted from those zips.
- **Kept:** the five proof-sheet PDFs (2.2 MB), because they are the image source of the tracks.

## Files (all under `runs\dev\media\fabric\fresh\`, which git ignores)
- **Duct tracks:** `tracks\psi25\train\T13.json` (mp4, R5), `tracks\psi25\train\T13__BASS_GMT100_SAMPLE_137_TEST_T1.json` (`.ts`, ok), `tracks\psi25\held_validation\T20.json` (R6); converted track in `tracks_scorer\psi25\train\`, with `left_out.json`
- **Top-view tracks:** `tracks_topview\train\T13.json`, `tracks_topview\validation\T20.json`
- **Train check:** `topview_train_check\` (T5, T6, T16 and its score)
- **Scores:** `scores\` (`front_*.json`, `*_detail.json`, `published_*.json`) and the summary `logs\batch_scores.json`
- **QC images and overlays:** `check\`, `check_topview\`, `ovl\`, `BURNS.jsonl`
- **Logs:** `logs\` (fetch log, the three driver scripts, `measure.log`, `deleted_raw.json`); `manifest_raw.jsonl`
- **In the repo:** new `tools\wsp\media\fabric\topview.py`; modified `tools\wsp\final\posthoc_bass_published.py`