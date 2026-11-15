# The experiments behind the models

What the three NASA experiments physically are, how their data came to be, what the train partitions
show, and which design decisions follow. Written before the phase-2 weights and revised as the train
data were read. Sources are the NASA documents and papers collected in
[research/exp_flex.md](research/exp_flex.md), [research/exp_acme.md](research/exp_acme.md) and
[research/exp_bass.md](research/exp_bass.md) (each number there carries its citation), and the
train partitions of the PSI archives as fetched into `data/raw/samples/`. No test-partition run was
opened. Where a statement comes from the train data it says so.

Exposure that must be stated: the literature dossiers contain aggregate statistics over whole
investigations (the FLEX data report's summary table; label counts, burn-time ranges and the 66 s
cluster of the full PSI-159 tables). Those aggregates include runs of the validation and test
partitions. No per-run value was transcribed, and every number in the weights comes from train
tables, but the aggregates were read.

---

## 1. FLEX — droplet combustion in the CIR (PSI-69)

### 1.1 Apparatus and procedure

The Multi-user Droplet Combustion Apparatus sits in the Combustion Integrated Rack: a sealed chamber
of about 90–100 L with eight windows, filled to the test atmosphere from bottles (O2 with N2, CO2 or
He; 0.5–3 atm). Two opposed 250 µm needles dispense the fuel; the operator stretches the droplet
between them, then hands over to the MDCA computer, which in quick succession starts the cameras,
retracts both needles at once (deployment), energises two hot-wire igniters for a preset time, and
withdraws them. Recording stops after about 40 s. Four to ten droplets are burned per session,
several in one chamber fill, so oxygen falls and products build up from burn to burn; the
tabulated atmosphere is computed after the fact from the fill and combustion history.

A free droplet keeps whatever velocity deployment and ignition gave it: typically 1–2 mm/s, up to
7–8 mm/s, and it often touches an igniter.

### 1.2 Instruments and what the data files are

| Channel | Instrument | What is recorded |
|---|---|---|
| Droplet diameter | HiBMs backlit telecentric view, 30 fps, about 30 mm field, 32.5 px/mm, 650–660 nm backlight | projected area → equivalent-circle diameter; ±50 µm stated uncertainty |
| Flame (UV) | LLUV intensified camera, 310 nm (OH*), about 50 mm field, 30 or 15 fps | outer edge of OH* emission; ±200 µm |
| Flame (visible) | colour camera, 640×480, about 93 mm wide | visible flame, about 3/4 of the LLUV diameter |
| Radiometers | broadband and water band | not usable in 2009–2011 |

Per test, PSI-69 holds (file names as in `data/raw/samples/PSI-69/`):

- `PSI-69_Raw Data_FLEX-NNN_DropletData.txt` — columns `DropletTime_sec`, `DropletDiameter_mm`,
  `ZeroedDropletXpos_mm`, `ZeroedDropletYpos_mm`. **Measured.** The stored IGNIS observations carry
  only the diameter; the droplet position is in the file but not ingested.
- `PSI-69_Raw Data_FLEX-NNN_TestSheet.txt` — date, CIR test number, atmosphere, then `D0 (mm)`,
  `kavg (mm2/s)`, `Dext (mm)` and a narrative comment by the science team. **Derived by the PI team**:
  `kavg` is their fitted burning-rate constant, `Dext` the droplet diameter at visible-flame
  extinction, extrapolated when the droplet had left the view.
- `PSI-69_Raw Data_FLEX-NNN_LLUVData.txt`, `…_ColorCamData.txt` — flame diameters, for few tests.
- `PSI-69_Experimental table_FLEX.csv` — one row per test.

**Time origin.** The data report defines time zero as the moment the igniters are energised, and D0
as the diameter then. The train files agree: the position columns are zero at t = 0, the series
starts at about −1.4 s (droplet on the needles, stretched), and the diameter starts falling 0.2–0.3 s
after zero. One train sheet (C01H101) says its recorder missed the start and its time zero is the
first recorded frame.

**Units.** mm, s and mm²/s in the files; the IGNIS records convert to SI.

### 1.3 What the train series show: methanol (68 runs, 27 day-sessions)

Read: every train test sheet with its narrative, and every train `DropletData` series (windowed
finite differences of D² over 0.2–1, 1–2, 2–3, 3–5, 5–8, 8–12, 12–18, 18–26 s; the time and diameter
where the series stops shrinking).

**The series often ends because the droplet left the view, not because it stopped burning.** 41 of 68
train narratives mention the droplet leaving the HiBMs field, about 30 of them before extinction; the
listed `Dext`
is then an extrapolation "using the burning rate constant from the last part of the burn", or is
missing. A series that continues after extinction shows the unlit droplet still shrinking slowly.

**Two extinction mechanisms, visible as two separate branches of Dext/D0.**

| Branch | Train evidence | Where |
|---|---|---|
| Diffusive | Dext/D0 = 0.36–0.57, no trend with D0 from 1.6 to 4.3 mm (water uptake scales with the droplet); 0.11–0.27 at 30–34 % O2 and 0.7 atm | all atmospheres with D0 below a critical diameter |
| Radiative | Dext/D0 = 0.75–0.96; the flame is lost 0.6–1.3 diffusion-clock units after ignition (t·1 mm²/s / D0²), i.e. 14 s for 4 mm and 7.5 s for 2.6 mm | D0 above the critical diameter |
| No sustained flame | Dext/D0 ≈ 0.99, burning rate 0.05–0.09 mm²/s from the start | D0 more than about 55 % above the critical diameter ("outside the quasi-steady flammability boundary") |

The critical diameter is bracketed by train pairs at one atmosphere: at 1 atm in N2, 12 % O2 between
1.9 and 2.6 mm, 14 % between 2.7 and 3.3 mm, 15 % between 3.4 and 4.0 mm, and no radiative
extinction up to 3.9 mm at 18 % and above. CO2 and helium both lower it. All eight train droplets
within 5 % below to 10 % above the bracketed value extinguished radiatively. This is why "the same
nominal condition lands on either side": the phase-1 model saw scatter where there is a size
threshold. The narratives name the branch ("diffusively extinguishes", "radiatively
extinguishing", "outside the quasi-steady flammability boundary"), and they agree with the
bracketing.

**Burning rate.** Plateau rates (2–8 s): 0.54–0.56 mm²/s in air for D0 ≤ 2.8 mm, 0.47 at 3.9 mm;
0.69 at 30 % O2; 0.44 at 18 %, 0.41 at 15 %, 0.37 at 12–14 %. Helium raises the rate by about 0.18
in ln K per 20 % He, linearly up to 45 %; CO2 at equal O2 changes it by a few percent. The rate falls
by about 17 % between a fifth and seven tenths of the burn (water uptake).

**Heat-up scales with D0².** In the first second the rate is 55–60 % of its plateau for 3.7–4.3 mm
droplets, 75–80 % for 2.6 mm and already at plateau for 1.7 mm. In clock units the plateau is reached
at 0.18 for every size (1.2 s at 2.6 mm, 2.7 s at 3.9 mm).

**After extinction the droplet keeps shrinking**: 0.06–0.08 mm²/s right after, 0.03 after 10 s and
0.02 after 20 s for 3 mm droplets; 0.005–0.01 mm²/s for 1 mm droplets. Over a 10 s horizon that is
up to 80 µm of diameter, more than the whole error of a good forecast.

**The first half second.** The "growth" of D² in the first frames of the 2009 runs is not growth.
The narratives of the nine first tests (CIR numbers with `CAL`) say the igniters overexposed the
HiBMs "resulting in the inability to accurately measure the droplet diameter". In the files the
tracker output collapses from about −0.5 s (20CAL08: 1.77 → 1.376 mm, held constant to nine digits
for ten frames; 40CAL01: 2.97 → 0.551 mm) and returns to the true value in one frame at 0.07–0.13 s.
After those tests the shutter went from 1/30 s to 1/1000 s and the artefact disappears. Two 3 atm
CO2 runs show a different one: a saturated backlight makes the measured size depend on which
quadrant of the image the droplet is in, so D drifts up by a few percent as the droplet drifts.

**Causal preprocessing rule that follows** (implemented in the forward, not a removal of early data):
a droplet does not grow. A prefix value is dropped when at least three later prefix values of D²
exceed it by more than 5 %, or lie below what the fastest flame of these experiments could have
burned in the time between (2 mm²/s, twice the fastest measured rate). The second half catches the
heptane artefact of section 1.4. The rule uses only data up to the cutoff, removes exactly the
frozen or collapsed frames (4–5 per `CAL` run), and leaves a late outlier's predecessors alone. The
level the forecast starts from is carried to the cutoff along the observed late slope limited to
[0, 2 mm²/s].

**Data-quality findings in train.**

| Finding | Evidence |
|---|---|
| One video analysed twice under two test ids | `FLEX-124` (C12M401, 2010-09-08, sheet: 18 % O2 / 40 % CO2) and `FLEX-141` (C14M201, 2010-11-04, 15 % O2 / 50 % CO2): the droplet positions agree within 0.15 mm on all 161 common samples between 0.5 and 6 s and the diameters within 0.4 %. No other pair among the 260 files does. The behaviour (immediate radiative extinction at 2.87 mm) fits the C14 atmosphere. Both are train runs; no pair crosses a partition |
| Pressure 0 atm, temperature 0 K | `FLEX-114` (C11M201): the sheet fields are zero; the forward refuses the run |
| Gas bubble | 240R003: "immediately begins to grow, disrupts and finally shatters"; kavg 1.72 mm²/s is not a burning rate |
| Recorder dropouts | gaps of 6–18 s in five train series (C11M201, C07M201, C14M301, C14M302, 189F001) |
| Needle contamination | every test of the data report used needles whose coating leached into the fuel; the report blames it for disruptive methanol extinctions |

### 1.4 What the train series show: n-heptane (49 runs, 22 day-sessions)

The heptane cohort (80 free-floating runs of 113 heptane records) and its split were frozen from
metadata before any heptane outcome was read; a literature-only W0 was frozen next; only then were
the train runs opened.

**Three stages, and NASA's `Dext` is only the first.** For a large droplet the visible flame goes
out by radiation within seconds (`Dext`/D0 = 0.72–1.0), the droplet goes on shrinking at 0.30–0.40
mm²/s for 10–25 s with nothing visible to the colour or UV camera (a cool flame), then the D² curve
plateaus at D/D0 = 0.41–0.64 as a vapour cloud forms. The narratives say so run by run ("rapid and
nearly constant vaporization … followed by a vapor cloud formation and plateau"). A model that
treats `Dext` as the end of burning is wrong by a factor of two in diameter 20 s later.

| Quantity | Train evidence | W0 (literature) said |
|---|---|---|
| Critical diameter, air, 1 atm | 3.4 mm burns nearly out, 3.56–3.58 mm "right on the edge of the radiative limit": 3.7 mm | 2.8 mm |
| Effect of CO2 on it | 2.9 mm at 5 % CO2, 2.3 mm at 10 %, 1.9 mm at 15 % (18–20 % O2) | a third of that |
| Clock at radiative extinction | 0.6–1.2 at the critical diameter, 0.13–0.23 at 70 % above it | 0.55, weak dependence |
| Cool flame after radiative extinction | always in N2 and N2/CO2 at 15–21 % O2; weak at 14 %; none with 30–35 % He | 0.85, falling with He |
| Cool-flame rate | 0.30–0.40 mm²/s | within 10 % |
| Cool-flame extinction diameter | 1.0–2.2 mm, about D0^0.8, larger at lower O2 and pressure | within 14 % on nine of eleven plateaus, from one published curve and the [O2]^-3/4 scaling |
| Small droplets | burn to De/D0 = 0.16–0.45 and usually disrupt there | 0.2 |
| After the cool flame | 0.03–0.06 mm²/s for many seconds | nearly flat |

**The rate of a large heptane droplet rises for seconds.** 3.2–3.5 mm droplets go from 0.40–0.47 to
0.55–0.60 mm²/s over 5–8 s, 2.5 mm droplets within 2–3 s: the flame takes 0.6 of a clock unit to
develop. It is a clock effect, not a burn-progress effect.

**Soot corrupts the diameter at high oxygen.** At 29–34 % O2 the narratives report "significant
sooting with large soot agglomerates" and "the degree of sooting made accurate measurement of the
droplet size difficult". In the files the first frames report the soot shell as droplet (195F002:
4.4 mm apparent against 3.36 mm on the sheet) and the apparent D² then falls at 6–13 mm²/s, which no
flame does. This is what the shrink bound of the validity gate removes.

**Data-quality findings in train.**

| Finding | Evidence |
|---|---|
| Atmosphere conflict | H19H101 and H19H201 (2011-12-08): the summary table gives 15 % He / 64 % N2; the sheet of H19H101 says air, the sheet of H19H201 says 64 % He / 15 % N2. The records mark both `unresolved_conflict` and the forward refuses them. C08H502: 20 % CO2 in the summary, 0 % in the sheet header |
| Time zero not at ignition | C01H101: recorder missed the start |
| Vapour cloud darkens the backlight | C04H201: "the HiBMS view dims when cloud forms and this is likely the cause of the droplet getting slightly larger at the end"; 301F001 reads 5.4 mm at 33 s |

### 1.5 Design decisions that follow

1. Extinction is a mixture of two mechanisms split at a critical diameter, not one distribution:
   diffusive extinction is decided by burn progress, radiative extinction by the diffusion clock.
2. Heptane needs a third state. The cool flame is invisible to every flame diagnostic of FLEX; only
   the droplet diameter shows it.
3. The time scales are D0² scales; the clock `s = t·K_ref/D0²` is the right time variable for
   heat-up, flame development and radiative extinction.
4. After a flame the droplet is not constant: a decaying evaporation law.
5. The state at the cutoff must be inferred (a droplet that stopped shrinking is extinguished;
   a heptane droplet that slowed by 15 % may be in a cool flame).
6. A validity gate from physics instead of a start time: no growth, no impossible shrink.
7. Conditions that the records mark as conflicting are refused, not guessed.

---

## 2. ACME CFI-G — spherical burner flames in the CIR (PSI-159)

### 2.1 Apparatus and procedure

A 6.35 mm porous stainless sphere on a feed tube, in the CIR chamber (about 83 L free volume with
the insert). Mass-flow controllers feed either fuel (ethane, propane or n-butane, pure or diluted
with N2) into an O2/N2 ambient — normal flames — or an O2/N2 mixture into a fuel/N2 ambient — inverse
flames. A retracting hot-wire igniter lights the flow. Tests are run from the ground by scripts; the
crew only changes hardware. Flows are constant within a test (after an ignition step of about 4.5 s
there is no ramp in any setpoint file).

A test ends in one of three automatic ways, and the train comments and control timelines show all
three:

1. **Extinction detected** by the script (photomultiplier or radiometer threshold); the flow shuts
   off 3 s later.
2. **A programmed duration or time-out** runs out. Train timeline of 21196A2: `Ignition and
   retraction` at 0 s, `Constant flow` from 7 s, `MFCs off` at 37 s.
3. **The burner temperature limit** (450 °C in one source, 650 °C in another). Train comments: "hit
   T limit", "Hit the burner temperature limit", "Terminated by burner temp hit the limit."

Tests share a chamber fill: the ambient O2 (normal) or fuel (inverse) falls from test to test and is
tabulated before and after each.

### 2.2 Instruments, files and labels

Colour camera, an intensified camera with CH*/formaldehyde filters, three photomultipliers,
radiometers, thin-filament pyrometry on some tests, burner and far-field thermocouples, chamber
pressure. PSI-159 files: `PSI-159_Experimental table_CFI-G_Normal Flames.csv` and `…_Inverse
Flames.csv` (one row per test), and for a few tests `PSI-159_Raw Data_<test>_TP.txt` (script
timeline), `_data.csv` (raw voltages), `_mfcSetpoints.csv`, `_ImageComments.txt`.

| Table column | Kind | Meaning |
|---|---|---|
| `XO2 before test`, `Xf`, `mHC (mg/s)`, `approx p (bar)`, fuel | set before the test | prospective conditions |
| `Tad (K)`, `Zst` | derived from the mixture | prospective |
| `burn time (sec)` | measured, to 1 s | hot-flame duration from ignition, most likely from the photomultiplier |
| `Self-ext or flow term` | judged | `SE`: the hot flame went out by itself. `FT`: the flow was stopped with the hot flame burning |
| `Peak burner T (C)`, `XO2 after test`, `TFP`, `Cool flame?`, comments | after the test | not predictors |

**`SE` is the end of the hot flame, not the end of combustion.** The reports state that the cool
flames of the first round followed tests labelled `SE` and were invisible to the colour cameras, and
that the two tests that cycled between hot and cool flames are labelled `FT` with the total flow time
as burn time (train: 21291C1, "5 transitions from hot to cool, 6 hot and 5 cool flames"). The train
timelines show the gap directly: for 21088A1 the table gives a burn time of 50 s and the script's
`Extinction detected` stamp is at 58 s; for 21088B1, 43 s and 52 s. So the label is a hot-flame
quench, the script's detection lags it by 8–9 s, and whether a cool flame burned in between is
recorded only in the sparse `Cool flame?` column. Cool flames occurred only for n-butane at 2 bar.

**Two clocks.** The script clock (0 = `Ignition and retraction`, 1 s resolution) and the burn-time
clock (ignition to hot-flame end, judged from signals) differ by 1–3 s at flow termination and by
8–9 s at detected extinctions. The stored endpoints use the burn time with a ±0.5 s interval.

### 2.3 What the train tables show

**Normal flames (81 runs, 7 sessions).**

- Self-extinction is radiative and its time diverges as the fuel flow falls. 30 % ethane in 39 % O2
  at 1 atm: 43 s at 2.36 mg/s, 71 s at 1.5, 158–232 s at 1.0, 304 s at 0.85. The product
  (time − 7 s)·(flow − 0.7 mg/s) is 45–68 mg for all four: a flame below about 0.7 mg/s can be held.
- Below that flow the test ends on the burner limit instead, sooner the lower the flow: 28–56 s at
  0.5 mg/s (burner above 500 °C), 80–160 s at 0.75 mg/s for butane. A small flame sits close to the
  burner and heats it. This is the physical reason the two outcomes exclude each other.
- Oxygen sets the scale: 5–20 s in air against 30–300 s at 33–40 % O2; 44 s against 88 s at 34 % and
  38 % O2 at the same flow. Pressure shortens the life (59, 43, 34 s at 0.5, 1, 2 atm).
- No train flame self-extinguishes before about 5 s unless it is one of the ten with an adiabatic
  temperature of 1600–1700 K, which all went out within a second or two of ignition.
- Programmed stops: 61–62 s (three flames, all at 1 mg/s or less), 98 s, 125–127 s, 274–276 s.
- One low-flow, very dilute test (21125A2: 0.5 mg/s, 3 % butane, 8 s) contradicts the flow law: a
  flame too small to survive on the burner. One test; not modelled.

**Inverse flames (73 runs, 7 sessions).**

- The ambient fuel fraction decides: 5–7 s at 7 % fuel, 8–18 s at 8–9 %, 46–51 s at 14 %, no
  self-extinction at 17 % and above.
- With air injected into 7–9 % fuel the flame either flashes out (1–3 s, three tests) or holds to the
  script stop (two tests).
- 12 of the 32 flow terminations are at 65–66 s and three more at 63–67 s (a 60 s constant-flow
  phase). The burner limit ends
  oxygen-rich tests in a fuel-rich ambient at 12–48 s, sooner at low flow.
- The thin-filament tests of a repeated condition last 5–7 s longer (16–17 s against 8–10 s); the
  fibre is recorded after the test and is not a predictor.

### 2.4 Design decisions that follow

1. Physical clocks instead of a hazard surface: a radiative self-extinction clock after an
   establishment floor, a weak-flame class, a burner-limit clock, programmed stops as atoms.
2. Flow enters the self-extinction clock through a hinge, because the dependence steepens towards
   the holding flow.
3. `extinction` in the model is the end of the hot flame, as in the table; cool flames are excluded
   from the target and said so.
4. Programmed stops are campaign policy, not physics: they are written as what they are, and the
   model is an observational forecast under that policy.
5. With 7 train sessions and 2 validation sessions per configuration, conditions transfer poorly
   between sessions: the validation sessions hold fuel–pressure combinations that train does not.
   The clocks are wide on purpose.

---

## 3. BASS-II — flame spread over thin fuels in the MSG (PSI-25)

### 3.1 Apparatus and procedure

A 7.6 × 7.6 cm duct with a 17 cm test section in the Microgravity Science Glovebox; a fan drives
0–55 cm/s through flow straighteners, with optional perforated restrictor plates. The crew mounts a
sample, sets the fan, ignites with a hot wire, and changes the fan setting during the burn to find
quench or blow-off limits; turning the fan off always extinguishes. The MSG work volume is sealed;
O2 is lowered with nitrogen and measured before and after each burn. The cohort here is T'ien's
tests: 75/25 cotton–fibreglass fabric (SIBAL), 1.2 or 2.2 cm wide, 10 cm exposed, 18.2 mg/cm², in
concurrent flow.

### 3.2 Instruments and what the files are

| Camera | View | Record |
|---|---|---|
| Nikon D300s still camera, 4320×2968, about one frame per second | through the top window, face-on to the sample; 34 px/mm at the sample plane in one published analysis | the images the investigators tracked flames in; in PSI as `… original videos.zip` and `…_Still_Sequence.mp4` for some tests |
| Panasonic video camera, 720×480, 29.97 fps | through the front window via a turning mirror: the sample edge-on | `PSI-25_Raw Data_BASS_GMT<day>_SAMPLE_<n>_TEST_<id>.mp4`, with an overlay |

The video overlay carries `FUEL` (N2 flow), `AIR` (duct anemometer, cm/s, about one minute response),
`FAN` (fan voltage × 10), `RAD` (uncalibrated radiometer, 5 Hz) and a date and time stamp that runs
one day ahead of the real date. `PSI-25_Reference Data_Fan calibration data.xlsx` relates fan
voltage to flow speed for each operations day and restrictor; `PSI-25_Science
Documents_BASS-II_ReadMe.docx` defines the test-matrix columns. In the experimental table
(`PSI-25_Experimental table_BASS-II.csv`) the `Air display` of a test is a sequence such as `10 2 1`:
the flow steps the operator made during the burn, without times.

Published tracking used the top-view stills: the leading edge is where the intensity of a 14 mm
central strip crosses a fixed threshold; the spread rate is a line through five positions.

### 3.3 What the train videos show (4 runs)

The four train duct videos were fetched (104 MB) and measured with a declared image algorithm
(`tools/wsp/bass_optical/extract.py`: per-pixel dark level, largest connected luminous region,
positions as fractions of the frame width).

| Train test | O2 | Flow steps (cm/s) | What the duct video yields |
|---|---|---|---|
| T5 | 18.8 % | 10, 2, 1 | igniter glow 49.5–55.6 s, then a two-lobed flame for 12 s whose leading edge moves from 0.807 to 0.706 of the frame width (0.012 widths/s) while it shortens from 0.11 to 0.02 and goes out |
| T6 | 18.8 % | 5, 2, 1, <1 | igniter glow for 7 s; nothing above the detection threshold afterwards |
| T16 | 17.6 % | 5, 3 | igniter glow for 9 s; nothing afterwards; a band of encoder noise at both frame edges |
| T10 | 16.4 % | 5 | nothing above threshold; its `_2ND_VIEW` file is a cabin camera, not the duct |

One usable track of 12 s from four runs. In low-oxygen air the fabric flame is a dim blue that an
8-bit, heavily compressed video does not hold above its noise; the lit frames before and after the
burn show the sample edge-on, so the camera sees height above the sheet and position along it, never
position across it.

### 3.4 What is legitimately measurable, and what is not

- **Not available:** a metric front position. No BASS run has a calibration; the video has no scale
  in frame; the duct camera's pixel scale is not published.
- **Available now, in image units only:** extent of the luminous region along the sample axis as a
  fraction of the frame width, its height, its area and summed signal — for flames bright enough,
  which in the train partition is one run.
- **Available with work that belongs to the data lane:** the top-view still sequences (the
  investigators' own source, with the sample width as an in-frame scale), and the fan and anemometer
  readings from the overlay as a control history.

No model was built on one 12 s track. `bass.wsp` W0 remains what it was: runnable on metric
contours, refusing pixel-only ones, never scored.

---

## 4. Open points at the lane boundaries (13 November)

> **Status, 14 November.** Nothing in this list is waited for. Section 6.4 says, point by point,
> what this lane built itself and what the models do without. The list is kept as it was written:
> it records what the models would use if the shared records held it.

**Data and calibration.**

1. FLEX: ingest `ZeroedDropletXpos_mm` and `ZeroedDropletYpos_mm` as `DropletPosition` (the
   quantity exists in the contracts). Drift speed is known at the cutoff and leaving the 30 mm field
   is the commonest reason a series ends.
2. FLEX: ingest `Dext`, `kavg` and the narrative of the test sheets as post-run facts with a flag
   for "extrapolated" (the narrative says when). They are the only adjudicated extinction diameters.
3. FLEX: resolve or exclude `FLEX-124`/`FLEX-141` (one video under two ids), H19H101/H19H201 and
   C08H502 (atmosphere conflicts), C11M201 (pressure and temperature zero), C01H101 (time zero is not
   ignition).
4. FLEX heptane: an adjudicated cool-flame plateau (time and diameter) would make the three-state
   model scorable as events; today only the diameter is scored.
5. ACME: keep `SE` documented as hot-flame end; ingest `Cool flame?` as a post-run fact; carry the
   script timeline (`Constant flow`, `MFCs off`, `Extinction detected`) for every test that has a
   `_TP.txt`, and the programmed duration when a parameter sheet gives it — a known script stop is a
   prospective control.
6. BASS: fetch the still-image sequences of the cohort; derive a pixel scale from the sample width
   in the top view; read `FAN` and `AIR` from the video overlay as controls; a front quantity in
   image units (fraction of frame width, declared as such) if metric calibration is not possible.
   None of this was written into the shared records by the WSP lane.

**Model service.** Releases find their forward by `model_name`; a family now has several
literal models (`flex.wsp`, `flex_methanol.wsp`, `flex_heptane.wsp`, …). A service must route by
release, not by family; must pass refusals through as artifacts (unresolved conditions, wrong fuel,
forward mismatch); and should expose the diagnostics block, which for droplets carries the state
probabilities (hot, cool, no flame) over time and the laws used. `ignis_wsp::models::by_name(…)
.artifact(…)` is the single call.

**Evaluation and visualisation.** (1) Heptane needs its own cohort and split in the final
protocol: `sha256:8ffa1875…5797` / `sha256:c43bbe35…2001`, frozen before any outcome was read.
(2) The fitted FLEX experts refuse heptane; the only baseline is the literal D²-law release. (3) For
droplets, plot the predictive band with the state probabilities under it: the model's claim is the
stage sequence, not only the diameter. (4) For ACME, score hot-flame end as the table defines it and
keep programmed stops visible: a model that knows the 66 s script stop is predicting policy. (5) Two
clusters of validation sessions per ACME configuration cannot rank models; the paired intervals say
so. (6) Candidates marked "calibration only" were revised after one aggregate validation look; the
test partition is the only clean measure left for them.

## 5. What the train audits added (14 November)

Four delegated audits of the second-generation models against their train partitions (reports in
[feedback/](feedback/)), rechecked by the author. Only what changed a design decision is listed.

### 5.1 FLEX

**Methanol** (68 train runs).

- **Radiative extinction is a cliff, not a slope.** At the critical diameter the visible flame is
  lost at one clock unit (s = t·1 mm²/s / D0² between 0.79 and 1.17 in six runs, N2 and CO2 alike).
  Above it the clock collapses: 0.60, 0.46, 0.32, 0.31 at D0/D_crit of 1.17, 1.30, 1.34, 1.47, and
  0.08 at 2.0, where the flame never settles. Within one atmosphere a droplet 20–40 % larger goes out
  at a third to half the clock (four pairs, three sessions), so sooner in seconds too. One line,
  ln s = −3.8 (ln(D0/D_crit) − 0.05), passes through all of them, including the no-sustained-flame
  runs that the second generation needed a separate term for.
- **Helium runs the clock faster.** The three helium runs lose the flame at 0.57–0.76 of that law
  (25–45 % He). The heptane helium runs do the same.
- **An unlit droplet sheds the flame's heat fast.** In the first two seconds after the visible flame
  is lost, D² falls at 0.066 mm²/s for a 1.3 mm droplet and 0.165 mm²/s for a 2.9 mm one, twice what
  was written; by 5–10 s it is back on the slow evaporation. The amplitude of the transient grows
  about linearly with diameter and its decay time (2.4 s and 3.3 s) hardly at all.
- **A prefix that ends in heat-up says little about the plateau.** Of the rate offset a prefix shows
  at a 3 s cutoff, a fifth to a third carries into the next seconds; at 5 s about half.
- **The state at the cutoff must be read from the end of the prefix.** A droplet that went out one
  or two seconds before the cutoff shows 0.4–0.9 of the burning rate over a window that straddles
  the extinction, and was read as burning in five of seven such cases.
- **Heat-up is slower in weak atmospheres** (below 16 % O2: 16 % under the law before s = 0.2, four
  of five sessions).
- Not found: any effect of the order of burns within a chamber fill, of drift speed, of CO2 or of
  pressure on the rate beyond what was written. 31 % of the train error of the model is tracker
  artefact (glare, bubbles, droplets leaving and re-entering the view, dropout frames inside the
  scored window) and cannot be forecast.

**n-Heptane** (49 train runs).

- **The droplet burns near its early plateau at once.** Over 0.3–1 s after the igniters the rate is
  16 % above the second-generation law and over 1–2 s 7 % above (13 and 12 of 16 sessions, every
  size, oxygen, diluent and pressure class): there is no deep heat-up deficit.
- **The radiative clock shortens with size beyond supercriticality**: 2.7–3.5 per e-fold of D0 at a
  fixed atmosphere (six pairs, five sessions) against the 2.0 written.
- **The cool-flame rate law is right to 6 %** (ten runs at 1 atm and 15–19 % O2, seven sessions);
  what was wrong was reading the cool rate of a run from a window that still held the hot flame or
  the dip between the two flames (in the first second after the visible flame is lost the rate dips
  to 0.4–0.6 of the cool rate when that happens early, and the cool rate is reached 1–2 s later).
- **After the cool flame** the droplet evaporates at 0.12 mm²/s for two seconds, decaying over
  about 3.5 s to 0.03.
- **Helium is a different regime.** After the visible flame is lost in 30–35 % helium, D² keeps
  falling at 0.5 mm²/s for two seconds and at 0.2–0.3 for four more, with no cool flame reported
  (three runs, one session). No law was written for it.
- 42 % of the train error of the model is measurement artefact (soot, glare, droplets leaving the
  view, frozen tracker values after the droplet is gone).

### 5.2 ACME

- **What matters for a normal flame is the flow relative to the flow a flame can hold for good, and
  that flow rises with oxygen.** At 39 % O2 the life grows as flow^−1.2 from 2.4 to 1.5 mg/s and far
  faster below (43, 71, 158–232, 304 s). In air it does not depend on flow from 2.4 down to 1.0 mg/s
  (9–10 s) and lengthens only below 0.75 mg/s (11, 16–19 s). One law in the reduced flow
  ln ṁ − 0.65·(x_O2 − 0.30)/0.10 + 0.24·ln p reproduces the ethane lives of three sessions at 0.5, 1
  and 2 atm within about 15 %.
- **Butane flames are shorter in enriched oxygen only** (ratio 0.57–0.83 to ethane; in air they are
  on the ethane law), and shorter when dilute. Propane in enriched oxygen has no train test.
- **The burner temperature limit was not one setting**: about 455 °C on one day, 505 °C or more on
  others. Ethane reached it in 28–56 s at 0.5 mg/s and not within 275 s at 0.75 mg/s.
- **Programmed stops are session facts.** Among normal flames, each stop time (61, 98, 125–127, 275 s)
  comes from one or two sessions. Among inverse flames the 66 s stop was used by four of the five
  sessions with a flame alive at 65 s (13 of 14 such flames); every other stop time occurs in one
  session. A time seen once does not transfer.
- **An inverse flame flashes out when air is injected into a lean ambient** (7–9 % fuel: three of six;
  none of nine at 19.6 % or more), not when its adiabatic temperature is low: flames at 1785 K burned
  for a minute, flames at 1835–1879 K flashed.
- Outside the inputs of any model: about 15 % of normal and 40 % of inverse outcomes (script and
  operator stops, the limit setting, the fibre, igniter dwell, one mis-recorded oxygen fraction).

### 5.3 BASS

The measurements are in the [optical card](cards/bass_optical.md). In short: three of the four train
burns can be tracked for about 100 s in both cameras; the front responds to a fan step within half a
second; at the lowest fan setting the three flames spread at the same rate (0.036 sample widths per
second); 16.4 % oxygen at 6 cm/s does not spread. The target that follows is the leading-edge
position in sample widths with the fan history as a known control.

### 5.4 Open points, additions to section 4

> **Status, 14 November.** As for section 4: see section 6.4.

**Data and calibration.**

7. FLEX: flag tracker artefacts as measurement status, not as values: dropout frames (191R001 at
   3.33–3.37 s reads 0.094 mm), frames after a recording gap (C07M201), frozen values after a droplet
   is gone (C02H202, C03H701 hold 0.49 and 0.47 mm), droplets re-entering the view (40CAL01). A third
   of the train error of any droplet model is scored against such values.
8. FLEX: the provenance of each `Dext` (direct, extrapolated, listed though the narrative says none,
   droplet not in view) as a field; seven methanol train values are listed although the narrative
   says there was no extinction in view.
9. FLEX helium: an adjudicated description of what follows the visible flame in 30–35 % helium
   (H12H101, H13H101, H13H201); the diameter says something keeps burning.
10. ACME: the burner-limit setting and the script durations per session, and the starting burner
    temperature of each test. They decide every flow termination and are known before the test.
11. ACME: 21256C2 is recorded at 50 % O2 with the comment "No nitrogen flowed".
12. BASS: the exposed sample width (2.0 or 2.2 cm) and the holder drawing; the original D300s stills
    (the proof sheets hold 100 of 117 for T6 and 18 of 43 for T10); the still-camera clock offset
    (0–2.3 s against the overlay); fan calibration for settings 41–46 on GMT 100 or a ruling on
    borrowing the rows of another day; the radiometer gains.

**Model service.** Two more literal models (`flex_methanol.wsp2`, `flex_heptane.wsp2`) run
on a third forward. Nothing changes in the call. The droplet diagnostics now also carry the end
rate, the state-evidence exponent and the time of the window the level was carried from; a client
that shows why the model thinks the flame is out should show `r_end` against the three state means.

**Evaluation and visualisation.** (7) Each generation has its own forward and its releases
are immutable, so the final protocol can score W0, D1 and the third generation side by side.
(8) The third-generation candidates were chosen on train by a rule written before their validation
scores existed; their validation numbers were looked at once and changed nothing. (9) For droplets a
score by true state at the cutoff (burning, already out, cool flame) separates what a model knows
from what the tracker did; the train audits could only do it with hand-read extinction times.
(10) For burner flames, score flow terminations separately for scripted stops and burner-limit
stops if the data lane can label them: they are different predictions.

---

## 6. The fourth generation (14 November): published science, measurement in this lane, and what validation said

Three literature reviews ([research/lit_droplets.md](research/lit_droplets.md),
[lit_spherical_flames.md](research/lit_spherical_flames.md),
[lit_flame_spread.md](research/lit_flame_spread.md)), three more train-only analyses
([feedback/](feedback/)), a blind measurement of the BASS validation burns, and one registered
validation look per candidate. Every number that went into a model was checked against the train
tables and typed by the author. Each finding below is given with what became of it.

### 6.1 FLEX

- **What follows a flame lost in helium is a cool flame.** Published work reads a droplet that keeps
  shrinking with no visible flame as a cool flame burning at 0.8–0.87 of the hot rate in every
  diluent, helium included, and transient at high helium dilution. The three train runs in 30–35 %
  helium shrink at 0.83 of the hot rate just after the flame is lost and fall by e every 3.4 s.
  *Became:* a helium law in the heptane candidate. It was typed as `exp(3.85 x_He)` with nothing
  bounding it at the hot rate, and the candidate that carried it was not adopted.
- **Radiative extinction is two-sided, not gradual.** Near the critical diameter a droplet either
  goes out early or burns on; published FLEX analyses call the transition bimodal. Five train
  droplets within −2 to +9 % of the critical diameter all went out radiatively. *Became:* a steeper
  class boundary in the methanol candidate; not adopted.
- **Where the train error is.** 39 % of the methanol train error and 57 % of the heptane train error
  is tracker artefact (soot shells, glare, bubbles, droplets leaving and re-entering the view). The
  rate plateau and both extinction clocks are unbiased by session. Four signals in a prefix say that
  the tracker is in trouble:

  | Signal in the prefix | Catches on train | False alarms on train |
  |---|---|---|
  | prefix D0 more than 10 % from the sheet's | a soot shell read as the droplet, a droplet hidden by the flame, a bubble (three cases; 30 % of the heptane train error) | none |
  | three or more frames dropped by the validity gate after 0.2 s | a bubble, a droplet re-entering the view (methanol) | twelve well-forecast heptane cases |
  | D² rising by more than 3 % | the above, saturated backlight, one run at 2 atm in 70 % CO2 | two |
  | end rate outside −1.5 to 3 times the law | a collapsing soot shell (97 times the law) | one |

  *Became:* a broad band for flagged prefixes, tried alone on the third-generation numbers
  (methanol D5). Better or level in 25 of 26 train sessions; 0.6 µm worse on validation. The
  artefacts are particular train runs, not a property of unseen sessions.
- **The tracker is far more precise than the bands.** Frame-to-frame scatter is 1.5 µm against a
  measurement band of 20–25 µm. *Became:* narrower bands, and that was the mistake of this
  generation: on unseen sessions the 90 % band then covered 0.82 (methanol) and 0.87 (heptane). The
  band has to cover the error of the law on a new session, which the tracker's scatter says nothing
  about.
- **Not predictable from a diameter prefix in principle:** which side a droplet near the critical
  diameter falls on, the exact end of a cool flame (published theory: the extinction condition is
  flat near its maximum), disruption, and drift out of view.

The carried droplet models are the third generation's.

### 6.2 ACME

- **The published growth law places the holding flow and the lives without tuning.** A
  burner-supported spherical flame grows towards `r_ss = m_tot / (4π ρD L)` and goes out when the
  reactant flux at the flame falls to a critical value; if `r_ss` is inside that radius the flame is
  held. With the published constants (ρD 0.0334 g m⁻¹ s⁻¹, critical fuel flux 0.2 g m⁻² s⁻¹, D 28
  mm²/s) the law puts the holding flow of 30 % ethane in 39 % oxygen at 0.65 mg/s (train: held at
  0.75, out after 304 s at 0.85), the lives at 46, 64 and 165 s (train: 43, 71, 158–232 s), and
  makes the life in air independent of flow (train: 9–10 s from 1.0 to 2.4 mg/s). The second
  generation typed both facts as hinges from seven sessions.
- **What the simple theory lacks, from train:** the clock runs faster with pressure (lives go as
  p^−0.45, where D ∝ 1/p alone would lengthen them), twice as fast in air as in 39 % oxygen, slower
  for a dilute burner gas, twice as fast for butane in enriched oxygen.
- **It does not describe lean inverse flames.** With the published inverse-flame constants it gives
  them minutes or holds them; they last 3–18 s. The reports describe a quench hole at the supply
  tube that oscillates and grows, faster for lean, dim flames. That is a different clock, kept
  empirical. The theory baseline of the inverse model scores worse than a constant guess, which is
  the measurement of this statement.
- **The stored flow of an inverse flame is the oxygen alone**, not the injected stream (the O2 flow
  over the hydrocarbon burn rate is exactly the stoichiometric ratio for every train row). The total
  burner flow of an air-injected flame is 4.3 times the stored value. The forward derives it.
- **The burner follows one heating law for normal and inverse flames.** It warms as
  `T0 + ΔT (1 − exp(−t / 30 s))` with `ΔT = 2012 K · (m_tot / 1 mg/s)^−0.4 · (r_ss / 1 mm)^−0.4`: the
  total flow through the burner cools it and blows the flame off it, and a larger flame is further
  away. Session medians of ln(measured / law) over the peak burner temperatures of some fifty normal
  train flames are −0.08 to +0.03 in five sessions. A lean inverse flame cannot reach the limit under
  this law, which the second generation had to be told.
- **The limit was two settings:** tests end at 505–544 °C in the sessions up to July and at
  454–460 °C in the last two. The model integrates over both.
- **Most of the train score is a handful of regime-level misses,** not session offsets: moving every
  radiative median by a session-sized ±0.4 changes the extinction score by 0.01–0.02, a flame in the
  wrong class costs 0.9. Not modelled: 3 % butane (one session), and the two butane flames at 2 bar
  that cycle between hot and cool and are labelled as flow terminations.
- **Two laws of different kinds make different mistakes.** The hinge law interpolates the lives the
  train table shows; the growth law is a mechanism with few constants. On validation they tie for
  normal flames (0.159 and 0.163). The mean of two forecasts is never scored worse than the mean of
  their scores, and better by a quarter of their mean squared disagreement. *Became:* a release that
  holds both laws and answers with the mean of their forecasts ([card](cards/acme_blend.md)). For
  normal flames it scores 0.155 and is the carried model. For inverse flames the two members share
  their self-extinction law, disagree little, and the blend (0.142) stays behind the better member
  (0.137).

### 6.3 BASS

The flame is a band. The front advances at a rate linear in flow; the visible tip is ahead of it by
an overhang that follows the flow within seconds; the base repeats the front one burn-through time
(6.3 s) later, so after a step down in flow the base keeps the old speed and the flame shortens from
both ends. The published theory gives the same picture and the linear laws. The model and its blind
validation are in [cards/bass_front.md](cards/bass_front.md): half the error of persistence on twelve
unseen burns. The quench boundary, written from one train burn and a boundary published for another
material, does not transfer; withdrawing it showed that it was still worth a little on flames
already burning. What is missing is a class (this sample at these conditions carries a flame, or does
not), not a better number for the boundary.

### 6.4 The open points of sections 4 and 5.4: what this lane did itself

Nothing in the earlier lists is waited for. Where a model needed something, it was built here, or
the model was written to do without it and says so.

| Point | What was done in this lane | What the models do without |
|---|---|---|
| Droplet position (drift out of view) | Not added to the prefixes: it would change the input hash of every FLEX prefix of the shared dataset and break pairing with every earlier release. Four tracker flags were built from the prefix instead and tested (6.1) | A droplet that leaves the view later is not foreseen; stated as a limit |
| The sheet's D0 as a prospective condition | The fourth-generation forward reads `initial_diameter` when a prefix carries it and switches the size flag off when it does not | The carried models do not use it |
| Adjudicated extinction diameters and cool-flame plateaus | For the train runs: extinction diameters read from NASA's test sheets and plateaus measured from the series by `tools/wsp/analysis/`, as measurement, never as an input | Event scores for droplet stages: only the diameter is scored |
| Dataset conflicts (one video under two ids, atmosphere conflicts, zero pressure) | Listed per run in the feedback files; the forwards refuse a prefix with an unusable condition | — |
| Tracker artefacts as measurement status | The validity gate and the four flags, in the forward | Artefact values remain in the scored series |
| ACME script timelines and durations | The control stages a prefix already carries are used (a stop known at the cutoff); the campaign's script times are typed as stop atoms with their frequencies | The duration programmed for a session |
| Meaning of the inverse-flame flow | Verified on every train row; the forward derives the total burner flow from it and the composition | — |
| Burner temperature at ignition, limit setting of the session | A fixed start temperature and a weighted pair of limits (455 and 510 °C) | Both are known before a test and not in the prefix |
| Flames that cycle between hot and cool | Named in the limits of the card | Two train flames are scored as the data label them |
| A BASS front observation | Measured here: track files (`wsp-bass-front-track/1`: tip and base per still in sample widths, the fan history from the video overlay, the day's calibration). The four train tracks are committed; the seventeen validation burns were measured by agents that knew nothing of the model, into files the author has not opened. Reader: `front_tools`; scoring: `wsp-dev front-feedback` | Metres: no run has a length calibration. The forward's metric route exists and is tested on a synthetic fixture |
| Fan calibration below its lowest row | A straight line to the stall point, declared as an assumption; a day with no air values gives no flow speed and the burn is refused (one validation burn) | — |
| A model service | Implemented: `ignis_wsp::WspModelService` (below) | — |
| Pairing fourth-generation droplet cases with older releases | Carry releases: the third-generation numbers on the fourth forward, identical forecasts on every train case | — |
| A longer score horizon for burner flames | Nothing: the evaluator's window (6–120 s) is used as it is | A flame out at 123 s and one held for five minutes are the same case |
| The BASS test burns | Untouched. This lane measured train and validation only, and the author of the model must not measure the test burns | — |

**The model service.** `WspModelService<P>` implements `ignis_core::services::ModelService` for
every literal release of the registry. A release finds its forward by `model_name`; it is read and
verified once per id and kept; a stored run's partition in the release's own split is checked
against the host's scope before the prefix is built, so a test-partition run is refused without its
records being read; a scientific refusal of a forward comes back as an artifact, a release this
build cannot run as an error. One line constructs it:

```rust
let models: Arc<dyn ModelService> = Arc::new(ignis_wsp::WspModelService::new(
    ignis_core::registry::default_root(),
    ignis_wsp::ViewSourcePrefixes::new(ignis_data::views::DataStore::new()),
));
```

`crates/ignis-wsp/tests/service.rs` covers the release list, routing, the single load under
concurrent requests, the scope checks (validation and test refused for a train scope; test only for
an evaluation scope), tampered and mismatched prefixes. A forecast by prefix hash is refused: no
store of admitted prefixes by hash exists in the core crate, and none was added here.

### 6.5 What validation said

One aggregate look per candidate, each candidate committed before its score existed
([PROTOCOL.md](PROTOCOL.md) section 9).

| Candidate | What it was | Validation | Carried | Decision |
|---|---|---|---|---|
| methanol D4 | sharper class, narrower bands, tracker flags | 44.8 µm | 39.5 µm | not adopted |
| methanol D5 | the tracker flags alone | 40.1 µm | 39.5 µm | not adopted |
| heptane D3 | helium law, narrower bands, tracker flags | 57.6 µm | 46.1 µm | not adopted |
| ACME normal, growth clock | published physics plus corrections from train | 0.163 | 0.159 | not adopted (tie) |
| ACME inverse, growth forward | heating law, two-condition weak class, pooled stops | 0.137 | 0.146 | **adopted** |
| ACME normal, two laws together | the hinge law and the growth clock, equal weights | 0.155 | 0.159 | **adopted** |
| ACME inverse, two laws together | second-generation clocks and the growth forward, equal weights | 0.142 | 0.137 | not adopted |
| BASS front D1 | the first BASS model | 0.117 widths | persistence 0.234 | **carried** |
| BASS front D2 | D1 without the oxygen quench boundary | tip identical; track-ended 0.183 against 0.178 | D1 | not adopted |

What held on unseen data: a mechanism written in quantities that mean the same thing in another
session (the heating law, the band-shaped flame), and letting two laws of different kinds answer
together where they really are of different kinds. What did not: anything made sharper or narrower
because the train runs looked that way, a boundary typed as a line from one burn on each side of
it, and averaging two laws that share their main statement.

## 7. The fifth generation (14 November): the mechanism of each experiment, and what validation said

The owner asked for models rebuilt from what each experiment physically does, with as many
parameters as have a physical meaning and none that only match a train table. Five physics
dossiers ([research/](research/) `phys_*.md`) evaluated the published theory of each experiment
against its train runs at literature constants; four of them were re-derived by fresh agents with
their own code ([feedback/](feedback/) `verify_phys_*.md`); every quoted property value was looked
up in standard sources (`research/properties_verified.md`). The review dossier
([dossier/](dossier/README.md)) records what each said and how far it was checked. This section
keeps only what entered a model.

### 7.1 FLEX

- **An unlit methanol droplet evaporates on its stored heat.** When the flame leaves, the liquid is
  near boiling; the latent heat comes out of the liquid, which cools, and the rate falls as
  `K0 / (1 + t/t0)` with `t0 = T* c_l D² / (1.5 K0 L)`: the decay time grows as the square of the
  diameter. With `K0 = 0.30 mm²/s`, `T* = 16 K`, `c_l = 2650 J/kg/K`, `L = 1.118 MJ/kg` from property
  tables and nothing adjusted, the after-flame rates of the train runs at two sizes are reproduced
  about as well as by the three numbers the carried model typed from them. For n-heptane the same
  law fails its size scaling after a cool flame and was not typed.
- **The critical diameter of methanol** was typed from train pairs at 12 to 15 % oxygen and
  extrapolated to 9 mm in air, where physics and earlier experiments say 4 to 4.5 mm. It is now
  capped from 18 % oxygen, where train stops supporting it, and falls with pressure above 1 atm.
- **The cool flame of n-heptane** burns at 0.345 mm²/s on train with no trend in oxygen or pressure
  below 1 atm, and ends at `D/D0 = 0.47 sqrt((0.18 − β)/(pO2 − β))` with β near 0.12 atm of oxygen.
  A flame born below its end diameter dies away within seconds; that replaces the helium patch of
  the fourth generation.
- **Flame cameras** entered the prefix as an observation of the state (a flame frame in the last
  quarter second, or none after five frames).

### 7.2 ACME

- **The plan of a test is known before ignition.** NASA's script parameter file of a test holds the
  stage durations; on train a test that ran to its scripted end stopped at their sum minus 0.4 s on
  the record's clock. The models had guessed stops as atoms typed from train sessions; with the plan
  in the prefix the guess is replaced by the plan. The plan is ingested into a private copy of the
  data root and is not in the shared records.
- **Inverse flames end by oscillation.** 33 of the 34 propane and n-butane train flames that went
  out after 5 s showed bursts of radiant output first; ethane flames and air-injected flames do not
  oscillate. The mixture fraction separates the outcomes far better than the
  total flow (16 flames in the ambiguous range against 45); inside its marginal band a flame at a
  high oxygen flow was held twice in two. That became a second scenario carried at one third.
- **The burner limit changed with the session**: about 500 °C in the normal-flame sessions of March
  to June, about 450 °C in October. A forward cannot see the date, so the weights of the two limits
  are the share of train sessions under each.
- Not typed: the pressure effect moved into the radiative length, which physics requires and one
  train flow contradicts; a pressure term for inverse flames, which made train worse.

### 7.3 BASS

An oxygen balance of the flame with no adjusted constant gives the mean of tip and base speed within
about 10 % on the train windows. The earlier campaign (PSI-26, outside every partition) has narrow
samples, which the train burns lack: there the flame is faster (1.26 to 1.54 times the wide law) and
shorter (0.62 to 0.86). Typed and dropped after one train score each: a slower growth of the flame,
a weaker oxygen dependence, flow paths.

### 7.4 What validation said

One aggregate look per family, on candidates and rules committed beforehand
([PROTOCOL.md](PROTOCOL.md) section 10, [RESULTS_GEN5.md](RESULTS_GEN5.md)).

| Candidate | What it was | Validation | Carried | Decision |
|---|---|---|---|---|
| methanol P1 | self-cooling evaporation, capped critical diameter, cameras | 42.5 µm | 39.5 µm | not adopted |
| heptane P0 | cool-flame laws, dying flame | 47.2 µm | 46.1 µm | not adopted |
| heptane P1 | P0 with cameras | 48.4 µm | 46.1 µm | not adopted |
| ACME normal committee, planned stop known | the carried pair, corrected limit weights, the plan | 0.145 / 0.075 | 0.155 / 0.098 | **adopted** |
| the same on the shared records | | 0.155 / 0.097 | 0.155 / 0.098 | **adopted** (level) |
| ACME inverse committee, planned stop known | the carried law and the hold scenario at 2 : 1, the plan | 0.135 / 0.048 | 0.161 / 0.094 | **adopted** |
| the same on the shared records | | 0.129 / 0.088 | 0.161 / 0.094 | **adopted** |
| BASS front P1 | D1 with the narrow sample corrected | tip 0.114; track ended 0.198 | 0.117; 0.178 | not adopted |

What held on unseen data: knowing the plan of the test, and a second scenario carried with a stated
probability. What was not shown either way: droplet laws rebuilt from mechanism against the laws
typed from train (1 to 3 µm apart, inside intervals of ±5 to ±10 µm). What did not help: the flame
cameras, and a correction for the narrow sample taken from another campaign.

