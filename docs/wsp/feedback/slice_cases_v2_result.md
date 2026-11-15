# SLICE (PSI-106): both case versions, development, in-sample

- Case version 1: `tools/wsp/ext/slice_cases.py`, files `cases_*.jsonl`; the registered looks were scored on it.
- Case version 2: `tools/wsp/ext/slice_cases_v2.py`, files `cases_*_v2.jsonl`. The tube is taken from the investigators' picture table where it holds the flame (`docs/wsp/feedback/slice_tube_check.md`).
  - 37 train lines change: GMT 058 tests 1 and 2 go from 0.8 to 1.6 mm; GMT 061 tests 63 to 67 and 63A go from 0.4 to 2.1 mm.
  - No validation line changes, and the outcomes are the same file.
  - The 3.2 mm reading is not used anywhere.
- Switches, all defaulting to version 1, so every registered number still reproduces:
  - scorer: `tools/wsp/ext/law_score.py`, `SLICE_CASES = "v1" | "v2"`, used by `slice` and `slice2`;
  - what-if tool: `--cases v1|v2` for slice and slice2, and for bass2/bass3/bass4, where it sets the plastics coding. The version is printed in each run and logged in each line;
  - final-run tool: the investigation's `"coding"` field in the frozen list. The slice entry has none, so it stays on v1.
- Final-run refusal: `run_final_gen7.py` refuses a final slice run on v2. The v2 builder skips the picture-table folders of the test days by name (`FOLDER_DAY` lists train and validation folders only). On test days v2 would therefore silently fall back to the log's tube. The data lane has to add those folders first.
- Rehearsal: `rehearse --slug slice --coding v2` reproduces v1 against the registered numbers, and v2 against the v2 work files (both REPRODUCED). The bass rehearsal still reproduces too.
- Releases: slice W0 `sha256:d10c94bc…3006e`; slice2 s1 `sha256:d586d0d3…0b61`; slice2 S2 `sha256:0f5763f5749bb669b6ed9b70ac2629c8923838b3308f6094b2efcd7d8313388a`, assembled by the second engineer from `runs/dev/gen7/study/slice2_s2.wsp`.
- Scores: on the lines all three answer (302: 268 train, 34 validation; the 41 lines of day 075 are refused by all, as before). Brier; mean of day means and pooled; lower is better. The baselines are built from the version's own train file, so "by nominal tube" follows the tube. Producer: `runs/dev/gen7/slice_cases_versions_dev.json`.

| release | partition | lines | blow_off: Brier, mean of day means v1 | blow_off: Brier, mean of day means v2 | blow_off: pooled v1 | blow_off: pooled v2 | lifted: Brier, mean of day means v1 | lifted: Brier, mean of day means v2 | lifted: pooled v1 | lifted: pooled v2 |
|---|---|---|---|---|---|---|---|---|---|---|
| slice W0 | train | 268 | 0.1023 | 0.0300 | 0.1101 | 0.0363 | 0.1559 | 0.0623 | 0.1798 | 0.0845 |
| slice W0 | validation | 34 | 0.1548 | 0.1548 | 0.1548 | 0.1548 | 0.0886 | 0.0886 | 0.0886 | 0.0886 |
| slice W0 | pooled | 302 | 0.1089 | 0.0456 | 0.1151 | 0.0497 | 0.1474 | 0.0656 | 0.1695 | 0.0849 |
| slice2 s1 | train | 268 | 0.1023 | 0.0300 | 0.1101 | 0.0363 | 0.0305 | 0.0244 | 0.0346 | 0.0284 |
| slice2 s1 | validation | 34 | 0.1548 | 0.1548 | 0.1548 | 0.1548 | 0.0536 | 0.0536 | 0.0536 | 0.0536 |
| slice2 s1 | pooled | 302 | 0.1089 | 0.0456 | 0.1151 | 0.0497 | 0.0333 | 0.0280 | 0.0368 | 0.0313 |
| slice2 S2 | train | 268 | 0.0500 | 0.0419 | 0.0575 | 0.0492 | 0.0305 | 0.0244 | 0.0346 | 0.0284 |
| slice2 S2 | validation | 34 | 0.1159 | 0.1159 | 0.1159 | 0.1159 | 0.0536 | 0.0536 | 0.0536 | 0.0536 |
| slice2 S2 | pooled | 302 | 0.0583 | 0.0511 | 0.0641 | 0.0567 | 0.0333 | 0.0280 | 0.0368 | 0.0313 |
| baseline: incidence pooled (simple) | train | 268 | 0.0352 | 0.0352 | 0.0428 | 0.0428 | 0.0258 | 0.0258 | 0.0290 | 0.0290 |
| baseline: incidence pooled (simple) | validation | 34 | 0.1605 | 0.1605 | 0.1605 | 0.1605 | 0.0563 | 0.0563 | 0.0563 | 0.0563 |
| baseline: incidence pooled (simple) | pooled | 302 | 0.0509 | 0.0509 | 0.0561 | 0.0561 | 0.0297 | 0.0297 | 0.0320 | 0.0320 |
| baseline: incidence by nominal tube | train | 268 | 0.0335 | 0.0333 | 0.0410 | 0.0409 | 0.0249 | 0.0250 | 0.0280 | 0.0283 |
| baseline: incidence by nominal tube | validation | 34 | 0.1493 | 0.1454 | 0.1493 | 0.1454 | 0.0596 | 0.0596 | 0.0596 | 0.0596 |
| baseline: incidence by nominal tube | pooled | 302 | 0.0480 | 0.0473 | 0.0532 | 0.0527 | 0.0293 | 0.0294 | 0.0316 | 0.0318 |
| baseline: never (persistence) | train | 268 | 0.0362 | 0.0362 | 0.0448 | 0.0448 | 0.0265 | 0.0265 | 0.0299 | 0.0299 |
| baseline: never (persistence) | validation | 34 | 0.1765 | 0.1765 | 0.1765 | 0.1765 | 0.0588 | 0.0588 | 0.0588 | 0.0588 |
| baseline: never (persistence) | pooled | 302 | 0.0538 | 0.0538 | 0.0596 | 0.0596 | 0.0306 | 0.0306 | 0.0331 | 0.0331 |

blow_off, Brier by day (v1 / v2):

| day | slice W0 | slice2 s1 | slice2 S2 | incidence pooled (simple) | incidence by nominal tube | never (persistence) |
|---|---|---|---|---|---|---|
| 040 | 0.0012 / 0.0012 | 0.0012 / 0.0012 | 0.0021 / 0.0021 | 0.0028 / 0.0028 | 0.0023 / 0.0027 | 0.0000 / 0.0000 |
| 048 | 0.0003 / 0.0003 | 0.0003 / 0.0003 | 0.0009 / 0.0009 | 0.0028 / 0.0028 | 0.0006 / 0.0004 | 0.0000 / 0.0000 |
| 058 | 0.1000 / 0.0999 | 0.1000 / 0.0999 | 0.0927 / 0.0925 | 0.1169 / 0.1169 | 0.1112 / 0.1095 | 0.1277 / 0.1277 |
| 061 | 0.5749 / 0.0686 | 0.5749 / 0.0686 | 0.2085 / 0.1516 | 0.0716 / 0.0716 | 0.0736 / 0.0731 | 0.0769 / 0.0769 |
| 065 | 0.1548 / 0.1548 | 0.1548 / 0.1548 | 0.1159 / 0.1159 | 0.1605 / 0.1605 | 0.1493 / 0.1454 | 0.1765 / 0.1765 |
| 069 | 0.0181 / 0.0181 | 0.0181 / 0.0181 | 0.0254 / 0.0254 | 0.0255 / 0.0255 | 0.0233 / 0.0240 | 0.0253 / 0.0253 |
| 072 | 0.0212 / 0.0212 | 0.0212 / 0.0212 | 0.0203 / 0.0203 | 0.0241 / 0.0241 | 0.0233 / 0.0237 | 0.0238 / 0.0238 |
| 079 | 0.0006 / 0.0006 | 0.0006 / 0.0006 | 0.0001 / 0.0001 | 0.0028 / 0.0028 | 0.0000 / 0.0000 | 0.0000 / 0.0000 |

v1: S2 against slice2 s1 on blow_off: lower on 5 of 8 days whose scores differ.

v2: S2 against slice2 s1 on blow_off: lower on 4 of 8 days whose scores differ.

lifted, Brier by day (v1 / v2):

| day | slice W0 | slice2 s1 | slice2 S2 | incidence pooled (simple) | incidence by nominal tube | never (persistence) |
|---|---|---|---|---|---|---|
| 040 | 0.0199 / 0.0199 | 0.0017 / 0.0017 | 0.0017 / 0.0017 | 0.0008 / 0.0008 | 0.0004 / 0.0003 | 0.0000 / 0.0000 |
| 048 | 0.0037 / 0.0037 | 0.0003 / 0.0003 | 0.0003 / 0.0003 | 0.0008 / 0.0008 | 0.0006 / 0.0004 | 0.0000 / 0.0000 |
| 058 | 0.2002 / 0.1982 | 0.0703 / 0.0701 | 0.0703 / 0.0701 | 0.0811 / 0.0811 | 0.0764 / 0.0763 | 0.0851 / 0.0851 |
| 061 | 0.7116 / 0.0589 | 0.1087 / 0.0663 | 0.1087 / 0.0663 | 0.0734 / 0.0734 | 0.0723 / 0.0731 | 0.0769 / 0.0769 |
| 065 | 0.0886 / 0.0886 | 0.0536 / 0.0536 | 0.0536 / 0.0536 | 0.0563 / 0.0563 | 0.0596 / 0.0596 | 0.0588 / 0.0588 |
| 069 | 0.1090 / 0.1090 | 0.0095 / 0.0095 | 0.0095 / 0.0095 | 0.0008 / 0.0008 | 0.0014 / 0.0018 | 0.0000 / 0.0000 |
| 072 | 0.0385 / 0.0385 | 0.0219 / 0.0219 | 0.0219 / 0.0219 | 0.0233 / 0.0233 | 0.0233 / 0.0235 | 0.0238 / 0.0238 |
| 079 | 0.0080 / 0.0080 | 0.0007 / 0.0007 | 0.0007 / 0.0007 | 0.0008 / 0.0008 | 0.0000 / 0.0000 | 0.0000 / 0.0000 |

v1: S2 against slice2 s1 on lifted: lower on 0 of 0 days whose scores differ.

v2: S2 against slice2 s1 on lifted: lower on 0 of 0 days whose scores differ.

**Reading.**

- Only train day 061 moves much, and day 058 slightly; validation (day 065) is identical in both versions.
- On v1, day 061 is the loss S2 was written against: blow-off W0/s1 0.5749 against S2 0.2085.
- On v2 the 2.1 mm tube makes W0 and s1 right on day 061 (0.0686, below every baseline). S2 is worse there (0.1516).
- Pooled blow-off (mean of day means):
  - v1: S2 0.0583 against s1 0.1089, S2 lower on 5 of 8 days;
  - v2: S2 0.0511 against s1 0.0456, S2 lower on 4 of 8 days. On v2, S2 no longer passes the keep rule against s1.
- Lifted: S2 does not change it (identical to s1 in both versions; s1 0.0333 on v1, 0.0280 on v2).
- All of this is development, in-sample. Which version holds depends on the tube question (2.1 or 3.2 mm), which is not settled. Neither number decides it.
