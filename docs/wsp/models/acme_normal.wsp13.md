# acme_normal.wsp13 — normal burner flames, committee F4 (hydrogen stage)

`acme_normal.wsp13` F4 forecasts how a normal (fuel into oxidizer) spherical burner flame ends: the probability over
time that it goes out by itself (extinction) or is stopped (flow termination by the burner limit or the script), for
ethane, propane, butane, ethylene, methane and hydrogen, diluted in nitrogen or helium. On the helium-diluted
flames of the s-Flame validation days it scores a summed integrated Brier score of **0.1105 against 0.3299** for the
CFI-G train incidence, and on the hydrogen flames of the s-Flame train days **0.3140 against 0.9087**
(`SpaceApps-Ardealu:docs/wsp/feedback/sflame_f4_result.md`); on ethane, propane, butane and ethylene in nitrogen it
returns the forecasts of F3 stage 1 exactly. It is data-informed: every number was typed by Claude Fable 5.2 after
reading development data, and the methane, helium and hydrogen parts are in-sample on every partition (see 3 and
6). Release `sha256:4cc2fa60cc8c40b316594034ab8f299946117705736677e941c99f645fb89436`.

| | |
|---|---|
| Model | `acme_normal.wsp13` |
| Investigation | ACME CFI-G (PSI-159), Flame Design (PSI-10), s-Flame (PSI-23): normal burner flames |
| Family | `acme_normal` (assemble key `acme_normal13`) |
| Candidate | F4 (`f4-hydrogen`), F4 stage 3 |
| Generation | seventh: numbers typed under PROTOCOL 13.6 before the final run; served after the post-final study of 13.8 (see 3) |
| Author | `claude-fable-5-2` (Anthropic); transcribed by the assembly engineer `claude-opus-5-5` |
| Authorship | data-informed |
| Release | `sha256:4cc2fa60cc8c40b316594034ab8f299946117705736677e941c99f645fb89436` |
| Forward | `ignis_wsp::committee4::forecast`, `crates/ignis-wsp/src/committee4.rs` (forward version 13), members on `flame8.rs`, `flame9.rs`, `flame10.rs`, tables in `fuel2.rs` |
| Parent release | `sha256:370ed3cb1d72bccacef3ce63e1f3c9e6d7bf6d08cbf8fa81ed4cd8b57e4a62c0` (`f4-helium`) |

## 1. Task and targets

Inputs (from the served entry, `configs/models/served/served.json`; examples are typed values, not measurements):

| Name | Kind | SI unit | Required | Note | Example |
|---|---|---|---|---|---|
| `fuel_species` | condition (text) | 1 | yes | ethane, propane, butane, ethylene, methane or hydrogen; a blend such as `hydrogen+methane` is refused | `propane` |
| `injected_mass_flow` | condition | kg/s | yes | fuel (or fuel-side) mass flow | 1e-06 |
| `pressure` | condition | Pa | yes | chamber pressure | 101325 |
| `tad` | condition | K | yes | adiabatic flame temperature | 2300 |
| `x_fuel_role` | condition | 1 | yes | fuel mole fraction of the fuel stream | 0.3 |
| `x_o2_role` | condition | 1 | yes | oxygen mole fraction of the oxidizer stream | 0.3 |
| `zst` | condition | 1 | yes | stoichiometric mixture fraction | 0.25 |
| `phase_established_at_s` | phase start | s | yes | time the flame was established; the origin is hot ignition, so 0 for a flame burning from ignition | 0 |
| `programmed_stop` | condition | s | no | planned stop from NASA's script file; without it the release answers as on the shared records | – |
| `session_id` | test date | 1 | no | date of the test; selects the burner-limit era; without it the era weight for an unknown date is used | `2023-03-01` |
| `burner_limit_c` | condition | degC | no | burner temperature limit of the test script | – |
| `diluent_species` | condition (text) | 1 | no | `nitrogen` or `helium` (absent: nitrogen); other texts are refused | – |
| `x_o2_role_b`, `tad_b`, `zst_b` | condition | 1, K, 1 | no | the higher of two candidate oxygen fractions for a day that names two; all three or none | – |

Target: the cause-specific end of the established hot flame, as a partition of four rows at each requested time:
`active_supported`, `extinction`, `operator_termination`, `unresolved_support` (they sum to one). An operator or
script stop is its own cause, never an extinction; extinction is the end of the hot flame (cool flames are outside
the model). The forecast is conditional on the flame burning at the cutoff; horizon 600 s
(`numerics.horizon_s`). Time origin: hot ignition, the table's "burn time (sec)".

A stored CFI-G run supplies the conditions; the planned stop is not in the shared records, so stored runs answer
without it. A typed scenario needs the seven required conditions and the time the flame was established; it takes
no observations.

## 2. Physical mechanism

The release is a committee: ten complete member models, a weight vector per era of the campaign, and three
readings of the script's planned stop. Its forecast is the weighted mean of the members' event tables
(`committee4.rs`, rules of `committee3.rs`):

$$P(t)=\sum_k o_k\sum_i w_i\,P_{i,k}(t),\qquad w_i=(1-p_{\rm late})\,\hat w^{\rm early}_i+p_{\rm late}\,\hat w^{\rm late}_i$$

with $o$ the normalised `script.offset_w`, the members run once per offset node with a scheduled stop at
`programmed_stop + script.offset_s[k]` (only when the planned stop is given; otherwise one pass with `mix.w_*`),
and $p_{\rm late}$ = 0 up to `era.last_early_day`, 1 from `era.first_late_day`, `era.p_late_between` in between,
`era.p_late_unknown` without a date (tensors `mix.*`, `era.*`, `script.*`). With two oxygen candidates the
committee forecast is formed twice and mixed, $P=(1-w_b)P_{\rm first}+w_bP_{\rm second}$ (`mix.o2_b_w`).

Each member reads the fuel row and the diluent row (`fuel2.rs`). With a Lewis number of oxygen different from one
(helium), the sheet temperature is

$$T_1=T_0+(T_{ad}-T_0)\,\frac{1-(1-Z_{st})^{1/Le}}{Z_{st}},\qquad T_0=298.15\ \mathrm{K}$$

and a fuel is refused when $T_1$ < `fuel.tad_min_k` (`dil.le_o2`, `fuel.tad_min_k`, `phys.t_stream_k`).

**Growth members (b0 to b3, `flame9.rs`).** The flame grows by far-field diffusion towards its quasi-steady radius
and goes out when the fuel flux at the sheet falls to a critical flux (Kim et al. 2023; Irace et al.):

$$r_{ss}=\frac{\dot m_{tot}}{4\pi\rho D\,L},\quad r_{ext}=\sqrt{\frac{\dot m_R}{4\pi\dot m''_{cr}}},\quad \Phi=\Big(\frac{r_{ss}}{r_{ext}}\Big)^2,\quad R(\Phi)=\frac{r_{ext}^2}{4D\,\mathrm{erfcinv}(\Phi^{-1/2})^2}$$

$$\ln\dot m''_{cr}=N^*_{se}(\text{se.flux};c)-\text{fuel.s}+\text{dil.dflux}-\text{fuel.gz}\,\max(0,Z_{st}-\text{fuel.zst0}),\qquad \ln D=N^*_{se}(\text{se.diff};c)+\ln \text{dil.d\_far}$$

with $\Phi\le1$ a held flame, a spread on $\ln\Phi$ (`se.sigma_phi`, 15 nodes), a weak class at ignition
$p_{weak}=\sigma(N_{flash}(c))$ and an establishment floor (`se.floor_s`). F4's methane step and hydrogen row act
through `fuel.s` (the critical flux: methane 0.18 → 0.33 g m⁻² s⁻¹, hydrogen 0.027), and the helium row through
`dil.dflux` (+1.15, the energy balance $\dot m''_{cr}=\rho D\,Y_F L/(\Theta\ell^*)$ at the helium flame's sheet
temperature, `phys_sflame_f4.md` M-a).

**Hinge members (a0, a1, `flame8.rs`; c0 to c3, `flame10.rs`).** The radiative life is a hinge law in the reduced
flow, $T=\text{floor}+R$ with $R$ log-logistic of median $\tau_R$,

$$\ln\tau_R=N^*_{se}(\text{se.tau};\,c\ \text{with}\ c_{logm}+\text{fuel.s}+\text{dil.dlogm})+\text{dil.dlnr}$$

so methane's `fuel.s` (1.0) moves its flames off the two low-flow hinges (M-b), and helium enters as a thermal
equivalent of a nitrogen flame at about 9 % less oxygen (`dil.dlogm` +0.6, `dil.dlnr` −0.75, M-c).

**Burner heating (b and c members).** The burner warms towards an equilibrium and the flow is cut when it crosses
the limit of the test's setting:

$$T_b(t)=T_0+\Delta T\,(1-e^{-t/\tau_b}),\qquad \ln\Delta T=N^*_{ft}(\text{bt.rise};b)+\kappa_{exp}\,\ln(1+\textstyle\sum_i x_i(k_{rel,i}-1))+\ln\frac{T_1-T_b}{T_{ref}-T_b}$$

with the limit `bt.limit_c[l] + (burner_limit_c − bt.limit_script_c)` (`bt.*`, `fuel.k_rel`, `dil.k_rel`). Hydrogen
(conductivity 7.0 times nitrogen) and helium (5.8) heat the burner faster through the conductivity term.

**Programmed stops** are stop atoms at the campaign's script times (`ft.atom_*`, `ft.log_base`); members whose
guessed stops are off carry them at logit −40 and are the ones weighed when the planned stop is known.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section acme_normal.wsp13.

## 3. Authoring

**Who.** Every number of F4 was written by `claude-fable-5-2` in
`SpaceApps-Ardealu:docs/wsp/research/phys_sflame_f4.md` section 6 (15 November 2026); the assembly engineer
`claude-opus-5-5` transcribed the lines verbatim into
`configs/models/acme/wsp13/normal/f4-hydrogen/edits.wsp` ("Transcribed by an engineer (claude-opus-5-5); revision of
f4-helium"; meta.json rationale: "transcribed verbatim; a comparator by PROTOCOL 13.6"). The tensors equal those of
the child of the author's own logged run (edit file sha256 `25d9e25a…dfee`). The prompt is in
[docs/research/PROMPTS.md](../../research/PROMPTS.md).

**Generation and step, exactly.** Seventh generation. The F4 numbers were typed under PROTOCOL 13.6 (a case-by-case
study of the development days, each candidate run once with its expectation logged first; at most five candidates
per law): methane 2, helium 1, hydrogen 2 logged candidates (`phys_sflame_f4.md` section 4). The s-Flame test days
(20142, 20170, 20176) and day 19245 were not opened when they were typed. F4 stage 3 was then carried, unchanged, as
a comparator into the one final run of 13.7. The decision to serve it was taken under 13.8, after the final run, with
every partition seen; no number was typed or changed in that step (`phys_sflame_post_final.md`: "No candidate was
typed and none was run"). By the labelling rule of 13.8 the release is therefore described as "typed after the final
run with every partition seen; in-sample on all data; no held-out score exists" (served `limits`), for its methane,
helium and hydrogen parts. The release record was created at 2026-11-15T08:31:02Z.

**What was read** (the record's exposure, `release.json` `construction.exposure`): CFI-G train rows and train
control timelines; aggregate CFI-G validation tables of earlier candidates and validation run conditions (inputs
only); raw channels and script files of train tests; the train days of Flame Design (`phys_ethylene_held.md`);
`phys_sflame_f3.md` (s-Flame train days, aggregates of the s-Flame validation days); and `phys_sflame_f4.md`: a
case-by-case study of the s-Flame train and validation days (traces of 122 of 125 constant-flow tests). Datasets in
the record: train (`acme_normal-c77bc5baee98d3e3`) and validation as aggregates and conditions. The record lists no
test-day source; the test days were read only in the post-final study, which changed nothing.

**Dossiers.** `phys_sflame_f4.md` (F4), on `phys_sflame_f3.md` (F3), `phys_ethylene_held.md`,
`phys_spherical_fuels.md`, `phys_acme_normal.md`, `acme_planned_stop_v2.md`.

**Decisions, from the parent chain** (each stage is a literal revision of the one before):

| Stage | Release | Lines | What and why (`phys_sflame_f4.md` 2 to 6) |
|---|---|---|---|
| F4 stage 1, methane | `f4-methane` `sha256:57fe4255…ebe2` | 8 | `fuel.s[4]` 0.35 → 1.0 in c0 to c3 (hinge: methane at 30 % oxygen has no long low-flow branch, its life falls with flow; F3 had given it 22 to 87 s lives, so the limit stop won on paper); −0.35 → −0.6 in b0 to b3 (growth: critical flux 0.255 → 0.33 g m⁻² s⁻¹; the energy balance gives 0.31 to 0.42). a0, a1 (weight 0) left at 0.35, as run |
| F4 stage 2, helium | `f4-helium` `sha256:370ed3cb…62c0` | 90 | the helium row in all ten slots: `dil.dflux` 0.3 → 1.15 (energy balance at the helium sheet temperature), `dil.dlogm` −1.9 → 0.6 and `dil.dlnr` −1.25 → −0.75 (thermal equivalence); `d_far` 3.5, `rho_d` 1.15, `le_o2` 1.37, `k_rel` 5.8 kept from Part A of `phys_sflame_f3.md`. The author states the data alone would have chosen +1.0 to +1.2, about 0, and about −0.5 |
| F4 stage 3, hydrogen | `f4-hydrogen` (this release) | 70 | hydrogen admitted in all ten slots: growth `fuel.s[5]` 1.9 (critical flux 0.027 g m⁻² s⁻¹), hinge `fuel.s[5]` −2.0 (no hydrogen flame up to 0.6 mg/s in air or 0.8 mg/s in helium went out), coordinates (1, 0), `tad_min_k` 1100 K (nothing flown colder than 1188 K), `dtad_k` +600 K (a lower extinction temperature; +900 K was tried in candidate Hy1 and left the temperature coordinate out of range), `k_rel` 7.0 (conductivity of hydrogen) |

Not typed: a blend law (the oxygen-share rule fails on the hydrogen-rich bottle of day 20013, 1.53 per case);
blends stay refused (by the serving layer, `refuse_blends`, since the forward itself would mix two admitted rows).

## 4. Weights

Totals (`release.json` `architecture.parameter_counts`): **5069 authored elements, 1271 nonzero; 170 constants**
(the `norm.*`, `num.*` and `phys.*` scalars of each slot), 5239 elements in all. Per slot (elements / nonzero,
constants included): a0, a1 468 / 131–132; b0 to b3 623 / 142–143; c0 to c3 440 / 147–148; committee lines 51 / 18.
Full list: `ignis wsp inspect --release sha256:4cc2fa60cc8c40b316594034ab8f299946117705736677e941c99f645fb89436 --edits`,
or the edits files of the chain (`configs/models/acme/wsp13/normal/f3-s1/edits.wsp` holds every slot value;
`f4-methane`, `f4-helium`, `f4-hydrogen` hold the three revisions).

**Where the member values come from.** Most member values are copies of earlier releases, made statement by
statement by `tools/wsp/committee4_edits.py` (F3 stage 1 edits, slot headers):

| Slots | Member kind | Law values copied from | Origin of those laws |
|---|---|---|---|
| a0, a1 | hinge laws, `flame8.rs` | `acme_normal.wsp9` f2-d4 `sha256:6fdab760…aaa593`, f2-d4-nostops `sha256:e8daeea4…48e9c9` | laws of `wsp2/normal/d4-fable`, fuel table of F2 |
| b0 to b3 | growth law with burner heating, `flame9.rs` | `acme_normal.wsp10` f2-d3-early `4cd34619…`, -late `d8fedc57…`, -early-nostops `3cb68481…`, -late-nostops `9558eae9…` | laws of `wsp3/normal/d3-early` / `-late` |
| c0 to c3 | hinge laws with burner heating, `flame10.rs` | `acme_normal.wsp11` f2-d1-early `3740e98d…`, -late `d5b0e2b7…`, -early-nostops `7f0a50c0…`, -late-nostops `c120f0c2…` | laws of `wsp8/normal/d1-early` (the P3 hinge member) |

Within a kind the members differ only in the era limit (`bt.limit_script_c`, `bt.limit_w`: early 500 °C setting
with the 510 °C limit, late 450 °C with 455 °C) and in whether the guessed stops are on (`ft.atom_logit`,
`ft.log_base`). The F3 stage 1 lines (heating-law factor and exponent, burner-limit input, fuel and diluent tables)
and the three F4 revisions are the only lines typed after the copy.

| Group (tensor pattern) | Shape / count | Value(s) | Physical meaning |
|---|---|---|---|
| `mix.w_early`, `mix.w_late` | [10] each, 2 nonzero | early: b0 = c0 = 1; late: b1 = c1 = 1 | weights without the planned stop: growth and hinge-with-heating, equal, by era; a0, a1 weight 0 |
| `mix.ws_early`, `mix.ws_late` | [10] each, 2 nonzero | early: b2 = c2 = 1; late: b3 = c3 = 1 | weights with the planned stop (members with guessed stops off) |
| `era.last_early_day`, `era.first_late_day` | 1 + 1 | 2021235 (23 Aug 2021), 2021256 (13 Sep 2021) | burner-limit setting changed between these days |
| `era.p_late_between`, `era.p_late_unknown` | 1 + 1 | 0.5, 0.15 | probability of the late era between the days and without a date |
| `script.offset_s`, `script.offset_w` | [3] + [3] | −1, 0, +1 s; 0.25, 0.5, 0.25 | three readings of the planned stop from the script file (`acme_planned_stop_v2.md`) |
| `mix.o2_b_w` | 1 | 0.5 | equal weight of two oxygen candidates (J, `phys_sflame_f3.md`) |
| `<a,c>.se.tau.*` | W1 [8,8] 15 nonzero, b1 [8] 6, skip [8] 3, w2 [8] 5, b0 | b0 2.43; skip −0.28 (c_logp), 0.84 (c_o2), −0.2 (f_butane); w2 1.4, 2.0, −0.35, −0.14, −0.12; hinge biases 0.5, −0.22, −5.0, −5.9, −4.1, 0.9 | hinge law of the radiative life in the reduced flow (copied) |
| `<a,c>.se.shape`, `se.floor_s` | 1 + 1 | 3.0; 4 s | clock shape; establishment floor |
| `<a,c>.se.flash.*`, `flash_tau_s`, `flash_shape` | b0, skip | b0 −18.0, skip c_tad −12.0; 1 s; 4.0 | weak class at ignition |
| `<b>.se.flux.*` | b0, 1 hidden unit | b0 −1.715 (0.18 g m⁻² s⁻¹); w2 0.5 relu(−0.9 + c_logp − c_o2) | critical flux of the growth law |
| `<b>.se.diff.*` | b0, W1 5, b1 2, skip 4, w2 2 | b0 3.92; skip 0.45, −0.385, 0.2, 0.27; w2 0.21, 0.18 | far-field diffusivity of the growth law |
| `<b>.se.rho_d_g_m_s`, `se.sigma_phi`, `se.shape`, `se.floor_s`, `se.hold_log_rate` | 5 scalars | 0.0334 g m⁻¹ s⁻¹; 0.25; 4.0; 4 s; −8.0 | ρD of the quasi-steady radius; spread of ln Φ; clock shape; floor; hazard of a held flame |
| `<b>.se.osc.*`, `osc_shape` | b0 + zeros | b0 13.0; 3.0 | oscillation clock, median beyond the horizon: switched off for normal flames |
| `<b>.se.flash.*` | b0, skip | −10.2; c_tad −6.7 | weak class at ignition (growth members) |
| `<b,c>.bt.rise.*` | b0, skip [7] 2, 1 hinge unit | b0 5.88 (ΔT ≈ 358 K at 3 mg/s and 25 mm); skip −0.4 (b_rss), −0.4 (b_mtot); −0.15 relu(c_xf − 0.5) | equilibrium rise of the burner, ΔT ∝ m_tot^−0.4 r_ss^−0.4 (D-24) |
| `<b,c>.bt.sigma`, `t0_c`, `tau_s`, `shape` | 4 scalars | 0.12; 30 °C; 30 s; 8.0 | spread of the rise; start temperature; time constant; trip sharpness |
| `<b,c>.bt.limit_c`, `bt.limit_w`, `bt.limit_script_c` | [3] + [3] + 1 | limits 455 and 510 °C; early [0, 1, 0] at setting 500 °C, late [1, 0, 0] at 450 °C | the burner limit by era, moved by `burner_limit_c − limit_script_c` when the test gives its setting |
| `<b,c>.bt.kappa_exp`, `bt.tflame_ref_k` | 1 + 1 | 0.45; 2650 K | conductivity power and the flame-temperature factor of the heating law (F3, `phys_sflame_f3.md` B) |
| `<a>.ft.tau.*`, `ft.shape` | b0, skip 3 | b0 6.5; skip 4.5 (c_logm), 0.54 (f_ethane), −0.56 (f_butane); 2.5 | the hinge member's own log-logistic burner clock (weight 0) |
| `*.ft.atom_t_s`, `ft.atom_logit`, `ft.atom_w` | [8], [8], [8,8] | times 61.5, 98, 126.5, 275.5 s; logits −2.0, −2.9, −2.2, 0.0 (members with stops), −40 (a1, b2, b3, c2, c3); atom 0 −2.0 per c_logm (a, c) | programmed stops of the campaign (guessed stops) |
| `*.ft.log_base` | 1 per slot | −7.6 (a0, c0, c1), −7.4 (b0, b1), −9.0 (members without stops) | background stop hazard |
| `*.fuel.admit`, `fuel.mw_g_mol`, `fuel.sigma` | [6] each | all six admitted; 30.07, 44.1, 58.12, 28.054, 16.043, 2.016 g/mol; oxygen demand 3.725, 3.628, 3.579, 3.422, 3.989, 7.936 | fuel rows 0 ethane … 5 hydrogen |
| `*.fuel.s` | [6] | hinge (c0 to c3): 0, 0, 0, −0.31, **1.0**, **−2.0**; hinge a0, a1: methane 0.35; growth (b): 0, 0, 0, −0.02, **−0.6**, **1.9** | oxygen-demand shift (growth: ln flux; hinge: flow coordinate). F4 lines in bold |
| `*.fuel.se_coord`, `fuel.ft_coord` | [6,2] each | se: (1,0), (0,0), (0,1), (1,0), (1,0), **(1,0)**; ft: (1,0), (0,0), (0,1), (1,0), (1.9,−0.9), **(1,0)** | coordinates in the ethane and butane directions, self-extinction and burner laws |
| `*.fuel.tad_min_k`, `fuel.dtad_k`, `fuel.k_rel` | [6] each | 1900 K ethylene and methane, **1100 K hydrogen**; −350 K methane, **+600 K hydrogen**; **7.0 hydrogen**, 1 otherwise | temperature bound; shift of the weak-flame temperature; conductivity relative to nitrogen |
| `<b>.fuel.zst0`, `fuel.gz` | [6] each | ethylene 0.3 and 1.0; others 1 and 0 | ethylene dilution term of the growth law (`phys_ethylene_held.md`) |
| `*.fuel.dosc` | [6] | 0 | not used for normal flames |
| `*.dil.*` (9 tensors) | [2] each | nitrogen: 1, 28.013, 1, 1, 1, 0, 0, 0, 1; helium: 1, 4.0026, ρD ×1.15, d_far 3.5, Le 1.37, **dflux 1.15**, **dlogm 0.6**, **dlnr −0.75**, k_rel 5.8 | diluent table (`admit`, `mw_g_mol`, `rho_d`, `d_far`, `le_o2`, `dflux`, `dlogm`, `dlnr`, `k_rel`); bold: F4 stage 2 |
| constants `*.norm.*`, `*.num.*`, `*.phys.*` | 170 | reference scales ṁ 1e-6 kg/s, p 101325 Pa, x_O2 0.3 ± 0.1, x_F 0.3, T_ad 2200 ± 300 K, Z_st 0.25 ± 0.2, m_tot 3e-6 kg/s, r 25 mm; step 0.1 s, horizon 600 s, 15 Φ nodes, 7 heating nodes, cap 1e5 s; T_stream 298.15 K, M_O2 31.998 | not authored |

## 5. Forward and assembly

| | |
|---|---|
| Forward | `crates/ignis-wsp/src/committee4.rs`, `ignis_wsp::committee4::forecast`, version 13, profile `strict_no_fitting` |
| forward_hash | `sha256:3baf682b0faf29052a25beb26f4600e1f3dfda3d88b1362eccd00486b78d4e8b` |
| code_version | `ignis-wsp@0.1.0+fwd.3baf682b0faf2905` |
| Spec | `acme_normal-wsp-v13`, `sha256:036dfd9169749be17380a4ffd886f84484e2d7c58d9cf4678d889e0022e00b61` |
| Patch | `acme_normal-wsp-v13-f4-hydrogen`, `sha256:b63899dfdc92e538ddffc55b37fe72f76f1736d80ea02a433a8476741a4b02e8` |
| weights_hash | `sha256:1edce22ea7d1f9d652c32a8997cccc0aa02a6be25fc3ffc166a9d27c1e9df840` |
| Binding | cohort `sha256:92fb4728e88ed5175bd47da77707d042a0f046abca1bd8df351fd1d69fea4701`, split `sha256:c77bc5baee98d3e3494de144fa3f98ce0dee904efa35d5f7c0b48b91dbc82d14`, source manifest `sha256:11a1bdbd…e126` (`configs/models/acme/wsp13/normal/binding.json`) |
| Candidate | `configs/models/acme/wsp13/normal/f4-hydrogen` (edits.wsp, meta.json, patch.json, release.json) |
| Checks in the record | spec_valid, patch_valid, tensors_complete, strict_forward_audit, finite_forward_on_fixture: passed |

Rebuild (the parent record `sha256:370ed3cb…62c0` is committed at
`configs/models/acme/wsp13/normal/f4-helium/release.json`; the root must hold it as `models/370ed3cb…/`):

```
cargo build --release -p ignis-cli
ignis wsp compile  --family acme_normal13 --edits configs/models/acme/wsp13/normal/f4-hydrogen/edits.wsp --meta configs/models/acme/wsp13/normal/f4-hydrogen/meta.json --out configs/models/acme/wsp13/normal/f4-hydrogen/patch.json
ignis wsp assemble --family acme_normal13 --patch configs/models/acme/wsp13/normal/f4-hydrogen/patch.json --binding configs/models/acme/wsp13/normal/binding.json --root <registry holding models/370ed3cb1d72bccacef3ce63e1f3c9e6d7bf6d08cbf8fa81ed4cd8b57e4a62c0/> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the record; `served_ids` pins the 14 served ids.
`cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at ignis-web a58c7c3: the
served id is the content hash of its record and loads through the registry checks.

## 6. Evaluation

Metric: integrated Brier score, 6 to 120 s, cutoffs 1, 3 and 5 s, case means; s-Flame tables give the **sum** of
extinction and operator termination; lower is better. Cells "a | b" are without | with the planned stop.

**Headline, s-Flame development days** (`SpaceApps-Ardealu:docs/wsp/feedback/sflame_f4_result.md`, stage 3, endpoint
version 1 unless marked; in-sample: the author had studied these days case by case):

| Days | Stratum | Cases | F4 (`f4-hydrogen`) | F3 stage 1 | F3 stage 2 | F2 | CFI-G train incidence | s-Flame train incidence |
|---|---|---|---|---|---|---|---|---|
| validation | hydrocarbons in helium, pooled | 35 | **0.1105** \| 0.1099 | refused | 0.9990 | refused | 0.3299 | 0.3244 |
| validation | ethylene in helium | 21 | 0.1799 \| 0.1790 | refused | 1.5120 | refused | 0.3639 | 0.3610 |
| validation | methane in helium | 14 | 0.0064 \| 0.0063 | refused | 0.2295 | refused | 0.2788 | 0.2696 |
| validation | hydrocarbons in nitrogen, pooled | 46 | **0.0815** \| 0.0811 | 0.2048 \| 0.2051 | – | 0.2619 \| 0.3698 | 0.2793 | 0.3218 |
| validation | methane in nitrogen | 37 | **0.0916** \| 0.0911 | 0.2449 \| 0.2452 | – | 0.3157 \| 0.4498 | 0.2930 | 0.3293 |
| validation | methane in nitrogen, endpoints v2 | 39 | 0.0401 \| 0.0396 | 0.1902 \| 0.1913 | – | 0.2707 \| 0.4101 | 0.2692 | 0.3118 |
| validation | ethylene in nitrogen | 9 | 0.0400 \| 0.0401 | same | – | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| train | hydrocarbons in nitrogen, pooled | 53 | 0.0730 \| 0.0763 | 0.0736 \| 0.0769 | – | 0.0914 \| 0.0891 | 0.2725 | 0.3008 |
| train | hydrogen, endpoints v2 | 76 | **0.3140** \| 0.2094 | refused | refused | refused | 0.9087 | 0.6443 |
| train | hydrogen, endpoints v1 | 70 | 0.4363 \| 0.3948 | refused | refused | refused | 0.8330 | 0.6117 |

F3 stage 2 column: `feedback/sflame_f3_result.md`. By day (`phys_sflame_f4.md` section 5, adjudicated labels),
hydrogen is below both incidence baselines on 3 of its 4 train days; on day 19352 it is 0.644 against 0.788 (CFI-G)
and 0.571 (s-Flame). Neutrality on every stage: CFI-G train 0 of 210 cases and Flame Design train days 0 of 444
tables moved.

**Final run, s-Flame test days** (PROTOCOL 13.7, `SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_SFLAME.md`, release list
`ba34f8f7…e9c9`; the F4 numbers had not seen these days; every test-day case is helium-diluted):

| Stratum | Cases answered / at risk | F4 stage 3 | F4 stage 2 | F3 stage 2 | CFI-G train incidence | s-Flame train incidence |
|---|---|---|---|---|---|---|
| hydrocarbons in helium, as built | 49 / 60 | 0.7080 \| 0.7094 | 0.7080 \| 0.7094 | 1.0566 \| 1.0584 | 0.4812 | 0.4163 |
| blends | 0 / 29 | refused | refused | refused | – | – |

By the rule written beforehand the helium row "does not answer helium" in that run (`PROTOCOL.md` 13.7); the
result stands as recorded.

**After the final run** (PROTOCOL 13.8, `feedback/sflame_post_final_result.md`; every partition seen; descriptions
of fit, not test results). Eleven of the 26 test-day ethylene tests ramp their flow after ignition, a kind of test
no development day contains; eight were scored and carried 78 % of the loss. Third case version: (b) flow-ramped
tests excluded; (c) also oxygen by pressure block.

| Case version | Set | Cases / tests | F4 (`f4-hydrogen`) | F3 stage 2 | CFI-G train incidence | s-Flame train incidence |
|---|---|---|---|---|---|---|
| (a) as built | test days | 49 / 20 | 0.7080 | 1.0566 | 0.4812 | 0.4163 |
| (b) ramped excluded | test days | 31 / 12 | **0.2419** | 1.0226 | 0.4143 | 0.3769 |
| (c) and oxygen by block | test days | 31 / 12 | 0.1664 | 1.1535 | 0.4143 | 0.3769 |
| (a), (b), (c) | validation days | 35 / 14 | 0.1105 | 0.9990 | 0.3299 | 0.3244 |
| (a), (b) / (c) | hydrogen, train days | 76 / 28 | 0.3140 / 0.3144 | refused | 0.9087 | 0.6443 |

**Ethane, propane, butane and ethylene in nitrogen.** F4 changes only the methane and hydrogen rows and the helium
row, which these flames never read, so it returns the forecasts of F3 stage 1 (measured: 0 of 210 CFI-G train
cases, 0 of 444 Flame Design train-day tables moved). F3 stage 1's numbers (`feedback/sflame_f3_result.md`, PROTOCOL
13.5; development numbers):

| Set | Path | F3 stage 1 = F4 (extinction / termination) | `acme_normal.wsp12` F2 | Baseline |
|---|---|---|---|---|
| CFI-G validation, 84 cases, 2 sessions | shared records | 0.1536 / 0.0956 | 0.1537 / 0.0961 | – |
| CFI-G validation | planned stop | 0.1472 / 0.0795 | 0.1473 / 0.0798 | F1 0.1442 / 0.0749 (old stop input, `RESULTS_GEN6.md`) |
| Flame Design validation days, 86 cases | without the stop | 0.0299 / 0.3737 | 0.0317 / 0.3933 | incidence 0.2703 / 0.5340 |
| Flame Design validation days | planned stop | 0.0066 / 0.0732 | 0.0068 / 0.0724 | – |

The sealed CFI-G test partition and the Flame Design test days were scored once, with `acme_normal.wsp12` F2
(`RESULTS_FINAL.md`), and are not scored again.

## 7. Adoption decision and lineage

**Why this release is served.** PROTOCOL 13.8 (owner's decision after the final run): a post-final revision is served
only if it is lower than the constant or incidence baseline on each partition for the target it repairs and changes
no other served answer of its investigation by more than rounding. `f4-hydrogen` is below the CFI-G train incidence
on the helium cases of the validation days (0.1105 against 0.3299) and of the test days once flow-ramped tests are
excluded (0.2419 against 0.4143); hydrogen on the train days 0.3140 against 0.9087; CFI-G and Flame Design unmoved;
nitrogen ethylene identical to F3 stage 1. It is served, s-Flame answers helium and hydrogen, and blends and
flow-ramped tests are refused (`PROTOCOL.md` 13.8; `COVERAGE_19.md`: "yes: helium and hydrogen answered"). It
replaces F3 stage 1 as the served normal-flame release. Under 13.6 the three F4 stages had been reported comparators,
since each changes fewer than six days.

**Lineage** (parent ids from the release records):

| Step | Release | Parent | What changed |
|---|---|---|---|
| root | `baseline-set` `sha256:9e2c18165d2e0db13beed17b3aec8d19ad609b126dd13a424916da2f30c76591` | – | lineage root of the fourth committee spec (Opus copy, no number typed): the slots of the root of `acme_normal.wsp12`, neutral tables, heating terms off |
| F3 stage 1 | `sha256:d1660331c1f96091dad7f6f148c08f50db0cd03317b6c1dec458bd92696a45aa` | root | the slot laws and committee lines of `acme_normal.wsp12` F2 (`sha256:55fcf4cbbf500ca57a4e4207426826d5c3d3b21d84fb279479e29b7e5a65971d`) copied; burner limit as an input, heating-law factor 2650 K and power 0.45, methane step (`dtad_k` −350 K), ethylene dilution term; helium, hydrogen, blends refused (PROTOCOL 13.5: replaced F2) |
| F4 stage 1 | `f4-methane` `sha256:57fe4255b51b4b988da8aa021d71c7191f91b9527eee765226b34a1ed1b6ebe2` | F3 stage 1 | methane `fuel.s` in the hinge and growth members |
| F4 stage 2 | `f4-helium` `sha256:370ed3cb1d72bccacef3ce63e1f3c9e6d7bf6d08cbf8fa81ed4cd8b57e4a62c0` | F4 stage 1 | the helium row in every slot |
| F4 stage 3 | `f4-hydrogen` (this release) | F4 stage 2 | the hydrogen row in every slot |

Tried and not adopted: F3 stage 2 (`sha256:819ec0b61814cdfb6b1e3fde5b2bf8ef8a67a9ce455a58f5a46d878407cc9e51`, the
zero-shot helium row of Part A: 0.9990 on the helium validation days, "does not answer helium"); hydrogen candidate
Hy1 (`dtad_k` 900 K); a blend law (none typed); the post-final `dil.dlnr` near −0.45 that the data of all three days
would choose (declined by the author as a fit to data seen in full).

**Scope** (served `limits`, condensed):

- Its methane, helium and hydrogen parts were typed with every partition seen by the time it was served; in-sample on
  all data, no held-out score exists. The nitrogen ethane, propane, butane and ethylene forecasts are those of F3
  stage 1.
- Typed for constant-flow tests: a test whose flow is ramped after ignition is outside the model (the one held result
  with ramped tests included, 0.708, was worse than the incidence, 0.481).
- A hydrogen and methane blend is refused: there is no blend law.
- On CFI-G validation with the planned stop the nitrogen alkane forecasts are slightly worse than the model before
  them (sums of two sessions; see 6).
- On the Flame Design validation days every normal-flame case ended by a flow stop, so extinction there only measures
  false extinction; two or three validation days are few.
- A nitrogen-diluted ethylene flame lives longer than the growth law says; ethylene below 1900 K is refused (and
  methane below 1900 K, hydrogen below 1100 K).
- The endpoint method misplaces the flow change on tests with an ignition-to-test ramp.

## 8. How to call it in IGNIS

**Forecast lab.** Web route `#/lab`; the Models screen `#/models` lists the served releases and opens the model card;
`web/src/views/LabModels.tsx` runs a stored run or a typed scenario. **Model card:** `GET /api/v1/model-cards` (fields
from `served.json`; card document `docs/wsp/cards/acme_committee.md`). **MCP:** `models_inspect`, `models_forecast`.

Stored runs work (`stored_runs: true`): a stored CFI-G run supplies the conditions and answers without the planned
stop. A typed scenario:

```http
POST /api/v1/models/sha256:4cc2fa60cc8c40b316594034ab8f299946117705736677e941c99f645fb89436/scenarios
```

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:4cc2fa60cc8c40b316594034ab8f299946117705736677e941c99f645fb89436",
  "cutoff_s": 1.0,
  "conditions": {
    "fuel_species": "propane",
    "injected_mass_flow": 1e-06,
    "pressure": 101325.0,
    "tad": 2300.0,
    "x_fuel_role": 0.3,
    "x_o2_role": 0.3,
    "zst": 0.25
  },
  "controls": [],
  "observations": [],
  "test_date": "2023-03-01",
  "phase_established_at_s": 0.0
}
```

The values are the served examples (typed, not measurements). The date 2023-03-01 lies after
`era.first_late_day`, so the late-era members (455 °C limit) answer. Add `"diluent_species": "helium"`, `"burner_limit_c"`
or `"programmed_stop"` as conditions where they are known; give `x_o2_role_b`, `tad_b`, `zst_b` together or not at
all. A blend (`fuel_blend_x`, or a `fuel_species` with `+`) is refused at admission. The response returns
`input_hash`; then

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:4cc2fa60cc8c40b316594034ab8f299946117705736677e941c99f645fb89436",
  "input": {"kind": "prefix", "input_hash": "<input_hash>"},
  "settings": {"times_s": [6, 10, 20, 30, 60, 120]}
}
```

to `POST /api/v1/analyses` (or `"input": {"kind": "stored_run", "run_id": "<CFI-G run>", "cutoff_s": 1.0}`). It
returns a job: poll `GET /api/v1/jobs/<job_id>`, then read `GET /api/v1/artifacts/<artifact_id>`, an
`AnalysisArtifact` of evidence class `model_output`. This release has one target, the event table (rows
`active_supported`, `extinction`, `operator_termination`, `unresolved_support` at each time), so `settings.target`
is not needed and `quantile_levels` do not apply; without `times_s` the forward uses `horizon_s` (default 60 s) and
`n_points` (default 41).

Cite: acme_normal.wsp13 F4 (f4-hydrogen), hand-written (literal WSP) release sha256:4cc2fa60…9436, experimental, not trained; its methane, helium and hydrogen parts were typed after the final run with every partition seen; event probabilities are model output, not a measurement.

## Sources

IGNIS (`D:\dev\ignis-web`, branch harness-docs-20261115):
- `configs/models/served/served.json`; `configs/models/served/models/4cc2fa60cc8c40b316594034ab8f299946117705736677e941c99f645fb89436/release.json`
- `configs/models/acme/wsp13/normal/f4-hydrogen/{edits.wsp,meta.json,release.json}`, `f4-helium/`, `f4-methane/`, `f3-s1/`, `f3-s2/`, `carry-f2/`, `baseline-set/` (edits.wsp, meta.json, release.json), `binding.json`, `spec.json`
- `configs/models/acme/wsp9/normal/f2-d4/meta.json`, `wsp10/normal/f2-d3-early/meta.json`, `wsp10/normal/f2-d3-late/meta.json`, `wsp11/normal/f2-d1-early/meta.json`, `wsp12/normal/f2-fable/{meta.json,release.json}`
- `crates/ignis-wsp/src/committee4.rs`, `committee3.rs`, `committee2.rs`, `flame.rs`, `flame3.rs`, `flame4.rs`, `flame6.rs`, `flame8.rs`, `flame9.rs`, `flame10.rs`, `fuel.rs`, `fuel2.rs`, `common.rs`, `models.rs`
- `crates/ignis-wsp-serve/src/lib.rs`, `crates/ignis-wsp-serve/tests/served_ids.rs`, `tools/wsp/verify_served.py`
- `docs/wsp/cards/acme_committee.md`

Research (`SpaceApps-Ardealu`, branch feat/handwritten-science, 542dad8):
- `SpaceApps-Ardealu:docs/wsp/research/phys_sflame_f4.md`, `phys_sflame_post_final.md`, `phys_sflame_f3.md` (excerpts), `phys_ethylene_held.md` (excerpts), `phys_spherical_fuels.md` (excerpts)
- `SpaceApps-Ardealu:docs/wsp/feedback/sflame_f4_result.md`, `sflame_f3_result.md`, `sflame_post_final_result.md`, `acme_planned_stop_v2.md` (excerpt)
- `SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_SFLAME.md`, `RESULTS_GEN6.md`, `RESULTS_FINAL.md` (excerpts), `COVERAGE_19.md`, `README.md`, `PROTOCOL.md` (12.10, 13.6 to 13.8)
- `SpaceApps-Ardealu:docs/wsp/dossier/04_decisions.md`, `dossier/06_models.md` (6.4 to 6.8), `dossier/11_overnight_20261115.md` (excerpt), `cards/acme_committee.md`
- Data pack `scratchpad/packs/acme_normal.wsp13.md` (served entry, release fields, params.npz tensors)
