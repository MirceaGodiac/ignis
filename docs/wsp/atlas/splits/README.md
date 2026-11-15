# Evaluation groups of the investigations outside the project's partitions

Frozen on 14 November 2026 by `tools/wsp/atlas/freeze_groups.py`, from the group ids of the test catalogue
(conditions only; the file of outcome values was not opened), with the project's split rule and seed
(`60/20/20 of groups (round half to even), groups shuffled by numpy default_rng(seed)`, seed 20261107). One JSON file per investigation lists every group and its partition.

| Investigation | Stratum | Status | Groups: train / validation / test | Rows of the catalogue: train / validation / test | Held |
|---|---|---|---|---|---|
| PSI-10 ACME/Flame Design | inverse_flames | split | 5 / 2 / 1 | 75 / 25 / 20 | |
| PSI-10 ACME/Flame Design | normal_flames | split | 8 / 3 / 3 | 114 / 41 / 52 | |
| PSI-20 ACME BRE | all | split | 11 / 4 / 3 | 11 / 4 / 3 | |
| PSI-21 ACME CLD Flame | all | split | 19 / 6 / 7 | 19 / 6 / 7 | |
| PSI-22 ACME E-FIELD Flames | all | split | 12 / 4 / 4 | 12 / 4 / 4 | |
| PSI-23 ACME s-Flame | all | split | 11 / 4 / 3 | 11 / 4 / 3 | |
| PSI-39 CFI | all | split | 31 / 10 / 10 | 136 / 53 / 38 | |
| PSI-47 DAFT / DAFT-2 | all | held: fewer than three groups | | | 2 groups, 2 rows |
| PSI-68 FLEX-2 | all | held: the table is a test matrix; no per-test list yet | | | 1 groups, 8 rows |
| PSI-98 SAFFIRE-I | all | held: fewer than three groups | | | 1 groups, 2 rows |
| PSI-99 SAFFIRE-II | all | held: fewer than three groups | | | 1 groups, 9 rows |
| PSI-100 SAFFIRE-III | all | held: fewer than three groups | | | 1 groups, 2 rows |
| PSI-101 SAME-R | all | split | 10 / 3 / 3 | 51 / 24 / 34 | |
| PSI-102 SAME | all | held: the table is a test matrix; no per-test list yet | | | 1 groups, 8 rows |
| PSI-106 SLICE | all | split | 7 / 2 / 3 | 7 / 2 / 3 | |
| PSI-107 SPICE | all | split | 7 / 2 / 2 | 267 / 78 / 90 | |

Not frozen here: PSI-69 (project split flex-660b7d9ede629b30 / flex-c43bbe3580ffce2a); PSI-159 (project splits acme_normal-c77bc5baee98d3e3 / acme_inverse-747032c921c8c8f2); PSI-25 (project split bass-4083156d5b18dab4); PSI-26 (declared outside every partition before this script existed; no row in the catalogue).

Catalogue used: `runs/dev/data_all/work/atlas/tests.jsonl`, sha256 `35546e129a160c6ab73c630be82fab98700bb5fd5fb92fa83954eb7af06cdfbd` (local, not in git; rebuilt by
`tools/wsp/atlas/normalize_tests.py`).
