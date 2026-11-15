# Astra's goal: weight-space programming for all of NASA

From the first author (`claude-fable-5-2`) at the owner's request, 14 November 2026, night. **This
file is Astra's whole goal from now on.** It replaces every earlier instruction to Astra where
they differ; [ASTRA_PROMPT_ALL_NASA.md](ASTRA_PROMPT_ALL_NASA.md) stays as the description of the
document lane's work and of the rules that both lanes share. Paths are relative to
`D:\dev\SpaceApps-Ardealu`, branch `feat/handwritten-science`.

## The owner's words

- "trebuie să facem modele folosibile PE TOT NASA"
- "să folosim Sonnet 5.5 High subagents atât pt citire cât și pt îmbogățire, iar Astra să se
  concentreze absolut doar pe WSP"
- "să fie cât se poate de ambițios, să dea research încă o dată la toate paper-urile, să gândească
  din first principles și să dea exercise la abstract thinking"
- "nu ține deloc cont de timp"; "best of the best", "cât timp e WITHIN SCOPE"
- from the first instruction: research everything that bears on weight-space programming, find
  the best techniques for extending these models by parameters, and be as ambitious as honest
  hand-writing allows.

Do not ask the owner for anything. Decide, do, and record.

## The goal

**Hand-written models that are usable on every one of NASA's 19 flight combustion investigations**
of the PSI repository. For each investigation, a release that:

1. accepts its tests instead of refusing them;
2. forecasts a quantity that matters for fire safety (ignition or not, extinction, burn time,
   flame size and shape, spread rate, soot and smoke, the response to flow, oxygen, pressure and
   gravity);
3. states where it may be used: the conditions it was checked on and how far outside them a
   forecast is extrapolation;
4. has a score on groups it has not seen, from a test registered beforehand, or the plain
   statement that PSI holds nothing that can score it yet.

Every number in every model is typed by its author with its reason. Nothing is fitted.

## What is not yours any more

Reading, labelling, enrichment, narratives, test labels, clusters, links, the table-against-sensor
discrepancies, the measurement of images and video, the export to the application and the upload
to the VM belong to the **document lane**: Fable with Claude Sonnet 5.5 sub-agents at high effort.

- **Stop** your Terra runner, page qualification, labels, discrepancy script and frame
  measurement. Commit what exists of them by explicit path, with one line in your ledger saying
  where each stopped. The document lane takes them over from that commit. Do not touch the
  transfer to the VM.
- Being out of those roles is what keeps you blind: an author who labels or measures sees
  outcomes.

## What you receive, and how to ask

From the document lane, as from any measurer, in `runs/dev/data_docs/release/` (it will appear as
the lane produces it):

- condition supplements (FLEX-2 fuel and mixture per test point, BASS PSI-26 test rows, commanded
  flows and planned stages);
- page cards of methods, apparatus, requirements and properties (never a page marked as revealing
  an outcome);
- measurements, test labels and table-against-sensor discrepancies of **train groups only**, and
  discrepancies of conditions for every group.

Ask for what a model needs in `docs/wsp/measure/REQUESTS.md`: what, for which tests, why. The lane
answers there. You never receive outcomes of validation, test or held groups, and you do not open
`runs/dev/data_docs/` outside `release/`.

## Rules

1. **Protected data.** The test partitions and `holdout/` of FLEX, ACME CFI-G, BASS and BASS-II are
   never read; their validation outcomes never per run. `runs/dev/bass_front/val_sealed/`,
   `runs/dev/holdout_ext/` and the validation series of `runs/dev/data_ext/` are never opened.
   `--final-test`, `--partition test` and `--partition all` are never passed. Tudor's lane owns
   the final evaluation.
2. **Frozen groups** (`docs/wsp/atlas/splits/`). For the 15 other investigations you read outcomes
   of train groups only; validation as an aggregate from a look registered beforehand; test groups
   never. A held investigation is not train: no outcome of it per test, scored only as an
   aggregate in a registered look.
3. **Every number is typed** by you with its reason and its kind of origin. No fit, regression,
   optimiser, search, scan or generated value. A zero-shot candidate is written in a clean room.
4. **Published per-test results of a modelled series are not read**; mechanism, methods,
   apparatus and correlations are.
5. **Existing forwards, releases and shared sources are not edited.** A better forward is a new
   file that returns exactly what its predecessor returns on the predecessor's cases.
6. **Shared contracts and shared records are not changed.** A quantity the contracts lack is a
   proposal in `docs/wsp/measure/ASTRA_CONTRACT_PROPOSALS.md`; no output takes another
   quantity's name to pass the registry. **Nothing is pushed.**
7. **Say what a candidate is, every time**: zero-shot or data-informed. Everything you type is
   data-informed. No hand-written model is called NASA-trained; no posterior is claimed.
8. **Conditions of consecutive tests in one chamber fill are not differenced** (exposure X-18).
9. **No frontier model reads documents in bulk.** Your research analysts read the few pages that
   matter, chosen from page cards and abstracts.

## Where things stand

- **The first four investigations**: five generations of models on FLEX, CFI-G and BASS-II
  (`docs/wsp/README.md`, `RESULTS_GEN5.md`, dossier pages 01 to 10). Adopted in the fifth: the two
  burner-flame candidates; not adopted: the droplet and fabric candidates. Validation of these
  families is worn (six to eight looks) and cannot separate close candidates.
- **Flame Design transfer** (PROTOCOL 11.1 and 11.3): with ethane standing in for ethylene, the
  normal-flame model beats the incidence baseline at extinction (0.120 against 0.247); the inverse
  model is undecided; the reference set with no train number does as well as the inverse model.
  What carried over is the structure of the law. Reading the plan of a test lowered the
  flow-termination score from 0.317 to 0.089.
- **What stops the models on the rest of NASA**: they take the fuel by name. Flame Design burns
  ethylene on 310 of 327 tests, CFI n-dodecane, its mixtures and farnesane on all 227.
- **Yours, on disk**: symbolic research for droplets, spherical burners, jets and the burning-rate
  emulator, solids and aerosols; a property audit; a physical core with ten tests; a property-based
  droplet W0 kernel frozen before any feedback; ten earlier candidates, all data-informed; contract
  proposals. Most of it is not committed.
- **What five generations taught**, by size of the measured effect: knowing more about a test
  before it starts; a second physical scenario with a typed probability; a mechanism in quantities
  that mean the same in another session; a law rebuilt from property tables (fewer train-read
  numbers, no measurable gain yet); anything sharpened because train looked that way was worse
  every time. Treat it as a claim to test, then spend effort by it.

## What to do

### 1. Find where the error is, before building

For each family write an error budget from train feedback (`wsp-dev feedback --partition train`,
the audits in `docs/wsp/feedback/`): which tests, conditions and mechanisms carry the error, how
much is measurement artefact no law can remove, what a perfect law could reach. Then a ranked list
of changes, each with the gain expected on unseen groups and the reason to believe it. A change
with no entry in that list is not built.

### 2. One shared property block

Fuel, diluent and solid enter as typed physical properties, never as a name: every fuel, diluent
and solid in the 19 tables, each value with its source and checked against a second source; a
tested block every family reads, so that one wrong number shows in two experiments.

### 3. Few families, not nineteen models

Droplets; spherical burner flames; the burning-rate emulator; jet and coflow flames; solids in a
flow; smoke and aerosol. Tie constants across families where the physics is the same. Where the
target is one number per test, one generic forward: conditions in, the target with a typed spread
out. Gravity is an input; a forecast for partial gravity is marked as one no PSI data can check. A
scenario without a test (a crew atmosphere, a material, a flow) gets a flag saying whether it lies
inside the data the model was checked on. Use what is known before a test starts (planned stages,
commanded flows, hardware): it was the largest gain of any generation; argue the classification of
every new input on dossier page 02 before a model reads it.

### 4. Zero-shot first, then data-informed

- A **clean-room candidate per family**, written by a fresh agent given only the spec, the physics
  and class A and B sources: no dossier, no feedback file, no train table, no number of yours. Its
  score on train groups is an out-of-sample score and costs no validation look; state the
  comparison before it is run and keep it to a few candidates.
- Then **data-informed candidates** on train groups, committed before any validation look, each
  with its count of numbers by origin. Where your candidate and Fable's differ in kind, the
  equal-weight committee is the honest joint statement.
- Candidates go under `configs/models/<family>/<spec>/<name>-astra/` with your `author.model_id`
  and an exposure record (`review/README.md` gives the steps, `09_reproduce.md` the commands).

### 5. Register before scoring

A subsection of `docs/wsp/PROTOCOL.md` section 11 per test, committed before its outcomes are read
(11.1 is the pattern): releases, tests included, endpoints, metric, baseline, what each result will
mean. The scoring chain first reproduces known train numbers (`tools/wsp/ext/transfer_acme.py`).
Scores of validation groups come to you only as the aggregate table of that registered look.

### 6. Research and first principles

- The combustion science of each family and of each of the 19 investigations, again, from the
  primary papers: what governs the forecast quantity, in which regime, with which limits.
- The method: programs compiled into weights and weights written by construction; what
  interpretability and model editing say about which weight structures a reader can verify;
  symmetries and tying; expert-set models and prior elicitation; grey-box science (dimensional
  analysis, reduced mechanisms, solvers written as layers, property libraries); forecasting
  without fitting (proper scoring rules, committees, competing risks, calibration without data,
  extrapolation under shift); how a model nobody trained can be said to generalise; how to type
  thousands of numbers and be right.
- The memo's questions: what a parameter is when the forward is code and the numbers are typed;
  where the generalisation of a hand-written model comes from and what destroys it; how a model
  grows by orders of magnitude in typed numbers without growing in freedom towards train; where
  honest uncertainty for an unseen condition comes from; where typing ends and a program choosing
  begins.
- The loop for each question: state it and what answer would change a model; find sources; derive
  in symbols before any number; a fresh checker reaches the result its own way; the difference is
  the result. Files: `docs/wsp/research/astra_*`, the memo
  `docs/wsp/dossier/ASTRA_FIRST_PRINCIPLES_ALL_NASA.md`; for every source, how much was read and
  what was taken.

### 7. Check the first author's models

Redo where it matters and try to break it: classification of inputs (page 02, section 2.8), the
claims of page 05 with their cheapest refuting test, the physics dossiers `research/phys_*.md` (the
methanol one was never re-derived), the fifth-generation validation look (X-20 to X-22), the
Flame Design transfer (M-25 to M-27, C-46 to C-48). File each finding in `docs/wsp/dossier/review/`
from the template; F-001 to F-011 wait for Fable's answer once committed.

### 8. Record

Every source, decision, claim, exposure and candidate in its dossier page, in the same commit as
the work, with an honest status; your ledger `ASTRA_ALL_NASA_PROGRESS.md` kept true. Failures and
negative results as plainly as successes. A refused candidate stays. **Commit often**, by explicit
path.

## Done looks like

1. An error budget and a ranked list of changes per family, and what became of each entry.
2. The shared property block, tested, every value sourced twice.
3. Forwards for the six families, each new file reproducing its predecessor where one exists.
4. A clean-room candidate per family and data-informed candidates, each with its count of numbers
   by origin.
5. For each of the 19 investigations: a release that accepts its tests, a registered test, and its
   result or the reason there is none.
6. The research files and the first-principles memo current.
7. Findings filed for every claim of Fable's you tested, with verdicts.
8. The dossier and the ledger current, everything committed.

## Working beside Fable

- Your places: `tools/wsp/astra/` (model tools), `docs/wsp/research/astra_*`,
  `configs/models/**/<name>-astra/`, new forward files, your dossier pages, the review folder,
  `docs/wsp/measure/REQUESTS.md` and `ASTRA_CONTRACT_PROPOSALS.md`, new subsections of PROTOCOL
  section 11.
- Not yours: `docs/wsp/atlas/`, `tools/wsp/atlas/`, `tools/wsp/ext/`, `tools/wsp/docs/`,
  `runs/dev/data_all/`, `runs/dev/data_docs/` outside `release/`.
- One checkout, two authors: commit only your own files by explicit path; never `git add -A`; no
  formatter over the whole crate; no file committed that differs by line endings only.
- Build into your own target folder (`CARGO_TARGET_DIR`).
- Gates before each commit: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`,
  `cargo test -p ignis-wsp -p ignis-wsp-dev -p ignis-core`.
