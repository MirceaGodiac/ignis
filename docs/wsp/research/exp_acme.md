# ACME / CFI-G (Cool Flames Investigation with Gases) — dossier for flame-duration and termination-mode forecasting

Main source keys:
- **FR** = ACME/CFI-G Final Report (Stocker, Sunderland, Axelbaum, Williams), 21 Nov 2022, PSI-159 Reports file, 88 pp.
- **EIR** = "ACME Engineering Implementation", 19 May 2022, PSI-159 Engineering Documents file.
- **K23** = Kim et al., Proc. Combust. Inst. 39 (2023) 1647–1656.
- **KT** = M. Kim, M.S. thesis, Univ. of Maryland, 2022.
- **J21** = Stocker, JASMAC-33 paper, NTRS 20210021781.
- **PSI** = PSI-159 metadata and files.

---

## A. Apparatus

**Combustion Integrated Rack (CIR) chamber**
- Nominal volume is 100 L, rated to 300 kPa, with a ring of 8 sapphire windows [EIR p.13, E-FIELD paper excerpt; J21 p.3].
- Free volume with the ACME insert installed:
  - "approximately 83 liters depending on the installed hardware" [J21 p.5; FR p.15];
  - "83 L" [K23 §2; KT §2.1];
  - older design statements give "about 90 liters" [EIR p.15; FR p.15]. The 83 L figure is the corrected value.
- All tests are constant volume, not constant pressure [J21 p.5; FR p.15].
- CFI-G nominal chamber pressures, from FR Tables 1–2:
  - 7.35, 13.7, 14.7, 29.4 and 44.1 psia, i.e. ≈0.5, 0.94, 1.0, 2.0 and 3.0 bar;
  - the PSI tables list "approx p (bar)" values 0.50665, 0.9443956, 1.0133, 2.0266 and 3.0399 for normal flames, and 0.495–3.31 bar for inverse flames [PSI-agg].
- K23 Table 1 gives p = 0.51–2.03 bar (normal ethane and n-butane), 0.94–3.04 bar (normal propane) and 0.46–3.04 bar (inverse).
- K23 also states "The variations in ambient pressure, temperature, and concentrations during these tests were negligible". KT §4.1 says the same.
- The CIR includes a Passive Rack Isolation System, but it was not used for ACME [FR App. A].

**ACME chamber insert**
- About 0.4 m in diameter. Burners are mounted on the chamber axis, one at a time [J21 p.4; FR p.7].
- It carries a retracting, resistively heated igniter with a stepper-motor drive and a crew-exchangeable tip and arm [J21 p.4; FR p.7].

**Spherical porous burner**
- Diameter 6.35 mm. Only this size was flown; the 9.5 and 12.7 mm sizes were waived (ACME-WAI-012). 2.2 s drop tests showed 6.35 mm was "as good or better" [EIR p.43; FR p.8]. K23 rounds this to 6.4 mm.
- Material: sintered 316 stainless steel, 10 µm media grade, porosity ≈41% (K23 says ≈40%), density 4.6 g/cm³ [EIR p.43; KT §2.1; K23 §2].
- The sphere is sinter-bonded to a stainless supply tube. A close-fitting support tube of 1.8 mm OD was added because the tubing lost its temper [EIR p.43–44].
- Deviation ACME-DEV-031: the tube OD is 28.8% of the sphere diameter instead of 25% [EIR deviations list].
- Burner serial number 1003 was used for all CFI-G tests [EIR p.44].
- Burner thermocouple: a fine sheathed type-K thermocouple embedded up to ≈1 mm in the lower hemisphere (hole 0.76–1.0 mm deep) [EIR p.43].
- Known flow asymmetry: the sintered joint between sphere and tube is more porous than the sphere itself. At high flow the flame shifts toward the supply tube, becomes ellipsoidal or parabolic, and locally quenches near the tube [FR p.16; EIR p.44].
- The supply and support tube is a cold thermal mass that quenches part of the flame [K23 §1].

**Burner temperature limit (tests are cut off when the burner overheats)** — the sources disagree:
- "Tests were terminated if the burner reached 723 K to prevent overheating" (723 K = 450 °C) [K23 §2; KT §2.1 and §4.1].
- "The test point automatically stops when the burner reaches 650 °C" (inverse-flame round, weekly summary caption) [FR p.62–63].
- PSI peak burner temperatures [PSI-agg]:
  - normal-flame FT tests reach up to 532 °C;
  - inverse-flame FT tests reach up to 544 °C, with one at 651.5 °C;
  - SE (self-extinction) tests stay ≤488 °C (normal) and ≤330 °C (inverse).
- [INF] The trip value may have differed between rounds, or the 450 °C trip overshoots by thermocouple lag. Treat the limit as ≈450–650 °C.

**Igniter**
- Kanthal A-1 wire.
- CFI-G used a coil tip (0.32 mm wire). Other ACME work mostly used a 0.25 mm loop tip [EIR p.54; FR p.8].
- Ceramic insulator 4.75 mm in diameter; pin spacing 2 mm [EIR Fig. 57].
- Example script parameters from an inverse-round parameter sheet (PSI "test parameter notes" scan):
  - preheat 3 V for 1000 ms, then ignition 4 V for 3000 ms;
  - `igniterDelayBeforeRetractMs` = 1800;
  - start and end positions 2170 and 2290 counts;
  - insertion speed 80 counts/s, retract speed 500 counts/s.
- Low-energy direct-cool-ignition attempts in the normal round used 2–2.9 V for 10–20 s (PSI normal-table comments).
- Retraction from the burner's "free area" took 0.6 s instead of the required 0.5 s (ACME-DEV-050). Speed uncertainty was 2.54 mm/s (ACME-DEV-049) [EIR deviations].
- PMT signals were used operationally to detect ignition and automatically switch off and retract the igniter [EIR p.17].
- The coil tip burned out on one inverse propane day and was replaced 2 weeks later [FR p.15–16].

**Mass-flow controllers (MFCs)**
- Three insert MFCs: A1 oxidizer (or inert), A2 nitrogen, A3 fuel [EIR p.24].
- Available full scales (N₂ basis) [EIR Table 2]:
  - A1: 0.5 or 20 slpm;
  - A2: 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 2 or 5 slpm;
  - A3: 0.05, 0.1, 0.2, 0.5, 1 or 2 slpm.
- Maximum fuel flow is 2 slpm. Oxidizer flow up to 20 slpm; nitrogen up to 5 slpm [FR p.8].
- The CIR's own 0–30 slpm MFCs (fuel 0–2 slpm) are simply held fully open [EIR p.25].
- Uncertainty is ±5% [K23 §2]. Control is poor below about 10% of full scale [FR p.16].
- Nitrogen dilution is possible for only one stream at a time [EIR p.23].

**Gases**
- CFI-G fuel bottles: 100% C₂H₆, 30% C₂H₆/N₂, 100% C₃H₈, 30% C₃H₈/N₂, 100% n-C₄H₁₀, 50% n-C₄H₁₀/N₂ and 30% n-C₄H₁₀/N₂ [EIR Table 1].
- Oxidizer bottles: 21, 30, 40 and 85% O₂ in N₂.
- The chamber atmosphere is limited to ≤40% O₂. The 85% O₂ bottle is for the inverse-flame burner flow [EIR p.23].
- n-Pentane was dropped (vapour pressure too low) and replaced by ethane [FR p.6, p.17].
- For inverse propane and n-butane flames, "the fuel supply was two-phase" [K23 §2].
- Nitrogen comes from the ISS [EIR p.22].
- CIR bottle sizes are 1, 2.25 and 3.8 L; fill pressure is limited as a hazard control [FR p.8].

**Chamber preparation**
- The chamber is filled with its target atmosphere (O₂/N₂ for normal flames; fuel/N₂ for inverse flames) before burner flow starts [KT §2.1].
- Example test sheet: "Vent before? yes, to 0.2 psia", then fill [PSI test sheet scan].
- Multiple tests run in the same fill. Recirculation through the filter or scrubber and venting happened only at the end of a day or between atmospheres, and only on normal-flame days [FR p.19].
- The gas chromatograph was "only sometimes used" for ACME. Chamber contents were otherwise computed by assuming the fuel was either fully oxidized or not oxidized at all [FR p.14].
- The PSI tables give "XO2 before/after test" (normal) and "Xf before/after test" (inverse) from these calculations [PSI].

**Consumable and safety limits**
- Two redundant CIR valve safety timers limit the total flow time of the limiting reactant (fuel, or oxidizer for inverse flames).
- They are set to the maximum of 999 s and "can only be reset by venting the combustion chamber". This acts as a per-fill flow-time budget [FR p.20].
- An example sheet shows the timer decreasing by ≈40–50 s per test [PSI flow-trace notes].
- Normal-table bottle comments mention "timer reset" planning [PSI].
- Daily data limit: 127,000 MB, so image data are sometimes discarded [FR p.20].
- No testing during thruster firings or dockings [FR p.30].

---

## B. Test procedure and timeline

**Who runs the tests**
- The ISS crew configures the hardware (bottles, MFCs, igniter tips) but does not run tests [FR p.18, p.30; J21 p.6].
- Tests are commanded from the ground, from the Glenn ISS Payload Operations Center (GIPOC/TSC), using uploaded scripts and parameter files.
- "No CFI-G tests were conducted in a `manual' mode … with the mundane exception of `manually' halting the test after the flame has extinguished or perhaps if a temperature has reached a critical value" [FR p.30, p.37].
- KT §2.3 adds that the intensified-camera gain was raised manually by the operator during tests to protect the intensifier. This partly contradicts FR; the parameter sheets also show scripted `intensifierIgnitionGain` 20000 and `intensifierTestGain` 60000.
- About 12 tests per test day [FR p.19]; 280 hot flames over 23 test days [FR p.43].

**Script phases.** These are the time-stamp names in each `_TP.txt` file [PSI]:
1. Script start.
2. Script paused #1 / resumed #1 (operator go). The pause typically lasts 10 s to ~1.5 min.
3. Pre-ignition data, then pre-ignition images.
4. Preheat igniter, MFCs on for priming. Never used in CFI-G.
5. **Ignition and retraction.**
6. Flame stabilization #1.
7. Dilution of fuel and inert. Never used.
8. Flame stabilization #2.
9. Ramp coflow / Flame stabilization #3 / Ramp to final velocity. Never used.
10. **Constant flow.**
11. **MFCs off.**
12. Post-flame images, then post-flame data.
13. Script complete.
14. **Extinction detected** (logged when automatic detection fires).

Aggregate timing over 305 TP files [PSI-agg]:
- Flow starts shortly before ignition. In a sample MFC file the fuel setpoint stepped up ≈1.5 s before the "Ignition and retraction" stamp.
- Ignition → Flame stabilization #1: 2 s (most tests).
- Flame stabilization #1 → Constant flow: 5 s (most tests).
- So "Constant flow" begins ≈7 s after the ignition stamp in most tests (102/183 normal, 71/122 inverse).
- This matches parameter-sheet values `stabilizationTime1Sec` = 0 and `stabilizationTime2Sec` = 5.
- Pre-ignition data → ignition: ≈20–45 s.
- MFCs off → post-flame data: 7 s.

**Flow schedule within a test**
- No ramps; at most one ignition-phase step [PSI-agg over 336 `_mfcSetpoints.csv` files].
- 208 of 336 tests use a single constant setpoint triple.
- About 128 tests have 2–3 setpoint levels. These are either a short ignition-phase level (typically switching ≈4.5 s after flow start, consistent with `oxidizer/nitrogenIgnitionFlowSlpm` vs `TestFlow1Slpm` and `...IgnitionDeltaMs`) or staggered MFC start and stop.
- **No test had a sustained setpoint change more than 8 s into the flow other than shutdown.**
- The "Ramp" stamps are always blank. Parameter sheets show `testFlow2/3` = 0, all ramp times 0, and `postExtinctionFlowChange*` = 0.
- [INF] Treat each test as one constant flow after a ≈5–7 s ignition transient.

**Three automatic ways a test ends**, as listed on the science test sheet ("Out: ext. detect / time out / temp. limit") [PSI test-sheet scan]:

1. **Extinction detected.**
   - PMT or radiometer measurements "can similarly be used to detect extinction and automatically end the gas flow and test" [EIR p.17].
   - After detection the MFCs close after `postExtinctionDelaySec` = 3 s [parameter sheet]. Extinction stamp → MFCs off is exactly 3 s in 148/158 normal and 79/79 inverse cases [PSI-agg].
   - **Detection lags the real extinction considerably** [PSI-agg]. Comparing the tabulated burn time (time of hot-flame extinction) with the TP "Extinction detected" stamp gives:
     - normal flames: median ≈12 s, typical 8–15 s, range 4–38 s;
     - inverse flames: median ≈19–20 s, range 13–42 s.
   - So the flow typically continues 10–45 s after a real self-extinction.
   - This is consistent with "When a flame extinguished radiatively, the flow was continued for at least 10 s to allow a possible transition to a cool diffusion flame" [K23 §4; KT §4.1].
   - Detection can be disabled. An inverse-round day had "Extinction detection effectively turned off so tests were ended `manually'" [PSI flow-trace notes].

2. **Flow time-out.**
   - Parameter `flowTimeoutSec`; example value 45 with `constantFlowTime1Sec` = 15 [parameter sheet].
   - Inverse tests without an extinction stamp cluster at ignition → MFCs off = **67 s** (19 of 37) [PSI-agg].
   - The inverse PSI table burn times for FT tests cluster at **66 s** (29 of 58 FT entries) [PSI-agg].
   - [INF] 67 s ≈ 2 s + 5 s stabilization + 15 s constant flow + 45 s time-out, so the time-out appears to count from the constant-flow start. Other time-out lengths were used too: inverse clusters at 97 s, and normal FT values spread 13–276 s with clusters near 127 s and 277 s [PSI-agg]. Not documented further.
   - FR p.44: the non-RE flames "were extinguished when their flow was terminated, either because the preset flow time was out or the burner overheated".

3. **Burner temperature limit** (see A for the 723 K / 650 °C discrepancy).
   - PSI comments include "Hit the burner temperature limit", "hit T limit", "Terminated by burner temp hit the limit", and "Burner T limit, most images lost".
   - 15 of 31 normal FT and 15 of 57 inverse FT tests had peak burner T ≥ 440 °C [PSI-agg].

4. **Manual.** Operators can halt a test after extinction or at a critical temperature [FR p.30].
   - In the first-round cool-flame tests, flows were cut while a cool flame was still burning because nobody knew it was there [FR p.15; K23 §4].
   - Also seen: IPSU (image recorder) aborts before gas flow, and "Flow timed out" or "Timed out" comments [PSI].

**Typical and maximum durations**
- Design intent: "tests will typically be 0.5–2 minutes in duration" [FR p.15]; "typically two minutes or less" [FR p.20]; "typically only a few minutes" [J21 p.5].
- CFI-G hot-flame burn times in the PSI tables [PSI-agg]:
  - normal SE: 1–304 s;
  - normal FT: 13–276 s;
  - inverse SE: 0.5–55 s;
  - inverse FT: 12–152 s.
- "Some lasting several minutes" (first days) [FR p.34].
- Flame Design (same burner, ethylene): burn times 28–336 s [Irace 2021 abstract].

**Repeat tests in one fill**
- Many tests share one fill, so the atmosphere is vitiated progressively.
- FR Tables 1–2 footnote: the "Atmosphere … is only nominal where the pressure increases and O₂% drops with vitiation".
- Per-test computed change [PSI-agg]: median ΔX_O2 ≈ 0.0006, maximum 0.023 (normal); median ΔX_f ≈ 0.0001, maximum 0.010 (inverse).
- Ambient changes during individual tests are called negligible [K23 §2].

**Campaign timeline** [FR §1, App. D]:
- Round 1 (normal flames): 29 Mar – 23 Jun 2021, 11 test days.
- Round 2 (inverse flames): 15 Jul – 4 Oct 2021, 12 test days, plus one added normal-flame day on 18 Oct 2021 to recreate cool-flame conditions with the near thermocouple rake.
- Increments 64–66 hosted 33, 225 and 22 hot flames respectively [FR p.38].

---

## C. Diagnostics and data products

**Cameras**
- **Analog colour operations camera**
  - Inside the chamber; 30 fps; downlinked live; recorded only on the ground [FR p.9; K23].
  - "This was not sensitive enough to image the cool flames" [K23 §2].
  - Gain was sometimes raised for dim flames [FR p.9].
- **ACME DataCam**
  - 12-bit colour, 1360×1024, up to 30 fps.
  - Often run at long exposures (≈1 fps) because the flames are dim [FR p.9].
  - Filter wheel options: clear (fused silica), BG7 (too thick, so the red is over-attenuated), CH* 430 nm, and a formaldehyde bandpass [FR p.9; EIR p.17].
  - Example pre-ignition exposure 1 s, test exposure 0.1 s [parameter sheet].
- **CFI intensified camera**
  - 1392×1040, 23 fps.
  - Formaldehyde bandpass 390–490 nm (CH₂O*) for CFI-G [K23 §2; KT §2.2; FR p.29].
  - Field of view partly obstructed by the TFP arms [FR p.11].
  - Concern about intensifier damage from the igniter or hot-flame light when gain is high [FR p.15].
- **High Bit-depth Multispectral (HiBMs) camera**
  - Red filter, with the CIR backlight; 15 fps archived.
  - Rarely used in CFI-G. Used in round 2 to check for soot on the windows [FR p.10, p.16].
  - Its image recorder (IPSU1) failed on the first test day [FR p.16].
- **LLL-UV OH* camera**: s-Flame only [FR p.11].

**Radiometers**
- CFI-G had its own array, with high- and low-gain channels per sensor [FR p.12].
- K23 / KT description:
  - a Schmidt-Boelter wide-angle thermopile at r = 154 mm, sensitive to 0.6–50 µm;
  - two others at 0.2–17.5 µm confirmed its readings;
  - mounted at +60, +30 and −10 mm from the assembly centerline.
- A fixed, faster offset radiometer sits downstream at 30° to the axis, with a nominal 0.2–11 µm range [FR p.12].
- Radiometers were the primary way cool flames were detected: after hot-flame radiative extinction the radiant flux falls to a nonzero plateau instead of decaying to zero [FR p.44; K23 Fig. 5].

**Photomultiplier tubes (PMTs)**
- Three wide-angle PMTs, 230–700 nm: one with a 310 nm (OH*) filter, one with a 430 nm (CH*) filter, one broadband with a neutral-density filter [FR p.13].
- K23 / KT list the bands as 230–700 nm ("CH₂O*"), 310 nm and 430 nm.
- Gains are adjustable; saturation occurred when set too high [FR p.13].
- "Only marginally able to detect the cool flames" [K23 §2].
- PMT-2 (230–700 nm) "analyzed to determine flame durations" [FR p.33, p.44].
- A normal-table comment reads "Extinction detected. Fuel flow stopped in 6 sec after PMT dropped to 0" [PSI].
- [INF] The tabulated burn time is the PMT-based hot-flame duration.

**Thermocouples**
- Burner type-K thermocouple (see A).
- Two far-field type-K thermocouples at 200 mm, upstream; one is bent [EIR p.63].
- Rake thermocouples:
  - far/short rake at 45, 60, 100 and 100 mm; near/long rake at 13, 25, 50 and 100 mm;
  - the two innermost are type B, the outer ones type K; 0.2 mm wire [EIR p.63].
- The near rake (type-B Pt/Rh at 13 mm) was used only on the added final day [FR p.17; K23 §4]. Its platinum catalyzed hot reignition from cool flames when it reached 720–820 K [K23 §4].
- For most of round 2, the nearest thermocouple tip of the short rake was bent ≈90° [FR p.13, p.17].

**Thin-filament pyrometry (TFP)**
- 14 µm SiC fibres, 5 mm apart. Only 3 of 5 fibres were left in CFI-G [FR p.10, p.16].
- Used in only some tests: 18 normal and 12 inverse [PSI-agg].
- Detection limit 950 K [K23 §4].

**Other measurements**
- Chamber pressure transducer [K23 §2].
- Burner internal pressure [FR p.7].
- Acceleration is measured by the Space Acceleration Measurement System (SAMS) but is not archived in PSI [FR p.14].

**Non-imaging data rate**: 125 samples/s, written as a per-test CSV [FR p.33, p.43].

**PSI-159 per-test files** [PSI; columns read from sample files]
- `…_data.csv`
  - Line 1 is a header with the start time, GMT day, experiment number, test point, MFC gas codes and full scales, igniter, burner, thermocouple, radiometer, IPSU and fibre configuration codes, and PMT gains.
  - Line 2 holds the column names:
    - `Time` (clock hh:mm:ss:ms);
    - `MFCA1StaticMixerFlow`, `MFCA2BypassFlow`, `MFCA3FuelFlow`;
    - `IonCurrent`, `PTA1`, `PTA2`, `PTA3`, `EfieldVoltage`;
    - `PMT1`, `PMT2`, `PMT3`;
    - `TCA`, `TCB`, `TCC`, `TCD`, `TC200A`, `TC200B`, `TCBurner`;
    - `Radiometer3+60LoGain`, `Radiometer3+60HiGain`, `Radiometer3+30LoGain`, `Radiometer3+30HiGain`, `Radiometer3-10LoGain`, `Radiometer3-10HiGain`;
    - `Ch13`–`Ch16`, `RadiometerOffset`, `RadiometerOffsetCJ`, `TCE`, `Ch20`, `Ch21`, `TCCJ`, `EfieldSetpointOut`.
  - **Values are raw voltages**; calibrations are in the characterization reports (ACME-RPT-011/013, CIR-RPT-4600), which were not found online.
  - Sampling interval is ≈8 ms (125 Hz).
- `…_mfcSetpoints.csv`: the same header line, then `Time, MFCA1Setpoint(V), MFCA2Setpoint(V), MFCA3Setpoint(V)`, logged at each script event.
- `…_TP.txt`: script-phase wall-clock stamps (1 s resolution) as listed in B, plus an image archive table (images archived per camera: IPSU1/HiBMs, IPSU2, ACME DataCam, Intensified).
- `…_Comments.txt` / `…_ImagesComments.txt` / `…_ImageComments.txt`: downlink bookkeeping (missing or redownlinked image files). No science content in the samples checked.
- Other files: `…_ALL.zip`, `…_Converted.zip`, `…_ACME.zip`, `…_Images.zip`, `…_HiBMs.zip`, `ACME.log`; for some days, "as run" scripts, scanned test sheets, test-parameter notes, flow traces and daily test-point logs; and a "Chamber Calculator" xlsx covering 2021-03-09 to 2021-10-21 [PSI file list].

**PSI experimental tables** (CSV). Column names only:
- Normal flames: `Flame Type, Day, Fuel, Test, Cool flame?, XO2 before test, XO2 after test, approx p (bar), Xf, mHC (mg/s), TFP, Tad (K), Zst, Hot ignition, burn time (sec), Self-ext or flow term, Peak burner T (C), Comments`.
- Inverse flames: `Flame Type, Day, Fuel, Test, Xf before test, Xf after test, approx p (bar), XO2, HC burnrate (mg/s), O2 flow rate (mg/s), TFP, Tad (K), Zst, Hot ignition, burn time (sec), Self-ext or flow term, Peak burner Temp (C), Peak burner temp (K), Color, Comments`.

**Time zero**
- Publications use "time after ignition, t" [K23 Fig. 4 and Fig. 5, where the axes start at −5 s; KT Fig. 4.1: "ignited (t = 0 s)"].
- The data CSV has no relative time; t = 0 must come from the TP "Ignition and retraction" stamp (1 s resolution).
- [INF from PSI-agg] For FT tests, (TP ignition → MFCs off) − table burn time is mostly +1–2 s. So the table burn time runs from roughly the ignition stamp, and the hot-flame end is taken as flame-out, slightly before the logged MFC-off second.
- Flow start precedes ignition by ≈1–2 s.

**How quantities are measured**
- Flame radius: circle fit to the contour of peak intensity on intensified images, ±2%. Hot-flame radii are taken just before oscillation onset [K23 §4].
- Cool-flame radius: half the distance between the two intensity peaks on images averaged over 20 frames [K23 §4].
- Hot-flame extinction: PMT-2 drop, plus the intensified camera [FR p.44].
- Cool-flame presence: a radiometer plateau after hot extinction, rising burner temperature, and intensified images with a 50× digital gain [FR p.44–45].

---

## D. Test matrix

- **Fuels**: ethane, propane, n-butane; generally diluted with N₂ [K23 Table 1].

- **Normal flames** (fuel and N₂ from the burner into an O₂/N₂ ambient) [K23 Table 1]:

| Fuel | X_O2 (ambient) | X_HC (burner) | p (bar) | ṁ_HC (mg/s) | RE | FT | Cool flames |
|---|---|---|---|---|---|---|---|
| Ethane | 0.18–0.40 | 0.25–1 | 0.51–2.03 | 0.50–2.36 | 30 | 10 | 0 |
| Propane | 0.21–0.40 | 0.05–1 | 0.94–3.04 | 0.13–2.89 | 34 | 6 | 0 |
| n-Butane | 0.21–0.40 | 0.02–0.5 | 0.51–2.03 | 0.10–3.22 | 46 | 17 | 15 |

  - For propane above 1 bar, X_O2 was limited to 0.21.
  - PSI normal table: ṁ_HC 0–3.215 mg/s; X_f values 0.018–1 (bottles at 0.3, 0.5 and 1, sometimes further N₂-diluted on orbit); X_O2 0.18–0.40 [PSI-agg].

- **Inverse flames** (O₂/N₂ from the burner into a fuel/N₂ ambient) [K23 Table 1]:

| Fuel | X_O2 (burner) | X_HC (ambient) | p (bar) | ṁ_O2 (mg/s) | RE | FT | Cool flames |
|---|---|---|---|---|---|---|---|
| Ethane | 0.12–0.85 | 0.06–0.30 | 0.46–3.04 | 0.93–9.08 | 25 | 22 | 0 |
| Propane | 0.18–0.85 | 0.08–0.43 | 0.51–3.04 | 1.36–9.07 | 24 | 13 | 0 |
| n-Butane | 0.21–0.85 | 0.06–0.36 | 0.51–3.04 | 1.34–8.95 | 30 | 23 | 0 |

  - The PSI inverse table also gives an "HC burnrate (mg/s)" of 0.25–2.5 (the stoichiometric fuel consumption implied by the O₂ flow) [PSI-agg].

- **Totals**:
  - 280 hot flames, including natural reignitions; 189 RE and 91 FT [K23 §4; FR p.44].
  - K23 counts: normal 110 RE + 33 FT; inverse 79 RE + 58 FT.
  - PSI table rows: normal 179 rows (SE 102, FT 31, "−" 46); inverse 154 rows (SE 81, FT 58, "−" 15) [PSI-agg].
  - Counts differ slightly between K23 and the PSI tables. K23 counts each reignited hot flame; the PSI tables have one row per test and include ignition failures.

- **Normal vs inverse differences**:
  - Inverse flames: soot forms outside the flame, burner temperatures stay low (<500 K) [K23 §4], burn times are shorter (SE ≤55 s), and many tests end on the time-out.
  - Normal flames: burner heating is stronger; there are long burns up to ~5 min.

- **Proposal vs actual scope**: ~50 flames and ~12 fills were proposed; 280 hot flames were achieved over 23 days [FR p.43].

---

## E. Physics of how these flames end

**1. Ignition and the early transient**
- A hot wire ignites the flame ≈1–2 s after flow start. An early premixed flash quickly becomes a diffusion flame at some standoff [Santa 2007 p.6].
- Failures:
  - normal flames: 47/179 PSI rows have "Hot ignition = No";
  - inverse flames: 15 "No" and 8 "flash" [PSI-agg].
- Many normal "No" rows were deliberate low-energy attempts to light a cool flame directly ("In 13 other tests, data is being analyzed to determine if a cool flame was ignited directly with a low-energy ignition") [FR p.53]. Igniter-voltage comments are in the PSI table.
- KT/K23 report no successful direct cool ignition; all 15 cool flames followed hot-flame radiative extinction [K23 §4].
- Comments such as "Lowest flow rate for ignition" and "Retracted too early" indicate a minimum-flow ignition limit [PSI].
- Flames lasting 1 s are common in the SE lists (10 normal, 12 inverse) and correspond to momentary or "flash" ignitions [PSI-agg].

**2. Growth**
- "All flames grow monotonically after ignition. Peak gas temperature generally decreases with time" [Irace 2021 abstract].
- Hot flames grow with a decreasing rate and approach a near-constant size [KT §4.4; K23 §4: "generally grew with time, with many reaching quasi-steady radii"].
- Diffusion time to steady state:
  - r²/D ≈ 30 s for r = 25 mm and D = 20 mm²/s;
  - "approximately 30 s of jitter-free (less than 10⁻⁵ g) microgravity is needed to approach quasi-steady state" [FR p.5, proposal text].
- Irace et al. (PCI 39, 2023) attribute growth to the diffusion-limited system approaching steady state, plus "radiation-induced growth": transport properties fall as radiation lowers the temperature [Irace 2023 summary via WUSTL].
- Quasi-steady radius model [K23 Eq. 8, Spalding-type]:
  - normal flames: r_hf = ṁ / (4π ρD ln(1/(1 − Z_st)));
  - inverse flames: r_hf = ṁ / (4π ρD ln(1/Z_st)) (Eq. 13).
  - The best fit is ρD = 0.0334 g/(m·s) at 300 K for hot and cool flames (R² = 0.92 of measured vs modeled radii) [K23 §4, Fig. 7].
  - So radius ∝ total burner mass flow ṁ, and grows as Z_st falls (more dilute oxidizer for normal flames).
- Size trends:
  - normal flames grow with flow rate and shrink with ambient O₂ [J21 p.8];
  - inverse flame size is set by the O₂ flow; extra N₂ at the same O₂ flow gives a similar size [FR p.61–62];
  - higher ambient fuel gives slightly smaller and brighter inverse flames [FR p.63].
- Pressure: flame radius decreases with increasing pressure in simulations [Waddell 2026 §3.1].
- Hot-flame radii just before oscillation in the n-butane cool-flame series: 12.4–17.7 mm at 0.75–1.5 mg/s [K23 Table 2; FR Table 3].

**3. Radiative (hot-flame) extinction** — the dominant end mode (189/280) [K23 §4]
- Mechanism: at constant flow the flame grows until radiative loss approaches heat release [Irace 2021 abstract].
- Santa et al. 2007 (2.2 s drop tower, transient flames):
  - extinction occurs when Q_R/Q_C ≈ 0.7;
  - extinction is "not as much driven by a rapid rise in radiative heat loss … as … by a reduction in heat release rate with time";
  - for large flames, radiation scales with flame surface area, not volume [Santa 2007 pp.7–10].
- **Critical extinction temperature**:
  - ≈1100 K, ethylene drop tests, model and TFP; it fell less than 50 K, to 1055 K, at 104 mg/s [Santa 2007 pp.8–9];
  - **1130 K** for all ISS ethylene normal and inverse flames at 1 bar [Irace 2023, PCI 39:1905];
  - simulations: hot flames radiatively extinguish at ≈1200 K below 4 atm, or turn into warm flames at ≥4 atm [Waddell 2026 §3];
  - compare 1500 ± 50 K for kinetic extinction [Santa 2007 p.9].
- **Critical reactant mass flux at extinction** (ethylene, 1 bar) [Irace 2023]:
  - 0.2 g/(m²·s) fuel-based for normal flames;
  - 0.68 g/(m²·s) O₂-based for inverse flames.
  - Irace et al. give a simple equation for flame size at extinction from the reactant mass flow alone (exact form not retrieved).
  - [INF] With flux ≈ ṁ/(4π r_ext²), this implies r_ext ≈ (ṁ / (4π · flux_crit))^{1/2}.
- **Extinction radius scalings**:
  - Nayagam, Dietrich & Williams 2019: r_ext ∝ (ṁ·Y_reactant / κ_P)^{1/3}, where κ_P is the Planck-mean absorption coefficient [abstract].
  - Santa 2007: fitted r_ext ∝ ṁ^0.44 (O₂ into 4% C₂H₄) and ṁ^0.36 (propane into 17% O₂) [p.8–9].
- **Extinction time vs flow** (drop-tower transient flames, Santa 2007):
  - rises with flow at low flow because burner heat loss lowers T early;
  - levels off above ≈10 mg/s to a flow-independent asymptote (≈1 s measured, 1.65 s computed for O₂ into 4% C₂H₄; ≈2 s for propane into 17% O₂).
  - These are drop-tower values. ISS flames take tens to hundreds of seconds.
- **ISS burn times**:
  - Flame Design (ethylene, 1 atm): 28–336 s [Irace 2021];
  - CFI-G normal SE: 1–304 s, median ≈35 s;
  - CFI-G inverse SE: 0.5–55 s, median ≈9 s [PSI-agg].
  - [INF] Inverse flames extinguish much sooner: compare the higher critical O₂ flux, short residence times [K23 §4] and the dim large flames [FR p.64].
- **Flame size and radiation trends**:
  - Very small flames have low radiative loss and approach steady state; large flames grow until radiative extinction [J21 p.8; Irace 2021].
  - "Radiative heat loss scales with flame size, and the larger flames are dimmer" [FR p.62]. Measured radiation from a large inverse flame was several times that of a small one [FR p.65].
  - Larger flames generally self-extinguish sooner. In BRE (flat burner), "larger flames were generally seen to self-extinguish within 1.5 minutes", and flames self-extinguished below ~25% O₂ [J21 p.7].
- **Dilution and composition**:
  - Inverse flames in 12% ethane burned longer than in 6% ethane [FR p.60].
  - An inverse flame in 14.0% butane oscillated and self-extinguished while one in 17.5% butane survived the test [FR p.65].
  - [INF] Higher T_ad and a richer ambient lengthen the hot-flame life. The PSI tables give T_ad and Z_st per test, which are natural model features.
- **Oscillations before extinction**:
  - Large flames enter "an unstable, oscillatory mode, in which they partially quench and reform. The oscillations grow in magnitude until the flame fully extinguishes" [J21 p.8; Irace 2021].
  - Oscillations are often initiated at the hole near the supply tube. The opening "often oscillates in size while growing until the flame fully extinguishes. In some tests the extinction occurs abruptly, but in others the oscillations can last for tens of seconds. Meanwhile, some flames are quasi-steady despite the opening" [FR p.62].
  - Escaping soot clusters add to heat loss and cause local quench and reform cycles [FR p.58].
  - Example (K23 Fig. 5a, KT Fig. 4.4a; n-butane 2 mg/s, 0.5 bar): oscillations from 18 s, extinction at 30 s.
- **Two extinction onset modes** [Santa 2007 p.5]:
  - base extinction near the tube;
  - hole extinction (a flame hole far from the tube, typical at high flow).

**4. Burner heating** — sets FT at low and moderate flow
- "Sufficiently small flames with low radiation losses exhibit quasi-steady behavior, but burner heating is significant, and the experimental duration is dictated by the temperature limit of the porous sphere" [Irace 2021 abstract].
- A smaller flame (lower flow) burns longer and heats the burner more [K23 §4].
- Hence the regime split [INF]:
  - low flow → long-lived, near-steady flame → burner-limit FT;
  - high flow → large flame → radiative SE;
  - intermediate flows may hit the time-out first.
- Burner heat loss also lowers early T and shortens time-to-extinction at low flow [Santa 2007 p.8].
- In normal tests, burner T at the end of the hot flame matters for cool flames: they formed only when the burner was hotter than 515 K [K23 §4].

**5. Kinetic (low-Damköhler) extinction**
- s-Flame (H₂/CH₄/C₂H₄ on the same burner): "Radiative extinction seems to occur at high flow rates and kinetic extinction at low flow rates … at low flow rates the flames are close to the burner where the velocities are highest" [J21 p.8].
- With He dilution, kinetic extinction is hard to discern [J21 p.8].
- For CFI-G no kinetic-extinction cases are reported explicitly (not found). [INF] Very short (≈1 s) SE events at the lowest flows may belong to this branch, or be failed ignitions.

**6. Hot-to-cool transition (normal n-butane only)**
- Conditions: 15 cool flames, all normal n-butane flames, with X_O2 ≈ 0.39–0.40, X_HC = 0.30, p = 2.03 bar, ṁ_HC = 0.75–1.5 mg/s and burner T > 515 K [K23 §4 and Table 2; FR p.44].
- None with ethane or propane, none below 2 bar, none below X_O2 ≈ 0.4, none in inverse flames.
- For inverse flames the reasons given are short residence times and low burner temperatures (<500 K) [K23 §4].
- Delay from hot extinction to cool-flame appearance: 0.7–1.1 s for all 15 [K23 §4]. "Roughly a second" with no visible luminosity [FR p.44, p.58].
- Cool-flame radius 5.1–7.6 mm. Hot/cool radius ratio 2.3–2.9, mean 2.6 (model predicts 3.2) [K23 Table 2].
- Cool-flame temperature: below the TFP limit of 950 K [K23]; inferred mean T_cf ≈ 869 K at Z_cf ≈ 0.53 and Z_s ≈ 0.75 [K23 §4]; 750 K assumed in the model.
- Cool diffusion flames in general peak at 500–1000 K [K23 §1; PSI research impacts].
- **Cool-flame lifetime**:
  - First-round cool flames lasted ≈13.6–14.3 s and "disappeared as soon as the fuel flow stopped" [FR p.44; K23 Table 2]. They were flow-terminated, and the radiometer and burner T were "steady enough to suggest this flame would have lasted indefinitely with continued flow" [K23 §4].
  - Example (K23 Fig. 5b): hot RE at 36 s, cool flame at 37 s, flow cut at 50 s.
  - On the final day the catalytic type-B thermocouple reignited hot flames. Cool phases lasted 10.9 s and 6.2 s (first cycles) and 3.4–5.8 s (later cycles). Cycles repeated until flow termination at 165 s in the example [K23 §4, Fig. 5c].
  - Proposal hypothesis (not tested): "decrease in fluent flow rate will cause extinction, whereas an increase will cause a transition to a hot diffusion flame" [FR p.6].
- Droplet cool flames burned up to 35 s before extinguishing [K23 §1].

**7. Simulated pressure effects** (Waddell, Dietrich & Nayagam 2026; burner model of droplet-like flames)
- Below 4 atm the hot flame radiatively extinguishes near 1200 K.
- At ≥4 atm it turns into a warm flame (~1000 K), which then radiatively extinguishes before a cool flame forms.
- At ≥5 atm the cool flame can reignite a hot flame through HO₂ chemistry.
- Optically thick reabsorption, especially of the CO₂ 4.3 µm band, matters at high pressure.
- Not CFI-G data, but relevant to pressure trends.

**8. Inverse flames: specifics**
- Soot forms outside the flame. Flames are initially sooty or yellow, then fade to blue as T drops; glowing soot needs ≳800 K [FR p.60–63].
- PSI "Color" column values: blue, yellow, yellow-blue, "yellow then blue" and "limit" [PSI-agg]. The meaning of "limit" is not documented (not found); see F.
- Heat loss to the supply tube produces a hole and oscillations [FR p.60, p.65].
- Ignition sometimes only flashed (8 "flash" rows).
- Higher ambient fuel fraction gives brighter, longer-lived flames (examples in E3).
- For ethylene inverse flames, the extinction O₂ flux is 0.68 g/(m²·s) [Irace 2023].

---

## F. What the outcome labels mean

**PSI tables, column "Self-ext or flow term"**: values `SE`, `FT` and `-` [PSI].
- `SE` = self-extinction (the hot flame went out while the reactant was still flowing).
- `FT` = flow termination (the hot flame was still burning when the MFCs closed). This covers time-out, burner-temperature limit, manual stop, and the comment "FC=flow cut".
- `-` = no hot ignition (burn time 0). In the normal table every `-` row has Hot ignition = No; one normal "No" row is labelled FT [PSI-agg].
- The inverse table "Hot ignition" column also uses `flash`; all 8 flash rows are labelled SE with short burn times [PSI-agg].

**K23 / KT labels**:
- `RE` = "radiatively extinguished";
- `FT` = "flow terminated";
- K23 states "189 flames that extinguished radiatively and the others extinguished when the flow was terminated, in some cases because the burner reached its temperature limit of 723 K."
- [INF] PSI `SE` corresponds to K23 `RE`. K23 calls all self-extinctions radiative, but the PSI table only says self-extinguished.

**Critical point: SE/RE marks only the end of the hot flame**
- "SE" can hide a cool flame that continued invisibly.
- PSI normal table, "Cool flame?" column: values `yes, PCI-A` … `yes, PCI-G` and `no, PCI-hot` [PSI-agg]. "PCI" is not defined in the sources found.
  - The five first-round cool-flame tests are labelled `SE`, and their burn time equals the hot-flame duration. Cross-check: K23 Fig. 5b shows hot RE at 36 s followed by a cool flame until the flow cut at 50 s.
  - The two final-day multi-transition tests are labelled `FT`, with burn time ≈ total flow duration (cf. K23 Fig. 5c: flow terminated at 165 s).
- So in the PSI table, for the same physics, "SE" plus a cool flame means the hot flame quenched, and the burn time does **not** include the cool-flame period. "FT" is used when hot reignitions persisted until flow end.
- FR p.15: during first-round testing "it wasn't known that the cool flames were present after hot flame extinction and the tests were terminated, which unknowingly and sadly shortened the duration of the cool flame combustion."
- Cool flames are invisible to the colour and analog cameras [K23 §2]. Their only evidence is intensified CH₂O* images, the radiometer plateau and burner temperature.
- Outside the 15 identified cases, K23 says only these conditions produced cool flames. [INF] Other SE tests most likely had no sustained cool flame, but undetected momentary ones cannot be excluded. FR p.34 notes "it is possible that momentary cool flames were created".

**Burn time definition**
- Hot-flame duration from ignition. [INF] It appears to be PMT-2 based; see C.
- For SE tests, the "Extinction detected" TP stamp is 8–40 s later than the true extinction, because detection lag plus the 3 s post-extinction delay keeps the flow on. **Do not use TP MFC-off times as extinction times.**

**Other comment and colour labels** (free text) [PSI]:
- "Not lasted long", "hit the temperature limit", "Flow timed out", "Timed out", "Aborted", "Not spherical shape", "Irregular Oscillation", "direct cool - no hot ignition", "Failed to ignite", "No ignition", "Flash".
- Inverse "Color" = `limit` appears with both SE and FT. Undocumented [INF: possibly a sooting-limit / near-limit flame].

**Test-sheet "Out" categories**: `ext. detect`, `time out`, `temp. limit` [PSI test-sheet scan].

---

## G. Data-quality and interpretation issues

- **Visibility**:
  - Cool flames are invisible to the analog and colour cameras and barely visible even at maximum intensifier gain. Images needed averaging, filtering and a 50× digital gain [FR p.43; KT §4.2].
  - The PMTs only marginally detect cool flames [K23].
  - Dim, blue, large hot flames near extinction are hard to see. Gains were raised mid-test [FR p.9; KT §4.2].
  - One cool flame's radius could not be measured because the camera gain was low [K23 Table 2].
- **Camera damage concern**: the intensifier was kept at lower gain during ignition (scripted `intensifierIgnitionGain` vs `intensifierTestGain`), so early cool flames could be missed [FR p.15].
- **Timing**:
  - TP stamps have 1 s resolution. The data CSV uses wall-clock time with ms.
  - Ignition t = 0 must be reconstructed. Flow start precedes ignition by ≈1–2 s.
  - The igniter retracts in ≈0.6 s [EIR deviations].
  - The extinction-detected stamp lags the true extinction by 4–42 s [PSI-agg].
  - Burn time vs MFC-off offsets of 1–2 s for FT tests [PSI-agg].
  - Inconsistency in KT: KT Fig. 4.1 caption gives a hot-flame oscillation at 82.6 s and a cool flame at 88.0 s for "Test D". KT Fig. 4.4b and K23 Fig. 5b give RE at 36 s for Test D. One of these is mislabelled.
- **MFC accuracy**: ±5%; poor below 10% of full scale [K23; FR p.16].
- **Ambient uncertainty**:
  - burner and ambient mole fractions ±0.02 / ±0.01 [K23];
  - ±0.6% for ambient X [KT §4.1].
  - Within-day vitiation means before and after compositions differ [PSI tables].
- **Flame asymmetry**: an off-centre flame at high flow and local quench at the tube. Inverse flames can be "not spherical shape" [FR p.16; PSI comments].
- **Hardware events**:
  - IPSU1 failure on the first test day (3 aborts);
  - IPSU-G and igniter failure on one inverse propane day: 2 good tests, 2 aborts, 3 no-ignitions, image data mostly lost;
  - TFP fibres broken (3 of 5 left);
  - bent thermocouple tips;
  - type-B catalysis caused reignition on the final day [FR pp.15–17].
- **Soot**: possible soot effect on the PMT and radiometer windows in the inverse round [FR p.16].
- **Raw data**: the CSV holds raw voltages; calibration reports were not found online.
- **g-jitter**: "possible that the tests are subtly affected … not evaluated" [FR p.43]. SAMS data are not in PSI.
- **Pressure rise**: constant-volume chamber; the flows are small relative to 83 L. K23 calls the ambient changes negligible.
- **Data loss**: images were sometimes deleted to fit the 127 GB/day limit [FR p.20, p.43].
- **Counting differences**:
  - K23's 280 flames include reignitions.
  - PSI table counts: normal 133 ignited, inverse ~139 plus flashes.
  - Weekly summaries use "sustained" and "momentary" flames.

---

## H. Quantitative reference table

| Quantity | Value | Conditions | Source |
|---|---|---|---|
| Chamber free volume | ≈83 L (design text: 90 L) | ACME insert installed | J21 p.5; K23 §2; FR p.15 |
| Chamber pressure capability | up to 300 kPa | CIR | EIR p.13 |
| Burner | 6.35 mm, 316SS, 10 µm, ~41% porosity | all CFI-G | EIR p.43 |
| Burner T trip | 723 K (K23/KT) vs 650 °C (FR caption) | — | K23 §2; FR p.63 |
| Valve safety timer | 999 s per fill (limiting reactant) | all ACME | FR p.20 |
| Post-extinction delay | 3 s | script parameter | PSI parameter sheet; 227/237 TP |
| Flow time-out example | flowTimeoutSec 45, constantFlowTime1Sec 15 → MFC off at ≈67 s after ignition | inverse round | PSI sheet; PSI-agg |
| Inverse FT burn-time cluster | 66 s (29/58 FT) | inverse | PSI-agg |
| Detection lag (extinction stamp − burn time) | normal 4–38 s (typ. 8–15); inverse 13–42 s (typ. 16–22) | SE tests | PSI-agg |
| Hot flames: total / RE / FT | 280 / 189 / 91 | all CFI-G | K23 §4 |
| Normal SE burn times | 1–304 s (median ≈35 s) | PSI normal | PSI-agg |
| Normal FT burn times | 13–276 s | PSI normal | PSI-agg |
| Inverse SE burn times | 0.5–55 s (median ≈9 s) | PSI inverse | PSI-agg |
| Inverse FT burn times | 12–152 s | PSI inverse | PSI-agg |
| Flame Design burn times | 28–336 s | ethylene, 1 atm, ISS | Irace 2021 |
| Critical extinction T | 1130 K | ethylene ISS normal and inverse, 1 bar | Irace 2023 |
| Critical extinction T | ≈1100 K (1055 K at 104 mg/s) | ethylene and O₂ drop tower | Santa 2007 |
| Hot-flame RE temperature (simulated) | ≈1200 K (p < 4 atm) | burner model | Waddell 2026 |
| Extinction mass flux | 0.2 g/m²s (fuel, normal); 0.68 g/m²s (O₂, inverse) | ethylene, 1 bar | Irace 2023 |
| r_ext scaling | ∝ (ṁ Y / κ_P)^{1/3} | theory and data | Nayagam 2019 |
| r_ext fit | ∝ ṁ^0.44 and ṁ^0.36 | drop tower | Santa 2007 |
| Q_R/Q_C at extinction | ≈0.7 | model | Santa 2007 |
| Quasi-steady radius | r = ṁ / (4πρD ln(1/(1 − Z_st))); ρD = 0.0334 g/m·s | normal flames | K23 Eq. 8 |
| Diffusion time to steady state | ≈30 s (r = 25 mm, D = 20 mm²/s) | estimate | FR p.5 |
| Oscillation lead time example | onset 18 s → extinction 30 s | n-butane 2 mg/s | K23 Fig. 5a |
| Cool-flame conditions | n-butane normal, X_O2 ≈ 0.39, X_f = 0.3, 2.03 bar, 0.75–1.5 mg/s, T_burner > 515 K | — | K23 §4 |
| Hot → cool delay | 0.7–1.1 s | 15 cases | K23 §4 |
| Cool-flame duration (flow-limited) | ≈13.6–14.3 s, ended by flow stop | first round | K23 Table 2 |
| Cool-flame duration (with Pt thermocouple) | 3.4–10.9 s per cycle | final day | K23 Table 2 |
| Cool / hot radii | r_cf 5.1–7.6 mm; r_hf 12.4–17.7 mm; ratio 2.3–2.9 (mean 2.6) | — | K23 Table 2; FR Table 3 |
| Cool-flame T | <950 K (TFP); inferred ≈869 K | — | K23 §4 |
| Type-B reignition T | 720–820 K | final day | K23 §4 |
| Inverse burner T | <500 K | — | K23 §4 |
| BRE self-extinction | below ~25% O₂; large flames within 1.5 min | flat burner | J21 p.7 |
| Non-imaging data rate | 125 Hz | — | FR p.33 |

---

## I. References

1. **PSI-159** — NASA Physical Sciences Informatics, "Advanced Combustion via Microgravity Experiments / Cool Flames Investigation with Gases".
   - DOI 10.60555/cyt1-kh64.
   - https://psi.nasa.gov/physci/repo/data/investigations/PSI-159. Metadata JSON at https://psi.nasa.gov/geode-py/ws/repo/investigations/PSI-159; files via `/geode-py/ws/studies/PSI-159/download?file=…`.
   - Experimental tables, TP, mfcSetpoints, data headers, scanned parameter and test sheets. Supports A–D, F, G, H.
2. **FR** — D.P. Stocker, P.B. Sunderland, R.L. Axelbaum, F.A. Williams, "ACME/CFI-G Final Report (2022)", 21 Nov 2022 (PSI-159 Reports file). Supports A–H.
3. **EIR** — "ACME Engineering Implementation", NASA GRC, 19 May 2022 (PSI-159 Engineering Documents). Supports A, C, G.
4. **J21** — D.P. Stocker, "Recent Research on Flames of Gaseous Fuel Aboard the International Space Station", 33rd JASMAC, Oct 2021. NTRS 20210021781, https://ntrs.nasa.gov/citations/20210021781. Supports A, B, E.
5. **K23** — M. Kim, K.A. Waddell, P.B. Sunderland, V. Nayagam, D.P. Stocker, D.L. Dietrich, Y. Ju, F.A. Williams, P.H. Irace, R.L. Axelbaum, "Spherical gas-fueled cool diffusion flames", Proc. Combust. Inst. 39 (2023) 1647–1656. DOI 10.1016/j.proci.2022.07.015. Free copy: https://par.nsf.gov/servlets/purl/10426596. Supports A–H.
6. **KT** — M. Kim, "Spherical Gas-Fueled Cool Diffusion Flames", M.S. thesis, Univ. of Maryland, 2022. DOI 10.13016/n8ln-uehb, https://drum.lib.umd.edu/handle/1903/29540. Supports A–C, E, G.
7. **Irace 2021** — P.H. Irace, H.J. Lee, K. Waddell, L. Tan, D.P. Stocker, P.B. Sunderland, R.L. Axelbaum, "Observations of long duration microgravity spherical diffusion flames aboard the International Space Station", Combust. Flame 229 (2021) 111373. DOI 10.1016/j.combustflame.2021.02.019. Abstract only. Supports E.
8. **Irace 2023** — P.H. Irace, K.A. Waddell, D. Constales, P.B. Sunderland, R.L. Axelbaum, "Critical temperature and reactant mass flux for radiative extinction of ethylene microgravity spherical diffusion flames at 1 bar", Proc. Combust. Inst. 39 (2023) 1905–1913. DOI 10.1016/j.proci.2022.07.043. Abstract summary only. Supports E, H.
9. **Nayagam 2019** — V. Nayagam, D.L. Dietrich, F.A. Williams, "Radiative extinction of burner-supported spherical diffusion flames: A scaling analysis", Combust. Flame 205 (2019) 368–370. DOI 10.1016/j.combustflame.2019.04.027. Abstract only. Supports E, H.
10. **Santa 2007** — K.J. Santa, B.H. Chao, P.B. Sunderland, D.L. Urban, D.P. Stocker, R.L. Axelbaum, "Radiative extinction of gaseous spherical diffusion flames in microgravity".
    - Combust. Flame 151 (2007) 665–675, DOI 10.1016/j.combustflame.2007.08.009.
    - Text read from the 5th US Combustion Meeting version (paper A06, 2007), NTRS 20070017909.
    - Supports E, H.
11. **Waddell 2026** — K.A. Waddell, D.L. Dietrich, V. Nayagam, "The effect of pressure on radiative losses in hot, warm, and cool microgravity spherical diffusion flames", Combust. Flame (2026). NTRS 20260002014. Supports E.
12. **Waddell 2023** — K.A. Waddell, H.J. Lee, V. Nayagam, R.L. Axelbaum, P.B. Sunderland, "Cool diffusion flames in a stably stratified stagnation flow", Combust. Flame (2023) 112852. DOI 10.1016/j.combustflame.2023.112852. Cool-flame peak temperatures 705–760 K, per the FR abstract. Not read in full.
13. **Cited but not read** (cited in the sources above):
    - S.D. Tse, D. Zhu, C.-J. Sung, Y. Ju, C.K. Law, Combust. Flame 125 (2001) 1265–1278;
    - K.J. Santa et al., Combust. Theory Model. 11 (2007) 639–652;
    - K. Mills, M. Matalon, Proc. Combust. Inst. 27 (1998) 2535–2541;
    - B.H. Chao, C.K. Law, J.S. T'ien, Proc. Combust. Inst. 23 (1991) 523–531;
    - A. Snegirev, Combust. Flame 247 (2023) 112499;
    - K.A. Waddell et al., 14th US National Combustion Meeting (2025) paper 2H05, "Pressure effects in spherical microgravity hot, warm, and cool diffusion flame transitions";
    - P. Dehghani et al., Combust. Flame 228 (2021) 315.

## Not found / open items
- Full text of Irace 2021, Irace 2023 (the exact extinction-size equation), Nayagam 2019 (the prefactor) and the Wang & Chao work.
- The ACME Software Interface Design Document (ACME-DOC-010) and the radiometer and PMT characterization reports: needed for voltage calibrations and for the exact extinction-detection algorithm and thresholds.
- Exact definition of `flowTimeoutSec` (what it counts from), and the time-out values used on each day.
- Meaning of "PCI" in the "Cool flame?" column and of the inverse `Color = limit` label.
- Any CFI-G publication on inverse-flame extinction limits or the hot-flame extinction-time scaling for ethane, propane and n-butane: none found beyond K23.
- Explicit reports of kinetic extinction in CFI-G: none found.
- A separate Sunderland–Axelbaum–Urban–Chao–Liu 2003 spherical-flame paper: not located.
