# bass5.wsp — kept burning and quench speed of plastics and fabrics (BASS, BASS-II)

`bass5.wsp` answers, for one sample in the slow duct flow of the Microgravity Science Glovebox (PMMA
rods, spheres, sections, films and slabs; SIBAL, Nomex and Ultem fabrics and sheets; BASS PSI-26 and
BASS-II PSI-25), whether the flame keeps burning, and the duct speed below which it goes out. Its
"kept burning" answer is identical to `bass4`, whose result on the unread test days of the final run
holds: **Brier 0.048 against 0.071 for the train incidence over 42 burns, fabrics included**
(SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md). Its quench speed, on the corrected coding, is
**0.114 / 0.108 / 0.126 cm/s against 0.671 / 0.547 / 0.916 for a constant** on train / validation /
test days (SpaceApps-Ardealu:docs/wsp/feedback/bass5_post_final_result.md): typed after the final
run with every partition seen; in-sample on all data; no held-out score exists. Data-informed,
written by `claude-fable-5-2`. Release
`sha256:050a4ceb6c094b9256c5568408f8574508530ce1b31ce45eae43e8785fa4b82d`.

| | |
|---|---|
| Model | `bass5.wsp` |
| Investigation | BASS and BASS-II plastics and fabrics (PSI-26, PSI-25): does the sample keep burning, and the quench speed |
| Family | `other` (generic law slug `bass5`) |
| Candidate | S1 (`configs/models/law/bass5/s1-fable`) |
| Generation | seventh (15 November 2026), PROTOCOL 13.8 (the one plastics revision after the final run) |
| Author | `claude-fable-5-2` (Anthropic); transcribed by the assembly engineer `claude-opus-5-5`, no number chosen |
| Authorship | data-informed; typed after the final run with every partition seen (in-sample on all data) |
| Release | `sha256:050a4ceb6c094b9256c5568408f8574508530ce1b31ce45eae43e8785fa4b82d` |
| Forward | `ignis_wsp::law::forecast`, `crates/ignis-wsp/src/law.rs` (version 1, profile `strict_no_fitting`); slug registered in `crates/ignis-wsp/src/law_slugs.rs` |
| Parent release | `sha256:1558a1c15d26b900b459ba473bc0f0c42ed332c22c896777b3af3d636643cf52` (neutral root of slug `bass5`); the numbers are every line of `bass4` S1 `sha256:ceefc15d4cb8cf658772ec3603c9dabd6565fb3db6528bb35c2205f0d3bdab86` plus the post-final block |

## 1. Task and targets

Inputs (all typed conditions, all required; `configs/models/served/served.json`, declaration
`configs/models/law/bass5/decl.json`; derived by `tools/wsp/ext/law_inputs.py` from condition fields
only). The domain is the declared domain of the hand-written law (`x.<feature>.lo`, `.hi`), not a
training-support region; a value outside it is refused with its name.

| Name | Kind | SI unit | Transform, centre | Domain | Note |
|---|---|---|---|---|---|
| `x_o2` | condition | 1 | ln, 0.18 | 0.12 to 0.25 | oxygen mole fraction of the gas entering the duct during the step |
| `flow_speed` | condition | m/s | ln, 0.01 | 0.003 to 0.6 | duct speed of the step |
| `nose_radius` | condition | m | ln, 0.01 | 0.002 to 0.03 | radius of the rod, sphere or spherical section as installed; 0.01 for a flat sample (weight zero in the flat targets) |
| `heat_time` | condition | s | ln, 200 | 20 to 4000 | time from ignition to the end of the step (for quench and blow-off speeds: to the event); 200 where not recorded |
| `is_concurrent` | condition | 1 | identity, 0 | 0 to 1 | flat samples: 1 if the flame spreads with the flow, 0 against it; 0 for rods and spheres |
| `is_rod` | condition | 1 | identity, 0 | 0 to 1 | 1 for a rod in any flow configuration |
| `flat_thickness` | condition | m | ln, 0.0005 | 0.00005 to 0.03 | flat samples: thickness as installed; rods, spheres, sections: 0.0005 |
| `u_pot1` | condition | m/s | ln, 0.044 | 0.01 to 0.15 | the burn's fan curve at fan display 46.5 (pot 1) |
| `is_one_sided` | condition | 1 | identity, 0 | 0 to 1 | flat samples: 1 if one side burns |
| `is_sibal` | condition | 1 | identity, 0 | 0 to 1 | 1 if the material is SIBAL fabric |
| `is_nomex_ultem` | condition | 1 | identity, 0 | 0 to 1 | 1 if Nomex (any) or an Ultem sheet |
| `flat_width` | condition | m | ln, 0.02 | 0.002 to 0.1 | PMMA films and slabs: width as installed; everything else, and a flat PMMA sample with no width recorded: 0.02 (new in bass5) |

Targets (`release.json`, `observation_model.targets`); all eight are served, chosen with
`settings.target`, default the first:

| Target | Kind | Unit | Meaning |
|---|---|---|---|
| `holds_low_flow` | binary | | rod tip or sphere: flame not quenched by heat loss at the end of the step |
| `holds_high_flow` | binary | | rod tip or sphere: stagnation flame not blown off |
| `oxygen_limit` | positive (quantiles) | 1 | rod tip or sphere: oxygen fraction at which the flame goes out at this speed and time |
| `quench_speed` | positive (quantiles) | m/s | rod tip or sphere: duct speed below which the flame goes out |
| `blowoff_speed` | positive (quantiles) | m/s | rod tip or sphere: duct speed at which the stagnation flame blows off |
| `flat_holds_low_flow` | binary | | film, slab or fabric: flame not quenched |
| `flat_holds_high_flow` | binary | | film, slab or fabric: flame not blown off |
| `flat_quench_speed` | positive (quantiles) | m/s | film or slab: duct speed below which the flame goes out |

The targets are not functions of time: the forecast is one answer for the stated step, and
`times_s` of the artifact holds the cutoff only (`law.rs`). There is no stored run that carries the
declared features (`stored_runs = false`), so every forecast is a typed scenario of conditions; no
controls or observations are read.

"Kept burning" as scored by the lane is the smaller of the two `holds` probabilities of the
sample's kind: `holds_low_flow` and `holds_high_flow` for rod tips, spheres and sections, the
`flat_` pair for films, slabs and fabrics (SpaceApps-Ardealu:docs/wsp/feedback/bass1_w0_result.md).

## 2. Physical mechanism

A flame on a solid in a slow stream lives between two limits in oxygen mole fraction $X$: on the
slow side the hot surface radiates and the cold interior of a thick solid soaks up heat while the
flame's supply falls with the stretch rate; on the fast side chemistry has no time
(SpaceApps-Ardealu:docs/wsp/research/phys_bass1_w0.md, section 0). Every target is one network of
the generic law forward (`law.rs`):

$$z_i = \ln(x_i/c_i)\ \text{or}\ (x_i - c_i),\qquad N(z) = b_0 + \mathrm{skip}\cdot z + \sum_{j<8} w_2[j]\,\mathrm{relu}\big(b_1[j] + W_1[j,:]\cdot z\big)$$

A binary target is $P = \mathrm{logistic}(N_\mathrm{logit})$; a positive target is a mixture of two
log-normals of one median $e^{N_\mathrm{median}}$, ln-sds $N_\mathrm{spread}$ and
$k\,N_\mathrm{spread}$, weights $1-w$ and $w$ (`tail_weight`, `tail_width`).

**Rod tips and spheres, kept burning** (`holds_low_flow.*`, `holds_high_flow.*`; W0 lines). The
thermal limit and the kinetic limit (a floor, then blow-off):

$$X_{th} = x_s + \frac{q_r + b_c/\sqrt{t}}{k_s\sqrt{a_e}},\quad a_e = \frac{1.5\,(U + D_c/R)}{R};\qquad X_k = x_b + s_b\,a_n,\quad a_n = 1.5\,\frac{U}{R}\Big(\frac{R}{r_0}\Big)^{0.25}$$

with $x_s = 0.030$, $q_r = 7.2$ kW/m², $b_c = 110$ kW m⁻² s^0.5, $k_s = 62$ kW m⁻² s^0.5,
$D_c = 0.88$ cm²/s, $x_b = 0.134$, $s_b = 0.00035$ s, $r_0 = 0.318$ cm. Typed as logits:

$$\ell_{low} = \min\big(28\ln(X/0.134),\ 6.3\,(\ln X - \ln X_{th})\big),\qquad \ln X_{th} = -1.750 - 0.25\,z_u + 0.583\,z_r - 0.208\,z_t$$

$$\ell_{high} = 28\,\big(\ln(X/0.134) - \ln(X_k/0.134)\big)\quad\text{(four chords at } a_n = 5, 25, 60, 150\ \mathrm{s^{-1}})$$

**Oxygen limit and blow-off speed** (`oxygen_limit.*`, `blowoff_speed.*`; W0 lines):
$X_\mathrm{lim} = \max(X_{th}, X_k)$ written as $\ln 0.134$ plus the excess of the thermal limit and
the blow-off rise, and

$$U_b = \frac{(X - 0.134)\,R^{0.75}\,r_0^{0.25}}{1.5\times 0.00035}$$

bent down towards $\ln(X - 0.134)$ by five knots at 19, 18, 17, 16 and 15 % O2.

**Quench speed of rods, spheres and sections** (`quench_speed.*`; bass3). Below about 4 cm/s the
recorded outcome is a fan setting, not a calibrated speed: rods, spheres, films and thin slabs go out
at pot 0.27 to 0.40 whatever the oxygen, diameter or configuration
(SpaceApps-Ardealu:docs/wsp/research/phys_bass_study.md, section 0). So

$$U_q = 0.941 \times 0.31 \times u_{pot1},\qquad \text{ln-sd } 0.15$$

where 0.941 is the lane's fan geometry and 0.31 the pot setting at which 22 development rods went
out. The W0 forced-plus-diffusive chords are still written in `quench_speed.median.W1` and `.b1`
but carry `w2 = 0`: they are inert.

**Films, slabs and fabrics, kept burning** (`flat_holds_*`; W0 lines, fabric shifts of bass4):

$$\ell_{low} = \min\big(20\ln(X/X_f),\ 4.5\,(\ln X - \ln X_{th})\big),\quad \ln X_{th} = -1.711 - 0.833\,z_u + 0.338\,\mathrm{concurrent}$$

$$X_k = 0.138 + 0.0016\,U\ (\mathrm{cm/s}),\ \text{half the slope concurrent}$$

The floor $X_f$ is 13.8 % for a PMMA film, moved to 17.0 % for SIBAL and 23.4 % for Nomex and Ultem
at low flow, and to 15.8 % and 21.5 % under the blow-off rise (`skip[9]`, `skip[10]`, `W1[0,9]`,
`W1[0,10]`; SpaceApps-Ardealu:docs/wsp/research/phys_bass_study2.md, sections A and E).

**Flat quench speed, two classes** (`flat_quench_speed.*`; bass3, width term bass5). A slab either
stays above its oxygen limit and burns down to the same fan floor as everything else, or it reaches
that limit during a long burn and dies at whatever flow it was held at, 8 to 18 cm/s:

$$\ln U_q = \mathrm{clamp}\big(\ln U_A + 30\,(z_c - z_x),\ \ln U_A,\ \ln U_B\big)$$

$$U_A = 0.941\times 0.39\,u_{pot1}\,(X/0.18)^{-0.6}\,1.5^{\mathrm{concurrent}},\qquad U_B = 11\ \mathrm{cm/s}\,(d/3\ \mathrm{mm})^{0.3}\,1.22^{\mathrm{one\text{-}sided}}$$

$$z_c = -0.204 + 0.147\ln\frac{d}{0.5\ \mathrm{mm}} + 0.28\,\mathrm{one\text{-}sided} - 0.11\ln\frac{W}{2\ \mathrm{cm}}$$

Unit 3 of the network carries $30\,(z_c - z_x)$ and unit 4 the cap $U_B$. The last term is the bass5
change: a slab is held along both long edges in a metal frame, and the heat conducted sideways into
it per unit of burning area is $4\lambda_s (T_s - T_\infty)\,d/W^2$, 11 kW/m² for 4 mm in a 1 cm frame
against 2.8 kW/m² in a 2 cm frame and 7.2 kW/m² radiated by the surface. A thick narrow slab
therefore needs more oxygen to keep spreading: its threshold is 1.079 times higher at half the width
(SpaceApps-Ardealu:docs/wsp/research/phys_bass_post_final.md, section 2). The size of the factor is
set between what the 1 cm slabs M10 and M11 need and what M8 allows; it is not derived.

Full derivations: [docs/research/FORMULAS.md](../../research/FORMULAS.md), section bass5.wsp.

## 3. Authoring

**Who.** `claude-fable-5-2` wrote the post-final dossier
SpaceApps-Ardealu:docs/wsp/research/phys_bass_post_final.md (sha256
`7135fe26bf992b15a3c7d47cbc1a9a5b547c4979e59d9ab7415a1fb3ff0352ce`); its Edits block (lines 130 to 142)
was copied verbatim to `runs/dev/gen7/study/bass5_block.wsp` and transcribed by the assembly engineer
(`claude-opus-5-5`) on top of every line of `bass4` S1: "no number chosen"
(`configs/models/law/bass5/s1-fable/meta.json`, rationale).

**When.** PROTOCOL 13.8, after the final run of 13.7, by the owner's decision: the test days of the
plastics investigations became study data, and one revision of the plastics quench speed was
allowed, no further rounds (SpaceApps-Ardealu:docs/wsp/PROTOCOL.md, section 13.8).

**What was read** (exposure in the release): the dossier's Edits; `bass5_block.wsp`; the plastics
test-day cases with their outcomes (`runs:transfer_bass1/cases_test_*_v2.jsonl`), read after the final
run as study data; train and validation cases (`bass1-train-and-validation-cases`); development
feedback: the 13.7 final-run scores, the test days as study data, and one what-if run of the width
candidate; prior candidate `bass4`. The exposure contract refuses a test-partition dataset entry, so
the test-day reading is recorded under `sources`
(SpaceApps-Ardealu:docs/wsp/feedback/bass5_post_final_result.md).

**Brief.** The prompt is in [docs/research/PROMPTS.md](../../research/PROMPTS.md); the author
worked from `phys_bass_post_final.md` and, for the inherited lines, the dossiers named in section 7.

**Decisions.** On the test days two burns carried 87 % of bass4's quench loss
(phys_bass_post_final.md, section 1):

- **O13a**, 62 % of the loss, is not a blow-off: the workbook row holds the pot settings in the fan
  cell and the fan displays in the air cell, so the coding read displays 49 to 40.5 as speeds in
  cm/s. The burn went out at pot 0.31, the floor the law already forecast. Nothing was typed for it;
  the data lane recoded it (third coding, rules R11 to R15 of `feedback/bass1_coding_v2.md`, four
  burns changed, none on validation days).
- **M10**, 25 %, is a 4 mm two-sided slab 1 cm wide at 20.1 %, the same fault as M11 on the train days.
  No threshold in thickness and oxygen alone separates the slabs; every 1 cm slab of 3 mm or more
  reached its limit, at up to 20.1 %, and no 2 cm slab did above 19 %. The width separates them, so
  one input (`flat_width`) and one number, `-0.11` per e-fold of width in $z_c$, written as `-3.3`
  (30 × 0.11) in five places, were added.

The expectation was written in the log before the one run: train day mean 0.166 to about 0.125
(realised 0.128), validation unchanged 0.108 (0.108), test 2.910 to about 2.47 (2.473), on the second
coding. Every element of bass5 equals bass4's except the 9 the block writes (4 domain lines, 5
weights; `runs/dev/gen7/bass5_parent_check.json`).

## 4. Weights

Every tensor of `params.npz` (`configs/models/served/models/050a4ceb…b82d/params.npz`). Each network
has `W1` [8, 12] (96), `b1` [8], `w2` [8], `skip` [12], `b0` []: 125 elements. Totals
(`architecture.parameter_counts`): **1556 authored elements, 250 non-zero, 1 constant**
(`num.substeps` = 512).

| Group | Elements / non-zero | Key values | Physical meaning |
|---|---|---|---|
| `holds_low_flow.logit.*` | 125 / 8 | `b0` 8.26, `skip[0]` 28; unit 0: `W1[0,0:4]` [21.7, -1.58, 3.67, -1.31], `b1` 8.04, `w2` -1 | `min(l_f, l_th)`: floor 13.4 % O2 (ln-sd 0.06), thermal limit from radiation and conduction into the cold solid |
| `holds_high_flow.logit.*` | 125 / 18 | `b0` 8.26, `skip[0]` 28; units 0 to 3 on `z_u - 0.75 z_r`, `b1` [-0.917, -2.527, -3.402, -4.319], `w2` [-1.09, -1.54, -3.00, -3.86] | blow-off: `X_k = 0.134 + 0.00035 a_n`, chords at 5, 25, 60, 150 /s |
| `oxygen_limit.median.*` | 125 / 27 | `b0` -2.010 (ln 0.134); units 0, 1 the thermal excess (`w2` 0.85, 0.15); units 2 to 5 the blow-off rise (`w2` 0.039, 0.055, 0.107, 0.138) | `ln X_lim = ln max(X_th, X_k)` |
| `oxygen_limit.spread.*`, `.tail_*` | 125 / 15; 2 / 2 | `b0` 0.06; `w2` [0.5, -0.5, 0.02]; tail 0.1, width 2.0 | 0.06 on the floor, up to 0.28 where heat loss sets the limit |
| `quench_speed.median.*` | 125 / 22 | `b0` -4.356 = ln(0.941 × 0.31 × 0.044 m/s), `skip[7]` 1.0 (`u_pot1`); units 0 to 3 written (`W1` [2.4, 0, -2.0, 0.5, 0, -20.0], `b1` [2.334, 0.534, -0.066, -0.366]) with `w2` = 0 | `U_q = 0.941 × 0.31 × u_pot1`; the W0 forced and diffusive chords and the bass2 rod switch are inert |
| `quench_speed.spread.*`, `.tail_*` | 125 / 11; 2 / 2 | `b0` 0.15 (units written, `w2` = 0); tail 0.1, width 2.0 | ln-sd 0.15: the setting reproduces to 0.03 pot in 0.31 |
| `blowoff_speed.median.*` | 125 / 17 | `b0` -0.347, `skip` [2.8, 0, 0.75]; five knots on `-z_x`, `b1` [0.054, 0, -0.057, -0.118, -0.182], `w2` [-0.85, -0.65, -1.06, -2.22, -3.4] | `U_b ∝ (X - 0.134) R^0.75`, bent to `ln(X - 0.134)` below 19 % |
| `blowoff_speed.spread.*`, `.tail_*` | 125 / 7; 2 / 2 | `b0` 0.36; `w2` [0.8, 1.5]; tail 0.1, width 2.0 | ln-sd 0.36, growing below 17 % O2 |
| `flat_holds_low_flow.logit.*` | 125 / 11 | `b0` 5.31, `skip` [20, 0, 0, 0, 0, …, -4.17 (SIBAL), -10.56 (Nomex, Ultem)]; unit 0 `W1[0,:]` [15.5, -3.75, 0, 0, 1.52, …, -4.17, -10.56], `b1` 5.33, `w2` -1 | film floor 13.8 %, fabric floors 17.0 % and 23.4 %; thermal limit of a flame edge |
| `flat_holds_high_flow.logit.*` | 125 / 23 | `b0` 5.31, `skip` [20, …, -2.71, -8.87]; units 0 to 4 on `z_u - 0.693 concurrent`, `b1` [0, -1.609, -2.303, -2.996, -3.689], `w2` [-0.56, -0.98, -1.32, -2.10, -2.30] | `X_k = 0.138 + 0.0016 U`; fabric floors 15.8 % and 21.5 % |
| `flat_quench_speed.median.*` | 125 / 22 | `b0` -4.126 = ln(0.941 × 0.39 × 0.044), `skip` [-0.6, 0, 0, 0, 0.405, 0, 0, 1.0]; unit 3 `W1` [-30.0 (z_x), 4.41 (thickness), 8.4 (one-sided), -3.3 (width)], `b1` -6.12, `w2` 1.0; unit 4 (cap) `W1` [-30.6, 0.405, 4.11, 1.0, 8.2, -3.3], `b1` -7.502, `w2` -1.0; units 0 to 2 written with `w2` = 0 | floor level `U_A`; class change `30 (z_c - z_x)`; cap `U_B`; bass5 width term `-3.3 ln(W / 2 cm)` |
| `flat_quench_speed.spread.*`, `.tail_*` | 125 / 23; 2 / 2 | `b0` 0.15; units 3 to 5 with the same `z_c` row (width -3.3), `b1` [-6.12, -7.05, -7.99], `w2` [0.7, -1.18, 0.48]; tail 0.15, width 2.5 | ln-sd 0.15 at the floor, 0.80 half-way between the classes, 0.35 at the limit level |
| `x.<feature>.centre`, `.scale`, `.lo`, `.hi` | 48 / 38 | as in the input table; every `scale` 1.0 | declared domain and feature centres |
| constant `num.substeps` | 1 | 512 | event integration steps (no event target here) |

Full list with the author's comments: `ignis wsp inspect --release sha256:050a4ceb6c094b9256c5568408f8574508530ce1b31ce45eae43e8785fa4b82d --edits`,
or `configs/models/law/bass5/s1-fable/edits.wsp` (362 lines: W0, D1, bass2, bass3, bass4 and bass5
blocks in order, each marked).

## 5. Forward and assembly

| | |
|---|---|
| Forward | `crates/ignis-wsp/src/law.rs`, entry point `ignis_wsp::law::forecast`, version 1, profile `strict_no_fitting` |
| forward_hash | `sha256:9baddddb61236c25f7c6eef83de0abebea0af23281d14c45aae35c9b0b8ddd70` |
| code_version | `ignis-wsp@0.1.0+fwd.9baddddb61236c25` |
| Spec | `law-bass5-v1`, `sha256:e61575773015bfd8c938a311ede7c8e3ced013c10cc610aa3f67008667bb0ba8` |
| Patch | `law-bass5-v1-s1-fable`, `sha256:389cd0812ca2699c4d0c36fc77590179698a415e12cdb61724e6614de9445558` |
| weights_hash | `sha256:ebdd8b6cddd55f3b68027b8f1a644968a6caccdf513b6c0da349cc5f636d9b15` |
| Binding | cohort `atlas:PSI-26+PSI-25-solids:catalogue`, split `sha256:b09d8f8f67ebd5e51e0efdaa88e55ed88550bfd0ca1b79106bf54359402600bd`, source manifest `sha256:432de55500c43586c8187264db5fb8be9db020d1b58dfd08dca97c2164d0c57d` (`configs/models/law/bass5/binding.json`) |
| Declaration | `configs/models/law/bass5/decl.json`, compiled in by `law_slugs.rs` (`slug!("bass5")`) |
| Candidate directory | `configs/models/law/bass5/s1-fable` (`edits.wsp`, `meta.json`, `patch.json`, `release.json`); root in `configs/models/law/bass5/root` |
| Numerics | positive quantiles in closed form for one log-normal, bisection of the fixed two-component mixture CDF otherwise; no request-time fitting |
| Checks | `spec_valid`, `patch_valid`, `tensors_complete`, `strict_forward_audit`, `finite_forward_on_fixture`: passed |
| Created | 2026-11-15T10:02:52Z |

Rebuild (the neutral root is assembled first into the same registry; `assemble` writes the release
into `--root`, default `artifacts/` or `IGNIS_ARTIFACTS`):

```
cargo build --release -p ignis-cli
B=configs/models/law/bass5
ignis wsp compile  --family bass5 --edits $B/root/edits.wsp --meta $B/root/meta.json --out $B/root/patch.json
ignis wsp assemble --family bass5 --patch $B/root/patch.json --binding $B/binding.json --root artifacts --out root.json
ignis wsp compile  --family bass5 --edits $B/s1-fable/edits.wsp --meta $B/s1-fable/meta.json --out $B/s1-fable/patch.json
ignis wsp assemble --family bass5 --patch $B/s1-fable/patch.json --binding $B/binding.json --root artifacts --out rebuilt.json
python tools/wsp/verify_served.py --ignis target/release/ignis --parents artifacts
cargo test -p ignis-wsp-serve --test served_ids
```

`verify_served.py` rebuilds every served release from its committed patch and binding and compares
the record; `served_ids` pins the 14 served ids. `cargo test -p ignis-wsp-serve --test served_ids`
passed (2 of 2) on 15 November 2026 at ignis-web a58c7c3: the served id is the content hash of its
record and loads through the registry checks.

## 6. Evaluation

Lower is better everywhere. Quench speed: interval CRPS in cm/s (the outcome is an interval of fan
settings), mean of day means with the case mean in brackets; `quench_display` is the same in fan
display units. Kept burning: Brier score.

**Kept burning: the held result of the final run** (test days, unread by every release scored;
second coding; SpaceApps-Ardealu:docs/wsp/RESULTS_FINAL_GEN7.md, and PROTOCOL 13.7 outcome). bass5's
kept-burning forecasts equal bass4's on every case of every partition, so this is bass5's answer:

| Release | Answered | Brier, case mean | Brier, mean of day means |
|---|---|---|---|
| bass4 (fabrics in kept burning) | 42 | **0.0477** | 0.0765 |
| bass3 | 38 | 0.0523 | 0.0758 |
| bass1 W0 and D1 | 34 | 0.0588 | 0.0821 |
| train incidence | on the model's cases | 0.0707 | 0.1120 |
| always lit | on the model's cases | 0.0750 | 0.1282 |

The 13.7 reading by the case mean: holds (PROTOCOL 13.7).

**Quench speed, third coding (the coding that counts from here); typed after the final run with
every partition seen; in-sample on all data; no held-out score exists**
(SpaceApps-Ardealu:docs/wsp/feedback/bass5_post_final_result.md):

| Partition | Cases | bass5 | bass4 | constant |
|---|---|---|---|---|
| train | 43 | **0.1143** (0.1585) | 0.1611 (0.2413) | 0.6709 (0.9407) |
| validation | 12 | **0.1082** (0.1057) | 0.1082 (0.1057) | 0.5466 (0.5480) |
| test | 17 | **0.1255** (0.1525) | 0.5633 (0.5130) | 0.9157 (1.1706) |
| pooled | 72 | 0.1156 (0.1483) | 0.2392 (0.2828) | 0.7011 (0.9295) |

On the display scale bass5 is also lower than the constant on all three partitions (test 0.3476
against 2.6716). Kept burning on the third coding: train 0.0220, validation 0.1221, test 0.0532 (40
cases), identical to bass4; train incidence 0.1036, 0.1160, 0.1120.

**Quench speed, second coding (the coding of the final run), for the record** (same source):
bass5 0.1281 / 0.1082 / 2.4726 against the constant 0.8334 / 0.6482 / 2.7049. In the final run itself
bass4 scored 2.9103 against 2.7049 on the test days and did not hold (RESULTS_FINAL_GEN7.md); 62 % of
that loss was O13a, the miscoded burn.

**Fabric kept burning, development days** (bass4, unchanged in bass5; SpaceApps-Ardealu:docs/wsp/feedback/bass4_result.md):
19 fabric and Ultem burns 0.0529 (case mean) against 0.3801 for the train incidence and 0.4737 for
always lit; SIBAL 9 burns 0.0003, every one forecast at 0.954 to 0.995 and every one kept burning;
Nomex and Ultem 10 burns 0.1003. The SIBAL numbers were typed before any fabric case was opened.

**Earlier looks of the lineage** (validation, second coding unless stated): bass1 W0, blind (13.2):
kept burning 0.0862 against 0.1203 for the train incidence, beats every baseline; quench speed
0.5169 against 0.2299 for the constant, loses (feedback/bass1_w0_result.md). bass1 D1 (13.3): quench
0.5067, kept burning 0.0862 (feedback/bass1_d1_result.md).

What the author had seen: the bass5 quench numbers describe fit to data the author had read, all
partitions included; the width term was typed after seeing M10. The kept-burning test-day result was
made before any plastics test day was read, by bass4, and bass5 reproduces it exactly.

## 7. Adoption decision and lineage

**Why this release is served.** PROTOCOL 13.8: a post-final revision is served only if it is lower
than the constant or incidence baseline on each of the three partitions for the target it repairs,
and changes no other served answer of its investigation by more than rounding. bass5 is lower than
the constant on train, validation and test for the quench speed (third coding), on both scales, and
every kept-burning forecast is identical to bass4: "By 13.8, the bass5 quench speed is served"
(bass5_post_final_result.md; PROTOCOL 13.8 outcome). Before it, 13.7 served bass4 for kept burning
and, chosen after the test, bass1 D1 for the quench speed; bass5 now answers both.

**Lineage.** Each slug has its own spec and neutral root (`ignis wsp law-root`); the numbers pass
from one slug to the next as the full parent block plus new lines.

| Step | Release | PROTOCOL | What changed |
|---|---|---|---|
| bass1 W0 | `sha256:97c0daf9de84482ed7b125b3d41302521f5dd7281c0667241807d93387e239a2` | 13.2 | zero-shot law from first principles (phys_bass1_w0.md section 7.4): thermal and kinetic limits for rods, spheres, films, slabs |
| bass1 D1 | `sha256:b3bb1869df2fc837f7957e925f55b357ecc6272d312f1ddbb46346da15376ca0` | 13.3 | one damped step from train days: conduction into cold slab at a spreading edge (four numbers); replaced its parent |
| bass2 S1 | `sha256:e008cba3f5101f218c1e7f353fed928a8e11a22500ebf1197dc9f050c57ca4c9` | 13.4 | features `is_rod`, `flat_thickness`: rods are not fed by diffusion from all sides; the film edge sits in the holder's boundary layer; slab conduction by thickness (phys_bass2.md). Quench pooled 0.4505 against 0.8135 for bass1 D1 |
| bass3 S1 | `sha256:a1eb577ae257a788109329a8cc54509925ecab4c1e3e88ddcf9decf0fd7703d1` | 13.6 | features `u_pot1`, `is_one_sided`: the floor at pot 0.31 and 0.39 on the burn's fan curve, the two-class slab law (phys_bass_study.md section 4). Quench pooled 0.1310 against 0.3926 |
| bass4 S1 | `sha256:ceefc15d4cb8cf658772ec3603c9dabd6565fb3db6528bb35c2205f0d3bdab86` | 13.6 | features `is_sibal`, `is_nomex_ultem`: fabrics in the kept-burning pair (phys_bass_study2.md sections A and E); every PMMA forecast identical to bass3 |
| **bass5 S1 (served)** | `sha256:050a4ceb6c094b9256c5568408f8574508530ce1b31ce45eae43e8785fa4b82d` | 13.8 | feature `flat_width`; `-3.3` in five places: sideways conduction into the frame (phys_bass_post_final.md) |

Neutral roots: bass1 `sha256:5c1fa50f…d21c`, bass2 `sha256:8d21f327…fd2a`, bass3 `sha256:15e6f7c6…bfb9`,
bass4 `sha256:27898a87…ce8b`, bass5 `sha256:1558a1c1…cf52` (`configs/models/law/bass*/root/release.json`).

**Tried and not adopted** (feedback/bass3_result.md, bass3_s1_q1_result.md; PROTOCOL 13.6): bass2 K1
`sha256:6298ab84749e6d65ae21e6e21b04710444541d4d11e0359390adc430c81fcae1` (one floor for rods,
spheres and films, no physics: quench 0.3138 against 0.3926; superseded by bass3); bass3 S1 comparator
`sha256:cf0b3c0163a6c95ca3b2f1fac0aa7bb320603a10bc246592e31eddd3e6378396` (a 21 % floor for one-sided
flat samples) and Q1 `sha256:3c0a5aaeaa67fb01d7c7f56f1041d9853c87cf599315c7a91a14323048b49b6a` (the 5 mm
threshold at the middle of its bracket), kept as comparators, "not part of the model unless the
coordinator decides so". The fabric quench boundary revision of 13.8 (`bass_front.wsp5`) was not
served, so "does the fabric keep burning" is answered by this model.

**Scope** (served `limits`, condensed):

- Typed after the final run with every partition seen; in-sample on all data; no held-out score
  exists.
- The coding of four burns was corrected after the final run (pot settings and fan displays in
  swapped cells; a flame that went out after the flow was raised).
- Below about 4 cm/s the recorded outcome is a fan setting, not a calibrated speed: read a quench
  speed there as a fan setting.
- It answers whether a fabric keeps burning: its SIBAL part was typed before the fabric cases were
  opened and held (9 of 9 on the development days); the fabric burns of the test days are among its 42.
- All targets served; choose one with `settings.target` (default the first).

## 8. How to call it in IGNIS

**Forecast lab.** Open `#/lab`; the Models screen `#/models` lists the served releases and opens this
model card. `web/src/views/LabModels.tsx` runs a typed scenario for this release; stored runs are not
available (`stored_runs = false`: no stored run of this investigation carries the declared features).

**Model card.** `GET /api/v1/model-cards` returns the served entry (inputs, scores, limits, cite).

**API.** First the scenario (conditions only; every feature is required):

```
POST /api/v1/models/sha256:050a4ceb6c094b9256c5568408f8574508530ce1b31ce45eae43e8785fa4b82d/scenarios
```

```json
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:050a4ceb6c094b9256c5568408f8574508530ce1b31ce45eae43e8785fa4b82d",
  "cutoff_s": 0.0,
  "conditions": {
    "x_o2": 0.20, "flow_speed": 0.05, "nose_radius": 0.01, "heat_time": 200.0,
    "is_concurrent": 0, "is_rod": 0, "flat_thickness": 0.004, "u_pot1": 0.044,
    "is_one_sided": 0, "is_sibal": 0, "is_nomex_ultem": 0, "flat_width": 0.01
  },
  "controls": [],
  "observations": [],
  "test_date": null,
  "phase_established_at_s": null
}
```

These are typed values for illustration (the served entry gives no example), not a measurement: a
4 mm PMMA slab burning on two sides in a 1 cm frame, opposed flow, 20 % oxygen, on a fan curve of
4.4 cm/s at pot 1. It returns `input_hash`. Then the analysis, which returns a job:

```json
POST /api/v1/analyses
{
  "schema_version": "wsp.v1",
  "release_id": "sha256:050a4ceb6c094b9256c5568408f8574508530ce1b31ce45eae43e8785fa4b82d",
  "input": { "kind": "prefix", "input_hash": "<input_hash from the scenario>" },
  "settings": { "target": "flat_quench_speed", "quantile_levels": [0.05, 0.25, 0.5, 0.75, 0.95] }
}
```

Poll `GET /api/v1/jobs/<job_id>` and read `GET /api/v1/artifacts/<artifact_id>`: an
`AnalysisArtifact` of evidence class `model_output`, labelled `bass5:<target>` in `diagnostics.law`;
targets without a shared quantity carry `quantity = null`. `settings.target` takes `holds_low_flow`
(default), `holds_high_flow`, `oxygen_limit`, `quench_speed`, `blowoff_speed`,
`flat_holds_low_flow`, `flat_holds_high_flow`, `flat_quench_speed`. A stored-run input is refused.
MCP: `models_inspect` (the card) and `models_forecast` (the same scenario and analysis).

Combination rule: "kept burning" is the smaller of the two `holds` probabilities of the sample's
kind (rods, spheres and sections: `holds_low_flow` and `holds_high_flow`; films, slabs and fabrics:
the `flat_` pair). For a fabric set `is_sibal` or `is_nomex_ultem`, `nose_radius` 0.01, `heat_time`
200, `is_rod` 0, `is_one_sided` 0, `flat_width` 0.02 and the fabric's thickness (the lane used
0.00032, 0.0003 and 0.00025 m; `is_concurrent` 0.5 where no direction is recorded;
feedback/bass4_result.md); its quench speed is not answered. For the flame-tip position on a fabric use
`bass_front.wsp` D1.

Cite: bass5.wsp S1, hand-written (literal WSP) release sha256:050a4ceb…b82d, experimental, not trained; typed after the final run with every partition seen; its output is model output, not a measurement.

## Sources

IGNIS (`D:\dev\ignis-web`, branch harness-docs-20261115, a58c7c3):

- `configs/models/served/served.json`; `configs/models/served/models/050a4ceb6c094b9256c5568408f8574508530ce1b31ce45eae43e8785fa4b82d/release.json`, `params.npz`
- `configs/models/law/bass5/s1-fable/edits.wsp`, `meta.json`, `release.json`; `configs/models/law/bass5/root/release.json`; `configs/models/law/bass5/binding.json`, `decl.json`
- `configs/models/law/bass1/{w0-fable,d1-fable,root}`, `bass2/{s1-fable,k1-fable,root}`, `bass3/{s1-fable,os1-fable,q1-fable,root}`, `bass4/{s1-fable,root}`: `release.json`, `meta.json`
- `crates/ignis-wsp/src/law.rs`, `crates/ignis-wsp/src/law_slugs.rs`, `crates/ignis-wsp/src/cli.rs`
- `crates/ignis-wsp-serve/src/lib.rs`; `crates/ignis-core/src/contracts/wsp.rs` (`ScenarioRequest`)
- `tools/wsp/verify_served.py`, `tools/wsp/reassemble.py`; `docs/wsp/README.md`

Research (SpaceApps-Ardealu, lane branch feat/handwritten-science, 542dad8):

- `SpaceApps-Ardealu:docs/wsp/PROTOCOL.md` (sections 13.7, 13.8), `RESULTS_FINAL_GEN7.md` (bass section), `COVERAGE_19.md`
- `SpaceApps-Ardealu:docs/wsp/research/phys_bass_post_final.md`, `phys_bass1_w0.md`, `phys_bass1_d1.md`, `phys_bass2.md`, `phys_bass_study.md`, `phys_bass_study2.md`, `phys_quench_w0.md`, `phys_quench_post_final.md`
- `SpaceApps-Ardealu:docs/wsp/feedback/bass5_post_final_result.md`, `bass4_result.md`, `bass3_result.md`, `bass3_s1_q1_result.md`, `bass2_second_law_result.md`, `bass1_w0_result.md`, `bass1_d1_result.md`
