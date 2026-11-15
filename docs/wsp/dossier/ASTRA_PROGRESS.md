# Astra work record

14 November 2026, branch `feat/handwritten-science`; starting commit
`2e05f8ebe6fb5c1c545ffee5ce681266a9f1448e`. Task scope: I2/I4/I5 research and
experimental handwritten extensions; no push, PR or merge.

## Review, research and design checkpoint

- Six findings filed in `review/`, indexed in its README. Train timing arithmetic
  independently recounted; broad physical/source claims retain their narrower evidence
  status. Original candidates and records are not corrected in place.
- Primary-source research collected in three `research/astra_*.md` ledgers and
  independently analyzed in `astra_research_analysis.md` and
  `astra_physics_analysis.md`. The latter two explicitly ran `gpt-6-astra`, high effort,
  after the owner's model-selection instruction. Earlier collectors inherited their
  model; no exact runtime build identity is asserted for them.
- `ASTRA_DESIGN.md` records the first-principles decisions before new candidate weights.
  All new candidates will be data-informed. No new validation look is planned.
- X-23 / F-004 records uncertain raw-file identity in one delegated check. Raw access
  stopped; all such raw material is excluded. No uncertain numeric result was sent to
  the author. The independent property arithmetic uses reference equations only.

## Verification actually run at this checkpoint

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | passed, exit 0 |
| `cargo check --workspace --all-targets` | passed, exit 0 |
| `cargo test -p ignis-wsp -p ignis-wsp-dev -p ignis-core --quiet` | passed, exit 0 |
| `python tools/wsp/astra/check_methanol.py` | run by the checker; reference-law arithmetic only, results in F-003 |

The three required Rust gates are checks of the existing code at the research checkpoint,
not validation of a new scientific mechanism. No live API test or protected evaluation ran.

## Next implementation checkpoint

New files are being implemented for coupled post-flame cooling (`droplet7.rs` with
`thermal.rs`), steady oxygen-supply spread (`front3.rs`), and persistent flow/pressure
scenario mixtures (`condition_committee.rs`). Composition perturbation is excluded from
the burner wrapper because the existing inputs also carry externally computed `Tad`
and `Zst`; changing one without recomputing the others would create inconsistent inputs.
The model author alone writes candidate values after reviewing the implementations.

## Forward and literal-authoring checkpoint

- Five additive identities implemented: `flex_methanol.wsp6`, `flex_heptane.wsp6`,
  `acme_normal.wsp6`, `acme_inverse.wsp6`, `bass_front.wsp3`. Full-distribution joint
  switches are available; existing source/hash closures remain unchanged.
- Main author runtime metadata verified as `gpt-6-astra`, effort `max`. All new
  `addition.wsp` and joint literals were written by that main author, not delegates.
- Added explicit `rewrite`/`rewrite-check` for new-spec complete literal authoring with
  real parent ancestry and full exposure, preserving ordinary same-spec assembly.
- F-007/F-008 filed; new active BASS event semantics corrected and train domain loading
  restricted by train IDs before outcome deserialization. No shared contract or record edit.
- The new droplet suite passed 15 tests, the fabric suite 11, condition committee 5 and
  rewrite suite 8 in the required combined run. These use synthetic fixtures, not NASA
  outcomes. The train loader's three tests include missing-row refusal and protected-row
  domain rejection. The one-ULP cross-grid comparison uses a documented tight tolerance;
  exact neutral identity checks stay exact.
- Required gates passed at this checkpoint: `cargo fmt --all -- --check`;
  `cargo check --workspace --all-targets`; and
  `cargo test -p ignis-wsp -p ignis-wsp-dev -p ignis-core --quiet` (all exit 0).
  Check reports an unused import used only by the train loader tests; it is not a test failure.
- `git diff` of every old forward and all four shared numerical sources is empty after
  correcting the formatter incident. The release executable is being rebuilt. No new
  candidate score or validation look exists at this checkpoint.

## Assembly and train-results checkpoint

- Ten data-informed releases assembled with immutable ancestry, complete patches and
  cards. Exact inherited transcriptions and predecessor/new forward hashes checked.
- First assembly attempt failed on whitespace in new SourceIds, before patch creation;
  syntax corrected without changing coefficients or removing exposure.
- Five fixed train comparisons executed on shared records. F-009's paired-key collision
  is bypassed by release-ID pairing. F-010's event-case export correction was replayed
  into separate files: all initial/replay aggregate objects are exactly equal.
- No weight changed after feedback. All principal new scores are slightly worse except
  the BASS joint on one train day; carried models remain. Details and all session
  differences are in `ASTRA_RESULTS.md` and `feedback/astra_train_summary.json`.
- Release-backed fixed-grid thermal step check: 36 successful cases, max diameter
  difference 0.000842 µm; source extrapolation remains explicit, not validated away.
- Historical origin audit adds conservative inherited measurement-backed lower bounds;
  exact full-model direct-train counts remain unknown. Cards include full explicit
  assignment counts, authored capacity, fixed positions and new N/A/L/S/U positions.
- Required gates passed again, all exit0: `cargo fmt --all -- --check`,
  `cargo check --workspace --all-targets`, and
  `cargo test -p ignis-wsp -p ignis-wsp-dev -p ignis-core --quiet`.
  The test-only import warning was corrected. Old numerical sources remain unchanged.

## Continuing owner scope

The owner supplied `ASTRA_PROMPT_ALL_NASA.md` and subsequently section2b. Both were read
in full. The objective now includes19 investigations, sequential media measurement,
shared-property family models, blind page/test labelling, reproducible clusters/links
and source-bound interpretations. The present checkpoint does not complete that objective.
Fable retains atlas schema/catalogue/group-freeze ownership. At the initial inspection
the atlas/split directories did not exist; new-investigation outcomes stay closed to
authors. Metadata-only downloads and independent theory/method design have begun, in
separate owned paths. The owner authorized sequential temporary-media deletion after
measurement retention; only verified task-owned downloaded media may be deleted.

The expanded work now has its own live ledger, `ASTRA_ALL_NASA_PROGRESS.md`.
Fable's schema, filename catalogue and fifteen split files are committed at
`5c75f6b`; the first-principles all-NASA memo precedes the new physical-core
implementation. Acquisition and protected page extraction have actual receipts;
measurement qualification, semantic labels, new family releases and evaluations
are still pending. The ten earlier candidates remain data-informed and unchanged.
