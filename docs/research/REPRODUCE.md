# Reproducing the IGNIS experiments

| | |
|---|---|
| **Scope** | Rebuild the hand-written model releases, rerun the development and final scores, rerun the published comparators, rerun the harness benchmark |

Commands run from the repository root, in `bash` (Git Bash on Windows). The scripts find the
repository root from their own path, so every `runs/...` path below is under this checkout.

## 0. Rules every command keeps

- **The test partition stays closed.** Do not pass `--final-test`, `--partition test` or
  `--partition all` to any development command, and never open `holdout/`. `ignis harness run`
  refuses a task file, variant or output under `holdout/`, a task set that declares `test`, and any
  scope that may read `test`. The final run of 15 November was the one run allowed on test data
  (section 3); its results are recorded, and its once-only markers stay in place.
- **Scripts never write a weight.** Every parameter of a release is typed by an author model, with
  its reason, in a committed edit file (`configs/models/**/edits*.json` or a `tools/wsp/*_edits.py`
  generator). Scripts assemble, score and report.
- **Every reveal is logged** in `runs/evaluation/reveal_log.jsonl` by the evaluator itself.

## 1. Prerequisites

| What | Version / source | Needed for |
|---|---|---|
| Rust toolchain | `rust-toolchain.toml` | `ignis` (CLI, evaluator, `ignis wsp`, `ignis harness run`) |
| Python | 3.13; standard library, plus `numpy` for the media and analysis scripts | `tools/wsp/**`, `benchmarks/harness/bench.py` |
| NASA PSI text files | `runs/dev/data_all/` (about 23 GB, 41,004 files, 19 investigations), built in section 1.2 | the seventh-generation laws, the atlas, the document lane, every transfer test |
| Model registry | `artifacts/models/`, rebuilt in section 2 | every score |
| Codex CLI signed in (`codex login`) and/or Claude Code (`claude /login`) | the versions recorded in each run's build manifest | the harness benchmark |

### 1.1 Build

```bash
cargo build --release -p ignis-cli            # target/release/ignis
cargo test -p ignis-wsp -p ignis-core         # the forwards and the registry
# wsp-dev must be supplied separately; its source is not included in this checkout:
```

Every script that runs `wsp-dev` takes `--wsp-dev "$WSP_DEV"` (default `target/release/wsp-dev`).

### 1.2 The NASA data

The project's own four investigations (FLEX PSI-69, CFI-G PSI-159, BASS-II PSI-25, BASS PSI-26) come
from the data build of [DATA_PIPELINE.md](../DATA_PIPELINE.md) and
[REPRODUCIBILITY.md §6](../REPRODUCIBILITY.md#6-data-build). The 19-investigation text archive is
fetched by script. Files are moved as opaque bytes, and image archives and video are left out:

```bash
python tools/wsp/atlas/fetch_listings.py     # PSI listings of the 19 investigations (metadata only)
python tools/wsp/atlas/classify_files.py     # file catalogue: runs/dev/data_all/work/atlas/files.jsonl
python tools/wsp/atlas/fetch_text.py         # every text, table and document file, latest version
python tools/wsp/atlas/normalize_tests.py    # test catalogue (conditions): work/atlas/tests.jsonl
python tools/wsp/atlas/build_cards.py        # one card per investigation + habitat ranking
python tools/wsp/atlas/check_atlas.py
```

**Expected:** `runs/dev/data_all/raw/MANIFEST.jsonl` with one row per selected file and its sha256.
`docs/wsp/atlas/FILES.md` and `INVESTIGATIONS.md` describe the result: 19 investigations, 41,004
files, 22 GB.

**Do not re-freeze the day groups.** The train, validation and test days of every investigation are
frozen in `docs/wsp/atlas/splits/<PSI>.json` and `data/splits/`. `tools/wsp/atlas/freeze_groups.py`
and `ignis data split freeze` refuse an existing id.

## 2. Rebuild every release from its typed edits

```bash
python tools/wsp/reassemble.py --ignis target/release/ignis
```

This rebuilds every lineage chain, in order, into `artifacts/models/`: FLEX droplets
(`flex*.wsp*`), ACME burner flames (`acme_normal*` / `acme_inverse*`, up to `wsp13`), the BASS
flame front, and the seventh-generation laws of `configs/models/law/` (`bre`, `spice`, `efield`,
`samer`, `same`, `daft`, `bass1` to `bass5`, `slice`, `slice2`, `cld`, `cld2`, `cld3`,
`efield2`, `efield3`). `--only a,b` limits it to some chains.

**Expected:** one line per candidate with its release id and `unchanged` against the id stored in
the chain's binding. A changed id means the edit file or the forward changed. Releases are
content-addressed, so the same edits always give the same id.

The served subset (the releases the product's tools and forecast lab use) is checked by:

```bash
python tools/wsp/verify_served.py --ignis target/release/ignis --parents artifacts
```

The edits of one release, as stored:

```bash
target/release/ignis wsp inspect --release sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654 --edits
```

## 3. Development scores and the final scores

### 3.1 Train feedback (any time)

```bash
"$WSP_DEV" feedback --release sha256:c8a90dabbed6af4185de94c3cd5ac27486c19eb52c697d86b028849a37fcb654 \
    --partition train --out runs/dev/review_fb_methanol.json
python tools/wsp/analysis/fb_summary.py runs/dev/review_fb_methanol.json
"$WSP_DEV" front-feedback --release sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c \
    --tracks tools/wsp/bass_optical/tracks/train --label review-train --out runs/dev/review_front_train.json
```

The table tools behind the typed numbers are described in
[`tools/wsp/analysis/README.md`](../../tools/wsp/analysis/README.md). Their `lib.py` returns train or
validation ids only.

### 3.2 Development tables (train and validation)

```bash
python tools/wsp/dev_eval.py --baselines runs/dev/baselines.json --wsp-dev "$WSP_DEV" --skip-validation \
    --out runs/dev/review_results_train.md                       # train only, any time
python tools/wsp/dev_eval.py --baselines runs/dev/baselines.json --wsp-dev "$WSP_DEV" --out docs/wsp/RESULTS.md
```

The validation path goes through the evaluator's sealed chain (`ignis infer forecast-cohort --seal`,
then `ignis eval compare`). It re-reads seals that already exist, so it repeats looks already
registered in PROTOCOL; it does not open new data. `baselines.json` maps each group to the fitted
reference releases (`ignis fit run … --execute`).

**Expected:** the tables of [RESULTS.md](../wsp/RESULTS.md), [RESULTS_GEN5.md](../wsp/RESULTS_GEN5.md)
and [RESULTS_GEN6.md](../wsp/RESULTS_GEN6.md). The carried releases on validation:

| Release | Validation |
|---|---|
| `flex_methanol.wsp2` D3 | CRPS 40.0 µm (56 cases) |
| `flex_heptane.wsp2` D2 | CRPS 46.1 µm (39 cases) |
| `acme_normal.wsp12` F2, with the planned stop | extinction 0.1473, termination 0.0798 (integrated Brier) |
| `acme_inverse.wsp12` F2, with the planned stop | 0.1349, 0.0480 |

The seventh-generation numbers on validation, per release, are in
`tools/wsp/final/expected_dev_gen7.json` (written by `expected_dev_gen7.py` from each lane's result
file).

### 3.3 The final run (spent once, rehearsable)

The final scores were run once on 15 November 2026 by the scoring agent (`claude-opus-5-5`), on
frozen release lists, aggregates only:

| Run | Script | Release list | Results |
|---|---|---|---|
| Project test partitions + test days of Flame Design, CFI, BASS, FLEX-2 | `tools/wsp/final/run_final.py` | `tools/wsp/final/releases_final_20261115.json` (sha256 `fe6ccec0…f594df686`) | [RESULTS_FINAL.md](../wsp/RESULTS_FINAL.md) |
| Seventh generation: the laws of nine investigations on their unread test days | `tools/wsp/final/run_final_gen7.py` | `tools/wsp/final/releases_final_gen7.json` | [RESULTS_FINAL_GEN7.md](../wsp/RESULTS_FINAL_GEN7.md) |
| s-Flame (PSI-23) test days 20142, 20170, 20176 | `tools/wsp/final/run_final_sflame.py` | `tools/wsp/final/releases_final_sflame.json` | [RESULTS_FINAL_SFLAME.md](../wsp/RESULTS_FINAL_SFLAME.md) |

Each step of final mode creates a marker in `runs/final/20261115/markers/` before it reads anything
and refuses to run when the marker exists, so a final score cannot be produced twice. What can be
rerun is the **rehearsal**: the identical path on the validation partition or validation days,
which must reproduce numbers already known.

The final-run scripts call the binaries in `target-merged/release/` (the build that carries every
forward of the branch). Build `ignis` there and put `wsp-dev` beside it. `preflight` prints their
sha256 against the ones the final run recorded (`ignis` `b03a35c8…`, `wsp-dev` `bd6055e0…`).

```bash
cargo build --release -p ignis-cli --target-dir target-merged
cp "$WSP_DEV" target-merged/release/
python tools/wsp/final/run_final.py preflight --releases tools/wsp/final/releases_final_20261115.json
python tools/wsp/final/run_final.py rehearse all
python -I tools/wsp/final/run_final_gen7.py rehearse
python tools/wsp/final/run_final_sflame.py rehearse
```

**Expected (rehearsal, step 1):** `flex_methanol.wsp2` D3 40.03 µm (56 cases), `flex_heptane.wsp2`
D2 46.08 µm (39 cases), `acme_normal.wsp12` F2 0.15367 / 0.09611 without the stop and 0.14729 /
0.07977 with it, `acme_inverse.wsp12` F2 0.12880 / 0.08804 and 0.13494 / 0.04796. Step 2 (Flame
Design) reproduces the decision table of `docs/wsp/feedback/flame_design_f2_result.md` with a largest
difference of 0.0. Every step and its expected numbers: [`tools/wsp/final/README.md`](../../tools/wsp/final/README.md).
Rehearsal results go to `runs/final/20261115/rehearsal/<step>.json`.

## 4. The published comparators

Each published model is rebuilt from its paper's equations and scored on the same cases as the
IGNIS release. Nothing is fitted to the held data and no IGNIS weight is read. Transcriptions of the
papers, with page provenance, are in `docs/wsp/research/*.json`.

| Comparator | Script | Investigation | Result page |
|---|---|---|---|
| Xi, Torero & Jahn 2021, droplet forecaster | `tools/wsp/final/posthoc_xi_comparator.py`, `posthoc_xi_window.py` | FLEX | [RESULTS_POINT_METRICS.md](../wsp/RESULTS_POINT_METRICS.md) |
| Point metrics (MAE, RMSE, nRMSE, MAPE) of the frozen FLEX releases | `tools/wsp/final/posthoc_point_metrics.py` | FLEX | [RESULTS_POINT_METRICS.md](../wsp/RESULTS_POINT_METRICS.md) |
| A published simulation curve traced from a figure | `tools/wsp/final/posthoc_published_curve.py` | FLEX | [RESULTS_POINT_METRICS.md](../wsp/RESULTS_POINT_METRICS.md) |
| Irace et al., PCI 39 (2023), radiative-extinction law | `tools/wsp/final/posthoc_irace_comparator.py`, `posthoc_irace_testdays.py`, `posthoc_irace_cfig_occurrence.py` | Flame Design, CFI-G | [RESULTS_PUBLISHED_ACME_BASS.md](../wsp/RESULTS_PUBLISHED_ACME_BASS.md) |
| Frolov et al., Fire 6 (2023); Snegirev, CNF 248 (2023) | `tools/wsp/final/posthoc_frolov_snegirev.py` | Flame Design | [RESULTS_PUBLISHED_ACME_BASS.md](../wsp/RESULTS_PUBLISHED_ACME_BASS.md) |
| Kim et al., PCI 39 (2023), CFI-G burner flames | `tools/wsp/final/posthoc_kim_comparator.py` | CFI-G | [RESULTS_PUBLISHED_ACME_BASS.md](../wsp/RESULTS_PUBLISHED_ACME_BASS.md) |
| Ferkul et al. 2012 / SIBAL spread rates (BASS-II fabric) | `tools/wsp/final/posthoc_bass_published.py` | BASS-II | [RESULTS_PUBLISHED_ACME_BASS.md](../wsp/RESULTS_PUBLISHED_ACME_BASS.md) |
| Lin & Faeth coflow law, Bhattacharjee et al. 2016 | the seventh-generation chains: `tools/wsp/ext/law_score.py`, with the cases of `spice_cases.py` and `bass1_cases.py` | SPICE, BASS plastics | [RESULTS_FINAL_GEN7.md](../wsp/RESULTS_FINAL_GEN7.md) |

```bash
python tools/wsp/final/posthoc_xi_comparator.py
python tools/wsp/final/posthoc_xi_window.py flex_methanol
python tools/wsp/final/posthoc_xi_window.py flex_heptane
python tools/wsp/final/posthoc_irace_testdays.py
python tools/wsp/final/posthoc_frolov_snegirev.py
python tools/wsp/final/posthoc_bass_published.py      # train and validation tracks
python tools/wsp/final/posthoc_kim_comparator.py      # train and validation records
```

The test-day comparators read the final run's own local caches (`runs/final/20261115/local/`) and
make no new reveal. They run where those caches exist, which is the machine of the final run. The
train and validation comparators run anywhere the data of section 1.2 is built.

**Expected** (test data, the comparator's error over IGNIS's): Frolov 2023 12×, Irace 2023 and
Snegirev 2023 8×, Xi, Torero and Jahn 2021 almost 7× (droplet diameter 151.7 against 22.4 µm),
the published BASS fit almost 4×, Lin and Faeth 2×, Bhattacharjee 2016 nearly 2×. Counts and
intervals are on the result pages.

## 5. The harness benchmark

What it measures and what it found: [HARNESS_BENCHMARK.md](HARNESS_BENCHMARK.md). The tooling is in
[`benchmarks/harness/`](../../benchmarks/harness/):

| File | What it is |
|---|---|
| `tasks.json` | the 22 questions, in the `ignis.harness.tasks.v0` format `ignis harness run` reads (no answers) |
| `references.json` | the same questions with kind, investigation and reference key points |
| `bench.py` | `raw`, `collect`, `judge`, `score` |
| `results/` | the results of 15 November 2026, unchanged |

**Prerequisites.** `target/release/ignis` (section 1.1). The NASA document corpus ingested, as for
`ignis serve` ([REPRODUCIBILITY.md §6](../REPRODUCIBILITY.md#6-data-build)), because IGNIS answers
from it. The Codex CLI signed in with `codex login` (OpenAI models), Claude Code signed in with
`claude /login` (Claude models and the Claude jurors). The prices used for OpenAI tokens are in
`bench.py` (`PRICES`) and `configs/pricing/llm_prices.toml`.

**1. The harness to test.** The benchmarked harness is commit `8b535b5`. To rerun that exact text:

```bash
mkdir -p runs/bench && git archive 8b535b5 harness | tar -x -C runs/bench && mv runs/bench/harness runs/bench/harness-8b535b5
```

Use `--variant harness` instead for the harness at HEAD.

**2. Inside IGNIS, one run per model.**

```bash
for m in gpt-6-astra gpt-6.1-sol gpt-6.1-terra gpt-6-sol gpt-6-luna; do
  target/release/ignis harness run --variant runs/bench/harness-8b535b5 --tasks benchmarks/harness/tasks.json \
      --out runs/bench/ignis-$m --agent-runtime codex --codex-model $m --task-timeout-s 900 --execute
done
target/release/ignis harness run --variant runs/bench/harness-8b535b5 --tasks benchmarks/harness/tasks.json \
    --out runs/bench/ignis-claude-fable-5-2 --agent-runtime claude --claude-model claude-fable-5-2 --execute
```

Without `--execute` the command prints the plan and spends nothing. Each run writes
`state/`, `traces/<task>.r1.json` (one `TraceBundle` per question, with tokens and cost) and
`summary.json`. Then collect the answers:

```bash
python benchmarks/harness/bench.py collect --label "IGNIS v2 · GPT-6 Luna" --model gpt-6-luna --run runs/bench/ignis-gpt-6-luna
# ... one line per run
```

**3. The same models alone**, plus any other raw model on the board:

```bash
python benchmarks/harness/bench.py raw --label "GPT-6 Luna" --runtime codex --model gpt-6-luna
python benchmarks/harness/bench.py raw --label "Claude Fable 5.2" --runtime claude --model claude-fable-5-2
# ... gpt-6-astra, gpt-6.1-sol, gpt-6.1-terra, gpt-6-sol, gpt-5.6-*, claude-opus-5-5, claude-sonnet-5-5, claude-haiku-5-5
```

**4. The committee and the table.**

```bash
python benchmarks/harness/bench.py judge --jurors claude:claude-opus-5-5 claude:claude-fable-5-2 codex:gpt-6.1-sol
python benchmarks/harness/bench.py score --out runs/bench/RESULTS.md
```

`judge` shuffles every answer with a fixed seed (`--seed`, default 20261115) and never shows a juror
which system wrote an answer. Every step appends to `benchmarks/harness/work/*.jsonl` and skips what
is already there, so an interrupted run resumes.

**Expected.** A table in the format of `results/RESULTS.md`: score, 95 % interval, score per kind,
precision, abstentions, cost per question, answers per dollar, latency, and the committee's
unanimity. On 15 November 2026: GPT-6 Luna 43 alone and 77 inside IGNIS, every model higher inside
IGNIS, precision 94 to 100 inside IGNIS, unanimity 87 %. A rerun is a new measurement. Model
versions, the corpus and the runtime change over time and LLM sampling varies, so scores move within
the intervals (about ±15 points on 22 questions).

## 6. The document lane (reading NASA's archive)

```bash
python tools/wsp/docs/prepare.py           # one input file per unique document, batches, the 200-page check sample
python tools/wsp/docs/brief.py b000        # the brief of one page-reader batch (run by Sonnet 5.5 subagents)
python tools/wsp/docs/validate.py          # every page back exactly once and valid; aggregates only
python tools/wsp/docs/gold_brief.py 5      # check-labeller briefs; then agreement.py
python tools/wsp/docs/agreement.py
python tools/wsp/docs/protect_brief.py     # the second protection pass
python tools/wsp/docs/visual_pass.py --sample      # Muse visual pass (needs META_API_KEY in a local env file)
python tools/wsp/docs/narratives.py        # the 19 investigation narratives (Muse)
python tools/wsp/docs/export.py            # runs/dev/data_docs/export/: what the application reads
python tools/wsp/docs/release.py           # what a model author may read
```

## 7. What is not in this repository

- **Data and run outputs.** `runs/`, `artifacts/models/` and the NASA files are built locally
  (sections 1.2 and 2). They are not in git.
- **NASA images and video.** The media measurers (`tools/wsp/media/`, `tools/wsp/bass_optical/`)
  fetch, measure and delete NASA media in batches. Only their measured tracks and series are kept
  (`tools/wsp/bass_optical/tracks/`).
- **The test vault.** `holdout/` and the evaluator's vault are not part of any copy.
- **The benchmark's run files of 15 November** (the raw answers, the juror labels and `results.json`
  with `index_other_families`) were not on this machine. The results are in
  `benchmarks/harness/results/`, and `bench.py` reruns the method.
