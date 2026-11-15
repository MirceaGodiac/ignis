# IGNIS hand-written science: experiments of generations 1 to 5

---

## Contents

1. [What the work is](#1-what-the-work-is)
2. [The five model families and their data](#2-the-five-model-families-and-their-data)
3. [Protocol rules and how they evolved](#3-protocol-rules-and-how-they-evolved)
4. [Owner instructions that shaped the work](#4-owner-instructions-that-shaped-the-work)
7. [Generation 3: train audits and the selection rule](#7-generation-3-14-november-0113-train-audits-and-the-selection-rule)
8. [Generation 4: published science, new forwards, the first BASS model](#8-generation-4-14-november-morning-published-science-new-forwards-the-first-bass-model)
9. [Generation 5: physics-first rebuild, committees and the planned stop](#9-generation-5-14-november-midday-physics-first-rebuild-committees-and-the-planned-stop)
10. [After the fifth generation, 14 November evening](#10-after-the-fifth-generation-14-november-evening)
11. [The carried model per family, generation by generation](#11-the-carried-model-per-family-generation-by-generation)
12. [What transferred and what did not](#12-what-transferred-and-what-did-not)
13. [Data classification and exposure ledger](#13-data-classification-and-exposure-ledger)
14. [Agents and authors](#14-agents-and-authors)
15. [Reproduction](#15-reproduction)

---

## 1. What the work is

Every parameter of every model is a literal number typed by an author model, assembled without
estimation, and run by a forward that cannot fit anything (`README.md`; `PROTOCOL.md §1`).

| Piece | Does | Never does |
|---|---|---|
| Spec (`ModelSpec`) | Fixes target, features, every tensor with role, bounds and fill rule, the forward entry point | Hold a default for a physical, noise or hazard parameter |
| Author (`claude-fable-5-2` for generations 1 to 5) | Types literal numbers in the edit language, with a reason beside each block | Run an optimizer, regression, search or a program that computes the numbers |
| Assembler (`ignis wsp compile / check / assemble`) | Applies assignments, checks shapes, bounds, completeness; records provenance | Estimate, fill or repair anything |
| Forward | Deterministic features, fixed weights, explicit numerical integration | Gradient, MAP, NUTS, least squares, sampling, a fallback to a fitted expert |
| Feedback (`wsp-dev`, `ignis eval`) | Scores forecasts on train and validation with the evaluator's functions | Read the test partition; change a parameter |

The edit language is literal assignment only (`rate.b0 = -0.58`, `rate.skip[0:7] = [...]`), with no
expressions, strides, in-place arithmetic or includes; one bad statement rejects the file
(`PROTOCOL.md §2`). A release record carries `trained_on = not_trained`, `state = experimental`, the
author's provider model id, its exposure, and hashes of spec, patch, weights and forward; the release
id is the content hash of that record and a release refuses to run under any other forward
(`PROTOCOL.md §3`).

**Naming.** `W0` is a candidate written before any outcome of its fuel or family was seen. Later
candidates are `D1`, `D2`, … (data-informed), `P0`, `P1` in the fifth generation, and their edits
files say in the first lines what was read (`PROTOCOL.md §7`; `dossier/04_decisions.md D-03`). The
first data-informed candidate of a new spec is a revision of a reference baseline that holds no data
(a condition-free D²-law, clock, published theory or persistence), which doubles as the
"knows-nothing" comparison.

---

## 2. The five model families and their data

| Family | Experiment | Archive | Target | Split file (`data/splits/`) | Train / validation / test runs | Groups |
|---|---|---|---|---|---|---|
| FLEX methanol | Free droplet combustion in the CIR | PSI-69 | Liquid diameter after the cutoff (CRPS, µm) | `flex-660b7d9ede629b30.json` | 68 / 24 / 22 | day-sessions 27 / 9 |
| ACME normal | CFI-G spherical burner flames, fuel into O2/N2 | PSI-159 | Extinction and flow termination (integrated Brier, 6–120 s) | `acme_normal-c77bc5baee98d3e3.json` | 81 / 28 / 20 | test days 7 / 2 |
| ACME inverse | CFI-G, O2/N2 into fuel/N2 | PSI-159 | Same | `acme_inverse-747032c921c8c8f2.json` | 73 / 24 / 34 | test days 7 / 2 |
| BASS | BASS-II flame spread over SIBAL fabric, concurrent flow | PSI-25 | Flame-tip position in sample widths (CRPS) from generation 4 | `bass-4083156d5b18dab4.json` | 4 / 17 / 4 | tests |

Source: `dossier/02_data_classification.md §2.1`. The earlier BASS campaign (PSI-26) belongs to no
partition and was used as external data in generation 5. A whole day sits in one partition, so no
chamber fill is split. Scoring cutoffs: droplets and burner flames 1, 3, 5 s (`RESULTS.md` header);
BASS 5, 10, 20, 40 s after the igniter is off (`PROTOCOL.md §9`).

---

## 3. Protocol rules and how they evolved

| When | Rule introduced | Source |
|---|---|---|
| Gen 3, 14 Nov | A forward is frozen with its releases; an improvement is a new forward file with new model names, so every earlier release keeps its id and score | `PROTOCOL.md §8`; `D-02` |
| Gen 3 | **Selection rule written before validation.** Droplets: median train CRPS over common cases not worse, and better or level (within 1 µm) in at least 60 % of train sessions. Burner flames: a change is taken when it holds in at least three sessions (or two with a physical reason); a number from one session is weakened, not sharpened | `PROTOCOL.md §8`; `D-40`, `D-41` |
| Gen 3 | Validation acts as a **veto only**: a candidate that passes on train but does not improve its family's primary validation score is not adopted | `PROTOCOL.md §8` |
| Gen 3 | Spread is not chosen by the train rule: band = train residual plus a stated margin for unseen sessions | `PROTOCOL.md §8`; `D-06` |
| Gen 4, 14 Nov | Literature reviews read mechanisms, theory and published correlations, never per-test results of the series being modelled; BASS-II concurrent results not read | `PROTOCOL.md §9`; `dossier/README.md` rule 5 |
| Gen 4 | Burner flames: a new-kind candidate replaces the carried one unless its validation extinction score is worse | `PROTOCOL.md §9` |
| Gen 4 | BASS validation burns measured blind by agents that knew nothing of the model, into sealed files the author never opens | `PROTOCOL.md §9`; `D-31` |
| Gen 4 | Carry releases: the carried numbers on a new forward with new entries neutral, identical forecasts case by case, so paired comparisons work across forwards | `PROTOCOL.md §9` |
| Gen 5, 14 Nov | New inputs live in a private copy of the data root (`runs/dev/data_ext/`); shared records and contracts untouched; nothing asked of other lanes | `PROTOCOL.md §10`; `D-07` |
| Gen 5 | One registered look per family; rules and candidates committed before the look; for burner flames the rule applied twice, with and without the planned stop | `PROTOCOL.md §10` |
| Gen 5 | Private validation continuations go into their own write-once vault (`IGNIS_HOLDOUT=runs/dev/holdout_ext`), written by the evaluator's own deposit command | `D-45` |
| 14 Nov evening | A model is run on an investigation it was not written on only after the test is registered in a commit that precedes scoring, and the scoring chain first reproduces known train numbers | `PROTOCOL.md §11`; `D-47` |
| 14 Nov evening | Evaluation groups of the 15 other investigations frozen before any author reads their outcomes | `PROTOCOL.md §11.2`; `D-48` |

The test partition was not read by anything on the branch through generation 5 (`README.md`;
`RESULTS_GEN5.md` header). `tools/wsp/analysis/lib.py` has no way to list it
(`dossier/02_data_classification.md §2.1`).

---

## 4. Owner instructions that shaped the work

As recorded in the repository:

| Instruction | Where recorded | Effect |
|---|---|---|
| Make it unmistakable which numbers are zero-shot | `dossier/04_decisions.md D-03` | `W0` reserved; `D`/`P` naming; registry records W0 as `literal_one_shot` and refuses development exposure on it |
| Nothing is asked of the other lanes (14 Nov) | `D-07` | The lane built its own measurements (BASS tracks, script stops, flame cameras) in a private data root |
| Rebuild the models from the physical mechanism; "many, many parameters" are welcome if each has physical logic; do not overfit; analysis by `claude-fable-5-2` at high effort (14 Nov) | `dossier/08_in_progress.md §8.1`; `PROTOCOL.md §10`; `D-42` | The fifth generation: five physics dossiers, four independent re-derivations, a property look-up |
| Models usable on all of NASA's combustion investigations (14 Nov) | `PROTOCOL.md §11` | Transfer test on Flame Design; groups of 15 investigations frozen |
| Freeze evaluation groups before outcome data are viewed | `PROTOCOL.md §11.2` | `docs/wsp/atlas/splits/` |
| A second author (Astra) reviews and extends; all Astra candidates compared against Fable on train under a committed protocol | `dossier/README.md`; `dossier/ASTRA_FROZEN_TRAIN_20261114.md` | Section 10 below |

---

## 7. Generation 3 (14 November, 01:13): train audits and the selection rule

**Result in one line.** Four train audits sharpened the laws; three of four candidates improved
validation (methanol 45.3 → 40.0 µm, heptane 51.9 → 46.1 µm, normal flames 0.170 → 0.159) and the
methanol and heptane models of this generation remained the carried droplet models through
generation 5 (`PROTOCOL.md §8`; `README.md`).

**How it was run.** Four analysis agents (`claude-opus-5-5`, medium) each audited one
model against its train partition: error budget, residuals by condition and session, what transfers
between sessions, law-level proposals with the runs that would contradict them. Rules: train only, no
fitting, no forward or weight change, nothing written to the repository (`PROTOCOL.md §8`). Reports:
`feedback/flex_methanol_d2_train_audit.md`, `flex_heptane_d1_train_audit.md`,
`acme_d1_train_audit.md`, `bass_train_measurement.md`. The author rechecked each claim acted on and
declined some (the heptane helium clock, single-session effects), then typed every number. A new
forward `droplet3.rs` (wired and tested by two engineering agents, reviewed by the author). Candidates
 before any validation score; selection on train by the rule of §3; validation
read once.

**Audit findings that became numbers** (`EXPERIMENTAL_SCIENCE.md §5`):

- Methanol radiative extinction is a cliff in the clock: `ln s = −3.8 (ln(D0/D_crit) − 0.05)` passes
  through every radiative and no-sustained-flame run.
- An unlit droplet sheds the flame's heat fast (0.066 mm²/s at 1.3 mm, 0.165 at 2.9 mm in the first
  two seconds); the state at the cutoff is read from the end rate of the prefix.
- Heptane burns near its plateau at once; its radiative clock shortens by 2.7–3.5 per e-fold of D0;
  the cool-flame rate law was right to 6 % once read from the right window.
- ACME normal: the life depends on the reduced flow `ln ṁ − 0.65·(x_O2 − 0.30)/0.10 + 0.24·ln p`,
  which reproduces the ethane lives of three sessions at 0.5, 1 and 2 atm within about 15 % (`D-22`).

| Family | Candidate | Id | Train | Selection on train | Validation | Decision |
|---|---|---|---|---|---|---|
| FLEX methanol | **`flex_methanol.wsp2` D3** | `sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654` | median 17.7 µm (D2 23.1) | better or level in 24 of 26 sessions | **40.0 µm** (D2 45.3; W2 41.7; best fitted 77.7) | **adopted** |
| FLEX n-heptane | **`flex_heptane.wsp2` D2** | `sha256:edbad4d6395961030e0b17d9d5c4d40f2de902ddcbcbd0ec2a4f4ec9adbeedef` | median 21.8 µm (D1 28.2) | better in 17 of 20 sessions | **46.1 µm** (D1 51.9; W0 60.3) | **adopted** |

Sources: bodies; `PROTOCOL.md §8`; `RESULTS.md`. Paired validation
differences to methanol D3: W0 +10.4 [−0.2, +21.4], fitted flex.v3 +37.7 [+14.2, +60.8] µm; to heptane
D2: W0 +14.2 [+1.7, +25.6] µm (`RESULTS.md`).

**BASS in this generation.** The train audit showed three of four train burns can be tracked for
about 100 s in both cameras; the front responds to a fan step within half a second; at the lowest fan
setting the three flames spread at 0.036 sample widths per second; 16.4 % oxygen at 6 cm/s does not
spread. The target became the leading-edge position in sample widths with the fan history as a known
control (`EXPERIMENTAL_SCIENCE.md §5.3`; `cards/bass_optical.md`).

```bash
python tools/wsp/analysis/session_compare.py runs/dev/fb_train_flex_d3.json runs/dev/analysis/flex_methanol_profile.json 0 1
```

---

## 8. Generation 4 (14 November, morning): published science, new forwards, the first BASS model

**Result in one line.** The first BASS model met real data and halved the error of persistence on
twelve burns it had never seen, measured blind (tip CRPS 0.117 against 0.234 sample widths); the
inverse burner flame was rebuilt on the published growth law with a burner-heating law and adopted
(0.146 → 0.137); and two burner-flame laws answering together became the normal-flame model
(0.159 → 0.155) (`PROTOCOL.md §9`).

**How it was run.**

- Three literature reviews by research agents (`claude-opus-5-5`, medium; 14 Nov 05:53):
  `research/lit_droplets.md`, `lit_spherical_flames.md`, `lit_flame_spread.md`. Rule: mechanisms,
  theory, model predictions and correlations with the kind of every number stated; no per-test result
  of the modelled series; no BASS-II concurrent results (`PROTOCOL.md §9`).
- Three train-only analyses (`claude-opus-5-5`): `feedback/flex_heptane_d2_train_anatomy.md`,
  `flex_methanol_d3_train_anatomy.md`, `acme_transfer_audit.md`; BASS train tracks packaged
  (`feedback/bass_train_tracks.md`; tracks in `tools/wsp/bass_optical/tracks/train/`).
- **Blind BASS validation measurement.** Two measurement agents (A and B, `claude-opus-5-5`, 05:50)
  measured the seventeen validation burns with the train method into sealed files
  (`runs/dev/bass_front/val_sealed/`, not in git). The author read their inputs only (oxygen, sample
  label, band width, restrictor, igniter interval, calibration rows, fan values). Result: fourteen
  tracks in sample widths, one in frame widths, two with no flame track (`PROTOCOL.md §9`).
- Three new forwards, earlier forwards untouched: `droplet4.rs` (tracker flags, helium law),
  `flame3.rs` (published growth law, burner-heating law), `front.rs` (BASS flame tip). Engineering
  agents built tests and the `front-feedback` scorer; the author finished and reviewed the code
 (`dossier/registers/agents.md`). The model service `WspModelService` was added.
- Candidates (09:49) before validation; first look (10:12); second set
 (10:39); outcome (11:01).

**Frozen rules** (`PROTOCOL.md §9`). Droplets: the session rule of §3. Burner flames: replaces the
carried model unless validation extinction is worse. BASS: D1 reported against persistence, nothing
adjusted afterwards. Blends: replace the carried model unless validation extinction is worse. BASS D2:
served only if its tip scores equal D1's and its track-ended score is not worse.

### 8.1 ACME inverse: growth forward adopted

*Mechanism.* A burner-supported spherical flame grows towards `r_ss = m_tot / (4π ρD L)` and goes out
when the reactant flux at the flame falls to a critical value; with published constants (ρD
0.0334 g m⁻¹ s⁻¹, critical fuel flux 0.2 g m⁻² s⁻¹) it places the holding flow and the lives of 30 %
ethane without tuning (46, 64, 165 s against train 43, 71, 158–232 s). Lean inverse flames keep an
empirical early-quench clock (the published inverse constants give them minutes; they last 3–18 s).
The burner warms as `T0 + ΔT (1 − exp(−t / 30 s))`, `ΔT = 2012 K · (m_tot / 1 mg/s)^−0.4 · (r_ss / 1 mm)^−0.4`,
with session medians of ln(measured/law) within −0.08 to +0.03; limits weighted at 455 and 510 °C.
The stored inverse flow was found to be the oxygen alone and the forward derives the total burner flow
(`EXPERIMENTAL_SCIENCE.md §6.2`; `D-23`, `D-24`).

| Candidate | Id | Train | Validation | Decision |
|---|---|---|---|---|
| published theory only | `sha256:d0a13660…184b` | 0.4883 / 0.2466 | 0.3462 / 0.1933 | reference |
| **`acme_inverse.wsp3` D1** | `sha256:a43fdbcf5c73206a5d9d442a182d5e7e5c4a407d53e64b44231c014cbb00c1de` | 0.0357 / 0.0639 | **0.1373 / 0.0879** | **adopted** (carried 0.1464 / 0.0971; best fitted 0.143 / 0.118) |
| `acme_inverse.wsp4` D1 (blend of wsp2 D1 and wsp3 D1, 1 : 1) | `sha256:c7c9b1d3e079c758243950e2f65e2014e06d98d1cb9ad0bd49756d19f64e9cbc` | 0.0410 / 0.0612 | 0.1416 / 0.0918 | not adopted: the members share their self-extinction law |

### 8.2 ACME normal: two laws answering together, adopted

*Mechanism.* `blend.rs` holds a complete `flame.rs` release (the hinge law, an interpolation of the
observed lives) and a complete `flame3.rs` release (the growth law, a mechanism with few constants)
plus two weights; the forecast is the weighted mean of the two member forwards. The mean of two
forecasts scores better than the mean of their scores by a quarter of their mean squared
disagreement. Member numbers were copied statement by statement by `tools/wsp/blend_edits.py`
(which chooses nothing) and checked tensor by tensor; the only typed numbers are two equal weights
(`PROTOCOL.md §9`; `cards/acme_blend.md`; `D-25`).

| Candidate | Id | Train | Validation | Decision |
|---|---|---|---|---|
| published theory only | `sha256:3d8d3b7a…15ad` | 0.1411 / 0.0751 | 0.2064 / 0.1143 | reference (constant guess 0.214) |
| `acme_normal.wsp3` D1 (growth clock) | `sha256:af1d3f4a83bc7469aed313b3a5f7cfe2b9be2d9dc5f91a5102b520af3fde094e` | 0.0631 / 0.0237 | 0.1629 / 0.0997 | not adopted alone (tie with 0.1592) |
| **`acme_normal.wsp4` D1** (wsp2 D3 + wsp3 D1, 1 : 1) | `sha256:dfdaa65d9171c063a5ff5d05820096aac1acee32e3821609c519173be2537c6b` | 0.0517 / 0.0275 | **0.1553 / 0.0984** | **adopted** (carried 0.1592 / 0.1000) |
| baseline pair (two references, no CFI-G number) | `sha256:d7bd7e2c…796a` | 0.1507 / 0.0681 | 0.1809 / 0.0994 | reference; ahead of the best fitted model (0.185) |

Source: `RESULTS.md`; `PROTOCOL.md §9`. The blend gained 0.006 below the mean of its members' scores
and 0.004 below the better member.

### 8.3 BASS: the first model on real burns, scored blind

*Mechanism.* The flame is a band: a front advancing at `v = (0.0215 + 0.0064 U)(X_O2/0.19)^1.5`
(U the flow speed), a visible tip ahead of it by an overhang that relaxes within about 2 s, a base
that repeats the front one burn-through time (6.3 s) later, so after a step down in flow the flame
shortens from both ends; a quench law; the fan history as a known control
(`D-32`; `cards/bass_front.md`). Scoring rules fixed in `PROTOCOL.md §9` (cutoffs 5, 10, 20, 40 s;
CRPS from quantiles 0.05–0.95; paired bootstrap over burns).

| Release | Id | Train tip CRPS | Validation tip CRPS | Abs. error of median | 50 % / 90 % coverage | Track ended | Spread or not |
|---|---|---|---|---|---|---|---|
| **`bass_front.wsp` D1** | `sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c` | 0.076 | **0.117** | 0.162 | 0.66 / 0.90 | 0.178 | 0.170 |
| persistence (reference) | — | 0.242 | 0.234 | 0.315 | 0.34 / 0.62 | 0.277 | 0.125 |
| D2 (D1 without the oxygen quench boundary) | `sha256:0187941810a89347a9ea3953add9aa6058152bb643b009ca1ffa3e9471f216a7` | 0.076 | 0.117 | 0.162 | 0.66 / 0.90 | 0.183 | 0.121 |

Source: `RESULTS.md` (BASS; 12 burns, 40 cases). Paired difference persistence minus D1: +0.117
[+0.051, +0.182], D1 better on 10 of 12 burns. By cutoff (D1): 5 s 0.182, 10 s 0.123, 20 s 0.094,
40 s 0.040. **D1 became the carried BASS model.** D2 was not adopted by its frozen rule (track-ended
0.183 against 0.178).

### 8.4 FLEX: candidates refused, third generation stays

| Candidate | Id | Train | Validation | Decision, one line |
|---|---|---|---|---|
| `flex_methanol.wsp3` D4 | `sha256:2dab61e4…a2b1` | mean 26.9 / median 16.4 µm; 24 of 26 sessions | 44.8 µm against carry 39.5 | not adopted: bands narrowed from train coverage (90 % coverage 0.82) |
| `flex_methanol.wsp3` D5 (tracker flags only) | `sha256:0b57dba7…c2c7` | median 17.7, 25 of 26 sessions | 40.1 against 39.5 | not adopted: the tracker trouble it flags lives in train runs |
| `flex_heptane.wsp3` D3 | `sha256:d3cf6b7c…d6f` | 39.6 / 19.1; 18 of 20 sessions | 57.6 against 46.1 | not adopted: narrower bands and an unbounded helium law `exp(3.85 x_He)` |
| heptane flags-only | `sha256:b9a121a1…c4cc` | median 22.1 against 21.8 | not scored | failed the train rule, not taken |

Source: `PROTOCOL.md §9`; `RESULTS.md` (fourth-generation FLEX tables; 59 methanol, 39 heptane
common cases). Carry releases on `droplet4.rs`: methanol `sha256:637921fa…1cf6`, heptane
`sha256:f312235d…2c29` (identical forecasts to generation 3 on train).

**Carried after generation 4:** `flex_methanol.wsp2` D3, `flex_heptane.wsp2` D2, `acme_normal.wsp4`
D1, `acme_inverse.wsp3` D1, `bass_front.wsp` D1 (`PROTOCOL.md §9`).

---

## 9. Generation 5 (14 November, midday): physics-first rebuild, committees and the planned stop

**Result in one line.** Both burner-flame models were replaced by committees that read NASA's planned
stop of a test from its script file: inverse-flame extinction fell from 0.161 to 0.135 and flow
termination roughly halved (0.094 → 0.048); normal-flame flow termination fell from 0.098 to 0.075
and extinction from 0.155 to 0.145; and the inverse committee improved extinction to 0.129 even on the
shared records, without the planned stop (`RESULTS_GEN5.md`, outcome table).

**What was asked.** The owner: rebuild the models from the physical mechanism of each experiment;
parameters are welcome when each has a physical meaning; build something that generalises; analysis
by `claude-fable-5-2` at high effort (`dossier/08_in_progress.md §8.1`; `PROTOCOL.md §10`).

**How it was run.**

- **Five physics dossiers** (`claude-fable-5-2`, high; 14 Nov 10:32–10:34), each evaluating the
  published theory of its experiment against train at literature constants:
  `research/phys_flex_methanol.md`, `phys_flex_heptane.md`, `phys_acme_normal.md`,
  `phys_acme_inverse.md`, `phys_bass.md`. Their literature helpers ran on `claude-opus-5-5` and
  `claude-fable-5-2` (`dossier/registers/agents.md`).
- **Four independent re-derivations** by fresh agents with their own code on train
  (`claude-fable-5-2`, high; 11:39): `feedback/verify_phys_flex_heptane.md`,
  `verify_phys_acme_inverse.md`, `verify_phys_acme_normal.md`, `verify_phys_bass.md`.
- **Every quoted property value looked up** in standard sources: `research/properties_verified.md`.
- **Train measurements:** FLEX flame cameras (`feedback/flex_flame_traces_train.md`), ACME raw
  125 Hz channels and script files (`feedback/acme_raw_signals_train.md`), the earlier BASS campaign
  PSI-26 measured and verified (`feedback/bass1_psi26_tracks.md`; ten tracks in
  `tools/wsp/bass_optical/tracks/external_psi26/`), and a covariate shift map of validation inputs only
  (`feedback/shift_map_validation_inputs.md`) (`PROTOCOL.md §10`).
- **Two new inputs in the private data root** `runs/dev/data_ext/` (not in git): flame-camera series
  of 156 droplet runs (49,525 UV and 34,675 visible flame-diameter observations; liquid diameters
  byte-identical to the shared root), and the condition `programmed_stop` on 108 normal and 97 inverse
  runs, written by `tools/wsp/ext/acme_script.py` from NASA's "as run" script parameter files
  (`dossier/02_data_classification.md §2.6`).
- **Forwards.** `droplet6.rs` and `front2.rs` written by delegated engineers (`claude-fable-5-2`) to
  the author's specification, each equal to its predecessor exactly with the new entries off (by
  test); `committee.rs` (up to six member releases with a weight each, plus a second weight vector and
 an offset for prefixes that carry the planned stop) (`PROTOCOL.md §10`).
- Burner-flame candidates frozen (14:03); all candidates and rules frozen (14:52);
 outcome (15:33).

**Frozen rules** (`PROTOCOL.md §10`). Droplets: the session rule first, then replace the carried model
when validation CRPS is not higher on the same cases (private root); if both heptane candidates pass,
the lower validation CRPS is taken. Burner flames: replace unless validation extinction is worse,
applied twice to the same release, with the planned stop (private root) and without it (shared
records). BASS: served if validation tip score is not higher than D1's and track-ended is not worse.

**How the look ran.** `tools/wsp/dev_eval.py` with sealed forecasts and `ignis eval compare
--intersect`. Droplets were scored on the private root against their own vault
(`IGNIS_HOLDOUT=runs/dev/holdout_ext`; 71 methanol and 43 n-heptane cases deposited by the evaluator's
own command). Burner flames were sealed on both roots; `tools/wsp/analysis/cmp_split.py` split the
evaluator's comparison into the two tables and reproduced the evaluator's numbers to 6e-17. The
fabric flame was scored once on the sealed tracks (`RESULTS_GEN5.md`; `PROTOCOL.md §10`).

### 9.1 ACME inverse: committee with a hold scenario, adopted

*Mechanism.* Inverse flames end by oscillation: 33 of the 34 propane and n-butane train flames that
went out after 5 s showed bursts of radiant output first. The mixture fraction separates outcomes far
better than total flow (16 flames in the ambiguous range against 45); inside its marginal band a flame
at a high oxygen flow was held twice in two. This became a second scenario (`acme_inverse.wsp3` D5)
carried with weight one third beside the carried law (weights 2 : 1), plus member variants without
guessed stops for prefixes that carry the planned stop (offset 0.4 s)
(`EXPERIMENTAL_SCIENCE.md §7.2`; `D-26`, `D-44`).

| Release | Id | Train ext. / op. | Validation, planned stop known | Validation, shared records |
|---|---|---|---|---|
| **`acme_inverse.wsp5` P1** | `sha256:aa05ddcbe976dbb1a5537987d05270b28066551c39ab24db671be65692dcea0e` | 0.0281 / 0.0341 (with stop); 0.0271 / 0.0608 (without) | **0.1348 / 0.0478** | **0.1288 / 0.0880** |
| carry of `wsp3` D1 (carried model) | `sha256:68c840f36971680d7b46fa16de4c62e79190215cfa82930787c6b951af3a80db` | 0.0357 / 0.0639 | 0.1606 / 0.0939 | 0.1606 / 0.0939 |
| baseline set (reference) | `sha256:d7260f5962954904503a756a8f529b40988697aeae2ea57d18c6b6bfa1e0ea5d` | 0.3148 / 0.1975 | 0.2556 / 0.1596 | 0.2556 / 0.1596 |

Source: `RESULTS_GEN5.md` (54 cases, 21 runs). **Adopted on both tables.** Cluster-equal difference,
carried minus P1, shared records: +0.0306.

### 9.2 ACME normal: committee reading the planned stop, adopted

*Mechanism.* The plan of a test is known before ignition: NASA's script parameter file holds the stage
durations, and on train a test that ran to its scripted end stopped at their sum minus 0.4 s on the
record's clock (M-10; 29 scripted ends, median −0.4 s, `dossier/09_reproduce.md §9.5`). With the plan
in the prefix, the guessed stop atoms are silenced. The burner limit changed with the session (about
500 °C March to June, about 450 °C in October); the limit weights of the growth member were set to the
share of train sessions under each (`EXPERIMENTAL_SCIENCE.md §7.2`; `D-27`).

| Release | Id | Train ext. / op. | Validation, planned stop known | Validation, shared records |
|---|---|---|---|---|
| **`acme_normal.wsp5` P1** | `sha256:1336c6307fc9023f2434994f49e56cada9ff844000b3ebda29e23bc6bb9a77f4` | 0.0536 / 0.0215 (with stop); 0.0518 / 0.0274 (without) | **0.1452 / 0.0747** | 0.1552 / 0.0968 |
| carry of `wsp4` D1 (carried model) | `sha256:1ac0af0b7756b9d62ca243fb624e355837d68d2ca34228b93f2860ba0131f5c7` | 0.0517 / 0.0275 | 0.1553 / 0.0984 | 0.1553 / 0.0984 |
| baseline set (reference) | `sha256:67aea1d70bc802554ae33a5855421123d4c130622c10407789ad31a1ca875fc4` | 0.1507 / 0.0681 | 0.1809 / 0.0994 | 0.1809 / 0.0994 |

Source: `RESULTS_GEN5.md` (84 cases, 28 runs). **Adopted on both tables** (on the shared records level
with the carried model, 0.0001 better).

### 9.3 FLEX: physics-first droplet laws, carried models stay

*Mechanisms written* (`EXPERIMENTAL_SCIENCE.md §7.1`): an unlit methanol droplet evaporating on its
stored heat, `K0 / (1 + t/t0)` with `t0 = T* c_l D² / (1.5 K0 L)` and property-table constants
(K0 0.30 mm²/s, T* 16 K, c_l 2650 J/kg/K, L 1.118 MJ/kg); the methanol critical diameter capped from
18 % oxygen and falling with pressure above 1 atm; the n-heptane cool flame at 0.345 mm²/s ending at
`D/D0 = 0.47 sqrt((0.18 − β)/(pO2 − β))`, β near 0.12 atm; a flame born below its end diameter dies
away; flame cameras as evidence of state; a heavy-tailed radiative clock (`D-17`, `D-18`).

| Candidate | Id | Train mean / median | Validation | Decision, one line |
|---|---|---|---|---|
| `flex_methanol.wsp5` P1 | `sha256:eafcb57cb78250628cf44a9b3b41887a4202f013e0fe65366a49dd46ee5d58e9` | 28.7 / 17.5 µm (carry 29.9 / 17.7) | 42.5 µm against 39.5 (65 cases) | not adopted: no gain shown, paired −3.1 [−12.1, +4.6] |
| `flex_methanol.wsp5` P0 | `sha256:4d873aa6…df8a` | 29.7 / 18.2 | not scored | failed the train rule by 0.5 µm on the median |
| `flex_heptane.wsp5` P0 | `sha256:9a108f1a7fbf6fe56161f5108bff1a66b824a0d0171ad70060a44cb5185fac0b` | 43.9 / 19.8 µm (carry 46.2 / 21.8) | 47.2 µm against 46.1 (43 cases) | not adopted: within noise |
| `flex_heptane.wsp5` P1 | `sha256:57fa63b20d00e53c9e16186fec09845039b70659bd5d0ef7a3896907e02f78ce` | 44.6 / 21.0 | 48.4 µm against 46.1 | not adopted: cameras added no gain |

Source: `RESULTS_GEN5.md`; `PROTOCOL.md §10`. What this generation established for droplets: laws
rebuilt from mechanism with constants from property tables match the laws typed from train, with
fewer numbers read from train (`README.md`, "What five generations taught").

### 9.4 BASS: narrow-sample correction, D1 stays

| Candidate | Id | Train | Validation tip / track ended | Decision, one line |
|---|---|---|---|---|
| `bass_front.wsp2` P1 (D1 with the narrow sample corrected from two PSI-26 burns) | `sha256:d53e2944b62863014b875a8023b57b50ca28095f4d69c86e1a0c28fcf49d429b` | 0.076 (identical to D1); PSI-26 burns 0.157 against 0.220 | 0.1137 / 0.1976 against D1 0.1168 / 0.1780 | not adopted: track-ended score worse |

Source: `RESULTS_GEN5.md` (12 burns, 40 cases; tip paired −0.0031 [−0.0096, +0.0032]). Typed and
dropped after one train score each: slower flame growth, a weaker oxygen dependence, flow paths
(train 0.082 and 0.085 against 0.076). An oxygen balance with no adjusted constant gives the mean of
tip and base speed within about 10 % on the train windows (`EXPERIMENTAL_SCIENCE.md §7.3`).

**Also tried on train and not registered** (`PROTOCOL.md §10`): a pressure term for inverse flames
(train extinction 0.041 against 0.036); a methanol critical-diameter flattening from 15 % oxygen
(worse by 3 µm); the self-cooling law for n-heptane (size scaling fails after a cool flame by the
independent check); the total-flow scenario for inverse flames (weakened by a recount).

**Carried after generation 5:** `flex_methanol.wsp2` D3, `flex_heptane.wsp2` D2, `bass_front.wsp` D1,
`acme_normal.wsp5` P1 `sha256:1336c630…77f4`, `acme_inverse.wsp5` P1 `sha256:aa05ddcb…ea0e`
(`PROTOCOL.md §10`).

---

## 10. After the fifth generation, 14 November evening

### 10.1 Transfer of the frozen burner-flame models to ACME Flame Design (PSI-10)

Registered (17:28) before any outcome was read; scored (18:40)
(`PROTOCOL.md §11.1, §11.3`). Releases. Flame Design burns ethylene, a fuel the
models do not know; Arm B ran ethylene with ethane's indicators as a declared surrogate. The scoring
chain first reproduced the CFI-G train numbers to within 1e-16; the endpoint method agreed with the
table label on 148 of 153 CFI-G train tests. Its four test days were left out by an amendment made
before any score.

| Arm B, without the planned stop | P1 | Carry | Reference set | Incidence baseline | P1 minus baseline, extinction |
|---|---|---|---|---|---|
| Normal flames, 86 tests, 9 sessions | 0.1203 / 0.3166 | 0.1201 / 0.3138 | 0.1591 / 0.3232 | 0.2473 / 0.3336 | −0.1270 [−0.1816, −0.0722] |
| Inverse flames, 80 tests, 7 sessions | 0.1466 / 0.2599 | 0.1466 / 0.2599 | 0.1453 / 0.4681 | 0.2460 / 0.3413 | −0.0994 [−0.2089, +0.0116] |

By the rule written beforehand, **`acme_normal.wsp5` P1 carries over to ethylene as a surrogate**; the
inverse model is undecided. With the planned stop, flow termination fell by 0.2251 (normal) and 0.1437
(inverse) (`PROTOCOL.md §11.3`; `feedback/flame_design_transfer_result.md`). The same evening the
evaluation groups of the 15 other investigations were frozen (`PROTOCOL.md §11.2`).

### 10.2 Second author: Astra's first extension (train only)

| Train metric, lower is better | Fable reference | Astra R1 | Equal joint |
|---|---|---|---|
| Methanol diameter CRPS, µm | 29.8681 | 29.9267 | 29.8979 |
| Heptane diameter CRPS, µm | 46.1562 | 46.3022 | 46.2205 |
| Normal ACME extinction IBS | 0.051752 | 0.052725 | 0.052232 |
| Inverse ACME extinction IBS | 0.027091 | 0.028194 | 0.027624 |
| BASS tip CRPS, sample widths | 0.076101 | 0.080217 | **0.073439** |

Source: `dossier/ASTRA_RESULTS.md` (ACME on the shared records). Decision: Fable's carried models
kept; the BASS joint improved train tip CRPS; Astra candidates retained as experimental alternatives
(`dossier/04_decisions.md` Astra-D1, Astra-D2). Astra's later rounds (R2, the BASS matched-window
operator, 15 November) are covered in [EXPERIMENTS_GEN6-ON.md](EXPERIMENTS_GEN6-ON.md).

---

## 11. The carried model per family, generation by generation

Validation primary score of the carried model after each generation (development numbers; sources in
the sections above).

| Family | After gen 1 (best phase-1 model) | After gen 2 | After gen 3 | After gen 4 | After gen 5 |
|---|---|---|---|---|---|
| FLEX methanol (CRPS µm) | W0 50.4 zero-shot; W2 41.7 | D2 45.3 | **D3 40.0** | D3 (39.5 on the 59 common cases) | D3 (39.5 on 65 cases) |
| FLEX n-heptane | — | W0 60.3 zero-shot; D1 51.9 | **D2 46.1** | D2 46.1 | D2 46.1 |
| ACME normal (extinction / flow term.) | W0 0.219 / 0.125 | D2 0.170 / 0.099 | D3 0.159 / 0.100 | **blend 0.155 / 0.098** | **committee 0.145 / 0.075** with planned stop; 0.155 / 0.097 shared |
| ACME inverse | W1 0.177 / 0.124 | D1 0.146 / 0.097 | D1 0.146 / 0.097 | **growth 0.137 / 0.088** | **committee 0.135 / 0.048** with planned stop; **0.129 / 0.088** shared |
| BASS (tip CRPS, sample widths) | W0 runnable, no metric data | measurement study | target defined (3 tracked train burns) | **D1 0.117** (persistence 0.234), blind | D1 0.117 |

The fitted reference models on the same partitions: FLEX best fitted 77.7 µm (flex.v3, MAP); ACME
normal best fitted 0.185 / 0.093 (linear); ACME inverse best fitted 0.143 / 0.118 (hazard)
(`RESULTS.md`).

---

## 12. What transferred and what did not

From `README.md` ("What five generations taught"), `EXPERIMENTAL_SCIENCE.md §6.5, §7.4`.

**Transferred to unseen sessions:**

- A mechanism written in quantities that mean the same thing in another session: the three droplet
  states split at a critical diameter, the reduced flow of normal flames, the burner-heating law, the
  band-shaped flame of BASS.
- Two laws of different kinds answering together (normal flames, generation 4).
- A second physical scenario carried with a stated probability (inverse flames, 0.161 → 0.129).
- Knowing the plan of a test: the script's planned stop halved inverse flow-termination error and
  carried to a new investigation (Flame Design, §10.1).
- The structure of the published burner-flame law: the reference set with no CFI-G number scored
  0.181 on normal extinction, ahead of the best fitted model (0.185) (`PROTOCOL.md §9`).

**Did not transfer:** bands narrowed because the train runs looked that way (generation 4); a boundary
typed as a line from one burn on each side of it (BASS quench); averaging two laws that share their
main statement (inverse blend); flame cameras and a narrow-sample correction from another campaign
(generation 5).

---

## 13. Data classification and exposure ledger

**Classes** (`dossier/02_data_classification.md §2.2`): P prospective condition (readable), C control
(readable when started by the cutoff; BASS fan history is given up to each forecast time and declared
as such), O observation (only up to the cutoff), T target (never), R post-run fact (never; read for
train runs as measurement), X not used.

| Family | Key classifications | Source |
|---|---|---|
| FLEX | Droplet diameter O/T; tabulated atmosphere P (computed after the fill, independent of the run's own outcome); `kavg`, `Dext`, narrative R; position columns X (would change every prefix hash, `D-16`); flame cameras O in the private root only | `§2.3` |
| ACME | Fuel, pressure, `XO2`, `Xf`, `mHC`, `Tad`, `Zst` P; burn time and `SE`/`FT` label T; peak burner T, `Cool flame?`, `TFP` R; `_TP.txt` stamps R; "as run" script parameter file **P by this lane's judgment** (`programmed_stop`, private root) | `§2.4` |
| BASS | Top-view stills and duct video O/T as measured by this lane; fan overlay C; oxygen and sample class P; `Air display` R; PSI-26 external | `§2.5` |

**Exposure ledger, generations 1 to 5** (`dossier/07_limits_and_disclosures.md §7.1`, X-01 to X-22).
In brief:

| Ids | What | Assessment in the ledger |
|---|---|---|
| X-01 to X-04 | Experiment and literature dossiers carried whole-campaign aggregates (FLEX summary table, PSI-159 label counts and the 66 s stop cluster) and, for BASS-II, published SIBAL results; later rule 5 forbids per-test results of the modelled series | BASS D1 quench law possibly near the published boundary; that law was the part that failed, so it bought nothing. Others: no number |
| X-08, X-16, X-17, X-19 | Accidental reads by agents and helpers (rows of non-train tests printed, search snippets) | Not used in any report or number |
| X-09, X-18 | The author read validation **inputs**; consecutive conditions in one chamber fill encode the previous test's outcome | Not computed by anyone |
| X-10 to X-15 | Pretraining recall, one mistaken scoring of an unlooked candidate (row removed unread), private-root validation series held but unread, physics-dossier helpers on another model, memory-sourced properties (looked up before typing) | No number typed from them |
| X-20 to X-22 | Generation-5 look on a private vault; `cmp_split.py` reading per-case scores to print aggregates; forecast-to-forecast comparison on validation prefixes | No outcome read |

**Validation looks through generation 5** (`dossier/07_limits_and_disclosures.md §7.2`): FLEX methanol
about eight aggregate looks, n-heptane about six, ACME normal and inverse about eight each, BASS three.
The validation numbers on this page are therefore development numbers; the test partition was the
clean measure kept unread for the final run (see [EXPERIMENTS_GEN6-ON.md](EXPERIMENTS_GEN6-ON.md)).

---

## 14. Agents and authors

| Role | Model | Generations | Source |
|---|---|---|---|
| Author of every weight, generations 1 to 5 | `claude-fable-5-2` (Anthropic) | 1–5 | `meta.json` of each candidate; `cards/*` |
| Research and experiment dossiers, train audits, literature reviews, BASS measurement (train and blind validation), forward wiring, tests, model service | `claude-opus-5-5`, medium effort | 1–4 (and helpers in 5) | `dossier/registers/agents.md` |
| Physics dossiers, independent re-derivations, property look-up, `droplet6.rs` and `front2.rs` engineers | `claude-fable-5-2`, high effort | 5 | `dossier/registers/agents.md` |
| Second author and reviewer | GPT-6 Astra | 14 Nov evening onward | `dossier/README.md`; `dossier/ASTRA_RESULTS.md` |

55 delegated agents are registered for 13–14 November. No agent and no script wrote a weight
(`D-04`).

---

## 15. Reproduction

From the repository root (`bash`; Git Bash on Windows). Never pass `--final-test`, `--partition test`
or `--partition all` (`dossier/09_reproduce.md`).

```bash
cargo build --release -p ignis-cli -p ignis-wsp-dev
cargo test -p ignis-wsp -p ignis-wsp-dev -p ignis-core
```

Rebuild every release from the committed edits in lineage order; prints whether each weight hash is
unchanged (ids are identical by construction):

```bash
python tools/wsp/reassemble.py
target/release/ignis wsp inspect --release sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654 --edits
```

Authoring chain of one candidate (`PROTOCOL.md §6`):

Train feedback and the droplet selection rule:

```bash
target/release/wsp-dev feedback --release sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654 --partition train --out runs/dev/review_fb_methanol.json
python tools/wsp/analysis/fb_summary.py runs/dev/review_fb_methanol.json
python tools/wsp/analysis/session_compare.py runs/dev/fb_train_flex_d3.json runs/dev/analysis/flex_methanol_profile.json 0 1
target/release/wsp-dev front-feedback --release sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c --tracks tools/wsp/bass_optical/tracks/train --label review-train --out runs/dev/review_front_train.json
```

Development tables, train only (a run without `--skip-validation` is a validation look and must be
registered in PROTOCOL first):

```bash
python tools/wsp/dev_eval.py --baselines runs/dev/baselines.json --skip-validation --out runs/dev/review_results_train.md
```

`dev_eval.py` needs the fitted baselines in the local registry (`ignis fit run <model> --cohort …
--split … --execute`; FLEX D² baselines with `--set fit.skip_invalid_condition_runs=true`) and the
validation continuations (`ignis eval deposit-cohort … --partition validation`) (`PROTOCOL.md §6`).

Private data root of generation 5 (`dossier/09_reproduce.md §9.6`):

```bash
cp -r data runs/dev/data_ext
IGNIS_DATA=runs/dev/data_ext target/release/ignis data ingest flex
python tools/wsp/ext/acme_script.py --root runs/dev/data_ext --fetch
```

The planned-stop check (M-10) and the inverse-separation recount (M-17) are in
`dossier/09_reproduce.md §9.5`; expected output `scripted ends: 29 … median -0.4`. The
generation-5 look itself (already taken; rerunning reads the same seals) is in
`dossier/09_reproduce.md §9.7`.

---

Continued in [EXPERIMENTS_GEN6-ON.md](EXPERIMENTS_GEN6-ON.md): generation 6 (the overnight run of
15 November), generation 7 and the final test, published-model comparisons, and the coverage of all
19 NASA investigations.
