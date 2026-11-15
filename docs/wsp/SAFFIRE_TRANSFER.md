# Transfer of the frozen fabric-flame models to the Saffire sheets

Registered 15 November 2026 by the author (Fable), in a commit that precedes every measurement and
every score of this test. Conventions: [PROTOCOL.md](PROTOCOL.md) sections 8 to 10 (a forward is
frozen with its releases, rules are written before the look, validation acts as a veto and is read
once as aggregates) and section 11 (transfer tests outside the partitions: registered before any
outcome is read, the scoring chain first reproduces known numbers), and the frozen TRAIN comparison of
[dossier/ASTRA_FROZEN_TRAIN_20261114.md](dossier/ASTRA_FROZEN_TRAIN_20261114.md).

## 1. Why, and what it costs

The fabric-flame model was typed from three scored train burns of one day of BASS-II, a 2 cm strip of
cotton-fibreglass fabric (SIBAL) in a low-speed duct. Four candidates since the adopted one sit within
the noise of it, and the owner and the author decided that no further fabric model work is useful on
that data. The one test left that can say something new: Saffire I and III burned the same fabric as
sheets 40.6 cm wide and 94 cm long, about twenty times the width, in a flow of 20 and 30 cm/s, and
Saffire II burned two 5 cm strips of it.

Each Saffire flight is one evaluation group, held whole (`GR-FLIGHT`,
[atlas/SCHEMA.md](atlas/SCHEMA.md); [atlas/splits/PSI-98.json](atlas/splits/PSI-98.json), `PSI-99`,
`PSI-100`): no outcome of a held group is read per test, and it is scored only as an aggregate in a
registered look. **This look spends all three groups.** After it, any model written with knowledge of
its result treats Saffire as development data and says so. No model weight changes at any point of
this test, and nothing is tuned or run again after a score is seen.

## 2. Releases under test (frozen; not edited)

| Role | Model | Release | Folder |
|---|---|---|---|
| Reference, first in the call | `bass_front.wsp` persistence | `sha256:ce53fe1fb3e6300c5e818dd7c9c5ec8e8f4c2cfd19673449d3936de05527de46` | `configs/models/bass/front/baseline-persistence/` |
| **Decision model** (adopted) | `bass_front.wsp` D1 | `sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c` | `configs/models/bass/front/d1-fable/` |
| Reported beside it | `bass_front.wsp3` joint (Astra) | `sha256:ffbb724a068338c789a7276254d1b9e81ee18a435c760d215b21ac8336a3db37` | `configs/models/bass/front3/joint-astra/` |
| Reported beside it | `bass_front.wsp4` matched-window operator (Astra) | `sha256:d58751447917a425a5f3d39e2c4144fddc6151b8000874a282648603949e15e4` | `configs/models/bass/front4/window-matched-astra/` |

Persistence keeps the tip speed the prefix shows, whatever the flow does. The decision is taken on D1
alone, the model the project serves; the two Astra releases are scored on the same cases and
reported, with the same sentence of section 7 applied to each, and they decide nothing.

## 3. Which burns count

The models were typed for flame spread **in the direction of the flow** (concurrent) on this fabric.

| Burn | Sample | Flow | Role |
|---|---|---|---|
| PSI-98 Saffire I, S1 | SIBAL sheet 40.6 cm × 94 cm | concurrent, 20 cm/s | **wide stratum: decides** |
| PSI-100 Saffire III, S1 | SIBAL sheet 40.6 cm × 94 cm | concurrent, 30 cm/s | **wide stratum: decides** |
| PSI-98 Saffire I, S2 | SIBAL sheet | opposed | refused |
| PSI-100 Saffire III, S2 | SIBAL sheet | opposed | refused |
| PSI-99 Saffire II, 2-5 | SIBAL strip 5 cm × 29 cm | concurrent, 20 cm/s | strip stratum: reported, decides nothing |
| PSI-99 Saffire II, 2-6 | SIBAL strip 5 cm × 29 cm | concurrent, 30 cm/s | strip stratum: reported, decides nothing |
| PSI-99 Saffire II, 2-1 to 2-4, 2-7 to 2-9 | silicone, PMMA, PMMA and Nomex | | not this fabric: not measured, not scored |

- **Opposed burns.** The forward is a statement about a flame that runs downstream; an opposed flame
  runs into the flow and the model has no law for it. The track reader refuses them at the flow
  condition (`front_tools::conditions_of`), and that refusal is the model's answer. They get a track
  file with their conditions and no stills, so the refusal is counted by the scorer; their frames are
  not measured.
- **The two Saffire II strips** are 2.3 times the reference width, between the train strips and the
  sheets. They are scored and reported as their own stratum and are not part of the decision, which is
  about the wide sheet. Saffire II flow: the commanded value of its Readme Table 1
  (`tools/wsp/atlas/transcribed/psi99_saffire2_readme_table1.csv`): 20 cm/s for 2-5 and 30 cm/s for
  2-6, where the experiment table says 20 for 2-6 (a known defect, [atlas/INVESTIGATIONS.md](atlas/INVESTIGATIONS.md)).
- A burn counts in a stratum only if it yields at least one scored case (section 6). If fewer than two
  wide burns yield one, the test is reported as **not decided** with the number of burns that did.

## 4. Inputs, and why each is known before the forecast cutoff

| Input | Value | Source | Known before the cutoff because |
|---|---|---|---|
| Oxygen mole fraction | the catalogue's `atmosphere.x_o2` where it is one resolved value (Saffire II 2-5 and 2-6: 0.221); else 0.21, nominal air in the Cygnus cabin (Saffire I S1, Saffire III S1) | `runs/dev/data_all/work/atlas/tests.jsonl` (conditions; the outcome file is not opened) | a condition of the atmosphere before ignition. The oxygen *over the burn* is a held outcome and is not read |
| Width ratio | `geometry.sample_width_m` / 0.022 m: 0.406 m for the sheets (18.5), 0.05 m for the strips (2.27) | the experiment tables and the Saffire I and III Readme | a property of the sample. The divisor is `norm.w_ref_m` of the forwards, the width their metric route already divides by (`front_tools::rules::REF_WIDTH_M`) |
| Window length | sample length over width: 94 / 40.6 = 2.315; 29 / 5 = 5.8 | the same | a property of the sample |
| Flow history | one entry at or before igniter-on with the commanded speed (`u_cm_s`), held to the end; a commanded change stated in the Readme becomes a further entry | experiment table; Readme | the commanded flow is a control set before ignition. The measured flow series of the sensor workbooks is not read: it is not what the BASS fan history is (a known schedule), and it could carry the burn |
| Igniter | `on_t_s` = minus the tabulated igniter time (Saffire I 8 s, Saffire II 9.2 s); Saffire III, which tabulates none: the first frame of igniter glow | experiment table; frames | the igniter is off at the time origin, before every cutoff |
| Prefix | tip and base of the measured stills at or before the cutoff | the tracks | observations up to the cutoff only, as on BASS |

Time origin: the igniter going off, `t_s = 0`. The measurer finds it as the last frame in which the
igniter wire glows apart from the flame, or as the first glow plus the tabulated igniter time when the
two cannot be told apart; which was used is recorded per track.

Not used by anything in this test: burn time, burn length, spread rate, oxygen over the burn (the held
outcome cells), `tests_outcomes.jsonl`, the sensor workbooks' series after ignition, any page marked
protected in `runs/dev/data_docs/`, `holdout/`.

## 5. Measurement, fixed before any frame is opened

The tracks are measured by a sub-agent that is not the author and knows nothing of the models'
forecasts, with a script, from NASA's **geometrically mapped** frame sequences (`..._JPEG_Mapped...`;
the `Raw2TIFF_Mapped` sets only where a JPEG set is missing). Listings:
`runs/dev/data_all/manifests/PSI-98.listing.jsonl`, `PSI-99`, `PSI-100`. Camera 1 is primary; camera 2,
where it exists, is a cross-check whose agreement is reported as a quality figure and is not averaged
in.

- **Scale.** The two long edges of the sample in the mapped frame give its width in pixels; positions
  are in sample widths, `position_unit = sample_width`. The scale and its spread over frames are
  reported.
- **Origin and axis.** The upstream (igniter-side) edge of the sample, positive downstream along the
  flow, as on BASS.
- **Tip and base.** Luminous flame pixels by a fixed threshold above each pixel's dark level (the
  method of `tools/wsp/bass_optical/extract.py`), restricted to the largest connected flame region.
  Project them on the flow axis as the fraction of the sample width that is luminous at each position.
  Tip: the most downstream position where that fraction is at least 0.05; base: the most upstream such
  position. The thresholds are fixed by the measurer before the first sheet is tracked, from the
  igniter frames and the frames before ignition, and are not changed afterwards.
- **Stills.** One per second of record time (every frame where frames are sparser), each with
  `state` (`igniter`, `flame`, `none`) and `quality` (`edge` when the flame touches the image border or
  the view is obstructed; such stills are not scored, as on BASS).
- **End of the track.** `extinction` when the flame disappears with its tip short of the downstream
  end of the sample; `left_view` at the first still whose tip reaches the downstream end of the sample
  (the track as the model sees it ends there; the forward's `p_reached_end`); `record_end` when the
  frames stop with a flame; `lost` when tracking fails. The time is the midpoint between the last still
  with the flame and the next.
- **Files.** `wsp-bass-front-track/1`, one per burn, `runs/dev/saffire/tracks_held/wide/` and
  `.../strip/` (git-ignored), `run_id` `PSI-98:S1` and so on, `partition` `validation` with
  `evaluation_group` (`PSI-98:flight`…) and `group_status` `held` (the split files define held use as
  validation use; the scorer reads train or validation only), `sample_label` naming SIBAL, and
  `conditions.flow` `concurrent` or `opposed`. A receipt in that folder lists each file's sha256.
- **Check by a cheaper model.** A second agent (Sonnet) is shown at least ten stills per tracked
  burn with the measured tip and base drawn on them, chosen by a fixed stride, and reports how many it
  accepts. It is not shown the models' forecasts.
- **What comes back to the author:** counts of stills and flame stills per stratum, the method,
  calibration (pixels per sample width and their spread), the time origin used, camera agreement, the
  check counts and quality flags. No position, speed, burn time, end time or end kind. The author does
  not open the track files.

## 6. Target and metric

`wsp-dev front-feedback` unchanged ([crates/ignis-wsp-dev/src/front.rs](../../crates/ignis-wsp-dev/src/front.rs)):
cutoffs 5, 10, 20 and 40 s after igniter-off; a case needs a scoreable track (at least six flame
stills), two flame stills at or before the cutoff and three in (cutoff, cutoff + 60 s], which are the
targets. Per target: CRPS from the quantiles 0.05, 0.25, 0.5, 0.75 and 0.95, absolute error of the
median, coverage of the central 50 % and 90 % bands, all in sample widths. Case = mean over targets;
run = mean over its cases; report = mean and median over runs. Paired against the first release
(persistence) over common cases: the mean run difference, and **the number of runs on which each is
lower**. Track-ended Brier at 10, 20, 30, 45, 60 s and the ignition score are reported as secondary.
The bootstrap interval the scorer prints is reported and is not evidence: two runs from two flights.

The only code change for this test is the reader rule of section 4 (`geometry.sample_width_m`), which
no train or validation track carries, so every earlier score is unchanged; `front_tools.rs` is not part
of any release's forward identity. Its test is in `crates/ignis-wsp/tests/front_forward.rs`.

## 7. What the result will be taken to mean

Primary metric: tip CRPS in sample widths, D1 against persistence, wide stratum.

> **The narrow-strip model transfers to the wide sheet** if D1's run tip CRPS is lower than
> persistence's on every wide burn scored, with at least two scored; it **does not transfer** if it is
> higher on every one; otherwise the result is **undecided**. With fewer than two scored wide burns the
> test is **not decided**.

The summary states that sentence's outcome, the number of burns and cases behind it, and the coverage
beside it: lower CRPS with 90 % coverage below one half is reported as a sharp model that wins by
being less wrong than persistence, not as a calibrated one. The strip stratum and the two Astra
releases are reported by the same sentence and decide nothing. No model is revised from these
outcomes in this generation.

## 8. Execution

1. This file, the reader rule and its test, and the wrapper `tools/wsp/saffire/score_transfer.py` are
   committed first.
2. The scorer is built from that commit into a private target and copied to a content-addressed
   snapshot; its sha256 and the release ids are written to
   `docs/wsp/feedback/saffire-transfer/freeze.json` and committed before measurement.
3. Measurement (section 5), then the owner is asked before scoring: a held test can be spent once.
4. On the owner's yes, one run of `python tools/wsp/saffire/score_transfer.py --execute`. It checks
   that this file, the reader, the wrapper and the freeze are committed and unchanged; checks the
   binary's sha256; **reproduces the committed TRAIN aggregate of
   `docs/wsp/feedback/astra-bass-operator-20261115/results/aggregate.json` for its three releases to
   1e-12** (if it does not, nothing is scored on Saffire); checks the run ids, partitions and receipt
   hashes of the held tracks without printing their contents; then scores the wide and strip folders
   once each with `--quiet-cases` and no `--detail`, into
   `docs/wsp/feedback/saffire-transfer/results/`, which must not exist beforehand. It prints aggregates
   only and applies the sentence of section 7 mechanically.
5. The author writes the readable summary from the aggregates and commits it with the results and the
   dossier rows.

## 9. Known threats, stated now

- **The width ratio is 18.5**, against 0.55 and 1 in everything the model has seen. Every width term
  of every release was typed from two classes a factor of two apart; at the sheet they are
  extrapolations, and the forward answers anyway (`front.rs` accepts any positive ratio).
- In sample widths the sheet is short (2.3 widths): a flame that crosses it at a few cm/s gives few
  targets per case, and the cut at the sample end carries much of the forecast.
- A 40 cm flame is two-dimensional across the sheet; the tip and base defined in section 5 are the
  extremes of a profile, not the edge of a band.
- The mapped frames are NASA's rectification; the measurer cannot check it beyond the sample edges.
- Oxygen 0.21 is nominal for two burns. Flow is the commanded value, not the measured one.
- Saffire I and III are one burn each in the decision: the result says what the model did on two
  flights, not how it generalises.
