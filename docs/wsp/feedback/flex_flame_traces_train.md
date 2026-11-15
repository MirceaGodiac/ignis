# FLEX flame cameras on the train runs: when the hot flame ends and what the first seconds say

> Written by a delegated analysis agent (claude-opus-5-5) on 14 November 2026; nothing fitted. Train runs only, outcomes included.
> The author rechecked the claims he acted on; which ones, and what stays unchecked, is in
> `docs/wsp/dossier/03_datasets_and_measurements.md`. Paths under `runs/dev/` are local working folders that
> are not in git; the dossier says how to rebuild them.

I've measured hot-flame end times for all 117 train runs, sorted the endings into the four kinds and run the two carried models at 1, 3 and 5 s. Nothing outside the train lists was opened; a few metadata-only exposures are listed at the end. The main findings:
- **Where the clock is off:** both models get the hot-flame clock right on average. It runs late for large-droplet radiative losses, especially heptane in helium (×1.5). It runs early for heptane burn-outs (×0.87) and for the heptane 2 atm CO2 run.
- **Wrong kind of ending:** the only clear case is heptane 240R002.
- **Task 5:** the first seconds of the flame trace do not separate early from late flame loss.

## 1. Coverage and what the traces mean

| | Train runs | UV trace | Colour trace | Both | Neither |
|---|---|---|---|---|---|
| methanol | 68 | 68 | 59 | 59 | 0 |
| n-heptane | 49 | 49 | 37 | 37 | 0 |

Four of the colour files have no data rows (FLEX-059, 224, 225, 247), so 55 methanol runs have both traces in practice.

**Format.**
- `*_LLUVData.txt` columns: `LLUVTime_sec`, `LLUVFlameDiameter_mm`, `LLUVFlameStandoff`, `DropletXpos_mm`, `DropletZpos_mm`.
- `*_ColorCamData.txt` columns: time, flame diameter, standoff.
- Lines end with a bare CR. Most traces run at 30 frames/s; 15 UV and 19 colour traces run at 15 frames/s.
- Time zero is igniter-on, the same as the droplet series.
- Standoff is exactly flame diameter divided by the droplet diameter from `DropletData` (checked frame by frame). It is blank whenever the droplet isn't tracked.
- The two position columns are the droplet's displacement since the start, not its distance from the edge of the view.

**The trace stops when the flame goes out, not when the camera stops.**
- The droplet recording ends at about 33.5–33.8 s, but only one flame trace reaches 33 s (FLEX-099, 33.0 s).
- In 30 methanol and 22 heptane runs the droplet is still tracked after the flame trace ends.
- About 10 traces show explicit zero frames after the flame. The last row is often a single spurious frame; I dropped any frame more than 1.6× its neighbours.

**Cross-checks against NASA:**
- The droplet diameter I read at the trace end matches NASA's Dext within 0.1 mm in 36 of 37 methanol runs (median difference 0.002 mm) and 26 of 29 heptane runs. The heptane exceptions are 0.30–0.74 mm off: FLEX-175 (Dext extrapolated), FLEX-102 and FLEX-267.
- All 25 narratives saying the droplet left the view "after flame extinction" have the droplet still tracked when the trace ends. All 13 "disruption coincident with extinction" runs whose droplet stayed in view have the droplet series ending within 0.3 s of the flame trace.

**When the two cameras disagree**, I took the later end, since the flame exists while either camera sees it. The disagreement is almost always a camera losing the droplet from its view:
- The colour end matches the UV end within 0.5 s in 42 of 55 methanol runs and 32 of 37 heptane runs.
- The 2009 calibration runs (CAL) have unreliable UV settings, as their sheets say; there the colour trace sets the end.
- Six narratives say the flame left the UV and colour views before it went out (FLEX-185, 010, 047, 207, 229, 209). Their trace end is only a lower bound, so I classed them (d).

## 2–3. Ending kinds

Classification rules:
- **(c) cool flame:** the droplet keeps shrinking at ≥0.20 mm²/s over both 0.2–3.2 s and 3.2–8.2 s after the flame.
- **(a) radiative, (b) diffusive / burn-out:** droplet diameter at flame end over D0 at least 0.65 is (a), below it is (b).
- **Source of that diameter:** measured first, then NASA's Dext, then √(1 − t·kavg/D0²) from NASA's kavg.
- **(d) cannot tell:** an indirect diameter ratio between 0.55 and 0.75, or a heptane droplet not seen afterwards.

| Fuel | a | b | c | d |
|---|---|---|---|---|
| methanol | 15 | 44 | 0 | 9 |
| n-heptane | 5 | 21 | 15 | 8 |

How the (b) diameters were obtained:
- methanol: 23 measured, 17 from Dext, 4 from the kavg clock.
- heptane: 7 measured, 4 from Dext, 10 from the kavg clock (sooty runs that left the view).

**By atmosphere (a / b / c / d).**

| Atmosphere | methanol | n-heptane |
|---|---|---|
| N2, 17–25 % O2, 1 atm | 0/12/0/0 | 0/6/1/3 |
| N2, <17 % O2 | 3/4/0/3 | 1/0/3/1 |
| N2, ≥30 % O2 (1 and 0.7 atm) | 0/7/0/0 | 0/4/0/1 |
| CO2, 17–25 % O2, 1 atm | 0/7/0/0 | 1/4/5/1 |
| CO2, 17–25 % O2, 0.7 atm | 3/6/0/1 | 0/4/3/1 |
| CO2, <17 % O2 (1 and 0.7 atm) | 6/0/0/0 | – |
| He, 17–25 % O2 (1 and 0.7 atm) | 1/7/0/2 | 3/3/1/1 |
| He, <17 % O2 | 2/0/0/0 | – |

Methanol also has 1 (b) at 0 atm (FLEX-114) and 3 (d) at 2–3 atm. Heptane also has 1 (c) at 2 atm CO2 (FLEX-018) and 1 (c) in N2 at 0.7 atm.

**Typical flame-to-droplet diameter ratio (UV), medians.**

| Kind | 1 s | 3 s | 5 s | Just before the end | End time | Droplet/D0 at end | Shrink after / before |
|---|---|---|---|---|---|---|---|
| methanol (a) | 4.9 | 5.6 | 5.7 | 5.6 (stays up) | 4.1 s | 0.87 | 0.41 (0.12 mm²/s) |
| methanol (b) | 4.5 | 5.2 | 5.5 | 4.4 (falls) | 12.8 s | 0.44 | 0.12 (0.05 mm²/s) |
| heptane (a), mostly helium | 8.4 | 10.2 | 12.0 | 11.9 | 5.4 s | 0.79 | 0.60, decaying |
| heptane (b) | 7.9 | 9.7 | 11.1 | 16.6 (keeps rising) | 12.0 s | 0.16 | – |
| heptane (c) | 7.3 | 8.8 | 9.3 | 9.1 (flat) | 4.4 s | 0.91 | 0.82 (0.34 mm²/s) |

- The helium heptane runs shrink fast right after the flame (0.35–0.46 mm²/s), but the rate halves within 3 s (0.13–0.18). Their narratives say there was no cool flame.
- The kinds agree with the narratives. Every (c) carries a "cool flame" note. The (a) methanol runs are tagged radiative or outside the flammability boundary, or say nothing. No (b) run is called radiative, except one heptane narrative that calls itself "on the edge".

## 4. Model clock against measurement

312 cases; 33 refused: 4 by the condition check (FLEX-114, 175, 279, 280), the rest semantic. The ratio below is the model's time for P(hot) to fall through 0.5 over the measured flame-end time; medians with ranges.

| Fuel, kind | n (1/3/5 s) | Ratio median | Range | Model early in |
|---|---|---|---|---|
| methanol (a) | 14/10/7 | 1.15–1.17 (late, +0.6 to +1.2 s) | 0.89–6.4 | 8 of 31 |
| methanol (b) | 43/42/37 | 0.98 (−0.2 to −0.4 s) | 0.42–1.22 | 76 of 122 |
| heptane (a) | 5/4/3 | 1.49–1.53 (late, +2–3 s) | 0.94–1.56 | 2 of 12 |
| heptane (b) | 19/18/14 | 0.86–0.88 (−1.2 to −1.5 s) | 0.39–1.59 | 49 of 50 timed (1 never fell below 0.5 by 45 s) |
| heptane (c) | 14/11/4 | 0.90–1.02 | 0.12–1.17 | 14 of 29 |

**Worst classes (kinds a–c only).**
- **heptane, 2 atm CO2, (c) (FLEX-018):** ratio 0.37. At 3 and 5 s the model already says the hot flame is gone, and it says so with probability 1.0. The UV trace shows the flame until 8.0 s.
- **heptane, helium 0.7 atm, (a):** ratio 1.51, n = 8. The model holds the hot flame 2–3 s too long.
- **heptane, CO2 0.7 atm, (c):** ratio 0.74, n = 8.
- **heptane, helium 1 atm, (b):** 0.81.
- **heptane, N2 1 atm, (b):** 0.87, n = 15.
- **methanol, radiative (a) at low O2:** helium 1.28, CO2 0.7 atm 1.21, CO2 1 atm 1.20, N2 1.14.
- **methanol, N2 ≥30 % O2:** early in all 20 cases (0.93–0.95).

**Wrong kind with probability above 0.7.**
- Methanol: none.
- Heptane FLEX-018 (240R002): the 0.95–1.0 probabilities go to the state at the cutoff, not to the kind. At 1 s the model says cool (P = 0.95), which is right, but its hot clock ends at the 1 s cutoff. At 3 and 5 s it says diffusive (P = 1.0) and that the flame is already gone.
- Heptane FLEX-182 (C02H201, model cool flame P = 0.74–0.79): my rule called this (a) only because the late shrink rate is 0.198 against the 0.20 threshold. Its narrative says cool flame, so the model is probably right.
- Just under the threshold: FLEX-124 (radiative at 2.4 s, model diffusive P = 0.66 at 1 s). This is one of the two FLEX-124/141 records the docs already say are one video analysed twice.

## 5. Would the first seconds of the flame trace help?

Not as a separator. Early loss means kinds (a) and (c); late means (b).

| Fuel, cutoff | Quantity | Early loss | Late | Runs inside the overlap |
|---|---|---|---|---|
| methanol 1 s | UV flame/droplet ratio | 4.05–6.33 | 3.51–6.54 | 47 of 58 |
| methanol 1 s | UV ratio growth per s | 0.22–1.94 | −1.11–1.19 | 50 of 58 |
| methanol 3 s | UV ratio | 4.66–7.29 | 2.81–7.10 | 37 of 50 |
| methanol 5 s | UV ratio | 4.84–6.78 | 4.23–7.51 | 34 of 42 |
| methanol 5 s | colour flame / D0 (best case) | 4.37–5.31 | 2.02–4.75 | 12 of 36 |
| heptane 1 s | UV ratio | 6.59–9.20 | 4.71–13.31 | 31 of 41 |
| heptane 3 s | UV ratio | 7.98–11.02 | 6.58–18.59 | 28 of 36 |
| heptane 5 s | UV ratio | 8.97–12.11 | 7.40–17.41 | 17 of 24 |
| heptane 5 s | UV ratio growth per s | −0.22–0.78 | 0.25–1.52 | 19 of 24 |

Early-loss runs have slightly higher medians in methanol (4.9 against 4.5 at 1 s) and lower ones in heptane at 5 s (9.7 against 11.1). The overlap is near-total at every cutoff. The size relative to the critical diameter is the stronger signal; this table does not separate the flame's contribution from it.

## Caveats
- The kind thresholds (0.65 diameter ratio, 0.20 and 0.25 mm²/s shrink) are my choices, set from the gaps the docs describe, not fitted.
- 21 of the (b) classifications rest on Dext or the kavg clock, not a measured diameter. The six view-censored runs are a lower bound only.
- Flame end is the later of the two cameras, and colour traces are missing for 13 methanol and 12 heptane runs.
- The 15 fps traces give 0.07 s resolution.
- FLEX-124/141 are one video counted twice, and FLEX-017 is the gas-bubble run.

## Files and bytes
Everything is in `D:\dev\SpaceApps-Ardealu\runs\dev\research5\flex_flame\`:
- `flame_end_train.csv` (one row per run, task 2) and `flame_end_train.json`
- `model_vs_measured.csv` and `.json`
- `summary_tables.md`
- `fc\*.json` (279 forecast artifacts)
- scripts: `trainmap.py`, `load.py`, `fetch.py`, `measure.py`, `forecast.py`, `summarize.py`
- `narratives_train.txt`, `listing_train_flame.json`, `fetch_log.json`

The flame files are in `D:\dev\SpaceApps-Ardealu\runs\dev\nasa\PSI-69\flame\`: 203 files, 3,990,177 bytes, every size matching the listing. 10 more train files were reused from `data/raw/samples/PSI-69/`.

Total downloaded: about 5.5 MB. Besides the 3.99 MB of data, the first pass ran without redirect-following and saved 203 HTML redirect pages (752,790 bytes, since deleted). The second pass received about the same again in redirect pages.

I modified no git-tracked file. `git status` shows earlier changes in `crates/ignis-wsp` and `tools/wsp/reassemble.py` that are not mine.

## Anything outside the train lists
No file of a non-train test was opened, downloaded, printed or summarised. Metadata-only exposures:
- **FLEX-238:** run H02H101 (train) has the alias `FLEX-238` in its sheet header, but the real FLEX-238 files belong to H03H101, which is not train. My first filter therefore listed the name and size of `FLEX-238_LLUVData.txt`. I printed H03H101's record aliases, without outcomes, to confirm it isn't train, then switched to the file-number alias only, before any download. No FLEX-238 file was fetched or read.
- **Sample folder listing:** an early `ls` showed the names of every file in `data/raw/samples/PSI-69/`, non-train tests included, but no contents.
