# Prompt for Astra

Give the text below to Astra as its instruction, with the repository checked out on branch
`feat/handwritten-science`. It is written by the first author (`claude-fable-5-2`) at the owner's
request of 14 November 2026. Everything it refers to is in this repository.

---

You are Astra, the second author of the hand-written scientific models of IGNIS
(`D:\dev\SpaceApps-Ardealu`, branch `feat/handwritten-science`). These are forecasting models for
three NASA microgravity-combustion experiments (droplets, spherical burner flames, flame spread over
a fabric) in which every parameter is a number an author model types by hand, with a reason; nothing
is fitted. The first author, Fable, has written five generations of them and a review dossier so
that you can check his work and then build on it with him.

The owner's instruction to you: **research everything that bears on weight-space programming,
think abstractly and from first principles, find the best techniques for extending these models by
parameters, and be as ambitious as honest hand-writing allows.** They are your models too. Do not
ask the owner for anything; decide, do, and record.

"The system" in this instruction is the five models, their forwards and the tools that score them.
The rest of the application (agent runtime, knowledge service, data ingestion, final evaluation)
belongs to other lanes and is not yours to change.

## What "as good as possible" means, and where to look for it

- **The target is the error on sessions nobody has seen**: the final test, which Tudor's lane runs
  once. It is not the train score and not the validation score. Validation has been read six to
  eight times per family; it cannot tell two droplet models apart below about ±5 to ±10 µm and holds
  two sessions per burner-flame family.
- **What has carried over so far, by size of the measured effect** (`docs/wsp/README.md`, "What five
  generations taught"; `10_extension_map.md`, 10.6). Treat this ranking as a claim to test, then
  spend your effort by it:
  1. knowing more about a test before it starts (the planned stop of a burner test: the largest
     gain of any generation);
  2. a second physical scenario carried beside the first with a typed probability;
  3. a mechanism written in quantities that mean the same thing in another session;
  4. a law rebuilt from property tables in place of numbers read from train: fewer train-read
     numbers, no measurable gain yet;
  5. anything made sharper or narrower because train looked that way: worse, every time it was
     tried.
- **Before you build, find where the error is.** For each family write an error budget from the
  train feedback (`wsp-dev feedback --partition train`; the audits and anatomies in
  `docs/wsp/feedback/`): which runs, conditions and mechanisms carry the error, how much of it is
  measurement artefact that no law can remove, and what score a perfect law could reach. Then list
  the changes you could make, each with the gain you expect on unseen sessions and your reason to
  believe that gain is real. Put the ranked list in your memo and build from the top. A change with
  no entry in that list is not built.

## Read first, in this order

1. `docs/wsp/dossier/README.md`, then `HANDOFF_ASTRA.md` beside it: the state and the rules.
2. `docs/wsp/dossier/07_limits_and_disclosures.md`: what is known to be weak or contaminated.
3. `docs/wsp/dossier/10_extension_map.md`: Fable's position on how a hand-written model can grow, and
   the mechanisms found per family. Treat it as a claim to test.
4. `docs/wsp/dossier/02_data_classification.md`, `03_datasets_and_measurements.md`, `05_claims.md`.
5. `docs/wsp/RESULTS_GEN5.md` and section 8.7 of `docs/wsp/dossier/08_in_progress.md`: what the last
   validation look decided, and how Fable reads it.
6. `docs/wsp/PROTOCOL.md` and `docs/wsp/EXPERIMENTAL_SCIENCE.md` when you need the detail.
7. `AGENTS.md` at the repository root: the rules of the whole project.

## Rules you may not break

They are spelled out in the dossier's README. In short:

- Never read the test partition, a `holdout/` folder, or anything derived from them; never pass
  `--final-test`, `--partition test` or `--partition all`. Tudor's lane owns the final evaluation.
- Validation outcomes are never read per run. Validation is read as inputs, or as an aggregate table
  from a look that was registered in `docs/wsp/PROTOCOL.md` with its rule before it was run. Do not
  open `runs/dev/bass_front/val_sealed/`, the validation series of `runs/dev/data_ext/`, or the
  private vault of validation continuations `runs/dev/holdout_ext/`. Note
  exposure X-18: conditions of consecutive tests in one chamber fill encode the previous test's
  outcome, so do not difference them.
- Train is open. The earlier BASS campaign (PSI-26) is open.
- No number is produced by a program: no fit, regression, optimiser, search or scan. You may measure,
  and you may evaluate a typed law at typed constants. You type every weight.
- Do not read published per-test results of the modelled series (the FLEX ISS droplet tests, the
  CFI-G tests of 2021, the BASS-II concurrent fabric tests).
- Do not change the shared contracts or shared records; do not edit an existing forward or the four
  shared forward sources; do not delete or re-identify anything. A better forward is a new file.
- Say plainly, every time, whether a candidate is zero-shot or data-informed. Once you have read this
  dossier you have read train outcomes and aggregate validation scores: what you type is
  data-informed. If you want a zero-shot candidate, have it written in a clean room: a fresh agent
  that is given only the spec, the physics and class A and B sources, and none of the dossier's
  results.
- Do not download the whole archive (3.4 TB). Fetch single files of train runs from the listings.

## Part 1. Check the first author

Redo the research where it matters and try to break it. `docs/wsp/dossier/README.md` lists where
Fable would attack first; `05_claims.md` gives each claim with the cheapest test that could refute
it. In particular:

- How documents were classified and what reached typed numbers from them (`01_sources.md`; exposures
  X-01 to X-04, X-14 to X-19).
- How the validation look of the fifth generation was run: droplets against a vault made from a
  private copy of the data root, burner flames from a comparison split by a script of this lane
  (X-20 to X-22; `docs/wsp/PROTOCOL.md` section 10; `docs/wsp/RESULTS_GEN5.md`).
- How each NASA file and column was classified (`02_data_classification.md`, section 2.8 lists the
  judgment calls).
- The dataset results, starting with those marked "on trust" (`03_datasets_and_measurements.md`).
- The five physics dossiers under `docs/wsp/research/phys_*.md`: they are proposals by delegated
  agents. Four were re-derived by fresh agents with their own code
  (`docs/wsp/feedback/verify_phys_*.md`); the methanol dossier was not. Their property values were
  looked up in standard sources by another agent (`docs/wsp/research/properties_verified.md`). Fable
  himself recomputed only what `HANDOFF_ASTRA.md` lists.

File each finding in `docs/wsp/dossier/review/` from the template there. A refuted claim changes the
dossier; it never edits a committed candidate.

## Part 2. Research weight-space programming as widely as it goes

Fable's reading of 13 November is in `docs/wsp/RESEARCH.md` and `docs/wsp/research/mit_wsp.md`,
`legora.md`, `related_work.md`. Redo it, then go far beyond it. Directions worth your time, none of
them exhaustive:

- The work that named the method and everything built on or against it since.
- Programs compiled into weights and weights written by construction: RASP and Tracr and their
  successors, constructed transformers that implement algorithms or in-context learning, the
  constructions used in expressivity proofs (how few hand-set units represent a product, a table, a
  piecewise function).
- The inverse problem: mechanistic interpretability, circuits, model editing. What do they say about
  which weight structures a reader can verify?
- Weight spaces as objects: symmetries, canonical forms, tying. What makes a set of typed numbers
  checkable?
- Models written by experts rather than fitted: structured expert judgment and prior elicitation,
  language models as sources of priors, mechanistic models in the sciences whose constants are set
  from canonical experiments.
- Grey-box science: dimensional analysis and similarity as feature design, reduced mechanisms,
  solvers written as layers (method of lines, operator splitting, explicit integrators), property
  libraries.
- Forecasting under uncertainty without fitting: proper scoring rules, the arithmetic of ensembles
  and committees, competing risks and censoring, calibration that needs no data, extrapolation under
  shift.
- Generalisation of a model nobody trained: description length, prior predictive checks,
  preregistration, any accounting of degrees of freedom that applies to typed numbers.
- How to type thousands of numbers and be right: tests of units and limits, redundancy, two
  independent derivations, cross-checks between models that share a constant.
- The combustion science of each family, where the physics dossiers stop or disagree with train.

Research the data and the sources of the experiments as hard as the method. The largest gain so far
came from a NASA file nobody had read as an input, not from a paper:

- **The archive itself.** `data/manifests/PSI-*.listing.jsonl` lists every file of the four
  datasets; `02_data_classification.md` says how each kind was classified and, in section 2.8, lists
  eight calls to make again. Look for what is known before a test starts, or up to the cutoff, and
  is not used: other fields of the script parameter file (whether a filament is in the flame, claim
  C-42), commanded flows where the table is wrong (section 2.7), instrument channels of train runs
  that no forward reads. Argue the classification of every new input on page 02 before a model reads
  it, and keep it in the private copy of the data root; the shared records are not yours to change.
- **Data outside every partition.** Other campaigns and experiments on the same apparatus or the
  same physics, in NASA's PSI catalogue and in the open literature: other fuels, drop towers,
  earlier campaigns. Only one has been used (the earlier BASS campaign, PSI-26). Such data may be
  read in full, can supply constants of nature and of the apparatus, and can serve as a held-out
  check that costs no validation look. Verify run by run that none of it belongs to the validation
  or test partition of a modelled series.
- **The published science of each experiment**, under the class rules of `01_sources.md`: mechanism
  and correlations yes, per-test results of the modelled series no. The three experiment dossiers of
  13 November were written under a weaker rule; redo them under the current one and do not read
  their per-test passages.
- **Property data.** `docs/wsp/research/properties_verified.md` is one agent's look-up of 374
  values. Check it against a second source before anything is built on it.

For every source, record how much of it you read and what you took. Say what you could not verify.
Save the result under `docs/wsp/research/` with your name in the file name.

## Part 3. Think from first principles

Do not start from the existing code. Start from questions such as these, and add your own:

- What is a parameter in a model whose forward is code and whose numbers are typed? Where should
  expressive power sit?
- Where does the generalisation of a hand-written model come from, and what destroys it? Five
  generations here suggest: mechanisms in quantities that mean the same in another session carry
  over; anything sharpened because train looked that way does not; knowing the plan of a test beats
  a better law of the flame; a second scenario with a stated probability beats a sharper single law.
  Is that a law or an anecdote?
- Two models score the same on every held-out set we have, and one of them reads fewer numbers from
  train. Which should be carried, and by what rule written before anyone looks again? The fifth
  generation refused three such candidates for showing no gain.
- How can a model grow by orders of magnitude in typed numbers without growing in freedom towards
  its train data? `10_extension_map.md` proposes a split by where a number's justification comes
  from (nature, apparatus, law, structure, uncertainty). Improve it or replace it.
- When theory gives the dependence and data give the level, where does honest uncertainty for an
  unseen condition come from?
- What is the largest model that could be defended: a resolved transport solver whose every constant
  is typed? A shared property block that all five models read, so that one wrong number shows in two
  experiments? A model of the instruments? One model across families?
- What is the line between typing a number and letting a program choose it (a copied member release,
  a transcribed handbook table, a constant solved from two train points, a table of train runs used
  as a lookup)? Where exactly does hand-written end?
- How should two authors who read different things combine what they wrote?

Write your answers as a design memo in `docs/wsp/dossier/` (your name in the file name), short
enough to be argued with.

## Part 4. Extend the models, as ambitiously as you can defend

- Choose the designs your research and your first-principles memo support. Write new forwards as new
  files with new model names; keep the predecessor reachable (a test that the new forward equals the
  old one when its new parameters are neutral is the house pattern).
- Type every number yourself, with its reason and its kind of origin beside it. Count in each card
  how many numbers were typed in total and how many were read from train.
- Candidates go under `configs/models/<family>/<spec>/<name>-astra/` with your own `author.model_id`
  and an exposure record that includes this dossier. `docs/wsp/dossier/review/README.md` gives the
  steps; `09_reproduce.md` the commands.
- Check on train (`wsp-dev feedback --partition train`). Commit a candidate before any validation
  score of it exists. A validation look needs a rule written in `PROTOCOL.md` first, agreed with
  Fable through the review folder; the partitions are worn, so spend looks rarely. The fifth
  generation spent one look per family on 14 November: the two burner-flame candidates were adopted,
  the droplet and fabric candidates were not.
- Where your candidate and Fable's differ in kind, the equal-weight committee of the two is the
  honest joint statement and needs no validation to justify.

Measure generalisation without spending validation. These cost no look:

- **A clean-room candidate scored on train.** An agent that was given only the spec, the physics
  and class A and B sources has not seen train, so its train score is an out-of-sample score on 68
  and 49 droplet runs and 81 and 73 burner tests: many times what validation holds, and not worn.
  Use it to compare ways of writing a model (which inputs, a law or a patch, one scenario or two).
  Keep the agent clean: no dossier, no feedback file, no train table, no number of yours. State the
  comparison before it is run and keep it to a few candidates, or train becomes selection data
  again. The forward itself was designed by authors who saw train; the score measures the typed
  numbers, not the structure.
- **A campaign outside the partitions** as a held-out check, where one exists for the family.
- **Checks that need no outcome**: units, limiting cases, and forecasts over the whole range of
  conditions an unseen session can hold (`docs/wsp/feedback/shift_map_validation_inputs.md` shows
  where the validation inputs lie relative to train; inputs only).
- **Session by session on train** (decision D-40). Necessary, known not to be sufficient.

What is waiting, concretely, is in `HANDOFF_ASTRA.md`.

## How to work

- Parallelise analysis with sub-agents at high effort, under these same rules, train only and no
  fitting. Recheck what you act on. No sub-agent and no script writes a weight.
- Keep the dossier current: every new source, measurement, decision, claim or exposure gets its row
  in the same commit, with an honest status (rechecked, spot-checked, on trust).
- Run the gates before each commit: `cargo fmt --all -- --check`, `cargo check --workspace
  --all-targets`, `cargo test -p ignis-wsp -p ignis-wsp-dev -p ignis-core`. Do not push or open a
  pull request unless the owner asks.
- Report failures and negative results as plainly as successes. A refused candidate stays.

## Done looks like

1. Findings filed for every claim you tested, with verdicts.
2. A sourced research file on weight-space programming and the techniques around it, and one on the
   data and sources of the experiments: every new file, input or outside campaign you found, how
   you classified it, and what it gave.
3. A first-principles design memo, with the error budget of each family and the ranked list of
   changes, and what became of each entry.
4. New forwards and candidates, committed before any validation look, each said to be zero-shot or
   data-informed, each with its count of numbers by origin.
5. The dossier updated so that a third reader could check you the way you checked Fable.
