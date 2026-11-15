# ACME burner flames, growth clock (`acme_normal.wsp3`, `acme_inverse.wsp3`)

**State: experimental, hand-written, data-informed. Inverse flames: carried. Normal flames: a
member of the carried blend.** Forward `flame3.rs`. The second generation
([card](acme_clock.md)) wrote the life of a flame as hinge laws in the conditions, typed from seven
train sessions; most of that skill did not carry to unseen sessions. This generation writes three
mechanisms as the physics they are, so that what was typed is a small number of physical constants
and the corrections the train table asks for. The second-generation forward, releases and ids are
unchanged.

| Model | Candidate | Release | Non-zero numbers |
|---|---|---|---|
| `acme_normal.wsp3` | **D1** | `sha256:221a2ea22aca8a8c1e3813745ea566e591863fca13d8dd456c69c38986d49027` | 55 / 499 |
| | published theory alone | `sha256:3d8d3b7a6e5cfbb035060711fa0cdaf392755936b42da55d38c6ac5df46115ad` | reference: no number from the PSI-159 tables |
| `acme_inverse.wsp3` | **D1** | `sha256:19c156e60036d15f46fa92f7827d1122f8e595f6747ad1d5b7363463777422eb` | 54 / 499 |
| | published theory alone | `sha256:d0a1366034b9f10aa91633c670071bb8029bcf7d9c945608f2cd1d8f5a8b184b` | reference |

## The three mechanisms

**1. Radiative self-extinction: the growth clock.** A burner-supported spherical flame grows towards
a quasi-steady radius and goes out when the reactant flux at the flame has fallen to a critical
value. If the quasi-steady radius is inside the extinction radius, the flame is held.

```text
r_ss  = m_tot / (4 π ρD L)          L = ln(1 / (1 − Z_st)) normal,  ln(1 / Z_st) inverse
r_ext = sqrt(m_R / (4 π m_cr))      m_R the burner-side reactant flow (fuel, or the O2 of the injected stream)
Φ     = (r_ss / r_ext)²             Φ ≤ 1: held
R(Φ)  = r_ext² / (4 D erfcinv(Φ^−½)²)    time for the growing flame to reach r_ext
```

The reduced flow Φ is not known exactly, and near Φ = 1 it decides between two minutes and never.
So the clock is integrated over a distribution of Φ (15 equally likely values of ln Φ, spread 0.25):
the forecast widens where the physics says the outcome is undecided, not everywhere.

**2. A second clock** for flames that end some other way: the lean inverse flames, taken by a quench
hole at the supply tube within seconds. Empirical, in the conditions.

**3. The burner limit as a heating law.** The burner warms as `T0 + ΔT (1 − exp(−t / 30 s))` with
`ΔT = 2012 K · (m_tot / 1 mg/s)^−0.4 · (r_ss / 1 mm)^−0.4`, and the test is stopped at a limit that was
455 °C in some sessions and 510 °C in others. The law was written from the peak burner temperatures
of the train flames, a post-run fact that is never an input. It is one law for normal and inverse
flames.

Plus what the second generation already had: a weak class that goes out at ignition, programmed
stops, a background rate.

## Normal flames, D1

| Number | Value | From |
|---|---|---|
| ρD | 0.0334 g m⁻¹ s⁻¹ | published (fitted to flame radii of ISS butane flames) |
| Critical fuel flux | 0.18 g m⁻² s⁻¹ | published 0.2 (ethylene); 10 % lower puts the ethane flames near the holding flow at 144–185 and 261 s (train: 158–232, 304 s) |
| D of the growth clock | 50 mm²/s at 30 % O2, 1 atm | published 28 (air properties); the train lives fix the scale |
| Pressure | clock ∝ p^−0.45 | 59, 43, 34 s at 0.5, 1, 2 atm; 88, 71, 51 s. The simple theory (D ∝ 1/p) has the opposite sign |
| Oxygen | clock twice as fast in air as at 39 % | 9–10 s in air where the law says 18–20 s; no train flame between 22 and 33 % |
| Dilution | +0.2 per e-fold of fuel fraction | three sessions with a range; half of what two of them ask for |
| Butane, enriched O2 | clock twice as fast at 39 % | 0.5–0.7 of the ethane clock in two sessions (soot); in air on it |
| Propane, enriched O2 | half of butane's term | no train test: by carbon number |
| Air above 1 atm | critical flux +0.5 per e-fold of pressure | propane at 2–3 atm goes out at 5–7 s (seven tests, one session): half of what it asks for |
| Weak class | logistic at 1740 K, 45 K wide | 1 s at 1602–1701 K (ten tests), 8–35 s at 1785–1795 K |
| Heating: undiluted fuel | 15–20 % less than the law | three flames of one session |

With the published constants alone the law already places the holding flow (0.65 mg/s for 30 %
ethane in 39 % oxygen; train: held at 0.75, out after 304 s at 0.85) and makes the life in air
independent of flow. Session medians of ln(observed / law) for the typed law: −0.32 to +0.06 over
seven sessions.

## Inverse flames, D1

| Number | Value | From |
|---|---|---|
| Growth clock | off | with the published inverse constants it gives air-injected flames 30–60 s; four were still burning at the 65–66 s stop. It also holds the lean flames, which last seconds |
| Second clock | the self-extinction law of the second generation's D1 | 5–7 s at 7 % ambient fuel, 8–18 s at 8–9 %, 46–51 s at 14 %, none at 17 % and above |
| Weak class | low adiabatic temperature and a lean ambient together | air into 7–9 % fuel at 1835–1879 K flashed in three of five tests; 15–21 % O2 into 20–30 % ethane at 1785–2120 K never did |
| Heating | the normal-flame law, ethane 0.3 above it | butane and propane sessions within 0.15; the two ethane sessions 0.3–0.5 above |
| 66 s stop | one window from 63 to 67 s, three in four flames | four of five sessions with a flame alive at 60 s |
| Other stops | a background rate, one per 250 flame-seconds | nine stops with no script time behind them, in five sessions |

The stored flow of an inverse flame is the oxygen alone, not the injected stream; the total burner
flow of an air-injected flame is 4.3 times that. A lean flame cannot reach the burner limit under the
heating law, which the second generation had to be told.

## Results

The candidates were committed (`f1f7a30`) before any validation score of them existed. The rule,
stated then: each replaces the carried candidate unless its validation extinction score is worse.

| Train, integrated Brier 6–120 s | Extinction | Flow termination |
|---|---|---|
| `acme_normal.wsp3` D1 | 0.063 | 0.024 |
| `acme_normal.wsp2` D3 (second generation, carried) | 0.052 | 0.041 |
| `acme_normal.wsp3` published theory alone | 0.141 | 0.075 |
| first-generation zero-shot / best fitted | 0.159 / 0.108 | 0.065 / 0.039 |
| `acme_inverse.wsp3` D1 | 0.036 | 0.064 |
| `acme_inverse.wsp2` D1 (second generation, carried) | 0.057 | 0.067 |
| `acme_inverse.wsp3` published theory alone | 0.488 | 0.247 |

On train the normal D1 is behind the hinge law on extinction and far ahead on flow termination; the
published theory alone, with nothing typed from the campaign, is ahead of the zero-shot model of
phase 1. For inverse flames the published theory is worse than a constant guess: it does not
describe them, and that row is the measurement of it.

| Validation, integrated Brier 6–120 s | Extinction | Flow termination | Decision |
|---|---|---|---|
| `acme_normal.wsp3` D1 | 0.163 | 0.100 | not adopted: the carried hinge law scores 0.159, 0.100 |
| `acme_normal.wsp3` published theory alone | 0.206 | 0.114 | reference; a constant scores 0.214, the zero-shot model of phase 1 0.219 |
| `acme_inverse.wsp3` D1 | **0.137** | **0.088** | **adopted**: the second generation scores 0.146, 0.097, the best fitted model 0.143, 0.118 |
| `acme_inverse.wsp3` published theory alone | 0.346 | 0.193 | reference; a constant scores 0.252 |

For normal flames the growth law ties the hinge law on unseen sessions while being behind it on
train: a less fitted law lost nothing where it mattered. The two together are
[`acme_normal.wsp4`](acme_blend.md), which scores 0.155 and is the carried normal-flame model.
For inverse flames this forward is the carried model.

## Limits

- **Not modelled:** 3 % butane (one session: the flame is held where the law takes it, and heats the
  burner four times more than the heating law says); the two butane flames at 2 bar that cycle
  between hot and cool and are labelled flow terminations; the burner's temperature at ignition,
  which carries over from the test before.
- **The score window is 6–120 s.** A flame that goes out at 123 s and one that is held for five
  minutes are the same case in it; the growth law differs most from a hazard surface beyond it.
- **Stops are campaign policy.** A session's script duration is not in the prefix; the atoms are the
  frequencies with which train sessions used each time.
- The corrections to the simple theory (pressure, oxygen, dilution, fuel) are typed from five to
  seven sessions; no train flame lies between 22 and 33 % oxygen.
- The published constants were measured on flames of the same ISS programme (other fuels, other
  campaigns); the theory baseline is "no number from the PSI-159 tables", not "no ISS data".
