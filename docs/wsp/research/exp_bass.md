# Dossier: NASA BASS (PSI-26) and BASS-II (PSI-25) on the ISS

Purpose: what can legitimately be measured from the BASS/BASS-II image record, and what physics a hand-built forecast of flame/front position versus time has to respect.

Conventions:
- Every number carries a source tag in square brackets. The tags resolve in section I.
- **[INF]** marks my own inference. Everything else is stated in the cited source.
- Where sources disagree, both values are given.
- Per-test result tables (BASS-II TM Appendix A, TM Table 5.1, TM Table 7.1, PSI experimental table) were read but are **not** transcribed. Only ranges, correlations and procedures are reported.

Source tags, in short:
- [TM] NASA/TM-20210011385, the BASS-II Summary Report (Oct 2021). The page numbers in tags are TM page numbers.
- [POS14] NTRS-20160000593, the 2014 ASGSR poster. This is the local PDF.
- [ICES15] ICES-2015-196.
- [ICES19] ICES-2019-101.
- [ICES16] ICES-2016-387.
- [SR18] Link et al., Sci. Rep. 2018.
- [ZH15] Zhao et al., 2015 US National Combustion Meeting (NTRS 20150008962).
- [OF15] Olson & Ferkul, 2015 US National Combustion Meeting (NTRS 20150008961).
- [POS12] NTRS 20130012832, the 2012 ASGSR fabric poster (BASS-I).
- [POS13] NTRS 20140011099, the 2013 ASGSR BASS-I poster.
- [FS] NASA BASS / BASS-II / BASS-M fact sheet (nasa.gov PDF).
- [SRD] BASS-II Science Requirements, a PDF on PSI-25.
- [PSI25] and [PSI26] are the PSI landing pages.
- [CAR19] Carmignani, PhD dissertation, UCSD/SDSU 2019.
- [DEB12] DeBold, MS thesis, UMD 2012 (covers the SPICE hardware).
- [GN14] NTRS 20140009169.

---

## A. Apparatus

**Heritage**
- BASS and BASS-II reused the on-orbit SPICE (Smoke Point In Co-flow Experiment) hardware with slight modifications. SPICE was launched on STS-126 [FS; SRD p.9; POS12].
- The SPICE/SLICE duct was "based on the preceding ELF experiment" [GN14].
- The hardware ran inside the Microgravity Science Glovebox (MSG) in the US Destiny Lab [TM p.3].
- MSG working volume:
  - 0.255 m³, of which BASS hardware takes ~0.005 m³, leaving a net ~0.25 m³ [TM App. B p.115].
  - Elsewhere quoted as a "250-L free volume" [TM p.66] or "255 L" [TM p.55].
- The MSG has filter banks and an ambient-temperature CO→CO2 catalyst [TM p.3].

**Flow duct**
- Black-anodized rectangular duct with rounded internal corners, 7.6 × 7.6 cm in cross-section [TM p.4, p.66; ICES15; SR18 gives "76 mm by 76 mm"].
- Length:
  - The test section is 17 cm long [TM p.4; SR18; ICES15]. TM p.66 calls it a "7.6 by 7.6 by 17 cm" duct.
  - The SPICE thesis gives the whole duct as "approximately 200 mm in length" [DEB12 p.18–19].
  - A web summary of a CFD paper also quotes 20 cm. **Disagreement:** 17 cm is the test section, ~20 cm is the full duct [INF].
- The duct was originally built for gas-jet diffusion-flame studies (Dotson et al. 2011, SPICE) [TM p.4].

**Fan and flow path**
- A variable-speed fan sits at the upstream end. Up to two flow restrictors could be fitted at the fan inlet [TM p.4, p.66].
- Maximum velocity is ~55 cm/s with no restrictor and ~35 cm/s with two [TM p.4; ICES15].
- Other quoted ranges:

| Range quoted | Source |
|---|---|
| "up to 40 cm/s" | [POS14; POS12; POS13; PSI25 approach text] |
| 0–55 cm/s | [POS14 hardware slide; ICES19] |
| 0–50 cm/s | [TM p.21, p.37–38; ICES16] |
| 5–50 cm/s, crew-adjusted | [FS] |
| 1–55 cm/s for the fabric tests | [TM p.94; ZH15] |
| 0.4–8 cm/s for the opposed rods | [SR18] |

- Downstream of the fan, the flow passes a honeycomb flow straightener, then an inlet screen to reduce swirl [TM p.4; ICES15; SR18]. The SPICE thesis calls the straightener a "ceramic flow straightener" [DEB12].
- The exit has a perforated copper plate, then a brass screen (heat rejection and soot trap). The flow then turns 90° to leave the duct [TM p.4].
- Measured inlet profile: "the measured flow profile for the BASS-II duct is quite irregular" [TM p.60, Miller's FDS modelling].

**Velocity measurement and calibration**
- Sensor: an omnidirectional spherical air-velocity transducer (TSI 8475) between the honeycomb and the screen [TM p.4, p.66].
  - It is described as a "hot wire anemometer" in [TM p.21; SR18; ICES16].
- Response time is ≈1 min [TM p.4, p.66].
- The display shows units only, giving an effective resolution of ±0.5 cm/s [TM p.66].
- Calibration:
  - The fan voltage was calibrated against the transducer at the end of every operations day, per flow restrictor, after fan warm-up [TM p.4, p.66; SR18].
  - Test velocities were then taken from the fan voltage through that calibration [TM p.21; ICES16].
  - A "Fan calibration data.xlsx" exists on PSI-25 (Reference Data) [PSI25]. **Not read.**
- The SPICE-era AIR-display calibration was v[cm/s] = −0.0044x² + 1.2899x − 11.828, done in 1 g, with "minor day-to-day variation" [DEB12 p.22–23]. This is SPICE, not BASS; use it only as an indication of the instrument behaviour.
- Low-velocity accuracy:
  - "significant uncertainties … calibration at such low velocity has high error bars" [TM p.13].
  - There is a time lag between a fan change and the new velocity reaching the flame [TM p.13].
  - The test plans and matrices give low flows as fan-potentiometer settings (e.g., "pot 0.35") [TM App. A].
- Measurement accuracy requirement: ±10%, met by SPICE before launch [SRD p.1].

**Video overlay** (on the video camera only)
- The overlay shows N2 flow (cm³/min), fan voltage (10× V), transducer reading (cm/s) and radiometer signal [TM p.4–5; ICES15; SR18].
- SPICE/SLICE also overlaid a time stamp [GN14 p.2].
- The overlay acquisition rate of 4 Hz "can not be changed from SPICE/BASS" [SRD p.1].

**Radiometer**
- Oriel 71768 thermopile with a CaF2 window, 0.13–11 µm [TM p.4].
- Mounted in the downstream top back corner of the duct, looking upstream [TM p.4]. It sits about 20° from the sample midpoint [CAR19 p.16].
- Not calibrated. It gives relative flame strength [TM p.5, p.66].
- Read at 5 Hz [TM p.66]. The SRD says 4 Hz for overlay data [SRD]. **Disagreement noted.**
- The sensor responds with a delay of a few seconds [CAR19 p.16].
- Gain settings were 1000, 250, 50 and 10 [TM App. A column headings].
- The radiometer was used to confirm extinction when the flame was not visible [TM p.21].

**Nitrogen system**
- BASS-I suppression nozzle:
  - N2 suppressant from a mass-flow controller at 0–500 cc/min [FS].
  - The N2 was applied at a gradually increasing level until extinction [FS].
  - A 500 cc/min jet failed to extinguish a thick PMMA sphere flame [POS13].
- BASS-II:
  - The same nozzle line, an MKS 179A meter (<0.5 LPM) that "entered the flow duct through a small nozzle just downstream of the fan", was used to dilute the MSG working volume [TM p.6].
  - ISS GN2 was supplied through the MSG GN2 interface at 0–500 sccm [SRD p.1].
  - The TM also lists a "nozzle for N2 flow" inside the test section [TM p.4]. **Disagreement or ambiguity on nozzle location:** possibly the same nozzle described differently [INF].

**Oxygen control**
- BASS (PSI-26, 2012–2013):
  - Fabric and most tests ran in ISS cabin air, about 21% O2 [ZH15: the four BASS fabric tests "all 21% O2"].
  - **Disagreement:** the 2013 BASS poster lists "ambient oxygen atmospheres 17% to 21.5% (cabin air)" and an operation step "Nitrogen vitiation (if desired) down to 16% O2", and shows PMMA spheres at 17% O2 [POS13].
  - The TM and ICES-2015 say "For the first time on ISS, BASS-II utilized MSG working volume dilution" [TM p.1].
  - The dilution model's on-orbit calibration test was on GMT 2013-095 [TM App. B p.115], during the BASS-I period.
  - [INF] Some late BASS-I sphere tests probably used early vitiation. Treat BASS-I O2 as 21% unless a record says otherwise.
- BASS-II:
  - The MSG working volume was diluted with ISS N2.
  - O2 ranged from ambient ISS (~21–22.2%) down to ~14%, and 13.6% final in the lowest test [TM p.6, p.64].
  - [ICES19] gives 13–22% overall. Other ranges: fabric 16–21% [TM p.94]; thin films 15–22.2% [CAR19 p.16]; opposed rods 16.8–20.9% [TM Table 4.2]; slabs 18–22.2% initial [TM p.56–57, range only].
- Dilution procedure:
  - Dilution took 1–3 h [TM p.6].
  - Rate: <0.5 LPM N2 (0.000493 m³/min in the calibration), with the duct fan at maximum (>100 LPM) to mix [TM p.6; App. B].
  - Model: c(t) = C0·exp(−r t/V), V ≈ 0.25 m³ [TM App. B eq. B.2].
- O2 measurement:
  - Measured only before and after each burn, with the CSA-CP, 0.1% O2 resolution [TM p.6].
  - Offset-corrected daily against the ISS MCA mass spectrometer. In the calibration example the CSA-CP read 20.9% against the MCA's 21.48% [TM App. B].
- O2 changes within and between burns:
  - O2 fell during each burn, because the products stay in the sealed 250-L volume [TM p.6, p.67].
  - Thin films: drift <0.1% within a test [TM p.37–38].
  - Long thick-slab tests (up to >30 min): O2 "dropped somewhat during each test" [POS14; TM p.58].
  - Miller's group assumed a linear O2 decay between the start and end readings [TM p.60].
  - Repeated tests in the same sealed volume ratchet O2 down until a purge [TM p.6].
  - Stated O2 uncertainty: ±0.3% [SR18 Fig. 7].

**Pressure**
- 1 atm nominal; pressure was not varied [ICES19].
- Total pressure monitored at 99.2–99.5 kPa in the thin-PMMA tests [TM p.38]. "101 kPa" in [CAR19 p.16].

**Igniter**
- Kanthal A-1, 29-gauge hot-wire coil [TM p.5].
- Electrical values (**disagreement**):

| Values | Source |
|---|---|
| 0.8–1 Ω, 3.5–3.8 A | [TM p.5; ICES15] |
| 1 Ω on a 4-A circuit | [TM p.67] |
| 1.1 Ω at 5 V | [SR18] |
| ignition energy ~16 W | [TM App. D] |

- Deployment: the astronaut pulls a lever to swing the igniter into position [TM p.5].
- Flat samples: the igniter is integral to the sample assembly ("Igniter Frame 2-pin connector") [SRD p.10]. The As-Run matrix records "Integral" [TM App. A].
- Reused or partially burnt samples were re-ignited with a retractable igniter on one side, at the fuel-burnout position [TM p.95].
- Position:
  - Opposed tests: igniter near the duct outlet, i.e. the **downstream** end of the sample.
  - Concurrent tests: igniter near the inlet, i.e. the **upstream** end [ICES19].
  - Rods (opposed): ignited at the downstream end [TM p.47, p.51].
  - Stagnation rods (concurrent): igniter hook placed within ~1 mm of the hemispherical tip [TM p.67–68].
- On-time:
  - Thin films: igniter on until a visible flame appears, then off [TM p.21]; 5–10 s [CAR19 p.16].
  - Rods: ignition took up to 230 s, with repeated on/off cycles [TM p.68].

**Containment and extinguishment**
- The assured way to extinguish was to turn off the duct flow [ICES19 p.2].
- Flames "always quenched when the fan was turned off" [ICES19].
- Cool-down between tests: at least 9 min [TM p.67]; 10 min for thick samples [TM App. D].

---

## B. Samples

**BASS (PSI-26)**
- Counts: 41 samples; 89 test points planned [FS]. "41 samples, 115 burns completed" [POS13].
- Categories: thin flat, spheres, thick flat and candles. 17 thin flat, 12 spheres, 4 thick flat and 8 candles [POS12; POS13 hardware slide].
- SIBAL fabric (BASS):
  - 75% cotton, 25% fiberglass; simple weave of 60 × 40 threads per inch [POS12].
  - Areal density 18 mg/cm² [POS12].
  - Width 1 and 2 cm [POS12].
  - Length conflicts:

| Length | Source |
|---|---|
| 8.5 cm ("Fuel sizes: 8.5 cm length") | [POS12] |
| 12 cm ("Thin flat samples (12 cm long by 1 and 2 cm wide)") | [FS] |
| 10 cm exposed | [ZH15] |

  - [INF] The 12 cm is probably the overall holder or fabric length; 8.5 cm or 10 cm is the burnable length.
- Other thin materials: Nomex (HT90-40, 12 mil fabric; Nomex-III) and Ultem 1000 (10-mil film) [FS; POS13].
- Thick flat: PMMA and wax-saturated fiberglass, 5 cm long × 1 cm wide × 1 and 2 mm thick [FS].
- PMMA spheres: 1 and 2 cm diameter, igniting either the front or the back [FS]. A "15-mm initial diameter" sphere is also shown [POS13]. One 1-cm sphere had a mass of 617.5 mg [TM App. D].
- Candles: inside a thin ceramic tube 6 mm in diameter × 25 mm long. Paraffin and "Japan wax" [FS].

**BASS-II (PSI-25)**
- Counts: 100 samples (≈20 per PI) plus 3 Nomex-III, and 27 spare igniters [SRD p.1].
  - 126 tests run, more than the 103 planned, because some samples were reburned [TM p.3].
  - The PSI-25 experimental table lists 130 rows [PSI25].
- SIBAL fabric:
  - 75/25 cotton/fiberglass; thickness about 0.32 mm; areal density 18.2 mg/cm² [TM p.94; ZH15].
  - Widths: 1.2 and 2.2 cm. Exposed length 10 cm [TM p.94].
  - Manifest: 15 of 2-cm and 4 of 1-cm SIBAL ("previously flown design") [SRD p.4].
  - [INF] "1/2 cm" is the nominal sample width; 1.2/2.2 cm is how the investigators report the same samples. Holder drawings were not read (see section I, not found).
- Thin PMMA films (cast PMMA):
  - Thicknesses 0.1, 0.2, 0.3 and 0.4 mm [SRD p.5]. "100–400 µm" [TM p.13, p.21].
  - Widths 20 mm and 10 mm. Bhattacharjee's set: 2 cm, with one 1-cm set at 0.2 mm [SRD p.5]. Also reported as 1.2/2.2 cm [ICES19].
  - Length conflicts: "about 95 mm" [TM p.21, p.37]; "about 105 mm" [ICES16]; "about 100 mm" [CAR19].
  - TM summary text says slabs range "0.125 to 5 mm" [TM p.5]. [ICES19] says 0.1–5 mm.
- PMMA slabs (Miller/Wichman, clear cast):
  - 1, 2, 3, 4 and 5 mm thick; 10 cm long; 1 or 2 cm wide (reported as 1.2 and 2.2 cm); burnt one-sided or two-sided [TM p.55–56].
  - One-sided samples sit on a metal substrate holder ("heat loss to the metal substrate") [TM p.58].
  - App. D's worst case is a 0.5-cm thick × 8.5-cm long × 2-cm wide slab [TM p.121].
- PMMA rods (cast, black or clear, initially hemispherical ends):
  - Diameters 6.35, 9.525 and 12.7 mm (¼, ⅜ and ½ in) [TM p.47; SRD p.5].
  - Length 57.15 mm [TM p.47]. 57.2 mm [SR18]. Olson's initial rod length L0 = 5.9 cm [TM p.64].
  - Olson's analysis uses radii 0.318, 0.476 and 0.635 cm [TM p.67].
  - **Caution:** some figure captions give radius as if diameter: "4.76 mm" and "3.18 mm" rods [TM Fig. 4.4; ICES19 Fig. 2]. [SR18] writes "0.64mm diameter" for 0.64 cm.
  - Black PMMA was blackened with a little soot during polymerisation [TM p.67].
- PMMA spherical section: one 4-cm-diameter section, used for forward-stagnation ignition and flame growth [SRD; TM p.5].
- Wax candle: one leftover BASS-I Japan-wax candle was burnt in BASS-II [TM p.46; App. A].
- Nomex-III: 3 samples, 2 cm wide. None sustained a flame in BASS-II [TM p.46].

**Holders and mounting**
- Holders: stainless steel [TM p.94].
- The top window carries the solid-fuel mounting rail. Each Solid Fuel Assembly attaches magnetically so the crew can install and remove it [SRD p.11].
- The sample holder is on the median plane of the tunnel [CAR19 p.16].
- Thin films are taped to the holder; the U-shaped pyrolysis edges show this [ICES16].
- "Sample distance" is an As-Run matrix column [TM App. A]. Its meaning was not stated (**not found**). [INF] It is probably the sample's position along the rail.
- Samples can be flipped 180° for wake ignition [POS14; POS12].
- Holder frame dimensions: **not found** in text. Drawings exist on PSI-26 but were not read: "Drawing_Flat.Sample.Frame_2cm", "Flat.Sample.Frame_1cm", "SolidFuelFrame", "PMMA.Flat.Fuel.Sample_2cm", "Sibal.Flat.Sample", "Blk.PMMA.Rod", "PMMA.Spherical.Section", and others [PSI26 file list].

**Physical scale references visible in images**
- A **movable scale** sits inside the test section [TM p.4; ICES15; SR18]. It is "a moveable scale to provide a reference at the sample plane" [OF15].
- The known sample width (1 or 2 cm; 1.2/2.2 cm as reported), rod diameter and holder edges are usable references [INF].
- The "dots along the axis, 6 mm apart" in the SDSU figures were **superimposed during processing**. They are not in the raw images [TM p.22, p.39].

---

## C. Cameras and geometry

**Digital still camera (top view)**

| Item | Value |
|---|---|
| Model | Nikon D300s, 12.3 MP colour [TM p.4]. "D300" in [ICES15; SR18; OF15]. Requirement: "The D300s still image requirements are the same as for BASS" [SRD]. BASS-I/SPICE used a Nikon D100 [DEB12] |
| Image size | 4,320 × 2,968 px [TM p.4] |
| Lens | 60-mm lens [TM p.4; SR18; ICES15]. **Disagreement:** "35 mm f/2.0 lens", CMOS 23.6 × 15.8 mm [CAR19 p.16] |
| View | Through the top window, normal to the flat-sample surface. This is the face-on view of fabric and films [TM p.4; TM p.56 Miller: "looks through the top of the flow duct normal to the sample surface"; TM p.94 T'ien: "top-view digital camera looked down on the top fuel surface"] |
| Pixel scale | **34 pixels/mm in the sample plane** [SR18, rod tests] |
| Field of view [INF] | From 34 px/mm and 4,320 × 2,968 px: ≈127 × 87 mm. That covers the ~10-cm sample length, not the full 17-cm test section |
| Cadence | ~1 frame/s [TM p.21, p.38; ICES16; SR18 "about every 1s"; TM p.66 "~1 frame per second"]. Fabric tests: 1.125 s and 1.25 s [POS12]; "about one image every 1.2 s" [TM p.94]. Spheres and slab: 1.3 s, sphere 1.5 s [POS13] |
| Exposure | 1/8 to 1/4 s for fabric, long enough to catch dim blue near-quench flames [TM p.95] |
| Settings recorded per test [TM App. A columns] | Release mode CH/CL (mostly CL, continuous low); exposure mode M/A (mostly A = automatic, with −3 to −5 EV compensation); bracketing; shutter speed; F-stop (recorded as "F2"); ISO (mostly 1,000; 1,600 in one); shutter hold (continuous); SD card serial; total frames shot. Manual 1/10 s was used for the Nomex tests |
| Card limit | "still camera card limit of 7 min" [TM App. D p.123]. The SRD provides 8 extra flash cards [SRD]. [INF] Very long burns can have gaps in the stills |
| Triggering | The crew triggers the still camera manually [SRD p.2]. The ground called "still camera imaging adjustments" in real time [TM p.67] |

- Investigators converted the still sequences to videos (posted at flame.sdsu.edu) and analysed those [TM p.39; CAR19 p.80].
- PSI-25 holds "…_Still_Sequence.mp4" files [PSI25].

**Video camera (side or edge view)**

| Item | Value |
|---|---|
| Model | Panasonic WV-CP654 colour camera, 760 × 480 pixel array [TM p.4–5; ICES15; SR18; CAR19]. **Disagreement:** Miller calls it "a high-definition video camera" [TM p.56]. [INF] The WV-CP654 is a standard-definition analog CCTV camera |
| Frame rate | 30 fps [TM p.66]. 29.97 fps, "standard video framing rates" [TM p.94; ZH15] |
| Optics | Looks in through the front window via a turning mirror [TM p.4–5] |
| View | Edge-on view of flat samples, orthogonal to the still camera [TM p.56: "uses a mirror to look through the front door of the duct at the edge of the flat sample"; TM p.94: "side-view video camera provided edge-on images of the flat samples"] |
| Field of view | "The field of view of the video can not be changed from BASS" [SRD]. FOV in cm: **not found** for BASS |
| Proxy scale | The SPICE-era ruler calibration of the uncompressed video was 336.02 px per 70 mm (≈4.8 px/mm) [DEB12 p.26]. Gas-jet configuration, before BASS. [INF] Do not assume it holds for BASS without re-deriving it from a known width or the scale in frame |
| Overlay | N2 flow, fan voltage, air velocity and radiometer [TM p.5]. SPICE also overlaid a time stamp [GN14] |
| Date quirk | The As-Run matrices carry the heading "Actual date (for video display date, add 1 for Leap Year)" [TM App. A]. [INF] The overlay date can be off by one day from the true date |

- On the flame-edge view of thin samples, flames on both faces are visible. The video shows the two-sided flame growth that T'ien tracked [TM p.95].
- Once note records "not quite a straight on edge view" for one test [TM App. A].

**Which view for which geometry**
- Flat fabric and films:
  - Top still = face-on, showing width structure and the pyrolysis front.
  - Video = edge-on, showing flame thickness and both faces, but no across-width structure [TM p.94–95].
- Rods and spheres: both cameras show side profiles from orthogonal directions [INF from the orthogonal-window description].
- Do both views exist for every test? Not always:
  - "No stills taken, cable came loose" in one test [TM App. A].
  - Video was intermittent after GMT 167 (2014) until the cable repair on GMT 170 [TM App. C].
  - PSI-25 file naming shows a primary "…mp4", often a "…_2ND_VIEW.mp4", sometimes a "…_Still_Sequence.mp4", and some "…_LOS_Portion…" files (loss of signal) [PSI25].
  - [INF] Which camera is "primary" versus "2ND_VIEW" is **not documented** in what I could read. Check the PSI-25 ReadMe.docx, which was **not read**.

**What appears in frame** (stated or illustrated)
- Igniter wire glowing red at the ignition end. In one SDSU top view it is at x = 36 mm, with burnout at x = 42 mm [TM p.22].
- Sample holder edges and the pyrolysis/burnout front: a black line on the top view [TM p.22].
- Sample-holder side losses: unburnt fuel strips along the holder [TM p.98].
- Bubble layer in PMMA [TM p.58, p.71].
- Bright smoulder spots behind the fabric flame base [TM p.95].
- Black duct backdrop ("low-reflection backdrop") [TM p.66].
- The movable scale [TM p.4].
- Orientation:
  - Flow runs right to left in the published top and edge views [TM p.95–99; TM p.21].
  - In the TM summary montage "forced flow is up in all images" [TM Fig. 1.2]. This is a rotated presentation.

**Length scale used by researchers when digitising**
- Pixel scale from the camera geometry: 34 px/mm for the stills [SR18].
- SDSU's analysis expresses position in mm along the sample axis, with 6-mm grid dots added for display [TM p.22, p.39].
- SPICE-era video was calibrated with a ruler shown before the tests [DEB12].
- Not found: an explicit statement that BASS analysts calibrated against the movable scale or the sample width. [INF] The movable scale and known sample width are the legitimate in-frame physical references.

---

## D. Test procedure and timeline

**Operations**
- The crew installs the hardware in the MSG and makes the connections [FS].
- Before each test point, the crew:
  - installs the sample and igniter;
  - adjusts the fuel location and igniter position within the camera fields of view [SRD p.2];
  - sets the flow restrictor and fan.
- In BASS-II, O2 was set by dilution (hours) before the test [TM p.6].
- Real-time "lab partner" operations:
  - The ground PI team watches the downlinked video and calls flow changes, still-camera settings and ignition instructions [TM p.67; FS].
  - Instructions came from the NASA Glenn Operations Center [TM p.21].
- Sensor readings (O2, CO, CO2) were taken before and after each burn [TM p.6].

**Generic sequence**
1. Set flow and let it stabilise [TM p.21].
2. Ignite; the igniter is on until a visible flame appears, then off. Repeated on/off cycles are possible [TM p.68].
3. The flame spreads.
4. Flow is changed in steps during the burn.
5. The test ends by one of:
   - quench, by turning the flow down or off;
   - blowoff, by turning the flow up;
   - burnout at the sample end;
   - N2 jet (BASS-I);
   - the fan turned off.

**BASS-I with the N2 jet**
- The flame burns for "about a minute", then the crew raises N2 until extinction [FS].

**Flow steps** (typical; this is the core BASS-II procedure)
- "The nominal test procedure for most samples was to turn the flow down in increments" [TM p.7].
- Thin PMMA (Bhattacharjee):
  - Establish steady spread, then reduce the flow "gradually", or cut it off abruptly. With an abrupt cut, extinction came within 2 s [TM p.13].
  - "In some of the tests, the flow velocity is changed during the spread to maximize the test matrix" [TM p.21].
  - Flow was often changed "before halfway through the propagation" [ICES16].
- Opposed rods (Fernandez-Pello / Link):
  - Ignite at a relatively high opposed flow; reach steady spread within 1 min.
  - Then reduce stepwise until extinction, pausing "about 30 s" at each step [SR18], or "at least 1 min" per step [SR18 Results]. **Internal inconsistency in [SR18].**
- Stagnation rods (Olson):
  - Ignite at ~5 cm/s.
  - Reduce in increments, holding each for at least 100 s so the solid temperature equilibrates [TM p.69].
  - Quench tests lasted up to 28 min [TM p.69].
  - For blowoff, increase the flow quickly (the plan: "increase flow every 5–10 seconds") [SRD p.12; TM p.73].
- Slabs (Miller):
  - Flow adjusted one or more times. Some tests lasted >30 min. Burn times ranged ~6–31 min [TM p.56–57, range only].
- Fabric (T'ien):
  - Ignite at ~10 cm/s, then reduce. "The rate of velocity change varies from case to case depending on the communication between the astronauts and the ground crews and the manual adjustment time" [TM p.95].
  - Total burn ~40 s at 10 cm/s and >60 s at 5 cm/s [TM p.95].
- Candle (Ferkul):
  - Started at 3 cm/s, then stepped down potentiometer settings 0.6 → 0.4 → 0.35 → 0.32 → 0.31 [TM p.46].

**Test naming and records**
- Each PI's tests carry a letter-coded test number: B (Bhattacharjee), F (Ferkul), P (Fernandez-Pello), M (Miller), and others for Olson and T'ien. There is also an "As-run test number", a sample number, and the date with GMT day-of-year [TM App. A].
- T'ien's naming is "GMT<day-of-year>-T<n>" [TM Table 7.1].
- PSI-25 video files: "BASS_GMT<doy>_SAMPLE_<n>_TEST_<id>[ _2ND_VIEW | _Still_Sequence | _LOS_Portion_<times> ].mp4" [PSI25].
- PSI-26 (BASS-I) raw data:
  - MP4s named by date and time plus ISS photo ID.
  - Daily ZIPs suffixed "_orig", "_xmp" and "_txt" (still originals, XMP sidecars, text) [PSI26].
- The As-Run matrix columns [TM App. A] are:
  - date/time (video display time)
  - As-run #, test #, sample #
  - material
  - igniter, flow restrictor
  - sample distance
  - fan display, air display
  - radiometer gain and display
  - GN2 pot and display
  - filter
  - camera release mode, exposure mode, EV compensation, bracketing, shutter, F-stop, ISO, shutter hold
  - SD card S/N, total frames shot
  - calibrated and raw initial/final O2, initial/final CO2 and CO
  - free-text observations: ignited, spread, extinguished, burn time, colour, flow sequence as pot settings
- The PSI-25 experimental table has a subset of these columns [PSI25].
- Sample reuse: partially burnt samples were reburned, sometimes with the burnt fuel cut off or igniter leads removed [TM p.10, p.95].

---

## E. Published data reduction

**SDSU "Flame Analyzer" / FIAT (thin PMMA, top-view stills)** [TM p.22, p.39; CAR19 p.18–21]
1. Take the top-view RGB image.
2. Isolate a central strip ~14 mm wide, "to eliminate the effect of edge propagation".
3. Convert to an intensity value. FIAT uses the luminance Y of YCrCb.
4. Average across the width to get a 1-D intensity profile along the axis.
5. Leading edge = location of a fixed threshold. A threshold of **30** worked for almost all BASS tests (8-bit). In one 1-g test, **70** was used.
6. Also tracked: the peak-intensity location ("bright edge"), the trailing edge, flame length (leading to trailing edge), and blue/yellow areas by RGB colour windows.
7. Spread rate = slope of a linear regression over a number of consecutive positions. **Five consecutive locations (≈5 s at 1 fps)** was the chosen compromise.
8. Reproducible within 5%, whether tracking the leading edge or the peak. Uncertainty <5% "except near ignition time when the flame bursts into its spreading form".

Error model [CAR19 eq. 2.7]: dVf/Vf = 1/(p·Vf·Δt) + 1/(f·Δt), with p = px/mm, f = fps and Δt the averaging window.

**Fabric (T'ien)**
- Side-view video tracking of flame tip, flame base and flame length [TM Fig. 7.2].
- When the side view is too dim (low flow or O2), top-view still tracking instead [TM p.95].
- "Steady flame spread" = constant spread rate for >20 s [TM p.95].
- Quenching sequences track flame length, flame width and flow versus time [TM Fig. 7.6].

**BASS-I fabric poster**
- Still-image tracking of flame tip and base position (mm) against time (s), with linear fits [POS12].

**Slabs (Miller)**
- Stills and video tracked with NASA Spotlight-16, then exported to Excel.
- Position versus time is split into constant-flow segments; a spread rate is given for each flow [TM p.56].

**Opposed rods (Link)**
- Leading edge tracked from the ~1-s stills with an "imaging tracking code".
- Linear position versus time indicates quasi-steady spread.
- Multivariate linear regression of Vf on O2, Vg and diameter [SR18].

**Stagnation rods (Olson)**
- From the high-resolution stills: flame length, width, standoff (measured to the outer boundary of the bright cyan zone), aspect ratio and rod-tip position versus time.
- Regression rate from the tip position.
- Radiometer used for oscillations [TM p.69–73].

**Flame radiation, thin PMMA (Carmignani)**
- Radiometer and anemometer read from the overlay on the first frame of each second of the side video.
- Because of the ~1 min anemometer lag, flow changes were timed by the astronauts' voice confirmations.
- Time profiles were converted to position along the sample [CAR19 p.80].

---

## F. Physics as observed (with numbers)

### F1. Opposed flow over thin PMMA films (0.1–0.4 mm)

**Regimes**
- Thermal: Vf is independent of Vg and ∝ 1/thickness.
- Radiative: at low Vg, Vf falls as Vg falls, down to quench.
- Kinetic: at high Vg, Vf falls, down to blowoff [TM §2.2–2.3; ICES16].

**Maximum spread rate**
- 2.2 mm/s for 100 µm, 21% O2, over Vg ≈ 1–40 cm/s. This matches the 1-g downward value of 2.0 mm/s [TM p.40–41].
- Thin-limit formula (de Ris with the π/4 factor) [TM eq. 2.11–2.12]:
  - Vf,thin = (π/4)·λg F/(ρs cs τ) = 2.34×10⁻⁸·F/τ, in m/s, with τ the half-thickness in m.
  - F = 4.76 fitted from experiment (21% O2); Fadb = 4.89.
- [INF] Implied 21% maxima: ≈2.2, 1.1, 0.74 and 0.56 mm/s for 100, 200, 300 and 400 µm.

**Critical (quench) velocity**
- Independent of thickness [TM p.13, p.23–25, Table 2.1]:

| O2 | Experiment | Computation | Theory eq. 2.5 |
|---|---|---|---|
| 21% | ≈1 cm/s (10 ± 5 mm/s); ~1.5 cm/s at 400 µm | 15 mm/s | 7.5 mm/s |
| ~16% | 2.6 cm/s (100 µm, 20 mm wide); 4 cm/s (200 µm, 10 mm wide) | 35 mm/s | 10 mm/s |
| 16% (Table 2.1) | 30 ± 5 mm/s | | |

- Closed form: Vg,cr ≈ (εσ/(cg ρg))·(Tv⁴ − T∞⁴)/(Tf − Tv) [TM eq. 2.5].
- Computation at 21%: 1D quench at 1.5 cm/s for all four thicknesses [TM Fig. 2.11].
- Downward 1-g computed: 2.8, 1.6, 1.2 and 0.9 mm/s for 100–400 µm [TM p.26].

**Kinetic and blowoff**
- Computed: the radiative regime starts below ~20 cm/s at 21% [ICES16; CAR19]. At 16% the kinetic effect sets in above ~16 cm/s [TM p.26].
- Measured blowoff: 28 and 33 cm/s at 17.3% and 17.4% O2. No blowoff at 45 cm/s in 21% [ICES19 p.3].
- The test matrix notes "did not blowoff" at 10 cm/s max [TM App. A].
- Computed blowoff depends on boundary-layer development length xd: 55 cm/s at xd = 20 mm and 111 cm/s at xd = 60 mm [ICES16].

**Path to quench**
- The flame shrinks, loses orange soot, and turns all blue. It went out within ~3 s of the flow dropping from 3 to 1.1 cm/s [TM Fig. 2.1].
- The radiometer falls and then shows a slope break at extinction.

**Boundary-layer effect**
- The opposed flame runs toward the upstream sample edge, so xd decreases as it spreads [INF from geometry].
- Flame length increases as xd decreases, and is larger at higher Vg [ICES16; CAR19 Figs 5.5–5.6].
- A spread-rate rise was seen only at 10 cm/s: 1.5 → 2.2 mm/s along the sample [ICES16].
- Computed Vf rises only mildly, 2.3 → 2.7 mm/s, at Vg = 5 cm/s [ICES16].

**Width**
- Halving the width (20 → 10 mm) shrinks the flame markedly; the radiometer peak drops ×3–6 [ICES16; CAR19].

**Timescales**
- Solid conduction 0.023–0.37 s against a gas-phase time of ~1 s.
- So the leading edge is pseudo-steady within the 15–40 s observation windows [CAR19 p.77].
- [INF] Thin-film fronts respond to flow steps within ~1–3 s.

### F2. Thin cotton-fiberglass fabric (SIBAL)

**Opposed (BASS-I)**
- The flame reached steady spread quickly at each flow. Spread was fastest at an intermediate flow (non-monotonic) [POS12; POS13].
- 2-cm fabric:
  - Spread at ~10 cm/s opposed: 1.21 mm/s, against 2.55 mm/s concurrent [POS12; POS13].
  - "Flame response to flow changes is very rapid" over steps from 10 → 6 → 4 → 2 → 1.5 → 1 cm/s [POS12].
  - Quench near 1 cm/s; total burn 90 s [POS12].
- In 1 g, downward (opposed) spread on this fabric is impossible in normal air [POS12].

**Concurrent: growth sequence**
- Ignition, then the flame grows, reaches a limiting length and spreads steadily, then decays at the sample end [TM p.95].
- 2.2 cm wide, 10 cm/s, 21%: limiting length reached at ~19 s; steady for ~9 s; whole burn ~40 s.
- 5 cm/s: steady ~21 s; burn >60 s; shorter, slower flame.
- 1-cm wide, 10 cm/s (BASS-I): steady after ~10 s. The flame tip and base advance at ≈1.7 mm/s, flame length ≈16 mm [POS12 tracking fit].

**Concurrent: spread-rate correlation** [POS12, BASS-I, 21% O2]
- Linear in flow speed: Vf ≈ 0.20·U + 0.56 (2-cm) and Vf ≈ 0.15·U + 0.07 (1-cm), with Vf in mm/s and U in cm/s.
- [INF] The poster plots concurrent flow as negative U, so the printed fits read "VF = −0.20U + 0.56" and "VF = −0.15U + 0.07". The magnitudes above reproduce the quoted 2.55 mm/s at 10 cm/s.
- BASS-II: spread is slower for the narrower sample, at lower flow and at lower O2 [TM Fig. 7.4, values not read].

**Concurrent: quench and blowoff**
- Quench velocity 1–5 cm/s, rising as O2 falls [TM p.93].
- 2.2-cm boundary: 2.2 cm/s at 18.7% to 3.2 cm/s at 16.9% [ICES19].
- Quench sequence: flame length and width both shrink to a circular blue flamelet, then extinction [TM p.98].
- Blowoff: 47 cm/s at 17.2% for 1.2-cm (one side first). No blowoff at 53 cm/s at 16.9% for 2.2-cm [ICES19].
- Non-ignition around 16.4–16.8% O2 [TM Table 7.1, range].
- The fabric does not burn away, so the flame spreads into a thickening boundary layer: lower base velocity downstream [ICES19 p.10].

### F3. Thick PMMA slabs (1–5 mm, opposed)

- Spread rates measured: 0.012–0.144 mm/s across 1–5 mm, flows of 2–21 cm/s, O2 ~17–22% [TM Table 5.1, range only].
- Trends [TM p.58, p.61–63]:
  - Vf falls with thickness and with lower flow.
  - Two-sided burns spread faster than one-sided.
  - 2-cm samples spread faster than 1-cm at 1 mm thickness.
  - One-sided samples need a higher flow to survive, because of heat loss to the substrate.
- Below ~19% O2, two-sided samples stop spreading over the surface. Instead the flame regresses and consumes the fuel from the trailing edge [TM p.58].
- BASS-I one-sided 1-cm slab at slow flow [TM App. D.7]:
  - Ignition took >1 min.
  - Steady size after ~3 min; flame only 15 mm long.
  - Vf = 0.037 mm/s; total test 7.5 min.
- Worst-case estimated growth to steady size: ~2.5 min [TM App. D].
- A 2-D FDS model over-predicts Vf [TM p.61].

### F4. Opposed PMMA rods (6.35–12.7 mm diameter)

- Vf rises with O2 and with smaller diameter. Microgravity spread on rods is faster than 1 g at comparable conditions [TM p.51].
- Vf against Vg is **non-monotonic**: it first rises, then falls. No spread below 0.6 cm/s [SR18].
- Sustained spread not seen below ~4 mm/s flow for the 6.35 and 9.525-mm rods [TM p.48].
- The flame is small, blue and open-tailed at low flow; yellow with a closed tip above ~20 mm/s [TM p.48].
- Minimum O2 for opposed rod spread: **conflicting**:

| Value | Source |
|---|---|
| 17% (μg) vs 18% (1 g) | [SR18] |
| 17.5% (μg) vs 19% (1 g) | [TM p.50] |
| 16.3% | [ICES19 p.8] |
| "not observed below 16.4%" | [TM p.73] |

- Magnitude of Vf: the TM averaged-rate plot has an axis spanning 0–3×10⁻² cm/s (0–0.3 mm/s) [POS14]. Individual values were not extracted.

### F5. Concurrent / stagnation-point PMMA rods (Olson)

**Flammability boundary** [TM eq. 6.1]
- X_O2 = 6·exp(−0.5a) + 0.0167a + 13.4, in %.
- a = (3/2)(U/r)(r/r0)^½ in s⁻¹, with r0 = 0.318 cm.
- The minimum is ≈13.5% at a = 8–12 s⁻¹.
- The blowoff branch is linear; the quenching branch is exponential.
- The transition from blowoff to quench happens near U ≈ U_D = 2 cm/s [TM p.88].

**Ignition**
- Ignition delay ≤~40 s at high O2, up to 230 s at low O2.
- No sustained ignition below ~14.5% O2, yet flames survive to 13.6% [TM p.68].

**Quench and blowoff**
- Quench comes >100 s after the last flow step, on the order of the solid response time ts = 0.7 r²/αs [TM p.70].
- Before quench, the radiometer shows oscillations at ≈1.1 Hz [TM p.72].
- Blowoff starts as a hole at the stagnation point (9–10 mm across) [TM p.73].

**Regression, standoff and bubble layer**
- Steady regression rate of 0.0385 mm/s in the lowest-O2 test [TM p.69].
- Standoff ∝ a^(−1/3); it increases sevenfold over two decades of a [TM p.77].
- Flame length minus standoff tracks the bubble-layer length. The flame adjusts "immediately" to a flow change; the bubble layer shrinks only at the tip-regression rate [TM p.72].

### F6. Spheres and candle (qualitative)

- 2-cm sphere at 17% O2 and 12 cm/s: high-resolution images allow flame-growth and regression measurement [POS13].
- 1-cm sphere: average burning rate ≈1.9 mg/s, about 0.3 mm/min regression [TM App. D].
- Candle: an elongated sooty flame at 3 cm/s becomes hemispherical, then short, wide and blue as the flow drops. It ejects wax droplets [TM p.46].

### F7. What a front-position forecast must account for [INF unless cited]

1. **Ignition transient.**
   - Igniter-on period of 5–10 s on films [CAR19]; up to minutes on rods [TM p.68].
   - Uncertainty is largest near ignition [TM p.39].
   - The flame then "bursts" into spreading form.
   - Concurrent flames grow to a limiting length over ~10–20 s [TM p.95; POS12].
   - Thick slabs take ~3 min to reach steady size [TM App. D.7].
2. **Steady spread** at a rate set by:
   - Vg, the effective boundary-layer velocity;
   - O2, which drifts down slowly through the burn;
   - thickness (thin: ∝ 1/τ);
   - width;
   - flow direction.
3. **Response to flow steps.**
   - Thin films and fabric: the gas phase responds almost immediately [POS13; POS12].
   - Thick fuels and rods: the solid response time is ~100 s [TM p.70].
   - The actual velocity change lags the fan command [TM p.13]. The anemometer reading lags ~1 min [TM p.4].
   - Use the fan command time, or the crew's voice confirmation [CAR19 p.80], rather than the anemometer value.
4. **Approach to the sample end.**
   - Opposed fronts move into a thinning boundary layer near the upstream edge, so the flame lengthens and may speed up slightly [ICES16].
   - Concurrent fronts decay as the preheat front reaches the end [TM p.95].
5. **Extinction** by:
   - quench, when Vg drops below Vg,cr(O2) (~1 cm/s at 21% for thin PMMA);
   - blowoff, at high Vg;
   - fan off, within ~2 s [TM p.13];
   - burnout;
   - N2 jet (BASS-I).
   - Near quench, the luminous flame becomes dim blue and can vanish from the video before true extinction.

---

## G. Known data-quality and interpretation issues

**Luminous flame versus pyrolysis front**
- The trackers follow luminous-intensity thresholds, not the pyrolysis front.
- The burnout line (the black line on the top view) is a separate observable [TM p.22].
- The pyrolysis edge is U-shaped because the film is taped to the holder [ICES16].

**Dim blue flames**
- At low flow and low O2, flames are "very dim blue and … difficult to see in the side video camera".
- The top stills with 1/8–1/4 s exposure still captured them [TM p.94–95].
- Threshold tracking is biased as flames dim [INF].

**Exposure behaviour**
- Stills are mostly auto-exposure with negative EV compensation [TM App. A]. "Camera auto-exposure" appears in a figure caption [ICES19 Fig. 1].
- [INF] Apparent brightness is not comparable across frames, so fixed thresholds can shift with exposure.

**Soot and saturation**
- Sooty orange flames at higher flow and O2; transient very bright soot during flow changes or bubble bursts in thick PMMA [POS14 Narrow Channel slide; TM p.57].
- Saturation is not quantified (**not found**).

**Flame leaving the field of view**
- Not explicitly reported for BASS.
- The FOV covers ~10 cm of sample, and concurrent fabric flames can be long relative to the sample [TM p.93] [INF].
- SPICE flames did leave the video FOV [DEB12].

**Time base and data gaps**
- Video is ~29.97 fps; stills are ~1–1.3 s apart, at a cadence that is not exactly regular [TM p.94; POS12; POS13].
- The video overlay date may differ by 1 day ("add 1 for Leap Year") [TM App. A].
- Loss-of-signal portions are filed separately [PSI25].
- The video cable failed and video was intermittent around GMT 167–170 (2014) [TM App. C].
- A still-camera cable came loose in one test [TM App. A].
- The 7-min still card limit [TM App. D].
- Frame drops: **not found**.

**Igniter glow at start**
- The igniter glows red and dominates the frame. The crew could not judge the flame in the glare [TM p.68].
- In SDSU's example the igniter sits at x = 36 mm [TM p.22].

**Crew hand in view**
- **Not found** in sources. The crew manually adjusts the sample and igniter within the fields of view before the test [SRD].

**Window and duct artefacts**
- Overheat bubbling of the Lexan front-door corner on GMT 51 (2014) after a long burn, repaired GMT 86/91 [TM App. C].
- A soot spot on the window after vapour-jetting on GMT 205, cleaned GMT 209 [TM App. C].
- A flow leak at the door corner [TM p.7].

**Side view cannot resolve across-width structure**
- Edge-on video integrates across the width [INF].
- Flames curve because of holder heat losses, and near quench they narrow and become circular flamelets [TM p.58, p.98].
- The SDSU top-view analysis deliberately uses only the central 14 mm [TM p.22].

**Flow uncertainty**
- ±0.5 cm/s display resolution; ~1-min transducer lag; daily calibration drift; an irregular inlet profile [TM p.4, p.60, p.66].
- Wall and fuel boundary layers mean the local velocity differs from the free stream [TM p.13; ICES16].

**O2 uncertainty**
- Only pre/post readings. Intra-test O2 is interpolated [TM p.60].
- Two BASS-II fabric tests have possibly inaccurate O2 readings [TM Table 7.1 note].
- The MCA offset is applied [TM App. B].

**Radiometer**
- Uncalibrated, gain-dependent, with a few seconds' lag [CAR19].
- After extinction it sees the hot sample holder [TM p.13].

**Other physics confounders**
- Vapour jets from bubble bursts in PMMA disturb the flame [POS14; TM p.52; POS13].
- Smouldering spots on fabric behind the flame [TM p.95].
- Reused samples were cut or partially burnt [TM p.10, p.95].

---

## H. Quantitative reference table

| Quantity | Value | Source |
|---|---|---|
| Duct cross-section | 7.6 × 7.6 cm, rounded corners | TM p.4 |
| Test-section length | 17 cm (full duct ~20 cm) | TM p.4; DEB12 |
| Max flow, 0 / 2 restrictors | ~55 / ~35 cm/s | TM p.4 |
| Velocity sensor | TSI 8475, ~1 min response, ±0.5 cm/s display | TM p.4, p.66 |
| MSG free volume | 0.25 m³ (0.255 gross) | TM App. B |
| N2 flow for dilution | ≤500 sccm; dilution 1–3 h | SRD; TM p.6 |
| O2 range, BASS-II | ~13.6–22.2% | TM p.64; ICES19 |
| Pressure | 1 atm (99.2–99.5 kPa) | TM p.38 |
| Still camera | Nikon D300s, 4,320 × 2,968 px, 60-mm lens (CAR19: 35 mm f/2) | TM p.4; CAR19 |
| Still pixel scale | 34 px/mm at sample plane | SR18 |
| Still cadence | ~1 fps (1.0–1.5 s) | TM p.21; POS12; POS13 |
| Still exposure, fabric | 1/8–1/4 s | TM p.95 |
| Video camera | Panasonic WV-CP654, 760 × 480, ~29.97 fps, turning mirror, front window | TM p.4–5, p.66, p.94 |
| SPICE-era video scale | 336.02 px / 70 mm | DEB12 |
| Overlay rate | 4 Hz | SRD |
| SIBAL | 75/25 cotton/glass, 0.32 mm, 18.2 mg/cm², 1.2/2.2 × 10 cm | TM p.94 |
| PMMA films | 0.1–0.4 mm; 10/20 mm wide; ~95–105 mm long | SRD; TM p.21; ICES16 |
| PMMA slabs | 1–5 mm; 1(1.2)/2(2.2) cm; 10 cm long; 1- or 2-sided | TM p.55–56 |
| PMMA rods | 6.35/9.525/12.7 mm dia., 57.15 mm long | TM p.47 |
| Tracking threshold | 30 (8-bit intensity), central 14-mm strip, 5-point regression, ≤5% | TM p.22 |
| Thin PMMA Vf max, 100 µm, 21% | 2.2 mm/s | TM p.41 |
| Thin-limit formula | Vf = 2.34×10⁻⁸ F/τ (m/s), F = 4.76 | TM eq. 2.11–2.12 |
| Quench velocity, thin PMMA | ≈1 cm/s at 21%; ≈3 cm/s at 16% | TM Table 2.1 |
| Blowoff, thin PMMA | 28–33 cm/s at ~17.3%; none at 45 cm/s, 21% | ICES19 |
| SIBAL concurrent vs opposed at 10 cm/s | 2.55 vs 1.21 mm/s (2 cm, 21%) | POS12 |
| SIBAL concurrent fit | Vf ≈ 0.20U + 0.56 (2 cm); 0.15U + 0.07 (1 cm) [mm/s, cm/s] | POS12 (sign INF) |
| SIBAL quench velocity | 1–5 cm/s; 2.2 at 18.7% → 3.2 at 16.9% (2.2 cm) | TM p.93; ICES19 |
| SIBAL blowoff | 47 cm/s at 17.2% (1.2 cm) | ICES19 |
| SIBAL steady duration | ~9 s at 10 cm/s; ~21 s at 5 cm/s | TM p.95 |
| Thick slab Vf | 0.012–0.144 mm/s | TM Table 5.1 (range) |
| Rod no-spread below | 0.6 cm/s (SR18); ~0.4 cm/s (TM) | SR18; TM p.48 |
| Stagnation boundary | X_O2 = 6e^(−0.5a) + 0.0167a + 13.4; min 13.5% at a = 8–12 s⁻¹ | TM eq. 6.1 |
| Rod pre-quench oscillation | 1.1 Hz | TM p.72 |
| Heat release | mostly <100 W (~50 W) | TM p.1; ICES19 |

---

## I. References

1. **[TM]** Olson, S.L.; Ferkul, P.V.; Bhattacharjee, S.; Miller, F.J.; Fernandez-Pello, C.; Link, S.; T'ien, J.S.; Wichman, I. *Burning and Suppression of Solids–II (BASS-II) Summary Report*. NASA/TM-20210011385, Oct 2021. NTRS 20210011385. https://ntrs.nasa.gov/citations/20210011385. Supports A–H. Includes the reprinted papers and App. A–E.
2. **[POS14]** Ferkul, P.V. et al. *Combustion of Solids in Microgravity: Results from the BASS-II Experiment*. ASGSR poster, Pasadena, Oct 2014. NTRS 20160000593 (local PDF). Supports A, B, D, F.
3. **[ICES15]** Olson, S.L. et al. *Results from On-Board CSA-CP and CDM Sensor Readings during BASS-II…*. ICES-2015-196. NTRS 20150019858. https://ntrs.nasa.gov/citations/20150019858. Supports A, B, C.
4. **[ICES19]** Olson, S.L.; Ferkul, P.; Fernandez-Pello, C.; Miller, F.; Bhattacharjee, S.; Wichman, I.; T'ien, J. *Flammability limits from BASS-II testing in microgravity compared to normal gravity limits*. ICES-2019-101. https://hdl.handle.net/2346/84505. Supports A, B, C, F, G.
5. **[ICES16]** Bhattacharjee, S.; Carmignani, L.; Simsek, A. *Boundary Layer Effect on Opposed-Flow Flame Spread in the Microgravity Regime*. ICES-2016-387. https://ttu-ir.tdl.org/items/41223c66-e96b-4a7e-b390-464e5241e88e. Supports B, D, E, F.
6. Bhattacharjee, S.; Simsek, A.; Olson, S.; Ferkul, P. *The critical flow velocity for radiative extinction in opposed-flow flame spread in a microgravity environment…*. Combust. Flame 163 (2016) 472–477. doi:10.1016/j.combustflame.2015.10.023. Reprinted as TM §2.2. Supports E, F.
7. Bhattacharjee, S.; Laue, M.; Carmignani, L.; Ferkul, P.; Olson, S. *Opposed-flow flame spread: a comparison of microgravity and normal gravity experiments to establish the thermal regime*. Fire Saf. J. 79 (2016) 111–118. doi:10.1016/j.firesaf.2015.11.011. Reprinted as TM §2.3. Supports E, F.
8. Bhattacharjee, S.; Simsek, A.; Miller, F.; Olson, S.; Ferkul, P. *Radiative, thermal, and kinetic regimes of opposed-flow flame spread: a comparison between experiment and theory*. Proc. Combust. Inst. 36 (2017) 2963–2969. **Full text not read**; regime content is taken from TM, ICES16 and CAR19.
9. Bhattacharjee, S. et al. *Measurement of instantaneous flame spread rate over solid fuels using image analysis*. Fire Saf. J. 91 (2017) 123–129. **Not read**; the method is taken from TM p.22 and CAR19.
10. Carmignani, L. et al. *Boundary layer effect on opposed-flow flame spread and flame length over thin PMMA in microgravity*. Combust. Sci. Technol. 190 (2018) 535–549. **Not read directly**; covered in CAR19 ch. 5.
11. **[CAR19]** Carmignani, L. *Opposed-Flow Flame Spread over Solid Fuels in Different Burning Regimes*. PhD dissertation, UC San Diego / SDSU, 2019. https://escholarship.org/uc/item/0rb693zc. Supports C, E, F, G.
12. **[OF15]** Olson, S.L.; Ferkul, P.V. *Microgravity Flammability of PMMA Rods in Concurrent Flow*. 9th US National Combustion Meeting, 2015. NTRS 20150008961. Supports A, B.
13. Olson, S.L.; Ferkul, P.V. *Microgravity flammability boundary for PMMA rods in axial stagnation flow: experimental results and energy balance analyses*. Combust. Flame 180 (2017) 217–229. doi:10.1016/j.combustflame.2017.03.001. Content read as TM §6.1. Supports D, E, F.
14. **[SR18]** Link, S.; Huang, X.; Fernandez-Pello, C.; Olson, S.; Ferkul, P. *The Effect of Gravity on Flame Spread over PMMA Cylinders*. Sci. Rep. 8 (2018) 120. doi:10.1038/s41598-017-18398-4. Supports A, C, D, E, F.
15. **[ZH15]** Zhao, X.; T'ien, J.S.; Ferkul, P.V.; Olson, S.L. *Concurrent flame growth, spread and extinction over composite fabric samples in low speed purely forced flow in microgravity*. 9th US National Combustion Meeting, 2015. NTRS 20150008962. The journal version is Proc. Combust. Inst. 36 (2017) 2971–2978 (not read; content matches TM §7.1). Supports B, C, D, E, F.
16. **[POS12]** Ferkul, P.; Olson, S.; Johnston, M.; T'ien, J. *Flammability Aspects of a Cotton-Fiberglass Fabric in Opposed and Concurrent Airflow in Microgravity*. ASGSR 2012. NTRS 20130012832. Supports B, C, E, F.
17. **[POS13]** Ferkul, P.V. et al. *Thickness and Fuel Preheating Effects on Material Flammability in Microgravity from the BASS Experiment*. ASGSR 2013. NTRS 20140011099. Supports A, B, C, F.
18. **[FS]** NASA fact sheet *Burning and Suppression of Solids (BASS)* / BASS-II / BASS-M. https://www.nasa.gov/wp-content/uploads/2025/07/burning-and-suppression-of-solids-bass.pdf. Supports A, B, D.
19. **[SRD]** *BASS-II Science Requirements*, 2010; manifest dated 2023. PSI-25 file "PSI-25_Science Documents_BASS-II_SRD.pdf". Supports A, B, C, D.
20. **[PSI25]** NASA PSI, PSI-25 BASS-II, DOI 10.60555/4qc4-de67. https://psi.nasa.gov/physci/repo/data/investigations/PSI-25. Supports C, D (file naming, table columns).
21. **[PSI26]** NASA PSI, PSI-26 BASS, DOI 10.60555/58h4-dy80. https://psi.nasa.gov/physci/repo/data/investigations/PSI-26. Supports B, D. Investigation dates 2009-10 to 2013-06; 152 GB.
22. **[DEB12]** DeBold, T.F. *Laminar Smoke Points of Coflowing Flames in Microgravity*. MS thesis, Univ. of Maryland, 2012. https://drum.lib.umd.edu/bitstream/1903/13569/1/DeBold_umd_0117N_13883.pdf. Covers the SPICE hardware, video scale and AIR calibration. Supports A, C (proxy only).
23. **[GN14]** Stocker, D.P. et al. *Gaseous Non-Premixed Flame Research Planned for the ISS*. 2014 Spring Technical Meeting, Central States Section of the Combustion Institute. NTRS 20140009169. Supports A, C (SPICE/BASS common hardware, overlay time stamp).
24. Endo, M.; T'ien, J.S.; Ferkul, P.V.; Olson, S.L.; Johnston, M.C. *Flame Growth Around a Spherical Solid Fuel in Low Speed Forced Flow in Microgravity*. Fire Technol. 56 (2020) 5–32. doi:10.1007/s10694-019-00848-2. **Not read.**
25. Dotson, K.T. et al. *Laminar smoke points of coflowing flames in microgravity*. Fire Saf. J. 46 (2011) 550–555. This is the original SPICE duct reference. **Not read.**

### Not found or not read
- PSI-25 ReadMe.docx, Ops.docx and Test.Matrices.xlsx, and the Fan calibration .xlsx. These are listed on PSI but could not be previewed, and I did not download them.
- The PSI-26 BASS final report: the preview link returned "NoSuchKey".
- The PSI-26 holder and frame drawings.
- Explicit video FOV (cm) or mm/pixel for BASS.
- Which PSI file ("main" versus "2ND_VIEW") is which camera.
- Video auto-gain behaviour.
- Frame-drop statistics.
- Any statement about crew hands in view.
- How analysts used the movable scale.
- Full texts of PCI 36 (2017) 2963 and 2971, Fire Saf. J. 91 (2017) 123, CST 190 (2018) 535, and Endo 2020.
