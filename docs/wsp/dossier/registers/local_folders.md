# Register: local working folders the dossier cites

Made by `tools/wsp/dossier_registers.py`. These folders are git-ignored; counts and bytes only. A reviewer
rebuilds what is needed with the commands of `09_reproduce.md`.

| Folder | What it is | Files | Bytes |
|---|---|---|---|
| `data` | shared data root (raw samples, manifests, records, splits); the test partition's files are in it and are never opened by this lane | 747 | 252,450,575 |
| `runs/dev/analysis` | train-only tables of tools/wsp/analysis (profiles, per-run finite differences) | 6 | 201,389 |
| `runs/dev/research5/shift` | where validation inputs sit relative to train (inputs only) | 8 | 124,075 |
| `runs/dev/research5/flex_flame` | flame-camera measurement of the FLEX train runs | 294 | 117,903,688 |
| `runs/dev/research5/acme_raw` | raw-channel measurement of the ACME train tests | 18 | 791,948 |
| `runs/dev/research5/bass1` | tracks of the earlier BASS campaign (PSI-26, outside every partition) | 150 | 53,763,915 |
| `runs/dev/research6` | physics dossiers of the fifth generation (working files) | 199 | 15,879,991 |
| `runs/dev/nasa/PSI-69` | downloads: flame-camera files of FLEX train runs | 203 | 3,990,177 |
| `runs/dev/nasa/PSI-159` | downloads: raw channels, timelines and script parameters of ACME tests | 797 | 734,636,533 |
| `runs/dev/nasa/PSI-26` | downloads: videos and documents of the earlier BASS campaign | 950 | 554,690,714 |
| `runs/dev/data_ext` | PRIVATE copy of the data root with flame-camera series and programmed stops added; holds validation series: read only through the prefix builder | 1000 | 315,797,779 |
| `runs/dev/bass_front/val_sealed` | SEALED validation front tracks: never opened by the author of the model | 17 | 338,180 |
| `runs/dev/ext` | train feedback of the fifth-generation candidates on the private data root | 4 | 4,005,724 |
| `runs/dev/gen5` | fifth generation: train feedback, single-change variants, the aggregate files of the registered validation look and its split comparison (the comparison files hold per-case validation scores: aggregates only are read) | 109 | 18,373,156 |
| `runs/dev/holdout_ext` | PRIVATE vault: validation continuations of the private data root, written by the evaluator's deposit command; never opened | 114 | 21,083,418 |
| `artifacts/models` | local release registry (rebuilt by tools/wsp/reassemble.py) | 374 | 45,936,806 |
