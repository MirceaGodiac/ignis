# FLEX-2 (PSI-68): recovering the ambient of test points without a gas reading

> Written on 15 November 2026 by the delegated data agent (claude-opus-5-5), owner of the FLEX-2
> catalogue. Conditions only. Read: the text tier of `runs/dev/data_all/raw/PSI-68/` (timelines, chamber
> readings up to each test point's igniter-on stamp, gas-reading `Mixture` files, settings logs, the
> event-log images of the gas reading points, the SRD, the DoD, the ISA metadata), the experiment table,
> and the catalogue. **No droplet series, burn rate, image or outcome was looked at.** Nothing was
> scored. Paths under `runs/dev/` are local working folders, not in git.

## The answer

117 of the 596 catalogued test points have no gas reading on their day, and the forward refuses them.
A defensible ambient was recovered for **26** of them; **91 stay unknown**.

Of the 40 tier-B tests of batch 2 (n-decane 26, n-octane 14), **2 are recovered** (n-decane A278D08 and
A278D09, 2012-10-05, O2 0.249 in N2) and **38 stay unknown**:
- the 0.5 atm n-decane days 2012-07-02 and 07-04;
- n-decane 2012-07-09 and 09-10;
- all 14 n-octane points of November and December 2013.

The text tier holds no source that fixes their atmosphere:
- there is no per-test plan;
- there is no fill or vent log;
- their nearest gas readings disagree, or lie months away.

## Sources looked for, and what each gives

| Source | What it gives | Usable |
|---|---|---|
| `<R>_Mixture.txt` of the gas reading points (126 points, 124 files with a composition) | O2, He, CO2, N2, Xe, H2O and fuel vapours at a reading, usually one per day at the start of a session | yes: the measured ambient |
| Planned atmosphere: experiment table (8 rows), SRD (section 4.2, table 2), DoD, ISA metadata | the envelope only: O2 0.17 / 0.21 / 0.25 / 0.30, N2 or He diluent, 0.5–3 atm; the SRD says a final test matrix "with individual test point specificity" would be added as an appendix, but it is not in the archive | no per-test value: **no test can be given the class "planned in the matrix"** |
| Chamber fill and vent operations | none written down. The event-log `.docx` of 80 points are screenshots of the timeline header (igniter, burn-to-completion, sequence stamps); comments are image-downlink notes; settings logs carry no gas operation | no |
| Chamber pressure and temperature (`<T>_Chamber_Readings.txt`, read up to igniter-on; gas and calibration points whole) | a snapshot at each point; pressure levels of 7.5, 15, 18.8, 30 and 45 psia | detects a change of pressure level, **not** a refill to the same level |
| Gas readings on neighbouring days | the composition of the session before and after | yes, when both sides agree (below) |
| Publications listed in the ISA metadata | one title speaks of n-heptane, n-octane and n-decane droplets "in the standard atmosphere" | a lead (air at 1 atm for part of 2012–2013), not a per-test value; not used |

One gas reading point has no composition file: `A254R01` (2012-09-10, a tier-B day). Its timeline,
chamber and settings files exist, its `Mixture.txt` is not in NASA's listing.

## Method and provenance classes

`tools/wsp/ext/flex2_chamber_history.py` builds the chamber history of all 780 rack points
(`runs/dev/data_all/work/atlas/flex2_chamber_history.jsonl`). `tools/wsp/ext/flex2_ambient.py` then
assigns a class:

| Class | Rule | Value |
|---|---|---|
| `measured_same_day` | a plausible gas reading (gases summing to 0.97–1.03, O2 ≥ 0.05) on the test's day; the existing `ambient` field, never changed | the reading |
| `bracketed` (carried from neighbouring days) | no reading on the day; the nearest plausible readings before and after, each at most 45 days away, agree (\|ΔO2\| ≤ 0.01, \|ΔHe\| ≤ 0.05); the test's pressure level matches the readings' days or no level change lies between | the mean of the two readings |
| `unknown` | otherwise | none; the readings before and after are kept as candidates |

**A rejected rule: carrying the nearest reading from one side** (within 3 days, same pressure level, no
level change seen between). On the measured days it is wrong 4 times in 9: 2014-04-11/14 swapped air for
O2/He at the same pressure, and 2014-12-22/24 changed 29 % to 24 % O2. A refill to the same pressure is
invisible in the snapshots, so "no fill between" cannot be established from the text tier. The rule's
value is kept in `ambient_recovered_evidence.adjacent_candidate` for information and is not assigned.

## The method's error, on the days that DO have a measurement (conditions only)

Each of the 86 reading days with a plausible reading was left out and recovered from the others by the
same rule (`runs/dev/data_all/work/atlas/flex2_ambient_method_error.json`):

| Rule | Days it applied to | Wrong (\|ΔO2\| > 0.01 or \|ΔHe\| > 0.05) | O2 error when right | Largest error |
|---|---|---|---|---|
| `bracketed` | 41 | 3 (7 %) | ≤ 0.0044 (median 0.0008) | O2 0.096 (2016-03-31), He ≤ 0.002 |
| one-side carry (rejected) | 9 | 4 (44 %) | ≤ 0.0037 | He 0.75, O2 0.053 |
| neither (stays unknown) | 36 | | | |

The three `bracketed` failures:
- **2016-03-31.** Its neighbours 2016-03-23 (19.7 % O2) and 2016-04-18 (20.7 %) agree, so the rule gives
  0.20. The day itself read 29.8 %: a session at another oxygen level between two air sessions.
- **2016-04-27 and 2016-05-11.** Two days whose readings are byte-identical (A118R01 and A132R01–R03
  carry the same `Mixture` row, 16.82 % O2). One may be a copied file, so these two "failures" may be a
  defect of the readings rather than of the rule.

The defensible uncertainty of a `bracketed` value is therefore:
- **O2 ± 0.005 and He ± 0.002 when right;**
- **a 7 % chance (3 in 41) of being wrong outright**, with O2 errors of 0.04–0.10 when it is.

Pressure is not recovered: every test point has its own chamber pressure before ignition (catalogue
field `pressure_pa`, 573 of 596). The 23 without one have no chamber file.

## Counts by class and fuel (all 596 catalogued test points)

| Fuel | measured_same_day | bracketed | unknown |
|---|---|---|---|
| n-decane | 145 | 3 | 42 |
| n-octane | 40 | 0 | 19 |
| n-heptane | 10 | 0 | 8 |
| iso-octane | 2 | 0 | 8 |
| ethanol | 16 | 0 | 1 |
| iso-octane \| n-octane (fuel unresolved) | 16 | 8 | 2 |
| 1-propanol / glycerol 95/5 | 31 | 7 | 10 |
| 1-propanol / glycerol 80/20 | 34 | 0 | 0 |
| n-heptane / iso-octane 50/50 | 25 | 5 | 0 |
| n-heptane / toluene 50/50 | 32 | 1 | 0 |
| n-decane / propylbenzene (three blends) | 76 | 2 | 0 |
| toluene, propylbenzene | 52 | 0 | 0 |
| conflicting fuel records | 0 | 0 | 1 |
| **total** | **479** | **26** | **91** |

Among the single free droplets of a pure fuel a candidate covers, the ambient is now
`measured_same_day` 125, `bracketed` 3 and `unknown` 59.

**Recovered days (`bracketed`):**

| Day | Points | Readings used |
|---|---|---|
| 2014-08-04 | 8 | 1 atm air; its own reading E216R01 is implausible (79 % CO2) |
| 2016-02-24 | 6 | |
| 2014-07-02 | 5 | |
| 2012-10-05 | 2 | 25 % O2 in N2 |
| 2012-05-01 | 2 | |
| 2012-11-20 | 1 | |
| 2016-02-29 | 1 | |
| 2014-09-05 | 1 | |

**Unknown, with the nearest readings (all in `ambient_candidates`):**
- **2012-07-02, 07-04 (0.5 atm), 07-09 and 08-28.** Air 21 % on 06-07, then 17 % O2 in N2 on 08-08 and
  08-27.
  - 08-28 is one day after the 08-27 reading at the same pressure. That is the one-side case the
    measured days reject, so it stays unknown.
  - The 0.5 atm days could be a partial vent of the air session (composition kept) or a new fill;
    nothing in the text tier tells which.
- **2012-09-10.** 17 % on 08-27, 21 % on 09-20. Its own reading `A254R01` has no composition file.
- **2013-03-28; 2013-11-19, 11-25 (1 atm); 2013-12-04, 12-16 (3 atm).** The nearest readings are air on
  2013-03-14 and O2/He 25/75 on 2014-02-10, 83 to 265 days away.
- **2014-06-12, 2014-12-03, 2015-01-05, 2015-03-30, 2016-02-08 and 5 undated points.** Neighbours
  disagree or are missing.

## The catalogue fields

New fields in `runs/dev/data_all/work/atlas/flex2_tests.jsonl`, written by `flex2_ambient.py` after
`flex2_catalogue.py` and `flex2_chamber_history.py`. The measured `ambient` and `ambient_how` are never
touched.
- `ambient_recovered_class`: `measured_same_day`, `bracketed` or `unknown`.
- `ambient_recovered`: {O2, He, CO2, N2}, or null.
- `ambient_recovered_uncertainty`:
  - `p_wrong` (0.073);
  - `o2_abs_if_right` (0.0044);
  - `o2_abs_max`, `he_abs_max`;
  - the number of measured days the figures come from.
- `ambient_recovered_evidence`: the readings before and after (point, day, composition, pressure level,
  gap), the test's pressure level, and the rejected adjacent candidate.
- `ambient_candidates` (unknown only): the readings before and after.

Re-running `flex2_catalogue.py` rewrites the catalogue without these fields. Run `flex2_ambient.py`
again after it.

## How the case builder passes it to the forward

The forward takes the ambient as conditions (`x_o2`, `x_co2`, `x_he`, `pressure`). It refuses an
unresolved or missing condition, so a recovered ambient has to be handed over as a resolved condition,
flagged. `tools/wsp/ext/flex2_cases.py` now does this:
- **Measured series:** a series that carries its measured ambient is written as before, to
  `cases_<fuel>.jsonl`, with the row field `ambient_class: measured_same_day`. Verified identical to the
  cases scored on 15 November apart from that field.
- **Bracketed series:** a series without `x_o2` / `x_co2` / `x_he` takes the catalogue's
  `ambient_recovered` only if its class is `bracketed`.
  - Each condition carries evidence `ignis_derived` and the flags `ambient_recovered` and
    `ambient_class:bracketed`.
  - Its note names the two readings and the uncertainty.
  - The row carries `ambient_class: bracketed`.
  - The case goes to its own file, `cases_<fuel>_ambient_bracketed.jsonl`.
- **Unknown series:** a usable series whose ambient is `unknown` is not written, and is counted as
  skipped. The forward would refuse it.
- **Scoring apart:** a score runs the `_ambient_bracketed` file as its own arm, so recovered-ambient
  cases are reported apart and never pooled silently with measured ones. A pooled figure, if wanted, is
  stated beside both.
- **No guessed values:** an `unknown` ambient is never filled with an envelope value or a guess. To score
  such series, a registration would have to name a declared surrogate atmosphere (for example air for
  the 1 atm 2012 and 2013 days), as section 11.1 did for fuel. That is a protocol decision, not a
  catalogue value.

## Not done

- The `Mixture` files that are byte-identical across days (A118R01 and A132R01–03; the 18 readings of
  A158) are taken as they are; whether some are copies is not settled.
- `A254R01` (2012-09-10) may have a composition elsewhere; NASA's listing has none.
- The publication lead ("standard atmosphere") was not followed to the papers.
