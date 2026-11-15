# Physics of a normal spherical burner flame, for a mechanistic model of how it ends

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. Outcomes and raw channels of train tests only; of validation tests the conditions only.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

# Dossier: ACME CFI-G normal flames — physics for a mechanistic forward

Everything below was evaluated at stated constants; nothing was fitted. Where I compare a few values, each is named and justified.

## 0. Main results

- **The radiative clock has a one-line energy balance behind it.** It reproduces the published constant critical flux, replaces the weak-flame class, and says which typed corrections are physics and which are patches.
- **Pressure belongs in the radiative length, not in the diffusivity.** Train supports a smaller flame and a thinner hot layer at higher pressure, and physics requires D ∝ 1/p. The size of the effect is not fully reproduced, and I say where.
- **The burner limit was 500 °C for every normal-flame session before late August.** The 455/510 mixture in the growth forward is wrong for the two validation sessions.
- **Six train rows have wrong inputs**, and three of them are the evidence for the weak-class boundary.
- **The two "FT" butane flames at 2 atm ended their first hot flame at 41 s and 89 s.** The validation butane session has no catalytic thermocouple, so those conditions should be forecast as self-extinction.

## 1. The physics, mechanism by mechanism

### 1.1 Where the flame sits and how it grows

- Quasi-steady radius: `r_ss = m_tot / (4π ρD L)`, with `L = ln(1/(1−Z_st))` and `m_tot = m_F / Y_F`.
- ρD = 0.0334 g m⁻¹ s⁻¹ (Kim 2023 fit) is a film value. With a hot layer about 10 mm thick, I get 0.030 from first principles: 30 % of the O2 path at ρD ≈ 0.055 (about 1000 K), 70 % at the cold 0.0248. It is pressure-independent.
- Growth is the far-field O2 transient. The forward's erfc form is the exact constant-property solution: `r_f = r_ss · erfc(r_f / 2√(Dt))`. D must scale as 1/p.
- Train confirms a slow transient at every pressure. All "held" flames are still drifting at 270 s: photomultiplier signals fall roughly as t^−0.5 and the burner temperature peaks near 100 s and then declines as the flame recedes.
- **Ignition flash.** Fuel flows about 2 s before ignition. The chamber pressure jumps about 1.2 kPa at ignition for 2.36 mg/s, which is roughly 250 J or 2 s of fuel burned at once. The diffusion flame therefore starts inside a hot ball, not at the burner. This head start matters for flames that live under 15 s.

### 1.2 Energy balance and radiative extinction

Calorimetry from the chamber pressure rise (0.4 V/bar, 83 L) shows that only a small fraction of the heat release stays in the gas between 20 and 60 s:

| Pressure | Retained fraction |
|---|---|
| 0.5 atm | 0.04–0.09 |
| 1 atm | 0.08–0.14 |
| 2 atm | 0.13–0.19 |

So 85–95 % of the heat is radiated or goes to hardware. The wide radiometer agrees: its plateau is 1.03 ± 0.08 V per mg/s of fuel in 37–40 % O2 for ethane and butane, all dilutions, 0.5–2 atm, extinguishing or held. Total radiant fraction therefore does not discriminate; what matters is how far heat must travel before it is radiated.

The flame-sheet balance per unit area, with O2-layer thickness δ_O and radiative length ℓ* (outer conduction–radiation layer and radiating interior in parallel):

```text
(T_ad − T_f) / (T_f − T∞) = δ_O / ℓ*          δ_O = r_f² / r_ss
extinction at T_f = T_cr:   r_ext² = Θ ℓ* r_ss      Θ = (T_ad − T_cr) / (T_cr − T∞)
reduced flow:               Φ = r_ss / (Θ ℓ*)       held if Φ ≤ 1
critical flux:              m″_cr = ρD · Y_F L / (Θ ℓ*)
```

What follows from it:

- **The constant published flux is explained.** Y_F·L/Θ is 0.048–0.058 for every high-T_ad cell (pure or diluted fuel, air or 39 % O2, all three fuels), so m″_cr is constant to ±10 %. More oxygen flux is offset by a larger temperature margin.
- **ℓ* from published numbers.** The ethylene flux of 0.2 g m⁻² s⁻¹ with T_cr = 1130 K gives ℓ* = 7.35 mm. A first-principles optically thin estimate, √(k/h_R) with TNF Planck-mean coefficients at 1000 K, gives 8.8 mm for one layer, larger with re-absorption. The numbers agree in order of magnitude with no fitted constant.
- **The weak class is the same law.** As T_ad approaches T_cr, Θ goes to zero. Flames at 1600–1700 K get Θℓ* of 3–5 mm and die inside the ignition transient. The one genuine 1795 K flame (21125A2) is predicted at 7–11 s and lived 8 s.
- **Pressure.** ℓ* ∝ p^−b: b = 0.5 if optically thin, less with re-absorption, more if soot contributes. A rough thermocouple estimate of the outer layer gives about 10.7, 9.3 and 7.2 mm at 0.5, 1 and 2 atm (b ≈ 0.28). That estimate assumes the flame radius is the same at all three pressures, which the next point contradicts; it is not established.
- **Direct size evidence (October, near rake).** At the same time and flow, the 25 mm thermocouple reads 200–250 K lower at 2 atm than at 1 atm. This shows a smaller flame, a thinner layer, or both.
- **Extinction temperature for butane.** The 25 mm thermocouple reads 995–1088 °C (bead) at the instant of extinction in all five 1-atm self-extinctions, at 1.0–1.5 mg/s and lives of 27–161 s. Either T_cr for butane is near 1350–1400 K, or the flame is hotter and still inside 25 mm; one probe cannot separate them. I carry T_cr as 1130–1400 K.
- **Soot and fuel.** The interior at 13 mm is 500–600 K colder than the 25 mm reading late in the 1-atm tests. That implies a strong interior sink (fuel C–H bands, pyrolysis, trapped soot). The early luminous phase on the broadband photomultiplier lasts about 3, 4 and 12 s at 0.5, 1 and 2 atm for the same ethane flame. This is the candidate for a pressure exponent above 0.5 and for the butane effect.

### 1.3 Does this explain the four unexplained trends?

| Trend | Verdict | Mechanism |
|---|---|---|
| Lives shorten with pressure | Partly | m″_cr ∝ p^b with D ∝ 1/p. b = 0.5 gives the right sign near the holding flow and a flat response far from it; train shows about p^−0.4 everywhere. The holding flow should fall as p^−b; train cannot exclude that (see 2.2). |
| Clock twice as fast in air | Not by the law | All four evaluated corners are 2–2.5× too long in air. Candidates: the flash head start (larger at low Z_st), a higher T_cr. Open. |
| Butane 0.5–0.7 of ethane in enriched O2 | Not by the law | Theory predicts butane slightly longer (smaller r_ss). The holding flow is about the same for both fuels; the gap is at high flow. Consistent with soot or interior radiation growing with flame size. Not demonstrated. |
| Dilute burner gas burns longer than the law | Plausible | A nitrogen-diluted interior is cooler and radiates less (larger ℓ_i). |

### 1.4 Flame held by the burner at low flow

Heat into the burner over 3–6 s (0.308 J/K × dT/dt) is 1.1–1.7 W at 2.36 mg/s (1–2 % of the heat release), 3–4 W at 0.75–1.0 mg/s (6–12 %), and 5–7 W at 0.5 mg/s (21–30 %). At 0.5 mg/s the burner is a first-order heat sink for the flame; this is the low-flow limit ("lowest flow rate for ignition" in a train comment). No train flame self-extinguishes there, because the burner limit ends the test first.

### 1.5 Burner temperature history

```text
C_b dT_b/dt = Q_cond + Q_rad,in − ε σ A_b (T_b⁴ − T∞⁴) − Q_tube − η m_tot c_p (T_b − T∞)
Q_cond = 4π k (T_f − T_b) · x/(eˣ − 1) / (1/r_b − 1/r_f),    x = (m_tot c_p / 4π k)(1/r_b − 1/r_f)
```

- **Why more than one time constant:** the source decays as the flame recedes and cools; the radiative loss is nonlinear (linearised time constant about 80 s at 600 K, 50 s at 700 K); the supply tube keeps warming.
- **What sets the asymptote:** the burner's own radiation, 1.0–1.05 W at 420–435 °C and 1.55 W at 500 °C, against an input of about 1–2 W late in the test.
- **Blowing:** conduction is cut strongly by total flow, but there is a floor of 1.1–1.5 W at high flow that conduction cannot supply (it gives about 0.1 W). I attribute it to radiation from the flame ball and tube conduction.
- **Gas heating:** the asymptote balances only if η is 0.1–0.4, consistent with Waddell's finding that the gas-heating term must be dropped.

### 1.6 Cool flames (n-butane at 2 atm)

Train confirms the published rule: a cool flame follows hot extinction if the burner is above about 515 K.

| Test | Burner at hot extinction | Radiometer after extinction |
|---|---|---|
| 21291A1 (2 atm) | 270 °C | plateau 0.43–0.55 V |
| 21291C1 (2 atm) | 408 °C | plateau 0.36–0.39 V |
| 21291B1, C2, H1 (2 atm) | 177–215 °C | decays to 0.09–0.10 V |
| 21291D1 (1 atm) | 257 °C | decays to 0.09 V |

The cool flame radiates 30–45 % of the hot signal, and the burner keeps warming.

In 21291A1 and C1 the hot flame returned five times each (dark gaps of 10.0 then about 2.7 s, and about 5–5.6 s), and they are labelled FT at 167 and 246 s. The literature attributes hot reignition to the platinum thermocouple, which was installed only on that day. The first hot extinctions, 41 s and 89 s, are the physical outcomes.

### 1.7 Property data

| Quantity | Value | Source |
|---|---|---|
| Lower heating value | ethane 47.51, propane 46.33, n-butane 45.72 kJ/g | NIST, via helper; matches standard values |
| O2 per gram fuel | 3.73, 3.63, 3.58 | stoichiometry |
| D(O2–N2), 300 K | 21 mm²/s, ∝ T^1.7/p | Marrero & Mason, via helper; matches standard value |
| D(fuel–N2), 300 K | ethane 15.6, propane 12.1, butane 10.5 mm²/s | McGivern & Manion, via helper (not re-checked; not used in any number here) |
| k(N2) | 0.026, 0.045, 0.065 W/m/K at 300, 600, 1000 K | helper |
| Planck-mean a_P at 1000 / 1200 / 1500 K | CO2 27.4 / 20.1 / 12.3; H2O 5.6 / 3.9 / 2.4 m⁻¹ atm⁻¹ | TNF fits; I recomputed the 1000 K values |
| Burner | 0.308 J/K (4.6 g/cm³, 500 J/kg/K), ε 0.6, area 1.267 cm² | Waddell thesis, apparatus documents |
| Critical temperature | 1130 ± 23 K (ethylene, 1 bar); about 50 K lower at 3 atm | Irace 2023 abstract, Waddell thesis |

## 2. Confrontation with train

### 2.1 Six train rows have wrong inputs

Fuel-controller readbacks show:

- **21125K1, K2, K3:** the dilution controller is off (−0.001 V). The burner gas is the undiluted 30 % bottle, not 3.1 %. True T_ad is about 2650 K and Z_st about 0.20; the table says 1795 K and 0.657.
- **21106G6, G7:** true flow 0.49 mg/s (same voltage as the 0.5 mg/s test), table 0.8.
- **21106G9:** true flow 0.66 mg/s, table 0.8.

The "8–35 s and one held flame at 1785–1795 K" behind the weak-class logistic is therefore one real test (21125A2, 8 s) and three mislabelled ones.

### 2.2 Lives against the clock

The erfc clock with r_ext² = Θℓ*r_ss, ℓ* = ℓ1·p^−0.5, D = D1/p, T_cr = 1130 K and a 3 s offset, evaluated at four corners. ℓ1 = 7.35 mm is the published ethylene value; 11.0 mm is 1.5× that for a weaker interior emitter. D1 = 21 mm²/s is O2–N2 at 300 K; 42 is a 450 K film. Two corners shown; lives in seconds.

| Cell | Observed | ℓ1 7.35, D1 21 | ℓ1 11, D1 42 |
|---|---|---|---|
| 30 % ethane, 1 atm, 39–40 % O2: 2.36 / 1.5 / 1.0 / 1.0 / 0.85 mg/s | 43 / 71 / 232 / 158 / 304 | 37 / 42 / 62 / 57 / 73 | 47 / 71 / 262 / 210 / 656 |
| same, 2 atm: 2.36 / 1.5 / 1.0 / 0.75 | 34 / 44–51 / 123–131 / alive at 276 | 38 / 37–38 / 43–44 / 56 | 42 / 47–49 / 78–79 / 183 |
| same, 0.5 atm: 2.36 / 1.5 / 1.0 (36.4 % O2) / 0.75 (33.2 %) | 59 / 88 / 258 / alive at 274 | 39 / 52 / 111 / 254 | 59 / 149 / held / held |
| 30 % ethane, air, 1 atm: 1.0–2.36 / 0.5 | 9–10 / 16–19 | 19–22 / 23–24 | 21–22 / 47–51 |
| 30 % butane, 1 atm: 3.0 / 1.5 / 1.25 / 1.0 | 17 / 35–47 / 60 / 161 | 40 / 51 / 59 / 92 | 48 / 103–106 / 173 / 1384 |
| 30 % butane, 2 atm: 1.75 / 1.5 / 1.0 (first hot extinction) | 27–32 / 41 / 89 | 40 / 43 / 53 | 51 / 61 / 119 |
| 30 % propane, air, 2–3 atm | 5–7 | 21–27 | 23–65 |
| 3 % butane, 1795 K | 8 | 11 | 16 |

What this says:

- **Ethane at 1 atm** is reproduced by the (11 mm, 42 mm²/s) corner within 15 %, except 0.85 mg/s. The implied holding flow is about 0.75 mg/s; the published ethylene ℓ* puts it at 0.53.
- **Pressure:** with b = 0.5 that corner is too short near the holding flow at 2 atm and too long at 0.5 atm. A smaller b (0.3–0.4) would fix that but flattens the high-flow response further. No single b reproduces both ends.
- **Holding flow against pressure:** the train evidence is confounded by oxygen drift. At 1 atm, 39.0 → 37.9 % O2 cuts the 1.0 mg/s life from 232 to 158 s; the 0.5-atm near-holding tests ran at 33–36 % O2.
- **Butane** needs ℓ1 near 7.4–8 mm, and even then 3.0 mg/s is 2× too long.
- **Air, and propane in air at pressure,** are 2–4× too long at every corner.

### 2.3 Burner

- The initial slope predicts the outcome. Over 3–6 s: 14 K/s or more reaches 500 °C (four tests, 14.4–23 K/s, peaks 506–531 °C); 12.4–13.6 K/s peaks at 435–480 °C; 11.6 K/s or less stays under 425 °C.
- Limit trips: 502–522 °C in March–June, 451–455 °C in October.
- Every programmed stop is a script parameter: 55, 90, 120, 180, 240 or 270 s of constant flow. Inside 120 s it produced only four train FT rows (three on the first day, one 55 s repeat on 2 June).

## 3. Audit of the two forwards

**Hinge forward** (`flame.rs`, `configs/models/acme/wsp2/normal/d3-fable/edits.wsp`)

| Tensor | Verdict |
|---|---|
| Reduced flow z, O2 coefficient −0.65 | Mechanism (L and Θ). Physics gives about 0.45–0.55 per 10 % O2; same sign, typed about 30 % stronger. |
| Reduced flow z, pressure +0.24 | Mechanism: this is b. Sign and size agree (0.3–0.5). |
| `se.tau.skip[2] = +0.84` (O2 at fixed z) | Mechanism: time scale ∝ Θ². Physics 0.3–0.5; typed about 2× larger. |
| `se.tau.skip[0] = −0.28` (pressure at fixed z) | Patch. Physics gives (Θℓ*)²/D ∝ p^(1−2b), zero or positive for gas radiation. |
| Hinges 1.4 and 2.0 | Approximate the divergence of the erfc clock (local slope about 0 at Φ = 4, −1.75 at Φ = 1.9, unbounded at Φ = 1). No true holding; harmless inside 120 s. |
| Units 2–5 (butane, propane) | Empirical. Soot is plausible for "enriched O2 only"; the dilution units duplicate what m_tot/L already does. |
| `ft.tau` | Empirical clock for a threshold phenomenon. Sign right; it gives a finite hazard where a 500 °C limit is never reached. |
| `se.floor_s = 4` | Ignition transient. Keep. |

**Growth forward** (`flame3.rs`, `configs/models/acme/wsp3/normal/d1-fable/edits.wsp`)

| Tensor | Verdict |
|---|---|
| `se.rho_d_g_m_s = 0.0334` | Mechanism. |
| `se.flux.b0` (0.18 g m⁻² s⁻¹, constant) | Mechanism, good to ±10 % for T_ad above 2000 K. Fails as T_ad approaches T_cr, where the flux should diverge. |
| `se.flux` unit 0 (+0.5 per e-fold of pressure, air only) | Right mechanism, wrong scope: physics applies it at every O2 level. |
| `se.diff.skip[0] = +0.45` | Sign contradicted: a diffusivity cannot rise with pressure. The trend belongs in the flux, with D ∝ p^−1. The two placements differ on the holding flow at 0.5 and 2 atm. |
| `se.diff.skip[2] = −0.385` (O2) | Patch on the wrong variable. Not a diffusivity; candidates are the flash head start and Θ. |
| `se.diff.skip[3] = +0.2`, units 0 and 1 (butane, propane) | Patches for interior or soot radiation; plausible sign. Train says the butane effect is at high flow, not at the holding flow. |
| `se.flash` (logistic at 1740 K) | Replace with Θ. Its boundary evidence is three mislabelled rows. |
| `se.sigma_phi = 0.25` | Reasonable: ±5 % flow, ±0.01 O2, session-level ℓ* and ρD. |
| `bt.rise` (2012 K, −0.4, −0.4), `bt.tau_s = 30` | Empirical. Both exponents have mechanisms (blowing, standoff). A single time constant is not physical. |
| `bt.limit_c / bt.limit_w = [455: 0.4, 510: 0.6]` | Wrong for unseen sessions: should be about 505 °C with weight near 1 before late August. |
| `bt.t0_c = 30` | Measured 24.3–25.4 °C. |
| `ft.atom_*` | Policy. The 61.5 and 98 s atoms are first-day scripts; later sessions used 120–270 s. |

Both forwards learn from 21291A1 and C1 that 2-atm butane at 1.0–1.5 mg/s survives past 120 s. Physically those flames went out at 41 and 89 s.

## 4. Blueprint for a mechanistic forward

**State:** r_f(t), T_f(t), T_b(t). All explicit.

```text
1. derived     Y_F, m_tot, L, r_ss, Θ = (T_ad − T_cr)/(T_cr − T∞)
2. radiative   ℓ* = ℓ1(fuel) · p^−b · (1 + c_dil · ln(0.3/X_F))  /  (1 + soot(fuel, p, T_ad, r_ss))
               Φ = r_ss / (Θ ℓ*);   r_ext² = Θ ℓ* r_ss
3. growth      t(r_f) = r_f² / (4 D erfcinv(r_f/r_ss)²) − t_head,   D = D1/p
               (explicit in r_f: tabulate on a grid of r_f and interpolate)
4. flame T     T_f = T∞ + (T_ad − T∞)(1 − Q_b/Q_C) / (1 + r_f²/(r_ss ℓ*))
5. extinction  when T_f ≤ T_cr, i.e. t_ext = t(r_ext) + t_0; integrate over ln Φ nodes as now
6. burner      the ODE of 1.5, explicit Euler at 0.1 s; stop when T_b ≥ T_lim for t_dwell
7. stops       session script duration as a distribution; background rate
```

No solver is needed: step 3 is a table lookup and step 6 is forward Euler.

**Parameters**

| Group | Parameter | Proposed | Uncertainty | Kind |
|---|---|---|---|---|
| Transport | ρD | 0.0334 g m⁻¹ s⁻¹ | ±15 % | constant, session-biased |
| | D1 (far-field O2) | 30 mm²/s at 1 atm, ∝ 1/p | 21–45 | constant |
| Radiation | T_cr | 1130 K | 1130–1400; fuel-dependent | constant per fuel |
| | ℓ1 ethane | 10 mm | 7.4–11 | constant per fuel |
| | ℓ1 butane | 8 mm | 7–9.5 | constant per fuel |
| | ℓ1 propane | 9 mm | 7.4–11; no enriched-O2 train test | constant per fuel |
| | b | 0.45 | 0.3–0.7 | constant |
| | soot term | 0 for ethane; grows with flow for butane | sign only | open |
| | c_dil | 0.2 | 0–0.4 | open |
| Ignition | t_head (flash head start) | 2 s | 0–5; larger at low Z_st | per test |
| | t_0 (establishment) | 3–4 s | ±1 | constant |
| Burner | C_b, ε, A_b | 0.308 J/K, 0.6, 1.267 cm² | C_b ±30 % | hardware |
| | k (film), c_p | 0.055 W/m/K; mixture value | ±20 % | constant |
| | Q_rad,in floor | 1.2 W | 0.8–1.6 | open |
| | η (gas heating) | 0.25 | 0.1–0.4 | hardware |
| | T_lim, t_dwell | 500 °C, 4 s (before late August); 450 °C after | trips at 502–522 and 451–455 | campaign era |
| Stops | script duration | 120 or 270 s for later sessions | session policy | session |
| Noise | ln Φ within session | σ 0.07 | flow ±5 % | per test |
| | ln ℓ*, ln D between sessions | σ 0.15–0.2 | | per session |

**Keep from the existing forwards:** the Φ-node integration, the competing-cause split, the hold rate, the floor, and the heating law as a fallback until the ODE is typed (it is within ±8 % of train peaks). Move the pressure exponent from `se.diff` to the flux, replace `se.flash` by Θ, and set the limit by era.

## 5. Unseen validation regimes, stated before any outcome

Estimates combine the nearest train cells with the physical direction. Ranges are wide on purpose.

**30 % propane, 1 atm (2021-05-17).** No burner-limit stop is expected inside 120 s: the lowest enriched flow is 1.0 mg/s, and the limit is 500 °C.

| Condition | Expected hot-flame life | Cause | Confidence |
|---|---|---|---|
| 39.5–40 % O2, 2.89 mg/s | 28 s (18–40) | radiative | medium |
| 39.5–40 % O2, 1.5 mg/s | 55 s (38–80) | radiative | medium |
| 39.5–40 % O2, 1.0 mg/s | 190 s (110–350); P(out before 120 s) about 0.2 | radiative, later | low |
| 35.5 % O2, 1.4 / 1.25 / 1.2 / 1.1 mg/s | 40 / 50 / 55 / 70 s; P(out before 120 s) 0.9 / 0.85 / 0.8 / 0.7 | radiative | medium-low |
| 15 % fuel, 39 % O2, 2.89 mg/s | 20 s (13–30) | radiative | medium |
| 8 % fuel, 37 % O2, 1.5 mg/s (T_ad 2216 K) | 15 s (8–28) | radiative, small Θ | low |
| Air, 30 % fuel, 2.89 mg/s | 8 s (6–11) | radiative | medium-high |
| Air, 30 % fuel, 0.5 mg/s | 14 s (8–22) | radiative | medium |
| Air, 15 % fuel, 0.5 mg/s | 12 s (7–20) | radiative | medium |

Direction against train: between ethane and butane in enriched oxygen; longer than propane in air at 2–3 atm (5–7 s); slightly shorter than ethane in air.

**30 % butane (2021-06-23).** Limit 500 °C. No catalytic thermocouple that day, so a hot flame that goes out is labelled SE at that moment even if a cool flame follows.

| Condition | Expected | Cause | Confidence |
|---|---|---|---|
| 0.5 atm, 3.2 / 2.0 / 1.5 mg/s | 24 / 42 / 60 s | radiative | medium |
| 0.5 atm, 1.0 mg/s, 37.2 % O2 | alive at 120 s (P about 0.8) | later extinction or stop | medium |
| 0.5 atm, 1.0 mg/s, 35.5 % O2 | 150 s (80–300); P(out before 120 s) about 0.35 | radiative | low |
| 2 atm, 3.2 / 2.0 mg/s | 15 / 24 s | radiative | medium |
| 2 atm, 1.5 mg/s (five tests, O2 39.5 → 38.4 %) | 38 falling to 33 s (26–50); cool flame follows | radiative, labelled SE | medium-high on label, medium on time |
| 2 atm, 1.0 mg/s (two tests) | 90 s (60–140); P(out before 120 s) about 0.7 | radiative | medium |
| 2 atm, 0.75 mg/s | alive at 120 s 0.6; limit trip 0.25; out 0.15 | burner peak estimated 480–500 °C | low |
| 2 atm, 0.5 mg/s | FT at about 35 s (25–50), P about 0.85 | burner limit | medium-high |

The 2-atm, 1.0–1.5 mg/s cells are where the current forwards most likely err, having learned "FT after 120 s" from the two cycling train flames.

## 6. What cannot be predicted from the conditions

- **True flow and fuel fraction.** Six of 81 train rows are wrong. The controller readbacks in the first second settle it.
- **Burner-limit outcome near threshold.** The 3–6 s burner slope decides it (thresholds in 2.3).
- **Limit setting and script duration.** These are session facts; a script-parameter record would fix them.
- **Near-holding lives.** 1 % of oxygen changes a 230 s life by a third; flow is ±5 %.
- **Soot loading.** Duration of the luminous phase on the broadband photomultiplier.
- **Ignition flash size.** Chamber pressure jump at ignition, or pre-flow time.
- **Cool flame and reignition.** Burner temperature at hot extinction; whether a platinum thermocouple sits at 13 mm.
- **Operator stops and aborts.**

## 7. Review aid: claims to try to break

| Claim | How to re-check |
|---|---|
| 21125K1–K3 are 30 % butane; 21106G6/G7/G9 flows are 0.49/0.49/0.66 mg/s | `mfc2_V`, `mfc3_V` in `runs/dev/research5/acme_raw/tests_burner.csv`; raw `_data.csv` columns MFCA2/MFCA3 |
| Thermocouple assignment: TCB at 13 mm, TCA at 25 mm on 21291 | My inference from which heats first (TCB within 1 s, TCA after about 5 s). Check the apparatus documents. Type-B table in `scripts/tclib.py` is typed from memory; verify against NIST. |
| Retained heat 4–9 %, 8–14 %, 13–19 % at 0.5, 1, 2 atm | `python runs/dev/research6/acme_normal/scripts/metrics.py`; assumes 0.4 V/bar, 83 L, γ = 1.4 |
| Radiometer plateau about 1.03 V per mg/s | same script, column `rad_end` |
| Y_F·L/Θ constant to ±10 % | hand calculation in 1.2; recompute from `scripts/cond.py` |
| Clock table and the "no single b" statement | `python runs/dev/research6/acme_normal/scripts/clock_table.py`; output in `clock_table.txt`. Only b = 0.5 is in that table; b = 0, 0.28 are in `clock_eval.py`. |
| Burner slope thresholds and 1.1–1.5 W floor | `python runs/dev/research6/acme_normal/scripts/burner_check.py`; the r_f used for Q_cond is illustrative |
| Limit 500 °C before late August | `T_burner_at_off` in `runs/dev/research5/acme_raw/tp_timelines.csv` (four normal-flame trips at 502–522 °C; two in October at 451–453 °C) |
| Cool-flame rule and first hot extinctions 41 s / 89 s | dark-interval scan was run inline, not saved; rerun on `traces/21291A1.npz`, `21291C1.npz` (PMT sum below 0.03 V) |
| ℓ* outer-layer exponent 0.28 | Weakest number here: one probe radius, equal flame radius assumed. Treat as unverified. |
| "No hot reignition without the platinum thermocouple" | Rests on the published statement only |

## Sources

**Read by me (repository):** `docs/wsp/EXPERIMENTAL_SCIENCE.md` §2, 5.2, 6.2 (full); `docs/wsp/research/lit_spherical_flames.md` and `exp_acme.md` (full); cards `acme_clock.md`, `acme_growth.md` (full), `acme_blend.md` (not read); both `edits.wsp` (full); `flame3.rs` (header, clocks, integrate, forecast); `flame.rs` (not read; audited from its edits file and card); `runs/dev/research5/acme_raw/` summary, tables and scripts; `runs/dev/research5/shift/tables.txt` (normal-flame section).

Those repository documents contain published aggregate CFI-G figures (hot-flame radii 12.4–17.7 mm at 2 bar, cool-flame conditions). I used the radii once as a soft cross-check during reasoning; no number in this dossier rests on them.

**Literature helper** (one agent, launched on opus at medium effort before the instruction to use Fable at high effort arrived; no further helper was launched). What it read:

| Source | Amount |
|---|---|
| Kim et al., PCI 39 (2023) 1647 (CFI-G) | apparatus and analytical model only |
| Waddell, UMD dissertation 2023 | chapters 1–4 in part, ethylene |
| Irace CNF 229 (2021); Irace PCI 39 (2023) 1905; Irace thesis | abstracts |
| Irace PCI 39 (2023) steady-state paper | none; conclusion via Waddell |
| TNF Workshop radiation page | full |
| Santa et al. AIAA paper (NTRS 20070010443); Law, Yoo & Christiansen (NTRS 20040053573) | full |
| Santa 2006 thesis | nearly full |
| Wang 2007 thesis; Nayagam et al. 2014; Farouk et al. | parts |
| Mills & Matalon; Chao, Law & T'ien; Tse et al.; Nayagam 2012; McCrain & Roberts | abstracts |
| Sunderland et al. CST 2004 | full except per-flame table |
| Lecoustre et al. CNF 2012; Waddell, Dietrich & Nayagam CNF 2026 | full |
| Li & Sunderland 2012; McGivern & Manion; Marrero & Mason; NIST | tables |
| Stocker 2014; Stocker JASMAC 2021 | instrument passages only |

The helper reported one breach: while searching the Kim 2023 text for headings, lines from its results section and fragments of Table 1 appeared in its tool output, and one keyword search of the JASMAC paper surfaced a CFI-G headline finding. It did not pass any of that to me and I used none of it.

Helper numbers I rely on and re-checked: Planck-mean coefficients (recomputed at 1000 K), heating values, D(O2–N2), 1130 K. Used qualitatively only: re-absorption statements, soot pressure exponents, sooting limits.

## Files written

All under `D:\dev\SpaceApps-Ardealu\runs\dev\research6\acme_normal\`:

- `scripts\extract_traces.py`, `tclib.py`, `cond.py`, `clock_eval.py`, `clock_table.py`, `metrics.py`, `burner_check.py`
- `traces\<test>.npz` (80 train tests at 0.2 s; 21130A9 has no data file) and `traces\headers.json`
- `train_metrics.csv`, `burner_check.csv`, `clock_table.txt`
- `validation_conditions_only.json`

No git-tracked file was modified.

## Statement on protected data

- **Validation:** I read only the `conditions` block of the 28 validation records (fuel, pressure, flow, fuel fraction, O2, T_ad, Z_st), through `lib.experiments`. No outcome, raw file or script-parameter file of a validation test was opened. The oxygen sequence within a fill encodes how much fuel the previous test burned; I did not compute or use it.
- **Test partition:** nothing was read. No `holdout` folder, `--final-test` or `tools/wsp/dev_eval.py` was touched.
- **Raw files:** only train tests, selected by `lib.partition_ids("acme_normal", "train")`.
- **Published per-test CFI-G results:** none read by me; the helper's inadvertent exposure is described above.
- `runs/dev/data_ext/` was not read.
