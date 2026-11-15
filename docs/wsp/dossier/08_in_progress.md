# 8. The fifth generation (14 November 2026): working record and outcome

**Finished.** The candidates were registered with their rules in [PROTOCOL.md](../PROTOCOL.md)
section 10, validated once, and decided by those rules: section 8.7 below and
[RESULTS_GEN5.md](../RESULTS_GEN5.md). Sections 8.2 to 8.6 are the working record as it was written
while the work was under way and are kept as they were; where a statement there was overtaken,
section 8.7 says what became of it.

## 8.1 What the owner asked for

On 14 November: the models are not good enough; research the NASA data exhaustively and extend the
models; do not overfit, build something that generalises; analyse with claude-fable-5-2 at high
effort; "they can be many, many parameters if it is added physical logic … actually understand what
they are meant to do, not just mathematically fit stuff". My reading: each model is rebuilt from the
physical mechanism of its experiment; a parameter needs a physical meaning; a term that only
matches a train table is dropped or explained.

## 8.2 What exists

| Piece | Files | State |
|---|---|---|
| Droplet forward with flame cameras and a heavy-tailed radiative clock | `crates/ignis-wsp/src/droplet5.rs`; models `flex_methanol.wsp4`, `flex_heptane.wsp4` (keys `flex_methanol4`, `flex_heptane4`); tests `tests/droplet5_forward.rs` (six, passing) | Built. With `rob.eps_r = 0` and `fl.trust = 0` it returns exactly what `droplet3.rs` returns (tested) |
| Droplet candidates | `configs/models/flex/wsp4/{methanol,heptane}/`: `baseline-d2` (root), `carry-d3` or `carry-d2` (the carried numbers on the new forward), `r1-fable` (carried numbers + `rob.eps_r = 0.1`), `f1-fable` (R1 + cameras: `fl.trust = 0.95`, `fl.t_min_s = 0.3`, `fl.gap_s = 0.25`, `fl.min_points = 5`, `fl.evap_mix = 0.5`) | Assembled. Ids: methanol R1 `sha256:c8a0c5fe…b6ae`, F1 `sha256:ae34b546…c89f`; heptane R1 `sha256:8279cbcf…f95b`, F1 `sha256:5af6e32a…0d0d` |
| Committee forward for burner flames | `crates/ignis-wsp/src/committee.rs`; models `acme_normal.wsp5`, `acme_inverse.wsp5`; six member slots (two hinge-law, four growth-clock) and a weight per slot; optional reading of `programmed_stop`; tests `tests/committee_forward.rs` | Built and patched for the script stop; roots, specs and tests regenerated on 14 November (six tests passing) |
| Inverse hold scenario | `configs/models/acme/wsp3/inverse/d2-fable/` (four statements over the carried D1; `sha256:3e4f850a…c3168`) | Assembled. A second member for the committee, not a candidate by itself |
| Committee candidates | `configs/models/acme/wsp5/{normal,inverse}/baseline-set`, `wsp5/inverse/d1-fable` (the carried inverse law and the hold scenario, weights 1 : 1) | Assembled: normal root `sha256:67aea1d7…5fc4`, inverse root `sha256:d7260f59…ea5d`, inverse D1 `sha256:fe1470e0…1dcb`. The script stop is switched off in all three (`mix.w_script` all zero) |
| Private data root | `runs/dev/data_ext/` (not in git); `tools/wsp/ext/acme_script.py` | Flame cameras ingested for 156 droplet runs; `programmed_stop` written for 108 normal-flame runs, **none for inverse yet** |
| Physics dossiers | five agents on claude-fable-5-2, high effort: methanol, n-heptane, ACME normal, ACME inverse, BASS; a sixth verified and completed the PSI-26 measurement | Finished and saved: BASS (M-14), n-heptane (M-15), ACME inverse (M-16), ACME normal (M-18), PSI-26 (M-11). Methanol was **running** when this page was written. Their synthesis is [10_extension_map.md](10_extension_map.md) |

## 8.3 What train says so far

Train numbers of models typed from train: an honesty check, not evidence of generalisation.

| Candidate | Data | Train CRPS, mean / median (µm) | Against the carried numbers on the same forward |
|---|---|---|---|
| methanol R1 | shared | 30.1 / 17.7 | +0.20 mean; 1 session better, 2 worse, 23 level |
| heptane R1 | shared | 46.5 / 21.8 | +0.31 mean |
| methanol F1 | private root (cameras) | 28.9 / 17.5 | −0.97 mean; largest single gain −146.8 µm (C12M401 at 3 s) |
| heptane F1 | private root (cameras) | 46.2 / 21.8 | +0.08 mean; largest loss +30.2 µm (H13H101 at 5 s, helium), largest gain −37.1 µm |

| Candidate | Train extinction / flow-termination Brier | Carried |
|---|---|---|
| inverse committee D1 (before the script patch) | 0.026 / 0.060 | 0.036 / 0.064 |

What the heptane result shows: once the camera says the flame is lost in helium, the carried law
for an unlit droplet under-predicts the shrinking that follows; and the 2 atm CO2 run has a hot
flame the radiative clock calls impossible. These are faults of the laws, which the cameras expose
and do not cure. That is the reason for the physics dossiers.

## 8.4 Known inconsistencies, to be fixed before anything is registered

1. **`script.onset_s`: resolved on 14 November.** The roots had typed 2.4 s (flame onset after the
   script's ignition stamp) while the stored `programmed_stop` is the bare stage sum, for which a
   scripted stop falls at the sum minus 0.4 s after flame onset on train (M-10). The stored condition
   stays what the file says; the tensor is now described as the offset between the stage sum and the
   record's clock; the roots, which hold no train number, type 0.0 and do not read the script; the
   candidate that switches the script on will type 0.4 as a data-informed number.
2. **Guessed stop atoms beside a known stop.** When the script's stop is known, the members should
   answer without their stop atoms. That needs member variants with the atoms off and typed
   `mix.w_script` weights. Not done.
3. **`acme_script.py` for the inverse family** has not been run to completion.
4. **Limit by date.** The burner limit was about 500 °C until late August 2021 and about 450 °C after
   (C-22); the heating law has more than one time constant (C-20). Neither is in a forward.
5. **The inverse hold scenario** (D-26) is written in total burner flow, which M-16 and M-17 say is the wrong variable; the committee candidate that carries it at half weight is not to be registered as it stands.
6. **The camera rule and view censoring** (C-12): six train runs lose the flame from the field of
   view, not from the droplet.
7. **Fetch commands of the private root** are not recorded in a script ([09](09_reproduce.md), 9.6).

## 8.5 What happens next, and what a reviewer can influence now

1. Read the five physics dossiers; recheck on train every statement that would become a number.
2. Write mechanistic forwards (new files, new model names). Keep from the carried models what the
   physics supports; drop the unexplained corrections or carry them as stated uncertainty.
3. Type the constants: literature values where they exist, train where a constant is specific to
   the apparatus, with the origin class of [06](06_models.md) beside each.
4. Register the candidates and their rules in PROTOCOL before any validation look; one look.

A review is most useful before step 4: a finding on a claim of [05](05_claims.md) or on a
classification of [02](02_data_classification.md) changes what gets typed. After a validation look
it can only change what is said about the result.

## 8.6 Physics-first candidates (working record, 14 November, later)

After the owner's "fix them all". Still **not registered and not run on validation** until section
10 of PROTOCOL says otherwise.

**Checks commissioned.** One independent re-derivation per physics dossier, each with its own code
on train (claude-fable-5-2, high effort), saved as `feedback/verify_phys_*.md`; a look-up of every
property value against standard sources (running). What they changed is recorded per family below.

**Burner flames (built by the author, weights only; no new forward).**

| Candidate | What it is | Train, extinction / flow termination (evaluator's times 6 to 120 s) |
|---|---|---|
| `acme_normal.wsp4` D1, carried | hinge law and growth clock, equal weights | 0.0517 / 0.0275 |
| `acme_normal.wsp3` D2 `sha256:edf28660…8ae8` | growth member with the burner-limit weights 0.15 (about 450 °C) and 0.85 (about 500 °C): the share of normal-flame train sessions under each setting | member |
| `acme_normal.wsp5` P1 `sha256:1336c630…77f4` | slots: hinge D3, growth D2, and the same two without guessed stops; `mix.w = [1,0,1,0,0,0]`, `mix.w_script = [0,1,0,1,0,0]`, `script.onset_s = 0.4` | shared data: 0.0518 / 0.0274. With the script's planned stop (private root): 0.0536 / 0.0215 |
| `acme_inverse.wsp3` D1, carried | | 0.0357 / 0.0639 |
| `acme_inverse.wsp3` D3 `sha256:cca9e531…fcd6` | D1 + a pressure term (+0.5 per e-fold) | 0.0406 / 0.0659: **worse, not taken**; kept as a record. The independent check finds no pressure dependence of the onset between 1.1 and 2.2 bar either |
| `acme_inverse.wsp3` D5 `sha256:bf575e19…d45f` | D1 + one unit: in the marginal band of the mixture fraction a flame at a high oxygen flow is held (unit 3: `W1[3,1] = 1`, `W1[3,5] = -0.6`, `b1[3] = -0.2`, `w2[3] = 20`) | 0.0198 / 0.0581 (a scenario, not the belief) |
| `acme_inverse.wsp5` P1 `sha256:aa05ddcb…ea0e` | slots: D1 (weight 2), D5 (weight 1), and the same two without guessed stops for the script path; `script.onset_s = 0.4` | shared data: 0.0271 / 0.0608. With the script's planned stop: 0.0281 / 0.0341 |

Reading the script's stop makes the extinction score of normal flames slightly worse on train
(+0.002) and the flow-termination score better (−0.006): a flame that went out a few seconds before
its planned stop is now given a chance of having been stopped instead, which is the honest
statement under the model's own clock uncertainty. For inverse flames both scores improve.
`tools/wsp/ext/acme_script.py` has now written `programmed_stop` for 108 normal and 97 inverse runs
of the private root (train and validation; counts only were printed).

What the independent checks added for burner flames (not typed, recorded for the next author):
21270H3 is labelled as a flow termination at 43 s and in fact went out by itself at 93 s; a seventh
normal row is mis-tabulated (21125D2, fuel fraction 0.037 not 0.045); the thin filament is
knowable before ignition from the script parameter file (`fiberTestPosition`); the pressure effect
on normal-flame lives is about p^-0.85 near the holding flow, not p^-0.4 everywhere; a dilute burner
gas does not burn longer in raw lives; the burner limit shows 3.2 to 4.6 s of delay at 500 °C and
0.5 to 1.2 s at 450 °C.

**Droplets and fabric flame (two new forwards, written by delegated engineers to the author's
specification, neutral by test: `droplet6.rs`, `front2.rs`).** The candidates, their train numbers
and the rules are in [PROTOCOL.md](../PROTOCOL.md) section 10. What the train diagnosis did to the
plan, change by change (each typed change was assembled as a scratch release and scored alone on
train; `runs/dev/gen5/variants/`):

| Change | Alone on train (mean CRPS against the carry, µm) | Kept? |
|---|---|---|
| methanol: self-cooling evaporation, property-table constants | 29.8 against 29.9; 6 runs better, 6 worse | yes: it replaces three typed train numbers at an equal score |
| methanol: critical diameter flattened from 15 % oxygen | 3 µm worse; it broke a cell train had pinned (18 % oxygen in 40 % CO2 at 0.7 atm) | no; replaced by a cap from 18 % oxygen (level on train) |
| methanol: critical diameter falling with pressure above 1 atm | level (4 runs better, 1 worse) | yes |
| methanol: no pressure term in the water extinction | level | yes |
| methanol: flame cameras and heavy-tailed clock | 28.9 against 29.9 | yes (P1) |
| heptane: cool-flame rate without oxygen and pressure terms | 1.0 worse, all of it on the one run at 2 atm | yes, with a term above 1 atm for the warm flame of that run: then 2.0 better |
| heptane: cool-flame end in oxygen partial pressure | 0.3 better; 2 runs better, none worse | yes |
| heptane: dying flame in place of the helium patch | 2.4 worse alone | yes, together with the next: the carried model holds the helium flame too long and then evaporates too slowly, and the two errors cancel |
| heptane: helium in the radiative clock | 2.4 worse alone; with the dying flame 0.3 better and the median 20.3 against 21.8 | yes |
| heptane: flame cameras and heavy-tailed clock | level alone; 0.7 worse on top of the laws | registered as P1 beside P0 |
| fabric: slower growth of the flame | 0.082 against 0.076 sample widths, worse on all three burns | no |
| fabric: oxygen power 1.0, flow paths | 0.085 | no |
| fabric: narrow-sample powers | no train burn is narrow; 0.157 against 0.220 on three burns of the earlier campaign (the burns the powers were typed from) | yes (P1) |

Three process notes for a reviewer. The scratch variants are single physical statements scored
once each, not a search over values; but six of them were dropped or reshaped because of a train
score, which is a use of train that the candidates' headers state. The earlier campaign's tracks
had to be relabelled in a scratch copy (partition field set to train, band width converted to the
proof-sheet scale) before the scorer would read them; the files in git are untouched. The methanol
candidate without cameras fails the train rule by 0.5 µm on the median and is not scored on
validation.

## 8.7 Outcome of the one validation look (14 November, evening)

| Family | Candidate | Validation | Carried model on the same cases | By the rule written beforehand |
|---|---|---|---|---|
| FLEX methanol | `flex_methanol.wsp5` P1 | CRPS 42.5 µm | 39.5 | not adopted |
| FLEX n-heptane | `flex_heptane.wsp5` P0, P1 | 47.2, 48.4 µm | 46.1 | not adopted |
| ACME normal | `acme_normal.wsp5` P1 | extinction 0.1452 / flow termination 0.0747 with the planned stop; 0.1552 / 0.0968 without | 0.1553 / 0.0984 | **adopted**, both ways; without the planned stop it is level |
| ACME inverse | `acme_inverse.wsp5` P1 | 0.1348 / 0.0478 with; 0.1288 / 0.0880 without | 0.1606 / 0.0939 | **adopted**, both ways |
| BASS | `bass_front.wsp2` P1 | tip 0.1137, track ended 0.1976 | 0.1168, 0.1780 | not adopted |

How I read it.

- **The plan of a test is worth more than a better law of the flame.** The planned stop halved the
  flow-termination score of inverse flames and took a quarter off that of normal flames. It also
  costs a little at extinction where the guessed stops had sat, by accident, near real extinctions.
- **A second scenario with a stated probability carried over** (inverse flames, 0.161 to 0.129 on
  the shared records). Two validation sessions; I hold it at medium confidence (C-44).
- **The droplet laws rebuilt from mechanism were not shown to be better or worse.** Better on train
  by 1 to 2 µm, worse on validation by 1 to 3 µm, intervals of ±5 to ±10 µm. The rule refuses, and I
  think it is right to: the claim that a law is better founded is a claim about its reasons, and a
  reader can check those in the edits file without a score. What I do not know is how the two of us
  should choose between two models that score the same (10.5 of the extension map).
- **The flame cameras gave nothing on unseen days**: the heptane candidate with cameras is 1.2 µm
  worse than the same laws without them, and its 90 % band covered 0.88.
- **The fabric candidate** changed only the narrow sample, from two burns of another campaign. Its
  tip score moved by 0.003 inside its interval and its track-ended score got worse; refused.

What became of the open items of 8.4:

1. `script.onset_s`: typed as 0.4 in both adopted candidates.
2. Guessed stops beside a known stop: done. Member variants with the stop atoms off
   (`d3-nostops`, `d2-nostops`, `d1-nostops`, `d5-nostops`) take the weight when the prefix carries
   the planned stop.
3. `acme_script.py` for inverse flames: run. The private root carries the planned stop for 108
   normal and 97 inverse tests.
4. Limit by date: in part. The weights of the two limits are corrected in the growth member of the
   normal committee. A forward still cannot see the date of a test, and the heating law still has
   one time constant.
5. The inverse hold scenario: rewritten in the mixture fraction (`acme_inverse.wsp3` D5) and carried
   at one third. The total-flow version stays unregistered.
6. The camera rule and view censoring (C-12): open. No carried model reads the cameras.
7. Fetch commands of the private root: open ([09](09_reproduce.md), 9.6).
