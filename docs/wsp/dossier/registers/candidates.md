# Register: every candidate under `configs/models`

Made by `tools/wsp/dossier_registers.py` from each candidate's `meta.json`, `release.json` and `edits.wsp`.
Full exposure records and rationales: `candidates.json` beside this file. "Statements" counts the
assignment lines of `edits.wsp` (for a blend or committee release most of them are copied member
values, see the file's header). "In git" is the state when the register was made; a release id shown
for a folder that is not in git may be stale (see `08_in_progress.md`).

| Candidate folder | Model | Built as | Release id | Parent | Statements | Elements (non-zero) | Partitions declared | In git |
|---|---|---|---|---|---|---|---|---|
| `acme/wsp2/normal/d3-nostops` | `acme_normal.wsp2` | literal_revision | `b1a334f3ede8` | `32822270de2c` | 2 | 353 (55) | train, validation | yes |
| `acme/wsp3/inverse/baseline-theory` | `acme_inverse.wsp3` | literal_one_shot | `d0a1366034b9` | `—` | 27 | 499 (25) | none | yes |
| `acme/wsp3/inverse/d1-fable` | `acme_inverse.wsp3` | literal_revision | `19c156e60036` | `d0a1366034b9` | 39 | 499 (54) | train, validation | yes |
| `acme/wsp3/inverse/d1-nostops` | `acme_inverse.wsp3` | literal_revision | `e6fbed49896d` | `19c156e60036` | 2 | 499 (54) | train, validation | yes |
| `acme/wsp3/inverse/d2-fable` | `acme_inverse.wsp3` | literal_revision | `3e4f850a187c` | `19c156e60036` | 4 | 499 (58) | train, validation | yes |
| `acme/wsp3/inverse/d3-fable` | `acme_inverse.wsp3` | literal_revision | `09749cb28b7c` | `19c156e60036` | 2 | 499 (55) | train, validation | yes |
| `acme/wsp3/inverse/d5-fable` | `acme_inverse.wsp3` | literal_revision | `9bdd4d278e21` | `19c156e60036` | 4 | 499 (58) | train, validation | yes |
| `acme/wsp3/inverse/d5-nostops` | `acme_inverse.wsp3` | literal_revision | `446e5ba70ed2` | `9bdd4d278e21` | 2 | 499 (58) | train, validation | yes |
| `acme/wsp3/normal/baseline-theory` | `acme_normal.wsp3` | literal_one_shot | `3d8d3b7a6e5c` | `—` | 27 | 499 (25) | none | yes |
| `acme/wsp3/normal/d1-fable` | `acme_normal.wsp3` | literal_revision | `221a2ea22aca` | `3d8d3b7a6e5c` | 43 | 499 (55) | train, validation | yes |
| `acme/wsp3/normal/d2-fable` | `acme_normal.wsp3` | literal_revision | `4fa8de0bc17e` | `221a2ea22aca` | 1 | 499 (55) | train, validation | yes |
| `acme/wsp3/normal/d2-nostops` | `acme_normal.wsp3` | literal_revision | `9354b037a0a6` | `4fa8de0bc17e` | 2 | 499 (56) | train, validation | yes |
| `acme/wsp4/inverse/baseline-pair` | `acme_inverse.wsp4` | literal_one_shot | `989c7b1e14c3` | `—` | 129 | 854 (43) | none | yes |
| `acme/wsp4/inverse/d1-fable` | `acme_inverse.wsp4` | literal_revision | `664694894f88` | `989c7b1e14c3` | 88 | 854 (98) | train, validation | yes |
| `acme/wsp4/normal/baseline-pair` | `acme_normal.wsp4` | literal_one_shot | `d579e6da2565` | `—` | 129 | 854 (43) | none | yes |
| `acme/wsp4/normal/d1-fable` | `acme_normal.wsp4` | literal_revision | `8be14acec4f1` | `d579e6da2565` | 92 | 854 (111) | train, validation | yes |
| `acme/wsp5/inverse/baseline-set` | `acme_inverse.wsp5` | literal_one_shot | `3d7d010fd848` | `—` | 417 | 2715 (134) | none | yes |
| `acme/wsp5/inverse/carry-d1` | `acme_inverse.wsp5` | literal_revision | `ff3449c4123f` | `3d7d010fd848` | 49 | 2715 (162) | train, validation | yes |
| `acme/wsp5/inverse/d1-fable` | `acme_inverse.wsp5` | literal_revision | `195ffcb19788` | `3d7d010fd848` | 97 | 2715 (196) | train, validation | yes |
| `acme/wsp5/inverse/p1-fable` | `acme_inverse.wsp5` | literal_revision | `d7292e8974ee` | `3d7d010fd848` | 183 | 2715 (261) | train, validation | yes |
| `acme/wsp5/normal/baseline-set` | `acme_normal.wsp5` | literal_one_shot | `6044b69bd5d1` | `—` | 417 | 2715 (134) | none | yes |
| `acme/wsp5/normal/carry-d1` | `acme_normal.wsp5` | literal_revision | `dce4d7f0b3ce` | `6044b69bd5d1` | 95 | 2715 (202) | train, validation | yes |
| `acme/wsp5/normal/p1-fable` | `acme_normal.wsp5` | literal_revision | `f71fce5f22f2` | `6044b69bd5d1` | 179 | 2715 (275) | train, validation | yes |
| `bass/front/baseline-persistence` | `bass_front.wsp` | literal_one_shot | `ce53fe1fb3e6` | `—` | 25 | 159 (15) | none | yes |
| `bass/front/d1-fable` | `bass_front.wsp` | literal_revision | `ac15e49f1cb6` | `ce53fe1fb3e6` | 32 | 159 (35) | train, validation | yes |
| `bass/front/d2-fable` | `bass_front.wsp` | literal_revision | `0187941810a8` | `ac15e49f1cb6` | 6 | 159 (31) | train, validation | yes |
| `bass/front2/baseline-persistence` | `bass_front.wsp2` | literal_one_shot | `a100f7286462` | `—` | 32 | 168 (16) | none | yes |
| `bass/front2/carry-d1` | `bass_front.wsp2` | literal_revision | `c50c7144e433` | `a100f7286462` | 39 | 168 (36) | train, validation | yes |
| `bass/front2/p1-fable` | `bass_front.wsp2` | literal_revision | `d53e2944b628` | `c50c7144e433` | 2 | 168 (36) | train, validation | yes |
| `flex/wsp2/heptane/baseline-d2` | `flex_heptane.wsp2` | literal_one_shot | `8e40db9dceb8` | `—` | 41 | 575 (33) | none | yes |
| `flex/wsp2/heptane/d2-fable` | `flex_heptane.wsp2` | literal_revision | `43f7871f7780` | `8e40db9dceb8` | 78 | 575 (93) | train, validation | yes |
| `flex/wsp2/methanol/baseline-d2` | `flex_methanol.wsp2` | literal_one_shot | `abf919b61998` | `—` | 33 | 378 (28) | none | yes |
| `flex/wsp2/methanol/d3-fable` | `flex_methanol.wsp2` | literal_revision | `55e0cb12a5a0` | `abf919b61998` | 59 | 378 (68) | train, validation | yes |
| `flex/wsp3/heptane/baseline-d2` | `flex_heptane.wsp3` | literal_one_shot | `24d751a0c57f` | `—` | 50 | 588 (39) | none | yes |
| `flex/wsp3/heptane/carry-d2` | `flex_heptane.wsp3` | literal_revision | `bb12a8887956` | `24d751a0c57f` | 87 | 588 (99) | train, validation | yes |
| `flex/wsp3/heptane/d3-fable` | `flex_heptane.wsp3` | literal_revision | `d3cf6b7c2569` | `24d751a0c57f` | 87 | 588 (107) | train, validation | yes |
| `flex/wsp3/heptane/d4-flags` | `flex_heptane.wsp3` | literal_revision | `b9a121a1a711` | `24d751a0c57f` | 87 | 588 (104) | train, validation | yes |
| `flex/wsp3/methanol/baseline-d2` | `flex_methanol.wsp3` | literal_one_shot | `38d9059ceea2` | `—` | 42 | 391 (34) | none | yes |
| `flex/wsp3/methanol/carry-d3` | `flex_methanol.wsp3` | literal_revision | `33bb9ae6212d` | `38d9059ceea2` | 68 | 391 (74) | train, validation | yes |
| `flex/wsp3/methanol/d4-fable` | `flex_methanol.wsp3` | literal_revision | `4e975d3b5b03` | `38d9059ceea2` | 76 | 391 (87) | train, validation | yes |
| `flex/wsp3/methanol/d5-flags` | `flex_methanol.wsp3` | literal_revision | `0b57dba76eda` | `38d9059ceea2` | 68 | 391 (80) | train, validation | yes |
| `flex/wsp4/heptane/baseline-d2` | `flex_heptane.wsp4` | literal_one_shot | `911530605aa8` | `—` | 47 | 581 (37) | none | yes |
| `flex/wsp4/heptane/carry-d2` | `flex_heptane.wsp4` | literal_revision | `02c1d7244317` | `911530605aa8` | 84 | 581 (97) | train, validation | yes |
| `flex/wsp4/heptane/f1-fable` | `flex_heptane.wsp4` | literal_revision | `78fe863515a2` | `911530605aa8` | 84 | 581 (99) | train, validation | yes |
| `flex/wsp4/heptane/r1-fable` | `flex_heptane.wsp4` | literal_revision | `759d14b31c09` | `911530605aa8` | 84 | 581 (98) | train, validation | yes |
| `flex/wsp4/methanol/baseline-d2` | `flex_methanol.wsp4` | literal_one_shot | `c1b1321c4da8` | `—` | 39 | 384 (32) | none | yes |
| `flex/wsp4/methanol/carry-d3` | `flex_methanol.wsp4` | literal_revision | `b57925a094c6` | `c1b1321c4da8` | 65 | 384 (72) | train, validation | yes |
| `flex/wsp4/methanol/f1-fable` | `flex_methanol.wsp4` | literal_revision | `c7d127b37fed` | `c1b1321c4da8` | 65 | 384 (74) | train, validation | yes |
| `flex/wsp4/methanol/r1-fable` | `flex_methanol.wsp4` | literal_revision | `8eaa4d1ef76d` | `c1b1321c4da8` | 65 | 384 (73) | train, validation | yes |
| `flex/wsp5/heptane/baseline-d2` | `flex_heptane.wsp5` | literal_one_shot | `adac8c30e190` | `—` | 67 | 603 (42) | none | yes |
| `flex/wsp5/heptane/carry-d2` | `flex_heptane.wsp5` | literal_revision | `03e0f63cf88c` | `adac8c30e190` | 104 | 603 (102) | train, validation | yes |
| `flex/wsp5/heptane/p0-fable` | `flex_heptane.wsp5` | literal_revision | `c2c7b700867f` | `03e0f63cf88c` | 14 | 603 (107) | train, validation | yes |
| `flex/wsp5/heptane/p1-fable` | `flex_heptane.wsp5` | literal_revision | `c27df7bb7f70` | `c2c7b700867f` | 6 | 603 (109) | train, validation | yes |
| `flex/wsp5/methanol/baseline-d2` | `flex_methanol.wsp5` | literal_one_shot | `575b0f63d593` | `—` | 59 | 406 (37) | none | yes |
| `flex/wsp5/methanol/carry-d3` | `flex_methanol.wsp5` | literal_revision | `2181b643430a` | `575b0f63d593` | 85 | 406 (77) | train, validation | yes |
| `flex/wsp5/methanol/p0-fable` | `flex_methanol.wsp5` | literal_revision | `2e4c55f44b78` | `2181b643430a` | 17 | 406 (90) | train, validation | yes |
| `flex/wsp5/methanol/p1-fable` | `flex_methanol.wsp5` | literal_revision | `7884e61c9995` | `2e4c55f44b78` | 6 | 406 (92) | train, validation | yes |

80 candidates. `literal_one_shot` = written with no outcome exposure (W0 and reference baselines);
`literal_revision` = typed over a parent after development data were read.
