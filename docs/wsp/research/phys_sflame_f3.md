# The burner-flame committee after s-Flame: what F3 needs

> Written by an author model (claude-fable-5-2) on 15 November 2026, in two parts and in this order.
> **Part A** (zero-shot) was written and saved before any per-test outcome of a helium-diluted, hydrogen or
> hydrogen/methane test was opened. **Part B** (data-informed) was written afterwards, from the s-Flame train days
> only. Nothing fitted, no search. It is a proposal: no number here is a weight until it has been typed into a table
> or header file and assembled.
> Origin classes: **P** property table, **T** theory, **L** published experiment, **J** judgement,
> **D** typed after reading train data.

# Part A. Helium, hydrogen and hydrogen/methane blends, zero-shot

## A0. What was known when this part was written

Read: `PROTOCOL.md` 13.1 with its outcome; `sflame_inputs.md`; `sflame_transfer_result.md`; the forwards `fuel.rs`,
`flame6.rs`, `flame7.rs`, `committee3.rs`; the F2 tables and header; the **inputs** of every constant-flow s-Flame
test of the train and validation days through `inputs.jsonl` (a file that holds no outcome): streams, flows, pressure,
adiabatic temperature, mixture fraction, planned stop, limit.

Not opened: `cases_train.jsonl`, `train_days_cases.jsonl`, `train_study.json`, `endpoints_train.jsonl`, any validation
outcome, the test days.

**This part is not blind to aggregates.** The two feedback files I was asked to read first state, for the train days:
hydrogen cases end mostly by operator termination (55 of 70), "mostly the 250 °C stop"; the median flame end is 12 s
for hydrogen, 3.4 s for the blends, 2.3 s for methane in air (all within 4.4 s); the endpoint label disagrees with the
script's account on 11 of 28 hydrogen tests. I knew these when typing. Where a number below would have been different
without them, it says so; none was adjusted to them.

## A1. Helium as the diluent

s-Flame's helium days replace nitrogen in the fuel bottle **and** in the chamber: the burner stream is 20 % ethylene or
45 % methane (or hydrogen, or a blend) in helium with no nitrogen, and the ambient is oxygen in helium.

**What changes, in the order of size.**

1. **Mass.** The laws are written in mass: `Y_F` from the mole fraction, `m_tot = m_F / Y_F`, `L` from the mass-based
   `Z_st`. With helium, 20 % ethylene by mole is 64 % by mass (30 % in nitrogen), and 30 % oxygen by mole is 77 % by
   mass (33 %). `Z_st` already arrives computed with the true masses. The forward computes `Y_F` with `phys.mw_n2`;
   that constant has to become the diluent's molar mass, 4.0026 g/mol (P). At one fuel mass flow, fuel fraction and
   oxygen fraction the product `Y_F L` is 2.5 times larger in helium (ethylene 20 %, 30 % O2: 0.194 against 0.078;
   methane 45 %, 21 % O2: 0.154 against 0.054), so the reduced flow `Φ ∝ 1/(Y_F L)²` is six to eight times smaller: a
   helium flame is a small flame.
2. **Far-field diffusion.** The growth clock is transient diffusion of oxygen through the cold ambient. In a binary
   mixture that is the binary coefficient whatever the composition: helium–nitrogen 0.703, nitrogen–nitrogen 0.209
   cm²/s at 298 K (P, NIST TN 2279); oxygen–helium about 0.73 (P, from memory). Ratio **3.5** (3.2 to 3.7). The clock
   runs 3.5 times faster.
3. **ρD at the flame.** Density falls as the diffusivity rises. In the ambient, `ρD` is 1.19 (21 % O2) to 1.48 (30 %)
   times the nitrogen value (molar mass 9.9 to 12.4 against 29, diffusivity ×3.5). At the flame, where a quarter of the
   moles are CO2 and H2O and oxygen is a trace, Blanc's law gives 0.42 to 0.47 cm²/s and a molar mass of 9.5 to 11:
   0.8 to 0.85 times the nitrogen value. Typed: **1.15** (0.85 to 1.5) (T).
4. **Lewis number of oxygen.** Heat leaves a helium flame faster than oxygen arrives. Thermal diffusivity of the ambient
   (Wassiljewa mixture rule, 300 K): 0.98 cm²/s at 30 % O2, 1.18 at 21 %; so `Le_O2` is 1.3 to 1.6 against 1.06 in
   nitrogen. For a normal flame the sheet temperature is then below the adiabatic one (T; the mirror image of the
   inverse-flame formula of `phys_acme_inverse.md`):

   ```text
   T_f − T_0 = η (T_ad − T_0),      η = (1 − (1 − Z_st)^(1/Le)) / Z_st
   ```

   With `Le` = 1.45 against 1.06, η falls from 0.95 to 0.72 at `Z_st` 0.18 to 0.26. The laws were typed with nitrogen's
   0.95 inside them, so what the forward needs is the ratio: a **relative** Lewis number **1.37** (1.25 to 1.5) (T, P).
   The helium flames of s-Flame, tabulated at 2440 to 2660 K, burn as nitrogen flames of 1900 to 2090 K would.
5. **Critical flux.** Two derivations disagree. Radiating column and temperature at extinction hardly depend on the
   bath gas, and the cooler flame is allowed a smaller radiative fraction: −0.15 to +0.1 in ln. The conduction–radiation
   balance of `phys_acme_normal.md` (`m″_cr = ρD Y_F L / (Θ ℓ*)`) gives +0.4 to +0.8, because `Y_F L / Θ` is three times
   its nitrogen value and the radiative length grows only with the square root or the first power of the
   conductivity. Typed: **+0.3** (−0.2 to +0.8) (J). This is the least known number of the helium row.
6. **Burner heating.** The gas between the burner and the flame is the burner stream, and its conductivity is 2.8
   (45 % methane in helium) to 3.6 (20 % ethylene in helium) times that of nitrogen (P, Wassiljewa, 300 K). Conduction
   to the sphere rises in proportion; the screening by the flow does not change much, because `c_p / k` of a helium
   stream is 0.8 of nitrogen's, so the mass flow remains the right measure of the blowing. The same holds for hydrogen
   in the stream (A2). See A4 for the one rule that covers both.

**What the hinge law can be given.** It reads a fuel mass flow and an oxygen mole fraction and knows no mass fraction.
Its flow coordinate is shifted by the logarithm of the ratio of reduced flows of 1 and 3 with the flux of 5:
2 ln(0.078 / 0.194) − 2 ln 1.15 + 0.3 = −1.8 for ethylene and −2.1 for methane; typed **−1.9** (−2.4 to −1.4) (T). Its
time is shifted by ln(1 / 3.5) = **−1.25** (T). These two numbers make the hinge member say what the growth member
says; they add no independent knowledge, and for helium the committee is in effect one law. That is stated, not hidden.

**What is not carried.** The thermal weakness of 4 enters only where the laws read a temperature: the weak-flame class
and the refusal bound. The radiative clocks read the oxygen mole fraction as their measure of flame strength, and I
did not translate a helium flame into an "equivalent oxygen fraction": two more judgements for one mechanism.

## A2. Hydrogen

For a normal flame the fuel's own diffusivity moves neither the flame nor its temperature (`phys_spherical_fuels.md`
1.2), so hydrogen's Lewis number of 0.29 is not what makes it different. Four things are.

1. **Mass again.** 45 % hydrogen in nitrogen is 5.6 % by mass. The growth law carries this through the molar mass
   (2.016). The hinge law cannot (finding 3 of the first dossier).
2. **Oxygen demand and radiation: the critical flux.** Per megajoule a hydrogen flame makes 4.1 mol of water and no
   CO2. With Planck-mean coefficients at the extinction temperatures (hydrocarbons 1130 K; hydrogen about 1000 K),
   the radiative loss per unit heat release is 0.34 of a hydrocarbon's (0.44 at equal temperature). The lower
   extinction temperature allows a radiative fraction 1.15 times larger. Critical heat-release flux: 0.30 of the
   hydrocarbon value; in fuel mass, with 119.9 against 47.5 kJ/g: 0.12, a shift of ln m″_cr of −2.1. With the oxygen
   rule and equal temperatures it is −1.6. Typed for the growth members: `fuel.s` = **+1.9** (1.5 to 2.3) (T, P): a
   critical flux of 0.027 g m⁻² s⁻¹, 3.2 kW m⁻².
3. **The same in the hinge law.** `Φ ∝ m_F m″_cr σ²` gives 2 × 0.757 − 1.9 = −0.4 for undiluted hydrogen; the missing
   mass-fraction geometry adds +0.45 at 45 % hydrogen in nitrogen and +1.3 at 15 %. Typed for the hinge members:
   `fuel.s` = **0.0** (−0.5 to +1.0) (T, J): the two cancel at the composition flown most.
4. **The kinetic limit.** The weak-flame class is a logistic near 1740 to 1750 K, an alkane number. Hydrogen burns
   far below that: its crossover temperature at 1 atm is near 950 to 1000 K and its lean limit flames are near 1000 K.
   s-Flame flew 15 % hydrogen at 1188 K. Typed: `fuel.dtad_k` = **+600 K** (450 to 700) (P, J), added to the
   temperature the weak-flame law reads, which puts its centre at 1140 to 1150 K; refusal below **1100 K**.
5. **Burner heating.** Hydrogen conducts as helium does: 0.183 W m⁻¹ K⁻¹ at 300 K, 7.0 times nitrogen (P).

Coordinates: (1, 0) in both groups (no butane term, no programmed-stop weight). `fuel.dosc` 0.

## A3. Hydrogen/methane blends

Two bottles in nitrogen (44.4 and 66.7 % hydrogen in methane) and two in helium (25/20/55 and 20/25/55). A blend is
not a seventh fuel: it is a mixture of two rows.

- **Molar mass**: mole-weighted, exact (P).
- **Everything else** (`fuel.s`, both coordinate pairs, `fuel.dosc`, `fuel.dtad_k`, `fuel.tad_min_k`): weighted with
  each species' **share of the oxygen demand**, `ω_i = y_i σ_i / Σ y_j σ_j`, `y` the mass fractions within the fuel.
  The laws are written per unit of oxygen consumed or heat released, and these shares are within 4 % of the
  heat-release shares. Hydrogen's share is 0.17 (44.4 % bottle), 0.33 (66.7 %), 0.24 and 0.17 (helium bottles).
  Checked against the direct calculation for the growth shift: linear mixing gives 0.37 and 0.68, the critical
  heat-release flux and heating value of the mixture give 0.29 and 0.57. Within 0.1 (T).
- **Conductivity** of the stream: per species, by the rule of A4.
- The oxygen-demand shares need σ per row: a new typed column (P).

## A4. The structure the forward needs

| New | Kind | Meaning |
|---|---|---|
| `diluent_species` | condition, text | balance gas of the burner stream and of the ambient: `nitrogen` or `helium`. Absent: nitrogen. Two different gases, or a mixture: refusal |
| `fuel_species` = `a+b` | condition, text | a blend of two rows of the fuel table |
| `fuel_blend_x` | condition | mole fraction of the first-named species within the fuel; required with a blend |
| `x_o2_role_b`, `tad_b`, `zst_b` | conditions, optional | a second candidate for the oxygen fraction with its temperature and mixture fraction |
| diluent table `dil.*` | tensors, in each member like the fuel table | rows 0 nitrogen, 1 helium |
| `fuel.sigma`, `fuel.dtad_k`, `fuel.k_rel` | three new columns of the fuel table | |
| `bt.kappa_exp` | scalar in each member | exponent of the stream conductivity in the heating law |
| `mix.o2_b_w` | scalar in the committee | weight of the second oxygen candidate |

Computation, in the order the members run (`k` fuel row or blend, `d` diluent row):

```text
row        a blend: every column of the fuel table mixed with ω (molar mass: mole-weighted) before anything else
Y_R        with dil.mw_g_mol[d] in place of phys.mw_n2
T_1        = T_0 + η (tad − T_0),  η = (1 − (1 − zst)^(1/dil.le_o2[d])) / zst,  T_0 = 298.15 K;  dil.le_o2 = 1: T_1 = tad
refuse     T_1 < fuel.tad_min_k
c_tad      from T_1 + fuel.dtad_k      (the weak-flame law, the burner feature, and the domain bound on c_tad)
r_ss       with se.rho_d × dil.rho_d[d]                     (both members: the growth law and the burner feature b_rss)
growth     ln m_cr += dil.dflux[d];      ln D += ln dil.d_far[d]
hinge      c_logm += dil.dlogm[d] (with fuel.s, in se.tau only);   ln tau_R += dil.dlnr[d]
burner     ln dT_eq += bt.kappa_exp × ln(1 + Σ_i x_i (k_rel,i − 1))      i over the species of the burner stream:
           the fuel species with fuel.k_rel, the diluent with dil.k_rel
two O2     P = (1 − mix.o2_b_w) P(x_o2_role, tad, zst) + mix.o2_b_w P(x_o2_role_b, tad_b, zst_b); a refusal of either
           refuses the forecast
```

The stream-conductivity rule deserves its reason. Heat reaches the sphere by conduction through the burner stream, so
the input scales with its conductivity. A linear mole-fraction mixture overstates the conductivity of a light gas in a
heavy one by 1.3 to 1.4 (5.0 against 3.6 for 80 % helium; 3.7 against 2.9 for 45 % hydrogen in nitrogen), and the
equilibrium temperature rises less than the input because the sphere's own radiation grows as T⁴. One exponent
carries both: **0.5** (0.3 to 0.8) (J). It gives 2.2 for 20 % ethylene in helium, 1.9 for 45 % methane in helium and
for 45 % hydrogen in nitrogen, 2.5 for 45 % hydrogen in helium. The heating time constant is left at 30 s; with a limit
of 250 °C only the early slope matters, and the slope is what the factor multiplies.

## A5. Every typed number

| Tensor | Nitrogen | Helium | Class | Range (helium) |
|---|---|---|---|---|
| `dil.admit` | 1 | 1 | | |
| `dil.mw_g_mol` | 28.013 | 4.0026 | P | exact |
| `dil.rho_d` | 1.0 | 1.15 | T | 0.85 to 1.5 |
| `dil.d_far` (growth) | 1.0 | 3.5 | P | 3.2 to 3.7 |
| `dil.le_o2` | 1.0 | 1.37 | T, P | 1.25 to 1.5 |
| `dil.dflux` (growth) | 0.0 | 0.3 | J | −0.2 to +0.8 |
| `dil.dlogm` (hinge) | 0.0 | −1.9 | T | −2.4 to −1.4 |
| `dil.dlnr` (hinge) | 0.0 | −1.25 | T | −1.35 to −1.15 |
| `dil.k_rel` | 1.0 | 5.8 | P | 5.6 to 6.0 |

| Fuel-table column | ethane | propane | butane | ethylene | methane | hydrogen | Class |
|---|---|---|---|---|---|---|---|
| `fuel.sigma` | 3.725 | 3.628 | 3.579 | 3.422 | 3.989 | 7.936 | P |
| `fuel.dtad_k` | 0 | 0 | 0 | 0 | 0 | 600 | P, J |
| `fuel.k_rel` | 1 | 1 | 1 | 1 | 1 | 7.0 | P; the hydrocarbons are 0.8 to 1.3 and are left at 1 so that F2's forecasts do not move |
| `fuel.admit` | | | | | | 1 | |
| `fuel.s`, growth members | | | | | | 1.9 | T, P (1.5 to 2.3); needs the bound of `fuel.s` widened to (−3, 3) |
| `fuel.s`, hinge members | | | | | | 0.0 | T, J (−0.5 to +1.0) |
| `fuel.se_coord`, `fuel.ft_coord` | | | | | | (1, 0) | J |
| `fuel.tad_min_k` | | | | | | 1100 | P, J |

Scalars: `bt.kappa_exp` = 0.5 (J, 0.3 to 0.8); `mix.o2_b_w` = 0.5 (J).

## A6. The nine methane-in-helium tests without an oxygen fraction

The law can answer them, as a mixture over the two values the table names, and it should not pretend to more. The day
(20160) shows no vent and refill in its pressure, so every test of it shares one unknown atmosphere; nothing in the
record favours 21 or 30 %. Weight 0.5 each. What it costs, from the inputs alone: the heating clock hardly moves (the
limit is reached 1 to 3 s earlier at 30 %), the effective temperature moves from 1904 to 2050 K, and the reduced flow
falls by 0.79. The two forecasts differ mainly for the three tests at 2.4 and 3.4 mg/s, where the growth law gives an
extinction before the stop 0.33 to 0.73 at 21 % and 0.07 to 0.27 at 30 %; the mixture says about one half, which is the
truth about what is known. One fragility: at 21 % the effective temperature (1904 K) is 4 K above methane's refusal
bound. With a relative Lewis number of 1.40 instead of 1.37 the 21 % candidate would be refused, and with it the nine
tests. The two tests at 5.13 mg/s are outside the declared flow range and are refused either way.

## A7. Registered expectations, and what would contradict them

Computed by hand from the inputs with the numbers of A5 and the F2 laws (central node of each clock; the heating law
with a limit of 250 °C as the scripts set it). "Limit" means the script's own account, "burner temperature limit
exceeded", not the sensor label.

| Flames | Expectation | Contradicted by |
|---|---|---|
| **Ethylene in helium**, 30 % O2, 1.5 to 3.1 mg/s (20164, 7 tests answered; 4.6 mg/s refused) | The burner reaches 250 °C at 7 to 14 s and the script stops the flow. The flames do not go out by themselves first: reduced flow 0.55 to 1.1, no growth clock inside 45 s. At least 5 of 7 end on the limit, none later than 25 s | three or more go out by themselves before the limit is reached; or three or more burn past 25 s |
| **Methane in helium** (20160, 7 tests answered) | 1.0 to 2.4 mg/s: limit at 6 to 14 s. 3.4 mg/s: limit at about 19 s, with an extinction before it about one time in three. At least 4 of 7 end on the limit | the flames last seconds only, as methane in air did (below) |
| **Hydrogen in nitrogen**, 45 to 70 % (19352, 20024 A–B) | No extinction by itself before the stop (probability below 0.15). Limit at 5 to 12 s below 0.25 mg/s, at 24 to 44 s at 0.4 to 0.6 mg/s | extinctions by the script's account on more than a quarter of these tests |
| **Hydrogen in nitrogen**, 15 to 30 % (20024 D–N) | Burns; limit at 29 to 38 s. At 15 % (1188 K) one flame in four or five flashes out at ignition | the 20 to 30 % flames do not ignite or go out within seconds: the +600 K is too large |
| **Hydrogen in helium** (20155, 20190) | Limit within 3 to 8 s on every test, at either oxygen value | any test burning past 20 s |
| **Blends in nitrogen**, air (20006, 20013) | Go out by themselves at 9 to 24 s; only the lowest flows (0.44 to 0.74 mg/s) can reach the limit first (15 to 26 s). 20013C5 (1615 K) is refused | most burn to the limit or the stop |
| **Blends in helium** (20132, 20136) | Split: the hinge members say out at 6 to 23 s, the growth members hold the flames below 2.5 mg/s; limit at 6 to 33 s by the heating law. No test burns to its planned stop | several burn to the stop |

Two doubts that the expectations do not resolve, written here because I already hold them.

- **Methane in helium may simply not burn.** Its effective temperature (1900 to 2050 K) is below that of the methane
  flames in air (2100 K), which the reports say all went out within 4.4 s. If that was a thermal limit of methane,
  the helium flames go out in seconds and the limit is never reached. My probability: 0.35. The law as typed gives
  this outcome 0.03.
- **The blends in air are probably shorter-lived than the law says** (the reports give a median end of 3.4 s). The law
  is late for them exactly as F2 is late for methane in air (about 10 s against 2 to 4 s). I did not correct it here:
  whatever Part B does to the methane row reaches the blends through the mixing rule.

Uncertainty of the heating expectations: the times to the limit scale inversely with the conductivity factor, whose
exponent is known to a factor of 1.5; and Part B has not yet said what a flame does when its burner is that hot.

# Part B. From the train days

## B0. What was read, and how the questions were asked

Read after Part A was saved: `train_days_cases.jsonl` (186 cases of 78 train-day tests: inputs, endpoint, the script's
stop reason, the limit flag, F2's tables and contributions); the sensor files of those same train-day tests, by name
from that file, for one channel pair (burner thermocouple and its cold junction) and the flame onset
(`flame_design.py measure`, unchanged). Of `train_study.json` only the key names. Not read: any validation outcome
or per-case file, the test days 20142, 20170, 20176, day 19245.

**Attribution.** `tools/wsp/attrib/attrib.py` runs the project's own train partitions and cannot be pointed at the
s-Flame case files. I used its method on them instead: a child of F2 with a stated set of literal statements,
compiled and assembled in a scratch registry outside the repository, run by `target-merged/release/ignis.exe` on the
372 stored train-day prefixes, scored by `sflame_cases.score`. The unperturbed run returns the published train
numbers (0.0914 without the planned stop, 0.0891 with it; 53 cases). Seventeen children were run on the s-Flame
train days and three on CFI-G train, each once: six on the limit and the heating law (the limit alone at 250 and
260 °C; the rise lowered by 0.3 and by 0.5; the temperature factor as two tangents), seven on the methane shifts
(hinge +0.3, +0.6, +1.0 and the typed 0.35; growth −0.3, −0.5, −0.8 and the typed −0.35), one on the ethylene
bound, and two with the typed changes together (0.0753 / 0.0784 against F2's 0.0914 / 0.0891, without the
weak-flame shift, which F2's forward cannot express). Those that bear on a typed number are given with their answers. On CFI-G train the same children were run through
`wsp-dev feedback` (210 cases; base 0.052162 / 0.023997). Nothing was searched and no value was chosen by score:
every typed number below has a derivation that does not use a score.

## B1. How Part A fared on the train days

The hydrogen and blend days are train days, so Part A's expectations can be read against them at once. By the
script's own account:

| Part A expectation | Train days | Verdict |
|---|---|---|
| Hydrogen in nitrogen, 45 to 70 %: no extinction by itself; limit at low flow | 8 tests: 4 limit stops (0.12 to 0.20 mg/s), 2 scripted ends (0.41, 0.61 mg/s), 2 script-detected extinctions of the smallest flames (0.12, 0.15 mg/s at 3 to 5 s) | direction right; the limit came 1.5 to 2 times later than typed, and not at all at 0.41 mg/s where I had it at 24 s |
| Hydrogen in nitrogen, 15 to 30 %: burns; limit at 29 to 38 s | 6 tests: 5 burned to the scripted end with the burner at 127 to 188 °C, one script-detected extinction at 15 % | burns: right, the +600 K holds. Limit: **wrong**, the heating was overstated by 1.5 to 2.5 |
| Hydrogen in helium: limit within 3 to 8 s, none past 20 s | 14 tests: 8 limit stops with the burner crossing 250 °C at 7 to 12 s, 2 more (no script account) at 13 to 16 s, 4 script-detected extinctions of the smallest flames at 2 to 8 s | right in kind, 1.5 times early |
| Blends in nitrogen go out by themselves at 9 to 24 s | 44 % hydrogen bottle: 8 of 8 went out, at 5 to 27 s, **later at higher flow**. 67 % bottle: **all 8 ran to the scripted end** (three of them are labelled extinction by the sensor rule while the burner kept heating) | half right; contradicted on the hydrogen-rich bottle by my own criterion |
| Blends in helium: split; none burns to its stop | 12 of 12 went out within 1.4 to 3.4 s | the registered doubt, not the registered expectation: a methane-rich flame in helium at 21 % oxygen does not stay lit |

Two lessons carry into Part B: the conductivity rule needs the flame temperature beside it (B2), and methane needs a
hotter flame than the alkanes to stay lit (B3).

## B2. The burner limit

### What ends these flames

- **The labels are right for ethylene and methane in nitrogen.** All 21 extinction-labelled tests that F2 answers
  carry the script's own "EXTINCTION DETECTED"; the inputs report finds the photomultipliers dark a median 1.2 s
  before that stamp. The one test the 250 °C limit stopped (19254H3, at 42.8 s) is labelled a stop.
- **They are not right for hydrogen and the blends.** 4 limit stops are labelled extinction (the dim hydrogen flame
  crossed the dark level first), 6 script-detected extinctions are labelled stops (5 hydrogen, 1 blend), and 5
  scripted ends are labelled extinction (2 hydrogen, 3 blends) while the burner went on heating for another 20 to
  40 s. A hydrogen row should be judged against the script's account.
- **The flames went out by themselves, and not because the burner was hot.** When the ten flames that F2's heating law
  sends to 250 °C before their stop went out, the burner was at 40 to 48 °C (three methane flames, 1 to 3 s), at 31 °C (an ethylene flash), at 94 and
  110 °C (two ethylene flames, 16 and 20 s) and at 174 to 182 °C (three ethylene flames at 0.43 to 0.64 mg/s, 24 to
  26 s). A fourth ethylene flame at the conditions of one of those three burned on, and its burner reached 250 °C at
  41 s. A warming burner takes less heat from the flame, not more; nothing in
  the traces suggests otherwise.
- **So the question "extinction or stop" has a plain answer on the train days: extinction, a real one.** F2 lost on
  the hot-burner tests of the validation day because its extinction clocks call a low-flow methane flame held, not
  because of what it assumed about the limit. With the limit as an input and nothing else changed, F2's train loss
  rises from 0.091 to 0.103 (limit 260 °C) or 0.111 (250 °C): its heating law is too fast and would stop flames that
  in fact went out.

### The heating law against the burner traces (train days, six days, four fuel streams)

Equilibrium rise implied by the measured rise at 20 and 40 s (time constant 30 s), against F2's law:

| Flames | Measured / F2's law (for hydrogen: the same law with hydrogen's molar mass) | Flame temperature factor (T − 303) / (2650 − 303) |
|---|---|---|
| Ethylene in air, 0.43 to 0.64 mg/s (two days, 4 flames, 1984 to 2158 K) | 0.72 to 0.75 | 0.72 to 0.79 |
| Hydrogen 45 % in nitrogen (1955 K) | 1.1 to 1.4 | 0.70 |
| Hydrogen 30 % (1665 K) | 0.86 to 0.96 | 0.58 |
| Hydrogen 25 % (1532 K) | 0.74 to 0.82 | 0.52 |
| Hydrogen 20 % (1376 K) | 0.60 to 0.68 | 0.46 |
| Hydrogen 15 % (1188 K) | 0.44 to 0.50 | 0.38 |

The five hydrogen rows are one day, one flow range and five fuel fractions. Divided by the temperature factor they
leave 1.6 to 2.0, 1.5 to 1.7, 1.4 to 1.6, 1.3 to 1.5 and 1.2 to 1.3: the conductivity of the stream (linear rule:
3.7, 2.8, 2.5, 2.2, 1.9) to a power of 0.35 to 0.5, centre 0.42. With both, the law gives the burner temperature
late in the four most diluted hydrogen flames as 183, 164, 144 and 122 °C where 181, 166, 137 to 155 and 111 to
127 °C were measured, and
puts hydrogen in helium at an equilibrium rise of 630 to 1390 K where the traces imply 530 to 1440 K.

What F2's law lacked is the oldest term of all: conduction to the sphere is proportional to the temperature difference
between flame and burner. It was written from CFI-G flames between 2480 and 2720 K and holds there. s-Flame burns at
1190 to 2400 K.

Not repaired: at total flows below 2 mg/s the law is 1.5 to 2 times fast (three hydrogen flames at 0.12 to 0.15
mg/s), and at ethylene flows above 0.7 mg/s it is 1.2 to 1.5 times fast early in the flame. Neither changes an
outcome on the train days.

### Decisions

1. **The limit is an input**: condition `burner_limit_c`, the script's `burnerTemperatureLimitC`.
2. **How a member uses it.** F2's members weigh 510 °C (early era) or 455 °C (late era). Those are the temperatures at
   which CFI-G burners tripped under script settings of 500 and 450 °C: the setting plus the heating during the
   script's dwell. Each member is given the setting it was typed under, `bt.limit_script_c` (500 or 450), and shifts
   its own limits by the difference: `limit = bt.limit_c + (burner_limit_c − bt.limit_script_c)`. For s-Flame that is
   260 °C; the measured peaks after a 250 °C trip in nitrogen are 258 and 263 °C. With the setting of the date the
   shift is zero and the member is F2's.
3. **What is forecast when the law reaches the limit**: operator termination, at the time the heating law crosses the
   shifted limit, with the spread the members already carry (0.12 in ln of the rise, shape 8). No extra certainty and
   no extra doubt is typed: whether a flame is still there to be stopped is the business of the extinction clocks,
   which compete with it as before.
4. **The flame-temperature factor**, in every member: `ln dT_eq += ln((T_1 − T_b) / (bt.tflame_ref_k − T_b))`, with
   `T_b = bt.t0_c + 273.15` and `bt.tflame_ref_k` = **2650 K** (D: the middle of the CFI-G flames whose burner trips
   the law was typed from, 2490 to 2700 K). The exponent is 1 because it is a conduction law (T); the train days ask
   for 0.9 to 1.1.
5. **The conductivity exponent**: Part A's 0.5 becomes **0.45** (the train days ask about 0.42 in nitrogen and
   about 0.5 in helium; half of the step).

Answers of the finite differences (s-Flame train days, 53 cases, sum without / with the planned stop; F2: 0.0914 /
0.0891):

| Question | Answer |
|---|---|
| limit 250 °C, nothing else | 0.1106 / 0.1099 |
| limit 260 °C, nothing else | 0.1030 / 0.1024 |
| limit 250 °C and the rise × 0.74 | 0.0787 / 0.0809 |
| limit 260 °C and the temperature factor (as a tangent in the rise network: intercept 5.669, slope 0.171) | 0.0794 / 0.0823 |
| the temperature factor on CFI-G train (tangent at 2650 K: 5.688, 0.128), limits as F2 | 0.076449 against 0.076159: three tests a little worse, one better |

## B3. The methane row

What the train days show (one day in nitrogen at 21 % oxygen, 7 tests; and, for what a methane-rich flame does, four
blend days):

- Every methane flame in air went out within 1.2 to 4.4 s, at 2101 and 1966 K, at every flow from 0.5 to 2.4 mg/s.
  Both laws give 9 to 15 s. The flames are dead before the 4 s the laws allow a main-class flame.
- The blends say the same in steps. In helium at 21 % (effective temperature about 1890 K): 12 of 12 out within
  3.4 s. In air with 17 % of the oxygen demand from hydrogen (2014 K): out at 5 to 27 s. With 33 % from hydrogen
  (2002 K): all burn to the stop.
- Lives rise with flow where the flame is weak (methane in air: 1.2, 2.3, 3.4 s at 0.51, 0.63, 0.76 mg/s; the 44 %
  blend: 5 to 6 s below 0.8 mg/s, 11 s at 1.0 to 1.5, 19 and 27 s at 1.9 and 2.2). That is the low-flow, kinetic
  regime that s-Flame set out to find, and both laws run the other way there.

One row step, three entries:

| Entry | F2 | F3 | Evidence and reason |
|---|---|---|---|
| `fuel.dtad_k`, methane | (no such column: 0) | **−350 K** | Methane needs a hotter flame to stay lit than the alkanes the weak-flame law was typed on. With −350 the centre of that law sits at 2100 K for methane: the air flames (7 of 7 dead at once) get 0.45 to 1.0, the helium blends 0.35 to 0.9 through the mixing rule (12 of 12 dead), the 44 % blend in air 0.05 to 0.15 (alive for 5 to 27 s) and the 67 % blend 0.01 (burned to the stop). Five days, one number. Allowed window from the train days: −360 or beyond puts the air flames past the centre; the known aggregate of the validation day (methane at 30 % oxygen mostly alive at 5 s, down to 2160 K) forbids much more than −420. Typed at the weak end of that window. Class D |
| `fuel.s`, methane, hinge members | 0.068 | **0.35** | Low-flow methane flames are shorter-lived than the hinge law says (3 of 3 tests below 0.8 mg/s; the probe at +0.3 lowers their loss, and nothing moves beyond +0.6). Theory gives +0.2 to +0.35 from the mass-fraction geometry the hinge law lacks (first dossier, 4.2) and +0.25 to +0.4 if methane's extinction temperature is 100 to 150 K above the alkanes'. Half of the step to 0.6 |
| `fuel.s`, methane, growth members | 0.068 | **−0.35** | All 7 tests ask for a higher critical flux (probes at −0.23, −0.5, −0.8: train loss on methane falls by 0.013, 0.020, 0.024 and is still falling). An extinction temperature of 1500 to 1650 K against the alkanes' 1350 raises the critical flux by 0.3 to 0.9 in ln. Half of the step to −0.8: a critical flux of 0.255 g m⁻² s⁻¹ |

Expected effect. Train methane: 0.033 to about 0.015 from the two shifts alone (measured on a child of F2); the
weak-flame shift cannot be probed in F2's forward and should take most of the rest. For the validation day's low-flow
methane flames, from their inputs: the growth members now put extinction at 13 to 20 s where they had the flames
held, before the burner reaches its limit (19 to 33 s on four tests, by the corrected law); the hinge members still
say 24 to 87 s. One test (17 % methane, 2035 K) is now called a flash at 0.8 to 0.9.

**What this step cannot do.** It cannot make a weak flame live longer at higher flow. That is a mechanism neither law
has (heat loss to the burner against a kinetic limit, strongest for the slowest fuel), and by PROTOCOL 12.3 it is a
new law, not a number. It is the first thing to build after F3. The evidence for it is above; the smallest form would
be a quench clock in both member kinds whose median rises with the total burner flow and with the margin of the flame
temperature over the fuel's limit.

## B4. The two open ethylene items

- **Nitrogen-diluted ethylene outlives the growth law.** The s-Flame train days cannot test the remedy
  `phys_ethylene_held.md` 6.4 specifies: every ethylene flame F2 answers there has a mixture fraction of 0.23 to 0.33,
  where the proposed term is zero or 0.03. What they do show is the same error at low flow: the growth member is 1.4
  to 1.8 times early for the five flames below 0.9 mg/s that ended by themselves after ignition (12 to 17 s against
  16 to 26 s) and right above 1 mg/s. The
  remedy rests on the Flame Design train days, where its full value takes the extinction score from 0.0746 to 0.0577.
  Since F3 is a new forward, the two columns are added with the damped values of that dossier: `fuel.zst0` = 0.30,
  `fuel.gz` = 1.0 for ethylene, inert for every other fuel. On s-Flame it changes nothing that can be measured.
- **Ethylene below 1900 K stays refused.** Answering the one train flame below the bound (20029R1, 1829 K, 0.32
  mg/s) costs 0.42 per case without the planned stop and 0.67 with it, two to three times the incidence baseline: the laws
  say 21 s or held and it went out at 7.3 s. On Flame Design the six flames below the bound all burned to their
  stops. One campaign says such flames die early and the other that they last; a bound that refuses both is the only
  statement both support.

## B5. Every number F3 changes or adds, in one table

| Number | F2 | F3 | Class | Evidence | Expected effect |
|---|---|---|---|---|---|
| limit of the burner clock | 510 or 455 °C by the date | the script's setting + 10 or + 5 °C | input | 162 train scripts carry 250 °C | on s-Flame the clock can fire; elsewhere nothing |
| `bt.tflame_ref_k` | absent | 2650 K | T, D | six train days, B2 | s-Flame rises × 0.4 to 0.8; CFI-G train sum +0.0003; removes the false stops the limit alone would add |
| `bt.kappa_exp` | absent | 0.45 (Part A: 0.5) | J → D | hydrogen day 20024, helium days | only with hydrogen or helium in the stream |
| `fuel.dtad_k` methane | 0 | −350 K | D | B3 | methane near and below 2100 K flashes |
| `fuel.s` methane, hinge / growth | 0.068 / 0.068 | 0.35 / −0.35 | T, D | B3 | shorter methane lives at low flow |
| `fuel.zst0`, `fuel.gz` ethylene (growth) | absent | 0.30, 1.0 | D (Flame Design train days) | `phys_ethylene_held.md` 6.4 | ethylene above Z_st 0.30 lives longer in the growth law |
| hydrogen row, helium row, blend rule | refused | Part A, zero-shot | P, T, J | not revised from the train days, except the exponent above | |
| `mix.o2_b_w` | absent | 0.5 | J | A6 | only with two oxygen candidates |

Kept on purpose: `fuel.tad_min_k` (all rows), the ethylene shifts, every law network, the committee weights, eras and
stop offsets, `bt.sigma`, `bt.shape`, `bt.tau_s`, `bt.t0_c` (the burner starts at 24 to 27 °C on s-Flame, 30 °C is
typed; 4 K in 230).

## B6. Neutrality

With nitrogen, a fuel F2 answers, no blend, one oxygen value and no `burner_limit_c` (or the setting of the member's
era), F3 differs from F2 in exactly these places, each on purpose:

1. the flame-temperature factor of the heating law: every forecast in which the burner clock has weight;
2. the methane row: `fuel.dtad_k`, `fuel.s` in both tables;
3. ethylene above a mixture fraction of 0.30, growth members only.

Everything else is an identity in IEEE arithmetic by construction: a nitrogen row of ones and zeros that is skipped,
`ln(1 + 0)` for a stream of unit relative conductivity, `+ 0.0` for the shifts, a limit shift of `x − x`. The
engineer's acceptance test: with `bt.tflame_ref_k` = 0 (which switches the factor off), F2's two fuel tables and the
new columns at their inert values, the new forward returns F2's event tables bit for bit on the 372 s-Flame train-day
prefixes, on CFI-G train and on the Flame Design chain check.

## B7. Expectations for the validation days after Part B

From the inputs alone (central clocks, limit 260 °C). These replace nothing in A7; they are what the revised table
says.

| Flames | F3 says |
|---|---|
| Ethylene in nitrogen (19256) | as F2: out at 9 to 12 s; the burner stays below 225 °C |
| Methane in nitrogen, 30 % O2, 1.1 mg/s and above | out at 10 to 16 s (F2: 15 to 21 s) |
| Methane in nitrogen, 30 % O2, 0.4 to 0.85 mg/s | growth members: out at 11 to 20 s; hinge members: 22 to 87 s; the limit at 19 to 33 s on the four tests with the least total flow. A divided forecast, on the right side of F2's |
| Methane 17 % at 2035 K | flash, 0.8 to 0.9 |
| Methane in helium (20160) | flash: 0.99 at 21 % O2, 0.7 to 0.9 at 30 %; those that survive reach the limit at 9 to 35 s |
| Ethylene in helium (20164) | no extinction by itself; limit at 12 to 16 s below 2.2 mg/s, 25 s at 3.1 mg/s |

Contradicted if: methane in helium burns for tens of seconds; ethylene in helium goes out by itself within 10 s on
most tests; the low-flow methane flames in nitrogen went out later than about 30 s (then the growth shift is too
large) or the burner limit stopped them (then they were stronger than both laws now say).

# For the engineer

Forwards are frozen with their releases, so these are new files beside `fuel.rs`, `flame6.rs`, `flame7.rs` and
`committee3.rs`, and a new spec (suggested `acme_normal.wsp13`). Names of files are yours; names of tensors and
conditions are below.

## E1. New inputs (conditions of the prefix)

| Key | Unit | From | Absent means |
|---|---|---|---|
| `diluent_species` | text | `diluent` and `ambient_diluent` of the case file: `nitrogen` or `helium` | nitrogen |
| `fuel_species` = `hydrogen+methane` | text | as in the case file | |
| `fuel_blend_x` | 1 | mole fraction of the first-named species within the fuel: h2 / (h2 + methane) of `flows.burner_stream` | refusal with a blend |
| `burner_limit_c` | °C | `burner_limit.burner_limit_c` (`acme_script2.py`) | the member's own limits, as F2 |
| `x_o2_role_b`, `tad_b`, `zst_b` | 1, K, 1 | the second entry of `inputs_by_o2_candidate`; `x_o2_role`, `tad`, `zst` then hold the first (lower oxygen) | one oxygen value |

For a blend `injected_mass_flow` is the sum over the fuel species and `x_fuel_role` the sum of their mole fractions,
as the case files already give them. Refusals at the condition gate: a diluent that is not a row, or that differs
between the burner stream and the ambient, or a burner stream with both gases; a blend with a species that is not a
row or without `fuel_blend_x`; only one or two of the three `_b` keys.

## E2. Tensors

Fuel table (`fuel.rs` → its successor), six rows as now:

| Tensor | Shape | Bounds | Unit | Meaning |
|---|---|---|---|---|
| `fuel.s` | [6] | **(−3, 3)** (was (−1, 1)) | 1 | unchanged meaning |
| `fuel.sigma` | [6] | (1, 10) | g/g | oxygen demand per gram; read only to form blend shares |
| `fuel.dtad_k` | [6] | (−1000, 1000) | K | added to the flame temperature the weak-flame law reads |
| `fuel.k_rel` | [6] | (0.1, 20) | 1 | thermal conductivity relative to nitrogen |
| `fuel.zst0` | [6] | (0, 1) | 1 | mixture fraction above which `fuel.gz` acts (growth law) |
| `fuel.gz` | [6] | (−10, 10) | 1 | fall of ln of the critical flux per unit of mixture fraction above `fuel.zst0` |

Diluent table, in every member like the fuel table; rows 0 nitrogen, 1 helium:

| Tensor | Shape | Bounds | Unit | Meaning |
|---|---|---|---|---|
| `dil.admit` | [2] | 0 or 1 | 1 | the member answers for the diluent |
| `dil.mw_g_mol` | [2] | (1, 200) | g/mol | replaces `phys.mw_n2` in Y_R |
| `dil.rho_d` | [2] | (0.1, 10) | 1 | factor on `se.rho_d_g_m_s` wherever r_ss is formed |
| `dil.d_far` | [2] | (0.1, 10) | 1 | factor on the growth diffusivity (growth law) |
| `dil.le_o2` | [2] | (0.2, 5) | 1 | Lewis number of oxygen relative to nitrogen's |
| `dil.dflux` | [2] | (−3, 3) | 1 | added to ln m_cr (growth law) |
| `dil.dlogm` | [2] | (−5, 5) | 1 | added to c_logm in `se.tau` (hinge law) |
| `dil.dlnr` | [2] | (−5, 5) | 1 | added to ln tau_R (hinge law) |
| `dil.k_rel` | [2] | (0.1, 20) | 1 | thermal conductivity relative to nitrogen |

Member scalars (both member kinds): `bt.kappa_exp` (0, 2); `bt.tflame_ref_k` (0, 4000) K, 0 switches the factor off;
`bt.limit_script_c` (0, 1500) °C. Committee scalar: `mix.o2_b_w` (0, 1). Constant: `phys.t_stream_k` = 298.15.

## E3. Computation, in the terms of the existing files

```text
fuel.rs      row_of: "a+b" names two rows; with fuel_blend_x = x (mole fraction of a within the fuel)
               y_a = x W_a / (x W_a + (1 - x) W_b), y_b = 1 - y_a           W = fuel.mw_g_mol
               w_a = y_a sigma_a / (y_a sigma_a + y_b sigma_b), w_b = 1 - w_a
               mw_g_mol = x W_a + (1 - x) W_b;   k_rel is kept per species
               every other column c: c = w_a c_a + w_b c_b   (s, se_coord, ft_coord, dosc, tad_min_k, dtad_k, zst0, gz)
               admit: both rows admitted
             a single fuel: its row, read as now (no arithmetic)
             d = row of diluent_species (0 without the key);  dil.admit[d] = 0: refusal

features     Y_R: dil.mw_g_mol[d] in place of phys.mw_n2                      (flame6.rs line 723, flame7.rs likewise)
             T_1 = tad                                                        if dil.le_o2[d] = 1
                 = T_0 + (tad - T_0) (1 - (1 - zst)^(1 / dil.le_o2[d])) / zst   otherwise, T_0 = phys.t_stream_k
             refuse if T_1 < fuel.tad_min_k                                   (fuel::check, with T_1 for tad)
             c_tad      = (T_1 + fuel.dtad_k - norm.tad_ref_k) / norm.tad_scale_k    entry 4 of c: se.flash, the domain bound
             c_tad_burn = (T_1 - norm.tad_ref_k) / norm.tad_scale_k                  entry 2 of b: bt.rise
             r_ss: rho_d = se.rho_d_g_m_s x dil.rho_d[d]                      (a factor of exactly 1 is not applied)

flame6.rs    ln m_cr = N*_se(se.flux ; c) - fuel.s + dil.dflux[d] - fuel.gz x max(0, zst - fuel.zst0)
(growth)     ln D    = N*_se(se.diff ; c) + ln dil.d_far[d]                   (d_far = 1: nothing added)

flame7.rs    ln tau_R = N*_se(se.tau ; c with c_logm + fuel.s + dil.dlogm[d]) + dil.dlnr[d]
(hinge)

burner       kappa   = sum over the species i of the burner stream of x_i (k_rel,i - 1)
(both)                 fuel species: x_fuel_role split by fuel_blend_x, with fuel.k_rel; diluent: 1 - x_fuel_role, dil.k_rel[d]
             ln dT_eq = N*_ft(bt.rise ; b) + bt.kappa_exp x ln_1p(kappa)
                        + ln((T_1 - T_b) / (bt.tflame_ref_k - T_b))           T_b = bt.t0_c + 273.15; omitted when bt.tflame_ref_k = 0
                        (T_1 <= T_b: refusal, the clock is not finite)
             limits:  with burner_limit_c, every weighted limit is bt.limit_c[l] + (burner_limit_c - bt.limit_script_c)
                      (the existing refusal of a weighted limit at or below bt.t0_c then applies to the shifted value)

committee3.rs with x_o2_role_b, tad_b, zst_b: every member is run twice (the second time with the three conditions
             replaced) inside each offset node, and
             P = (1 - mix.o2_b_w) P_first + mix.o2_b_w P_second;  a refusal of either refuses the forecast
```

The order matters in one place: the temperature T_1 is formed once and used three ways (refusal bound, weak-flame law
with the fuel's shift, heating law without it).

## E4. Typed lines

Fuel table, **hinge** members (successor of `normal-f2-hinge.wsp`):

```text
# rows: 0 ethane, 1 propane, 2 butane, 3 ethylene, 4 methane, 5 hydrogen
# Hydrogen is answered: zero-shot row (Part A of docs/wsp/research/phys_sflame_f3.md).
fuel.admit[0:6] = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
fuel.mw_g_mol[0:6] = [30.07, 44.1, 58.12, 28.054, 16.043, 2.016]
# Oxygen demand per gram (P). Read only for the shares of a blend.
fuel.sigma[0:6] = [3.725, 3.628, 3.579, 3.422, 3.989, 7.936]
# Methane, 0.068 -> 0.35 (D, s-Flame train day 19304): low-flow methane flames are shorter-lived than this law
# says; +0.2 to +0.35 is the mass-fraction geometry the law lacks, the rest a higher extinction temperature.
# Half of the step to 0.6. Hydrogen 0.0 (T, J): oxygen demand and a critical flux a seventh of ethane's give
# -0.4, the geometry of a 45 % mixture in nitrogen +0.45.
fuel.s[0:6] = [0.0, 0.0, 0.0, -0.31, 0.35, 0.0]
fuel.se_coord[0:6, 0:2] = [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 0.0], [1.0, 0.0]]
fuel.ft_coord[0:6, 0:2] = [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.9, -0.9], [1.0, 0.0]]
fuel.dosc[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
# Hydrogen burns down to about 1000 K (P, J).
fuel.tad_min_k[0:6] = [0.0, 0.0, 0.0, 1900.0, 1900.0, 1100.0]
# Shift of the temperature the weak-flame law reads. Methane -350 K (D): in air at 2101 and 1966 K seven of
# seven flames went out at once, and twelve of twelve methane-rich blends in helium; the centre of the law is
# then at 2100 K for methane. Hydrogen +600 K (P, J): its flames at 1188 to 1665 K burned to their stops.
fuel.dtad_k[0:6] = [0.0, 0.0, 0.0, 0.0, -350.0, 600.0]
# Thermal conductivity relative to nitrogen (P). The hydrocarbons are 0.8 to 1.3 and are left at 1.
fuel.k_rel[0:6] = [1.0, 1.0, 1.0, 1.0, 1.0, 7.0]
# Not read by the hinge law.
fuel.zst0[0:6] = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
fuel.gz[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
```

Fuel table, **growth** members (successor of `normal-f2-growth.wsp`): the same lines except

```text
# Methane, 0.068 -> -0.35 (D): all seven train flames ask for a higher critical flux; an extinction temperature
# of 1500 to 1650 K against the alkanes' 1350 gives -0.3 to -0.9. Half of the step to -0.8: 0.255 g m-2 s-1.
# Hydrogen 1.9 (T, P): water alone radiates a third of what CO2 and water do per unit heat, the flame may lose
# a larger fraction before it goes out, and a gram of hydrogen is 2.5 grams of ethane in heat.
fuel.s[0:6] = [0.0, 0.0, 0.0, -0.02, -0.35, 1.9]
# Ethylene above a mixture fraction of 0.30 loses less heat (D, Flame Design train days,
# docs/wsp/research/phys_ethylene_held.md 6.4): half of the 2.0 those days ask for.
fuel.zst0[0:6] = [1.0, 1.0, 1.0, 0.3, 1.0, 1.0]
fuel.gz[0:6] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
```

Diluent table, **every** member (rows: 0 nitrogen, 1 helium); the hinge members read `dlogm`, `dlnr`, the growth
members `d_far`, `dflux`:

```text
# Zero-shot (Part A). No helium-diluted ethylene or methane flame has been read.
dil.admit[0:2] = [1.0, 1.0]
dil.mw_g_mol[0:2] = [28.013, 4.0026]
# rho D of oxygen: 1.2 to 1.5 times nitrogen's in the ambient, 0.8 at the flame (T).
dil.rho_d[0:2] = [1.0, 1.15]
# Binary diffusion of oxygen in the cold ambient: 0.73 against 0.21 cm2/s (P).
dil.d_far[0:2] = [1.0, 3.5]
# Lewis number of oxygen, 1.45 in helium against 1.06 in nitrogen (T, P): the flame burns below its adiabatic
# temperature by (1 - (1 - Zst)^(1/Le)) / Zst.
dil.le_o2[0:2] = [1.0, 1.37]
# Critical flux in helium: two derivations, -0.15 to +0.1 and +0.4 to +0.8 (J).
dil.dflux[0:2] = [0.0, 0.3]
# What the hinge law is told, since it reads no mass fraction: the reduced flow of a helium flame is a seventh
# of a nitrogen flame's at one fuel flow (T), and its clock runs 3.5 times faster (P).
dil.dlogm[0:2] = [0.0, -1.9]
dil.dlnr[0:2] = [0.0, -1.25]
# Thermal conductivity relative to nitrogen (P).
dil.k_rel[0:2] = [1.0, 5.8]
```

Member lines, in the slot blocks of the committee header (`b0` to `b3`, `c0` to `c3`):

```text
# Heat reaches the burner by conduction through the burner stream: the equilibrium rise goes as the stream's
# conductivity to this power (J, then D: hydrogen at 15 to 45 % in nitrogen on one day asks about 0.42,
# hydrogen in helium about 0.5).
<slot>.bt.kappa_exp = 0.45
# ... and in proportion to the temperature difference between flame and burner (T). The law was typed from
# flames at 2490 to 2700 K; this is the temperature at which it is left as it was (D).
<slot>.bt.tflame_ref_k = 2650.0
# The script setting under which this member's limits were typed: its limits moved by (burner_limit_c - this).
b0.bt.limit_script_c = 500.0
b2.bt.limit_script_c = 500.0
c0.bt.limit_script_c = 500.0
c2.bt.limit_script_c = 500.0
b1.bt.limit_script_c = 450.0
b3.bt.limit_script_c = 450.0
c1.bt.limit_script_c = 450.0
c3.bt.limit_script_c = 450.0
```

Committee header, one new line:

```text
# Two candidate oxygen fractions and nothing to choose between them (s-Flame day 20160): equal weight.
mix.o2_b_w = 0.5
```

The slots a0 and a1 (weight zero) take the same tables with the hinge values; they have no heating law and need no
member line.

# Exposure, and the weakest assumptions

**Exposure of F3.** Typed from: the s-Flame train days (11 days; all fuels); the aggregates of the s-Flame validation
days in PROTOCOL 13.1 and `sflame_transfer_result.md`, which the author knew and used in two places, both marked (the
upper edge of the window for methane's weak-flame shift, and the reading that F2's validation loss sits in its
extinction clocks); the CFI-G train partition; the Flame Design train days through `phys_ethylene_held.md`. Not read:
any validation per-case file or outcome, the test days. The helium and hydrogen rows were typed before any of their
outcomes was opened, with the aggregates of A0 known; of those rows only the conductivity exponent was touched
afterwards.

**Weakest assumptions.**

1. **That methane's early death is a flame-temperature threshold (−350 K).** One nitrogen day and the blend days fit
   it, but the train days also show lives rising with flow, which a temperature threshold cannot produce, and the
   window left for the number by the validation aggregate is 60 K wide. If the cause is the low-flow limit, F3 will
   call the weakest validation flames flashes that in fact burned for some seconds, and will still hold low-flow
   flames too long in its hinge half.
2. **The helium row.** No helium-diluted ethylene or methane flame has been seen. The critical flux in helium is
   known to a factor of two, the hinge members are told two numbers derived from the growth law, and the weak-flame
   reading of a helium flame rests on a Lewis-number correction of 570 K whose own uncertainty (±60 K) is as wide as
   the weak-flame law.
3. **The heating law outside its origin.** The temperature factor and the conductivity power are supported by six
   train days, but the law is still 1.5 to 2 times fast below 2 mg/s of total flow, its time constant was never
   checked at a limit of 250 °C in the first 15 s, and on the validation day the forecast for four low-flow methane
   tests turns on whether the flame or the limit comes first.

Also open: the blend rule fails for the hydrogen-rich bottle (the flames burn to their stops where the rule has them
out at 15 s); hydrogen's endpoint labels disagree with the script on a third of its tests, so its row cannot be
judged with the sensor labels as they are.
