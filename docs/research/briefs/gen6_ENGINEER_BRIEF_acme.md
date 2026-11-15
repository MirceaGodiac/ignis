# Brief and amendments given to the engineer of flame4.rs and committee2.rs (14 Nov, recovered from the session transcript)


===== Agent 2026-11-14T23:42 UTC =====
You are an engineer on the IGNIS project. Repository: D:\dev\SpaceApps-Ardealu, branch feat/handwritten-science. Read AGENTS.md at the repo root first; its rules bind you. Then read `docs/wsp/PROTOCOL.md` sections 8 to 10 (conventions for forwards and releases).

## Task

Implement a new strict forward, `crates/ignis-wsp/src/committee2.rs`, for two new model names, `acme_normal.wsp7` and `acme_inverse.wsp7`. It generalises the existing committee forward `crates/ignis-wsp/src/committee.rs` (models `acme_normal.wsp5`, `acme_inverse.wsp5`), which you must read in full first, together with how it was added (commit 88951ea: spec, root, binding, tests, registration, reassembly). Mirror that addition exactly in structure.

I am the author of the model and I type its numbers. You type NO scientific number. Every value you need is given below or is copied from an existing stored release.

## Frozen files: never edit

`committee.rs`, `flame.rs`, `flame3.rs`, `blend.rs`, `condition_committee.rs`, `common.rs`, `nn.rs`, `quad.rs`, `spec.rs`, and every other existing forward file in `crates/ignis-wsp/src/`. A forward is frozen with its releases; existing release ids must not change. The new forward is a new file. You may add to `lib.rs`, `models.rs`, `audit.rs` (the new forward's source list), `cli.rs`, and the `ignis-wsp-dev` crate where a model table needs the new names, with additions only.

## Specification of `committee2.rs`

Same six slots as `committee.rs`: `a0.`, `a1.` are full `flame.rs` releases, `b0.` to `b3.` full `flame3.rs` releases, tensors named with the slot prefix, each member run unchanged by its own forward through the same `member_release` mechanism. Spec ids `acme_normal-wsp-v7` and `acme_inverse-wsp-v7`, forward version "7", entry point `ignis_wsp::committee2::forecast`, profile StrictNoFitting. Model keys `acme_normal7`, `acme_inverse7`.

Tensors besides the slots (replace `mix.w`, `mix.w_script`, `script.onset_s`):

| Tensor | Shape | Bounds | Meaning |
|---|---|---|---|
| `mix.w_early` | [6] | 0 to 1e3 | slot weights for a test of the early era, when the prefix does not carry the planned stop |
| `mix.w_late` | [6] | 0 to 1e3 | the same for the late era |
| `mix.ws_early` | [6] | 0 to 1e3 | slot weights for the early era when the prefix carries the planned stop |
| `mix.ws_late` | [6] | 0 to 1e3 | the same for the late era |
| `era.last_early_day` | scalar | 1900001 to 2100366 | last calendar day known to be in the early era, as year x 1000 + day of year |
| `era.first_late_day` | scalar | 1900001 to 2100366 | first calendar day known to be in the late era, same encoding |
| `era.p_late_between` | scalar | 0 to 1 | probability of the late era for a test dated strictly between the two days |
| `era.p_late_unknown` | scalar | 0 to 1 | probability of the late era when the prefix gives no date |
| `script.offset_s` | [3] | -20 to 20 | offsets added to the stored planned stop: the scheduled stop handed to the members is `programmed_stop + script.offset_s[k]` |
| `script.offset_w` | [3] | 0 to 1e3 | weights of the three offsets, normalised by their sum |

Roles and units as the analogous tensors of `committee.rs` (weights: `TensorRole::Weight`, unit "1"; offsets: `PhysicalParameter`, "s"; the days: `PhysicalParameter`, unit "day"; the probabilities: the role `committee.rs`-style specs use for a probability, unit "1"). Write the descriptions and the spec `notes` in the same plain style as `committee.rs`, stating every rule below.

**Date of the test.** Read from the prefix, never from an outcome:
1. `prefix.session_id`, if present: the first substring of the form four digits, hyphen, two digits, hyphen, two digits that is a valid calendar date (year, month, day).
2. Otherwise `prefix.run_id`: take the part after the last `:`; if its first five characters are ASCII digits `YYDDD` with DDD between 1 and 366 (and no more than 365 in a common year), the date is day DDD of year 2000 + YY.
3. Otherwise there is no date.
The ordinal is year x 1000 + day of year. Implement the calendar arithmetic yourself (no new dependency unless one is already in the workspace).

**Era probability.** `p_late` = 0 if the ordinal is at or before `era.last_early_day`; 1 if at or after `era.first_late_day`; `era.p_late_between` if strictly between; `era.p_late_unknown` if there is no date. If `era.last_early_day >= era.first_late_day` the release cannot be run (`Refusal`, like the forward-mismatch case).

**Which weights.** The script path is taken when the prefix carries a resolved, finite, positive condition `programmed_stop` AND at least one entry of `mix.ws_early` or `mix.ws_late` is positive AND at least one entry of `script.offset_w` is positive. Then the pair is (`mix.ws_early`, `mix.ws_late`); otherwise (`mix.w_early`, `mix.w_late`). A vector of the pair that is used with a non-zero era probability must be all >= 0 with a positive sum, else a scientific refusal (Semantic gate, as `committee.rs` does for `mix.w`). Normalise each used vector to sum one, then the slot weight is `w[i] = (1 - p_late) * early[i] + p_late * late[i]`. A slot with weight zero is not run. A vector whose era has probability zero is not checked and not used.

**Script path.** For each offset node k with `script.offset_w[k] > 0` (weights normalised to sum one, `o[k]`): build the prefix without the `programmed_stop` condition and with one more control stage `scheduled_stop` at `programmed_stop + script.offset_s[k]` (same `ControlStage` fields as `committee.rs`), and run every slot with positive weight on it. The forecast table is `sum_k o[k] * sum_i w[i] * P[i,k]`, row by row, nodes in the outer loop and slots in the inner. If a member refuses on any node, the forecast is refused with that member's gate and reasons, prefixed with the slot and the node index. Without the script path there is one pass on the prefix as given (if the prefix carries `programmed_stop` but the script path is not taken, remove nothing: behave as `committee.rs` does in that case, and check what it does).

**Bit-for-bit rule.** When exactly one (node, slot) pair is run, return that member's table unchanged, as `committee.rs` does for one member.

**Diagnostics.** As `committee.rs`, plus: the date read and its source (`session_id`, `run_id` or none), the ordinal, `p_late`, the offsets used with their weights, and per member the node index.

`ALLOWED` condition keys: those of `committee.rs`.

## Root releases

Create the roots `configs/models/acme/wsp7/normal/baseline-set` and `configs/models/acme/wsp7/inverse/baseline-set` (with `spec.json`, `binding.json`, `header.wsp`, `edits.wsp`, `meta.json`, `patch.json`, `release.json`) the same way the wsp5 roots were made. Slots: exactly the stored slot values of the corresponding wsp5 `baseline-set` root, copied statement by statement by script (see `tools/wsp/committee_edits.py`; extend it to serve the wsp7 models without changing what it does for wsp5, or add a sibling script). New tensors in the roots, which hold no train number:

- `mix.w_early` = `mix.w_late` = the wsp5 root's `mix.w`
- `mix.ws_early` = `mix.ws_late` = the wsp5 root's `mix.w_script`
- `era.last_early_day` = 1900001, `era.first_late_day` = 2100366, `era.p_late_between` = 0.5, `era.p_late_unknown` = 0.5
- `script.offset_s` = [0.0, 0.0, 0.0], `script.offset_w` = [1.0, 0.0, 0.0]

Add the roots to `tools/wsp/reassemble.py` and assemble them into the local registry (`artifacts/models/`, git-ignored) with the existing CLI, as the wsp5 roots are. I will add candidate folders next to the roots afterwards, so tell me the exact commands to generate a candidate's slot statements from stored member releases and to assemble a candidate (revision of the root) for `acme_normal7`.

## Tests: `crates/ignis-wsp/tests/committee2_forward.rs`

Use the synthetic fixtures and helpers `tests/committee_forward.rs` uses. Required, each asserting exact equality unless stated:

1. **Neutral against wsp5.** A wsp7 release whose slots are those of the wsp5 normal root (and one built from `configs/models/acme/wsp5/normal/p1-fable` slots if the test helpers make that practical), with `mix.w_early = mix.w_late = mix.w` of that wsp5 release, `mix.ws_early = mix.ws_late = mix.w_script`, `script.offset_s = [-onset, 0, 0]` where onset is that release's `script.onset_s`, `script.offset_w = [1, 0, 0]`: the event table equals the wsp5 release's table exactly, on a prefix without `programmed_stop` and on one with it.
2. **Era.** With different early and late weights: a date at or before `last_early_day` gives the early-only forecast; at or after `first_late_day` the late-only one; strictly between, `(1-p) early + p late` of those two (within 1e-12); no date, the same with `p_late_unknown`. Slots of an era with probability zero are not run (check through the diagnostics).
3. **Date reading.** `session_id` containing `2021-03-29` gives 2021088; a run id `PSI-159:21270H3` with no session id gives 2021270; `session_id` wins when both are present; a run id that does not start with five digits gives no date; day 366 of a common year gives no date.
4. **Offsets.** Three nodes with different offsets and weights equal the weighted sum of three single-node runs (within 1e-12).
5. **Refusals.** `last_early_day >= first_late_day`; a negative weight; all weights of a used vector zero.
6. **Identity of old releases.** Whatever test pins forward hashes or release ids for existing models still passes; if `python tools/wsp/reassemble.py` is the check, run it and report whether every existing id is unchanged.

## Working rules

- Another three sessions commit to this same checkout and branch. Do NOT commit, stage, stash, switch branch or push. Touch only the files this task needs. Do not run a formatter over files you did not write: run `rustfmt` on your own files, and `cargo fmt --all -- --check` only to check.
- Build in a private target directory so you do not fight other sessions: `--target-dir runs/tooling/fable-gen6-target` (git-ignored), with `-j 4`.
- Never read `holdout/`, `runs/dev/holdout_ext/`, validation or test outcomes; never pass `--final-test`; do not run `dev_eval.py`, `ignis eval` or any scoring. Synthetic fixtures only.
- Gates to run before you report, with their real output: `cargo fmt --all -- --check`; `cargo check --workspace --all-targets --target-dir runs/tooling/fable-gen6-target -j 4`; `cargo test -p ignis-wsp -p ignis-wsp-dev -p ignis-core --target-dir runs/tooling/fable-gen6-target -j 4`. If the format check fails only on files you did not touch, say so and leave them.

## Report (your final message)

Files added and changed; the gates' actual results (counts, failures verbatim); the two root release ids; whether every existing release id is unchanged and how you know; the exact commands for generating and assembling an `acme_normal7` candidate; where the built `ignis` and `wsp-dev` binaries are; anything you could not do or had to decide yourself, stated plainly.

===== SendMessage 2026-11-14T23:48 UTC =====
Amendment to the committee2 task, from the author: add a third member forward and two more slots. Fold this in before you finalise the roots and tests; nothing has been released from committee2 yet, so its spec may still change. Everything else in the original brief stands.

## 1. New member forward `crates/ignis-wsp/src/flame4.rs`

Model names `acme_normal.wsp8`, `acme_inverse.wsp8`; keys `acme_normal8`, `acme_inverse8`; spec ids `acme_normal-wsp-v8`, `acme_inverse-wsp-v8`; forward version "8"; entry point `ignis_wsp::flame4::forecast`; StrictNoFitting. It is the hinge-law member of `flame.rs` with its burner-limit clock replaced by the burner-heating law of `flame3.rs`. Why: the `flame.rs` clock is a log-logistic in fuel flow alone and gives a limit trip at 15 to 17 s to a cool low-flow flame; the heating law knows the total flow and the flame radius.

**Tensors.** Every tensor of the `flame.rs` spec except those of its burner-limit clock (`ft.tau.*` and `ft.shape`), with the same names, shapes, roles and bounds; plus, with the same names, shapes, roles and bounds as in the `flame3.rs` spec, every tensor the `flame3.rs` burner-limit clock reads: `bt.*` (rise net, `bt.tau_s`, `bt.t0_c`, `bt.sigma`, `bt.shape`, `bt.limit_c`, `bt.limit_w`), and whatever else its burner features need (the flame radius r_ss uses `se.rho_d_g_m_s`; keep that name) and its numerics setting for the heating nodes (`num.bt_nodes` or whatever `flame3.rs` calls it). Read `flame3.rs` to find the exact set; do not invent names. Features and domain: those of the `flame3.rs` spec (the hinge law reads the first eight, as the `committee.rs` notes say).

**Forecast.** Exactly the `flame.rs` forecast (weak and main classes, floor, flash, the extinction clock `se.tau`, programmed-stop atoms, base rate, the scheduled-stop control, refusals, numerics), with one change: where `flame.rs` adds the hazard of its burner-limit log-logistic to the flow-termination hazard, `flame4.rs` uses the burner-limit survival of `flame3.rs`, S_bt(t), computed exactly as `flame3.rs` computes it from the same tensors and the same prefix (heating nodes, limits and their weights, the log-logistic around each node-and-limit time, "never" when the limit is not reached; all weights zero means S_bt = 1). Over an integration step [t, t+h] the integrated trip hazard is ln(S_bt(t) / S_bt(t+h)), and it joins the other flow-termination hazards of the step in the competing-risk split the way the old clock's hazard did. A weighted limit at or below the burner temperature at ignition is a refusal, as in `flame3.rs`. If `flame.rs` and `flame3.rs` keep the needed pieces private, duplicate the logic in the new file; never edit the frozen files.

**Diagnostics.** Those of `flame.rs`, with the burner block of `flame3.rs` (limits, times to each limit, p_limit_never) in place of the old clock's median.

**Roots.** `configs/models/acme/wsp8/{normal,inverse}/baseline-…`: hinge-law tensors copied statement by statement from the `flame.rs` root of the family (`configs/models/acme/wsp2/<family>/baseline-clock`), burner tensors from the `flame3.rs` root (`configs/models/acme/wsp3/<family>/baseline-theory`). No new number. Add to `reassemble.py`, register the keys, assemble.

**Tests** (`tests/flame4_forward.rs`, synthetic fixtures):
1. With `bt.limit_w` all zero, the table equals a `flame.rs` release with the same hinge tensors whose burner-limit clock is pushed out of reach (`ft.tau.b0` = 60 with zero skip and units), within 1e-12.
2. The burner diagnostics (limits, times, p_limit_never) equal those `flame3.rs` reports for a release with the same `bt.*` tensors on the same prefix, exactly.
3. With the extinction clock made negligible in a main-class case and atoms and base off, the flow-termination incidence at each forecast time equals 1 - S_bt(t)/S_bt(cutoff) within 1e-9.
4. Scheduled stop and refusals behave as in `flame.rs` (reuse its test cases).

## 2. Two more slots in `committee2.rs`

Slots are now eight, in this order: `a0.`, `a1.` (`flame.rs`), `b0.` to `b3.` (`flame3.rs`), `c0.`, `c1.` (`flame4.rs`, run by `flame4::forecast` through the same member-release mechanism). The four weight tensors `mix.w_early`, `mix.w_late`, `mix.ws_early`, `mix.ws_late` have shape [8] in that order. Everything else in the committee2 specification is unchanged. In the wsp7 roots the `c` slots hold the stored values of the wsp8 root of the family and have weight zero. The neutrality test against wsp5 keeps its meaning: the six old slots carry the wsp5 weights, the two new ones zero. Add one test that a release with all weight on `c0` returns the `flame4` release's own table exactly.

`tools/wsp/committee_edits.py` (or its sibling) must be able to fill `c` slots from stored wsp8 releases.

## 3. Report

As before, plus the two wsp8 root ids, and the exact commands to compile and assemble a wsp8 candidate (a revision of the wsp8 normal root).

===== SendMessage 2026-11-14T23:48 UTC =====
Correction to the amendment, from the author: committee2 needs FOUR flame4 slots, not two, so ten slots in total.

Slot order: `a0.`, `a1.` (`flame.rs`), `b0.` to `b3.` (`flame3.rs`), `c0.` to `c3.` (`flame4.rs`). The four weight tensors `mix.w_early`, `mix.w_late`, `mix.ws_early`, `mix.ws_late` have shape [10] in that order. Reason: a flame4 member carries the burner limit of one era and comes with and without guessed stops, so a candidate needs four of them, exactly like the four flame3 slots.

Everything else stands: in the wsp7 roots all four `c` slots hold the stored values of the wsp8 root of the family with weight zero; the neutrality test against wsp5 puts the wsp5 weights on the six old slots and zero on the four new ones; one test puts all weight on `c0` and expects the flame4 release's own table exactly.