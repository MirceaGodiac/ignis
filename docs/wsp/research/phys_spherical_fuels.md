# Fuel dependence of the spherical burner flames, rebuilt from properties

> Written by an author model (claude-fable-5-2) on 15 November 2026. Nothing fitted, no score computed, no
> model or weight changed. It is a proposal: no number here is a weight until it has been typed into an
> edits file and assembled. Read for it: the cards, forwards, edits files and dossiers named in section 6;
> the 81 normal and 73 inverse **train** rows of CFI-G through `tools/wsp/analysis/lib.py` (two recounts,
> marked "train recount"); the aggregate tables of the Flame Design transfer test as PROTOCOL 11.3 and its
> report give them. Not read: any per-test outcome of Flame Design or s-Flame, anything under
> `runs/dev/data_all/`, any validation or test outcome of CFI-G, `holdout/`, `runs/dev/holdout_ext/`.
> Origin classes: **P** property table, **T** theory, **L** published experiment, **J** judgement,
> **D** carried from numbers typed after CFI-G train data. Exposures are listed in section 6.

## 0. Main results

- **The two indicator directions are one direction.** Ethane, propane and n-butane are a homologous series:
  molar mass, sooting tendency and Lewis number all rise together with carbon number, and in every adopted
  law propane was typed at the midpoint (or was never tested). The models hold one fuel axis, "heavier
  alkane", and each law uses it for a different property. Ethylene (light, sooty, reactive) and methane
  (light, clean) leave that axis in different directions, so a single pair (x, y) per fuel cannot be right
  in all laws at once.
- **What each indicator weight stands for** (section 1.4): in the hinge member's burner clock, molar mass
  (the typed +0.54 / −0.56 are what the growth member's heating law predicts from molar mass alone at 30 %
  fuel: 1.10 typed against 1.09 computed). In the radiative clocks, something that makes butane flames go
  out sooner in enriched oxygen and that neither stoichiometry nor gas radiation explains; soot is the only
  candidate with the right sign. In the inverse heating law, either the Lewis number or pressure: the train
  rows cannot tell which.
- **One thing no indicator carries**: the oxygen a gram of fuel demands. The critical fuel flux typed for
  ethane from train (0.18 g m⁻² s⁻¹) and the one published for ethylene (0.20) are the same number in
  oxygen units: 0.670 and 0.684 g O2 m⁻² s⁻¹. That is the third direction, and it costs one property
  number per fuel.
- **Smallest change** (section 3): a typed fuel table with a molar mass, an oxygen-demand shift, and
  coordinates in the two existing directions given separately for the self-extinction laws and for the
  burner laws; the forward blends each law's output at the three CFI-G fuels with those coordinates. With
  ethane (1, 0), propane (0, 0), butane (0, 1) every CFI-G forecast is, by construction, bit for bit
  what it is now (to be shown by the acceptance test of 3.3; nothing was built or run for this file).
- **Typed for ethylene**: self-extinction (0.5, 0.5), burner (1, 0), oxygen-demand shift −0.085.
  **For methane**: self-extinction (1, 0), burner (1.9, −0.9), shift +0.068. Inverse flames (ethylene only):
  burner (0.5, 0) and a shift of the quench clock of +0.7 in ln.
- **Two directions are not enough** for three reasons: coordinates must be per law group; normal flames need
  the oxygen-demand direction; inverse flames need a direction the adopted inverse laws do not contain at
  all (how a lean flame of a low-Lewis-number fuel ends). Hydrogen and helium-diluted tests need more than
  that and should be refused.

## 1. Why the fuel matters, mechanism by mechanism

### 1.1 What the conditions already carry

A fuel enters a flame-sheet model through its stoichiometry and heat release. Both are already inputs:
`zst` and `tad` are computed per test from the fuel's own stoichiometric ratio and heat of combustion, so
`c_zst` and `c_tad` are fuel-aware for any fuel. What remains for a fuel descriptor is what the conditions do
not hold:

| Property | Where it should act | Carried today by |
|---|---|---|
| Molar mass W | mass fraction of the burner gas at a given mole fraction, so total burner flow and flame radius | the indicators choosing one of three molar masses (growth forward); an indicator weight (hinge burner clock) |
| Oxygen demand per gram, σ | the flow feature: a fuel *mass* flow means a different oxygen demand and heat release per fuel | nothing (the three alkanes differ by 4 %) |
| Radiative loss beyond CO2 and H2O (soot, fuel bands) | how fast the flame cools on its way to the critical flux | the butane and propane hinge units |
| Fuel diffusivity (Lewis number) | inverse flames only: radius, flame temperature, pulsation | one weight (ethane heating); otherwise nothing |
| Chemistry (critical temperature, kinetic limit) | critical flux, weak-flame class | nothing (the three alkanes are alike) |

### 1.2 Normal flames

**Flame position and temperature do not depend on the fuel's diffusivity (T).** Outside the flame, oxygen
diffuses inward against the burner flow; solving that layer alone gives
`r_ss = m_tot / (4π ρD_O2 ln(1/(1 − Z_st)))`. The fuel's diffusivity does not appear, and with unit Lewis
number on the oxygen side the sheet temperature is the adiabatic one whatever the fuel's Lewis number. So for
normal flames a heavier fuel is not a "slower" fuel. This is why the published theory (and the first
dossier) predicts butane *slightly longer* than ethane.

**Reduced flow and the oxygen-demand direction (T, checked against L and D).** With
`Φ = (r_ss / r_ext)² = m_F m″_cr / (4π (ρD)² (Y_F L)²)` and `Y_F L = Y_F ln(1 + a/Y_F)`, `a = Y_O/σ`:

- For a concentrated fuel `Y_F L → Y_O/σ`, so `Φ ∝ m_F m″_cr σ²`.
- The energy balance of the first dossier gives `m″_cr = ρD Y_F L / (Θ ℓ*)`, hence `m″_cr ∝ 1/σ`: the
  critical flux is a fixed *oxygen-equivalent* flux. Check: ethylene 0.20 × 3.422 = 0.684 (L, Irace et al.
  2023, normal flames) and the published inverse value 0.68 g O2 m⁻² s⁻¹ (L, same paper); ethane, typed from
  train, 0.18 × 3.725 = 0.670 (D). Three numbers from two configurations and two fuels within 2 %. The
  "10 % below the published value" that the growth candidate typed for ethane is the stoichiometric ratio
  of the two fuels (0.918).
- Together: `Φ ∝ m_F σ`. At the same fuel mass flow ethylene has 8.5 % less reduced flow than ethane,
  methane 7 % more, hydrogen 2.1 times more. Propane and butane are 2.6 % and 4.0 % below ethane; the
  models were typed without it and absorbed it.

**Dilution is geometry, not a butane property (T).** `d ln Φ / d ln Y_F = −2 [1 − (a/Y_F) / ((1 + a/Y_F) ln(1 + a/Y_F))]`:
−0.21 at Y_F = 0.47 (30 % butane), −0.36 at 0.24, −0.56 at 0.12 (39 % O2). Any fuel burns shorter, near its
holding flow, when its burner gas is leaner *by mass*. At one mole fraction a light fuel is the leaner one:
30 % by mole is Y_F = 0.47 for butane, 0.32 for ethane, 0.30 for ethylene, 0.20 for methane.

**Radiation (P, T).** Gas-band emission per unit heat release hardly depends on the fuel. Moles of CO2 and
H2O per MJ, weighted with the Planck-mean coefficients at 1200 K (20.1 and 3.9 m⁻¹ atm⁻¹), relative to
ethane: methane 0.96, ethylene 1.00, propane 1.02, butane 1.04, hydrogen 0.44. A 4 % difference cannot make
butane flames last 0.65 to 0.80 of ethane's. What differs strongly between these fuels is soot (section 2)
and, for ethylene, reactivity.

**Burner heating (T, D).** The growth member's law, `ΔT ∝ m_tot^−0.4 r_ss^−0.4`, needs no fuel term once
`m_tot = m_F / Y_F` uses the fuel's molar mass: a heavy fuel at 30 % by mole brings less nitrogen through
the burner per gram of fuel, so the burner is cooled less and the flame sits no further out.

### 1.3 Inverse flames

Here the fuel is the ambient gas and must diffuse inward; its diffusivity matters three times (T, the second
dossier): the radius is `Le_F` times the one the code computes; the steady sheet temperature is
`T_amb + (T_ad − T_amb)(1 − Z_st^(1/Le_F)) / (1 − Z_st)`; and pulsation near extinction needs a large Lewis
number of the reactant diffusing against the stream. For 10 % ambient fuel and 85 % injected oxygen that
fraction of the adiabatic rise is 0.55 for butane, 0.62 for propane, 0.76 for ethane, 0.81 for ethylene.
Train agrees in kind: every propane and butane flame that went out pulsed first (33 of 34), and none of the
18 ethane flames pulsed. But ethane was flown only in rich ambients (20 to 30 %), where the adopted law holds
any fuel, so the adopted inverse model has never been told that a lean ethane flame differs from a lean
propane flame, and it has no weight that could say so.

### 1.4 Every fuel-indicator weight in the adopted releases

| # | Weight | What it says | Property it should follow | Sign | Size |
|---|---|---|---|---|---|
| 1 | a0 `se.tau` unit 4: `W1[4,7] = 5`, `b1 = −4.1`, `w2 = −0.25` | butane: ln life −0.25 per 10 % O2 above air (×0.64 at 39 %) | soot or interior radiation; not stoichiometry (which gives butane 15 % *less* reduced flow at 30 % fuel), not gas bands (+4 %) | agrees only if it is soot | at the strong edge of train: the independent recount found 0.65 to 0.80 in five matched pairs |
| 2 | b0 `se.diff` unit 0: `W1[0,7] = 5`, `w2 = 0.36` | butane: growth clock ×1.9 at 39 % O2 (life ×0.52 far from the holding flow) | the same; a far-field oxygen diffusivity cannot depend on the fuel | as 1 | **disagrees**: 0.52 against the recounted 0.65 to 0.80 |
| 3 | a0 `se.tau` unit 5 (−0.12), b0 `se.diff` unit 1 (+0.18): propane, gated by both indicators | propane at half of butane, "by carbon number"; no train test | soot index puts propane 0.70 of the way from ethane to butane (0.45 to 1.0, section 2) | agrees | agrees within the property's own uncertainty; low rather than high |
| 4 | a0 `se.tau` units 2, 3: `W1[2,7] = W1[3,7] = 5`, `w2 = ∓0.35` | butane only: −0.35 per e-fold of fuel fraction between 30 % and 12 %, flat below | not a fuel property: the `Y_F L` geometry of 1.2, which holds for every fuel and steepens as the gas gets leaner | agrees for butane | **disagrees in scope and shape**: none for ethane, where theory gives the same; levels off where theory steepens. Train: one session of three shows it |
| 5 | a0 `ft.tau.skip[6] = +0.54`, `skip[7] = −0.56` | burner limit later for ethane, sooner for butane | molar mass, through total burner flow and flame radius. From the heating law at 30 % fuel, 39 % O2, relative to propane: ethane +0.68, butane −0.41 | agrees | sum 1.10 typed, 1.09 computed. **But it is a property of the mixture**: computed sum 0.09 for undiluted fuel, 0.73 at 50 %, 1.47 at 15 % |
| 6 | b0: indicators choose `phys.mw_*` | molar mass | molar mass | — | exact (P) |
| 7 | inverse b0 to b3 `bt.rise.skip[5] = +0.3` | ethane heats the burner 35 % more than the law | Lewis number: radius ∝ Le_F gives +0.10 against propane, +0.17 against butane | agrees | **three times the property**; and confounded, see below |
| 8 | inverse `se.osc`: no indicator weight | a lean flame of any fuel ends as a propane or butane flame does | Lewis number (pulsation), critical flux | — | **missing**; untested for ethane |
| 9 | normal `se.flux.b0 = ln 0.18` (no indicator, but typed on ethane) | critical fuel flux | oxygen-equivalent flux 0.68 g O2 m⁻² s⁻¹ | agrees | agrees with ethylene's published value to 2 % once σ is counted |

### 1.5 Findings

1. **Weight 5 is molar mass and nothing else.** Its size is reproduced from 30.07, 44.10 and 58.12 g/mol by
   the other member's law. It is right at the composition the campaign flew most (30 to 50 % fuel) and ten
   times too large for undiluted fuel. Train shows the direction: undiluted ethane at 0.5 mg/s reached the
   limit at 28 s, 30 % ethane at 49 and 56 s (three pressures, so a direction, not a size). The forward of `acme_*.wsp8` (`flame4.rs`), which gives the
   hinge laws the heating law as their burner clock, removes the need for this weight.
2. **Weights 1 and 2 disagree with each other and one of them with train.** The same butane effect is 0.64 in
   one member and 0.52 in the other; the recount supports 0.65 to 0.80. No property predicts either size.
3. **Weight 4 gives butane a law that belongs to every fuel.** The hinge member therefore has no dilution
   dependence for any other fuel. For a light fuel in nitrogen this is its largest error (section 4, methane).
4. **Weight 7 may not be a fuel term (train recount).** Rise implied by the peak burner temperature and the
   burn time, against the typed law, flames that burned 8 s or more: ethane at 1.1 atm, 4 flames, +0.31,
   +0.10, −0.24, −0.26; ethane at 0.5 atm, 14 flames, median +0.40; propane at 0.54 atm, 2 flames, +0.38 and
   +1.68; butane sessions −0.10 and +0.10; lean propane −0.10 and −0.26. Every flame at half an atmosphere
   is above the law, whatever its fuel; ethane at one atmosphere is on it. Peaks cut by a limit trip are
   lower bounds, so this is a count, not a measurement.
5. **Fractional indicators do not interpolate.** The hinge units gate on `5 × indicator`: a fuel fed in as
   (0.5, 0.5) would lose its fuel terms instead of getting half of them (unit 4 acts only above 46 % oxygen,
   unit 2 only below 2.5 % fuel, units 3 and 5 nowhere in the domain), and `flame3.rs` refuses any
   indicator that is not 0 or 1. Coordinates have to weigh the laws' outputs, not replace their inputs.
6. **The surrogate run of PROTOCOL 11.3 was wrong in a known direction.** With ethane's molar mass and the
   ethane flux, the growth member gave ethylene 9 % (undiluted) to 23 % (13 % fuel) too little reduced flow,
   that is, too many held flames and too long lives near the holding flow; and 4 to 5 % too much burner
   heating for diluted fuel.

## 2. A property-based fuel description

Four descriptors. Values for the fuels of the three investigations: ethane, propane, n-butane (CFI-G);
ethylene, propane (Flame Design); hydrogen, methane, hydrogen/methane blends, ethylene (s-Flame).

| Fuel | W, g/mol (P) | σ, g O2 per g fuel (T, stoichiometry) | Le_F in N2, 300 K (P) | Soot index 1000 / L_SP, mm⁻¹ (P) |
|---|---|---|---|---|
| Methane | 16.043 | 3.989 | 0.92 | no smoke point in the compilation; taken as ≤ ethane (J) |
| Ethane | 30.07 | 3.725 | 1.39 | 3.1 (2.1 to 6.0) |
| Ethylene | 28.054 | 3.422 | 1.28 | 8.3 (7.2 to 9.8) |
| Propane | 44.10 | 3.628 | 1.78 | 5.0 (3.5 to 8.3) |
| n-Butane | 58.12 | 3.579 | 2.10 | 5.7 (4.8 to 7.0) |
| Hydrogen | 2.016 | 7.936 | 0.29 | 0 (T: no carbon) |
| Hydrogen + methane | by mixture | by mixture | two species | between; not typed, refused |

Sources and what was checked:

- **W**: standard molar masses; the three alkanes as in `phys.mw_*` of the growth spec.
- **σ**: moles of O2 per mole of fuel (2, 3.5, 3, 5, 6.5, 0.5) × 31.999 / W. The alkane values are those of
  `properties_verified.md`.
- **Le_F**: the alkane values are the repository's (McGivern and Manion diffusivities at 300 K, thermal
  diffusivity of N2 0.222 cm²/s; verified there within 3 to 5 %). Ethylene and methane are scaled from
  ethane with the recommended coefficients of NIST Technical Note 2279 at 298 K, read by me from its table:
  ethane–air 0.150, ethylene–air 0.163, ethane–N2 0.154, methane–N2 0.233 cm²/s. Range for ethylene 1.25 to
  1.35, for methane 0.90 to 1.03 (Marrero and Mason give 0.216 for methane, from memory). Hydrogen from
  D ≈ 0.78 cm²/s, from memory, not re-verified.
- **Soot index**: normalised laminar smoke points of Li and Sunderland, Combust. Sci. Technol. 184 (2012),
  read by me from the paper's table: ethane 319 ± 153, propane 202 ± 82, n-butane 175 ± 33, ethene 120 ± 18
  mm. One over the smoke point is sooting per unit heat release, the basis on which these flames are
  compared. Position on the line from ethane (0) to butane (1): propane 0.70, ethylene 2.0; in logarithms
  0.76 and 1.63. Ethane's own uncertainty moves these by a factor of two.
- **A second ranking (L)**: published microgravity smoke-point work orders the fuels methane < ethane <
  ethylene < propane < propylene, with ethylene and propane exchanged against normal gravity. Only the
  ranking is used here; see the exposure note in section 6.

Used in checks, not as descriptors (P): lower heating values 50.01, 47.51, 47.16, 46.33, 45.72, 119.94 kJ/g
(methane, ethane, ethylene, propane, butane, hydrogen); heat per gram of oxygen 12.54, 12.76, 13.78, 12.77,
12.78, 15.11 kJ. For ethylene the oxygen rule and a heat rule for the critical flux differ by 8 %; between
ethane (D) and ethylene (L) the oxygen rule fits to 2 % and the heat rule to 9 %, and the typed 0.18 is not
known to better than 10 %.

No descriptor is typed for chemistry. Ethane, propane and butane burn alike (laminar flame speeds within
10 %), so the weights hold no sensitivity to it; ethylene is markedly more reactive and methane slightly
less. The published critical temperature (1130 K) is ethylene's own. This is an open direction with an
unknown coefficient, not a fourth typed number.

## 3. The smallest change

### 3.1 What it is

A new strict forward for each member law (the laws of `flame.rs` and of `flame3.rs`, unchanged) that reads
the fuel from a typed table instead of two indicators, under the unchanged committee of `committee.rs`. For
each law network it evaluates the network at the CFI-G fuels and weighs the **outputs** with the fuel's
coordinates; it takes the molar mass from the table; and it applies one shift for the oxygen demand and one
for the inverse quench clock. Nothing else is touched.

Rows of the table are fixed by the spec as fuel names (not numbers): 0 ethane, 1 propane, 2 butane,
3 ethylene, 4 methane, 5 hydrogen. Normal and inverse releases have separate tables, as they have separate
everything.

### 3.2 Tensors

| Name | Shape | Bounds | Unit | Meaning |
|---|---|---|---|---|
| `fuel.admit` | [6] | 0 or 1 | 1 | 1: the release answers for this fuel; 0: refusal at the condition gate |
| `fuel.mw_g_mol` | [6] | (1, 200) | g/mol | molar mass; replaces `phys.mw_ethane`, `phys.mw_propane`, `phys.mw_butane` |
| `fuel.s` | [6] | (−1, 1) | 1 | ln of the oxygen demand per gram relative to ethane, ln(σ / σ_ethane); 0 for the fuels the laws were typed on |
| `fuel.se_coord` | [6, 2] | (−2, 3) | 1 | coordinates (e, b) of the fuel in the ethane and butane directions for the self-extinction laws |
| `fuel.ft_coord` | [6, 2] | (−3, 4) | 1 | the same for the flow-termination laws (burner limit, stop atoms) |
| `fuel.dosc` | [6] | (−3, 3) | ln(1) | shift of ln of the quench-clock median; 0 for the fuels the clock was typed on |
| `fuel.tad_min_k` | [6] | (0, 3000) | K | below this adiabatic temperature the fuel is refused (the weak-flame law is an alkane law); 0: no bound |

### 3.3 Computation

```text
k   = row of `fuel_species`; no row, or fuel.admit[k] = 0                      -> refuse (condition gate)
      balance gas of either stream not nitrogen; flow not constant from ignition -> refuse
      tad < fuel.tad_min_k[k]                                                    -> refuse
      stored indicators `fuel_is_ethane`, `fuel_is_butane` present and not the row's own -> refuse

vertices  E: (f_ethane, f_butane) = (1, 0)    P: (0, 0)    B: (0, 1)
weights   (w_E, w_P, w_B) = (e, 1 - e - b, b)      with (e, b) = fuel.se_coord[k, :] or fuel.ft_coord[k, :]

N*(x)  =  sum over the vertices v with w_v != 0 of  w_v * N(x with its two indicator entries set to v)
          (a vertex with weight zero is not evaluated)

hinge laws (flame.rs)
  ln tau_R   = N*_se(se.tau ; c with c_logm + fuel.s[k])      the shift enters this one law only
  logit weak = N*_se(se.flash ; c)
  ln tau_B   = N*_ft(ft.tau ; c)
  stop atoms : logit_a + W_a . c, indicator entries of c = fuel.ft_coord[k, :]

growth laws (flame3.rs)
  W_F        = fuel.mw_g_mol[k]              then Y_R, m_tot, L, r_ss, b_rss, b_mtot exactly as now
  ln m_cr    = N*_se(se.flux ; c) - fuel.s[k]          normal family; inverse family: no shift
  ln D       = N*_se(se.diff ; c)
  logit weak = N*_se(se.flash ; c)
  ln tau_osc = N*_se(se.osc ; c) + fuel.dosc[k]
  ln dT      = N*_ft(bt.rise ; b)
  stop atoms : as above

everything after the clocks (classes, nodes, limits, integration, committee weights, planned stop): unchanged
```

The domain bounds on the six condition features are checked on the unshifted `c`, as now.

**Why this leaves the three fuels exactly as they are.** For ethane, propane and butane the weight vector is
a unit vector, so `N*` is one evaluation of the same network on the same numbers, multiplied by 1; the
shifts are `+ 0.0`; the molar mass is the same literal the indicators selected. In IEEE arithmetic all of
these are identities. The engineer's acceptance test: on every CFI-G train prefix and, forecasts only, every
validation prefix, the event tables of the new members equal those of the members of `acme_normal.wsp5` P1
and `acme_inverse.wsp5` P1 bit for bit (the chain check of PROTOCOL 11.3 counts 1,242 train artifacts).

**Rejected alternatives.** Feeding fractional indicators into the old forwards (finding 5). A formula from
descriptors to coordinates inside the forward: it would need typed coefficients of its own and, applied to
all fuels, would move propane and butane (the oxygen-demand rule alone moves butane by −0.04). A molar-mass
interpolation: at fixed mole fraction the effect goes as the mass fraction, not as W.

### 3.4 Literal assignments

Normal flames (one table for the committee; slots a0, a1 read `se_coord`, `ft_coord`, `s`; slots b0, b1
read all but `ft_coord`'s effect, since their `bt.rise` has no indicator weight):

```text
# rows: 0 ethane, 1 propane, 2 butane, 3 ethylene, 4 methane, 5 hydrogen
# Hydrogen is not answered: no CO2, no soot, Lewis number 0.29, 2.1 times the oxygen demand per gram.
fuel.admit[0:6] = [1.0, 1.0, 1.0, 1.0, 1.0, 0.0]
# Molar masses (P). The first three are the constants the indicators chose until now.
fuel.mw_g_mol[0:6] = [30.07, 44.10, 58.12, 28.054, 16.043, 2.016]
# ln(sigma / 3.725), sigma in g O2 per g fuel (T). Zero for the three fuels the laws were typed on (D:
# their true values, 0, -0.026 and -0.040, are inside the typed numbers). Ethylene 3.422, methane 3.989.
fuel.s[0:6] = [0.0, 0.0, 0.0, -0.085, 0.068, 0.757]
# Self-extinction laws. Ethylene: half-way from ethane to butane (J): soot position 1.0 (between "below
# propane" in microgravity, L, and 2.0 by smoke point, P) times one half that the butane terms are soot.
# Methane: as ethane (J): a flame cannot soot less than one that hardly soots.
fuel.se_coord[0, 0:2] = [1.0, 0.0]
fuel.se_coord[1, 0:2] = [0.0, 0.0]
fuel.se_coord[2, 0:2] = [0.0, 1.0]
fuel.se_coord[3, 0:2] = [0.5, 0.5]
fuel.se_coord[4, 0:2] = [1.0, 0.0]
fuel.se_coord[5, 0:2] = [0.0, 0.0]
# Burner laws. Ethylene: ethane's molar mass to 7 % (T: the heating law puts it at most 0.18 sooner than
# ethane in ln time, a quarter of the clock's width at most). Methane: the heating law at 50 % fuel puts
# it 1.0 later than ethane (T); on the line through ethane and butane that is (1.9, -0.9).
fuel.ft_coord[0, 0:2] = [1.0, 0.0]
fuel.ft_coord[1, 0:2] = [0.0, 0.0]
fuel.ft_coord[2, 0:2] = [0.0, 1.0]
fuel.ft_coord[3, 0:2] = [1.0, 0.0]
fuel.ft_coord[4, 0:2] = [1.9, -0.9]
fuel.ft_coord[5, 0:2] = [0.0, 0.0]
# The quench clock is off for normal flames.
fuel.dosc[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
# The weak-flame boundary (1740 to 1750 K) was typed on alkanes; a kinetic limit is a fuel property.
fuel.tad_min_k[0:6] = [0.0, 0.0, 0.0, 1900.0, 1900.0, 0.0]
```

Inverse flames (ethylene is the only new fuel any of the three investigations burned inverted):

```text
fuel.admit[0:6] = [1.0, 1.0, 1.0, 1.0, 0.0, 0.0]
fuel.mw_g_mol[0:6] = [30.07, 44.10, 58.12, 28.054, 16.043, 2.016]
# Not read by the inverse laws: the burner-side reactant is oxygen and the flux is an oxygen flux.
fuel.s[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
# No adopted inverse self-extinction law has an indicator weight; ethylene is written as its nearest fuel.
fuel.se_coord[0, 0:2] = [1.0, 0.0]
fuel.se_coord[1, 0:2] = [0.0, 0.0]
fuel.se_coord[2, 0:2] = [0.0, 1.0]
fuel.se_coord[3, 0:2] = [1.0, 0.0]
fuel.se_coord[4, 0:2] = [0.0, 0.0]
fuel.se_coord[5, 0:2] = [0.0, 0.0]
# Ethane's +0.3 in the heating law is three times what its Lewis number gives and follows pressure as
# well as fuel in train. Ethylene: half of it, +0.15 (J): the Lewis-number rise (+0.13, T) and the median
# of the one ethane session near 1 atm (+0.13, D).
fuel.ft_coord[0, 0:2] = [1.0, 0.0]
fuel.ft_coord[1, 0:2] = [0.0, 0.0]
fuel.ft_coord[2, 0:2] = [0.0, 1.0]
fuel.ft_coord[3, 0:2] = [0.5, 0.0]
fuel.ft_coord[4, 0:2] = [0.0, 0.0]
fuel.ft_coord[5, 0:2] = [0.0, 0.0]
# A lean ethylene flame should not pulsate (Lewis number 1.28, below ethane's) and then ends at the
# published critical flux of 0.68 g O2 m-2 s-1 (L) instead of the 1.0 at which propane and butane begin
# to pulsate (D, model-inferred). Twice the clock (J): two routes give 1.8, and 1.5 to 3.
fuel.dosc[0:6] = [0.0, 0.0, 0.0, 0.7, 0.0, 0.0]
fuel.tad_min_k[0:6] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
```

New numbers that are not property values: four (0.5, 0.5 for ethylene's self-extinction; the methane burner
pair; the inverse 0.5; the inverse 0.7) and one bound (1900 K).

### 3.5 Where each typed number comes from

- **−0.085 and +0.068**: ln(3.422 / 3.725), ln(3.989 / 3.725). In the growth member the ethylene shift makes
  the critical flux 0.196 g m⁻² s⁻¹, against the published 0.20. In the hinge member the shift is exact for
  undiluted fuel; for diluted fuel the true shift of the reduced flow is −0.05 (30 %) and −0.02 (13 %) for
  ethylene, and +0.19 and +0.33 for methane (30 % and 13 %), because of the geometry of 1.2 that the hinge member lacks.
- **(0.5, 0.5)**: on the line through the two fuels that have train evidence; propane's own vertex, which
  was typed without a test, is not used. It differs from propane's law only by half of the dilute-butane
  term.
- **(1.9, −0.9)**: the equivalent shift of ln τ_B is −(4.5 / 0.8) × Δln ΔT with ΔT from the heating law at
  the same fuel flow, fuel fraction and oxygen. For methane against ethane: +0.15 undiluted, +1.0 at 50 %,
  +1.3 at 30 %, +1.6 at 15 %. One constant cannot serve all of these; 50 % is typed.
- **+0.7**: (a) the transient fuel supply of the second dossier, `K(Z_st) σ Y_F ρ √D_F`, at 9 % ambient fuel
  and 85 % oxygen is 0.91 of propane's for ethylene; time to a critical flux goes as (supply / flux)²:
  (0.91 / 0.68)² = 1.8. (b) The published law itself (flux 0.68, D 28 mm²/s, the reference set of the
  inverse spec) gives lean ethylene flames 18 to 63 s at 5 to 11 mg/s and 8 to 12 % fuel, where the adopted
  quench clock gives 8 to 28 s at 85 % injected oxygen and 4 to 15 s at 50 %: a factor 1.5 to 3 at 85 %,
  2.5 to 5 at 50 %. The published law depends on
  the flow near its holding flow (1.0 to 2.3 mg/s of oxygen for these ambients); a shift cannot say that.

## 4. Registered expectations

Stated before any comparison with Flame Design or s-Flame outcomes. "Life" is the median hot-flame life of
the main class; ratios are at the same pressure, fuel mass flow, fuel mole fraction and oxygen, each fuel
with its own Z_st and T_ad. Ranges are the author's 80 % intervals.

### 4.1 Ethylene, normal flames

| Regime | Ethylene life / ethane life | Why |
|---|---|---|
| Air | 1.00 (0.90 to 1.10) | no radiative term in air; reduced flow 3 to 8 % lower but the clock is flat in flow there |
| 30 % O2, away from the holding flow | 0.90 (0.70 to 1.25) | half of the butane term (−0.11 in ln) against 2 to 8 % less reduced flow |
| 38 to 40 % O2, flow at least 1.5 times the holding flow | 0.85 (0.55 to 1.25) | hinge member 0.73 to 1.06, growth member 0.70 to 0.90; the two ends of the range are "as butane" and "stoichiometry only" |
| Within 30 % of the holding flow | held more often than ethane | holding fuel flow higher by 8 % (undiluted) to 2 % (13 % fuel) |

Against the surrogate run of PROTOCOL 11.3 the table changes the growth member's forecasts in one direction:
reduced flow up by 9 to 23 %, clock 1.4 times faster at 39 % oxygen, so more self-extinction inside
6 to 120 s. The hinge member moves less and both ways.

### 4.2 Methane, normal flames

| Regime | Methane life / ethane life | Why |
|---|---|---|
| Air | 0.95 (0.80 to 1.10) | flat clock |
| Undiluted, enriched O2 | 0.90 (0.60 to 1.20) | 7 % more reduced flow; radiation as ethane; chemistry unknown, and if anything against methane |
| In nitrogen at 30 to 50 % by mole, enriched O2 | 0.75 (0.40 to 1.10) | at one mole fraction methane is the leanest gas by mass: reduced flow +20 % at 30 %, +39 % at 13 %. Growth member 0.45 to 0.90, hinge member 0.80 to 1.00: **the members disagree**, and the growth member is the one that holds the mechanism |
| Near the holding flow | goes out where ethane is held | holding fuel flow lower by 6 % (undiluted), 17 % (30 %), 28 % (13 %) |

### 4.3 Inverse flames

- **Ethylene against the adopted clock** (which is the same for every CFI-G fuel at one ambient mole
  fraction): lean flames last 2.0 times as long (1.2 to 3.7), without pulsating first. In seconds, at 85 %
  injected oxygen: about 19, 35 and 59 s at 8, 10 and 12 % ambient ethylene (the adopted clock: 10, 18,
  30 s). Above 13.5 % the adopted law holds the flame and the table changes nothing.
- **Ethylene against ethane at the same conditions**: 0.9 (0.5 to 1.5) from the physics (supply ratio 0.97, so 0.95
  in time at one critical flux; ethane's critical flux is unobserved, 0.45 to 0.7). The model gives exactly 2.0, because ethane keeps the
  adopted clock: this is the price of leaving ethane untouched, and it is ethane's forecast in a lean
  ambient that is the less credible of the two.
- **Methane**: never burned inverted in these investigations; refused. If it had to be answered, the same
  argument gives a clock about 1.75 times the adopted one.

### 4.4 Burner heating

| Case | Equilibrium rise against ethane, same conditions | Consequence |
|---|---|---|
| Ethylene, normal | +3 % undiluted, +1 % at 30 %, 0 at 15 % (±5 %) | limit trips where ethane trips; 4 to 5 % less heating than the surrogate run assumed for diluted fuel |
| Methane, normal | −3 % undiluted, −16 % at 50 %, −21 % at 30 %, −26 % at 15 % (±8 %) | the flow below which the limit is reached falls to 0.97, 0.80, 0.74, 0.70 of ethane's |
| Ethylene, inverse | +0.15 in ln above the propane and butane law (0 to +0.4) | half of ethane's typed excess. If ethane's excess is a pressure effect, ethylene at 0.5 atm will sit +0.3 to +0.4 above the law and at 1 atm on it |

### 4.5 Where the law should refuse

- A fuel without an admitted row: hydrogen, hydrogen/methane blends, methane in inverse flames.
- A balance gas other than nitrogen in either stream (the helium rounds of s-Flame): ρD, the Lewis numbers
  and the far-field diffusivity are all nitrogen values.
- A flow that is ramped or stepped (60 normal and 9 inverse rows of the Flame Design tables; most of
  s-Flame): every clock counts from ignition at one flow.
- Ethylene or methane with T_ad below 1900 K: the weak-flame boundary is an alkane number. The published
  ethylene flames lie at 2000 to 2800 K (L).
- Everything the declared domain refuses today, unchanged.
- Not a refusal but outside the target: the kinetic, low-flow extinction that s-Flame sought. The models end
  a low-flow flame on the burner limit and know no other way.

## 5. What would show this is wrong, and what to look at first

Each line names the observation on **train days only** of Flame Design or s-Flame that would contradict a
typed number. An author who makes these looks treats those days as development data from then on.

| Typed | Contradicted by | Then |
|---|---|---|
| `fuel.s` = −0.085 (ethylene) | reduced flow computed with ethylene's molar mass, its own Z_st and flux 0.196 does not separate held from extinguished flames near Φ = 1 as it does on CFI-G train (held at 1.15, out at 1.4 to 1.6); a shift of the boundary beyond ±20 % | the oxygen rule, or ρD, does not carry to ethylene |
| `fuel.se_coord` = (0.5, 0.5) (ethylene) | far from the holding flow (Φ above 2.5) at 30 to 40 % O2, ln(observed / law): near +0.2 means ethane-like (b = 0), near −0.2 butane-like (b = 1). **Decisive**: if the residual orders by Z_st or by flame colour and not by ambient oxygen, the butane direction is the wrong carrier for soot | a soot term gated by Z_st, which no existing direction provides |
| `fuel.ft_coord` (normal) | peak burner temperature against the heating law with the true molar mass: session medians of ln(measured / law) outside ±0.15 | the law's exponents are campaign numbers, not physics |
| `fuel.ft_coord` = (0.5, 0) (inverse) | the offset of ethylene flames from the law, split by pressure | if it follows pressure, weight 7 is a pressure term and belongs to all fuels; if it is constant, it is the fuel |
| `fuel.dosc` = +0.7 | lean ethylene flames show radiometer bursts and die within a few cycles, as propane does | 0.0, and the Lewis-number argument fails |
| the same | they do not pulse, and their end depends on the oxygen flow (short at high flow, held at 1 to 2 mg/s) | the right member for ethylene is the published growth clock (flux 0.68), not a shifted quench clock |
| methane (1, 0), +0.068 | undiluted methane lives below 0.6 of the law | the missing chemistry direction is real; type a critical-flux shift for methane |
| methane, diluted | lives of diluted against undiluted methane at one fuel mass flow do not shorten as `(Y_F L)²` predicts | the geometry of 1.2 fails, and with it finding 3 |

**First looks, in this order.**

1. *Inputs before outcomes.* Table flows against set-points and read-backs (CFI-G had eleven wrong rows in
   154); Z_st from the initial composition; the burner-limit setting of each day from the trip
   temperatures; the script stage sums; which rows are constant-flow.
2. *The reduced-flow ordering* of the ethylene normal flames (first row of the table above). It tests the
   one direction that costs no judgement.
3. *The residual against Z_st and against oxygen* for flames far from the holding flow. Flame Design varied
   where the nitrogen was placed for exactly this reason; it is the experiment that can say what the butane
   direction is.
4. *Inverse flames*: bursts or none; burner offset against pressure; and adjudication of the 18 tests where
   the sensor label and the script's account disagree, before any of them is used.
5. *s-Flame*: the fuel-bottle fractions and how many nitrogen-diluted methane or ethylene tests ran at one
   flow. If few did, s-Flame cannot test this table and needs a forward that follows a flow history.

## 6. Checks, sources, exposure

**Recomputed by me** (scratch scripts, not in the repository; every formula is in the text): σ, heating
values and gas-band emission per MJ; `Y_F L` and its slope; the heating-law equivalents of weight 5 for five
fuels at four fuel fractions and three oxygen levels; the median clocks of both normal members for ethane,
ethylene and methane on a grid (fuel fraction 1.0, 0.3, 0.13; oxygen 21, 30, 39 %; 0.75 to 4 mg/s) under four
coordinate choices; the inverse supply ratios and the published-law lives. The ratios of section 4 are read
from those tables at the central node of the reduced-flow distribution, not from a forecast.

**Train recounts** (CFI-G train rows through `lib.py` and `acme_table.py`, written to the session scratch
folder): the inverse heating residuals of finding 4; the three ethane burner-limit times of finding 1.

**Repository documents read**: `AGENTS.md`; `PROTOCOL.md` sections 2, 8, 10, 11.1 to 11.3; cards
`acme_clock.md`, `acme_growth.md`, `acme_committee.md`; `flame.rs`, `flame3.rs`, the head of `flame4.rs` and
`committee.rs`; the edits files of `wsp2/normal` (baseline, D3), `wsp3/normal` (baseline, D1, D2),
`wsp3/inverse` (baseline, D1, D5), `wsp5` (both P1; the baseline sets for their indicator entries);
`phys_acme_normal.md`, `phys_acme_inverse.md`, both `verify_phys_acme_*.md`, `lit_spherical_flames.md`;
`flame_design_transfer_result.md`; the atlas entries and the `tests` and `how_tests_differ` fields of the
cards of PSI-10, PSI-23 and PSI-159 (their `findings` fields were not opened; one truncated line of each was
printed while listing the structure: the 1130 K statement already in the literature review, and that both
extinction modes were seen in s-Flame).

**Literature opened for this file**: NIST Technical Note 2279 (table of diffusion coefficients, pages 13 to
16); Li and Sunderland 2012 (the table of normalised smoke points). Property values marked "from memory"
were not looked up.

**Exposures.**

- The author of this file had read the aggregate outcome of the Flame Design transfer test, including its
  sensitivity arms (extinction score with ethane's, propane's and butane's indicators: 0.1203, 0.1039,
  0.1289 for normal flames). The self-extinction coordinate typed here for ethylene, half-way to butane, is
  close to propane's law, the best of the three arms. The number was derived from the soot indices before
  that comparison was drawn, and the arms differ in molar mass as well, so they do not test it; but the two
  are not independent, and a candidate built from this file treats the train and validation days of Flame
  Design as seen in aggregate. Its test days are unread.
- A web search for smoke-point tables returned, in its summary, the published microgravity sooting ranking
  of gaseous fuels. That work rests on ISS smoke-point experiments which the atlas lists among the
  investigations whose groups are frozen (PSI-107). Only the ranking was kept; no number was fetched. It
  moved the ethylene soot position from "twice butane" towards "about butane".
- All coordinates multiply weights typed after CFI-G train data: everything they produce is class D.

**Weakest assumptions.**

1. That the butane terms are soot and carry to ethylene at half strength. The mechanism is not demonstrated
   for butane; ethylene's soot follows Z_st, and the terms are gated by ambient oxygen.
2. That a lean ethylene inverse flame does not pulsate and ends at the published flux, so that twice the
   adopted clock describes it. It combines a model-inferred flux for propane and butane with a measured one
   for ethylene, and ethane, the nearest fuel, was never flown lean.
3. That methane differs from ethane only by molar mass and oxygen demand. Its chemistry is not in any
   descriptor, the hinge member cannot follow its dilution, and one burner coordinate stands for a quantity
   that changes sixfold with the fuel fraction.
