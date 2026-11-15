# Physics of concurrent flame spread over the SIBAL fabric, for a mechanistic front model

> Written by a delegated analysis agent (claude-fable-5-2, high effort) on 14 November 2026 for the fifth generation;
> nothing fitted. Of PSI-25 only the four train burns and dataset-level documents were opened; the earlier campaign (PSI-26) in full, from tracks that were still being verified.
> It is a proposal to the author, not a model: no number here is a weight until the author has rechecked it
> on train and typed it. What was rechecked is recorded in `docs/wsp/dossier/03_datasets_and_measurements.md`.
> Paths under `runs/dev/` are local working folders that are not in git.

# Dossier: BASS-II concurrent flame spread over SIBAL, for a mechanistic forward

**Bottom line.** The physics does not justify replacing the carried forward's skeleton, and both first-principles transient closures I ran scored worse than its law path. What it adds is one relation that holds without any adjusted constant, a derivation of the typed speed line (so its oxygen and width scalings are no longer guesses), two width and time-constant errors in the carried forward, and an explanation of why the quench law failed.

Everything below is direct evaluation at stated literature constants. Nothing was fitted or scanned, and I launched no helpers.

## 1. The physics, mechanism by mechanism

### 1.1 Property data

| Quantity | Value | Source |
|---|---|---|
| SIBAL areal density | 18.05 mg/cm² total, 13.54 mg/cm² cotton; per face 90 and 67.7 g/m² | Saffire manuscript via `lit_flame_spread.md` [14]; BASS-I paper gives 18.0 |
| Thickness; weave | 0.32 mm; 60 × 40 threads per inch | BASS-II summary report via `exp_bass.md`; BASS-I paper |
| Specific heat | cotton 1.26 J/g/K, glass 0.7; blend 1.12 at 300 K | Ferkul (0.30 cal/g/K); [14] |
| Pyrolysis | onset about 673 K; first order, A = 1.626e11 1/s, E = 161.44 kJ/mol | [14] |
| Pyrolysis e-folding time | 58 s at 650 K, 21 s at 673 K, 6.9 s at 700 K, 3.2 s at 720 K | evaluated |
| Latent heat of pyrolysis | 0.753 kJ/g (180 cal/g at 300 K) | Ferkul CR-191111, nomenclature |
| Heat of combustion; O2/fuel mass ratio | 16.74 kJ/g; 1.185 | Ferkul |
| Emissivity; re-radiation | 0.8; 7.7, 8.9, 11.8 kW/m² at 650, 673, 720 K | [14]; evaluated |
| Gas at 1250 K | k = 0.0808 W/m/K, ρ = 0.275 kg/m³, μ = 4.1e-5 Pa·s, ν = 1.49e-4 m²/s, α = 2.13e-4 m²/s, c_p = 1381 J/kg/K; ρ∞ = 1.15 | Ferkul |
| Adiabatic flame temperature | 1850, 1945, 2037, 2202, 2261 K at 16.4, 17.6, 18.8, 21, 21.8 % | Ferkul's eq. 83, evaluated |
| Per-face loads | 37.6 kJ/m² to reach 673 K; 51 kJ/m² latent; 1133 kJ/m² chemical | evaluated |

### 1.2 Spread rate: oxygen delivered equals fuel consumed

The flame is anchored at the burnout line (Ferkul: peak reactivity sits very near the burnout point). It burns the oxygen the flow carries into a layer δ on each face:

`½(v_tip + v_base) · m_F · r = ρ∞ · Y_O · U · δ`, with `δ² = ν*·L/U + (α*/U)²`

- L is the visible length (tip minus base), m_F the cotton per face, r = 1.185.
- The first term in δ is the boundary-layer thickness at the tip; the second is the thermal-diffusion length (Ferkul's reference length).
- The mean of tip and base speed appears because a growing flame also consumes the fuel inside its lengthening pyrolysis zone.
- This is Vetturini's scaling (ρ_s·v/L ∝ k(T_f − T_p)·√(U/L)/L_v) with the constant supplied by oxygen stoichiometry.

It is close to linear in U because L itself grows about linearly with U, so δ stays near 6–7 mm.

**Steady closed form.** With one time `t_res = L/v`:

`v = [a + √(a² + 4b)]/2`, where `a = K²·Y_O²·ν*·U·t_res`, `b = K²·Y_O²·α*²`, `K = ρ∞/(r·m_F)`

- Intercept: `K·Y_O·α*` = 0.64 mm/s = 0.029 w/s at 18.8 %. It scales as Y_O¹, 1/m_F, and 1/W in widths.
- Slope: `K²·Y_O²·ν*·t_res` = 0.0061 w/s per cm/s at 18.8 % with t_res = 10 s. It scales as Y_O²·t_res/W.
- The carried forward typed 0.0215 + 0.0064·U. The intercept is diffusive oxygen supply, not an artefact; the quench limit cuts it off.

### 1.3 Flame length, burn-out, tip and base

- **Burn-through.** `t_b = m_F·L_g / (k·ΔT_f/δ − q_rad)`. With δ = 7 mm at 18.8 % this gives about 13 s; measured L/v is 9.3–10.7 s. It does not depend on U while δ is constant, and it diverges as the flame flux approaches the re-radiation.
- **Length.** `L = v·t_res`. Surface re-radiation caps the pyrolysis zone where the flame flux falls to q_rad, which gives L ∝ U and independent of areal density (Vetturini eq. 5; Ferkul Fig. 27).
- **Measured L/U:** 0.24–0.37 s on the wide sample, 0.11–0.16 s on the narrow one.
- **Measured t_res:** 9.3–10.7 s (train, 2.4–3 cm/s, 17.6–18.8 %), 14–15 s (BASS-I wide, 3–5 cm/s, 22 %), 8.5–8.9 s (narrow, 21.6 %).
- **What "overhang" is.** Published work has L_f ≈ L_p for this fabric, and Ferkul's model has the visible flame shorter than the pyrolysis zone at low speed. The carried forward's overhang is the fast-responding part of the length, not a physical overhang.

### 1.4 Response to a flow step

| Process | Time scale | Basis |
|---|---|---|
| Gas phase | L/U ≈ 0.3 s | evaluated |
| Shrinking | 1–2 s | a 25 K surface cooling halves pyrolysis (E/RT² = 0.04 per K); C·25 K/5 kW/m² ≈ 0.6 s |
| Growing | 10–20 s or more | fresh fuel must be heated (37.6 kJ/m² at a few kW/m²) and the pyrolysis front must travel |
| Base after a step down | keeps the old speed for about t_b | it runs through the old, long, partly burnt zone |

- Through the T5 ramp (61→42 over 6 s) the measured length stays within ±0.12 w of the steady length at that second's flow.
- Growth is slow: BASS-I flames are steady only after 17–20 s at 5–11 cm/s, and never at 10 cm/s or more on the 2 cm sample.

### 1.5 Ignition transient

- Ignition comes about 2 s after igniter-on (BASS-I paper).
- The flame stays anchored at the edge until the first burn-through. The base takes off 8.8–13.2 s after igniter-on on the 2 cm sample.
- At igniter-off the length is 0.42–0.85 of steady for 5–6 s of igniter, and 1.33 for 9.7 s of igniter (T16, which then shrinks 0.28 w in 2.5 s).
- In the first 5 s the tip runs 1.05–1.46 times the law while the flame is still lengthening, and 0.48 times after the long igniter.
- The flame forgets the igniter in about one t_res (10–15 s).

### 1.6 Quench, blow-off, and the class

- **Low-speed limit.** The flame flux scale is ρ·c_p·U·ΔT_f (Ferkul's radiation parameter is σεT³/(ρc_pU)). It equals q_rad(673 K) at 1.9, 1.8, 1.7, 1.55, 1.5 cm/s for 16.4, 17.6, 18.8, 21, 21.8 %. Adding the pyrolysis demand gives 3.25 to 2.57 cm/s.
- **Oxygen dependence.** This balance moves only about 0.07 cm/s per % O2. The steeper published dependence and the minimum oxygen come from chemistry near the nose of the U-shaped boundary, where peak temperature falls to roughly half the adiabatic value.
- **Blow-off.** Da = α*·ρ*·B_g/U², so the blow-off speed goes as exp(−E_g/2RT_f). BASS-I shows none at 44 cm/s in 22.7 %.
- **Why a class.** T10 (16.4 %, 6 cm/s) is three times above the radiative quench speed and still never spread. Whether a sample carries a flame is set at the kinetic nose and by the ignition event, not by a rate per second.
- **Extinction after a step below the limit** takes one or two burn-through times (BASS-I test 5: about 5 s). It can also come as the flame moves into the thicker boundary layer downstream.

### 1.7 Narrow sample and the fibreglass

- **Side losses.** Standoff of 6–7 mm against a 12 mm width makes the narrow flame all edge. The loss fraction 2δ/W is about 1.1, against 0.6 on the wide sample.
- **Holder loss** scales as 2/W. Train post-test stills show unburnt strips along both long edges.
- **Side oxygen diffusion** helps at low flow and low oxygen (Shih and T'ien, abstracts only). The sign near the limit is unsettled.
- **Fibreglass.** The sheet stays flat, so the no-slip surface and a growing boundary layer persist upstream of the flame (Vetturini on Saffire). Train speeds fall 7–10 % over 1.5 w of travel; Blasius wall shear would fall 17–22 %. Glass adds about 0.1 kJ per g of cotton to the heat load, which is negligible for flame temperature.

## 2. Confrontation with the tracks

### Oxygen relation, no adjusted constant

Ratio of predicted to measured mean speed, using the measured L:

| Phases | Boundary layer only | With diffusion length |
|---|---|---|
| Train, 14 settled or transient windows (2.4–5.2 cm/s) | 0.57–0.82 | 0.91–1.08 |
| Train T5, growing at 10.9 cm/s | 1.12 | 1.18 |
| BASS-I at 9–22 cm/s, both widths (5 phases) | 0.94–1.07 | 1.01–1.09 |
| BASS-I at 5 and 3 cm/s | 1.04, 0.90 | 1.20, 1.21 |
| BASS-I quenching phase, 1 cm/s | 0.77 | 1.69 |

The 20 % excess at BASS-I low flows cannot be separated from the fan-to-flow mapping: BASS-I publishes 5 and 3 cm/s where the BASS-II calibration gives 6.5 and 4.7 for the same display.

### Steady values and the carried forward's laws on BASS-I

| Burn | U (cm/s) | O2 | v / carried law | L / carried L |
|---|---|---|---|---|
| Test 5, 2 cm | 5 | 22.0 % | 0.87 | 0.94 |
| Test 5, 2 cm | 3 | 22.0 % | 0.86 | 0.98 |
| Test 2, 2 cm (transient) | 9 | 22.7 % | 0.93 | 0.63 |
| Test 4, 2 cm (growing) | 22 | 21.6 % | 1.04 | 0.79 |
| Test 10, 1 cm | 11 | 21.6 % | 0.83 | 0.47 |
| Test 11, 1 cm | 20 | 21.5 % | 0.68 | 0.35 |

Closed-form steady speed with t_res = 10 s at the train low flows: 0.0371 and 0.0363 w/s against measured 0.037–0.039 and 0.0376; lengths 0.37 and 0.36 w against 0.37–0.38 and 0.356.

### Transients: both mechanistic closures lost

| Evaluation | Result |
|---|---|
| Closure A (pyrolysis front plus the carried overhang, mass balance), law only from igniter-on | Worse than the carried law path on T5 and T6 (tip error 0.14–0.33 w against 0.05–0.22) and on all five BASS-I burns; better on T16 after 10 s (0.02–0.03 against 0.03–0.07) |
| Closure B (one length, t_res = 10 s), from the first still | Steady states right, transients wrong: tip left 0.17–0.62 w behind on train, where the shifted carried law is within 0.02–0.11 w |

- Closure A fails because an overhang that appears within 2 s of ignition overshoots; closure B because one t_res cannot hold both a 10.9 cm/s growing flame and a near-limit flame.
- The carried law itself overshoots BASS-I ignition: tip at igniter-off 0.98 against 0.79, 1.72 against 0.91, 3.21 against 1.89.
- I stopped at two closures; a third would have been tuning.

### Endings

- T5 (train) dies as its tip reaches the window end.
- BASS-I: the flame is gone 4–12 s after the tip reaches the end (0.44–0.88 of t_res).
- BASS-I test 5 quenches mid-sample within 5 s of fan 42/41.

### Are the BASS-I burns comparable?

- **Same** fabric, duct and holder design. Cabin air only (21.5–22.7 %). Exposed length 8.5 cm per the paper; windows are 4.2–4.3 w against 4.5–4.65 w.
- **Usable for numbers in widths:** test 5 (5 and 3 cm/s phases and the quench), test 4, test 2's first 10 s, tests 10 and 11 (narrow).
- **Not usable:** opposed tests 3 and 9, Nomex test 14, and all step times (inferred from the flame response, so circular).
- **Provisional.** The tracks are still being verified. Their speeds in mm/s sit 12–20 % below the published BASS-I fit for the 2 cm sample (1.24 against 1.56 mm/s at 5 cm/s; 4.37 against 4.96 at 22 cm/s) and 13–15 % either side of it for the 1 cm sample. Flows are AIR-display values.

## 3. Audit of the carried forward

| Tensor | Typed | Verdict |
|---|---|---|
| `speed.b0`, `speed.skip[0]` | 0.0215 + 0.0064·U | Supported. Physics gives 0.029 and 0.0061 at 18.8 %. Mechanism: diffusive plus convected oxygen. |
| `speed.pow[0]` | 1.5 | Wrong form, not wrong sign. Intercept goes as Y_O¹, slope as Y_O²·t_res. BASS-I implies an effective 0.5 at 3–5 cm/s and 1.6 at 22 cm/s. Overstates 21 % low-flow speed by 10–14 %. |
| `speed.pow[1]` | −1 (factor 1.83) | Too large. BASS-I gives 1.25–1.6 in own widths (power −0.4 to −0.8); the published BASS-I fits give 1.0–1.2. |
| `over.b0`, `over.skip[0]` | h = 0.065·U | Effective patch. Steady length within 6 % on the 2 cm sample at 2.4–5 cm/s in both campaigns; 20 % long at 4.7 cm/s and 17.6 %. Not a physical overhang. |
| `over.pow[0]` | 3 | Empirical, from one burn. Not contradicted (BASS-I 0.94, 0.98). Range 2–4. |
| `over.pow[1]` | −1 | **Sign contradicted.** Narrow flames are 0.64–0.86 of the wide law in their own widths; the rule makes them 1.83 times. The power should be about +0.5 (+0.25 to +0.7). |
| `over.tau_s` | 2 s | Right for shrinking, wrong for growth (10–20 s). Cause of the ignition overshoot. |
| `burn.b0` | 6.3 s | Fine as a delay. The physical time rises toward the limit. |
| `end.linger_s` | 5 s | Low-flow value. BASS-I gives 4–12 s, about 0.4–0.9 of t_res. |
| `ign.gain`, `ign.delay_s` | 1, 0 | Ignition takes about 2 s; harmless once a still exists. |
| `quench.W1[0,:]`, `b1[0]`, `w2[0]` | U_q = 2.27 − 1.33·c_o2; 2.5 per cm/s | Three faults, below. |
| `quench.W1[1,1]`, `b1[1]`, `w2[1]` | wall below 17.2 % | One burn; a class question written as a rate. |
| `unc.*` | — | No flow-uncertainty term. One cm/s at fan 42 is 17 % of speed. |

The three faults in the quench velocity unit:
- **Oxygen slope.** It is 0.67 cm/s per % O2. The heat balance gives about 0.07, and the published tissue model 0.15–0.25 between 17 and 21 %.
- **Placement.** The line was put through flames that survived 90 s, which only shows they were above it.
- **Sensitivity to flow.** At fan 42 the calibration reads 0 to 3.75 cm/s across 11 days (median 1), and the line to stall gives 0.9–2.7. A slope of 2.5 per cm/s turns that into a factor of about 40 in hazard. This is why it did not carry to another day.

Missing altogether: slow growth of the length, tip faster than base at high flow, the downstream slow-down, and flow uncertainty.

## 4. Blueprint

### State and equations

Keep the carried skeleton and change four things. Explicit Euler at 0.25 s; the closed form needs one square root.

- **State:** front x, fast length part h, base as x delayed by t_b, a class c, a run offset E.
- **Speed:** `dx/dt = E · v_ss(U_eff, X, W)` from the closed form of 1.2. The typed law is t_res(U, X, W), not v.
- **Length:** h relaxes to its steady value with τ_dn when above it and τ_up when below.
- **Tip while growing:** tip = x + h. While h is below its steady value the tip moves at v plus dh/dt, about 1.25 times the oxygen speed in four growth phases.
- **Effective flow:** `U_eff = U_fan · (1 − κ·(x − x_ref))`.

### Parameters

| Group | Parameter | Proposed | Uncertainty |
|---|---|---|---|
| Oxygen delivery | ρ∞, r, m_F, ν*, α* | 1.15 kg/m³, 1.185, 67.7 g/m², 1.49e-4, 2.13e-4 m²/s | ν*, α* ±30 % (reference temperature) |
| Residence | t_res, near limit (U ≤ 3 cm/s, X ≤ 19 %) | 10 s | ±1 |
| | t_res, wide, robust flame | 14 s | ±2; the oxygen and flow split is not identified |
| | t_res, narrow at 21 % | 8.7 s | ±1; unknown at low oxygen |
| Length | steady length, wide | 0.135 + 0.105·U w at 19 % | ±10 % at 5 cm/s or less; unknown above 10 |
| | oxygen power | 3 | 2–4 |
| | width power, own widths | +0.5 | +0.25 to +0.7 |
| | narrow saturation | 1.5 w at 11 cm/s, 1.9 w at 20 | two burns |
| Times | τ_dn | 2 s | 1–3 |
| | τ_up | 10 s | 6–20 |
| | t_b | 6.3 s | 5–10 |
| Ignition | delay; first burn-through after igniter-on | 2 s; 10 s | ±1; 8.8–13.2 |
| | length at igniter-off, short and long igniter | 0.6 and 1.3 of steady | ±0.25 |
| Position | κ | 0.05 per w | 0.03–0.12 |
| End | linger | 0.6·t_res | 0.4–0.9 |
| Flow | σ_U below fan 47; scale above | 1 cm/s; ±15 % | from 23 calibration days |
| Limit | radiative floor U_q(X) | 1.5–1.9 cm/s | factor 1.5 |

### What the first seconds identify

- The class: any flame still after igniter-off.
- L directly, as tip minus base in every still.
- From L and U, the oxygen relation gives the mean speed without waiting for displacement. That targets the 5 s cutoff, where the error is largest.
- Igniter duration tells whether the length will shrink or grow.
- Whether the base has taken off.

### The class

Two states, not a hazard: carries or does not.

- **Prior.** A logistic in oxygen centred near 16.8 % with a width of about 0.7 %, from two train burns. Floor it at the aggregate you already hold (about 2 of 16 validation burns did not spread).
- **Update.** One flame still after igniter-off sets the class to "carries".
- **After that,** extinction is a separate, bounded process: only when U_eff is below U_q, taking one or two t_res, and with a longer tail downstream.

### Uncertainty

Carry it as a small set of explicit paths rather than one Gaussian: flow at three values (U, U ± σ_U), two t_res regimes, and E. Mix the quantiles.

### Keep from the carried forward

The band picture, the delayed base, the typed speed line at 17.6–19 %, the 2 s shrink time, the prefix gain logic, and the end logic.

## 5. Conditions the train burns do not hold (stated before any outcome)

| Condition | Statement | Confidence |
|---|---|---|
| Flow above 11 cm/s, wide | Length never becomes steady on 10 cm. Tip runs 1.3–1.6 times the base. The carried law lags the tip. Tip reaches the end within 10–25 s; flame gone about 10 s later. No blow-off below 44 cm/s at 21 % or more. | High at 21 %; medium at 17–19 %, where blow-off at high flow is possible |
| 21 % oxygen | Speed about 0.054 w/s at 5 cm/s and 0.19 at 22. The carried law is 10–14 % fast at low flow, right at high flow. Lengths 0.6 and 0.85 w at 3 and 5 cm/s. t_res near 14 s. Quench near 1–1.5 cm/s. | Medium–high |
| 16–17 % oxygen | About even odds of carrying a flame. If it does: L of 0.30–0.36 w, 0.030–0.036 w/s at 3–5 cm/s, t_res of 9–10 s, more mid-sample endings. | Low on the class; medium on the kinematics |
| Narrow sample | Own-width speed 1.25–1.6 times the wide law at 21 %. Length 0.65–0.85 of the wide law. Steady within 20 s even at 10–20 cm/s. Window about 8 w. Near the limit, weaker than the wide sample. | Medium at 21 %; low at low oxygen |

## 6. What cannot be predicted in principle

| Not predictable from these inputs | Signal that would change it |
|---|---|
| True flow below fan 47 (0–3.75 cm/s at fan 42 across days) | A fan 42 calibration row for every day; the AIR display, deconvolved for its one-minute lag |
| The class within about 1 % of the oxygen nose; sample reuse (T10 is sample "134M") | Sample history; the post-test char still |
| Whether and when a near-limit flame dies | Trend of `area_px` and `blue_sum` (already in the track files); flame width across the sample; the RAD radiometer, which is proportional to m_F·W·v·Q |
| The visible tip as an isophote under auto-exposure | Exposure settings per still |
| Oxygen to better than ±0.3 % | — |
| Where the stills stop | — |

## 7. Review aid: claims to try hardest to break

| Claim | How to re-check |
|---|---|
| The oxygen relation holds within 0.91–1.09 with no adjusted constant | `python runs/dev/research6/bass/physics_eval.py`, sections 1 and 4. Check that 1250 K gas values and m_F are the only inputs. Try whether ν* at another reference temperature breaks it. |
| The typed speed line is reproduced by the closed form (slope 0.0061, intercept 0.029) | Hand-evaluate the formula in 1.2. `python runs/dev/research6/bass/blueprint_eval_b.py` prints the steady-state table. |
| The narrow-sample length power has the wrong sign | `tables.md` section 4, rows S131_T10 and S222_T11. Depends on `mm_per_w_estimate` in the BASS-I tracks and their pending verification. |
| Growth is slow, shrink is fast | `tables.md` sections 3 and 5; `python runs/dev/research6/bass/blueprint_eval.py` for the ignition overshoot. |
| The flow near stall is unknown to about 1 cm/s | `python runs/dev/research6/bass/fan_cal_days.py`. Its parser misses the last four days. |
| t_res is 10 s near the limit and 14 s for robust wide flames | Confounded between campaign, oxygen and flow. Only train low-flow and BASS-I test 5 support it. |
| The downstream slow-down is a boundary-layer effect | `tables.md` section 2. It could equally be oxygen depletion or fan drift. |

Run all scripts from the repository root.

## Sources

| Source | How much read | What I took |
|---|---|---|
| Ferkul, NASA CR-191111 (scanned; pages rendered locally) | contents (2 pp), nomenclature (3 pp), pp. 56, 93, 94, 95, 97, 102 | property values, Fig. 27 length scales, quench and blow-off mechanism |
| Ferkul, Olson, Johnston, T'ien, 2013 US National Combustion Meeting (BASS-I) | full, from the text extract in `runs/dev/research5/bass1/work/` | apparatus, 8.5 cm sample, ignition in 2 s, growth statements, Table I |
| Vetturini et al., Fire Technology (accepted manuscript) | abstract, introduction, sections 3.1 (part), 3.2.1, 3.2.4, 3.3, part of 3.4 | scaling eqs. 1–5, boundary-layer note on Saffire. No BASS-II data in what I read. |
| `docs/wsp/research/lit_flame_spread.md` | full | secondary source for its refs 1–21 |
| `docs/wsp/research/exp_bass.md` | sections A–E, F2, F7, G, H | apparatus, cameras, BASS-I poster fits |
| Repo documents | `front.rs` lines 1–330, `edits.wsp`, `cards/bass_front.md`, `EXPERIMENTAL_SCIENCE.md` 3, 5.3, 6.3–6.5, `bass_train_tracks.md` (full), `bass_train_measurement.md` (summary, sections 2, 4, 5) | the carried laws and train tables |
| PSI-25 fan calibration workbook | whole sheet | day-to-day flow spread |
| BASS-I files, read 12:35–12:55 on 14 Nov | `results.json`, `work/results_table.txt`, `quench_table.json` in full; `tracks/*.json` loaded by script; `inventory.json` not opened | phases, first 10 s, endings. Unverified. |
| One web search | — | nothing used |

## Files written

- `runs\dev\research6\bass\physics_eval.py`
- `runs\dev\research6\bass\tables.md`
- `runs\dev\research6\bass\fan_cal_days.py`
- `runs\dev\research6\bass\fan_calibration_days.md`
- `runs\dev\research6\bass\blueprint_eval.py`
- `runs\dev\research6\bass\blueprint_eval.md`
- `runs\dev\research6\bass\blueprint_eval_b.py`
- `runs\dev\research6\bass\blueprint_eval_b.md`

No git-tracked file was modified. Downloaded PDFs and page renders are in the session scratchpad only.

## Statement on protected material

- **PSI-25.** I opened only the four train track files (T5, T6, T16, T10), the two train notes, and the dataset-level fan calibration workbook. I did not open the ReadMe, any other test, `val_sealed`, `data_ext`, or any `holdout` folder, and I ran no `dev_eval` or `front-feedback`.
- **One exposure to disclose.** Before the coordinator's caution arrived I had read `exp_bass.md` section F2 and the matching rows of its table H. These carry published BASS-II concurrent SIBAL results: two quench-boundary points, two blow-off points, a non-ignition oxygen range, and burn and steady durations at two flows. None of them is used as evidence for any number here. Section 5's 16–17 % statement rests on the two train burns and the aggregate in the task brief; a reviewer should still treat that row as possibly anchored.
- **Otherwise** I read no measured per-test BASS-II concurrent SIBAL result in any paper, table or figure.
