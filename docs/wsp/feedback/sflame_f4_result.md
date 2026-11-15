# s-Flame development tables of F4 (PROTOCOL 13.6), comparators

Releases on `acme_normal.wsp13`, transcribed from `docs/wsp/research/phys_sflame_f4.md` section 6 (each a revision of the one before; tensors equal to the dossier's runs): `f4-methane` `sha256:57fe4255…ebe2` (of F3 stage 1), `f4-helium` `sha256:370ed3cb…62c0`, `f4-hydrogen` `sha256:4cc2fa60…9436`. F3 stage 1 (`sha256:d1660331…45aa`) stays the model; these are comparators carried to the final run.

Tool: `tools/wsp/ext/sflame_whatif.py --base configs/models/acme/wsp13/normal/f3-s1 --edits <stage lines> --compare F2=<F2>:nodiluent --outcomes coded,v2`; the child of each run has the tensors of the assembled stage. Development days of s-Flame only (train and validation; the test days are not opened). Sum of the integrated Brier scores of extinction and operator termination, 6 to 120 s, case means; each stratum scored as its own set. F2 has no diluent input and is shown where 13.1 registered it to answer. Blends are refused by the lane's adapter for every release (no blend law). Two outcome versions: **version 1** (coded, the case files on which the registered looks were scored) and **version 2** (the data lane's general rule, `docs/wsp/feedback/sflame_endpoints_v2.md`, which reproduces 15 of the dossier's 16 adjudicated labels; 20006C1 is an operator termination there), the second beside the first, never replacing it. These are in-sample development numbers for the dossier's author.

## Stage 1, methane (`f4-methane`), `sha256:57fe4255b51b4b988da8aa021d71c7191f91b9527eee765226b34a1ed1b6ebe2`

Expectation (as logged before the run): F4 stage 1 (phys_sflame_f4.md 4, M2), official table v1 and v2: validation methane 0.245 -> about 0.09 on v1; nitrogen ethylene, CFI-G, Flame Design exact

Neutrality: CFI-G train 0 of 210 cases moved; Flame Design train days 0 of 444 tables moved.

### version 1 (coded) endpoints

| days | stratum | group | cases | F4 stage | F3 stage 1 | F2 | CFI-G inc. | s-Flame inc. |
|---|---|---|---|---|---|---|---|---|
| train | hydrocarbons in nitrogen | pooled | 53 | 0.0730 \| 0.0763 | 0.0736 \| 0.0769 | 0.0914 \| 0.0891 | 0.2725 | 0.3008 |
| train | hydrocarbons in nitrogen | ethylene in nitrogen | 43 | 0.0887 \| 0.0928 | 0.0887 \| 0.0928 | 0.1050 \| 0.1022 | 0.2710 | 0.3131 |
| train | hydrocarbons in nitrogen | methane in nitrogen | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 19254 | 28 | 0.0864 \| 0.0840 | 0.0864 \| 0.0840 | 0.1279 \| 0.1081 | 0.3050 | 0.3248 |
| train | hydrocarbons in nitrogen | day 19304 | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 20029 | 15 | 0.0930 \| 0.1093 | 0.0930 \| 0.1093 | 0.0621 \| 0.0911 | 0.2077 | 0.2912 |
| train | hydrogen | – | 0 | refused (0 answered) | | | | |
| train | blends | – | 0 | refused (0 answered) | | | | |
| validation | hydrocarbons in nitrogen | pooled | 46 | 0.0815 \| 0.0811 | 0.2048 \| 0.2051 | 0.2619 \| 0.3698 | 0.2793 | 0.3218 |
| validation | hydrocarbons in nitrogen | ethylene in nitrogen | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | methane in nitrogen | 37 | 0.0916 \| 0.0911 | 0.2449 \| 0.2452 | 0.3157 \| 0.4498 | 0.2930 | 0.3293 |
| validation | hydrocarbons in nitrogen | day 19256 | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | day 19308 | 37 | 0.0916 \| 0.0911 | 0.2449 \| 0.2452 | 0.3157 \| 0.4498 | 0.2930 | 0.3293 |
| validation | hydrocarbons in helium | – | 0 | refused (0 answered) | | | | |

### version 2 (the data lane rule) endpoints

| days | stratum | group | cases | F4 stage | F3 stage 1 | F2 | CFI-G inc. | s-Flame inc. |
|---|---|---|---|---|---|---|---|---|
| train | hydrocarbons in nitrogen | pooled | 53 | 0.0730 \| 0.0763 | 0.0736 \| 0.0769 | 0.0914 \| 0.0891 | 0.2725 | 0.3008 |
| train | hydrocarbons in nitrogen | ethylene in nitrogen | 43 | 0.0887 \| 0.0928 | 0.0887 \| 0.0928 | 0.1050 \| 0.1022 | 0.2710 | 0.3131 |
| train | hydrocarbons in nitrogen | methane in nitrogen | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 19254 | 28 | 0.0864 \| 0.0840 | 0.0864 \| 0.0840 | 0.1279 \| 0.1081 | 0.3050 | 0.3248 |
| train | hydrocarbons in nitrogen | day 19304 | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 20029 | 15 | 0.0930 \| 0.1093 | 0.0930 \| 0.1093 | 0.0621 \| 0.0911 | 0.2077 | 0.2912 |
| train | hydrogen | – | 0 | refused (0 answered) | | | | |
| train | blends | – | 0 | refused (0 answered) | | | | |
| validation | hydrocarbons in nitrogen | pooled | 48 | 0.0401 \| 0.0397 | 0.1620 \| 0.1630 | 0.2276 \| 0.3409 | 0.2605 | 0.3079 |
| validation | hydrocarbons in nitrogen | ethylene in nitrogen | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | methane in nitrogen | 39 | 0.0401 \| 0.0396 | 0.1902 \| 0.1913 | 0.2707 \| 0.4101 | 0.2692 | 0.3118 |
| validation | hydrocarbons in nitrogen | day 19256 | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | day 19308 | 39 | 0.0401 \| 0.0396 | 0.1902 \| 0.1913 | 0.2707 \| 0.4101 | 0.2692 | 0.3118 |
| validation | hydrocarbons in helium | – | 0 | refused (0 answered) | | | | |

Cells: without \| with the planned stop. "–": the release does not answer every case of the group.

## Stage 2, helium (`f4-helium`), `sha256:370ed3cb1d72bccacef3ce63e1f3c9e6d7bf6d08cbf8fa81ed4cd8b57e4a62c0`

Expectation (as logged before the run): F4 stage 2 (He1), official table v1 and v2: ethylene in helium about 0.18, methane in helium about 0.006, helium pooled 0.11; 0 and 0 moved

Neutrality: CFI-G train 0 of 210 cases moved; Flame Design train days 0 of 444 tables moved.

### version 1 (coded) endpoints

| days | stratum | group | cases | F4 stage | F3 stage 1 | F2 | CFI-G inc. | s-Flame inc. |
|---|---|---|---|---|---|---|---|---|
| train | hydrocarbons in nitrogen | pooled | 53 | 0.0730 \| 0.0763 | 0.0736 \| 0.0769 | 0.0914 \| 0.0891 | 0.2725 | 0.3008 |
| train | hydrocarbons in nitrogen | ethylene in nitrogen | 43 | 0.0887 \| 0.0928 | 0.0887 \| 0.0928 | 0.1050 \| 0.1022 | 0.2710 | 0.3131 |
| train | hydrocarbons in nitrogen | methane in nitrogen | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 19254 | 28 | 0.0864 \| 0.0840 | 0.0864 \| 0.0840 | 0.1279 \| 0.1081 | 0.3050 | 0.3248 |
| train | hydrocarbons in nitrogen | day 19304 | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 20029 | 15 | 0.0930 \| 0.1093 | 0.0930 \| 0.1093 | 0.0621 \| 0.0911 | 0.2077 | 0.2912 |
| train | hydrogen | – | 0 | refused (0 answered) | | | | |
| train | blends | – | 0 | refused (0 answered) | | | | |
| validation | hydrocarbons in nitrogen | pooled | 46 | 0.0815 \| 0.0811 | 0.2048 \| 0.2051 | 0.2619 \| 0.3698 | 0.2793 | 0.3218 |
| validation | hydrocarbons in nitrogen | ethylene in nitrogen | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | methane in nitrogen | 37 | 0.0916 \| 0.0911 | 0.2449 \| 0.2452 | 0.3157 \| 0.4498 | 0.2930 | 0.3293 |
| validation | hydrocarbons in nitrogen | day 19256 | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | day 19308 | 37 | 0.0916 \| 0.0911 | 0.2449 \| 0.2452 | 0.3157 \| 0.4498 | 0.2930 | 0.3293 |
| validation | hydrocarbons in helium | pooled | 35 | 0.1105 \| 0.1099 | – \| – | – \| – | 0.3299 | 0.3244 |
| validation | hydrocarbons in helium | ethylene in helium | 21 | 0.1799 \| 0.1790 | – \| – | – \| – | 0.3639 | 0.3610 |
| validation | hydrocarbons in helium | methane in helium | 14 | 0.0064 \| 0.0063 | – \| – | – \| – | 0.2788 | 0.2696 |
| validation | hydrocarbons in helium | day 20160 | 14 | 0.0064 \| 0.0063 | – \| – | – \| – | 0.2788 | 0.2696 |
| validation | hydrocarbons in helium | day 20164 | 21 | 0.1799 \| 0.1790 | – \| – | – \| – | 0.3639 | 0.3610 |

### version 2 (the data lane rule) endpoints

| days | stratum | group | cases | F4 stage | F3 stage 1 | F2 | CFI-G inc. | s-Flame inc. |
|---|---|---|---|---|---|---|---|---|
| train | hydrocarbons in nitrogen | pooled | 53 | 0.0730 \| 0.0763 | 0.0736 \| 0.0769 | 0.0914 \| 0.0891 | 0.2725 | 0.3008 |
| train | hydrocarbons in nitrogen | ethylene in nitrogen | 43 | 0.0887 \| 0.0928 | 0.0887 \| 0.0928 | 0.1050 \| 0.1022 | 0.2710 | 0.3131 |
| train | hydrocarbons in nitrogen | methane in nitrogen | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 19254 | 28 | 0.0864 \| 0.0840 | 0.0864 \| 0.0840 | 0.1279 \| 0.1081 | 0.3050 | 0.3248 |
| train | hydrocarbons in nitrogen | day 19304 | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 20029 | 15 | 0.0930 \| 0.1093 | 0.0930 \| 0.1093 | 0.0621 \| 0.0911 | 0.2077 | 0.2912 |
| train | hydrogen | – | 0 | refused (0 answered) | | | | |
| train | blends | – | 0 | refused (0 answered) | | | | |
| validation | hydrocarbons in nitrogen | pooled | 48 | 0.0401 \| 0.0397 | 0.1620 \| 0.1630 | 0.2276 \| 0.3409 | 0.2605 | 0.3079 |
| validation | hydrocarbons in nitrogen | ethylene in nitrogen | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | methane in nitrogen | 39 | 0.0401 \| 0.0396 | 0.1902 \| 0.1913 | 0.2707 \| 0.4101 | 0.2692 | 0.3118 |
| validation | hydrocarbons in nitrogen | day 19256 | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | day 19308 | 39 | 0.0401 \| 0.0396 | 0.1902 \| 0.1913 | 0.2707 \| 0.4101 | 0.2692 | 0.3118 |
| validation | hydrocarbons in helium | pooled | 35 | 0.1105 \| 0.1099 | – \| – | – \| – | 0.3299 | 0.3244 |
| validation | hydrocarbons in helium | ethylene in helium | 21 | 0.1799 \| 0.1790 | – \| – | – \| – | 0.3639 | 0.3610 |
| validation | hydrocarbons in helium | methane in helium | 14 | 0.0064 \| 0.0063 | – \| – | – \| – | 0.2788 | 0.2696 |
| validation | hydrocarbons in helium | day 20160 | 14 | 0.0064 \| 0.0063 | – \| – | – \| – | 0.2788 | 0.2696 |
| validation | hydrocarbons in helium | day 20164 | 21 | 0.1799 \| 0.1790 | – \| – | – \| – | 0.3639 | 0.3610 |

Cells: without \| with the planned stop. "–": the release does not answer every case of the group.

## Stage 3, hydrogen (`f4-hydrogen`), `sha256:4cc2fa60cc8c40b316594034ab8f299946117705736677e941c99f645fb89436`

Expectation (as logged before the run): F4 stage 3 (Hy2), official table v1 and v2: hydrogen below both baselines, near the dossier's adjudicated 0.315 / 0.210 on v2; blends refused by the adapter; 0 and 0 moved

Neutrality: CFI-G train 0 of 210 cases moved; Flame Design train days 0 of 444 tables moved.

### version 1 (coded) endpoints

| days | stratum | group | cases | F4 stage | F3 stage 1 | F2 | CFI-G inc. | s-Flame inc. |
|---|---|---|---|---|---|---|---|---|
| train | hydrocarbons in nitrogen | pooled | 53 | 0.0730 \| 0.0763 | 0.0736 \| 0.0769 | 0.0914 \| 0.0891 | 0.2725 | 0.3008 |
| train | hydrocarbons in nitrogen | ethylene in nitrogen | 43 | 0.0887 \| 0.0928 | 0.0887 \| 0.0928 | 0.1050 \| 0.1022 | 0.2710 | 0.3131 |
| train | hydrocarbons in nitrogen | methane in nitrogen | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 19254 | 28 | 0.0864 \| 0.0840 | 0.0864 \| 0.0840 | 0.1279 \| 0.1081 | 0.3050 | 0.3248 |
| train | hydrocarbons in nitrogen | day 19304 | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 20029 | 15 | 0.0930 \| 0.1093 | 0.0930 \| 0.1093 | 0.0621 \| 0.0911 | 0.2077 | 0.2912 |
| train | hydrogen | pooled | 70 | 0.4363 \| 0.3948 | – \| – | – \| – | 0.8330 | 0.6117 |
| train | hydrogen | hydrogen in helium | 34 | 0.3018 \| 0.3018 | – \| – | – \| – | 0.9553 | 0.6871 |
| train | hydrogen | hydrogen in nitrogen | 36 | 0.5634 \| 0.4826 | – \| – | – \| – | 0.7176 | 0.5405 |
| train | hydrogen | day 19352 | 9 | 0.8875 \| 0.8874 | – \| – | – \| – | 0.7193 | 0.5384 |
| train | hydrogen | day 20024 | 27 | 0.4554 \| 0.3476 | – \| – | – \| – | 0.7170 | 0.5412 |
| train | hydrogen | day 20155 | 14 | 0.5657 \| 0.5657 | – \| – | – \| – | 0.8580 | 0.6385 |
| train | hydrogen | day 20190 | 20 | 0.1171 \| 0.1170 | – \| – | – \| – | 1.0235 | 0.7211 |
| train | blends | – | 0 | refused (0 answered) | | | | |
| validation | hydrocarbons in nitrogen | pooled | 46 | 0.0815 \| 0.0811 | 0.2048 \| 0.2051 | 0.2619 \| 0.3698 | 0.2793 | 0.3218 |
| validation | hydrocarbons in nitrogen | ethylene in nitrogen | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | methane in nitrogen | 37 | 0.0916 \| 0.0911 | 0.2449 \| 0.2452 | 0.3157 \| 0.4498 | 0.2930 | 0.3293 |
| validation | hydrocarbons in nitrogen | day 19256 | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | day 19308 | 37 | 0.0916 \| 0.0911 | 0.2449 \| 0.2452 | 0.3157 \| 0.4498 | 0.2930 | 0.3293 |
| validation | hydrocarbons in helium | pooled | 35 | 0.1105 \| 0.1099 | – \| – | – \| – | 0.3299 | 0.3244 |
| validation | hydrocarbons in helium | ethylene in helium | 21 | 0.1799 \| 0.1790 | – \| – | – \| – | 0.3639 | 0.3610 |
| validation | hydrocarbons in helium | methane in helium | 14 | 0.0064 \| 0.0063 | – \| – | – \| – | 0.2788 | 0.2696 |
| validation | hydrocarbons in helium | day 20160 | 14 | 0.0064 \| 0.0063 | – \| – | – \| – | 0.2788 | 0.2696 |
| validation | hydrocarbons in helium | day 20164 | 21 | 0.1799 \| 0.1790 | – \| – | – \| – | 0.3639 | 0.3610 |

### version 2 (the data lane rule) endpoints

| days | stratum | group | cases | F4 stage | F3 stage 1 | F2 | CFI-G inc. | s-Flame inc. |
|---|---|---|---|---|---|---|---|---|
| train | hydrocarbons in nitrogen | pooled | 53 | 0.0730 \| 0.0763 | 0.0736 \| 0.0769 | 0.0914 \| 0.0891 | 0.2725 | 0.3008 |
| train | hydrocarbons in nitrogen | ethylene in nitrogen | 43 | 0.0887 \| 0.0928 | 0.0887 \| 0.0928 | 0.1050 \| 0.1022 | 0.2710 | 0.3131 |
| train | hydrocarbons in nitrogen | methane in nitrogen | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 19254 | 28 | 0.0864 \| 0.0840 | 0.0864 \| 0.0840 | 0.1279 \| 0.1081 | 0.3050 | 0.3248 |
| train | hydrocarbons in nitrogen | day 19304 | 10 | 0.0054 \| 0.0054 | 0.0084 \| 0.0083 | 0.0330 \| 0.0329 | 0.2788 | 0.2482 |
| train | hydrocarbons in nitrogen | day 20029 | 15 | 0.0930 \| 0.1093 | 0.0930 \| 0.1093 | 0.0621 \| 0.0911 | 0.2077 | 0.2912 |
| train | hydrogen | pooled | 76 | 0.3140 \| 0.2094 | – \| – | – \| – | 0.9087 | 0.6443 |
| train | hydrogen | hydrogen in helium | 36 | 0.1818 \| 0.1818 | – \| – | – \| – | 1.0037 | 0.7014 |
| train | hydrogen | hydrogen in nitrogen | 40 | 0.4330 \| 0.2342 | – \| – | – \| – | 0.8232 | 0.5929 |
| train | hydrogen | day 19352 | 13 | 0.6438 \| 0.6437 | – \| – | – \| – | 0.7880 | 0.5706 |
| train | hydrogen | day 20024 | 27 | 0.3315 \| 0.0370 | – \| – | – \| – | 0.8402 | 0.6036 |
| train | hydrogen | day 20155 | 14 | 0.2927 \| 0.2927 | – \| – | – \| – | 0.9675 | 0.6791 |
| train | hydrogen | day 20190 | 22 | 0.1113 \| 0.1113 | – \| – | – \| – | 1.0266 | 0.7157 |
| train | blends | – | 0 | refused (0 answered) | | | | |
| validation | hydrocarbons in nitrogen | pooled | 48 | 0.0401 \| 0.0397 | 0.1620 \| 0.1630 | 0.2276 \| 0.3409 | 0.2605 | 0.3079 |
| validation | hydrocarbons in nitrogen | ethylene in nitrogen | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | methane in nitrogen | 39 | 0.0401 \| 0.0396 | 0.1902 \| 0.1913 | 0.2707 \| 0.4101 | 0.2692 | 0.3118 |
| validation | hydrocarbons in nitrogen | day 19256 | 9 | 0.0400 \| 0.0401 | 0.0400 \| 0.0401 | 0.0409 \| 0.0410 | 0.2229 | 0.2907 |
| validation | hydrocarbons in nitrogen | day 19308 | 39 | 0.0401 \| 0.0396 | 0.1902 \| 0.1913 | 0.2707 \| 0.4101 | 0.2692 | 0.3118 |
| validation | hydrocarbons in helium | pooled | 35 | 0.1105 \| 0.1099 | – \| – | – \| – | 0.3299 | 0.3244 |
| validation | hydrocarbons in helium | ethylene in helium | 21 | 0.1799 \| 0.1790 | – \| – | – \| – | 0.3639 | 0.3610 |
| validation | hydrocarbons in helium | methane in helium | 14 | 0.0064 \| 0.0063 | – \| – | – \| – | 0.2788 | 0.2696 |
| validation | hydrocarbons in helium | day 20160 | 14 | 0.0064 \| 0.0063 | – \| – | – \| – | 0.2788 | 0.2696 |
| validation | hydrocarbons in helium | day 20164 | 21 | 0.1799 \| 0.1790 | – \| – | – \| – | 0.3639 | 0.3610 |

Cells: without \| with the planned stop. "–": the release does not answer every case of the group.

