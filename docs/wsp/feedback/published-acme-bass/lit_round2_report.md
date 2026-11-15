# Agent report (Opus 5.5 medium, 15 Nov 2026, extracted from transcript): literature round 2 for ACME and BASS

I found three new published comparators worth building on. Only one has per-test numbers we can match by test id today: Frolov et al. 2023 for Flame Design. The second can be rebuilt now from published constants but predicts *whether*, not *when*. The third, a quench predictor for the BASS fabric, is paywalled and I could only read its abstract and introduction.

The list is in `docs\wsp\research\lit_round2_acme_bass.md`. I committed nothing, opened nothing under `holdout/`, the evaluator vaults or the split files, and touched no IGNIS weight.

**Before you use item 1:** Frolov's table names flames from days 19123, 19200 and 19206, which the earlier Irace report flagged as possible Flame Design test days. The table says, for each flame, whether it went out and at what radius. Use it only post hoc, like the test-day comparison that is already spent, never to tune anything.

## Top three

1. **Frolov, Ivanov, Frolov, Semenov, *Fire* 6 (2023) 285** (open access). Verdict: **per-test numbers matchable.**
   - **What it covers:** our test ids for 24 normal and 13 inverse ethylene flames, with conditions, measured and modelled extinction radius, and a modelled extinction temperature of 1200 K. I transcribed both tables into the doc.
   - **Extinction time:** their transient 1-D model computes the burner heating itself rather than using the measured burner temperature, so it is a real forecast from test inputs. From their figures I read, per flame, the 10 s window (up to 50 s) in which the model's flame goes out.
     - 19123L1, 19206G1 and 19206A5 go out between 10 and 20 s.
     - Their two full time histories show the model can be right or late: 19189K1 goes out at about 41 s against about 41 s observed, but 22018G3 goes out at about 28.5 s against an observed end near 17.7 s.
   - **Closed-form fits:** they publish formulas for flame radius at 10 s and at extinction as a function of fuel flow and mixture fraction, for normal and inverse flames.
   - **Caveat:** their companion *Mathematics* 2023 paper lists different fuel flows for some of the same ids (19206L6, 19171D4, 19189K1). Check against our condition rows before matching.

2. **Snegirev, *Combust. Flame* 248 (2023) 112499** (open UGent copy, read in full). Verdict: **rebuildable comparator for *whether*.**
   - It gives a closed-form radius just before extinction (his Eq. 20) and a steady-burning test: the flame goes out if its no-loss steady radius exceeds that critical radius.
   - Every constant is published: critical flame temperature 1100 K for ethylene, 1200 K for propane and 1300 K for methane, plus c_p, k, absorption coefficient and heats of combustion.
   - It was checked against 19 self-extinguishing Flame Design flames, but only in a plot without ids.
   - It is a second published *whether* criterion built on different physics from Irace's R, and its fuel-specific temperatures carry to CFI-G propane and s-Flame methane.
   - It gives no time. The burner radius for the spherical burner isn't stated in what I read; 3.2 mm would be our assumption.
   - Nayagam, Dietrich and Williams 2019 is the uncalibrated special case of this formula (constant about 0.75).

3. **Thomsen, Fernandez-Pello, Urban, Olson, Ferkul, *PCI* 42 (2026) 106538** (paywalled). Verdict: **rebuildable once we have the full text.**
   - It is an analytical oxygen limit for concurrent spread on a thin solid, as a function of flow speed and pressure: a heat-transfer (quench) branch plus a chemical-kinetics (blow-off) branch.
   - It is correlated with the available limiting-oxygen data for SIBAL fabric, which is the BASS-II fabric. Its reference list includes Zhao et al. 2017, the BASS-II fabric paper.
   - It would be the first published closed-form quench predictor for the BASS fabric. Our current quench boundary comes from Kimwipes model values.

## Runners-up, all reference only

- **Xi, Torero, Guibaud, *CNF* 2022 (open):** the flame-spread sibling of the Xi droplet comparator. It fits two coefficients to the early pyrolysis-length record, then forecasts. It needs CFD and was tested only on one Saffire-IV PMMA slab, which isn't in our data.
- **Xia, Rutgers MS thesis 2015 (open):** pre-flight s-Flame extinction times, for example 4.96, 3.73, 7.63 and 1.96 s for four H₂/CH₄/N₂ blends at 5 cc/s. It used a different burner and flow units from the ISS hardware. It predicts earlier extinction at higher flow, which is the opposite of what `phys_sflame_f3.md` reports for the 44 % H₂ bottle.
- **Han et al., *PCI* 42 (2026) (open):** Saffire spread-rate power laws in pressure and oxygen, with exponents only. It gives model and measured spread rate for Saffire IV and VI.
- **Thomsen et al., *PCI* 37 (open):** a mixed-convection scaling with no fitted constants.
- **Waddell et al., *PCI* 40 (2024), the paper you asked about:** abstract only; the open manuscript link returned 404. Its three inverse flames are Waddell thesis Table 4.1, which we have already matched. It has no time predictor.
- **Waddell et al., *CTM* 2024:** closed, kinetics only.

## Not found or blocked

- No published *when* predictor for CFI-G alkanes beyond Kim 2023.
- No s-Flame paper with ISS results (the ACME publication list of August 2023 has none).
- No machine-learning or data-driven forecaster applied to ACME or BASS ISS data.
- Olson et al., ICES 2019 (the BASS-II flammability map) and the Thomsen ICES papers sit behind the Texas Tech repository's bot wall. The NASA BASS-II report we already have covers most of the map.

Downloads are in `D:\Caches\Temp\claude\c2f4976cab650b4b\199e808b-35cd-44e2-918a-d476aca69199\scratchpad\papers\round2\`.