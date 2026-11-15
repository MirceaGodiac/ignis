# F-008: train feedback loaded all family records before filtering

- **About:** new; partition isolation in `ignis-wsp-dev::dev_archives`
- **Reviewer:** GPT-6 Astra task agent, 14 November 2026
- **Verdict:** confirmed by static source inspection
- **Severity:** integrity
- **Partitions touched while reviewing:** other — source code only; no run records or outcomes opened

## What the dossier says

The dossier permits train-only feedback and prohibits author access to test outcomes and per-run
validation outcomes. The feedback command takes a partition and says that test is unavailable.

## What I did

Inspected `crates/ignis-wsp-dev/src/main.rs` around `dev_archives` and `feedback`, plus
`crates/ignis-data/src/views.rs` around `DataStore::run_archives` and
`crates/ignis-core/src/records.rs` around `Registry::load` and `Registry::archive`. I did not
open any dataset JSONL, Parquet view, evaluation artifact, validation output, or holdout.

## Evidence

`feedback` loaded the frozen split, called `dev_archives`, then iterated returned archives for
prefix construction and scoring. `dev_archives` previously called `DataStore::run_archives(family)`
before checking `split.partition_of`. `DataStore::run_archives` calls `Registry::load`, which reads
and deserializes all family experiment, observation, contour, control, and endpoint rows. Therefore
the train feedback path materialized non-train outcome-bearing records before filtering them. This
proves a software isolation gap; it does not prove that an author saw or used any such outcome.

## What should change

For the train path, select run identities from the frozen split first, inspect only top-level row
identity in the mixed JSONL registries, and deserialize domain records only for selected train
identities. Keep the ordinary loader for other existing workflows. Add synthetic tests in which an
unselected row has a shape that fails domain deserialization, demonstrating it is skipped. Record
that this is a source-level guard, not proof of past human exposure or an audit of every other
development command.

## Response

(The author answers here.)
