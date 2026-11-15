# Agent report (Opus 5.5 medium, 15 Nov 2026, extracted from transcript): published flame-spread models vs BASS

I've written both files. The short answer: a meaningful BASS head-to-head isn't possible with our data, because the test set is only 2 burns (T11, T14). Scoring them would mean opening the test tracks, so I stopped before that. Nothing was committed or pushed, no IGNIS weight was fitted or touched, and the branch is unchanged.

**Full texts and what each one gives**

| Source | Status | What it gives |
|---|---|---|
| Zhao et al., PCI 36 (2017) | Paywalled, not read | The same work is open in two places: the 2015 NTRS paper 20150008962 and §7.1 of NASA/TM-20210011385, which I read in full. Its Table 7.1 lists all 27 BASS-II burns plus 4 BASS burns (width, flow start→end, O2, outcome). Fig. 7.4 gives the steady spread rate (Vf) of 10 burns as a raster plot, which I digitised. There is no predictor equation. |
| Zhao & T'ien, CF 2015 | Paywalled | A 3D transient CFD model; there is no closed form to rebuild. |
| Li et al., PCI 37 (2019) | Paywalled | Saffire (large samples in a large duct), not BASS. |
| Li & Liao, PCI 38 (2021) | Author manuscript blocked (HTTP 403) | Only second-hand, via Olson et al.: Vf ∝ P·U·X², with no constants, for wide samples. Not usable without anchoring it to some other number. |
| Budzinski & DesJardin, CF 2023 | Read (OSTI copy) | A 2D model checked against BASS opposed-flow 100 µm PMMA films; it overpredicts. It does not cover SIBAL concurrent spread, so it can't be compared with the fabric tip. |
| NASA PSI-62 | Listed | One file only: a 9,694-byte metadata zip (`i_Investigation.txt`, `s_PSI-62.txt`). No reports, tables or per-test predictions. |

I also read three open papers:
- **Ferkul et al. 2012 BASS-I poster:** the only predictor with published constants. 2 cm sample: Vf[mm/s] = 0.20·U + 0.56. 1 cm sample: 0.15·U + 0.07. Both at about 21 % O2, U in cm/s.
- **Olson et al. accepted manuscript:** two BASS burns in its Table 1, with Vf 0.27 and 0.51 cm/s. Its spread correlation needs the measured flame length and height, so it can't forecast.
- **Li et al., CF 227 (2021):** says Vf ∝ √U, with no constants.

The Ferkul–T'ien Kimwipes model, scaled by area density (×0.11), gives about 0.29 mm/s at 2.5 cm/s and 18.8 % O2. The measured value is about 0.8 mm/s, so I didn't use it.

**Which burns are whose**

All of these are recorded with sources in `docs/wsp/research/bass_published_burns.json`. The split groups come from `data/splits/bass-4083156d5b18dab4.json`.

- **Train:** T5 (0.784 mm/s at 2.4 cm/s), T6 (0.838 at 2.6) and T16 (0.825 at 2.99) are in Fig. 7.4. T10 did not ignite.
- **Validation:** T1, T2, T4, T9, T15, T18, T19, T21–T29. T19 is in Fig. 7.4 (1.2 cm, 1.52 mm/s) but its track failed QC. T1, T2 and T18 have tracks but aren't in Fig. 7.4, and the paper flags T1/T2's O2 readings as possibly wrong.
- **Test:** T11 (0.982 at 4.0 cm/s) and T14 (0.878 at 3.7 cm/s) are in Fig. 7.4. T14 is the probable match: T17 can't be fully excluded.
- **External (the earlier BASS campaign, PSI-26):** GMT131-T10 (2.03 mm/s), GMT222-T11 (2.77 mm/s, Olson gives 2.7) and Olson's "BASS 4" = GMT131-T4 (5.1 mm/s). All three match our existing tracks.

**Reproducing their numbers** (steady slope of our tracks against theirs)

| Burn | Error (100 mm / 4.302 w = 23.25 mm per w) | Error (22 mm per w) |
|---|---|---|
| T5 | +6.9 % | +1.2 % |
| T6 | −0.3 % | −5.6 % |
| T16 | +3.6 % | −1.9 % |
| GMT131-T10 (12 mm per w) | +0.1 % | |
| GMT222-T11 (12 mm per w) | −4.3 % | |
| GMT131-T4 (22 mm per w) | +18.6 % | |

GMT131-T4 is the one poor match, and the crew noted that burn was "not steady". Within about 7 %, our tracks are their burns.

One discrepancy to know about: the investigators' flows (2.4, 2.6, 3.0 cm/s) are not what our fan calibration gives. Ours reads 1.78–2.96 cm/s during the steady phase of T5/T6 and 2.37–3.55 for T16.

**Comparators in the script** (`tools/wsp/final/posthoc_bass_published.py`)

It copies the scorer's cases exactly; D1's train CRPS comes out at 0.0843, the same as in `CONVERSION.md`. All four comparators start from the same position at the cutoff, so they differ only in spread rate:
- **steady_prefix:** the tip moves at a constant rate fitted over the last 10 s before the cutoff. This is their way of measuring Vf, applied to what is known at the cutoff.
- **pos12:** the BASS-I fit at the track's own flow history, no O2 correction.
- **pos12_o2:** the same, scaled by X/0.21. That scaling is my assumption, not a published fit.
- **fig74_oracle:** each burn's own published steady Vf. It reads the burn's future, so it's a ceiling for any constant-rate forecast, not a forecast.

**Dry run** (sample widths; a point forecast's absolute error equals its CRPS):

| | D1 CRPS | D1 median error | steady_prefix | pos12 | pos12_o2 | fig74_oracle |
|---|---|---|---|---|---|---|
| Train (4 files, 16 cases) | 0.0843 | 0.110 | 0.468 | 0.384 | 0.206 | 0.093 (0.084 at 22 mm per w) |
| Validation (3 burns, 11 cases) | 0.121 | 0.179 | 0.228 | 0.449 | 0.352 | — |

D1 is lower than every forecast comparator on every run. It roughly ties the oracle, which is lower than D1 on 3 of the 4 train files. Two caveats: D1 was written after its author had seen train, and train is the only place the oracle can be scored.

**Verdict**
- **Number of burns:** the test holds 2 burns, 7 cases, from one day. Both have published rates, so a head-to-head on the same cases is possible but has no statistical power.
- **Best comparators:** pos12_o2 is the most faithful forecast built only from published constants, though the O2 scaling is mine. steady_prefix is their method applied at the cutoff. fig74_oracle should be reported only as a ceiling, never as a forecast.
- **Quench and ignition outcome:** with 2 burns, these can't be compared meaningfully.
- **Next step:** scoring the test needs the evaluator's final path, which the script doesn't implement.

The dry-run output went to the scratchpad, not to the script's default `runs/final/20261115/posthoc_bass_published.json`. While working I noticed untracked files from another session, `docs/wsp/feedback/published-acme-bass/` and `docs/wsp/research/phys_bass_study2.md`; I left them alone.

Files are in D:\dev\SpaceApps-Ardealu:
- docs\wsp\research\bass_published_burns.json
- tools\wsp\final\posthoc_bass_published.py

Downloads and dry-run output are in D:\Caches\Temp\claude\c2f4976cab650b4b\199e808b-35cd-44e2-918a-d476aca69199\scratchpad:
- papers\bass\
- bass_scripts\dry\posthoc_bass_published.json