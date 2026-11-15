# Instruction for Astra: the whole goal

From the first author (`claude-fable-5-2`) at the owner's request, 14 November 2026, night. **This
file is the whole goal.** It replaces every message relayed to you today and it carries what still
holds of the first instruction, [ASTRA_PROMPT.md](ASTRA_PROMPT.md), which stays in the repository as
history and need not be read again. Where an earlier text and this file differ, this file holds.
Paths are relative to the repository root, `D:\dev\SpaceApps-Ardealu`, branch
`feat/handwritten-science`.

Steps 1 to 8 and 4b keep their numbers and headings from the previous version. What is new: the
goal in short, the section on which model does what, the section on how research and findings work,
step 9, the section on the first instruction, rules 9 to 11, and an updated state.

## The goal in short

You are Astra, the second author of the hand-written scientific models of IGNIS: forecasting models
for NASA's microgravity combustion experiments in which every parameter is a number an author model
types by hand, with its reason, and nothing is fitted. Two things are asked of you.

1. **Make the models usable on all of NASA**: every one of the 19 flight combustion investigations
   of the PSI repository, not only the four the models were written on.
2. **Turn NASA's archive into something a model and a reader can use**: every page read and
   labelled, every investigation described from its documents, every test labelled, clustered and
   linked, NASA's tables checked against NASA's own sensors, images and video turned into numbers.

Do not ask the owner for anything. Decide, do, and record.

## Who does which part (owner's decision, 14 November, night)

The owner: "să folosim Sonnet 5.5 High subagents atât pt citire cât și pt îmbogățire, iar Astra să
se concentreze absolut doar pe WSP".

- **Astra: weight-space programming only.** Steps 6 and 7 (the property block, the forwards, the
  clean-room and data-informed candidates, the registered tests, the research of the method and
  of the physics), step 8 for that work, and the review of the first author's models. Nothing
  else.
- **The document lane: everything else**, run by the first author (`claude-fable-5-2`) with
  Claude Sonnet 5.5 sub-agents at high effort: step 1 (acquisition is done; the corpus is on the
  VM), step 2 (page cards and search enrichment, read and written in one pass by Sonnet), step 3
  (narratives), step 4 (test labels, clusters, links, findings), step 4b (discrepancies), step 5
  (measurement, by scripts and blind Sonnet measurers) and step 9 (the export).
- **Stop** the Terra runner, the page qualification, the narratives, the labels, the discrepancy
  script and the frame measurement. Commit what exists of them by explicit path, with a line in
  your ledger saying where each stopped; the document lane takes them over from the commit.
- **What you receive from the document lane**, as from any measurer: condition supplements,
  measurements and test labels of train groups only, the discrepancies of conditions, the page
  cards of methods, apparatus and properties. You never receive outcomes of groups outside train.
  Ask for what a model needs in `docs/wsp/measure/REQUESTS.md`; the lane answers there.
- **Being out of the document roles helps you**: an author who measures or labels cannot be blind.
  Step 5's rule that a model's author does not measure now holds by construction.
- The rest of this file still describes the document steps, so that you know what you will get and
  under which rules. Where it says "your" for steps 1 to 5, 4b or 9, read "the document lane's".

## What the owner said

- "trebuie să facem modele folosibile PE TOT NASA"
- "să descarce toate astea și să generalizeze modelele să ocupe TOT, să analizeze el foarte multe
  videouri/imagini și să facă best possible stuff, să fie cât se poate de ambițios, să dea research
  încă o dată la toate paper-urile, să gândească din first principles și să dea exercise la abstract
  thinking"
- "nu ține deloc cont de timp ... ignoră complet aspectul timpului"
- "vreau să avem best of the best; nu trebuie nici să extindem prea mult scope-ul, dar ce putem
  trata hai să tratăm cât timp e WITHIN SCOPE"
- on reading documents with a frontier model (superseded on 14 November, night, by the section on who does which part): "nu citi cu Fable că e enorm de scump"; a cheap model
  (GPT 6.1 Terra) checks the pages, and the interpretation of the experiments stays with your own
  sub-agents.
- "noi o să aplicăm și SIRA (by Meta) în repo": the search enrichment of each page is written at
  reading time.
- on the differences between NASA's tables and NASA's sensors: "hai să îi dăm la Astra și chestia
  asta", and, on which side is wrong, "trebuie să ne gândim cum rezolvăm problemele astea".
- from the first instruction: research everything that bears on weight-space programming, think
  abstractly and from first principles, find the best techniques for extending these models by
  parameters, and be as ambitious as honest hand-writing allows.

## Scope

The challenge is NASA Space Apps 2026, "Flame in Freefall": a dashboard that summarises, ranks and
interprets NASA's microgravity combustion investigations to give fire-safety insight for crewed
spacecraft, stations, surface bases and rovers, and that extracts data which supports predictive
modelling.

- **Inside:** the 19 flight combustion investigations of NASA's PSI repository; their documents,
  tables, sensor text, images and video; labels, links and findings about them; hand-written models
  that forecast a fire-safety quantity of their tests; the crew atmospheres the challenge names
  (Earth and ISS; Moon base and rovers at 30 % oxygen or more and 10.2 psi or less; the spacesuit);
  gravity as bounded by microgravity and normal gravity.
- **Outside:** rockets and propellants; data of other agencies; the five ground studies in PSI as
  data (they are literature); anything in the application that belongs to another lane.
- Be as ambitious as you can defend inside that line, and do not move the line.

**What the work is for.** The application (`MirceaGodiac/ignis`, another lane) shows NASA's
documents with their results, one card per investigation, the ranking by crew habitat, the
forecasts of the models and the list of discrepancies. Step 9 says what you hand over. The
challenge requires that the use of AI is declared, so every artefact you produce names the model
that wrote it.

## Which model does what

Reading at volume is where the cost goes, and a frontier model adds little there. The work is split
by role. No role does another's job.

| Role | Who | Gets | Never gets | Produces |
|---|---|---|---|---|
| Acquisition | a script | NASA's listings | | files on disk, the fetch log |
| Page reader | **Claude Sonnet 5.5 at high effort** sub-agents of the document lane, no tools | the extracted text of one whole document, pages marked; the image only of a page that is mostly a figure or a table | files, tools, the models, partitions | one row per page: the page card and the search enrichment (step 2) |
| Gold labeller | a strong sub-agent, on the 200-page sample only | the same pages, blind to Sonnet | Sonnet's rows | the labels Sonnet is measured against |
| Risk reviewer | a strong sub-agent or a conservative rule | only the exception pages: possible outcome of a test, low confidence, two passes that disagree | | the final label of those pages |
| Narrative writer | a Sonnet 5.5 High sub-agent of the document lane | the page cards of one investigation and the dozen or two pages Sonnet marked as summary or conclusions | whole reports; outcomes per test | `narratives/PSI-<id>.json` (step 3) |
| Research analyst | an Astra sub-agent at high effort, one per family or question | primary sources of class A and B, chosen from page cards and abstracts; for a long source, Sonnet's distillate first and then the pages that matter | per-test results of a modelled series; validation or test outcomes | a derivation, a research file, review findings |
| Independent checker | a fresh Astra sub-agent | the question and the sources, not the first analyst's answer | the first analyst's code and numbers | the same result reached its own way, or the difference |
| Test labeller | an agent outside the modelling roles | measurements and documents of a test | models, hypotheses | a phenomenon label with its evidence (step 4) |
| Measurer | **a script**, once its method is proven; blind agents only on the sample of frames that proves or audits the script | the instrument and the frames | a model, a hypothesis, an expected value, a partition | series with uncertainty (step 5) |
| Cross-checker | a script | a table and a sensor file of one test | | discrepancy records (step 4b) |
| Clean-room author | a fresh Astra agent | the spec, the physics, class A and B sources | the dossier, train tables, feedback files, any number of yours | a zero-shot candidate |
| Data-informed author | you | train groups, aggregate validation tables from registered looks | everything rule 1 and rule 2 close | candidates, each number typed with its reason |
| Refuter | an Astra sub-agent | one finding and its evidence | | the cheapest test that could break it, run |

- **Measuring at volume is a script's work too.** An agent measuring frame by frame costs like an
  agent reading page by page. Agents measure enough frames to prove a tracker against NASA's own
  numbers and to audit it afterwards; the tracker measures the rest.
- An agent that has seen something its role must not see leaves that role for good, and the ledger
  says so. Your ledger already does this; keep it.
- Sub-agents work under these same rules. No sub-agent and no script writes a weight.
- **At the end, a committee of three other models** (Muse Spark 1.3 Contributor, Opus 5.5, GPT 6.1
  Sol) will judge the claims of this work. It is not yours to run. It means that every claim you
  leave must stand on its own: the statement, the evidence in a form that can be rerun, and what
  it does not show.

## How research and findings work

**The loop, for each family and each open question.**

1. State the question and what answer would change a model, a label or a claim.
2. Find the sources: NASA's own documents through the page cards; the open literature under the
   class rules of `01_sources.md` (mechanism, methods, apparatus and correlations yes; per-test
   results of a modelled series no).
3. Sonnet distils what is long. A strong analyst reads the distillate and the few pages that matter.
4. The analyst derives from first principles, in symbols, before any number: what the conserved
   quantities are, which dimensionless groups govern, what the limits must be.
5. A fresh checker reaches the same result its own way, with its own code. Where they differ, the
   difference is the result.
6. The outcome goes into the research file of the family (`docs/wsp/research/astra_*`), with, for
   every source, how much of it was read and what was taken, and what could not be verified.
7. For each family, an error budget from train feedback: which tests, conditions and mechanisms
   carry the error, how much is measurement artefact no law can remove, what a perfect law could
   reach. Then a ranked list of changes, each with the gain expected on unseen groups and the
   reason to believe it. A change with no entry in that list is not built.

**Three kinds of finding, three places.**

| Kind | What it is about | Where | Form |
|---|---|---|---|
| Review finding | a claim, decision, measurement, exposure or classification of the dossier, Fable's or your own | `docs/wsp/dossier/review/F-NNN-short-title.md`, from the template | what you did, evidence that can be rerun, verdict (confirmed, refuted, weakened, cannot tell, new), severity (integrity, model, record). The other party answers under `## Response`; nothing is edited, replies are appended |
| Scientific finding | an investigation, a cluster of tests, a family | the narrative files (step 3), `findings.jsonl` (step 4), the research files | a statement of at most 45 words, marked "NASA states" (with locator and quote) or "interpretation" (with the evidence and the test that could refute it) |
| Discrepancy | two NASA sources that disagree about one test | `discrepancies.jsonl` (step 4b) | both values, the method, the check that decided the cause, the verdict |

- A refuted claim changes the dossier. It never edits a committed candidate.
- A disagreement between the two authors is settled by a train-only check stated before it is run,
  or by a class A or B source, or by carrying both as two members; never by a validation look and
  never by the test partition (`review/README.md`).
- Before a finding is filed, a refuter tries the cheapest test that could break it.
- Findings F-001 to F-011 are filed and wait for Fable's response. He answers what is committed:
  commit the review folder and the work the findings point to.

**Where something new could be found.** These are the places this project has already seen a
signal. Hunt there, inside the scope:

- **Systematic differences between NASA's tables and NASA's sensors.** The readback flow is 1.083
  times the table flow across CFI-G train, a constant factor nobody has explained. A factor that
  holds across experiments on the same hardware is a statement about the hardware.
- **What carries from one investigation to another.** On Flame Design the structure of the
  burner-flame law carried over to another fuel while the numbers read from CFI-G added little.
  Which structures carry across fuels, burners and families is a result about the physics.
- **Constants that two experiments share.** A property or a hardware constant that enters two
  families can be checked by both. Two investigations that disagree about one shared property
  first test the reading, then the property.
- **Where NASA's tests do not reach the crew atmospheres.** Of 1,612 catalogued tests, 918 carry
  oxygen and pressure, 186 lie in the Earth and ISS atmosphere, 17 in the planned Moon-base
  atmosphere (all of them CFI-G), none in the spacesuit atmosphere. A map of what has never been
  tested is a fire-safety finding in itself.
- **Tests whose record contradicts itself** (step 4b).

Say "new to us" until the literature has been searched for it. Negative results are kept and
reported as plainly as positive ones.

## Where things stand

**Fable's, finished. Read, never redo, never write there.**

| What | Where |
|---|---|
| File listings and NASA's metadata record of all 19 investigations | `runs/dev/data_all/manifests/`, `runs/dev/data_all/raw/samples/_meta/`; the four first ones also in `data/manifests/` |
| File catalogue: 66,501 listing rows, 60,800 distinct files (NASA's listings repeat 5,701 rows), each with kind, instrument and test id | `docs/wsp/atlas/FILES.md`, `runs/dev/data_all/work/atlas/files.jsonl` |
| Test catalogue: 1,612 rows of 18 investigations in one schema, conditions and group ids (BASS, PSI-26, has no row yet) | `docs/wsp/atlas/SCHEMA.md`, `INVESTIGATIONS.md`, `runs/dev/data_all/work/atlas/tests.jsonl` |
| Evaluation groups, frozen: 9 investigations split 60 / 20 / 20 by test day (10 strata), 6 held whole (Saffire-I, II, III, DAFT, SAME, FLEX-2) | `docs/wsp/atlas/splits/`, `docs/wsp/PROTOCOL.md` 11.2 |
| One card per investigation, and the ranking by crew habitat and fire-safety need | `docs/wsp/atlas/cards/`, `CARDS.md`, `HABITAT_RANKING.md`, `tools/wsp/atlas/build_cards.py` |
| What the server stores (13.9 GB of 13.47 TB: 636 documents, 24,482 sensor text files) | `docs/wsp/atlas/SERVER_STORAGE.md`; also on `main` of `MirceaGodiac/ignis` |
| Transfer test of the frozen burner-flame models on Flame Design | `docs/wsp/PROTOCOL.md` 11.1 and 11.3, `docs/wsp/feedback/flame_design_transfer_result.md` |
| Five generations of models on FLEX, CFI-G and BASS-II; the fifth decided on 14 November (the two burner-flame candidates adopted, the droplet and fabric candidates not) | `docs/wsp/README.md`, `docs/wsp/RESULTS_GEN5.md`, the dossier pages 01 to 10 |
| Table values of 14 CFI-G train tests contradicted by the sensors, 3 unsettled | `02_data_classification.md`, section 2.7 |

Do not open `runs/dev/data_all/work/atlas/tests_outcomes.jsonl`, `runs/dev/data_all/work/flame_design/`
or `runs/dev/data_all/work/transfer_acme/`: they hold outcomes per test.

**Yours, by your own ledger** (`ASTRA_ALL_NASA_PROGRESS.md`, as it stood when this was written):
24,701 of 25,118 objects acquired, 417 unresolved; 5,866 pages inventoried with their text; the
author boundary with its tests and a screened projection of the conditions; the 200-page strong
sample frozen before any Terra output; the Terra batch runner in preparation; a measurement pilot
of 18 of 1,468 frames by blind operators; symbolic research for droplets, spherical burners, jets
and the burning-rate emulator, solids and aerosols; a property audit; a physical core with ten
tests; a first clean-room droplet kernel frozen before any feedback; ten earlier candidates, all
data-informed; proposals for quantities the shared contracts lack. No narrative file, no full Terra
pass, no new release and no registered test of a new family exists yet. Most of this is not
committed.

**Facts that shape the work.**

- The existing models take the fuel by name. Flame Design burns ethylene on 310 of 327 tests and CFI
  burns n-dodecane, its mixtures and farnesane on all 227; the forwards refuse both. That is what
  stops the models on the rest of NASA.
- On Flame Design, with ethane standing in for ethylene, the normal-flame model beats the incidence
  baseline at extinction (0.120 against 0.247); the inverse model is undecided; the reference set
  with no train number scores as well as the inverse model. What carried over is the structure of
  the law. Reading the plan of a test lowered the flow-termination score from 0.317 to 0.089.
- For FLEX-2 the fuel and the gas composition of a test point are available (folder names, 124
  small `_Mixture.txt` files); which reading belongs to which burn is not worked out.
- What five generations taught, by size of the measured effect: knowing more about a test before it
  starts; a second physical scenario carried beside the first with a typed probability; a mechanism
  written in quantities that mean the same in another session; a law rebuilt from property tables
  (fewer train-read numbers, no measurable gain yet); anything sharpened because train looked that
  way was worse every time. Treat the ranking as a claim to test, then spend effort by it.
- The target is the error on groups nobody has seen. Validation of the first four investigations
  has been read six to eight times per family and cannot separate close candidates.

## Rules

1. **Protected data.** The test partitions and `holdout/` of FLEX, ACME CFI-G, BASS and BASS-II are
   never read; their validation outcomes never per run. `runs/dev/bass_front/val_sealed/`,
   `runs/dev/holdout_ext/` and the validation series of `runs/dev/data_ext/` are never opened.
   `--final-test`, `--partition test` and `--partition all` are never passed. Tudor's lane owns the
   final evaluation.
2. **Frozen groups.** For the 15 other investigations an author reads outcomes of train groups only;
   validation as an aggregate from a look registered beforehand; test groups never. A held
   investigation is not train: no outcome of it is read per test by anyone who types a number, and a
   model is scored on it only as an aggregate in a registered look.
3. **Measuring is separate from modelling.** An agent that measures frames or traces gets the
   instrument and the frames, never a model, a hypothesis, an expected value or a partition. An
   author reads measurements of train groups only.
4. **Every number in a model is typed** by its author with its reason. No fit, regression,
   optimiser, search or scan. A zero-shot candidate is written in a clean room.
5. **Published per-test results of a modelled series are not read by an author**; mechanism,
   methods, apparatus and correlations are.
6. **Existing forwards, releases and shared sources are not edited.** A better forward is a new
   file that returns exactly what its predecessor returns on the predecessor's cases, by test.
7. **Shared contracts and shared records are not changed. Nothing is pushed.**
8. **No frontier model reads documents in bulk** (the section on which model does what; step 2).
9. **Say what a candidate is, every time**: zero-shot or data-informed. Having read the dossier you
   have read train outcomes and aggregate validation scores; what you type is data-informed. No
   hand-written model is called NASA-trained; no posterior is claimed.
10. **Conditions of consecutive tests in one chamber fill are not differenced**: they encode the
    previous test's outcome (exposure X-18).
11. **The whole archive is not downloaded** (13.47 TB). Images and video come one archive at a time
    (step 1).

## What to do, in this order

### 1. Finish the acquisition

Documents, analysed data and sensor text of all 19 investigations, into your own root
(`runs/dev/data_astra/`). Images and video one archive at a time: measure, keep the measurements and
a few frames as evidence, delete the archive; peak use under 40 GB. Log every fetch. Close the 417
unresolved objects or list each with the reason it cannot be had.

### 2. Page cards and search enrichment: a cheap model reads every page

- **Claude Sonnet 5.5 at high effort writes the page card of every page**: what kind of page it is (methods,
  apparatus, requirements, results table, figure, discussion, drawing); which investigation and
  which tests or conditions it speaks about, by the test ids of the catalogue; which quantities it
  reports, with units; which phenomenon and which fire-safety need, from the closed lists of
  `docs/wsp/atlas/SCHEMA.md`.
- **The same pass writes the search enrichment** for the page: the concepts it is about, the
  synonyms and abbreviations it uses, the phrasings a reader would search with, and the named
  instruments, materials and test ids. No second model and no second reading.
- **Give Sonnet the whole document, not a slice of it.** A page read alone loses what the document
  said earlier: what a test id means, what an abbreviation stands for, which experiment a table
  belongs to. One request carries the extracted text of the entire document, every page marked
  with its page id, and returns one row per page. Sonnet opens no file and calls no tool. Only a
  page that is mostly a figure or a table gets its image as well.
- **A long document is still read whole, and answered in parts.** Where the rows of all pages would
  not fit one answer (set the page count from the measured output size on the qualification
  sample; expect a few dozen pages), send the same whole document as the unchanged leading part of
  each request and ask for the rows of one run of pages at a time, so the provider's prompt cache
  serves the document after the first request. Never cut the document to make it fit; a document
  larger than the model's input is split at its own chapter boundaries, with the front matter and
  the list of abbreviations repeated.
- A script validates every row against the schema, checks that every page id of the document came
  back exactly once, and sends back what fails.
- **Measure Sonnet before trusting it.** On the 200 pages of the frozen sample, labelled by a strong
  agent and by Sonnet blind to each other. Keep Sonnet for a kind of label only where they agree on
  nine pages in ten; where they do not, repair the definition of the label and measure again.
- **Three things stay with a strong agent or a conservative rule:** whether a page reveals how a
  test ended (when in doubt it does, because a miss hands a held-out outcome to an author); a
  figure or table page on which Sonnet reports low confidence; any label two passes disagree on.
- **Stop labelling pages with strong agents** beyond that sample.
- **Make the pass fast.** Documents are independent, so the full pass is a matter of how many
  run at once. Three requests at a time is too few for 253 documents: run as many as the
  provider's rate limit allows (start at 24, back off on rate-limit responses, never drop a
  batch). Record the wall time and the tokens per batch on the qualification sample.
- **Qualify two efforts on the same 200 pages**: the one the owner set and the next one down. If
  the lower effort passes the same nine-in-ten rule on every kind of label, use it for the full
  pass and say so; the exception pages go to a strong agent either way.

### 3. The document part of the 19 cards

Fable's cards already carry NASA's own objective, approach and impacts and the computed conditions.
They lack what only the documents give. For each investigation write
`docs/wsp/atlas/narratives/PSI-<id>.json` (the one place under `docs/wsp/atlas/` you write to):

```text
{ "schema": "ignis.atlas.narrative/1", "investigation_id": "PSI-..",
  "how_tests_differ": {"text": "<at most 90 words>", "sources": [LOC]},
  "sensors_and_measurements": [{"instrument": "..", "measures": "..", "source": LOC}],
  "findings": [{"statement": "<at most 45 words>", "kind": "NASA states" | "interpretation",
                "source": LOC, "quote": "<at most 20 words, or null>"}],      3 to 6 of them
  "fire_safety_relevance": {"text": "<at most 60 words>", "kind": "..", "sources": [LOC]},
  "need_check": {"agrees_with_card": true | false, "comment": ".."},
  "not_found": [".."], "documents_read": [{"file": "..", "pages_read": ".."}],
  "written_by": "<model id>", "date": ".." }
LOC = {"file": "<NASA file name or 'PSI metadata record'>", "page": <number or null>, "field": ".."}
```

- A strong sub-agent writes it **from the page cards and from the dozen or two pages Sonnet marked
  as summary or conclusions**, never from a whole report.
- Findings are about the investigation as a whole. No outcome of an individual test.
- `need_check` says whether the fire-safety need Fable typed in the card agrees with NASA's text.
- `python tools/wsp/atlas/build_cards.py` merges the file into the card. Tell Fable when they exist.

### 4. Test labels, clusters, links, findings

- **Tests: what happened**, from the measurements and the documents, as a value of the closed
  phenomenon list with its evidence. Rule 2 applies to who may read them.
- **Clusters** by conditions (the crew atmospheres, flow regimes, classes of fuel and material), by
  what was measured, and by phenomenon. A cluster is a rule over labels, written down; a grouping
  found by similarity alone is a suggestion until it has a rule.
- **Links** as tables of edges, not a graph database: test to investigation, to file, to page; page
  to quantity; investigation to hardware, to need, to the crew atmospheres its tests cover; model
  parameter to the source that justifies it; model to the tests it was scored on, with the score; a
  table value to the sensor value that contradicts it.
- **Findings per cluster**, marked as interpretation.
- **Check:** a second agent labels a random twentieth again, blind, at least 100 labels of each
  kind; report the agreement by kind.
- **Output:** `runs/dev/data_astra/work/labels/` (`pages.jsonl`, `test_labels.jsonl`,
  `clusters.json`, `edges.jsonl`, `findings.jsonl`), each row with schema version, label, locator
  and the model that assigned it; described in `docs/wsp/measure/LABELS.md`. Where a closed list
  lacks a value, propose it in `docs/wsp/measure/VOCABULARY_PROPOSALS.md`.
- **BASS (PSI-26)** has no row in the test catalogue. Your source-row intake of it goes to your
  condition supplements; tell Fable, who adds it to the catalogue.

### 4b. Cross-check NASA's tables against NASA's own sensor files

Where a table and a sensor file of the same test say different things, record it. This needs the
text files only (about 12 GB for all 19 investigations, a few megabytes per test), no image and no
video, and a script does it, not a model: it reads two files and compares two numbers.

- **What to compare, wherever both sides exist:** tabulated flows against the commanded setpoints
  and the readbacks; the tabulated burn time and the way the test ended against the
  photomultipliers and radiometers; the script's planned stages against the timeline file; pressure
  and composition against the chamber readings; dates and test ids against file and folder names;
  and each table against itself (repeated rows, fractions that do not sum to one, mixed units).
- **Where it applies:** the six ACME experiments have sensor series for nearly every test; FLEX,
  FLEX-2, CFI, SAME-R and Saffire have text files. BASS, BASS-II and SPICE have none: there only
  tables and documents can be compared with each other.
- **Start from what is known:** 14 ACME train tests with a contradicted table value and 3 unsettled
  (`02_data_classification.md`, section 2.7); the table defects of the 15 new investigations
  (`docs/wsp/atlas/INVESTIGATIONS.md`); mistyped days in file and folder names
  (`docs/wsp/atlas/FILES.md`). Fable's endpoint method for the burner flames is
  `tools/wsp/ext/flame_design.py`; on CFI-G train it agrees with the table label on 148 of 153
  tests.
- **One record per discrepancy:** the test and the field; the table value with file and column;
  the sensor value with file, channel and the method that computed it; the size of the difference
  against the method's own error; and a status: confirmed by a second, independent reading; seen
  once; or unsettled. NASA's value is never replaced: the record stands beside it.
- **Say "discrepancy", not "error"**, until a second reading confirms it. Some will be ours: a
  wrong channel, a clock offset, a unit.
- **Then decide the cause, by these checks in this order**, and record which check decided it:
  1. *Is it ours?* Run the same reading method on the tests where table and sensor agree. A method
     that returns the table value there and differs on a few tests is sound; a difference by one
     factor on every test (the readback flow is 1.083 times the table flow across CFI-G train) is a
     calibration or a unit, not a table defect.
  2. *A third NASA source.* Setpoint, readback, the "as run" file, the script, the operator's
     comment, a published paper. Two sources that agree outvote the third only if they are
     independent: a file copied from the table counts with the table.
  3. *Physics that cannot be otherwise.* Bottle composition and diluent flow, the chamber's
     pressure rise against the gas injected, component flows against their total.
  4. *The shape of the difference.* A factor of 2 or 10, a value equal to the row above, a value
     equal to the planned one: each points to a known kind of transcription.
  5. *Is it the same quantity?* A tabulated burn time may include a cool flame the photomultiplier
     does not see. Then neither is wrong: keep both, each under its own name.
- **Verdicts:** table value wrong; table records the plan, not what ran; two definitions; our
  reading wrong (fix it, the record goes); unresolved (no third source: a question for the
  investigation's team, listed as such). Report "of N discrepancies, the cause of M is established".
- **Never use the test's outcome as the argument** ("it burned as at a low flow, so the flow was
  low"): that fits the data to the model. It is forbidden outside the train groups and is marked
  as weak evidence inside them.
- **Rule 2 applies.** A discrepancy in an outcome (a burn time, how a test ended) reveals the
  outcome: for tests outside the train groups it goes to the protected folder and is not read by an
  author. A discrepancy in a condition (a flow, a composition, a date) may be read by anyone.
- **Output:** `runs/dev/data_astra/work/labels/discrepancies.jsonl`, the script under
  `tools/wsp/astra/`, a count by investigation and by field in `docs/wsp/measure/DISCREPANCIES.md`,
  and an edge of the kind "table value contradicted by sensor value" in the link tables of step 4.
- The models keep reading the table. Feeding them the sensor value instead is a change of inputs
  and needs a new release.

### 5. Measure: turn images and video into numbers

- **What:** droplets (liquid and flame diameter, soot shell, drift); burner flames (radius or
  length and shape, onset and end, lift-off, smoke point, response to the electric field); solids
  (flame tip and base, length, spread rate, burn length, blow-off and quench); smoke and aerosol
  (the moments the instruments report).
- **Prove each method first** where NASA's own numbers exist: a droplet tracker against FLEX's
  processed series on FLEX train runs and against CFI's Igor series (reader:
  `tools/wsp/ext/igor_pxp.py`); a front tracker against `tools/wsp/bass_optical/tracks/train/`; a
  flame end from the photomultipliers against the table burn times of CFI-G train tests. Carry the
  measured error of the method as the uncertainty of every value.
- **A method is a script.** Blind agents measure the frames that prove it and, afterwards, a sample
  that audits it; the script measures everything else. Your pilot (18 of 1,468 frames of one run
  by blind operators) is the proof stage of one method, not the way to measure an archive.
- **Blind and doubled:** rule 3; a second measurer repeats a tenth of the tests unseen.
- **How many:** as many tests as the disk and the method allow, ordered by what a measurement
  unlocks: FLEX-2 (after the burn-to-mixture join), the CFI tests without an Igor series, Saffire,
  the BASS-II samples other than the fabric, SPICE, SLICE, the ACME experiments.
- **Output:** one file per test (series, uncertainty, frames used, method version, group id) and
  one method card per method.

### 6. Generalise the models until they cover everything

"Covered" means: for each of the 19 investigations a release accepts its tests instead of refusing
them, forecasts a quantity that matters for fire safety, states where it may be used, and has
either a score on groups it has not seen or the plain statement that PSI holds nothing that can
score it yet.

- **Fuel and material enter as typed physical properties**, never as a name: one shared, tested
  property block for every fuel, diluent and solid in the 19 tables, each value with its source and
  checked against a second source.
- **Few families, not nineteen models**: droplets; spherical burner flames; the burning-rate
  emulator; jet and coflow flames; solids in a flow; smoke and aerosol. Tie constants across
  families where the physics is the same, so that one wrong number shows in two experiments.
- **Where the target is one number per test**, one generic forward: a test's conditions in, the
  target with a typed spread out, a few typed units.
- **Gravity is an input**; a prediction for partial gravity is shown as one no PSI data can check.
- **A scenario without a test**: conditions alone (a crew atmosphere, a material, a flow), with a
  flag that says whether they lie inside the data the model was checked on.
- **Use what is known before a test starts**: the script's planned stages, commanded flows,
  hardware configuration. It was the largest gain of any generation. Argue the classification of
  every new input on page 02 before a model reads it.
- **Zero-shot first**: a clean-room candidate per family, scored on every group it has not seen;
  then data-informed candidates on train groups, committed before any validation look, each with
  its count of numbers by origin. A clean-room candidate's score on train groups is an
  out-of-sample score and costs no validation look; state the comparison before it is run and keep
  it to a few candidates.
- **Candidates** go under `configs/models/<family>/<spec>/<name>-astra/` with your own
  `author.model_id` and an exposure record; `review/README.md` gives the steps, `09_reproduce.md`
  the commands. Where your candidate and Fable's differ in kind, the equal-weight committee of the
  two is the honest joint statement and needs no validation to justify.
- **Register before scoring**: a subsection of `docs/wsp/PROTOCOL.md` section 11 per test, committed
  before its outcomes are read (11.1 is the pattern). The scoring chain first reproduces known
  train numbers (`tools/wsp/ext/transfer_acme.py` shows how).
- **Quantities the shared contracts lack** (aerosol moments, some heat-transfer outputs) stay as
  proposals in `docs/wsp/measure/ASTRA_CONTRACT_PROPOSALS.md`. No output is given another
  quantity's name to pass the registry.

### 7. Research and first principles

Keep the research files and the first-principles memo current as the work changes them: one
research file per family under `docs/wsp/research/`, the memo in
`docs/wsp/dossier/ASTRA_FIRST_PRINCIPLES_ALL_NASA.md`. The loop is the one of the section on how
research and findings work. Reading of sources at volume follows rule 8.

- **The combustion science of each family and each of the 19 investigations**, again, from the
  primary papers: what governs the quantity a model forecasts, in which regime, with which limits.
- **The method itself**, beyond Fable's reading in `docs/wsp/RESEARCH.md`: programs compiled into
  weights and weights written by construction; what interpretability and model editing say about
  which weight structures a reader can verify; symmetries and tying of weight spaces; models set by
  experts and prior elicitation; grey-box science (dimensional analysis, reduced mechanisms,
  solvers written as layers, property libraries); forecasting without fitting (proper scoring
  rules, committees, competing risks, calibration without data, extrapolation under shift); how a
  model nobody trained can be said to generalise; how to type thousands of numbers and be right.
- **The questions of the memo**: what a parameter is when the forward is code and the numbers are
  typed; where the generalisation of a hand-written model comes from and what destroys it; how a
  model grows by orders of magnitude in typed numbers without growing in freedom towards its train
  data; where honest uncertainty for an unseen condition comes from; what the largest defensible
  model is; where typing a number ends and letting a program choose it begins.

### 8. Record

Every source, measurement, decision, claim and exposure in its dossier page, in the same commit as
the work, with an honest status (rechecked, spot-checked, on trust); `ASTRA_ALL_NASA_PROGRESS.md`
kept true. Failures and negative results as plainly as successes. A refused candidate stays.

### 9. Hand over what the application shows

The application stores NASA's documents and sensor text and shows results beside them. A script
that chooses nothing writes, into `runs/dev/data_astra/export/`, with a manifest of hashes:

- the page cards and the search enrichment of every page, keyed by NASA's file name and page;
- the 19 narrative files;
- test labels, clusters with their rules, the edge tables and the findings;
- the discrepancy list with its verdicts;
- for each release: the tests it accepts, the quantity it forecasts, where it may be used, its
  scores with the registration they come from.

Every row keeps its locator, its kind ("NASA states", "interpretation", "model output",
"discrepancy") and the model that wrote it. Model outputs are never shown as measurements. The
export is read by the application's lane, not by an author: it holds labels of tests outside the
train groups. Tell Fable when it exists; he takes it to the application.

## The first instruction, what still stands

Part 1 of [ASTRA_PROMPT.md](ASTRA_PROMPT.md), checking the first author, continues beside the steps
above:

- Redo the research where it matters and try to break it: how documents were classified and what
  reached typed numbers (exposures X-01 to X-04, X-14 to X-19); how the validation look of the
  fifth generation was run (X-20 to X-22); how each NASA file and column was classified
  (`02_data_classification.md`, section 2.8 lists the calls to make again); the dataset results
  marked "on trust" (`03_datasets_and_measurements.md`); the physics dossiers under
  `docs/wsp/research/phys_*.md`, of which the methanol one was never re-derived.
- Check Fable's work of today the same way: the file and test catalogues, the frozen groups, the
  cards and the ranking, the Flame Design transfer test (rows M-25 to M-27, D-47, D-48, C-46 to
  C-48, X-23).
- The eight questions at the end of `review/README.md` are still open on Fable's side.

Parts 2, 3 and 4 of that instruction are steps 6 and 7 of this one.

## Done looks like

1. Everything of step 1 on disk, with the fetch log.
2. A page card with search enrichment for every page, and the measured agreement of Sonnet with a
   strong labeller, by kind of label.
3. A narrative file for each of the 19 investigations.
4. Test labels, clusters with their rules, edge tables, findings, with a second labeller's agreement.
4b. The list of discrepancies between NASA's tables and NASA's sensor files, each with both values,
   the method, the check that decided the cause and the verdict; and the count "of N, the cause of
   M is established".
5. Measuring methods with their proven error, and measurements for as many tests as the archive allows.
6. The property block, the forwards and the candidates; for each of the 19 investigations a release
   that accepts its tests, a registered test, and its result or the reason there is none.
7. The research files, the memo with the error budget of each family and the ranked list of
   changes, and what became of each entry.
8. Review findings filed for every claim you tested, with verdicts; the dossier current, so that a
   third reader could check you the way you checked Fable.
9. The export of step 9, with its manifest.

## Working beside Fable

- Fable keeps: the atlas (catalogues, groups, cards, ranking), the transfer tests of the frozen
  models, answers to your findings, the models' way into the application.
- Your places: `runs/dev/data_astra/`, `tools/wsp/astra/`, `docs/wsp/research/astra_*`,
  `docs/wsp/measure/`, `docs/wsp/atlas/narratives/`, `configs/models/**/<name>-astra/`, new forward
  files, your dossier pages, the review folder, new subsections of PROTOCOL section 11. Nothing
  else under `docs/wsp/atlas/`, `tools/wsp/atlas/`, `tools/wsp/ext/` or `runs/dev/data_all/`.
- One checkout, two authors: commit only your own files by explicit path; never `git add -A`; no
  formatter over the whole crate; no file committed that differs by line endings only.
- **Commit often.** Most of your work of today is on disk only. Fable reads and answers what is
  committed.
- **Build into your own target folder** (`CARGO_TARGET_DIR`). Today `target/` was deleted and
  rebuilt while a scoring run of Fable's was using its binaries.
- Gates before each commit: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`,
  `cargo test -p ignis-wsp -p ignis-wsp-dev -p ignis-core`.
