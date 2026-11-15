# DAFT / DAFT-2 (PSI-47): what a particle counter reads on cabin air and on test dust in orbit. A W0 law

Author: claude-fable-5-2, physicist-author of the seventh generation, 15 November 2026. Zero-shot. DAFT is
held whole: no DAFT outcome was opened. Siblings: [phys_samer_w0.md](phys_samer_w0.md),
[phys_same_w0.md](phys_same_w0.md) (the smoke experiments this instrument check was flown for).

## 0. The result in one screen

DAFT burned nothing. A condensation counter (the P-Trak, modified to return its alcohol by capillarity)
and a light-scattering photometer (the DustTrak) sampled the same air: filtered air, ISS cabin air, cabin
air while Velcro was being torn open, and Arizona road dust blown into a 15 litre bag. The engineer asks
for the number concentration `N` the counter reads, given what the photometer read on the same air.

The two readings are tied by one quantity, the mean mass of a particle:

```text
N = c / m_p = 6 c / (pi rho D30^3)
```

`c` the mass concentration, `D30` the diameter of average mass of what both instruments see. The forecast
is a statement about `D30` for each kind of air:

| Air | What sets it | Forecast of `N` (median) | 90 % band |
|---|---|---|---|
| Filtered (HEPA) | nothing gets through; only false counts | 0.05 per cm3 | 0.004 to 0.6 |
| Cabin | source (two crew, equipment) against the filter loop; no settling in orbit | 1.5 per cm3 at a DustTrak reading of 0.003 mg/m3 (1.1 at the floor of 0.001, 2.2 at 0.01) | a factor 12 either way |
| Cabin, Velcro being separated | coarse fibre fragments, few fines | 8 per cm3 at 0.03 mg/m3 (4.6 at 0.01, 15 at 0.1) | a factor 12 |
| Road dust in a bag, 2.5 um inlet cut | the fine tail of a mineral dust, `D30` = 0.9 um | 900 per cm3 for each mg/m3 read (100 at 0.1, 8000 at 10, 37 000 at 50 mg/m3) | a factor 4.4 |

For comparison, room air on the ground carries several thousand particles per cm3 at 0.02 mg/m3: the
count per unit mass in orbit is about 500 times lower, because the combustion and outdoor particles that
make the number on the ground are absent and the coarse particles that make the mass do not settle.

## 1. Mechanism from first principles

### 1.1 Two instruments, two moments

The counter saturates the sample with isopropanol vapour, cools it, and every particle above about 20 nm
grows into a droplet that is counted singly: it reads the zeroth moment, whatever the size. The photometer
measures light scattered at 90 degrees by all particles in its beam; for a fixed size distribution that is
proportional to mass, and the factory constant is that of Arizona road dust. So the pair (`N`, `c`) is
the pair (M0, M3) of the smoke experiments, and their ratio is a size:
`D30 = (6 c / (pi rho N))^(1/3)`.

Dimensionless groups:

| Group | Definition | Values | Decides |
|---|---|---|---|
| Settling against ventilation | `v_s / (H lambda_v)`, settling speed over room height times air-change rate | ground room, one air change per hour: 1 at 5 um, 4 at 10 um; orbit: 0 at every size | on the ground coarse particles leave by settling before the filter sees them; in orbit every size waits for the filter |
| Bag settling number | `v_s t / H_bag` | ground: 1 after 20 min for 1.5 um mineral grains in a 25 cm bag, after 1 min for 7 um; orbit: 0 | on the ground a dust bag loses its coarse end within minutes; in orbit it does not |
| Filter loop | `N = S / (eta Q)`, residence time `V / Q` | Destiny: about 100 m3 through 0.2 m3/s of HEPA, 8 min | cabin level = source over filtered flow |
| Coagulation number | `K N t` | 3.5e-16 m3/s * 1e10 per m3 * 600 s = 2e-3 | the dust does not coagulate; `N/c` is constant in a bag |
| Bond number of the alcohol return | `rho g L^2 / sigma` | ground: 1 at L = 1.7 mm; orbit: 0 | on the ground gravity drains the condenser; in orbit only a wick does. This is what DAFT tested |
| Coincidence | `N * Q_s * t_pulse` | below 0.1 up to 1e5 per cm3 | the counter is linear over the whole range of these tests |

### 1.2 Cabin air: a source against a filter

No combustion, no outdoor air, no cooking fumes: the fine-particle sources of a room on the ground are
absent. What remains is the crew (skin and clothing fragments) and equipment.

- A person in light activity sheds of order 5e5 particles of 0.3 um and larger per minute (clean-room
  practice: 1e5 at rest, several 1e6 when walking). Two crew (February 2005): 1.7e4 per second.
- The US segment draws about 0.25 m3/s through HEPA elements (my recollection of the Destiny cabin fan,
  about 0.2 m3/s, plus Node 1; class J, factor 2).
- Steady state for 0.3 um and larger: `N = S/Q` = 7e4 per m3, 0.07 per cm3.
- The counter also sees 20 to 300 nm. Equipment, and air exchanged with the Russian segment, whose
  filters are coarse, add a multiple that I cannot derive: I take a factor 20 (range 5 to 100).

Result: 1.5 per cm3 (1.5e6 per m3), ln-sd 1.5. On the ground the same counter reads thousands.

The mass. A crew member puts of order 0.3 mg/min of particulate into the air (a fifth of the daily skin
and lint loss); two crew over 0.4 m3/s of total filtered flow give 0.025 mg/m3, of which the photometer,
which responds to 0.1 to 10 um, sees perhaps a tenth: 0.003 mg/m3, three times its resolution. With the
count above, `D30` = 1.4 um for unit-density organic matter (1300 kg/m3): the mean cabin particle is ten
times larger than the mean room particle on the ground (0.15 to 0.2 um).

`N` and `c` are only loosely tied in cabin air (different particles make the number and the mass), and
`c` is read at the instrument's floor: I let `N` rise as `c^0.3`.

### 1.3 Velcro

Tearing hook-and-loop tape breaks nylon hooks and sheds fibres and fragments, mostly above a few
micrometres. The photometer sees them (mass rises tenfold), the counter gains a few particles per cm3
averaged over a window of 20 s or more, on top of the cabin level. I take 8 per cm3 at 0.03 mg/m3 and
`N ~ c^0.5`.

### 1.4 Road dust in a bag

Arizona test dust (ISO 12103-1, quartz and feldspar, 2650 kg/m3) has a volume median of 4.5 um (ultrafine
grade) or more, and a broad distribution (`sigma_g` about 2). The 2.5 um impactor cuts at an aerodynamic
diameter, that is at a geometric diameter of `2.5 / sqrt(2.65)` = 1.54 um. For a lognormal dust of count
median 1.07 um (`4.5 exp(-3 ln^2 2)`) the fraction below the cut holds 70 % of the particles and 6 % of
the mass, and its diameter of average mass is 0.97 um. Real crushed dusts have more fines than a
lognormal: I take `D30` = 0.85 um (factor 1.3). Then

```text
N / c_true = 6 / (pi * 2650 * (0.85e-6)^3) = 1.2e15 per kg = 1200 per cm3 for each mg/m3
```

The photometer was calibrated on the respirable fraction of the same dust, slightly coarser than what
passes a 2.5 um cut, and finer particles scatter more per unit mass: it over-reads this fraction by about
1.3. Per unit of DustTrak reading: **900 per cm3 for each mg/m3** (9e14 per kg), ln-sd 0.9: 0.8 from
`D30^3`, 0.3 from the calibration, 0.3 from whether the counter draws through the same cut (if not, it
also counts the 30 % of grains above it, which in orbit stay airborne).

Within a bag `N/c` should not drift: nothing settles, nothing coagulates. I type `N ~ c^0.95`, the 0.05
for coincidence at the top of the range.

### 1.5 Filtered air, and the instrument itself

Behind a HEPA capsule the true concentration is zero. The reading is the false-count rate: alcohol
droplets shed from a flooded wick or optics would give counts with no particles, which is the second
failure mode NASA named. A counter that works reads 0 with an occasional 1: I put the median of a window
mean at 0.05 per cm3 with a wide band, and 15 % of the weight in a tail for a counter that misbehaves.

## 2. The forecast target (from the engineer's note)

`docs/wsp/feedback/daft_feasibility.md`, sections "For the physicist", 1 to 3.

| Target | Definition | Unit | Score | A forecast may know |
|---|---|---|---|---|
| `number_concentration` | what the modified P-Trak reads on the sample: mean over a window (2005) or one reading (2006 bags) | 1/m3 | quantile CRPS of ln(1 + N / 1e6 m^-3), levels 5/25/50/75/95 % | the kind of air (hepa, cabin, velcro, bag), location, bag number and elapsed time, window duration, the DustTrak mass concentration on the same air (kg/m3), the DustTrak inlet, the analyst's assumed density |

I use the kind of air and the DustTrak reading. Location, bag number, elapsed time and window duration
carry no term: I have no physical reason for one.

## 3. The law

```text
x = max(DustTrak reading, 1e-9 kg/m3)                      the floor is the instrument's resolution

hepa     ln N = ln(5.0e4)                                   ln-sd 1.5
cabin    ln N = ln(1.5e6) + 0.30 ln(x / 3e-9 kg/m3)         ln-sd 1.5
velcro   ln N = ln(8.0e6) + 0.50 ln(x / 3e-8 kg/m3)         ln-sd 1.5
bag      ln N = ln(9.0e8) + 0.95 ln(x / 1e-6 kg/m3)         ln-sd 0.9
N in 1/m3; tails: 15 % of the weight in a component 2.5 times wider
```

How unsure the theory is: for the bag, the size of the fine tail of the dust (a factor 1.3 in `D30`,
2.2 in `N`); for cabin air, the fine-particle sources I could not derive (a factor 4) on top of the
shedding rate and the filtered flow (a factor 2 each); for filtered air, whether the counter is quiet.
The score compresses small counts (`ln(1 + N per cm3)`), so the bag rows, 42 of 53, carry it.

## 4. Constants

Class: P property table, T theory, L published about this experiment, J judgement.

| Symbol | Value | Class | Uncertainty | Origin |
|---|---|---|---|---|
| Counter lower size | 20 nm | P | - | P-Trak specification |
| Photometer resolution | 1e-9 kg/m3 (0.001 mg/m3) | P | - | DustTrak specification; the engineer's note says the same |
| Dust density | 2650 kg/m3 | P | 3 % | quartz and feldspar |
| Dust volume median, `sigma_g` | 4.5 um, 2.0 | P/J | 1.5 um, 0.3 | ISO 12103-1 A1 from memory; the grade flown is not stated |
| Cut of the impactor, geometric | 1.54 um | T | 5 % | 2.5 um aerodynamic over the square root of 2.65 |
| `D30` of the dust below the cut | 0.85 um | T/J | factor 1.3 | 1.4: 0.97 um for a lognormal, less for a real dust |
| Photometer over-reading of the fine fraction | 1.3 | J | 1.0 to 2.0 | 1.4 |
| `N / c` in the bag | 9.0e14 per kg | T/J | factor 2.5 | the three lines above |
| Exponent in the bag | 0.95 | T | 0.85 to 1.0 | no settling, no coagulation; coincidence |
| Shedding, 0.3 um and larger | 5e5 per person per minute | J | factor 3 | clean-room practice, from memory |
| Crew | 2 | L | - | Expedition 10, February 2005 |
| Filtered flow, US segment | 0.25 m3/s | J | factor 2 | recollection of the cabin fan flow |
| Multiple for 20 to 300 nm | 20 | J | 5 to 100 | 1.2: not derivable |
| Cabin `N` | 1.5e6 per m3 at 3e-9 kg/m3 | T/J | factor 4.5 (ln-sd 1.5) | 1.2 |
| Cabin exponent on `c` | 0.30 | J | 0 to 0.6 | 1.2 |
| Velcro `N` | 8.0e6 per m3 at 3e-8 kg/m3, exponent 0.50 | J | factor 4.5 | 1.3 |
| Filtered-air `N` | 5.0e4 per m3 | J | factor 4.5 | 1.5 |

## 5. Registered expectations

Written before any comparison. `N` in particles per cm3 (1e6 per m3); quantiles of the core distribution.

| Air | DustTrak reading, mg/m3 | 5 % | 25 % | Median | 75 % | 95 % | `D30` implied, um |
|---|---|---|---|---|---|---|---|
| hepa | any | 0.004 | 0.018 | 0.05 | 0.14 | 0.59 | - |
| cabin | 0.001 or below | 0.09 | 0.39 | 1.08 | 2.97 | 12.7 | 1.1 |
| cabin | 0.003 | 0.13 | 0.55 | 1.5 | 4.1 | 17.7 | 1.4 |
| cabin | 0.010 | 0.18 | 0.78 | 2.15 | 5.9 | 25 | 1.9 |
| velcro | 0.010 | 0.39 | 1.7 | 4.6 | 12.7 | 54 | 1.5 |
| velcro | 0.030 | 0.68 | 2.9 | 8.0 | 22 | 94 | 1.8 |
| velcro | 0.100 | 1.2 | 5.3 | 14.6 | 40 | 172 | 2.2 |
| bag | 0.1 | 23 | 55 | 101 | 185 | 444 | 0.89 |
| bag | 1 | 205 | 490 | 900 | 1650 | 3950 | 0.93 |
| bag | 10 | 1820 | 4360 | 8000 | 14 700 | 35 200 | 0.97 |
| bag | 50 | 8400 | 20 100 | 36 900 | 67 700 | 162 000 | 1.0 |

(`D30` implied: from the median `N` and the reading, density 1300 kg/m3 for cabin and Velcro, 2650 for
the dust.)

Ratios and orderings:

| Statement | Forecast |
|---|---|
| Order of `N` in session D2 | hepa < cabin < velcro, each step a factor 5 or more |
| Velcro window over the cabin window next to it | 3 to 8 |
| `N` per unit DustTrak reading within one bag, first reading to last | constant within a factor 1.5 |
| The same between the three bags | within a factor 2 |
| Count per unit mass, cabin air in orbit over room air on the ground | 1/500 (1/100 to 1/3000) |

## 6. What would show the law is wrong

1. **The bag ratio is not a constant near 900 per cm3 per mg/m3.** A ratio below 150 or above 5000, or
   one that changes by more than a factor 3 along a bag, means that the two instruments did not see the
   same dust (different inlets, losses in a hose, the impactor passing coarse grains by bounce) or that
   the dust was not the grade I assumed. A steady fall of `N/c` with time would point at loss of fines
   to the charged Mylar wall; a rise, at coarse grains being lost in orbit after all.
2. **Undisturbed cabin air reads above 50 per cm3.** Then there is a source of fine particles I did not
   count, or the HEPA loop is not the sink for the air sampled; the picture "a spacecraft cabin is a
   clean room with people in it" fails.
3. **Filtered air reads above 1 per cm3, or a Velcro window reads no more than the cabin window beside
   it.** The first is a counter making false counts in orbit, the failure DAFT was flown to rule out,
   and it would put every SAME number concentration in doubt; the second would mean the counter does
   not respond to a real change that the photometer sees.

## 7. Interface for the strict forward (`docs/wsp/research/law_forward.md`)

### 7.1 Declaration for the slug `daft`

Four features. The three indicators are derived by the engineer from `air` (hepa is the reference, all
at 0). `dusttrak_mass_floored` is `max(dusttrak_mass_concentration, 1e-9 kg/m3)`: the forward's ln
transform refuses zero and negative readings, which a photometer at its floor does give; the floor is
the instrument's resolution, not a tuned number.

| # | Name | Unit | Transform | Centre | Scale | Domain | Meaning |
|---|---|---|---|---|---|---|---|
| 0 | `is_cabin` | 1 | identity | 0 | 1 | 0 to 1 | cabin air through the hose |
| 1 | `is_velcro` | 1 | identity | 0 | 1 | 0 to 1 | cabin air while Velcro is separated |
| 2 | `is_bag` | 1 | identity | 0 | 1 | 0 to 1 | road dust in a bag |
| 3 | `dusttrak_mass_floored` | kg/m3 | ln | 1e-6 | 1 | 1e-9 to 2e-4 | `z = ln(x / 1 mg/m3)`, -6.9 to 5.3 |

Target: `number_concentration` (positive, 1/m3, lane-local).

The slope on the reading differs by kind of air. A pair of hinges switched by an indicator does that:
`s relu(z + 12 (i - 1)) - s relu(-z + 12 (i - 1))` equals `s z` when `i` = 1 and 0 when `i` = 0, for
`|z|` below 12. Three pairs, six units.

### 7.2 Literal assignments

```text
# daft.wsp W0: zero-shot law of docs/wsp/research/phys_daft_w0.md. Author claude-fable-5-2.
# features: 0 is_cabin, 1 is_velcro, 2 is_bag, 3 dusttrak_mass_floored. Filtered air is the reference.
x.is_cabin.centre = 0.0
x.is_cabin.scale = 1.0
x.is_cabin.lo = 0.0
x.is_cabin.hi = 1.0
x.is_velcro.centre = 0.0
x.is_velcro.scale = 1.0
x.is_velcro.lo = 0.0
x.is_velcro.hi = 1.0
x.is_bag.centre = 0.0
x.is_bag.scale = 1.0
x.is_bag.lo = 0.0
x.is_bag.hi = 1.0
x.dusttrak_mass_floored.centre = 0.000001
x.dusttrak_mass_floored.scale = 1.0
x.dusttrak_mass_floored.lo = 0.000000001
x.dusttrak_mass_floored.hi = 0.0002

# ln(N / 1 per m3). b0: filtered air, 5e4 per m3.
number_concentration.median.b0 = 10.82
# value of each kind of air extrapolated to a reading of 1 mg/m3, minus b0:
# cabin 1.5e6 at 3e-9 kg/m3 with exponent 0.30; velcro 8e6 at 3e-8 with 0.50; bag 9e8 at 1e-6
number_concentration.median.skip[0:4] = [5.144, 6.828, 9.798, 0.0]
# units 0, 1: cabin, exponent 0.30 on the reading
number_concentration.median.W1[0:2, 0:4] = [[12.0, 0.0, 0.0, 1.0], [12.0, 0.0, 0.0, -1.0]]
number_concentration.median.b1[0:2] = [-12.0, -12.0]
number_concentration.median.w2[0:2] = [0.30, -0.30]
# units 2, 3: velcro, exponent 0.50
number_concentration.median.W1[2:4, 0:4] = [[0.0, 12.0, 0.0, 1.0], [0.0, 12.0, 0.0, -1.0]]
number_concentration.median.b1[2:4] = [-12.0, -12.0]
number_concentration.median.w2[2:4] = [0.50, -0.50]
# units 4, 5: bag, exponent 0.95
number_concentration.median.W1[4:6, 0:4] = [[0.0, 0.0, 12.0, 1.0], [0.0, 0.0, 12.0, -1.0]]
number_concentration.median.b1[4:6] = [-12.0, -12.0]
number_concentration.median.w2[4:6] = [0.95, -0.95]
# spread: 1.5 for filtered air, cabin and velcro; 0.9 for the dust
number_concentration.spread.b0 = 1.5
number_concentration.spread.skip[0:4] = [0.0, 0.0, -0.6, 0.0]
number_concentration.tail_weight = 0.15
number_concentration.tail_width = 2.5
```

Check values: cabin at the floor (1e-9 kg/m3): `z` = (1, 0, 0, -6.908); unit 1 gives 6.908, the others
0; `ln N` = 10.82 + 5.144 - 0.30 * 6.908 = 13.892, `N` = 1.08e6 per m3. Bag at 1e-5 kg/m3: `z` =
(0, 0, 1, 2.303); unit 4 gives 2.303; `ln N` = 10.82 + 9.798 + 0.95 * 2.303 = 22.806, `N` = 8.0e9 per m3;
ln-sd 0.9. Filtered air at any reading: 5.0e4 per m3, ln-sd 1.5.

## 8. Sources, limits, exposure

**Sources.** Card `docs/wsp/atlas/cards/PSI-47.json`; `docs/wsp/atlas/INVESTIGATIONS.md`, section PSI-47;
`docs/wsp/feedback/daft_feasibility.md`, sections "For the physicist", 1, 2 and 3. W. C. Hinds, *Aerosol
Technology*, 2nd ed. (settling, aerodynamic diameter, condensation counters, photometers, coincidence,
Hatch-Choate relations). ISO 12103-1 (test dust grades) and the instrument specifications (TSI P-Trak
8525: 20 nm, up to 5e5 per cm3; TSI DustTrak 8520: 0.001 to 100 mg/m3, calibrated on A1 test dust), from
memory. Clean-room shedding rates: the figures commonly quoted in contamination-control practice, from
memory.

**Could not find or check.** The grade of the road dust and the mass put in each bag; whether the counter
sampled through the same 2.5 um cut as the photometer; the size distribution of the dust below 1 um; the
filtered flow of the US segment in 2005 and the particle removal of the Russian segment's filters; any
measurement of fine-particle sources in a spacecraft cabin that predates DAFT; the counter's false-count
rate with the capillary condenser. No web source was opened for DAFT.

**Exposure.** The card (allowed) carries NASA's findings: the modified counter worked reliably; cabin air
carries very little particulate; the count-to-mass ratio in orbit fell below the ground relation, read as
larger particles staying airborne; only three test points were run and the bag electronic data were
lost. These told me the sign of two forecasts before I derived them (a clean cabin, a low count per unit
mass); no number came from them, and the card holds none. The engineer's note holds no outcome in the
sections I read; its section 7 ("what was seen that should not have been") I did not read. My pretraining
may include the DAFT reports; I cannot recall a number from them, and the cabin median of 1.5 per cm3 is
the source-over-filter estimate of 1.2, not a remembered value.

**The earlier author's kernel** (`crates/ignis-wsp/src/property_aerosol_v2.rs`). It propagates a supplied
aerosol through a stirred chamber with coagulation, settling and wall loss, and observes it with an
extinction detector for Rayleigh-size spheres. For DAFT: coagulation is irrelevant (1.1), settling is
zero in orbit, the dust is neither a sphere nor in the Rayleigh range for any light source (grains of
1 um), and neither a condensation counter nor a 90 degree photometer exists in it. The question DAFT
poses, what one instrument reads given the other, needs the mean particle mass of each kind of air,
which is a statement about sources and size distributions that the kernel takes as an input. Its
drift-diffusion wall flux would be the right tool for one thing this note leaves out: the loss of fine
dust to the wall of a charged bag.
