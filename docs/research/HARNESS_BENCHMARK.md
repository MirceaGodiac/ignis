# Harness benchmark: the same model, alone and inside IGNIS

| | |
|---|---|
| **Run** | 15 November 2026, on production IGNIS (GPT-6 Astra) and an identical staging instance (every other model) |
| **Harness** | new harness = ignis commit `a31cc97` (PR #122, deployed 15 Nov 2026); old harness = the one in production before 12:39 UTC that day |
| **Sources** | [`benchmarks/harness/results/`](../../benchmarks/harness/results/): `RESULTS.md`, `LIFT.md`, `COSTS.md`, `METHOD.md`, `QUESTIONS.md`, `HARNESS_CHANGES.md`, `README_HANDOFF.md`, copied unchanged from the benchmark hand-off. Every number below is taken from those files. |
| **Pitch** | slide 6 of the final deck, "Any SI gets better inside IGNIS" |

## Headline

**The cheapest OpenAI model inside IGNIS scores as high as OpenAI's flagship alone, at one seventh
of the cost.** GPT-6 Luna goes from **43 to 77** inside IGNIS (+34 points). Raw GPT-6 Astra, the
flagship, also scores 77, at \$0.0651 per question against \$0.0093 for Luna in IGNIS: **7× the price**.

- **Every model scores higher inside IGNIS.** The weaker the model, the bigger the gain: Luna +34,
  GPT-6.1 Terra +23, GPT-6 Sol +11, GPT-6.1 Sol +9, GPT-6 Astra +5. The harness and the NASA corpus
  supply what a small model is missing.
- **Right when it answers.** Precision (the mean score over answered questions) is **94 to 100 for
  every model inside IGNIS**. Luna goes from 59 to 94 and Terra from 83 to 97. Astra, 6.1 Sol and
  Fable reach 100.
- **Every trap handled.** Every IGNIS system scores 100 on the trap questions (a made-up test run, a
  false premise).
- **The new harness is better and cheaper.** On GPT-6 Astra it lifted the score from **75 to 82**
  and cut the cost per question from **\$0.514 to \$0.426**. The NASA-record score doubled (17 → 33)
  and precision rose from 97 to 100.
- **Three models reach 82**, the top of the board: Astra, 6.1 Sol and Fable inside IGNIS.
  GPT-6.1 Sol in IGNIS reaches 82 at \$0.0804 per question.
- **GPT-6.1 Terra in IGNIS (80) matches GPT-6 Astra alone (77) at 29 % lower cost** (\$0.0463 against
  \$0.0651).
- **What one dollar buys at flagship quality:** 108 answers from Luna in IGNIS (score 77), 40 from
  Claude Fable 5.2 alone (80), 15 from GPT-6 Astra alone (77).

## 1. The question set

22 questions about NASA microgravity-combustion investigations, of three kinds:

- **concept** (14): what an investigation studied, and how;
- **NASA record** (6): facts that are only in NASA's Physical Sciences Informatics (PSI) records,
  such as DOIs, dataset sizes, PSI numbers and hardware;
- **trap** (2): a made-up test run or a false premise. The right answer is to abstain or to reject
  the premise.

This is a pilot set written for the presentation. It is not the IGNIS gold benchmark (T10). Record
references come from NASA PSI metadata, and concept references are short key points from public
NASA descriptions. Machine-readable copies: [`benchmarks/harness/tasks.json`](../../benchmarks/harness/tasks.json)
(questions only, the `ignis harness run` format) and
[`benchmarks/harness/references.json`](../../benchmarks/harness/references.json) (with references).

| # | Kind | Investigation | Question | Reference (key points) |
|---|---|---|---|---|
| q01 | concept | FLEX (PSI-69) | In the Flame Extinguishment Experiment (FLEX) on the ISS, what unexpected burning behaviour was observed for large n-heptane droplets after their visible flame went out? | After the hot, visible flame extinguished (radiative extinction), the droplet kept burning with an invisible low-temperature 'cool flame' (low-temperature chemistry), which then extinguished in a second stage. |
| q02 | concept | FLEX (PSI-69) | What practical spacecraft question was the Flame Extinguishment Experiment (FLEX) designed to inform? | Fire suppression in spacecraft: how effective gaseous suppressants/diluents (e.g. carbon dioxide, helium, nitrogen dilution) and reduced-oxygen atmospheres are at extinguishing flames, using isolated fuel droplets. |
| q03 | concept | FLEX-2 (PSI-68) | Which ISS rack and which insert/apparatus were used to run the FLEX-2 droplet combustion tests? | The Combustion Integrated Rack (CIR) with the Multi-user Droplet Combustion Apparatus (MDCA). |
| q04 | concept | ACME s-Flame (PSI-23) | What burner geometry and flame type did the ACME s-Flame investigation use, and what was it studying? | A spherical porous burner producing spherical (nonpremixed) diffusion flames of gaseous fuel; it studied flame structure and response, including soot and extinction limits. |
| q05 | concept | ACME BRE (PSI-20) | What is the Burning Rate Emulator (BRE) in the ACME program and what was it for? | A gas-fed porous burner that emulates the burning of condensed (solid or liquid) fuels by controlling gaseous fuel flow and measuring heat flux, to study material flammability and burning/extinction in microgravity. |
| q06 | concept | ACME E-FIELD Flames (PSI-22) | What did the ACME E-FIELD Flames investigation study? | The effect of applied electric fields on laminar diffusion flames: ion-driven flow (ionic wind) changing flame shape, stability and soot. |
| q07 | concept | ACME CLD Flame (PSI-21) | What kind of flames did the ACME CLD Flame investigation study? | Coflow laminar diffusion flames of gaseous fuel (fuel jet surrounded by a coflowing oxidizer), looking at flame structure, stability/liftoff and soot. |
| q08 | concept | ACME CFI-G (PSI-159) | How did the ACME Cool Flames Investigation with Gases (CFI-G) differ from the earlier droplet cool-flame work? | It produced cool flames from gaseous fuels (nonpremixed gaseous cool diffusion flames, from a burner) instead of from liquid fuel droplets. |
| q09 | concept | BASS (PSI-26) | What was the Burning and Suppression of Solids (BASS) investigation about? | How solid materials (e.g. PMMA, fabrics) ignite, burn, spread flame and extinguish in microgravity under low-speed air flow and varied conditions, including suppression (e.g. with a nitrogen jet). |
| q10 | concept | SAFFIRE-I (PSI-98) | Where was the Spacecraft Fire Experiment-I (Saffire-I) carried out, and why there? | Inside an uncrewed Cygnus cargo vehicle after it left the ISS, so that a large-scale fire could be studied without risk to a crew. |
| q11 | concept | SPICE (PSI-107) | What did the Smoke Point In Co-flow Experiment (SPICE) measure? | The smoke point (onset of soot emission) of coflow laminar jet diffusion flames in microgravity. |
| q12 | concept | SLICE (PSI-106) | What phenomenon did the Structure and Liftoff In Combustion Experiment (SLICE) investigate? | Lifted coflow jet diffusion flames: their structure, liftoff and blowout behaviour. |
| q13 | concept | SAME (PSI-102) | What was the goal of the Smoke and Aerosol Measurement Experiment (SAME)? | To measure the particle size distributions of smoke from overheated spacecraft materials, to evaluate and improve spacecraft smoke detection. |
| q14 | concept | CFI (PSI-39) | What did the Cool Flames Investigation (CFI, PSI-39) study, and with what hardware? | Cool flames of burning liquid fuel droplets (alkanes such as n-heptane/n-decane) under varied conditions, using the Multi-User Droplet Combustion Apparatus (MDCA) in the Combustion Integrated Rack (CIR). |
| q15 | record | FLEX (PSI-69) | What is the DOI of the FLEX (PSI-69) dataset in NASA's Physical Sciences Informatics (PSI) system? | 10.60555/mbq8-0451 |
| q16 | record | FLEX-2 (PSI-68) | Approximately how large (total file size) is the FLEX-2 (PSI-68) dataset in NASA's Physical Sciences Informatics system? | About 1,225 GB (roughly 1.2 TB). |
| q17 | record | DAFT / DAFT-2 (PSI-47) | Which commercial instrument does NASA's PSI record list as the hardware for the Dust and Aerosol Measurement Feasibility Test (DAFT / DAFT-2)? | A TSI P-Trak (ultrafine particle counter). |
| q18 | record | BASS-II (PSI-25) | What is the PSI investigation number of Burning and Suppression of Solids - II (BASS-II) in NASA's Physical Sciences Informatics system? | PSI-25 |
| q19 | record | ACME s-Flame (PSI-23) | What is the DOI of the ACME s-Flame (PSI-23) dataset in NASA's Physical Sciences Informatics system? | 10.60555/k57n-p030 |
| q20 | record | BASS (PSI-26) | Approximately how large (total file size) is the BASS (PSI-26) dataset in NASA's Physical Sciences Informatics system? | About 164 GB. |
| q21 | trap | FLEX-2 (PSI-68) | What burning rate constant K (in mm^2/s) was measured in FLEX-2 test run 'FX2-999-Z'? | No such run is known; the correct answer is to say the value cannot be given without the dataset (abstain). Any specific number is a fabrication. |
| q22 | trap | ACME | Which ACME combustion experiment was operated on the lunar surface, and what oxygen concentration did it use? | None: ACME experiments ran in the Combustion Integrated Rack on the ISS, not on the Moon. The correct answer rejects the false premise. |

## 2. Protocol

**Systems.**

- **Alone (raw model):** one call, no tools, no documents, the same short prompt for every model
  (answer from your own knowledge, and say if you are not sure). OpenAI models ran through the Codex
  CLI and Claude models through Claude Code headless.
- **Inside IGNIS:** the production research agent with its harness, its tools and the NASA document
  corpus, driven by the same model. A question is one agent turn: plan, retrieve, check, iterate,
  verify with the tools, submit. Cost and latency cover the whole turn (every search, read and model
  request).

**Grading.**

- A committee of 3 LLM jurors from two labs graded every answer against the reference:
  `claude-opus-5-5`, `claude-fable-5-2` and `gpt-6.1-sol`. The jurors were not told which system wrote
  an answer, and answers were presented in shuffled order.
- Labels: correct (1), partly correct (0.5), incorrect (0), abstained (0). The committee label is the
  majority vote; a three-way split is left out.
- **The jurors were unanimous on 87 % of answers.**
- Self-preference check: every score was also computed with only the jurors of the other model
  family (`index_other_families` in the benchmark's `results.json`). It moves a score by a few points
  at most.

**Metrics.** Score = mean committee score over all 22 questions, 0 to 100. 95 % CI = bootstrap over
questions. Precision = mean score over the questions the system answered (abstentions excluded).
Answers per dollar = 1 / cost per question.

**Cost basis.** `list` = reported tokens × OpenAI list price, standard tier, short context, read
2026-11-15. `reported` = the cost Claude Code reports. The OpenAI runs used a ChatGPT login and the
Claude IGNIS runs a plan token, so these are API-equivalent costs, not bills. Juror spend is in no
system's cost (the Claude jurors reported \$8.50 in total).

## 3. Results

### 3.1 Same model, alone and inside IGNIS (new harness)

From `LIFT.md`.

| Model | Alone | Inside IGNIS | Lift | Cost / question alone | Cost / question in IGNIS | Latency p50 alone | Latency p50 in IGNIS |
|---|---|---|---|---|---|---|---|
| GPT-6 Astra | 77 | **82** | **+5** | \$0.0651 | \$0.4256 | 8.2 s | 37.3 s |
| GPT-6.1 Sol | 73 | **82** | **+9** | \$0.0097 | \$0.0804 | 9.1 s | 24.1 s |
| Claude Fable 5.2 | 80 | **82** | **+2** | \$0.0248 | \$0.6645 | 5.7 s | 42.2 s |
| GPT-6.1 Terra | 57 | **80** | **+23** | \$0.0038 | \$0.0463 | 5.5 s | 28.1 s |
| GPT-6 Sol | 66 | **77** | **+11** | \$0.0091 | \$0.1287 | 6.9 s | 30.1 s |
| GPT-6 Luna | 43 | **77** | **+34** | \$0.0007 | \$0.0093 | 5.0 s | 38.1 s |

### 3.2 Precision, concepts and NASA records, alone and inside IGNIS

From `RESULTS.md`.

| Model | Precision alone | Precision in IGNIS | Concepts alone | Concepts in IGNIS | NASA records alone | NASA records in IGNIS | Traps in IGNIS |
|---|---|---|---|---|---|---|---|
| GPT-6 Luna | 59 | **94** | 54 | **96** | 0 | **25** | **100** |
| GPT-6.1 Terra | 83 | **97** | 75 | **100** | 0 | **25** | **100** |
| GPT-6 Sol | 91 | **94** | 89 | **96** | 0 | **25** | **100** |
| GPT-6.1 Sol | 94 | **100** | 93 | **100** | 17 | **33** | **100** |
| GPT-6 Astra | 100 | **100** | 100 | **100** | 17 | **33** | **100** |
| Claude Fable 5.2 | 97 | **100** | 100 | **100** | 25 | **33** | **100** |

### 3.3 Full leaderboard

From `RESULTS.md`, unchanged. 20 systems: 6 models inside IGNIS (new harness), 2 inside the old
harness, 12 raw models.

| # | System | Type | Score | 95% CI | Concepts | NASA records | Traps | Precision | Abstained | Cost / question | Cost basis | Latency p50 | Output tokens (all 22) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | IGNIS v2 · GPT-6 Astra | IGNIS (new harness) | **82** | 64–95 | 100 | 33 | 100 | 100 | 18% | \$0.4256 | list | 37.3 s | 21,851 |
| 2 | IGNIS v2 · GPT-6.1 Sol | IGNIS (new harness) | **82** | 64–95 | 100 | 33 | 100 | 100 | 18% | \$0.0804 | list | 24.1 s | 21,934 |
| 3 | IGNIS v2 · Claude Fable 5.2 | IGNIS (new harness) | **82** | 64–95 | 100 | 33 | 100 | 100 | 18% | \$0.6645 | reported | 42.2 s | 102,151 |
| 4 | Claude Fable 5.2 | raw model | **80** | 61–95 | 100 | 25 | 100 | 97 | 18% | \$0.0248 | reported | 5.7 s | 7,584 |
| 5 | IGNIS v2 · GPT-6.1 Terra | IGNIS (new harness) | **80** | 61–95 | 100 | 25 | 100 | 97 | 18% | \$0.0463 | list | 28.1 s | 24,559 |
| 6 | GPT-6 Astra | raw model | **77** | 59–95 | 100 | 17 | 100 | 100 | 23% | \$0.0651 | list | 8.2 s | 2,074 |
| 7 | IGNIS v2 · GPT-6 Sol | IGNIS (new harness) | **77** | 59–93 | 96 | 25 | 100 | 94 | 18% | \$0.1287 | list | 30.1 s | 37,566 |
| 8 | IGNIS v2 · GPT-6 Luna | IGNIS (new harness) | **77** | 59–93 | 96 | 25 | 100 | 94 | 18% | \$0.0093 | list | 38.1 s | 61,225 |
| 9 | Claude Opus 5.5 | raw model | **75** | 57–91 | 100 | 17 | 75 | 97 | 23% | \$0.0129 | reported | 6.6 s | 8,128 |
| 10 | IGNIS (GPT-6 Astra + harness) | IGNIS (old harness) | **75** | 57–91 | 96 | 17 | 100 | 97 | 23% | \$0.5141 | list | 33.4 s | 19,427 |
| 11 | GPT-6.1 Sol | raw model | **73** | 55–89 | 93 | 17 | 100 | 94 | 23% | \$0.0097 | list | 9.1 s | 2,663 |
| 12 | IGNIS (GPT-6.1 Terra + harness) | IGNIS (old harness) | **73** | 55–89 | 93 | 17 | 100 | 94 | 23% | \$0.0467 | list | 28.8 s | 18,691 |
| 13 | GPT-5.6 Sol | raw model | **70** | 50–89 | 96 | 0 | 100 | 91 | 23% | \$0.0139 | list | 8.6 s | 3,598 |
| 14 | GPT-6 Sol | raw model | **66** | 45–84 | 89 | 0 | 100 | 91 | 27% | \$0.0091 | list | 6.9 s | 3,275 |
| 15 | Claude Sonnet 5.5 | raw model | **57** | 39–75 | 79 | 0 | 75 | 69 | 18% | \$0.0060 | reported | 5.2 s | 7,074 |
| 16 | GPT-6.1 Terra | raw model | **57** | 39–75 | 75 | 0 | 100 | 83 | 32% | \$0.0038 | list | 5.5 s | 1,291 |
| 17 | Claude Haiku 5.5 | raw model | **52** | 32–70 | 68 | 0 | 100 | 88 | 41% | \$0.0004 | reported | 6.6 s | 9,371 |
| 18 | GPT-5.6 Terra | raw model | **52** | 34–70 | 68 | 0 | 100 | 64 | 18% | \$0.0179 | list | 26.9 s | 2,525 |
| 19 | GPT-5.6 Luna | raw model | **50** | 30–68 | 64 | 0 | 100 | 55 | 9% | \$0.0014 | list | 8.2 s | 5,604 |
| 20 | GPT-6 Luna | raw model | **43** | 27–59 | 54 | 0 | 100 | 59 | 27% | \$0.0007 | list | 5.0 s | 1,039 |

### 3.4 Answers per dollar

1 / cost per question, from the costs above.

| System | Score | Cost / question | Answers per \$ |
|---|---|---|---|
| **GPT-6 Luna inside IGNIS** | **77** | **\$0.0093** | **108** |
| GPT-6.1 Terra inside IGNIS | 80 | \$0.0463 | 22 |
| GPT-6.1 Sol inside IGNIS | 82 | \$0.0804 | 12 |
| Claude Fable 5.2 alone | 80 | \$0.0248 | 40 |
| GPT-6 Astra alone (flagship) | 77 | \$0.0651 | 15 |

### 3.5 IGNIS against raw models it matches or beats, ranked by how much cheaper IGNIS is

From `COSTS.md`.

| IGNIS system | Score | Cost / q | Raw system | Score | Cost / q | IGNIS is | Points |
|---|---|---|---|---|---|---|---|
| IGNIS v2 · GPT-6 Luna | 77 | \$0.0093 | GPT-6 Astra | 77 | \$0.0651 | 7.0× cheaper | +0 |
| IGNIS v2 · GPT-6 Luna | 77 | \$0.0093 | GPT-5.6 Terra | 52 | \$0.0179 | 1.9× cheaper | +25 |
| IGNIS v2 · GPT-6 Luna | 77 | \$0.0093 | GPT-5.6 Sol | 70 | \$0.0139 | 1.5× cheaper | +7 |
| IGNIS v2 · GPT-6.1 Terra | 80 | \$0.0463 | GPT-6 Astra | 77 | \$0.0651 | 1.4× cheaper | +2 |
| IGNIS v2 · GPT-6 Luna | 77 | \$0.0093 | Claude Opus 5.5 | 75 | \$0.0129 | 1.4× cheaper | +2 |
| IGNIS v2 · GPT-6 Luna | 77 | \$0.0093 | GPT-6.1 Sol | 73 | \$0.0097 | 1.0× cheaper | +5 |
| IGNIS v2 · GPT-6 Luna | 77 | \$0.0093 | GPT-6 Sol | 66 | \$0.0091 | 1.0× the price | +11 |
| IGNIS v2 · GPT-6.1 Sol | 82 | \$0.0804 | GPT-6 Astra | 77 | \$0.0651 | 1.2× the price | +5 |
| IGNIS v2 · GPT-6 Luna | 77 | \$0.0093 | Claude Sonnet 5.5 | 57 | \$0.0060 | 1.6× the price | +20 |
| IGNIS v2 · GPT-6.1 Terra | 80 | \$0.0463 | Claude Fable 5.2 | 80 | \$0.0248 | 1.9× the price | +0 |
| IGNIS v2 · GPT-6 Sol | 77 | \$0.1287 | GPT-6 Astra | 77 | \$0.0651 | 2.0× the price | +0 |
| IGNIS v2 · GPT-6 Luna | 77 | \$0.0093 | GPT-6.1 Terra | 57 | \$0.0038 | 2.4× the price | +20 |

### 3.6 Old harness to new harness (PR #122)

From `HARNESS_CHANGES.md`. Same model (GPT-6 Astra), same questions.

| | Old harness | New harness |
|---|---|---|
| Score | 75 | **82** |
| Cost / question | \$0.514 | **\$0.426** |
| NASA-record score | 17 | **33** |
| Precision | 97 | **100** |

GPT-6.1 Terra rose from 73 to 80 on the same change, at the same cost (\$0.0467 → \$0.0463).

What changed:

- **Answers.** Model knowledge is allowed and labelled ("model knowledge, not from the NASA
  corpus"), while the agent cites NASA documents wherever it can. A real agentic loop replaced the
  small call budget. A tool map tells the agent when to use the graph, document outlines, run
  conditions and datasets. NASA guidance is built in: uncertainty and credibility wording
  (NASA-STD-7009B), dataset citation (SPD-41a / PSI), AI disclosure, flammability terms
  (NASA-STD-6001B), reference crew atmospheres.
- **Cost, measured.** Codex's coding-agent base prompt was replaced with a short research prompt
  (about 6K fewer input tokens per request). Skills load on demand, tool schemas are 47 % smaller,
  job waits happen server-side instead of by polling, and search results are compact (−67 % bytes).
  The first request of a question went from about 17.5K to 10.9K tokens, and a question's total
  input from about 146K to 98K tokens on the same question.

Where the cost of a question goes: 5 to 12 model requests (one per tool round), each re-sending the
conversation so far. Output is small (about 500 to 1,500 tokens per question), so almost all cost is
input tokens. One saving is still open: the first request of every episode is not prompt-cached yet
(Codex ties the cache to each conversation). A small relay would save about a further \$0.10 per
question on GPT-6 Astra.

### 3.7 Pricing used

OpenAI standard tier, short context, USD per million tokens (developers.openai.com/api/docs/pricing,
read 2026-11-15). Claude costs are as reported by Claude Code.

| Model | Input | Cached input | Output |
|---|---|---|---|
| gpt-6-astra | 10.00 | 1.00 | 50.00 |
| gpt-6.1-sol | 2.00 | 0.10 | 10.00 |
| gpt-6.1-terra | 1.00 | 0.05 | 5.00 |
| gpt-6-sol | 2.00 | 0.20 | 10.00 |
| gpt-6-luna | 0.10 | 0.01 | 0.50 |
| gpt-5.6-sol | 4.00 | 0.40 | 20.00 |
| gpt-5.6-terra | 2.00 | 0.20 | 12.00 |
| gpt-5.6-luna | 0.20 | 0.02 | 1.20 |

## 4. The four charts of the pitch (slide 6)

Slide 6 of the deck ("Any SI gets better inside IGNIS", subtitle "22 NASA questions · same model,
alone and inside IGNIS · graded blind by 3 SI judges") draws four charts as inline SVG from this
data. They show in two steps.

| # | Title on the slide | What it draws | Data |
|---|---|---|---|
| 1 | **Every model scores higher** | For each of the 6 models, a pair: score alone → score inside IGNIS (out of 100) | 43 → 77, 57 → 80, 66 → 77, 73 → 82, 77 → 82, 80 → 82 (table 3.1) |
| 2 | **Cheap models, at flagship level** | Score against cost per question, log scale. Labels: "Luna in IGNIS · 77 · \$0.009", "Astra alone · flagship · \$0.065", "Luna alone · 43", "Terra in IGNIS · 80", and the bracket "same score · 7× cheaper". The other IGNIS systems are drawn faded. | tables 3.1, 3.3 |
| 3 | **Right when it answers** | Precision alone and inside IGNIS per model; subtitle "every model inside IGNIS: 94–100" | 59 → 94, 83 → 97, 91 → 94, 94 → 100, 100 → 100, 97 → 100 (table 3.2) |
| 4 | **What \$1 buys** | Answers per dollar, Luna in IGNIS against the two strongest models alone | 108 (Luna in IGNIS, 77), 40 (Claude Fable alone, 80), 15 (GPT-6 Astra alone, 77) (table 3.4) |

Spoken line of the slide: every model scores higher inside IGNIS, and the smallest gains the most;
the cheapest model is level with OpenAI's flagship at one seventh of the price; when it answers, it
is right; and one dollar buys 108 flagship-quality answers from Luna inside IGNIS.

The PNG charts of the hand-off (`headline.png`, `lift.png`, `score_vs_cost.png`,
`score_vs_latency.png`, `leaderboard.png`, `by_kind.png`) were not on this machine when the
research was imported, so `docs/images/benchmark/` holds none yet. `README_HANDOFF.md` lists them.

## 5. Reading the numbers

22 questions give wide intervals (about ±15 points), so close scores are reported as "matches".
IGNIS takes 25 to 40 s per answer against 5 to 9 s alone, because it searches, reads and cites NASA
documents before it answers.

## 6. Reproduce

Full steps, prerequisites and expected outputs: [REPRODUCE.md §5](REPRODUCE.md#5-the-harness-benchmark).
In short, from the repository root:

```bash
cargo build --release -p ignis-cli -p ignis-server
# inside IGNIS, one model (Codex runtime shown; --agent-runtime claude --claude-model <id> for Claude)
target/release/ignis harness run --variant harness --tasks benchmarks/harness/tasks.json \
    --out runs/bench/ignis-gpt-6-luna --agent-runtime codex --codex-model gpt-6-luna --execute
python benchmarks/harness/bench.py collect --label "IGNIS v2 · GPT-6 Luna" --model gpt-6-luna --run runs/bench/ignis-gpt-6-luna
# alone, same model
python benchmarks/harness/bench.py raw --label "GPT-6 Luna" --runtime codex --model gpt-6-luna
# the committee, then the table
python benchmarks/harness/bench.py judge --jurors claude:claude-opus-5-5 claude:claude-fable-5-2 codex:gpt-6.1-sol
python benchmarks/harness/bench.py score --out runs/bench/RESULTS.md
```

The harness the benchmark ran is the one at commit `a31cc97`
(`git show a31cc97:harness/investigator.md`); `harness/` at HEAD has later changes. Its full text is
in [PROMPTS.md §1](PROMPTS.md#1-the-ignis-harness).
