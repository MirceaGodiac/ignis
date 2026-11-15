# FLEX (Flame Extinguishment Experiment, ISS / CIR / MDCA, PSI-69) — dossier for D(t) and flame-phase modelling

Scope: free-floating methanol and n-heptane droplets. Every number is tagged with a source key (see section I).
"[INF]" marks my own inference, not a source statement. "NOT FOUND" means I searched and found no source.
Per-test result tables were deliberately not transcribed; the only per-test-derived numbers are aggregate
statistics (ranges/medians) that I computed from the published summary table of [TP15], marked [AGG].

Source keys (full citations in section I):
[TP15] Dietrich et al., NASA/TP-2015-216046 (cited elsewhere as NASA/TP-2013-216046), NTRS 20150023456 — the
       FLEX data report, March 2009–Dec 2011 (first 284 tests). The most important single source.
[MST14] Dietrich et al., Microgravity Sci. Technol. 26 (2014) 65–76 (open access).
[N13] Nayagam, Dietrich, Hicks, Williams, 8th US Natl. Combust. Mtg. 2013, paper 070HE-0303 (NTRS 20130014061, local PDF).
[N11] Nayagam, Dietrich, Ferkul, Hicks, Williams, Eastern States Sect. CI Fall Mtg. 2011 (NTRS 20120016855).
[N14] Nayagam, Dietrich, Hicks, Williams, Central States Sect. CI 2014 (NTRS 20140006750) = preprint of C&F 162 (2015) 2140.
[N15s] Nayagam, Dietrich, Williams, ASGSR 2015 slides (NTRS 20160000344).
[N20s] Nayagam, Dietrich, Williams, ASGSR 2020 slides (NTRS 20205008449).
[N21s] Nayagam, Dietrich, Williams, JASMAC-33 2021 slides (NTRS 20210022617, local PDF).
[H12] Hickman, Dietrich, Hicks, Nayagam, Stocker, "FLEX: A decisive step forward…" 2012 (NTRS 20120015928).
[Hk12] Hicks, Dietrich, Nayagam, Williams, ASGSR 2012 slides on water absorption (NTRS 20130010979).
[OM02] O'Malley & Myhre, IAC-02-J.5.05 (2002), CIR and MDCA hardware (NTRS 20030032248).
[FS09] NASA ISS experiment fact sheet "MDCA-FLEX", last update 02/23/2009 (NTRS 20090014812) — PRE-FLIGHT PLAN.
[T15] Takahashi, Katta, Hicks, 9th US Natl. Combust. Mtg. 2015 (NTRS 20150018397) — 2-D simulation.
[F19] Farouk, Dietrich, Dryer, Proc. Combust. Inst. 37 (2019) (NTRS 20190002838) — CFI-era hardware.
[W17] F.A. Williams, "Cool Flames in Microgravity", 1st Intl. Workshop on Near-Limit Flames, 2017 (slides).
[WN20] Williams & Nayagam, Combust. Flame 212 (2020) 242–244 (NSF PAR copy).
[YS14] Yu & Shaw, Gravitational and Space Research 2 (2014) 82–93 (backlit image interpretation).
[W26] Waddell, Dietrich, Nayagam, 2026 manuscript on radiative losses (NTRS 20260002014) — CFI radiometers.
[PSI] PSI repository public metadata record for PSI-69 (psi.nasa.gov search API).
[ISS13] Nayagam, ISS R&D Conference 2013 slides "Cool Flames in Space…".

---------------------------------------------------------------------------------------------------

## A. Apparatus

### A.1 Combustion Integrated Rack (CIR) and chamber
- Chamber: cylindrical vessel with domed end caps, **100 L free internal volume, 40.0 cm ID × 90.0 cm long**,
  **8 removable sapphire windows** [OM02]. Insert envelope max 60.0 cm long × 39.6 cm diameter [OM02].
- Volume as quoted by FLEX papers — sources disagree:
  - "90-liter combustion chamber" [TP15 §2.1], "90 l" [MST14];
  - "free-volume of approximately **95 liters with the MDCA insert installed**" [N13 §2; N11; N14; H12];
  - "100 Liters capacity" [N21s slide 3]; 100 L free internal [OM02].
  [INF] Use ~95 L as the gas volume available to the burning droplet.
- Pressure: "operates at pressures ranging from 0.02 to 3 atmospheres", design 120 psig (~8 atm) [OM02];
  "approximately 0 to 9 atm", FLEX limited by safety analysis to **~3 atm** [TP15 §2.1; MST14]; "0.02 to 5 atm and
  contains pressures up to 9 atm" [N21s].
- Gas supply: FOMA (TP15 calls it Fuel and Oxidizer *Mixing* Apparatus; OM02 Fuel and Oxidizer *Management* Assembly):
  gas bottles (1.0 L up to 80 % O2, 2.25 L up to 50 % O2, 3.8 L up to 30 % O2), ISS N2 line, mass-flow controllers,
  partial-pressure or dynamic blending; 30 slm per manifold, 90 slm total oxidizer flow [OM02]. Exhaust vent package
  with adsorber cartridge (LiOH, activated carbon, silica gel, mol-sieve options) and recirculation loop (max 20 slm)
  [OM02]. Gas chromatograph, lower detection limit 100 ppm, ±2 % accuracy [OM02]. Chamber instrumented with 2
  thermistors, 2 pressure transducers [OM02].
- Vibration: Passive Rack Isolation System (PaRIS) attenuates >1 Hz [OM02]; SAMS data showed g-levels typically
  **< 1e-5 g0** across the spectrum [MST14]; g-jitter nonetheless visibly drove droplet drift [TP15 §2.3, §3.1.1].
- Circulation fan run ~1 min between tests in the same fill to mix the gas [TP15 §2.3 step 6].

### A.2 MDCA (Multi-user Droplet Combustion Apparatus)
- Based on the Space-Shuttle DCE design [TP15; MST14]; Chamber Insert Assembly with dispensers, deployment needles,
  fibre support, igniters, internally mounted colour camera, radiometers; managed from the GRC Telescience Support
  Center [OM02].
- **Deployment needles**: two opposed stainless-steel tubes, **0.010 in (250 µm) OD**, ceramic sleeve, bent 90° and
  flared at the tip, on rotary servomotors; fuel metered to the tips, droplet centres between the tips, then "the
  needles are rapidly accelerated apart … leaves the droplet in place with minimal to no residual motion" [OM02].
  FLEX description: 250 µm OD tubes "with specially treated ends" [TP15] / "polished and fluted ends" [MST14];
  fuel dispensed between horizontally opposed needles, needles then **slowly stretched to slightly less than the
  pull-off distance** (manual, operator judgement), then retracted rapidly just before ignition [TP15 §2.1, §2.3];
  the stretch "minimizes the post-deployment drift velocity and internal circulation" [MST14].
- Fuel reservoirs: gas-tight syringe + remotely actuated solenoid valve, Teflon tubing; ~**2.5 ml methanol, 1.25 ml
  heptane** per reservoir [TP15; MST14]. Experiments were fuel-limited (cubic dependence of volume on D0), which is
  why large methanol droplets at high O2 were not run [MST14 footnote 1].
- Needle contamination: needles had a conformal coating (Dow Corning 1204 primer + Dow Corning 3140 RTV); it dissolved
  /flaked into both fuels; all tests in [TP15] used coated needles; later tests used uncoated needles [TP15 §5.2].
- **Igniters**: two independent hot-wire igniters opposed 180°, each a **4 mm diameter loop of Kanthal A-1 wire,
  30 AWG** on a ceramic mount, moved in by linear motors "in precise linear steps" and quickly retracted after
  ignition [OM02]. Located "180° to each other and in the same plane as the deployment needles" [TP15 §2.1; MST14],
  but TP15 §2.3 step (iii) says "nearly orthogonal to the deployment needles" — internal inconsistency [TP15].
  Distance from droplet ground-controllable; on-time preset; power set by ground-controllable voltage [TP15 §2.3].
  - Igniter power (W) / voltage: NOT FOUND.
  - Igniter on-time in flight: NOT FOUND as a number. Pre-flight plan: "selectable amount of time ranging from
    1 second to 5 seconds" [FS09]. [MST14 Fig. 2] shows the powered (glowing) coils at t = 0.12 s after ignition.
  - CFI-era (later, not FLEX-1) change: radiometer signal used to detect hot-flame initiation and then automatically
    power off and retract the igniter [F19].
- **Fibre mode**: 80 µm SiC fibre [TP15; MST14] (79 µm silicon carbonitride fibre on a 229 mm Retractable Indexing
  Fiber, tension 3000 psi [OM02]); fibre can translate at programmed speed (FLEX used ~3 mm/s) [TP15 captions].
  Fibre tests were used when drift was unacceptable or to study sub-buoyant flow [TP15 §2.1]. Later CFI: 40 µm fibre
  [W26]. Exclude fibre tests from free-droplet modelling (fibre oscillation, flame quenching, translation) [TP15 captions].

### A.3 Residual motion and drift of free droplets
- "Rarely completely motionless; residual velocity from deployment and ignition and then drifted more as a result
  of … g-jitter" → droplets often left the HiBMs FOV (sometimes returning) [TP15 §4.2].
- Typical drift **~1–2 mm/s** [MST14 Fig. 3 caption]; poor deployments **~7–8 mm/s** [TP15 captions].
- Recurrent sequence in TP15 captions: drift after deployment → droplet touches an igniter → direction change after
  igniter withdrawal; igniter heating can impart an impulse (attributed in one case to a trapped bubble) [TP15 App. B].
- Pre-flight requirement was a ≥10 s dwell after needle retraction for internal motion to subside [FS09]; in flight
  deployment occurred "just before ignition" in an automated rapid sequence [TP15] — so the dwell was not used
  [INF from the two sources].

## B. Test procedure and timeline

### B.1 As flown (FLEX 2009–2011) [TP15 §2.3]
1. Activate CIR; 2. FOMA fill (pressure history recorded so composition can be computed a posteriori);
3. activate MDCA; 4. open fuel valve, purge line until fuel appears at needle tip;
5. (a) dispense droplet (operator uses stepper counts as a guide, consults science team), (b) stretch manually,
   (c) operator zooms colour camera out, turns white LED off (HiBMs backlight stays on), hands control to MDCA
   computer, which then in **rapid progression**: (i) start recording (HiBMs, LLUV, colour), (ii) deploy (needles
   rapidly and simultaneously retracted), (iii) igniters on for preset time/power, (iv) igniters off and rapidly
   retracted "a relatively large distance", (v) stop recording after a preset time "**typically on the order of 40 s**".
6. Post-test housekeeping; fan ~1 min if another test follows in the same atmosphere.
- Pre-flight settling: ≥2 min after chamber fill for T and p to stabilise [FS09].
- 4–10 droplets per operating session [TP15 Exec. Summary]; several burns per fill before venting [N13; N14].

### B.2 Durations (published)
- Droplets burned for as long as ~20 s [H12]. [AGG from TP15 Table IX, ~183 parsed rows incl. fibre tests]:
  burn time (TP15 column "Burn time, s") methanol median ~11 s (IQR ~7.6–16.4 s, max ~39 s); heptane hot-flame
  median ~7.7 s (IQR ~3.9–11.4 s). Heptane cool-flame stage adds of order 15–25 s (e.g. MST14 Fig. 2/3: visible
  extinction ~5 s, D²-plateau ~23–28 s after ignition for a ~3.4 mm droplet in 18 % O2, 1 atm).
- Recording ~40 s [TP15] → late cool-flame extinction can occur near/after end of recording [MST14: plateau "near
  the end of the camera recording"].

### B.3 Definition of t = 0
- **TP15 Appendix A (Table IX)**: "Actual D0 — initial droplet diameter or droplet diameter at time zero where
  **time zero is the time when the igniters are turned ON**". **TP15 Appendix B**: "Time zero in the plots corresponds
  to when the igniter was energized. This time came from data files downlinked from the CIR."
- Exception documented: when the HiBMs recorder missed the start, "the plotted data used the first measured droplet
  as time zero even though the droplet had been burning for some time" [TP15 App. B caption, a fibre test].
- [MST14] and [N13] label figure time axes "time after ignition"; [MST14 Fig. 2] shows "t = 0.10 s before ignition"
  as a frame with the droplet still on the needles → needle retraction occurs <~0.1 s before t = 0 [INF].
- K-fit windows differ by paper:
  - [TP15 App. A]: linear fit of D² vs t "from shortly after ignition to extinction".
  - [MST14]: fit between ~0.1 τb and 0.9 τb, **τb = total burn time from igniter withdraw to extinction or burnout**.
  - [N13]: "Only the middle 80 % of the droplet lifetime … to avoid the initial heat-up period and the later
    non-linear behavior close to extinction."
- PSI-69 data-file time column definition: NOT FOUND (PSI file listing is access-restricted; see C.5). [INF] Expect
  timestamps either relative to igniter-on (as in TP15 plots) or absolute acquisition times per camera.

### B.4 What happens in the first ~0.5–1 s
- Droplet shape: "misshaped by the needle retraction but assumes the expected spherical shape within approximately
  1 s" [TP15 §2.3, Fig. 4b]. Shape-oscillation physics: Shringi, Dwyer, Shaw, Microgravity Sci. Technol. 22 (2010)
  17–26 (abstract not retrieved).
- Igniter glow: "a clear ambiguity in the droplet size during ignition (after deployment and before the igniters
  withdraw)" because broadband igniter glow changes droplet/background discrimination; severe in the first nine
  tests (HiBMs shutter 1/30 s); afterwards shutter **1/1000 s** with higher backlight, which minimised the distortion
  [TP15 §5.1]. Igniters can overexpose HiBMs so D cannot be measured until they leave the FOV [TP15 captions].
- Heptane: luminous yellow sooty flame immediately after ignition, becoming blue [MST14; TP15 captions]; residual
  soot "receding in the direction of igniter retraction" [W17].
- Ignition can impart drift; droplet may hit an igniter [TP15 captions].
- Methanol flame diameter and FSR "increase relatively quickly for a very short time (2 s or less)" then decrease
  [MST14]. Heptane flame grows continuously for large droplets [MST14].
- Heat-up: papers exclude an "initial heat-up period" from fits [N13] — no FLEX source quantifies thermal expansion
  (initial dD/dt > 0). [INF] Liquid heptane density falls ~10 % from 298 K to its boiling point (371 K), methanol
  ~7 % from 298 K to 338 K; a fully heated droplet would grow ~3 % in D, but only a thin surface layer heats in the
  first second, so expect at most a ~1 % apparent swelling, below/near the ±50 µm measurement uncertainty for 3 mm
  droplets. Unsteady D² early is dominated by heat-up (K rising from ~0) and the igniter-image artefacts above.

## C. Diagnostics and data reduction

### C.1 Backlit droplet view — HiBMs (High-Bit-Depth Multispectral imaging package)
- Telecentric optics, 12-bit camera; **1024×1024 full frame, FOV ~30 mm on a side (TP15 error section: 30.25 mm),
  ~30 fps**, used for almost all tests [TP15 §2.2, §3.1; MST14]. Scale factor **32.5 px/mm** (TP15 §3.1.1 step 6
  — text says "flame size", a copy error; it is the droplet view). Resolution ±29.5 µm [TP15 §3.1.2].
- Illumination: laser diode **below lasing threshold** (non-coherent), **650–660 nm** centre, collimated [TP15; MST14];
  CIR Illumination Package spec: 660 nm peak, 7 nm FWHM, 90 mm beam [OM02]. [YS14] (FLEX/FLEX-2) instead describes a
  fibre-coupled LED with partial coherence. Images supplied as 12-bit scaled to 16-bit TIFF; droplet dark on light [TP15].
- Generic HiBMs capability: 7.5/15/30 fps, 50 mm or 80 mm FOV options, LCTF 650–1050 nm [OM02]. CFI later used a
  **5 cm × 5 cm** droplet FOV [F19] — not FLEX-1.
- Shutter: 1/30 s (first nine tests), then 1/1000 s [TP15 §5.1].
- Diameter algorithm [TP15 §3.1.1]: choose frames with droplet in FOV; crop; despeckle; manual threshold (constant for
  a test, varies test to test; guidelines from ground tests with precision spheres in the GIU; threshold "had little
  impact on the droplet size" within reasonable limits); fill holes / remove stray objects; measure **projected area →
  equivalent-circle diameter** (ImageJ or Igor Pro). [MST14] same (equivalent circle, after Struk et al. 1998).
- **Droplet-size uncertainty ±50 µm** [TP15 §3.1.2; W26]. For a 3 mm droplet: ±1.7 % in D, ±3.3 % in D² [INF].
- Edge-physics corrections [YS14]: diffraction of partially coherent light; true edge lies between normalised
  intensity In = 0.25 (coherent) and 0.5 (incoherent); calibration sphere best at In = 0.31; diameters vary 0.97–1.06
  mm for a 1.00 mm sphere as In varies 0.25–0.50 (i.e. up to ~6 %). Circle fit (Kasa) with k-means outlier rejection.
  Soot obscuration methods: Image Anal. Stereol. 38 (2019) 53 (Shaw group). Related: Shaw & Yu, MST 26 (2014) 89–99
  "Uncertainties in droplet sizes and burning rates" (paywalled; NOT READ).

### C.2 Flame view — LLUV (Low Light Level Ultra-Violet)
- 1024×1024 frame-transfer CCD, 12-bit, 18 mm Gen-II UV MCP intensifier with gating; **310 nm filter, 10 nm FWHM
  (OH*)**; used **2×2 binned (512×512), FOV ~50 mm (TP15 error section: 57.8 mm)**; **30 fps mostly, some 15 fps** for
  dim flames; intensifier gain varied between tests [TP15 §2.2; MST14]. Scale **8.86 px/mm** (TP15 text says
  "droplet size", copy error). Resolution ±113 µm; **flame-size uncertainty ±200 µm** [TP15 §3.2].
- Viewing window became dirty/smudged early → asymmetric images; flames dim; no Abel deconvolution done; reported
  flame diameter = **outer edge** of OH* (low threshold, likely larger than peak-OH* or peak-T radius) [TP15 §3.2].
- [MST14]: flame diameter from equal-area circle with automatic Ridler–Calvard iterative threshold, or max dimension
  in least-noisy direction when area impossible. [N11]: UV data are "outermost digitally enhanced threshold".
- Cool flames were **invisible** to LLUV and colour camera in FLEX [MST14; N14]; later imaged by removing spectral
  filters / turning down backlight and frame-averaging (cool FSR 3.2) [N15s; W17]; CFI added a 1392×1040 intensified
  camera with 430 nm (CH*) and 390–490 nm (HCHO*) filters, FOV 90×68 mm [F19].

### C.3 Colour camera (MDCA)
- NTSC 640×480; white LED and zoom used during droplet formation; zoomed out and LED off at deployment [TP15; MST14].
- FOV disagreement: ~110×90 mm [TP15 §2.2]; 93.2 mm wide [TP15 §3.3.2]; ~93×70 mm [MST14]. Scale 6.87 px/mm;
  resolution ±146 µm; **uncertainty ±250 µm** [TP15 §3.3].
- Colour-camera flame diameter ≈ **3/4 of LLUV** diameter (outer visible edge vs outer OH*); gap larger for heptane
  because soot luminosity peaks inside the blue envelope [TP15 Exec. Summary, §5.1]. [N13] reports only UV diameters.
- Real-time downlink, used for operations [TP15]. Some tests lost colour frames (IPSU failures) [TP15 captions].

### C.4 Radiometers, pressure, temperature
- FLEX 2009–2011: broadband + water-band radiometers "not sensitive enough to provide reliable quantitative
  information … data are not reported" [TP15 §2.2]; redesigned radiometers flown later [H12].
- Later radiometer specs — sources disagree:
  - broadband **0.6–5.0 µm**, narrowband **5.0–7.0 µm**, sampled at **100 Hz** [N14];
  - wide band **0.6–10.0 µm**, narrow band **5.0–6.5 µm** [N21s];
  - CFI: wideband Si-thermopile, KRS-5 window, **0.58–50 µm**; narrowband thin-film thermopile, **5–7 µm**; both
    **110 mm** from deployment site; radiance uncertainty up to **±20 %** [W26].
  - Narrowband saturates during hot-flame stage of large alkane droplets [N14].
- Radiometer signatures: hot-flame extinction = sudden drop of broadband signal; cool-flame extinction = rapid drop
  of narrowband signal; then decay without abrupt changes during pure evaporation [N14].
- Pressure/temperature: CIR records chamber p and T during fill/evacuation; composition computed a posteriori and
  corrected for fuel vaporised, O2 consumed, CO2 and H2O produced [TP15 §2.3]. In-burn pressure-rise data: NOT FOUND.
- Accelerometers: SAMS heads on rack [MST14].

### C.5 Data products
- Downlinked to science team [TP15 §2.4]: (1) HiBMs settings, (2) LLUV settings, (3) colour-camera settings (two
  files), (4) CIR chamber and pressure data, (5) radiometer data, (6) CIR facility settings, (7) MDCA settings,
  (8) test comments (anomalies, missing data); plus per-camera text files with **image number, image name, time of
  acquisition**. HiBMs/LLUV frames 12-bit TIFF, colour frames BMP. ~24 h to downlink 3–6 test points.
- TP15 Table IX columns: FLEX test number; CIR "FLEX identifier"; date; GMT time; fuel; ambient pressure (**mmHg**);
  initial O2, N2, CO2, He mole fractions; actual D0 (mm, at igniter-on); visible-flame extinction diameter (mm; "-"
  if burn to completion or disruption; extrapolated if droplet left FOV shortly before extinction); burning rate
  (mm²/s); burn time (s); test end (Extinction / Disruption / Completion); ambient H2O, heptane and methanol vapour
  mole fractions "from detailed records of the chamber fill, recirculation, filter, and combustion history".
- CIR identifiers of the first nine cabin-air engineering tests contain "CAL" (e.g. pattern "20CALnn", "30CALnn",
  "40CALnn") and TP15 captions call them "calibration run/test point" with non-optimal LLUV settings [TP15 Table IX,
  App. B]. Meaning of later identifier codes: NOT FOUND.
- PSI-69 [PSI]: title "Flame Extinguishment Experiment", investigation 2007-08-01 to 2016-07-31, public release
  2016-08-01 (epoch 1469923200), grant NNX09AM33A, hardware CIR/MDCA. The study/file listing endpoint returns
  "permission" errors without login. File names DropletData / LLUVData / ColorCamData / TestSheet / Comments /
  MDCA testpoint settings, their columns and units: NOT FOUND in any accessible source. [INF] They very likely map to
  TP15 items: droplet D(t) (mm, s from igniter-on), LLUV and colour-camera flame D(t), Table-IX-style test sheet,
  item (8) comments, item (7) MDCA settings.
- TP15 says its data are also available as text files and compressed videos on a DVD addendum and "from the NASA
  Physical Science Informatics data repository (psi.nasa.gov, registration required)".

## D. Test matrix

- Overall FLEX family: "over 1500 tests" incl. FLEX, FLEX-2, FLEX-ICEGA, FLEXJ, CFI, ISS 2008–2016 [N21s].
- FLEX-1 (PSI-69 core): hardware launched Nov 2008 [H12]; first test 5 Mar 2009 [TP15]; main sequence ended Dec 2011
  when fuel was depleted; supplementary heptane and methanol reservoirs plus a **40 % O2 / 60 % Xe** bottle for
  Nov 2012–Jan 2013 [H12]. 276 tests in 225 unique conditions by mid-2012 [H12]; TP15 documents the first 284 tests.
  >100 methanol tests [N13]; 104 methanol tests of which 54 gave extinction diameters with droplet in view (as of
  2011) [N11]. Pre-flight plan: 196 test points [FS09].
- Planned ranges [FS09]: O2 0.10–0.50 (procedure text 0.1–0.4), suppressant (CO2, He) 0–0.70, pressure 0.5–1.0 atm
  (procedure text 0.5–3.0 atm), D0 2–5 mm. PSI approach text: test pressure 0.5–1.5 atm, ~3 mm droplets [PSI].
- As flown [TP15; MST14; H12; N13; N11]: fuels methanol and n-heptane; D0 ~1.5–5 mm [MST14] (TP15: 2–5 mm; [AGG]
  methanol D0 1.7–4.5 mm, median 3.2; heptane 1.1–4.6 mm, median 2.9); planned nominal sizes ~2, 3, 4 mm [H12];
  pressures 0.70–3.05 atm, mostly 0.7–1 atm [TP15]; O2 ~0.12–0.40 [TP15; N13 methanol 0.12–0.30];
  diluents N2, CO2 (to 0.70), He (to ~0.45 for methanol per [N21s]; 5–45 % He, 5–75 % CO2, 13–30 % O2, D0 1–4.5 mm
  for methanol per [N21s]); Xe only from late 2012 [H12].
- Campaign order [TP15 §4.1]:
  1. 9 engineering ("CAL") tests in ISS cabin air (0.21 O2, 0.005 CO2, 1.0 atm), Mar–Apr 2009;
  2. high-pressure CO2-enriched (0.21 O2 / 0.70 CO2 / 0.09 N2 at ~2–3 atm) — stopped after a few tests because the
     fuel system did not tolerate high/low pressure cycling;
  3. 0.7 and 1 atm, N2-diluted, higher O2 (0.30–0.40) — showed disruptive burning, no finite extinction;
  4. N2-diluted series starting at 0.21 O2 (1 atm), progressively diluted with N2 in the same chamber until no
     quasi-steady burning (LOI);
  5. CO2 series: (a) 1.0 atm starting from 0.21/0.79 and adding CO2 at constant pressure (O2/N2 ratio fixed 0.27);
     (b) 0.70 atm starting ~0.30 O2 (EVA pre-breathe) with CO2 from 0.15 upward;
  6. He-diluted tests (incl. 0.21 O2 / 0.30 He at 0.7 atm for methanol) [MST14; N13].
  Typically both fuels tested in a given atmosphere on different days over ~3–4 mm; fibre/translation tests
  interleaved [TP15].
- **Same-fill contamination**: atmospheres were frequently reused; tests "conducted in numerical order, so there was
  a small amount of accumulated carbon dioxide, water vapor, and fuel vapor"; TP15 ambient values are corrected for
  fuel vaporised, O2 consumed, CO2 and H2O produced [TP15 §2.3, §4.1]. Table IX carries ambient H2O and fuel-vapour
  mole fractions. Pre-flight purity requirement: fuel vapour < 0.005 (tests without CO2) and < 0.02 each of CO,
  CO2 and other products [FS09]. [INF] One 3 mm heptane droplet (~10 mg) in 95 L at 1 atm (~3.9 mol gas) consumes
  ~1e-3 mol O2 → ΔX_O2 ≈ −2.5e-4 per burn; methanol 3 mm (~11 mg) ≈ −5e-4 mol O2 → ΔX_O2 ≈ −1.3e-4; CO2/H2O rise
  similar magnitude. So a fill used for ~5 burns drifts by ~0.1 percentage point of O2 — small vs. test-to-test
  O2 steps (~1 point) but not zero; ΔP from one burn is ~+0.01–0.03 % [INF].
- Chamber gas temperature: "ambient temperature" (cabin); numeric T∞ in FLEX papers: NOT FOUND ([T15] simulations
  assume 300 K; [W26] 298 K).

## E. Methanol physics as observed

### E.1 Burning-rate constant K
- Classical QS comparison: Kcal = (8 λg/(ρl cpg)) ln(1+B), B = (qf YO,∞ νf + cpg(T∞−Tb))/L with Law–Williams
  property rule [N13 Eqs. 1–2]. Measured Kex ≈ 0.2–0.8 mm²/s (Fig. 3 axes) [N13].
- **Small droplets burn faster than QS theory** (water-condensation heat release dominates before early extinction);
  larger droplets lose this gain once water evaporation begins [N13 §3.1]. Run-to-run scatter "fairly substantial" [N13].
- K decreases with decreasing O2; replacing N2 by CO2 slightly decreases K; deviation (Ke−Kc)/Ke grows strongly
  negative at low O2 (measured < QS) [N11 Fig. 1]; below ~14 % O2 only transient combustion [H12].
- Diluent: He raises K (conductivity), CO2 lowers it [N13; MST14]. MST14 Fig. 6 (2.67–2.83 mm, 0.21 O2): K(He) /
  K(N2) ≈ 1.35 and K(CO2)/K(N2) ≈ 0.87 [ratios computed from MST14 values].
- [AGG, TP15 Table IX, incl. fibre tests, contaminated needles]: methanol K median 0.46 mm²/s (IQR 0.38–0.53, range
  0.28–0.73); by O2 bin median 0.37 (O2<0.16), 0.39 (0.16–0.19), 0.49 (0.19–0.22), 0.54 (>0.22); He-containing
  median 0.59 vs 0.41 N2-only and 0.41 CO2. (Pressure effect confounded with O2/He choices.)
- Instantaneous K decreases just before visible extinction; after extinction vaporisation drops rapidly but
  continues slowly in the hot gas [MST14; N13].
- 2009 fact sheet: pressure "does not significantly influence the droplet burning rate" [FS09] (expectation, not result).

### E.2 Flame standoff ratio (FSR = Df/D, LLUV outer edge)
- Quasi-steady FSR ≈ **5** in 0.21 O2 atmospheres with N2, CO2 or He [MST14 Fig. 6]; FSR decreases with increasing
  O2 (≈3–8 over 0.12–0.30 O2) with scatter from D0 and pressure [MST14 Fig. 7; N11 Fig. 2a]. Large droplets that
  extinguish radiatively never reach a QS FSR [MST14].
- Water-absorption QS model [N13 Eq. 3–4; N11 Eq. 1; Nayagam C&F 157 (2010) 204]:
  Df/D = ½[a + √(a² − 2a + 4b + 1) + 1], with a ≈ (Wav νo ρf K)/(8 Wf νf ρg Do xo,∞) and b a group in
  (Ww/Wf)(νf/νo)(Dw/Do)(xw,f/xo,∞) and liquid densities; K = measured burning constant, Do/Dw effective O2/H2O
  diffusivities, xw,f water mole fraction at the flame. The PDF text extraction lost Greek symbols, so the exact
  placement of densities in b must be checked on N13 p. 7. Measured He FSR consistently above prediction [N13].
- Flame diameter first increases then decreases to extinction (signature of diffusive extinction) [N13; MST14].

### E.3 Extinction mechanisms and correlations
- **Diffusive (residence-time) extinction** via water absorption: water condenses/absorbs at ~constant rate until
  ~**30 % of droplet mass is water**, then water flux reverses, flame temperature drops below the 1200–1700 K kinetic
  extinction range → abrupt extinction [N13 §1]. Extinction flame temperature ~1300–1600 K depending on atmosphere
  [N11]; AEA with cp raised 60 % (Tad ≈ 1700 K) matches air data [N13 Fig. 6].
- **De increases ~linearly with D0** for diffusive extinction (USML-2, FLEX) [N13; MST14 Fig. 8; Hk12]; in air at
  1 atm data lie above RRA/AEA/numerical predictions, slope steeper than AEA-2 [N13 Fig. 6]. Diffusive data at
  different diluents "fall approximately along a straight line, irrespective of the diluent" (dashed line in N13
  Fig. 5). **Numerical slope/intercept of that line: NOT FOUND in text** (figure only). [INF] Illustrative published
  examples at 0.21 O2, 2.7–2.9 mm give De/D0 ≈ 0.40–0.55; [AGG] all methanol De/D0 range 0.11–0.99, median 0.60.
- Drop-tower precursor (D0 1.25–1.72 mm, 21 % O2, 1 atm): De increases with D0; more CO2 slightly decreases De;
  critical-Damköhler theory [Hicks et al. C&F 157 (2010) 1439, abstract].
- **Radiative extinction**: large D0, low O2; flame large and far from droplet; De close to D0 (points near the 45°
  line) [N13 Fig. 5]. CO2 promotes radiative extinction (IR emissivity) [N11].
- **Regime map (1 atm, N2/CO2 diluent) [N11 Fig. 3]**: O2 ≥ 18 %: all diffusive (small De); O2 ≤ 14 %: all radiative
  (De near D0, "transient", De depends on ignition energy, arguably outside LOI); **15–16 % O2: critical D0** —
  diffusive below, radiative above, critical D0 larger at 16 % than at 15 %. Below ~14 % O2 radiative extinction
  replaces water-absorption extinction [N11; H12].
- **LOI**: 0.12 (O2/N2), ~0.14 for CO2 added to air (0.14/0.56/0.30), ~0.14 for He added to air, 1 atm [MST14];
  "12–13 %" [N13; N21s]; some burning observed at 13 % O2 [N11]. Ground drop-tower (Shaw & Wei 2012): 0.15 ± 0.02.
  (MST14 text writes the He LOI composition as "0.14/0.56/0.35 O2/N2/CO2", evidently a typo.)
- Extinction appearance: diffusive = flame present in one frame, gone the next (≤ 33 ms); radiative = gradual over
  several frames; occasional ~1 Hz oscillation before radiative extinction (less common than for heptane) [MST14].
- **Scatter / bimodality explanations** in sources: (i) two extinction branches with a critical D0 at 15–16 % O2
  [N11]; (ii) radiative-branch De depends on ignition energy [N11]; (iii) D0 and pressure variations [MST14];
  (iv) needle-coating contaminant → disruptive extinction "probably due to the contaminant"; diffusive De minimally
  affected, radiative De unaffected [TP15 §5.2]; (v) trapped gas bubbles → growth/disruption [TP15 captions].
- Water-flux transition detectable in LLUV OH* intensity history (condensation → evaporation) [Hk12].
- No soot for methanol at FLEX pressures; flame standoff small [MST14].

## F. n-Heptane physics as observed

### F.1 Hot-flame burning
- Strongest controlling variable: **D0** [MST14]. Example: ~2.1 vs ~3.0 mm in 0.20/0.75/0.05 O2/N2/CO2, 1 atm:
  K ≈ 0.53 vs 0.49 mm²/s (smaller droplet faster); 3.41 mm in 0.18 O2: K ≈ 0.38 mm²/s [MST14 Figs. 3–4].
- [AGG, TP15 Table IX, visible-flame stage]: heptane K median 0.55 mm²/s (IQR 0.46–0.65, range 0.05–0.90); by O2
  bin median 0.43 (<0.16), 0.48 (0.16–0.19), 0.53 (0.19–0.22), 0.63 (>0.22); He-containing 0.67 vs N2-only 0.56,
  CO2 0.52. Note TP15 K was fitted "shortly after ignition to extinction" — for radiatively extinguished droplets this
  is a short, transient window.
- Smaller-droplet flame: luminous yellow → bright blue, Df peaks mid-burn then shrinks, FSR increases ~linearly
  until just before extinction; larger droplet: Df grows throughout, flame dims continuously, FSR rises more slowly;
  visible-flame duration about the same for both [MST14]. Hot-flame FSR ~10 ("lie in the outer transient zone") [W17].
- Soot: moderately sooting [MST14]; soot shell can obscure the droplet edge in HiBMs (Dembia et al. 2012) [MST14];
  soot luminosity dominates colour-camera images [TP15]; luminous soot generally not present after the ignition
  transient in large-droplet flames [W26 for dodecane/heptane]. Soot volume fraction measurable with HiBMs but not
  reported in TP15 (Suh et al. 2011) [TP15 §3.0].
- Decane/octane comparison values [N14] (K hot → K cool, mm²/s): decane 0.5 atm 21 % O2 4.03 mm 0.41 → 0.33;
  17 % O2 0.31 → 0.27; 14 % O2 0.29 → 0.25; decane 25 % O2 (no radiative extinction) 0.52; octane 1 atm air 3.6 mm
  0.48 → 0.38; +15 % He 0.54 → 0.43; +50 % He 0.63 → 0.53.

### F.2 Hot-flame radiative extinction
- Occurs frequently for heptane; extinction begins locally on the side opposite to drift and retreats over
  **5–10 frames**; sometimes flame oscillations (min size → flash back, growing amplitude) at **~1 Hz**, only for
  radiative extinction [MST14].
- Size threshold: radiative extinction in air for droplets "greater than about 3 mm" [Paczko et al. CTM 2014, via
  search snippet]; Liu et al. (C&F 171, 2016): for D0 ≳ 2 mm flames "often disappeared" (radiative) [abstract via
  search]. [AGG] heptane visible-flame De/D0 median 0.76 (range 0.04–1.00).
- O2: lowering O2 → larger diffusive-extinction De, smaller radiative-extinction De; LOI where equal [MST14].
  Lower O2 → earlier hot and cool extinctions [N14]. Higher O2 (25 % at 0.5 atm, decane) prevents radiative
  extinction; flame then burns to small size and ends by micro-explosion or diffusive extinction [N14].
- **LOI heptane**: 0.15 (O2/N2), 0.17 (0.17/0.63/0.20 O2/N2/CO2), ~0.15 for He into air, 1 atm [MST14]; drop-tower
  0.16 ± 0.01 (Shaw & Wei 2011). Below LOI transient burning still possible from ignition energy [MST14].
- Diffusive extinction of the hot flame is hard to observe: droplet disrupts before the (small) diffusive De is
  reached [MST14]; theoretically heptane diffusive De is independent of D0 [MST14 footnote 2].
- Disruption: common at small sizes; may partly be the needle contaminant but clean-needle tests also disrupted
  [TP15 §5.2]. [AGG] test-end counts (heptane, parsed rows): Extinction 39, Disruption 22, Completion 11.
- 2-D simulation: hot flame extinguishes at ~1200 K (CO + OH → CO2 + H shuts off) [T15].

### F.3 Cool-flame (second-stage, low-temperature) burning
- After radiative extinction of the visible flame, D² continues to decrease linearly ("vigorous vaporisation") for an
  extended period, then an **abrupt plateau** (cool-flame extinction) slightly **preceding a large vapour cloud** in
  the colour view [MST14; N12 abstract via search]. Seen over air, N2, CO2 and He-diluted atmospheres [MST14];
  "only occurs after radiative extinction of the hot flame" [MST14]. Also n-octane, n-decane, n-dodecane [N14; N21s].
- **Cool-flame K slightly lower than hot-flame K** [N14; T15]: numerical ratios available for decane/octane (F.1):
  Kc/Kh ≈ 0.79–0.87. Heptane-specific Kc numbers: in Nayagam et al. C&F 159 (2012) 3583 (paywalled) — NOT FOUND in
  accessible text. [INF] Expect heptane Kc ≈ 0.8–0.9 × Kh, i.e. ~0.3–0.45 mm²/s in air/N2 at 1 atm.
- Cool-flame K increases with D0 (attributed to O2 depletion by the preceding hot flame; asymptotic correction
  improves but does not close the gap) [N15s]; partial-burning (Liñán) QS model predicts Kc within ±20 % but not its
  D0 or pressure dependence [N15s; AIAA J 54 (2016) 1235 abstract].
- Cool-flame FSR ≈ **3** (measured 3.2 for decane in air 1 atm, predicted 3.1; Farouk–Dryer heptane simulations
  2.6–3.3, depends only on O2, not on D0 or CO2) [N15s; W17]. Cool flame is broad, close to the droplet [N21s].
- Cool-flame temperatures ~700–800 K [MST14 (F&D sim ~700 K); T15; N21s]; fuel and O2 leak through; main products
  H2O, CO, H2O2, CH2O, C2H4, little CO2, no soot [Paczko et al. CTM 18 (2014) abstract].
- Duration: order 15–25 s for 3–4 mm droplets at 1 atm (MST14 Fig. 2: ~5 s → ~23 s after ignition; 2-D gaseous-
  source simulation 18 s [T15]).
- **Cool-flame extinction diameter Dce**:
  - O2/N2: **Dce ∝ [O2]^(−3/4)** (molar concentration), from Arrhenius crossover argument (R decomposition Ea ≈ 120
    kJ/mol, KHP decomposition Ea ≈ 180 kJ/mol → chemical time ∝ [O2]^(−3/2), and D² ∝ time) [N14 §3.4, Fig. 4a].
  - All diluents and pressures: **Dce²/Kc ∝ C_O2^(−3/2)** (C_O2 in mol/cm³, range ~2e-6–2e-5) [N14 Fig. 4b]; the
    correlation "does not work well at higher pressures" [N15s].
  - Dce **increases with D0** and with **ignition energy** [W17].
  - Dce decreases with pressure much faster than predicted [W17]; for dodecane the best-fit inverse power is **1.7–1.8**
    vs theory 0.5 [WN20]. Heptane exponent: NOT FOUND.
  - He: higher Kc, shorter stages; 75 % He at 21 % O2 (octane) → hot flame dies within a fraction of a second, cool
    flame transient, no quasi-steady state [N14]; He dilution makes cool burning "highly transient — QS not valid"
    [N20s; N21s]. Xe: lowers burning rates but sustains cool flames at much lower O2 than He [N20s].
  - Simplified 6-step AEA theory (Williams & Nayagam CTM 23 (2019)): agreement reasonable for heptane and octane,
    dodecane over-predicted [N21s; N20s]. Peters/Seshadri 770-step mechanism agrees with heptane Dce; 62-step
    over-predicts ~10× [Paczko CTM 2014 abstract; W17].
- Vapour cloud: forms "slightly prior to or following" cool-flame extinction [N14]; sometimes toroidal ring; HiBMs
  background intensity drops as it forms [TP15 captions]; attributed to condensation of leaked fuel / partially
  oxidised products [Paczko CTM 2014; T15 (n-heptane XnC7H16 ≈ 0.37, XH2O ≈ 0.1 in cool zone)].
- Post-cool-flame: pure evaporation in hot ambient, radiometers decay smoothly [N14]; D² nearly flat (plateau)
  [MST14]. Post-extinction evaporation-rate numbers: NOT FOUND.
- History dependence: for a given size range a droplet can burn with either a hot or cool flame depending on initial
  condition [MST14].

### F.4 Elevated pressure, re-ignition, warm flames
- Possible re-ignition of hot flame from cool flame "when the atmospheric pressure is slightly elevated" [T15 intro];
  "Recent results show re-ignition with decane" [ISS13]; 2-D simulation re-ignition after 18 s cool stage [T15].
- Warm flames (~970 K) between hot (~1500 K) and cool (~765 K) for 3 atm with He substitution >20 % (simulation,
  heptane 4 mm); >60 % He → only cool flame [F19]. Dodecane 3 atm air: warm-flame K > hot K, extinction at 0 mm,
  oscillatory extinction; FSR close to hot flame [N21s]. These are CFI-era (post-2016) results, mostly dodecane.
  FLEX-1 heptane at 2–3 atm (CO2 0.70) was only a handful of early tests [TP15 Table II]; one 2 atm test showed
  continued vigorous vaporisation after visible extinction ("probably cool flame") [TP15 caption].

### F.5 Regime map for heptane (synthesis — [INF] unless tagged)
| Ambient (≈1 atm) | small D0 (≲2–2.5 mm) | large D0 (≳3 mm) |
|---|---|---|
| O2 ≥ ~0.25 (N2) | burn to small size; disruption/micro-explosion or completion [TP15; N14] | same; radiative extinction avoided [N14] |
| air / 0.18–0.21 O2 (N2 or ≤0.15 CO2) | hot flame to small size, disruption or completion; no cool flame [MST14; N14 abstract] | hot flame → radiative extinction (≈ seconds) → cool-flame QS burning (~15–25 s) → cool extinction + vapour cloud → evaporation [MST14; N14] |
| 0.15–0.17 O2 | short hot flame, early extinction | early radiative extinction; cool flame possible; earlier cool extinction [N14] |
| ≤ LOI (0.15 N2; 0.17 CO2; 0.15 He) | transient burning only / no ignition [MST14] | same |
| heavy He (≥ ~50–75 %) | — | hot flame very short, transient cool flame, no QS [N14] |

## G. Known data-quality issues
1. **Drift out of HiBMs FOV** (30 mm): data end abruptly or have gaps; sharp spurious decline of D on either side of
   a gap when droplet is partially out of view; extinction De sometimes extrapolated or omitted [TP15 §4.2, App. A].
2. **Igniter glow / overexposure** during ignition; igniters obscure droplet until withdrawn; droplet–igniter contact
   [TP15 §5.1, captions].
3. **Needle-retraction deformation** for ~1 s [TP15]; trapped bubbles → growth, disruption, shattering [TP15 captions].
4. **Soot obscuration** of the droplet edge (heptane), soot luminosity in colour camera [MST14; TP15].
5. **Diffraction/partial coherence** edge bias (up to ~6 % in D depending on threshold) [YS14]; threshold set
   manually per test [TP15].
6. **LLUV window smudge**, dim flames, intensifier gain changes, 15 vs 30 fps, outer-edge definition [TP15].
7. **Needle coating contamination** in all TP15 tests (disruptions, "sparklers") [TP15 §5.2].
8. **Recorder dropouts** (IPSU failures) — missing frames/segments; at least one test re-zeroed at first measured
   frame [TP15 App. B].
9. **Radiometer saturation / insensitivity** (2009–2011 none usable) [TP15; H12; N14].
10. Fibre effects (oscillation, quenching, conduction) — fibre tests only [TP15; N13 refs].
11. Camera synchronisation: each camera has its own image-time file; t = 0 from CIR igniter-on record [TP15]. Inter-
    camera offset magnitude: NOT FOUND.
12. Same-fill ambient drift (D above); in-burn pressure rise: NOT FOUND ([INF] ≪ 0.1 %).
13. 30 fps limits extinction timing to ±33 ms; diffusive methanol extinction is one-frame [MST14].
14. Non-sphericity: free droplets spherical after ~1 s; equivalent-area diameter hides residual ellipticity [TP15].

## H. Quantitative reference table
| Quantity | Value | Source |
|---|---|---|
| Chamber volume | 100 L free (empty); ~95 L with MDCA; "90 L" in TP15/MST14 | OM02; N13; TP15 §2.1 |
| Windows | 8 sapphire | OM02 |
| Needles | 250 µm OD stainless | TP15 §2.1; OM02 |
| Igniters | 2 × 4 mm loops, Kanthal A-1, 30 AWG, opposed 180° | OM02 |
| Igniter on-time | planned 1–5 s (flight value not found) | FS09 |
| Recording length | ~40 s | TP15 §2.3 |
| t = 0 | igniter energised | TP15 App. A/B |
| HiBMs | 1024², ~30 mm FOV, ~30 fps, 32.5 px/mm, 650–660 nm backlight | TP15 §2.2/3.1 |
| D uncertainty | ±50 µm | TP15 §3.1.2 |
| LLUV | 512² binned, ~50 mm FOV, 30 (15) fps, 310 nm ±5 nm, 8.86 px/mm, ±200 µm | TP15 §2.2/3.2 |
| Colour cam | 640×480, ~93 mm wide, 6.87 px/mm, ±250 µm; Df ≈ 0.75 Df(LLUV) | TP15 §3.3, §5.1 |
| Radiometers | 0.6–5 µm & 5–7 µm at 100 Hz (N14) / 0.6–10 & 5–6.5 µm (N21s) | N14; N21s |
| Shape relaxation | ~1 s after retraction | TP15 §2.3 |
| Drift | typical 1–2 mm/s, up to 7–8 mm/s | MST14 Fig. 3; TP15 |
| g-level | < 1e-5 g0 | MST14 |
| K fit window | 0.1–0.9 τb (τb from igniter withdraw) / middle 80 % / shortly after ignition→ext. | MST14; N13; TP15 |
| Methanol K | ~0.28–0.73 mm²/s, median ~0.46; He ×1.35, CO2 ×0.87 vs N2 | AGG TP15; MST14 Fig. 6 |
| Methanol FSR | ≈5 at 0.21 O2; ~3–8 decreasing with O2 | MST14 Figs. 6–7 |
| Methanol De | linear in D0 (diffusive); ~D0 (radiative); water ~30 % mass at reversal | N13; MST14 Fig. 8 |
| Methanol regime | ≥18 % O2 diffusive; ≤14 % radiative; 15–16 % critical D0 (1 atm) | N11 Fig. 3 |
| Methanol LOI | 0.12 (N2); 0.14 (CO2 into air); 0.14 (He into air); "12–13 %" | MST14; N13; N21s |
| Heptane hot K | ~0.38–0.53 (examples); median ~0.55 [AGG] | MST14; AGG TP15 |
| Heptane LOI | 0.15 (N2); 0.17 (CO2); 0.15 (He) | MST14 |
| Radiative extinction | slow, 5–10 frames; ~1 Hz oscillations | MST14 |
| Cool/hot K ratio | ~0.79–0.87 (decane, octane) | N14 |
| Cool FSR | ~3 (3.2 measured, 3.1 QS) | N15s; W17 |
| Hot FSR (heptane) | ~10 | W17 |
| Dce scaling | Dce ∝ [O2]^(−3/4) (O2/N2); Dce²/Kc ∝ C_O2^(−3/2) | N14 Fig. 4 |
| Dce vs P (dodecane) | ∝ P^(−1.7…−1.8) observed; theory P^(−0.5) | WN20 |
| Cool-flame T | ~700–800 K; hot extinction ~1200 K | T15; MST14 |
| Cool stage duration | ~15–25 s (3–4 mm, 1 atm) | MST14 Fig. 2; T15 |

## I. References
1. [TP15] D.L. Dietrich, P.V. Ferkul, V.M. Bryg, M.V. Nayagam, M.C. Hicks, F.A. Williams, F.L. Dryer, B.D. Shaw,
   M.Y. Choi, C.T. Avedisian, "Detailed Results From the Flame Extinguishment Experiment (FLEX) March 2009 to
   December 2011", NASA/TP-2015-216046, Nov 2015. NTRS 20150023456,
   https://ntrs.nasa.gov/api/citations/20150023456/downloads/20150023456.pdf — A, B, C, D, E, F, G, H.
2. [MST14] D.L. Dietrich et al., "Droplet Combustion Experiments Aboard the International Space Station",
   Microgravity Sci. Technol. 26 (2014) 65–76, doi:10.1007/s12217-014-9372-2 (open access) — A, B, C, D, E, F, G, H.
3. [N13] V. Nayagam, D.L. Dietrich, M.C. Hicks, F.A. Williams, "Methanol Droplet Combustion in Oxygen-Inert
   Environments in Microgravity", 8th US Natl. Combust. Mtg., paper 070HE-0303, 2013. NTRS 20130014061 (local PDF)
   — A, B, C, E, H.
4. [N11] V. Nayagam, D. Dietrich, P. Ferkul, M.C. Hicks, F.A. Williams, "Methanol Droplet Extinction in
   Oxygen/Carbon-dioxide/Nitrogen Mixtures in Microgravity: Results from the ISS Experiments", Eastern States
   Section CI Fall Technical Meeting, Storrs CT, Oct 9–12 2011. NTRS 20120016855 (PDF uses a shifted font; decoded)
   — D, E, H.
5. [N14] V. Nayagam, D. Dietrich, M.C. Hicks, F.A. Williams, "Cool-flame extinction during n-alkane droplet
   combustion in microgravity", Central States CI Spring Mtg. 2014, NTRS 20140006750; journal version Combust.
   Flame 162 (2015) 2140–2147, doi:10.1016/j.combustflame.2015.01.012 — C, F, H.
6. [N12, not read] V. Nayagam, D.L. Dietrich, P.V. Ferkul, M.C. Hicks, F.A. Williams, "Can cool flames support
   quasi-steady alkane droplet burning?", Combust. Flame 159 (2012) 3583–3588, doi:10.1016/j.combustflame.2012.07.012
   (paywalled; only abstract snippets seen) — F.
7. [N15s] V. Nayagam, D.L. Dietrich, F.A. Williams, "Aspects of Cool-Flame Supported Droplet Combustion in
   Microgravity", ASGSR 2015, NTRS 20160000344 — C, F, H.
8. [N20s] V. Nayagam, D.L. Dietrich, F.A. Williams, "Effects of Ambient Inert Gas Properties on Cool-Flame
   Combustion of Normal-Alkane Droplets in Microgravity", ASGSR 2020, NTRS 20205008449; journal: Combust. Flame
   (2021), "Effects of properties of atmosphere diluents on cool-flame combustion of normal-alkane droplets",
   ScienceDirect pii S0010218021001358 (not read) — F.
9. [N21s] V. Nayagam, D.L. Dietrich, F.A. Williams, "Isolated Single Droplet Combustion: FLEX Experiments onboard
   the ISS", JASMAC-33, 15 Oct 2021, NTRS 20210022617 (local PDF) — A, C, D, E, F.
10. [H12] J.M. Hickman, D.L. Dietrich, M.C. Hicks, V. Nayagam, D.P. Stocker, "FLEX: A Decisive Step Forward in
    NASA's Combustion Research Program", 2012, NTRS 20120015928 — B, C, D, E.
11. [Hk12] M.C. Hicks, D.L. Dietrich, V. Nayagam, F.A. Williams, "Dynamics of Water Absorption and Evaporation
    During Methanol Droplet Combustion in Microgravity", ASGSR 2012, NTRS 20130010979 — E.
12. [OM02] T.F. O'Malley, C.A. Myhre, "The Fluids and Combustion Facility Combustion Integrated Rack and the
    Multi-User Droplet Combustion Apparatus…", IAC-02-J.5.05, 2002, NTRS 20030032248 — A, C.
13. [FS09] NASA, "Multi-User Droplet Combustion Apparatus – Flame Extinguishment Experiment (MDCA-FLEX)" ISS
    experiment summary, updated 02/23/2009, NTRS 20090014812 — A, B, D.
14. [T15] F. Takahashi, V.R. Katta, M.C. Hicks, "Combustion Characteristics in a Non-Premixed Cool-Flame Regime of
    n-Heptane in Microgravity", 9th US Natl. Combust. Mtg. 2015, NTRS 20150018397 — F.
15. [F19] T.I. Farouk, D. Dietrich, F.L. Dryer, "Three stage cool flame droplet burning behavior of n-alkane
    droplets at elevated pressure conditions: Hot, warm and cool flame", Proc. Combust. Inst. 37 (2019), NTRS
    20190002838 — A, C, F.
16. [W17] F.A. Williams, "Cool Flames in Microgravity", 1st Intl. Workshop on Near-Limit Flames, Boston, 29 Jul 2017,
    http://www.princeton.edu/~yju/1st-near-limit-flames-workshop/presentations/FA%20Williams-Microgravity.pdf — B, F.
17. [WN20] F.A. Williams, V. Nayagam, "Cool-flame dodecane-droplet extinction diameters", Combust. Flame 212 (2020)
    242–244 (NSF PAR https://par.nsf.gov/servlets/purl/10176660) — F.
18. [YS14] F. Yu, B.D. Shaw, "Interpretation of Backlit Droplet Images from ISS Droplet Combustion Experiments",
    Gravit. Space Res. 2 (2014) 82–93, doi:10.2478/gsr-2014-0007 — C, G.
19. [W26] K.A. Waddell, D.L. Dietrich, V. Nayagam, "The effect of pressure on radiative losses in hot, warm, and cool
    microgravity spherical diffusion flames", 2026, NTRS 20260002014 — C.
20. [ISS13] V. Nayagam, "Cool Flames in Space, a Hot Prospect on Earth!", 2nd ISS R&D Conf., 2013 — F.
21. [PSI] NASA Physical Sciences Informatics, PSI-69 investigation record,
    https://psi.nasa.gov/physci/repo/data/investigations/PSI-69 (metadata via psi.nasa.gov/geode-py/ws/repo/search) — C, D.
22. Papers known but not read (paywalled): T.I. Farouk & F.L. Dryer, Combust. Flame 161 (2014) 565–581,
    doi:10.1016/j.combustflame.2013.09.011; M.C. Hicks, V. Nayagam, F.A. Williams, Combust. Flame 157 (2010)
    1439–1445, doi:10.1016/j.combustflame.2010.05.007 (abstract read); T.I. Farouk & F.L. Dryer, Combust. Flame 159
    (2012) 3208–3223, doi:10.1016/j.combustflame.2012.04.005; Farouk & Dryer, C&F 159 (2012) 200–209 (tethered
    methanol, CO2); G. Paczko, N. Peters, K. Seshadri, F.A. Williams, Combust. Theory Model. 18 (2014) 515–531,
    doi:10.1080/13647830.2014.934296 (abstract read); V. Nayagam, C&F 160 (2013) 2638–2640 (AEA methanol);
    V. Nayagam et al., AIAA J. 54 (2016) 1235–1239, doi:10.2514/1.J054437 (abstract read); Y.C. Liu, Y. Xu,
    M.C. Hicks, C.T. Avedisian, C&F 171 (2016) 27–41; B.D. Shaw & F. Yu, MST 26 (2014) 89–99,
    doi:10.1007/s12217-014-9377-x; D. Shringi, H.A. Dwyer, B.D. Shaw, MST 22 (2010) 17–26 (needle retraction);
    A.J. Marchese & F.L. Dryer, C&F 105 (1996) 104–122; Zhang, Card, Williams C&F 105 (1996) 267–290;
    A. Cuoci et al. (2013, 2017); Robbins & Shinn, MDC-DOC-1790A (2010); Banu, CIR-DOC-4064 (2008).
