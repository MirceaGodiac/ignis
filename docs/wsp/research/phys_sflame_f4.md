# The burner-flame committee after s-Flame: F4, from a case-by-case study of the development days

> Written by an author model (claude-fable-5-2) on 15 November 2026 under PROTOCOL 13.6. Every number typed here is
> class **D** (typed after reading development data) with a physical reason of class **T** or **P** beside it. Nothing
> is fitted, nothing was searched; each candidate was run once, with its expectation written into the log before the
> run. It is a proposal: no number here is a weight until it is typed into a table and assembled.
> All scores are **in-sample development numbers**. The test days 20142, 20170, 20176 and day 19245 were not opened
> and are the only judge.

## 0. In short

1. **Methane (the 7 low-flow tests).** My F3 reading was wrong. No mechanism is missing for them: at 30 % oxygen
   methane lives *fall* with flow (14.6 s at 0.51 mg/s to 6.7 s at 2.38 mg/s) and the burner never comes near its
   limit before the flame is out (80 to 223 °C). The fault was that the hinge law gave methane the long low-flow lives
   of hot CFI-G flames near holding (22 to 87 s), so the limit stop won the race on paper. Two typed numbers
   (`fuel.s` of methane in the hinge and growth members) take validation methane from 0.245 to 0.092 (0.190 to 0.040
   with one wrong label corrected), with every nitrogen ethylene, CFI-G and Flame Design forecast unmoved.
2. **Ethylene in helium.** All 8 flames ended at 10.3 to 13.0 s, almost independent of flow. Three of the five
   helium numbers of F3 stage 2 are contradicted (critical-flux shift, both hinge-law equivalents); the far-field
   diffusion ratio, `ρD` and the Lewis number are not. With the three retyped from the energy balance and a thermal
   equivalence, ethylene in helium goes from 1.512 to 0.180 and methane in helium from 0.230 to 0.006.
3. **Hydrogen and blends.** 15 endpoint labels disagree with the script's account; for 13 the script is right, for 2
   the sensor is right (the script's own extinction detector cut the flow of a flame that was still burning). With a
   typed hydrogen row hydrogen scores 0.315 without and 0.210 with the planned stop (incidence baselines 0.911 and
   0.647). Blends: the oxygen-share mixing rule is good on three of four days and badly wrong on the hydrogen-rich
   bottle (day 20013, 1.53 against 0.57 to 0.77). No blend law is typed; blends should stay refused.
4. Counts of logged candidates: methane law 2, helium law 1, hydrogen law 2 (limit five each).

## 1. What was read, what was not

Read, case by case: every constant-flow target test of the s-Flame train days (91 tests) and validation days (34
tests): `cases_train.jsonl`, `cases_validation_inputs.jsonl`, `cases_validation_outcomes.jsonl`,
`endpoints_train.jsonl`, `inputs.jsonl`, and the raw records of those tests under `runs/dev/data_all/raw/PSI-23/`
(burner thermocouple, radiometer, three photomultipliers, mass flows, the script's test-point stamps). 122 of the 125
tests have a usable trace; three hydrogen-in-helium tests of day 20190 (T1, U1, V1) have no ignition reference and
are not in any table.

Not opened: the test days 20142, 20170, 20176; day 19245; any validation or test outcome of CFI-G or Flame Design;
`holdout/`; `runs/dev/holdout_ext/`. The neutrality check reads CFI-G train and Flame Design train days through the
tool only.

Exposure. The s-Flame validation days have already judged three releases and are study data from 13.6 on; every
number below that is called "validation" is therefore in-sample for what I typed. Hydrogen and blends have train days
only, and the hydrogen row was typed from those same days. The helium row was typed after I had read what the helium
flames did; I typed the theory values (energy balance, thermal equivalence), not the values the data would have
chosen, and say below where the two differ.

Scoring. Integrated Brier 6 to 120 s, cutoffs 1, 3 and 5 s, a "case" is a test at a cutoff at which it is still at
risk. The logged tool `tools/wsp/ext/sflame_whatif.py` scores with the sensor labels. Where a table says
"adjudicated", the same forward and the same `sflame_cases.score` were run with the 16 endpoints of section 2.3
replaced; that scoring was done with a scratch harness of mine and is reproducible from the table in 2.3.

## 2. What the data showed

### 2.1 Ethylene in helium (day 20164, validation)

One mixture: 20 % ethylene in helium burning in 30 % oxygen in helium, `T_ad` 2657 K, `Z_st` 0.262, sheet
temperature after the Lewis-number correction `T_1` 2089 K. Limit 250 °C, planned stop 45 s.

| test | fuel mg/s | total mg/s | end s | what ended it | burner at 10 s, °C | burner at the end, °C | F3 s2 loss | F4 loss |
|---|---|---|---|---|---|---|---|---|
| A2 | 1.54 | 2.42 | 12.3 (flow off) | limit stop: burner crossed 250 °C at 10.3 s | 246 | 291 | 0.009 | 0.353 |
| P1 | 1.70 | 2.66 | 12.9 | flame out; the script wrote "burner limit" and cut the flow at 14.0 s, the photomultipliers were dark at 12.9 s | 236 | 278 | 1.845 | 0.442 |
| N1 | 1.85 | 2.91 | 13.0 | extinction (script stamp 15.0 s) | 225 | 266 | 1.832 | 0.302 |
| M1 | 2.16 | 3.39 | 12.3 | extinction | 208 | 238 | 1.805 | 0.125 |
| B1, B2, B3 | 3.08 | 4.84 | 11.2, 11.3, 11.5 | extinction | 175 to 179 | 188 to 194 | 1.698 | 0.012 |
| C1 | 4.62 | 7.26 | 10.3 | extinction | 148 | 151 | refused (flow above the domain) | refused |

Losses are per case, without the planned stop.

**What F3 stage 2 assumed.** The growth members held every one of these flames (reduced flow Φ 0.55 to 1.10,
that is at or below holding), the hinge members gave lives of 170 to 1770 s, and the burner clock gave the limit at
12 to 25 s. So the forecast was "runs to the limit stop", and seven flames went out at 10 to 13 s first.

**Which helium numbers the data contradict.**

| number (helium row) | F3 s2 | what the 8 flames say | verdict |
|---|---|---|---|
| `dil.dflux`, shift of ln critical flux | +0.3 | with `d_far` 3.5 the lives need a critical flux of 0.50 to 0.61 g m⁻² s⁻¹, a shift of +1.0 to +1.2 | **contradicted** |
| `dil.dlogm`, hinge-law flow equivalent | −1.9 | lives are flat in flow over a factor of three, so the flames sit on the flat branch of the hinge law: shift of about 0 or above | **contradicted** |
| `dil.dlnr`, hinge-law life equivalent | −1.25 | flat-branch life of 6.3 to 9 s after the 4 s floor: about −0.5 | **contradicted in size** |
| `dil.d_far`, far-field diffusion | ×3.5 | not separable from the critical flux: life goes as `r_ext²/D`. With `d_far` 1 the same lives would need a flux of 0.85 to 0.92 | not contradicted, not confirmed |
| `dil.rho_d` | 1.15 | enters with the flux in the same product | not separable |
| `dil.le_o2`, Lewis number of oxygen | 1.37 | methane in helium (day 20160, 7 answered tests) all went out at 3.0 to 4.8 s, which is what the weak class says at the corrected temperature | consistent |
| `dil.k_rel` and the heating law | 5.8 | burner at 10 s goes as total flow to the power −0.53, the law has −0.8: right at 2.4 mg/s, 15 to 25 K low at 4.8 mg/s | small, left alone |

The Part A error is plain in the first row. I typed +0.3 as a judgement. The energy balance I had already written in
`phys_acme_normal.md`, `m_cr = ρD · Y_F · L / (Θ ℓ*)` with `Θ = (T − T_cr)/(T_cr − T_0)`, evaluated at the helium
flame's own sheet temperature with `ℓ*` unchanged, gives `Y_F L / Θ` = 0.168 against 0.053 for the nitrogen flames
the flux was typed on: a factor of 3.17, a shift of **+1.15**. I should have typed that.

### 2.2 Methane

**30 % oxygen (day 19308, validation), by fuel fraction, in order of flow.**

| fuel fraction | `T_ad` K | fuel mg/s → life s (burner at the end, °C) |
|---|---|---|
| 0.450 | 2396 | 0.51 → 14.6 (223); 0.76 → 12.6 (165); 1.27 → 8.2 (89); 2.38 → 6.7 (46) |
| 0.298 | 2277 | 0.84 → 8.2 (98) [label corrected, 2.3] |
| 0.275 | 2248 | 0.51 → 10.3 (145); 0.84 → 8.5 (90); 1.69 → 7.3 (46) |
| 0.223 | 2161 | 0.42 → 5.0 (92); 0.70 → 7.6 (80); 1.41 → 6.1 and 7.0 (42, 44) |
| 0.174 | 2035 | 1.13 → 5.3 (38) |
| 0.127 | 1846 | 0.84 → 3.0 (30), refused by the temperature bound |

Every one of the 14 flames went out by itself; none was stopped. At fixed oxygen and fuel fraction **life falls with
flow**, roughly as flow to the power −0.5 at 45 % fuel and −0.3 at 27.5 %; the one exception is the smallest flame of
all (0.42 mg/s at 22 % fuel, 5.0 s).

**21 % oxygen (day 19304, train).** All 9 flames died at 0.8 to 4.4 s: 0.25 mg/s 0.8 s; 0.51 1.2 s; 0.63 2.3 s;
0.76 3.4 s; 1.27 1.0 s; 2.38 2.7, 3.5 and 4.4 s; 27.5 % fuel at 2.38 mg/s 2.2 s. Burner 28 to 48 °C.

**In helium (day 20160, validation; 45 % methane).** 1.03 mg/s 3.0 s; 1.37 3.4 s; 1.71 3.5 and 4.4 s; 2.39 4.5 s;
3.42 4.3 and 4.5 s; 5.13 4.8 and 4.6 s.

**The seven low-flow tests.** They are the tests of day 19308 at 0.42 to 0.84 mg/s (A3, A2, K3, K2, H5, H3, H4). F3
gave them hinge lives of 22 to 87 s and the burner limit at 19 to 33 s, so its forecast was a limit stop. They went
out at 5 to 15 s with the burner at 80 to 223 °C. Losses per case under F3: A3 1.000, H5 0.618, K3 0.411, A2 0.095,
H3 0.065, H4 0.042, K2 0.040.

**Is heat loss to the burner what ends them?** No. The fraction of the heat release that the burner takes up,
measured from the burner's own heating rate between 1 and 3 s (0.31 J/K times dT/dt over `m_F · LHV`), is 0.22 for
A3 (the longest life, 14.6 s), 0.18 for K3 (10.3 s), 0.20 for H5 (5.0 s), 0.13 for A2 (12.6 s). The flame that loses
most lives longest.

**What the growth law needs.** Read backwards through the growth law, the lives give a methane critical flux of 0.25
to 0.28 g m⁻² s⁻¹ at 45 % fuel and low flow, 0.33 at 1.27 mg/s, 0.49 at 2.38 mg/s; 0.31 to 0.40 at 27.5 %; 0.35 to
0.45 at 22 %; about 0.6 at 17 %. F3 has 0.255 for all of them. The energy balance with an extinction temperature of
1500 to 1550 K gives 0.31 to 0.42 over the same mixtures: the trend with fuel fraction is the trend of `Θ`.

**"Lives rising with flow" is real, but elsewhere.** It appears only in flames below the limit of the configuration:
methane in air (0.8 to 4.4 s above), methane in helium (3.0 to 4.8 s), and the 44 % hydrogen blend in air (day
20006: 0.44 mg/s 6.1 s; 0.74 4.7 and 6.3 s; 1.03 11.2 s; 1.47 10.9 and 9.8 s; 1.92 18.6 s; 2.21 26.5 s). In these
the burner takes only 0.4 to 3.5 % of the heat release, so burner heat loss is not the cause there either.

### 2.3 Hydrogen and blends: which endpoint to trust

The sensor label (end of the photomultiplier signal) and the script's own account of why it ended the test disagree
on 15 hydrogen and blend tests. I looked at each with three independent witnesses: the script's stamps and flow
commands, the radiometer, and whether the burner was still heating.

| test | fuel, diluent | sensor label | script's account | radiometer end s | burner heats until s | adjudicated | why |
|---|---|---|---|---|---|---|---|
| 20155B1 | H2, He | extinction 5.3 | burner limit, flow off 11.2 | – | 10.0 | stop 9.7 | script |
| 20190G1 | H2, He | extinction 2.7 | burner limit, flow off 11.0 | – | 9.5 | stop 9.9 | script |
| 19352D5 | H2, N2 | extinction 1.4 | burner limit | – | 14.5 | stop 17.4 | script |
| 19352D6 | H2, N2 | extinction 1.1 | burner limit | – | 20.5 | stop 20.9 | script |
| 20024M1 | H2, N2 | extinction 17.0 | scripted end | 51.9 | 48.5 | stop 46.8 | script |
| 20024N3 | H2, N2 | extinction 9.0 | scripted end | 53.2 | 48.5 | stop 47.1 | script |
| 20013C3 | blend, N2 | extinction 38.9 | scripted end, flow off 50.2 | 50.9 | 51.0 | stop 47.9 | script |
| 20013C4 | blend, N2 | extinction 28.5 | scripted end, flow off 50.3 | 50.7 | 50.5 | stop 47.8 | script |
| 20013C5 | blend, N2 | extinction 21.2 | scripted end, flow off 51.6 | 51.2 | 50.5 | stop 48.0 | script |
| 20155A2 | H2, He | stop 2.4 | extinction detected 2.0, flow off 3.4 | 4.0 | 2.5 | extinction 2.4 | script |
| 20190A1 | H2, He | stop 2.6 | extinction detected 1.7, flow off 3.5 | 4.2 | 2.5 | extinction 2.6 | script |
| 19352D4 | H2, N2 | stop 4.9 | extinction detected 4.4, flow off 7.0 | 5.8 | 5.0 | extinction 4.9 | script |
| 20006C1 | blend, N2 | stop 26.8 | extinction detected 26.5, flow off 27.9 | 29.6 | 29.5 | extinction 26.5 | script |
| 20155H1 | H2, He | stop 7.9 | extinction detected 7.3, flow off 8.5 | 9.2 | 8.0 | **stop 7.3** | sensor |
| 20024N2 | H2, N2 | stop 15.4 | extinction detected 15.0 | 18.0 | 16.5 | **stop 15.0** | sensor |
| 19308H4 | CH4, N2 | stop 1.7 | extinction detected 9.7, flow off 11.0 | 8.7 | – | **extinction 8.2** | script and radiometer |

Reasons, by group.

- **Nine "extinctions" that were stops (first nine rows).** The photomultipliers lose a dim hydrogen-rich flame long
  before it is out: the burner keeps heating and, where it sees the flame at all, the radiometer stays on until the
  flow stops. The script says it stopped the test at the burner limit or at its scripted end. Trust the script.
- **Four "stops" that were extinctions.** The script's detector reported extinction, the burner stopped heating one
  to two seconds before the flow went off (0.3 s for 20006C1, where the photomultipliers went dark 0.3 s after the
  stamp). The sensor label is "stop" only because the flow command followed quickly. Trust the script.
- **Two detector stops.** In 20155H1 and 20024N2 the script's detector reported extinction, but the burner was still
  heating and the radiometer still on when the flow was cut. The flame was alive and the operator's script ended it.
  The sensor label (operator termination) is right, by accident.
- **19308H4** is the sixteenth, a methane test. The photomultiplier "end" at 1.7 s is the change from ignition flow
  to test flow; radiometer and script put the extinction at about 8.2 s. This one label costs 0.04 to 0.05 of the
  validation methane score of any model.

**Rule proposed for the catalogue.** For hydrogen and hydrogen blends take the endpoint from the script's account
and time it with the radiometer and the burner thermocouple; the photomultipliers are not a flame sensor for these
flames. Keep the two detector stops as operator terminations.

**Hydrogen outcomes with the adjudicated labels.**

| day | diluent | mixtures | what happened |
|---|---|---|---|
| 19352 (5 tests) | N2 | 45, 60, 70 % H2; 0.12 to 0.15 mg/s; total 1.05 to 2.18 mg/s | two went out at 3.1 and 4.9 s; three ran to the limit stop at 16.4 to 20.9 s. The two that died have twins at the same condition that did not |
| 20024 (9 tests) | N2 | 15 to 45 % H2; 0.13 to 0.61 mg/s; `T_ad` 1188 to 1955 K | seven ran to the scripted end at 46.8 to 47.4 s (burner 119 to 196 °C); one limit stop at 26.4 s (0.20 mg/s, 45 %); one detector stop at 15.0 s (15 % H2) |
| 20155, 20190 (16 tests with a trace) | He | 45 % H2 | 0.29 mg/s: two never ignited (0.5, 0.6 s), three went out at 1.7 to 2.6 s, one ran to the limit (9.9 s). 0.57 mg/s: four limit stops (9.7 to 16.9 s), one detector stop (7.3 s). 0.80 mg/s: five limit stops (11.4 to 18.9 s) |

No hydrogen flame above 0.15 mg/s in nitrogen or 0.29 mg/s in helium went out by itself, down to an adiabatic
temperature of 1188 K. The smallest flames are bistable: at one condition some die within 5 s and some burn to the
limit. The ones that die lose 27 to 35 % of their heat release to the burner in the first seconds.

**Blends.**

| day | diluent | bottle (H2 share of the fuel by mole) | what happened |
|---|---|---|---|
| 20006 (8 tests) | N2 | 44 % | all went out, at 4.7 to 26.5 s, later with more flow |
| 20013 (7 tests) | N2 | 67 %, diluted to `T_ad` 1615 to 2002 K | all ran to the scripted end (47.8 to 61.7 s), burner 89 to 149 °C |
| 20132 (8 tests) | He | 56 % | all flashed: out at 1.9 to 3.4 s |
| 20136 (8 tests) | He | 44 % | all flashed: out at 0.5 to 3.2 s; none is at risk at 1 s inside the flow domain |

## 3. Mechanisms

**M-a. The critical flux follows the thermal margin of the flame, not the diluent's name.** The growth law's
extinction is the flame outgrowing the radius at which the fuel flux falls below a critical flux. That flux is an
energy balance: what the sheet can lose and stay above its extinction temperature. A helium ethylene flame at 2089 K
has three times the margin-weighted supply `Y_F L / Θ` of the nitrogen ethylene flames on which the law was typed, so
it goes out at three times the flux, that is at a third of the area, and with a far-field diffusivity 3.5 times
larger it gets there in 10 to 13 s. The same balance with an extinction temperature of 1500 to 1550 K for methane,
higher than that of ethylene and the alkanes, gives methane's higher flux and its rise as the fuel is diluted.

**M-b. Methane has no long low-flow branch.** The two hinges of the hinge law (lives lengthening steeply as the
reduced flow falls) were typed on CFI-G flames at 2500 to 2700 K that approach holding at low flow. A methane flame
at 30 % oxygen is 300 to 500 K colder and nowhere near holding at 0.5 mg/s: its life rises from 7 s to 15 s over a
factor of five in flow. A shift of the fuel's flow coordinate moves methane off the hinges.

**M-c. In helium the hinge law sees a colder flame, not a smaller one.** The helium ethylene lives are flat in flow,
so the flames are on the flat branch. What helium changes for that branch is the sheet temperature (Lewis number):
2089 K is the temperature of a nitrogen flame of the same fuel stream at about 9 % less oxygen. In the hinge law's
own coefficients (0.65 and 0.84 per 10 % oxygen) that is a flow shift of +0.6 and a life shift of −0.75. F3 stage 2
had tried to carry helium's small flame size in these two numbers (−1.9, −1.25), which pushed the flames deep onto
the long branch.

**M-d. Flames below the limit live on their ignition.** Methane in air, methane in helium and the 44 % blend are in
the weak class; they never reach a steady flame and last longer with more flow. A candidate reason (T, not tested):
while fuel is still accumulating the flame radius grows as `(m t)^(1/3)`, the radiating area as `(m t)^(2/3)`, so
the radiated fraction of the heat release goes as `m^(−1/3) t^(2/3)` and reaches a fixed critical value at a time
proportional to `m^(1/2)`. The observed exponents are 0.3 (methane in helium), 0.7 (methane in air) and 0.9 (the
blend), so this is the right direction and not yet a law. The forward's weak class has no flow in its clock. On the
score it costs little, because the measure starts at 6 s and most of these flames are out by then; it costs on the
blend at its highest flows (0.06 to 0.14 per case).

**M-e. Hydrogen holds where a hydrocarbon would not, and the smallest hydrogen flames can be quenched by the
burner.** Hydrogen flames at 1188 K burned for 47 s; nothing in the range flown above 0.2 to 0.3 mg/s went out. The
only spontaneous hydrogen extinctions are within 5 s at heat releases of 14 to 35 W, where the cold burner takes a
quarter to a third of the heat; half of those flames survive the first seconds and then run to the limit. This is a
class of outcome (early quench with a probability), not a clock.

**M-f. A hydrogen/methane blend is not a linear mixture of its fuels.** Between a 44 % and a 67 % hydrogen share the
flame changes from "goes out in 5 to 26 s at 2014 K" to "burns to the stop at 1615 K". By oxygen demand the two
bottles are 17 % and 33 % hydrogen, by mole 44 % and 67 %; neither weighting, used linearly, gives both days.

**M-g. The burner heats more slowly than the law says in small hydrogen-rich flames.** On day 19352 (60 to 70 % H2,
total flow up to 2.2 mg/s) the limit came at 16 to 21 s; the heating law with the conductivity exponent 0.45 says 7
to 9 s, a rate 1.4 to 1.85 times too high. In helium hydrogen it is about right (10 to 14 s forecast, 10 to 19 s
observed).

## 4. Candidates: expectation and result

All runs are in `runs/dev/gen7/whatif_log.jsonl` with the expectation in the note, written before the run. Sensor
labels unless marked. "Neutrality" is the tool's check: CFI-G train, 210 cases, and Flame Design train days, 444
tables.

| # | candidate (edits sha256) | law | changed lines | expected | realised | verdict |
|---|---|---|---|---|---|---|
| – | no-op on stage 2 (`2bae9e67…`) | – | none | identical | identical | tool check, not a candidate |
| M1 | on F3 s1 (`9e90e736…`) | methane 1/5 | hinge members `fuel.s[4]` 0.35 → 1.0 | validation methane 0.245 → about 0.19; the limit-first group 0.53 → about 0.40; train methane unchanged or a hair lower; ethylene, CFI-G, Flame Design exact | 0.2449 → 0.1336 (with stop 0.2452 → 0.1328); limit-first group 0.531 → 0.192; train methane 0.0084 → 0.0081; ethylene unchanged; 0 of 210 and 0 of 444 moved | sign right in every stratum, size underestimated by half; kept as the base of M2 |
| M2 | M1 + growth (`29d2d988…`) | methane 2/5 | growth members `fuel.s[4]` −0.35 → −0.6 | 0.134 → about 0.10; growth clocks 8.5 to 10 s; about 4 s early for the two longest lives; train unchanged | 0.0916 (with stop 0.0911); validation nitrogen pooled 0.0815 against 0.2048 for F3; limit-first group 0.0848; train methane 0.0054; growth clocks 8 to 10 s; 0 and 0 moved | as expected; **final methane lines** |
| – | M2 on the stage-2 base (same edits) | – | – | methane in helium 0.2295 → about 0.20; ethylene in helium unchanged | methane in helium 0.141; ethylene in helium 1.512 unchanged | re-run, not a candidate; sign right, size under |
| He1 | M2 + helium row (`15c4221b…`) | helium 1/5 | every member: `dil.dflux[1]` 0.3 → 1.15, `dil.dlogm[1]` −1.9 → 0.6, `dil.dlnr[1]` −1.25 → −0.75 | ethylene in helium 1.512 → 0.15 to 0.25; methane in helium → about 0.08; nitrogen as M2; 0 and 0 moved | ethylene in helium 0.1799 (with stop 0.1790); methane in helium 0.0064; helium pooled 0.1105 against baselines 0.330 and 0.324; nitrogen as M2; 0 and 0 moved | as expected for ethylene, better than expected for methane; **final helium lines** |
| Hy1 | He1 + hydrogen row (`f81602cb…`) | hydrogen 1/5 | every member: hydrogen admitted, growth `fuel.s[5]` 1.9, hinge `fuel.s[5]` −2.0, `fuel.dtad_k[5]` 900, `fuel.tad_min_k[5]` 1100, `fuel.k_rel[5]` 7.0, coordinates (1, 0) | sensor labels: hydrogen in nitrogen about 0.5 without the stop and 0.25 with it; hydrogen in helium about 0.25; blends in helium about 0.05; day 20006 about 0.10; day 20013 about 0.85, worse than both baselines | sensor: hydrogen in nitrogen 0.607 and 0.515 (day 20024 0.462 and 0.360); hydrogen in helium 0.302; day 20006 0.244; day 20013 1.06. Adjudicated: hydrogen in nitrogen 0.406 and 0.158; hydrogen in helium 0.180; day 20024 0.331 and 0.038; day 19352 0.807 on 5 cases only; day 20155 0.292; day 20190 0.109; day 20006 0.034; day 20013 1.53; day 20132 0.006. **Not foreseen:** three tests of day 19352 refused, because +900 K puts hydrogen flames above 2000 K outside the declared temperature coordinate; 0 and 0 moved | below both baselines where expected and above them on day 20013 as expected; with the planned stop the sensor-label number is twice the expectation, because those labels call five stopped flames extinctions; superseded because of the unforeseen refusals |
| Hy2 | Hy1 with `fuel.dtad_k[5]` 600 (`25d9e25a…`) | hydrogen 2/5 | that one number, back at the Part A value | adjudicated: day 19352 answers about 11 cases and falls from 0.81 to about 0.5; day 20024 with the stop 0.038 → about 0.05; everything else unchanged | day 19352 0.644 on 13 cases; day 20024 0.335 and 0.041; the rest unchanged; re-run with the neutrality check: 0 of 210 and 0 of 444 moved | sign right, day 19352 smaller gain than expected; **final hydrogen lines** |

Two remarks on honesty of the count. The adjudicated-label scores of these same children were computed with my
scratch harness; they are re-scorings of logged candidates, not further candidates. The tool gives a different child
id to two runs of the same edit file, so the edits hash is the stable name of a candidate.

**What the candidates did not fix, test by test.**

- Methane, test 19308A3 (0.51 mg/s, 45 %, out at 14.6 s with the burner at 223 °C): 0.20 per case. The growth clock
  says 8 to 10 s, the hinge clock 18.5 s. It is the flame closest to the limit stop and the longest-lived; one test.
- Ethylene in helium, the two lowest flows: A2 (a true limit stop at 10.3 s) went from 0.009 to 0.353, because the
  hinge members now put extinction at 9.4 s, ahead of the limit at 12 s; P1 0.44, N1 0.30. Here extinction (about 13
  s) and the limit (10 to 14 s) are within one to three seconds of each other and no typed number decides the race.
  The hinge clock is 2 to 3.5 s early on the six answered flames that went out: the data would have chosen a life shift of about −0.5; I kept
  the equivalence value −0.75.
- Hydrogen: the five early quenches (20155A2, A3, 20190A1, 19352D1, D4) cost 1.9 to 2.0 per case, seven cases in
  all, and are most of what is left; the detector stop 20024N2 costs 0.6 without and 0.3 with the planned stop; the
  seven flames that ran to the scripted end cost about 0.33 each without the planned stop, which that path cannot
  know, and nothing with it; the limit stops cost 0.01 to 0.08.

## 5. By day

F4 here is the last child (all three stages). Adjudicated labels; per case, without | with the planned stop where
they differ. "F3" is stage 1 on nitrogen days and stage 2 on helium days.

| day | partition | stratum | cases | F4 | F3 | CFI-G incidence | s-Flame incidence | with sensor labels |
|---|---|---|---|---|---|---|---|---|
| 19254 | train | ethylene, N2 | 28 | 0.086 \| 0.084 | same | 0.305 | 0.325 | same |
| 19256 | validation | ethylene, N2 | 9 | 0.040 | same | 0.223 | 0.291 | same |
| 20029 | train | ethylene, N2 | 15 | 0.093 \| 0.109 | same | 0.208 | 0.291 | same |
| 19304 | train | methane, N2 | 10 | 0.005 | 0.008 | 0.279 | 0.248 | same |
| 19308 | validation | methane, N2 | 39 | 0.040 | 0.190 \| 0.191 | 0.269 | 0.312 | 37 cases: 0.092 \| 0.091 against 0.245 |
| 20160 | validation | methane, He | 14 | 0.006 | 0.230 | 0.279 | 0.270 | same |
| 20164 | validation | ethylene, He | 21 | 0.180 \| 0.179 | 1.512 | 0.364 | 0.361 | same |
| 19352 | train | hydrogen, N2 | 13 | 0.644 | refused | 0.788 | 0.571 | 9 cases: 0.888 |
| 20024 | train | hydrogen, N2 | 27 | 0.335 \| 0.041 | refused | 0.844 | 0.606 | 0.455 \| 0.348 |
| 20155 | train | hydrogen, He | 14 | 0.292 | refused | 0.971 | 0.683 | 0.566 |
| 20190 | train | hydrogen, He | 22 | 0.109 | refused | 1.029 | 0.718 | 20 cases: 0.117 |
| 20006 | train | blend, N2 | 23 | 0.034 | refused | 0.251 | 0.301 | 0.244 |
| 20013 | train | blend, N2 | 18 | 1.533 \| 1.541 | refused | 0.766 | 0.571 | 1.058 |
| 20132 | train | blend, He | 8 | 0.004 | refused | 0.279 | 0.243 | same |

Pooled, adjudicated, without | with the planned stop:

| stratum | cases | F4 | CFI-G incidence | s-Flame incidence |
|---|---|---|---|---|
| hydrocarbons in nitrogen | 101 | 0.057 \| 0.059 | 0.267 | 0.304 |
| hydrocarbons in helium | 35 | 0.110 | 0.330 | 0.324 |
| hydrogen | 76 | 0.315 \| 0.210 | 0.911 | 0.647 |
| blends | 49 | 0.580 \| 0.583 | 0.445 | 0.391 |

Reading against the by-day rule of 13.6.

- **Methane stage.** Lower pooled; lower on 2 of the 2 days whose forecasts change (19304, 19308); no other target
  moves at all. Fewer than six days change.
- **Helium stage.** Lower pooled; lower on 2 of 2 days (20160, 20164) against F3 stage 2, and below both incidence
  baselines on both, where stage 2 was above them. Fewer than six days change.
- **Hydrogen stage.** No current model to compare with (refused). Below both baselines on 3 of 4 days; on day 19352
  (0.644) it is below the CFI-G incidence (0.788) and **above** the s-Flame incidence (0.571).
- **Blends.** Below both baselines on days 20006 and 20132, far above on day 20013. Pooled it is above both.

By the letter of the rule each stage changes fewer than six days and is a reported comparator unless the coordinator
counts the stages together (the hydrocarbon stages change four days, hydrogen answers four more). That decision is
not mine.

Refused under F4, 30 cases: 19308M1 (methane, `T_1` 1846 K below 1900 K); 20029R1 (ethylene, 1829 K below 1900 K);
20013C5 (blend, below the mixed bound); flows above the declared flow coordinate (20132C1 and J1, 20136C1, G1, G2 and
J2, 20160C2 and J1, 20164C1). Several tests of days 20136 and 20155 are not at risk at 1 s and give no case.

## 6. For the engineer

Forward: `acme_normal.wsp13`, unchanged. Slot names as in `configs/models/acme/wsp13/normal/f3-s1`: hinge members
`a0 a1 c0 c1 c2 c3`, growth members `b0 b1 b2 b3`. Fuel rows: 4 methane, 5 hydrogen. Diluent rows: 0 nitrogen, 1
helium. Each stage contains the one before it, so each can be a separate release.

### Stage 1, methane (child of F3 stage 1, `sha256:d1660331…45aa`)

```text
c0.fuel.s[4] = 1.0        # was 0.35; likewise c1, c2, c3
b0.fuel.s[4] = -0.6       # was -0.35; likewise b1, b2, b3
```

Eight lines; the edit file as run has sha256 `29d2d98868e30d6251a647987993c2760252c122cb2d06ff9d3ff3342282f66a` (the
four `c` lines, then the four `b` lines). `a0` and `a1` have weight 0 and were left at 0.35; set them to 1.0 too if
the table is shared.

- Hinge, 1.0: methane's flow coordinate moved up by 0.65 so that flames at 0.4 to 0.8 mg/s are off the two low-flow
  hinges (M-b). Expected from the energy balance (+0.4 to +0.8) and the mass-fraction geometry (+0.1 to +0.3).
- Growth, −0.6: critical flux 0.255 → 0.33 g m⁻² s⁻¹ (M-a; the balance gives 0.31 to 0.42 over the mixtures flown).
- Unchanged: `fuel.dtad_k[4]` −350, `fuel.tad_min_k[4]` 1900.
- Checked: validation methane 0.2449 → 0.0916 (sensor), 0.190 → 0.040 (adjudicated); every forecast on nitrogen
  ethylene identical; CFI-G 0 of 210 cases and Flame Design 0 of 444 tables moved.

### Stage 2, helium (stage 1 plus the helium row; base as run: F3 stage 2, `sha256:819ec0b6…9e51`)

The helium row of every member (all ten slots; the values of the row, not edit syntax):

```text
X.dil.admit     = [1.0, 1.0]
X.dil.mw_g_mol  = [28.013, 4.0026]
X.dil.rho_d     = [1.0, 1.15]
X.dil.d_far     = [1.0, 3.5]
X.dil.le_o2     = [1.0, 1.37]
X.dil.dflux     = [0.0, 1.15]      # was 0.3
X.dil.dlogm     = [0.0, 0.6]       # was -1.9
X.dil.dlnr      = [0.0, -0.75]     # was -1.25
X.dil.k_rel     = [1.0, 5.8]
```

As run on the stage-2 base: the eight methane lines, then for each slot in the order `a0 a1 b0 b1 b2 b3 c0 c1 c2 c3`
the three lines `X.dil.dflux[1] = 1.15`, `X.dil.dlogm[1] = 0.6`, `X.dil.dlnr[1] = -0.75`; 38 lines, sha256
`15c4221b4838916006bd608539f55bf855474689b85354b4f2bca29c5a4537b3`.

- `dflux` 1.15: the energy balance at the helium flame's own temperature, `ℓ*` unchanged (M-a). The data alone would
  say +1.0 to +1.2.
- `dlogm` 0.6 and `dlnr` −0.75: thermal equivalence with a nitrogen flame at 9 % less oxygen (M-c). The data alone
  would say about 0 or above, and about −0.5.
- Checked: ethylene in helium 1.512 → 0.180, methane in helium 0.230 → 0.006; nitrogen days exactly as stage 1;
  CFI-G and Flame Design unmoved.

### Stage 3, hydrogen (stage 2 plus the hydrogen row)

For each of the ten slots, in the order above:

```text
X.fuel.admit[5] = 1.0
X.fuel.s[5] = 1.9            # growth members b0..b3
X.fuel.s[5] = -2.0           # hinge members a0, a1, c0..c3
X.fuel.se_coord[5, 0] = 1.0
X.fuel.ft_coord[5, 0] = 1.0
X.fuel.tad_min_k[5] = 1100.0
X.fuel.dtad_k[5] = 600.0
X.fuel.k_rel[5] = 7.0
```

Seven lines per slot, appended to the 38 of stage 2; 108 lines, sha256
`25d9e25a9686fbe29a87c19f0962b6daea31d894f0fd1cf8e4b41d19f915dfee`. `fuel.mw_g_mol[5]` 2.016 and `fuel.sigma[5]`
7.936 are already in the table.

- Growth 1.9: critical flux 0.027 g m⁻² s⁻¹, the Part A value (hydrogen's heat per unit mass and its diffusivity);
  no hydrogen flame above 0.3 mg/s went out.
- Hinge −2.0: typed from the train days; hydrogen up to 0.6 mg/s in air and 0.8 mg/s in helium did not go out.
- `dtad_k` +600 K: hydrogen's extinction temperature is far below a hydrocarbon's (flames at 1188 K burned to the
  stop). +900 K was tried and leaves the declared temperature coordinate above 2000 K.
- `tad_min_k` 1100: nothing was flown colder than 1188 K.
- `k_rel` 7.0: thermal conductivity of hydrogen against nitrogen.
- **Scoring.** Use the adjudicated endpoints of 2.3 for hydrogen and blends, and for 19308H4.
- **Blends are not handed over.** With hydrogen admitted the forward answers blends through its oxygen-share rule.
  That rule fails on the hydrogen-rich bottle (day 20013: 1.53). Until there is a blend law the release should
  refuse `fuel_blend_x`; the present forward has no switch for that (7.4). If it cannot be refused, report blends as
  a comparator and not as the model.

## 7. Structure the forward lacks

Named, not typed (13.6: a number is not asked to do the work of a missing input).

1. **The critical flux as an energy balance with a per-fuel extinction temperature** (`fuel.tcr_k`; methane 1500
   to 1550 K, ethylene and the alkanes lower, hydrogen far lower): `m_cr = m_cr0 · (Y_F L / Θ) / (Y_F L / Θ)_ref`.
   It would replace the constant shifts `fuel.s` (growth) and `dil.dflux` and would give methane's rise of the flux
   with dilution (0.25 to 0.6 across day 19308), which a constant cannot. For neutrality it has to be switched off
   for the alkanes and ethylene in nitrogen, or typed so that it returns exactly the present constants there.
   A smaller alternative for the hinge members: a per-fuel life shift `fuel.dlnr`, as the diluent table already has.
2. **A flow term in the weak-class clock** (M-d): life of a flame below the limit rising with flow. Needs the
   exponent, which three strata put between 0.3 and 0.9.
3. **An early-quench class for very small flames** (M-e): a probability that the flame is lost within about 5 s,
   as a function of heat release against what the cold burner takes. The weak class today depends on temperature
   only. No new input is needed (fuel, flow and burner are known).
4. **A blend law and a blend admission switch** (M-f). A per-release flag to refuse `fuel_blend_x`, then a law that
   is not linear in the hydrogen share.
5. **Detector stops.** The script's extinction detector ends some dim hydrogen flames that are still burning (2 of
   the 30 hydrogen tests with a trace). It is a property of the instrument at low light; it could be a small constant hazard for
   hydrogen below some heat release. Not predictable case by case.
6. **The heating law at small total flow with much hydrogen** (M-g): 1.4 to 1.85 times too fast on day 19352; and
   its flow exponent in helium ethylene (−0.53 observed against −0.8). The conductivity exponent `bt.kappa_exp` is
   shared with every other flame, so I did not touch it.

## 8. What only the test days can judge

- **The methane step.** Typed from one day at 30 % oxygen and one at 21 %, both now in-sample. Whether 1.0 and −0.6
  hold at another oxygen level or pressure is open; so is the single remaining miss (a 14.6 s flame at 0.51 mg/s).
- **The helium row.** One composition per fuel on two days. `d_far` 3.5 and `dflux` 1.15 are not separable on one
  mixture: any helium flame at another fuel fraction or oxygen level separates them. Ethylene in helium at 21 %
  oxygen was never seen.
- **Which wins at low flow in helium, extinction or the limit stop.** The two are within 1 to 3 s on development.
- **The hydrogen row.** Four days, all train, all read before typing. The no-stop path cannot know the 45 to 47 s
  script stop and will lose about 0.33 per case on every hydrogen flame that runs to it.
- **The early quench of the smallest hydrogen flames.** The model gives it almost no probability; each one costs
  about 2 per case.
- **Any blend rule.**
- **The label rule of 2.3**, if the test days' labels are built the same way: the sensor labels of hydrogen and
  blend tests will be wrong at about the same rate (13 of the 61 hydrogen and blend tests here), and the score of any model on them will
  carry that.

## 9. Corrections to `phys_sflame_f3.md`

- Part B named "lives rising with flow, from heat loss to the burner" as the mechanism missing for the seven low-flow
  methane tests. It is not: at 30 % oxygen lives fall with flow, the burner takes 13 to 22 % of the heat of those
  flames and the one that loses most lives longest. The fault was the hinge law's low-flow branch applied to methane.
  Lives do rise with flow, in flames below the limit only, and not through the burner.
- Part A typed the helium critical-flux shift as a judgement (+0.3). The energy balance already in the project gives
  +1.15, and the data agree with it.
- Part A carried helium's flame size in the two hinge-law equivalents (−1.9, −1.25). The hinge law needs only the
  thermal equivalence (+0.6, −0.75).
- Part B's oxygen-share rule for blends was presented as adequate. It fails for the hydrogen-rich bottle.
