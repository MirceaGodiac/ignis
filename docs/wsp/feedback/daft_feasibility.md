# DAFT / DAFT-2 (PSI-47, held whole): what can be forecast, from what, and how it is scored

> Written by a delegated data agent (claude-opus-5-5) on 15 November 2026 for the seventh generation, from
> `runs/dev/gen7/BRIEF_DATA.md`. DAFT is held whole (`docs/wsp/atlas/splits/PSI-47.json`, two sessions, too few
> groups): inputs read, outcomes written by script to a separate file, never printed. No shared contract changed.

## For the physicist

A forecast is asked for **the number concentration `N` (1/m3) that the modified P-Trak condensation counter reads on
a sample of air**, from the inputs of `runs/dev/data_all/work/transfer_daft/cases_held_inputs.jsonl`: `air` (hepa:
filtered air; cabin: ISS cabin air through the sample hose; velcro: cabin air while Velcro is being separated; bag:
Arizona Road Dust dispersed in nitrogen in a 15 L Mylar bag), `aerosol`, `location` (2005 windows), `bag` and
`elapsed_time` (s, 2006 readings), `window_duration` (s), `dusttrak_mass_concentration` (kg/m3: what the reference
photometer read on the same air), `dusttrak_inlet`, `analyst_assumed_density` (kg/m3, bags only). In this form:
**quantiles at 5, 25, 50, 75 and 95 % of `N` in 1/m3**; the score is the quantile CRPS of ln(1 + N / 1e6 m^-3),
that is of ln(1 + particles per cm3), so that a filtered sample near zero can be scored.

## 1. Why this target

DAFT burned nothing. It tested whether the P-Trak, modified to return its isopropanol by capillarity, counts
correctly in microgravity, against a DustTrak, so that SAME could rely on it. The recorded pair on the same air is
a number (P-Trak) and a mass (DustTrak); their ratio is the mean mass per particle, which is the particle size
(the 2006 sheets compute a diameter from it). For fire safety it is the background a smoke detector must see
through, and the check of the counter SAME and SAME-R used. A detector alarm is not recorded: no smoke detector
was in the DAFT set-up.

Not chosen: the DustTrak reading itself (it is the reference input), the instrument agreement per HEPA zero only
(four cases), and anything from video (not in the archive).

## 2. Sources and cases

| Group | Source | Case | Cases |
|---|---|---|---|
| `PSI-47:session:D2` (catalogue, held) | logs `Raw Data/PSI-47_Raw Data_{PTrk,DTrk}000{2,3}.txt` (1 s); events from `Analyzed Data/PSI-47_Analyzed Data_flight data 063_TIMING.xls`, sheet TIMING (seconds after each instrument's log start, read by NASA from the video) | a window between two events, first 10 s dropped (the DustTrak's 10 s time constant), at least 20 s left | 11: 4 hepa, 4 cabin, 3 velcro |
| `PSI-47:2006-08-21:bags` (DAFT-2; not in the catalogue, held by the split file's note) | voice-downlinked readings, `Analyzed Data/PSI-47_Analyzed Data_ISS_Daft_Aug21_0611zgy time hist.xls`, sheet Data | one reading with both instruments | 42 in 3 bags |

The window rules were fixed in the script before any P-Trak mean was computed. Test 02 (ER4, Velcro) gives 2 hepa,
3 cabin and 3 velcro windows (a fourth Velcro burst of 26 s and two short gaps fall under 20 s); test 03 (Lab 1
Port 6) 2 hepa and 1 cabin window.

**Left out:** session D1 (GMT 053, near EXPRESS rack 4) and test 04 of D2: there is no event table, so the HEPA
zero cannot be separated from the sampled air, and the two instrument clocks differ (P-Trak and DustTrak logs start
51 min apart by their own clocks on D1). NASA synchronised them on video, which is not in the archive. The 2006
logged series (tests 5 to 7 in the "Retrieved Data Summary" workbook) duplicate the bag readings at 1 s; they are
not used, to keep one source per case.

## 3. Inputs and trust

| Name | Unit | Source | Trust |
|---|---|---|---|
| air, aerosol, location | - | TIMING events and labels; experiment table | high for the state; the Velcro windows are stamped from video by NASA |
| window_duration | s | TIMING | high |
| dusttrak_mass_concentration | kg/m3 | DustTrak log mean over the window (2005) or the reading (2006) | low in cabin air: the instrument resolves 0.001 mg/m3 and cabin air reads near that floor; NASA's note on D1 calls both instruments' readings low |
| dusttrak_inlet | - | final report (2.5 um impactor for the bags); not stated for 2005 | medium |
| elapsed_time, bag | s, - | the bag sheet ("elapsed time applied to all") | nominal, not the GMT of each reading |
| analyst_assumed_density | kg/m3 | the bag sheet | an analyst's assumption |

Log check: DustTrak test 02 holds 779 values against a declared 778 points; the extra trailing value does not
change any window. Shared quantity list: neither a number nor a mass concentration is a `Quantity`; the score is
lane-local (`score()` in `tools/wsp/ext/daft_cases.py`).

## 4. Counts (held)

53 cases in 2 groups, all with the reference input and an outcome: 11 in session D2, 42 bag readings. Session D1
(catalogue) has none. With so few groups the aggregate is the only number a look can report, and it is dominated
by the 42 bag readings; a group-equal mean is reported beside it.

## 5. Baseline for the registered look

No DAFT day is train, so nothing can be set on DAFT. The reference, fixed now from SAME-R train days only, is the
smoke number per mass: `N_q = k_q * dusttrak_mass_concentration`, with k at 5/25/50/75/95 % = 2.38e15, 8.86e15,
2.17e16, 3.56e16, 2.42e17 per kg (49 SAME-R train rows; written to
`runs/dev/data_all/work/transfer_daft/baseline_smoke_number_per_mass.json`). It is a declared transfer from smoke to
dust and expects dust to be counted like smoke of the same mass; dust particles are larger, so it should
overcount. It has not been scored. There is no classical correlation between a photometer's mass and a counter's
number for an unknown aerosol.

## 6. Images and video

One representative image (1.3 MB) and slides; no video in the archive. The video NASA used to synchronise D1 and
test 04 would add two sessions of windows; it is not obtainable from PSI.

## 7. What was seen that should not have been

While learning the formats: the first four P-Trak values of the D1 log, one P-Trak value of D2 test 03 (all in the
opening seconds of the logs), and NASA's one-paragraph note on D1 (readings low on both
instruments). D1 is not in the case files.

## 8. Files

`tools/wsp/ext/daft_cases.py` (`python -P tools/wsp/ext/daft_cases.py build`; -P, not -I, because xlrd is in the
user site-packages); `runs/dev/data_all/work/transfer_daft/cases_held_inputs.jsonl` (53),
`cases_held_outcomes.jsonl` (53; never printed), `baseline_smoke_number_per_mass.json`.
