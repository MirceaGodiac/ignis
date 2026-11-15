# Harness task sets (dev only)

`dev_tasks.json` is a **synthetic fixture** (`origin = synthetic_fixture`) for `ignis harness run`
(task M10): its questions are written for the offline fake runtimes (their keywords choose what the
fake does) and are not gold questions, NASA data or results. It carries no expected answers.

Tudor's T10 dev set replaces it through `--tasks-format gold` once its `GoldTask` contract is on
`main`. The private hold-out is never read here: `ignis harness run` refuses any path under a
`holdout/` directory and any task set that declares the `test` partition.
