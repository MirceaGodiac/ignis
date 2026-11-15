# acme_inverse.wsp12 — inverse burner flames, committee F2

`acme_inverse.wsp12` F2 forecasts how an inverse (oxidizer into fuel) spherical burner flame ends: the probability
over time that it goes out by itself (extinction) or is stopped (flow termination), for ethane, propane, butane and
ethylene. On the sealed CFI-G test partition, read once in the final run of 15 November, its extinction score is
**0.0998** on 93 cases in 3 sessions (termination 0.1103); on the 69 cases the fitted baselines also answer it
scores 0.1045 against 0.2233 for the constant baseline and 0.1645 for the fitted hazard
(`docs/wsp/RESULTS_FINAL.md`). It is data-informed (CFI-G train data and published theory); its
ethylene row was typed zero-shot and confirmed by a blind run. Release
`sha256:7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7`.

| | |
|---|---|
| Model | `acme_inverse.wsp12` |
| Investigation | ACME CFI-G (PSI-159) and Flame Design (PSI-10), inverse burner flames |
| Family | `acme_inverse` (assemble key `acme_inverse12`) |
| Candidate | F2 (`f2-fable`) |
| Generation | sixth (PROTOCOL 12.10, 15 November 2026) |
| Author | `claude-fable-5-2` (Anthropic); slot values copied by `tools/wsp/committee3_edits.py` |
| Authorship | data-informed (ethylene row zero-shot) |
| Release | `sha256:7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7` |
| Forward | `ignis_wsp::committee3::forecast`, `crates/ignis-wsp/src/committee3.rs` (forward version 12), members on `flame5.rs`, `flame6.rs`, `flame7.rs`, fuel table in `fuel.rs` |
| Parent release | `sha256:d33a625b66980212a91a42323d727973f5bd7a830c68de3216782de60ccd9634` (`baseline-set`, the lineage root of the spec) |

## 1. Task and targets

Inputs (from the served entry, `configs/models/served/served.json`; examples are typed values, not measurements):

| Name | Kind | SI unit | Required | Note | Example |
|---|---|---|---|---|---|
| `fuel_species` | condition (text) | 1 | yes | the fuel of the ambient; the release admits ethane, propane, butane and ethylene (see Scope) | `propane` |
| `injected_mass_flow` | condition | kg/s | yes | injected (oxidizer-side) mass flow | 5e-06 |
| `pressure` | condition | Pa | yes | chamber pressure | 101325 |
| `tad` | condition | K | yes | adiabatic flame temperature | 2400 |
| `x_fuel_role` | condition | 1 | yes | fuel mole fraction of the fuel stream (the ambient) | 0.1 |
| `x_o2_role` | condition | 1 | yes | oxygen mole fraction of the oxidizer stream | 0.5 |
| `zst` | condition | 1 | yes | stoichiometric mixture fraction | 0.45 |
| `phase_established_at_s` | phase start | s | yes | time the flame was established; the origin is hot ignition, so 0 for a flame burning from ignition | 0 |
| `programmed_stop` | condition | s | no | planned stop from NASA's script file; without it the release answers as on the shared records | – |
| `session_id` | test date | 1 | no | date of the test (era weights; this release weighs both eras alike) | `2023-03-01` |

Target: the cause-specific end of the established hot flame, as a partition of four rows at each requested time:
`active_supported`, `extinction`, `operator_termination`, `unresolved_support`. An operator or script stop is its own
cause, never an extinction; extinction is the end of the hot flame. Conditional on the flame burning at the cutoff;
horizon 600 s. Time origin: hot ignition, the table's "burn time (sec)".

A stored CFI-G run supplies the conditions and answers without the planned stop. A typed scenario needs the seven
required conditions and the time the flame was established.

## 2. Physical mechanism

A committee of ten slots (`committee3.rs`, rules of `committee2.rs`): the forecast is the weighted mean of the member
event tables,

$$P(t)=\sum_k o_k\sum_i w_i\,P_{i,k}(t),$$

with the pair `mix.w_early`/`mix.w_late` weighing the slots on the shared records, the pair `mix.ws_early`/`mix.ws_late`
when the planned stop is given, and then three passes with a scheduled stop at `programmed_stop + script.offset_s[k]`
weighed by `script.offset_w`. Only the four growth-clock members carry weight: the carried law of the inverse flames
(weight 2) and the same law with a hold scenario (weight 1).

Each member is the growth-clock forward with a typed fuel table (`flame6.rs`, laws of `flame3.rs`). For an inverse
flame the burner-side reactant is oxygen,

$$r_{ss}=\frac{\dot m_{tot}}{4\pi\rho D\,\ln(1/Z_{st})},\qquad r_{ext}=\sqrt{\frac{\dot m_{O_2}}{4\pi\dot m''_{cr}}},\qquad \Phi=(r_{ss}/r_{ext})^2 .$$

The critical flux is typed so low (`se.flux.b0` = −4.6) that lean flames do not reach $r_{ext}$ and the growth clock
does not act: with the published inverse constants it gives lean flames minutes (decision D-23,
`dossier/04_decisions.md`). What ends a lean inverse flame is an oscillatory instability once the oxygen flux at the
sheet falls to about 1.0 g m⁻² s⁻¹, set by the diffusion of ambient fuel (`phys_acme_inverse.md` 1.1, 1.4). The
model carries it as an empirical oscillation and quench clock,

$$S_i(t)=G_i(t')\;\frac{1}{1+(t'/\tau_{osc})^{k}}\;e^{-h t'},\qquad \ln\tau_{osc}=N^*_{se}(\text{se.osc};c)+\text{fuel.dosc},\quad t'=\max(t-\text{floor},0),$$

with a weak class at ignition $p_{weak}=\sigma(N_{flash}(c))$ (air injected into a lean ambient). The hold scenario
(slots b1, b3) adds one hidden unit to $N_{osc}$, $20\,\mathrm{relu}(c_{logm}-0.6\,c_{zst}-0.2)$: in the marginal
band of the mixture fraction a flame at a high oxygen flow is held in a limit cycle (`acme_inverse.wsp3` D5). The
burner warms as

$$T_b(t)=T_0+\Delta T\,(1-e^{-t/\tau_b}),\qquad \ln\Delta T_j=N^*_{ft}(\text{bt.rise};b)+\sigma_{bt}z_j,$$

and trips at the weighted limits 455 and 510 °C (`bt.*`); programmed stops are atoms at the campaign's script
times (`ft.atom_*`), switched off in b2, b3, which answer when the planned stop is known.

The fuel enters through the table (`fuel.rs`): every law network is evaluated at the alkane vertices E (1, 0),
P (0, 0), B (0, 1) and weighed by the fuel's coordinates, $N^*(x)=\sum_v w_vN(x_v)$, $(w_E,w_P,w_B)=(e,1-e-b,b)$.
Ethylene is (1, 0) in the self-extinction laws, (0.5, 0) in the burner laws, and its quench clock is doubled
(`fuel.dosc` = 0.7, $e^{0.7}\approx2.0$): a lean ethylene flame, with a Lewis number below ethane's, should not
pulsate and ends at the published critical flux of 0.68 g O₂ m⁻² s⁻¹ (`phys_spherical_fuels.md` 3.4).

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section acme_inverse.wsp12.

## 3. Authoring

**Generation and step.** Sixth generation, PROTOCOL 12.10 (15 November 2026): one revision of F1 from the train days,
judged once on CFI-G validation and the Flame Design validation days. The record was created at
2026-11-15T03:55:03Z.

**What was read** (`release.json` `construction.exposure`): the PSI-159 inverse-flame table train rows and TP control
timelines of train tests; train-only audits by delegated agents (`acme_d1_train_audit.md`, `acme_transfer_audit.md`,
rechecked by the author); a covariate-shift map and the carried models' forecasts on validation conditions (inputs
and model outputs only); `lit_spherical_flames.md` (published theory); aggregate validation tables of earlier
candidates; raw channels and script files of train tests (`acme_raw_signals_train.md`); the as-run script parameter
files read into a private data root as `programmed_stop` for train and validation runs (counts only);
`phys_spherical_fuels.md`; `acme_planned_stop_v2.md`. Datasets: train (`acme_inverse-747032c921c8c8f2`) and
validation as aggregate scores and run conditions, no per-run outcome. The test partition was not read.

**Dossiers.** `phys_acme_inverse.md` (the inverse physics, fifth generation), `phys_spherical_fuels.md` (the fuel
table), `acme_planned_stop_v2.md` (the planned stop).

**Decisions.**

- **One change from F1** (meta.json rationale): the planned stop is the one computed from the script file alone by
  `tools/wsp/ext/acme_script2.py` (`acme_planned_stop_v2.md`: stop = D + S + F − G), read at −1, 0 and +1 s with
  weights 0.25, 0.5, 0.25. F1 and P1 read the old stage sum at a single node, −0.4 s. "The planned stop from the
  script file lands on the recorded stop ... Three readings a second apart, centred on it" (`edits.wsp`). Without the
  planned stop F2 returns the forecasts of F1.
- **Members and fuel table unchanged**: "the fuel table did what its dossier expected on the train days of Flame
  Design and is not touched".
- **Weights 2 : 1**, as in P1 and F1: one third is the author's probability that a marginal flame at a high oxygen
  flow is held; train shows it twice in two flames at 9 mg/s (`cards/acme_committee.md`, D-44). No era is told apart
  (identical early and late vectors).

## 4. Weights

Totals (`release.json` `architecture.parameter_counts`): **4564 authored elements, 495 nonzero; 168 constants**
(the `norm.*`, `num.*` and `phys.*` scalars of each slot), 4732 elements in all. Per slot (elements / nonzero,
constants included): a0, a1 419 / 41; b0, b2 572 / 90; b1, b3 572 / 94; c0 to c3 389 / 49; committee lines 50 / 17.
Full list: `ignis wsp inspect --release sha256:7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7 --edits`,
or `configs/models/acme/wsp12/inverse/f2-fable/edits.wsp` (it writes only the statements that differ from the
parent's slots; the rest are the parent's, `baseline-set/edits.wsp`).

**Where the member values come from** (copies of earlier releases):

| Slots | Member kind | Copied from | Origin of the laws | Weight |
|---|---|---|---|---|
| a0, a1 | hinge laws, `flame5.rs` | baseline set (neutral clock: `se.tau.b0` 3.4, weak class and stops at −40) | lineage root, no train number | 0 |
| b0 | growth clock, `flame6.rs` | `acme_inverse.wsp10` f1-d1-fable `sha256:748af0b0…` | laws of `acme_inverse.wsp3` D1 `sha256:a43fdbcf…c1de`, the carried law | 2 (without stop) |
| b1 | growth clock | f1-d5-fable `sha256:f151ebce…` | `acme_inverse.wsp3` D5 `sha256:bf575e19…d45f`: D1 plus the hold scenario | 1 (without stop) |
| b2, b3 | growth clock | f1-d1-nostops `sha256:359402bb…`, f1-d5-nostops `sha256:e7a14c3b…` | the same two with guessed stops off | 2, 1 (with stop) |
| c0 to c3 | hinge with heating law, `flame7.rs` | baseline set (neutral) | lineage root | 0 |

Each b slot's fuel table is the F1 inverse table of `phys_spherical_fuels.md` 3.4 (zero-shot for ethylene).

| Group (tensor pattern) | Shape / count | Value(s) | Physical meaning |
|---|---|---|---|
| `mix.w_early`, `mix.w_late` | [10] each, 2 nonzero | b0 = 2, b1 = 1 (both eras) | carried law and hold scenario, 2 : 1, on the shared records |
| `mix.ws_early`, `mix.ws_late` | [10] each, 2 nonzero | b2 = 2, b3 = 1 | the same pair without guessed stops, with the planned stop |
| `era.last_early_day`, `era.first_late_day`, `era.p_late_between`, `era.p_late_unknown` | 4 scalars | 1900001, 2100366, 0.5, 0.5 | no era told apart (the vectors are equal) |
| `script.offset_s`, `script.offset_w` | [3] + [3] | −1, 0, +1 s; 0.25, 0.5, 0.25 | three readings of the computed planned stop (the F2 change) |
| `<b>.se.osc.*`, `se.osc_shape` | b0, skip 2, 3 or 4 hidden units | b0 2.17 (median about 8.8 s at the reference); skip 0.45 (c_o2), 3.0 (c_xf); units 10 relu(c_xf − 0.3), −8 relu(−c_logm − 1.0), 8 relu(−c_o2 − 0.6); b1, b3 also 20 relu(c_logm − 0.6 c_zst − 0.2); shape 3.0 | oscillation and quench clock of lean flames; the extra unit is the hold scenario |
| `<b>.se.flash.*`, `se.flash_tau_s`, `se.flash_shape` | b0, skip, 1 unit | b0 −5.33; skip c_tad −5.0; −10 relu(c_xf − 0.2); 1.5 s; 3.0 | weak class at ignition |
| `<b>.se.flux.b0`, `se.diff.b0`, `se.rho_d_g_m_s` | 3 scalars | −4.6; 3.332; 0.0334 g m⁻¹ s⁻¹ | growth clock in effect off for lean inverse flames (D-23); ρD of the quasi-steady radius |
| `<b>.se.sigma_phi`, `se.shape`, `se.floor_s`, `se.hold_log_rate` | 4 scalars | 0.3; 4.0; 2 s; −8.0 | spread of ln Φ; clock shape; floor; hazard of a held flame |
| `<b>.bt.rise.*` | b0, skip [7] 3 | b0 5.88; skip −0.4 (b_rss), −0.4 (b_mtot), +0.3 (f_ethane) | burner heating law, ΔT ∝ m_tot^−0.4 r_ss^−0.4 (D-24); ethane heats more |
| `<b>.bt.sigma`, `t0_c`, `tau_s`, `shape` | 4 scalars | 0.25; 30 °C; 30 s; 8.0 | spread; start; time constant; trip sharpness |
| `<b>.bt.limit_c`, `bt.limit_w` | [3] + [3] | 455 and 510 °C; 0.4 and 0.6 | the two campaign limit settings |
| `<b>.ft.atom_t_s`, `ft.atom_logit` | [8] each | 63, 64, 65, 66, 67, 90 s; logits −2.94, −2.59, −2.2, 0.49, −1.73, −1.39 (b0, b1), −40 (b2, b3) | programmed stops of the campaign (the 66 s cluster), off when the plan is known |
| `<b>.ft.log_base` | 1 per slot | −5.5 (b0, b1), −9.0 (b2, b3) | background stop hazard |
| `<b>.fuel.admit` | [6] | 1, 1, 1, 1, 0, 0 | ethane, propane, butane, ethylene admitted; methane and hydrogen refused |
| `<b>.fuel.mw_g_mol` | [6] | 30.07, 44.1, 58.12, 28.054, 16.043, 2.016 | molar masses (not read by the inverse laws) |
| `<b>.fuel.se_coord`, `fuel.ft_coord` | [6,2] each | se: (1,0), (0,0), (0,1), ethylene (1,0); ft: (1,0), (0,0), (0,1), ethylene (0.5,0) | fuel coordinates; ethylene's burner +0.15 is half of ethane's +0.3 |
| `<b>.fuel.dosc` | [6] | ethylene 0.7 | quench clock of ethylene doubled |
| `<b>.fuel.s`, `fuel.tad_min_k` | [6] each | all 0 | no oxygen-demand shift (inverse laws do not read it); no temperature bound |
| `<a,c>.*` | 43 and 54 tensors per slot | neutral: `se.tau.b0` 3.4, `se.flash.b0` −40, `ft.log_base` −40, `ft.tau.b0` 4.6 (a), `bt.rise.b0` 5.0 (c); alkane-only fuel table | baseline-set placeholders, weight 0, not run |
| constants `*.norm.*`, `*.num.*`, `*.phys.*` | 168 | ṁ 5e-6 kg/s, p 101325 Pa, x_O2 0.5 ± 0.25, x_F 0.1, T_ad 2200 ± 300 K, Z_st 0.45 ± 0.2, m_tot 3e-6 kg/s, r 25 mm; step 0.1 s, horizon 600 s, 15 Φ nodes, 7 heating nodes; M_N2 28.013, M_O2 31.998 | not authored |

## 5. Forward and assembly

| | |
|---|---|
| Forward | `crates/ignis-wsp/src/committee3.rs`, `ignis_wsp::committee3::forecast`, version 12, profile `strict_no_fitting` |
| forward_hash | `sha256:0f46fde758dbcd9b22990658c2b894f37194419e278706f821c77756fff8c8de` |
| code_version | `ignis-wsp@0.1.0+fwd.0f46fde758dbcd9b` |
| Spec | `acme_inverse-wsp-v12`, `sha256:5aa7fde8e18c90e767d6fb7cf2cd8c32705aacbdad005ce0ee2808355e6dbc7b` |
| Patch | `acme_inverse-wsp-v12-f2-fable`, `sha256:4eba4ee2388581c54ec5be432003457f928f973699e072935d2b19472deaedfa` |
| weights_hash | `sha256:8bcb457f769f577333b38f962b530f6c3746e2e49ee714af9168538eca17c8e8` |
| Binding | cohort `sha256:ed3e9beb717728fc4d371bccfc7896a940d38deea28502728be977f9fb79e45f`, split `sha256:747032c921c8c8f2b976fceb6bda8c90e4c5e18c761fdbd8dafeb5bc44e2b598`, source manifest `sha256:11a1bdbd…e126` (`configs/models/acme/wsp12/inverse/binding.json`) |
| Candidate | `configs/models/acme/wsp12/inverse/f2-fable` (edits.wsp, header.wsp, meta.json, patch.json, release.json) |
| Checks in the record | spec_valid, patch_valid, tensors_complete, strict_forward_audit, finite_forward_on_fixture: passed |

Rebuild (the parent record is committed at `configs/models/acme/wsp12/inverse/baseline-set/release.json`; the root
must hold it as `models/d33a625b…/`):

```
cargo build --release -p ignis-cli
ignis wsp compile  --family acme_inverse12 --edits configs/models/acme/wsp12/inverse/f2-fable/edits.wsp --meta configs/models/acme/wsp12/inverse/f2-fable/meta.json --out configs/models/acme/wsp12/inverse/f2-fable/patch.json
ignis wsp assemble --family acme_inverse12 --patch configs/models/acme/wsp12/inverse/f2-fable/patch.json --binding configs/models/acme/wsp12/inverse/binding.json --root <registry holding models/d33a625b66980212a91a42323d727973f5bd7a830c68de3216782de60ccd9634/> --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents <registry holding the parents>
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release and compares the record; `served_ids` pins the 14 served ids.
`cargo test -p ignis-wsp-serve --test served_ids` passed (2 of 2) on 15 November 2026 at a58c7c3: the
served id is the content hash of its record and loads through the registry checks.

## 6. Evaluation

Metric: integrated Brier score, 6 to 120 s, case means; extinction / flow termination; lower is better.

**Headline: CFI-G sealed test partition** (final run of PROTOCOL 12.11, `docs/wsp/RESULTS_FINAL.md`,
release list `a46df540…8415`; held out: no release of the generation was written or chosen from it). 98 cases of 34
runs; F2, P1 and the baseline set refused 5 (condition outside the domain).

| Release | Input | Cases | Extinction | Termination | Sum |
|---|---|---|---|---|---|
| **F2** | shared records | 93 | **0.0998** | 0.1103 | 0.2100 |
| **F2** | with the planned stop | 93 | 0.1045 | 0.0793 | 0.1838 |
| `acme_inverse.wsp5` P1 | shared | 93 | 0.0998 | 0.1103 | 0.2100 (identical forecasts) |
| baseline set of the spec | either | 93 | 0.2222 | 0.2325 | 0.4547 |
| F2, on the cases the fitted baselines answer | shared | 69 | **0.1045** | 0.0752 | 0.1797 |
| hazard (fitted) | shared | 69 | 0.1645 | 0.0817 | 0.2462 |
| linear (fitted) | shared | 69 | 0.1744 | 0.0809 | 0.2553 |
| piecewise (fitted) | shared | 69 | 0.2270 | 0.1385 | 0.3655 |
| constant (fitted) | shared | 69 | 0.2233 | 0.1562 | 0.3796 |

Paired difference F2 minus comparator (3 sessions): constant −0.1286 [−0.1849, −0.1003] / −0.0865 [−0.1168, −0.0685];
hazard −0.0553 [−0.1181, −0.0086] / −0.0044 [−0.0149, +0.0093]; baseline set with the stop −0.1224 [−0.1876, −0.0034]
/ −0.1445 [−0.3809, −0.0186].

**Flame Design test day** (same run, step 2; 20 ethylene tests on 1 day, 60 cases, all ended by operator
termination, so extinction measures only false extinction):

| Path | F2 | F1 | Ethylene as ethane | Incidence baseline |
|---|---|---|---|---|
| without the planned stop | 0.0003 / 0.2743 | 0.0003 / 0.2743 | 0.0004 / 0.2616 | 0.2532 / 0.4234 |
| with the planned stop | **0.0001 / 0.0020** | 0.0000 / 0.0398 | 0.0000 / 0.0398 | 0.2532 / 0.4234 |

**Development numbers** (`RESULTS_GEN6.md` 12.8 and 12.10; aggregates of these partitions had been read before):

| Set | Path | F2 | F1 | Earlier |
|---|---|---|---|---|
| CFI-G validation, 54 cases, 2 sessions | shared records | 0.1288 / 0.0880 | identical | `acme_inverse.wsp3` D1 0.1606 / 0.0939 (`cards/acme_committee.md`) |
| CFI-G validation | planned stop | 0.1349 / 0.0480 | 0.1348 / 0.0478 (old stop input) | – |
| Flame Design validation days, 75 cases | without the stop | 0.0871 / 0.1959 | identical | – |
| Flame Design validation days | planned stop | **0.0920 / 0.0018** | 0.0921 / 0.0240 | – |
| decision score (mean of the sums), without / with | | 0.2499 / **0.1384** | 0.2499 / 0.1493 | – |
| CFI-G train, 204 cases | shared records | 0.0271 / 0.0608 (F1 = P1, `flame_design_fuel_table_result.md`) | | D1 0.0357 / 0.0639 |

**Ethylene, the zero-shot row, blind** (12.8, `feedback/flame_design_fuel_table_result.md`; Flame Design, 236
inverse cases, 7 sessions, extinction without the planned stop; F2 forecasts equal F1's on this path): ethylene as
itself **0.1077** against 0.1466 as ethane, difference −0.0388 [−0.0658, −0.0101] over sessions: "adds something".

## 7. Adoption decision and lineage

**Why this release is served.** By the rule of PROTOCOL 12.10, F2 replaces F1 where its decision score (the mean
over CFI-G validation and the Flame Design validation days of extinction plus termination) and its extinction mean
are not higher on either path: 0.2499 / 0.1384 against 0.2499 / 0.1493, extinction 0.1080 / 0.1134 for both
(`RESULTS_GEN6.md`). It was then measured once on the held test data (`RESULTS_FINAL.md`). The served list keeps it
as the inverse-flame release (`COVERAGE_19.md`: "equal to the model it replaced on ethane, propane and butane").

**Lineage.**

| Step | Release | What it changed |
|---|---|---|
| fourth generation | `acme_inverse.wsp3` D1 `sha256:a43fdbcf5c73206a5d9d442a182d5e7e5c4a407d53e64b44231c014cbb00c1de` | growth-clock forward; for lean inverse flames the growth law off and an empirical early-quench clock (D-23); adopted (0.137 against 0.146) |
| fifth generation | `acme_inverse.wsp3` D5 `sha256:bf575e191a2d00dda73062a763b22e7b3af6ee9bf6a865b7a73e525e1e67d45f` | D1 plus the hold scenario (four statements) |
| fifth generation | `acme_inverse.wsp5` P1 `sha256:aa05ddcbe976dbb1a5537987d05270b28066551c39ab24db671be65692dcea0e` | committee D1 : D5 = 2 : 1, stop variants, offset 0.4 s; adopted (validation extinction 0.129 against 0.161, D-44) |
| sixth generation | `acme_inverse.wsp10` F1 members (`748af0b0…`, `f151ebce…`, `359402bb…`, `e7a14c3b…`) | each P1 member reading the typed fuel table; zero-shot ethylene row |
| sixth generation, root | `baseline-set` `sha256:d33a625b66980212a91a42323d727973f5bd7a830c68de3216782de60ccd9634` | lineage root of the third committee spec (Opus copy, no number from the PSI-159 tables) |
| sixth generation | `acme_inverse.wsp12` F1 `sha256:e2fb1fccf8c688f4ac357db7ad0abdc283dfed894a6a3f66f23e2b75ca1b2b2f` (parent: root) | P1 with the fuel table in every member; answers ethylene |
| sixth generation | **F2** (this release; parent: root) | F1 with the planned stop from the script file at −1, 0, +1 s |

F2 is assembled as a literal revision of the root, not of F1, so that the registry records it with its development
exposure (`baseline-set/meta.json`); its slot values are F1's. Not adopted along the way: the second-generation lean
revision of the inverse hinge law (0.146 → 0.156, vetoed, D-22), the equal-weight blend of two laws (0.142 against
0.137, D-25), the 50 : 50 high-flow hold scenario of D-26 (rewritten as D5).

**Scope** (served `limits`, condensed, with what the release itself holds):

- Two or three validation days per family on Flame Design are few; the Flame Design inverse test day is one day,
  and all its cases ended by a flow stop.
- The fuel table admits ethane, propane, butane and ethylene only; methane and hydrogen are refused at the condition
  gate (`fuel.admit` 0).
- The served list states that ethylene below 1900 K is refused; this release's `fuel.tad_min_k` is 0 in every row, so
  its forward sets no such bound for inverse flames.
- The endpoint method misplaces the flow change on tests with an ignition-to-test ramp.
- Also covers Flame Design (PSI-10): ethylene as itself 0.108 against 0.147 as ethane (`COVERAGE_19.md`).

## 8. How to call it in IGNIS

**Forecast lab.** Web route `#/lab`; the Models screen `#/models` lists the served releases and opens the model card;
`web/src/views/LabModels.tsx` runs a stored run or a typed scenario. **Model card:** `GET /api/v1/model-cards` (fields
from `served.json`; card document `docs/wsp/cards/acme_committee.md`). **MCP:** `models_inspect`, `models_forecast`.

Stored runs work (`stored_runs: true`): a stored CFI-G run supplies the conditions and answers without the planned
stop. A typed scenario:

```http
POST /api/v1/models/sha256:7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7/scenarios
```

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7",
  "cutoff_s": 1.0,
  "conditions": {
    "fuel_species": "propane",
    "injected_mass_flow": 5e-06,
    "pressure": 101325.0,
    "tad": 2400.0,
    "x_fuel_role": 0.1,
    "x_o2_role": 0.5,
    "zst": 0.45
  },
  "controls": [],
  "observations": [],
  "test_date": "2023-03-01",
  "phase_established_at_s": 0.0
}
```

The values are the served examples (typed, not measurements); the date does not change the forecast, since both
eras carry the same weights. Add `"programmed_stop"` (s) where the script's planned stop is known. The response
returns `input_hash`; then

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7",
  "input": {"kind": "prefix", "input_hash": "<input_hash>"},
  "settings": {"times_s": [6, 10, 20, 30, 60, 120]}
}
```

to `POST /api/v1/analyses` (or `"input": {"kind": "stored_run", "run_id": "<CFI-G run>", "cutoff_s": 1.0}`). It
returns a job: poll `GET /api/v1/jobs/<job_id>`, then read `GET /api/v1/artifacts/<artifact_id>`, an
`AnalysisArtifact` of evidence class `model_output`. The release has one target, the event table (rows
`active_supported`, `extinction`, `operator_termination`, `unresolved_support`), so `settings.target` is not needed
and `quantile_levels` do not apply; without `times_s` the forward uses `horizon_s` (default 60 s) and `n_points`
(default 41).

Cite: acme_inverse.wsp12 F2, hand-written (literal WSP) release sha256:7404f3ad…a7f7, experimental, not trained; event probabilities are model output, not a measurement.

## Sources

IGNIS (`D:\dev\ignis-web`, branch harness-docs-20261115):
- `configs/models/served/served.json`; `configs/models/served/models/7404f3ad888f8c01d38ce4267588f5ea4532343dd23d491eb21097ae304aa7f7/release.json`
- `configs/models/acme/wsp12/inverse/f2-fable/{edits.wsp,header.wsp,meta.json,release.json}`, `f1-fable/{edits.wsp,meta.json,release.json}`, `baseline-set/{edits.wsp,meta.json,release.json}`, `binding.json`, `spec.json`
- `configs/models/acme/wsp10/inverse/f1-d1-fable/meta.json`, `f1-d5-fable/meta.json`, and the release records of `wsp3/inverse/*`, `wsp5/inverse/p1-fable`, `wsp10/inverse/*`
- `crates/ignis-wsp/src/committee3.rs`, `committee2.rs`, `flame3.rs`, `flame6.rs`, `fuel.rs`, `common.rs`, `models.rs`
- `crates/ignis-wsp-serve/src/lib.rs`, `crates/ignis-wsp-serve/tests/served_ids.rs`, `tools/wsp/verify_served.py`
- `docs/wsp/cards/acme_committee.md`
