# 9. Reproducing the work

Commands run from the repository root (`bash`; on Windows, Git Bash). Nothing here reads the test
partition; do not add `--final-test`, `--partition test` or `--partition all` to any of them.

## 9.1 Build and tests

```bash
cargo build --release -p ignis-cli -p ignis-wsp-dev
```

```bash
cargo test -p ignis-wsp -p ignis-wsp-dev -p ignis-core
```

## 9.2 Releases from the committed edits

Rebuilds every chain in lineage order into the local registry (`artifacts/models/`) and prints, per
candidate, whether the weight hash is unchanged.

```bash
python tools/wsp/reassemble.py
```

The edits of one release, as stored:

```bash
target/release/ignis wsp inspect --release sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce --edits
```

## 9.3 Train feedback (per case; train only)

```bash
target/release/wsp-dev feedback --release sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce --partition train --out runs/dev/review_fb_methanol.json
```

```bash
python tools/wsp/analysis/fb_summary.py runs/dev/review_fb_methanol.json
```

Session-grouped comparison of two releases in one feedback file (the droplet selection rule):

```bash
python tools/wsp/analysis/session_compare.py runs/dev/fb_train_flex_d3.json runs/dev/analysis/flex_methanol_profile.json 0 1
```

BASS, on the four train tracks:

```bash
target/release/wsp-dev front-feedback --release sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c --tracks tools/wsp/bass_optical/tracks/train --label review-train --out runs/dev/review_front_train.json
```

`front-feedback` on the sealed validation folder is a registered look, not a review step.

## 9.4 Train tables behind the typed numbers

The tools are described in `tools/wsp/analysis/README.md`; `lib.py` returns train or validation ids
only.

```bash
python tools/wsp/analysis/flex_sheets.py flex runs/dev/analysis/flex_methanol_sheets.md
```

```bash
python tools/wsp/analysis/flex_profile.py flex runs/dev/analysis/flex_methanol_profile.json
```

```bash
python tools/wsp/analysis/acme_table.py acme_inverse runs/dev/analysis/acme_inverse_train.md
```

```bash
python tools/wsp/analysis/neardup.py
```

The heptane split is the second FLEX split: use `flex_heptane` in place of `flex`.

## 9.5 The delegated measurements (M-06 to M-08)

Each agent left its scripts beside its tables. They were written to run from the repository root.

| Measurement | Script | Needs |
|---|---|---|
| Shift map (inputs only) | `runs/dev/research5/shift/shift.py` | the shared records |
| FLEX flame traces | `runs/dev/research5/flex_flame/` `trainmap.py`, `fetch.py`, `measure.py`, `forecast.py`, `summarize.py` | 203 flame files of train runs (4 MB) in `runs/dev/nasa/PSI-69/flame/`; `fetch.py` downloads them from the listing |
| ACME raw channels | `runs/dev/research5/acme_raw/scripts/` `rawlib.py`, `analyze.py` (argument `23.5`, the assumed cold junction in °C), `lawcheck.py`, `timeline.py`, `summary.py` | 744 files of train tests (712 MB) in `runs/dev/nasa/PSI-159/` |

These folders are local (see [registers/local_folders.md](registers/local_folders.md)). A reviewer
without them reruns the scripts after fetching; the listing with every `stable_url` is
`data/manifests/PSI-<n>.listing.jsonl`. Use a generic client name and fetch files of train tests
only: build the file list from `lib.partition_ids(family, "train")` first.

The author's check of the programmed stops (M-10), on the tables of the raw-channel measurement:

```bash
python - <<'EOF'
import csv
rows = list(csv.DictReader(open('runs/dev/research5/acme_raw/tp_timelines.csv', encoding='utf-8')))
tb = {r['test']: r for r in csv.DictReader(open('runs/dev/research5/acme_raw/tests_burner.csv', encoding='utf-8'))}
f = lambda x: float(x) if x not in (None, '') else None
K = ['ar_stabilizationTime1Sec', 'ar_stabilizationTime2Sec', 'ar_ignitionToTestRampTimeSec', 'ar_constantFlowTime1Sec',
     'ar_testFlowRamp1To2TimeSec', 'ar_constantFlowTime2Sec', 'ar_testFlowRamp2To3TimeSec', 'ar_constantFlowTime3Sec']
early = [r for r in rows if 'extinction' in r['stop_class'].lower() or 'limit' in r['stop_class'].lower()]
d = sorted(f(r['programmed_off_from_onset']) - f(r['table_burn_time']) for r in early
           if f(r['programmed_off_from_onset']) is not None and f(r['table_burn_time']) is not None)
print('ended early:', len(d), 'programmed minus burn time: min', d[0], 'median', d[len(d) // 2])
s = sorted(f(r['actual_off_from_onset']) - sum(f(r[k]) or 0.0 for k in K) for r in rows
           if r['stop_class'].startswith('scripted') and f(r['actual_off_from_onset']) is not None)
print('scripted ends:', len(s), 'stop after onset minus stage sum: min', s[0], 'median', s[len(s) // 2], 'max', s[-1])
EOF
```

Expected: `ended early: 99 … min 10.2 median 79.4` and `scripted ends: 29 … min -1.3 median -0.4 max 1.7`
(rounded).

The author's recount of which input separates the inverse train outcomes (M-17) is `tools/wsp/analysis/inverse_separation.py`:

```bash
python tools/wsp/analysis/inverse_separation.py
```

## 9.6 The private data root (fifth generation)

`runs/dev/data_ext/` is a copy of `data/` with more files fetched and ingested by the data lane's
own commands, selected through the environment variable `IGNIS_DATA`:

```bash
cp -r data runs/dev/data_ext
```

```bash
IGNIS_DATA=runs/dev/data_ext target/release/ignis sources fetch PSI-69 --category "Raw Data" --pattern '<regex of the LLUVData and ColorCamData files of train and validation runs>' --execute
```

```bash
IGNIS_DATA=runs/dev/data_ext target/release/ignis data ingest flex
```

```bash
python tools/wsp/ext/acme_script.py --root runs/dev/data_ext --fetch
```

**Gap:** the exact `--pattern` used for the 283 flame files is not recorded in a script. It was
built from the file-number aliases of the train and validation run ids. Before the fifth-generation
candidates are registered this step will be a tool under `tools/wsp/ext/` that builds the list from
`lib.partition_ids` and refuses anything else. **This was not done before the fifth-generation look;
the gap stands.** The same care applies as everywhere: the root then holds validation series, which
nobody who writes weights opens.

The evaluator scores a held-out forecast against continuations it keeps in a write-once vault
(`holdout/`, or the folder named by `IGNIS_HOLDOUT`). The shared vault holds the continuations of the
shared records and refuses those of the private copy where a run's record differs (the droplet runs,
which gained camera series). For the fifth-generation look the private copy's validation
continuations were deposited into a vault of their own; nobody opens either vault:

```bash
IGNIS_DATA=runs/dev/data_ext IGNIS_HOLDOUT=runs/dev/holdout_ext target/release/ignis eval deposit-cohort --cohort <cohort_id of configs/models/flex/wsp5/methanol/binding.json> --split <its split_id> --partition validation
```

Train feedback on the private root:

```bash
IGNIS_DATA=runs/dev/data_ext target/release/wsp-dev feedback --release sha256:c7d127b37fedd52896860c6964fd853ed463601c94fdb4054537b0b2f8f769e6 --partition train --out runs/dev/ext/review_fb_methanol_f1.json
```

## 9.7 Development scores and the page

These score every release of a group on **validation** as well as train. Running them is a
validation look: do it only for a look registered in PROTOCOL, and keep out of the group any
candidate that must not be looked at. For a review, use the train-only form.

```bash
python tools/wsp/dev_eval.py --baselines runs/dev/baselines.json --skip-validation --out runs/dev/review_results_train.md
```

```bash
python tools/wsp/overview.py
```

The tables of [RESULTS_GEN5.md](../RESULTS_GEN5.md) (a look already taken; rerunning it reads the
same seals and repeats the look). Droplets, on the private copy and its vault:

```bash
IGNIS_DATA=runs/dev/data_ext IGNIS_HOLDOUT=runs/dev/holdout_ext python tools/wsp/dev_eval.py --baselines runs/dev/baselines.json --only flex_gen5,flex_heptane_gen5 --out runs/dev/RESULTS_gen5_flex.md
```

Burner flames. The releases are sealed on both data roots, so the evaluator's comparison holds every
run twice, once per prefix; the page `dev_eval.py` writes for these groups is that doubled table and
is not to be quoted. The split:

```bash
python tools/wsp/dev_eval.py --baselines runs/dev/baselines.json --only acme_normal_gen5,acme_inverse_gen5 --out runs/dev/RESULTS_gen5_acme_mixed.md
```

```bash
python tools/wsp/analysis/cmp_split.py --mixed runs/dev/gen5/cmp_validation_acme_normal_gen5_mixed.json --private runs/dev/gen5/cmp_validation_acme_normal_gen5_script.json
```

The second file of that command is the comparison written when only the private copy had been
sealed. The evaluator cannot write it again now that both roots are sealed; it is kept in
`runs/dev/gen5/` (local, not in git) and names the prefixes of the private copy. The fabric flame:

```bash
target/release/wsp-dev front-feedback --release sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c --release sha256:d53e2944b62863014b875a8023b57b50ca28095f4d69c86e1a0c28fcf49d429b --tracks runs/dev/bass_front/val_sealed --label gen5-validation --quiet-cases --out runs/dev/gen5/front_validation_gen5.json
```

The results page itself is assembled from those aggregate files by a script kept with the session's
working files; its text is in git as `docs/wsp/RESULTS_GEN5.md`.

## 9.8 Registers of this dossier

```bash
python tools/wsp/dossier_registers.py
```

With `--transcripts <folder> …` it also rebuilds `registers/agents.md` from session transcripts,
which only the author's machine has.

## 9.9 Astra first-extension checkpoint — train only

These scoped tools read five explicit predecessor releases and new Astra directories.
They do not run global historical reassembly or evaluation scripts. Different existing
candidate artifacts are refused; no script chooses a coefficient.

```powershell
cargo build --release -p ignis-cli -p ignis-wsp-dev
python tools/wsp/astra/candidates.py prepare
python tools/wsp/astra/candidates.py assemble
python tools/wsp/astra/cards.py
python tools/wsp/astra/train_feedback.py
python tools/wsp/astra/train_feedback.py --receipt-directory runs/dev/astra/session_receipts
python tools/wsp/astra/train_summary.py
```

Feedback refuses to overwrite existing files. The second folder records the F-010 export
replay; the summarizer requires identical aggregates and checks case/session means. In an
already populated workspace use a fresh receipt directory for another execution, retaining
the originals. The helper forces shared `data` and train only; BASS uses explicit train
tracks. No new validation command is provided.

The outcome-free command actually run was:

```powershell
cargo run -p ignis-wsp --example astra_thermal_convergence -- --execute --registry-root artifacts --methanol-release sha256:ef2339fe8a8e0fa377adf9ff3aa9e44308f1161dc2792675698e2dcfe3ac2311 --heptane-release sha256:51ce4c7c3622f2563eb5472df7c751b6132a01eb50747e4b2406a3d64e14fac0
```

Exact execution commands/times and feedback hashes: `../feedback/astra_train_summary.json`.
Scientific interpretation: [ASTRA_RESULTS.md](ASTRA_RESULTS.md). Complete literals and
assembly receipts sit beside each new candidate; the synthetic convergence JSON is committed.
