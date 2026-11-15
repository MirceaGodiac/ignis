# Review dossier of the hand-written models

**Astra review, 14 November 2026:** the ten findings in [review/](review/README.md),
[first-principles design](ASTRA_DESIGN.md), and the `astra_*.md` research files are
additions to Fable's historical handoff below. The user requested GPT-6 Astra for research
analysis; the explicitly selected Astra analyses are `research/astra_research_analysis.md`
and `research/astra_physics_analysis.md` (one directory above this dossier). Collector
reports are separately attributable. New Astra candidates are **data-informed**, and no
new validation look is planned. Current carried releases below remain unchanged.

For a second author or reviewer (Astra) who wants to redo the research behind the literal models of
this lane, check how each document and each data file was classified, check the results read from
the datasets, argue with the decisions, and then work on the models together with their author.

It is a working file, not a diary. It holds what a reviewer needs in order to disagree usefully:
what was read and how far, what was measured and where the table is, what was decided and against
which alternatives, which claims carry the models, where the author knows the work is weak, and how
to rerun each check. The author is `claude-fable-5-2`; statements in the first person are his.

State: 14 November 2026, branch `feat/handwritten-science`. The fifth generation is finished: its
candidates were registered with their rules, validated once, and decided by those rules. Two were
adopted and four were not ([08_in_progress.md](08_in_progress.md), section 8.7;
[RESULTS_GEN5.md](../RESULTS_GEN5.md)).

## Read in this order

| File | What it answers |
|---|---|
| [01_sources.md](01_sources.md) | Which documents were read, by whom, how far, under which rule, and what each was used for. The classification scheme and where it was applied unevenly |
| [02_data_classification.md](02_data_classification.md) | The NASA archives file by file: what counts as an input known before a run, an observation admitted up to the cutoff, an outcome, a fact recorded after the run. Partitions. The judgment calls |
| [03_datasets_and_measurements.md](03_datasets_and_measurements.md) | Every measurement made on the data: result, who made it, where the table is, what the author rechecked and what he took on trust |
| [04_decisions.md](04_decisions.md) | The decision log: what was chosen, against what, why, what became of it, what would reverse it |
| [05_claims.md](05_claims.md) | The claims the models rest on, each with its evidence, the author's confidence and the cheapest way to break it |
| [06_models.md](06_models.md) | The carried models: forward, edits file, where each group of typed numbers comes from, which numbers the author trusts least |
| [07_limits_and_disclosures.md](07_limits_and_disclosures.md) | Every known exposure to protected data, the mistakes made, how worn the validation partitions are |
| [08_in_progress.md](08_in_progress.md) | The fifth generation: what was built, what train said change by change, what was registered, and what validation decided |
| [09_reproduce.md](09_reproduce.md) | Commands that rebuild releases, scores, tables, the private data root and the registers |
| [10_extension_map.md](10_extension_map.md) | How a hand-written model can honestly become large, and the mechanisms the physics dossiers found per family, with how far each is checked |
| [review/](review/README.md) | How to file a finding, how a disagreement is settled, how to write a candidate of your own |
| [registers/](registers/) | Machine-made lists: every candidate with its declared exposure, every delegated agent with the model that answered, the local folders cited |

Older documents the dossier builds on and does not repeat: [EXPERIMENTAL_SCIENCE.md](../EXPERIMENTAL_SCIENCE.md)
(the experiments and what each generation found), [PROTOCOL.md](../PROTOCOL.md) (rules, every
registered validation look and its outcome), [RESULTS.md](../RESULTS.md), [RESULTS_GEN5.md](../RESULTS_GEN5.md), [cards/](../cards/),
[research/](../research/), [feedback/](../feedback/).

## Rules that bind a reviewer as they bind the author

A review that breaks one of these turns a clean partition into development data for both of us.

1. **The test partition is not read.** No file of a test run, no `holdout/` folder, no `--final-test`,
   `--partition test` or `--partition all`, no summary table or derived artifact of it, no paper's
   per-test result for a test run. The final evaluation belongs to Tudor's lane.
2. **Validation outcomes are not read per run.** Validation is read as inputs (conditions, controls)
   or as an aggregate table produced by a look that was registered in [PROTOCOL.md](../PROTOCOL.md)
   with its rule before it was run. The sealed BASS validation tracks
   (`runs/dev/bass_front/val_sealed/`) and the private data root (`runs/dev/data_ext/`, which holds
   validation flame series) are not opened by anyone who writes or revises a model; neither is the
   private vault of validation continuations (`runs/dev/holdout_ext/`).
3. **Train is open**, outcomes included. The earlier BASS campaign (PSI-26) is outside every partition
   and is open.
4. **No number is produced by a program.** No fit, regression, optimiser, search or scan chooses a
   parameter. A script may measure (finite differences, window means, counts, medians) and may
   evaluate a typed law at typed constants. The author types every weight.
5. **Published per-test results of the modelled series are not read** (the FLEX ISS droplet tests,
   the CFI-G tests of 2021, the BASS-II concurrent SIBAL tests): they are the validation and test
   runs. This rule dates from the fourth generation; [01_sources.md](01_sources.md) lists where the
   earlier dossiers went further than it allows.
6. **Shared contracts and shared records are not changed** by this lane. What a model needs beyond
   them lives in a private copy of the data root.
7. **Nothing is deleted or re-identified.** Every candidate, release, card and feedback file stays;
   a forward is frozen with its releases; a refused candidate stays as a negative result.

## What is carried today

| Family | Model | Candidate | Validation (development number) |
|---|---|---|---|
| FLEX methanol | `flex_methanol.wsp2` | D3 `sha256:c8a90dab…b654` | CRPS 40.0 µm |
| FLEX n-heptane | `flex_heptane.wsp2` | D2 `sha256:edbad4d6…edef` | 46.1 µm |
| ACME normal | `acme_normal.wsp5` | P1 `sha256:1336c630…77f4` | extinction Brier 0.145, flow termination 0.075 when the planned stop of the test is known; 0.155, 0.097 on the shared records (the model before it, `acme_normal.wsp4` D1: 0.155, 0.098) |
| ACME inverse | `acme_inverse.wsp5` | P1 `sha256:aa05ddcb…ea0e` | 0.135, 0.048 when the planned stop is known; 0.129, 0.088 on the shared records (the model before it, `acme_inverse.wsp3` D1: 0.161, 0.094 on the same 54 cases) |
| BASS | `bass_front.wsp` | D1 `sha256:ac15e49f…855c` | tip CRPS 0.117 sample widths (persistence 0.234) |

The two burner-flame models were adopted in the fifth generation; the droplet and fabric candidates
of that generation were refused by the rules written beforehand and the models of the third and
fourth generation stay. The planned stop is an input of a private copy of the data root, not of the
shared records.

All five are data-informed: typed after their author read train outcomes. None is zero-shot. The
validation partitions have been read as aggregate tables many times, so these are development
numbers ([07_limits_and_disclosures.md](07_limits_and_disclosures.md), section 2).

## Where I would attack first

In the order in which I think a finding is most likely and matters most. Each item points at a
numbered claim or decision.

1. **Published results of the modelled series reached the author through the dossiers of 13
   November.** `research/exp_bass.md` section F2 quotes a quench boundary measured on the BASS-II
   SIBAL samples; the quench law of the BASS model was typed a day later and passes near it. The
   FLEX and ACME dossiers carry whole-campaign aggregates. See X-01 to X-04 in
   [07](07_limits_and_disclosures.md).
2. **The script parameter file as a prospective input** (claim C-21): that NASA's "as run" file of an
   ACME test holds the planned stage durations and nothing of how the test went. Both carried
   burner-flame models now read it when the prefix carries it, and most of their gain comes from it.
3. **Flame-camera frames as evidence of flame loss** (C-12, C-13): absence of frames near the cutoff
   is read as a lost flame; six train narratives say the flame merely left the view.
4. **What decides a lean inverse flame.** My first answer, the total burner flow (C-24), rested on
   one train flame and did not survive my own recount (M-17). Its replacement (C-37: an oscillation
   that starts at a critical oxygen flux, with the mixture fraction as the class variable, C-38)
   comes from a transport model whose temperatures its own author calls wrong. The carried inverse
   model holds such a flame with probability one third (C-44), on the evidence of two train flames.
5. **The tabulated atmosphere of a FLEX run is computed after the fact** from the fill and the
   combustion history of earlier burns (C-04), and is used as a prospective condition.
6. **The fan history of a BASS burn is given to the model up to each forecast time** (D-30), so the
   forecast is conditional on what the operator did next.
7. **The ending kinds of the FLEX train runs** (thresholds 0.65 and 0.20 mm²/s, M-07) and every
   count built on them.
8. **What "rechecked by the author" covered** in each delegated analysis (the status column of
   [03](03_datasets_and_measurements.md)); several reports were acted on after spot checks only.
9. **The band widths of the droplet models** (D-06, D-15): typed from train residuals with a margin
   that is a judgment.
10. **The validation vetoes** (D-05): two sessions per ACME configuration decided which model is
    carried.
11. **How the fifth-generation look was run** (X-20, X-21, D-45, D-46): droplets against a vault of
    their own made from the private copy of the data root; burner flames sealed on both data roots
    and the evaluator's one comparison split by a script of this lane. Both are departures from the
    shared procedure that the evaluation lane has not seen.
