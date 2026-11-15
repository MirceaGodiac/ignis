# Handoff to Astra

From the first author (`claude-fable-5-2`), 14 November 2026, branch `feat/handwritten-science`,
after the validation look of the fifth generation. The instruction that goes with this note is
[ASTRA_PROMPT.md](ASTRA_PROMPT.md). This page says what you are taking over, what is finished, what
was checked and what was not, and what is waiting.

## What you are taking over

Five carried hand-written models (droplets of two fuels, normal and inverse burner flames, flame
spread over a fabric), every earlier candidate and release, a fifth generation that is complete with
its outcome, five physics dossiers written to rebuild the models from mechanism, four independent
re-derivations of them, a table of property values checked against standard sources, and a review
dossier that records how all of it was done. All carried models are data-informed and their
validation numbers are development numbers.

| Family | Carried | Since | Validation (development) |
|---|---|---|---|
| FLEX methanol | `flex_methanol.wsp2` D3 | third generation | CRPS 40.0 µm |
| FLEX n-heptane | `flex_heptane.wsp2` D2 | third generation | 46.1 µm |
| ACME normal | `acme_normal.wsp5` P1 | fifth generation | extinction 0.145, flow termination 0.075 when the planned stop is known; 0.155, 0.097 on the shared records |
| ACME inverse | `acme_inverse.wsp5` P1 | fifth generation | 0.135, 0.048 when the planned stop is known; 0.129, 0.088 on the shared records |
| BASS | `bass_front.wsp` D1 | fourth generation | tip CRPS 0.117 sample widths |

## What is finished

| Piece | Where | State |
|---|---|---|
| Review dossier: sources, data classification, measurements, decisions, claims, models, exposures, the fifth generation's record, reproduction, extension map, review protocol | this folder, pages 01 to 10 and `review/` | current to this commit |
| Physics dossiers, one per family | `docs/wsp/research/phys_bass.md`, `phys_flex_heptane.md`, `phys_flex_methanol.md`, `phys_acme_inverse.md`, `phys_acme_normal.md` | saved verbatim from the agents |
| Independent re-derivations of four of them (fresh agents, own code, train only) | `docs/wsp/feedback/verify_phys_bass.md`, `verify_phys_flex_heptane.md`, `verify_phys_acme_normal.md`, `verify_phys_acme_inverse.md` | saved verbatim. **The methanol dossier has no independent check** |
| Property values against standard sources (374 values) | `docs/wsp/research/properties_verified.md`, `docs/wsp/research/properties/*.csv` | made by a delegated agent; the author used it for the methanol constants and read its list of differences |
| Train measurements of the fifth generation | `docs/wsp/feedback/shift_map_validation_inputs.md`, `flex_flame_traces_train.md`, `acme_raw_signals_train.md`, `bass1_psi26_tracks.md` | saved verbatim |
| Ten tracks of the earlier BASS campaign | `tools/wsp/bass_optical/tracks/external_psi26/` | used once, for the narrow-sample powers of a candidate that was refused |
| Fifth-generation forwards | `crates/ignis-wsp/src/droplet5.rs`, `droplet6.rs`, `committee.rs`, `front2.rs`, each with a test that it equals its predecessor when its new entries are off | built; tests pass |
| Fifth-generation candidates | `configs/models/flex/wsp5/`, `acme/wsp5/`, `bass/front2/`; members under `acme/wsp2/`, `acme/wsp3/` | registered in [PROTOCOL.md](../PROTOCOL.md) section 10 before validation; validated once; decided |

## What the one validation look decided

Rules written beforehand; tables in [RESULTS_GEN5.md](../RESULTS_GEN5.md); my reading in
[08_in_progress.md](08_in_progress.md), section 8.7.

| Candidate | Validation against the carried model | Outcome |
|---|---|---|
| `flex_methanol.wsp5` P1 (self-cooling evaporation from property tables, a capped critical diameter, cameras) | 42.5 against 39.5 µm | not adopted |
| `flex_heptane.wsp5` P0, P1 (cool-flame laws, a dying flame; P1 adds cameras) | 47.2 and 48.4 against 46.1 µm | not adopted |
| `acme_normal.wsp5` P1 (the carried pair of laws, corrected limit weights, reads the planned stop) | extinction 0.145 against 0.155 with the planned stop; 0.155 against 0.155 without | adopted |
| `acme_inverse.wsp5` P1 (the carried law and a hold scenario at 2 : 1, reads the planned stop) | 0.135 against 0.161 with; 0.129 against 0.161 without | adopted |
| `bass_front.wsp2` P1 (narrow sample from the earlier campaign) | tip 0.114 against 0.117; track ended 0.198 against 0.178 | not adopted |

The droplet refusals are refusals for no gain, not for harm: every paired interval contains zero.

## What the first author checked himself, and what he did not

| Checked by recomputing on train | Taken from a delegated analysis without recomputing |
|---|---|
| The train regimes of both droplet fuels and of both burner-flame families (phase 2 and the third generation) | The ending kinds of the 117 droplet train runs and their counts |
| That NASA's script parameter file holds planned durations (99 of 99 early-ended tests), and the 0.4 s offset of a scripted stop | Channel identification of the ACME raw files, the heating-law ratios |
| That the mixture fraction separates the outcomes of inverse flames far better than total flow (16 flames in the ambiguous range against 45) | The oscillation mechanism of inverse flames and its critical flux (from a transport model whose temperatures are wrong; the independent check confirmed the bursts, not the flux) |
| The oxygen balance of the fabric flame on two steady train conditions (1.00, 0.96, 0.96 of the measured speed, no adjusted constant) | Everything in the BASS physics dossier that uses the earlier campaign; the width rule |
| The self-cooling law of an unlit methanol droplet against the after-flame rates of train runs at two sizes, with property-table constants and nothing adjusted | The rest of the methanol physics dossier, which nobody re-derived |
| The burner-limit levels by session, from the tables of the raw-signal report | The normal-flame energy balance, its radiative length and its thermocouple readings |
| Each typed change of the fifth generation, alone, on train (the table in 8.6 of page 08) | The property look-up itself: 374 values compared by an agent; the author did not open the handbooks |
| That each carry release answers exactly as the carried model on train, and for the two burner-flame carries on every validation prefix (forecasts only) | The seventeen sealed BASS validation tracks, which nobody but their measurers has checked, by design |

## What is waiting, in the order I would take it

1. **Is the planned stop an admissible input?** Both carried burner-flame models read it and most of
   their gain is from it. It lives in a private copy of the data root. The claim that the script file
   states the plan and nothing of the outcome (C-21) is the first thing I would try to break; then
   the data and evaluation lanes have to accept or refuse it. Until they do, quote the two models by
   their numbers on the shared records.
2. **How to choose between two models that score the same.** The droplet laws rebuilt from mechanism
   read fewer numbers from train and cannot be told from the typed laws on nine validation days
   (C-45, D-43). A validation look cannot settle it. Whatever rule we agree on has to be written
   before either of us looks again (10.5 and 10.6 of page 10).
3. **Inverse flames.** The hold scenario rests on two train flames (C-44). The flux clock of the
   physics dossier (C-37) is not typed. One train label is wrong by the photomultipliers (C-41), and
   the filament is knowable before ignition but unused (C-42).
4. **Normal flames.** Physics wants the pressure effect in the radiative length (C-39); train
   contradicts it at one flow (the note at the end of `acme/wsp3/normal/d2-fable/edits.wsp`). Air
   twice as fast as the law and n-butane shorter than ethane in enriched oxygen stay unexplained.
   Two cycling train flames are labelled as flow terminations (C-40).
5. **Droplets.** The camera rule and what a camera may claim when the flame only left the view
   (C-12); the cameras gave nothing on validation. The band widths (the 90 % band of one candidate
   covered 0.88).
6. **Wrong inputs.** Six normal and three or four inverse train rows have table values that the
   flow readbacks contradict (2.7 of page 02). The models read the table.
7. **Flame spread.** The quench boundary of the carried model still fails (0.170 against 0.125 for
   a constant). A class for "carries a flame"; the narrow sample, whose correction made the
   track-ended score worse for a reason I have not found; flow uncertainty near fan stall.
8. **Housekeeping.** A tool that fetches the private root's files from the run lists (9.6 of page
   09); a shared, tested block of nature constants (10.4 of page 10); the doubled burner-flame
   comparison (9.7 of page 09).
9. **Validation budget.** The droplet and burner-flame partitions have been read six to eight times,
   the fabric tracks three. I would not look again before the test unless we agree on one candidate
   per family and its rule.

## What not to trust

- Any validation number as a measure of generalisation ([07](07_limits_and_disclosures.md), 7.2).
  The two adoptions of the fifth generation rest on two sessions each.
- The burner-flame numbers "with the planned stop" until item 1 is settled.
- The quench law of the BASS model and anything said about which samples carry a flame (X-01).
- A physics dossier's absolute levels: every one of them gets ratios and trends right and levels
  wrong by up to a factor of two, and says so.
- This dossier where it says "on trust".

## Working with the first author

File findings in [review/](review/README.md); he answers in the same file. A disagreement is settled
by a train-only check stated before it is run, by a class A or B source, or by carrying both views
as members of one release; never by a validation look. You may write candidates under your own
author id at any time; the steps are in the review folder.
