# Agent report (Opus 5.5 medium, 15 Nov 2026, extracted from transcript): unscored ACME sessions inventory

I found almost no ACME burner sessions that IGNIS has never scored. Every CFI-G and Flame Design day is already in a partition that was spent, and the only ACME days never opened (s-Flame) are flames the adopted F2 releases can't score. All that's left are a few dozen Flame Design tests that the transfer rule excluded on those spent days. I wrote a draft batch of 55 of them; 26 could be scored by F2 after small, pre-ignition fixes. Nothing needs downloading: their text files are already on disk.

**Which investigations count.** Only three ACME investigations have spherical-burner flames with the extinction vs operator-termination target: CFI-G (PSI-159), Flame Design (PSI-10) and s-Flame (PSI-23). BRE (PSI-20) is a flat burner, and CLD (PSI-21), E-FIELD (PSI-22) and SLICE (PSI-106) are jet or coflow flames. The local NASA listings were fetched earlier today (15 Nov), so I used them and did not re-query NASA.

**CFI-G (PSI-159)**
- **Normal flames:** 179 table rows. 129 are in the cohort (81 train, 28 validation, 20 test), all scored.
  - 47 had no hot ignition.
  - 3 have an unresolved endpoint cause (21088C1 on a train day, and two 50 % butane tests on test days).
- **Inverse flames:** 154 table rows. 131 are in the cohort (73 train, 24 validation, 34 test), all scored.
  - 23 had no hot ignition or only a flash.
- **Tests in the NASA files but not in the table:** 12 (21106A1–A3, 21106G1–G5, 21106G8, 21176M1, 21242F1, 21337A7).
  - 3 of them (21176M1, 21242F1, 21337A7) lack a needed text file.
  - None has table conditions, and whether a hot flame formed is unknown without reading the sensor channels.
- **Never-used CFI-G tests worth scoring: none.** The excluded rows have no flame to forecast, or their table outcome is already in `data/work/records`.

**Flame Design (PSI-10).** 327 table rows: 232 were included and scored (train, validation, and the test days in Step 2), and 95 were excluded:

| Why excluded | Count | Can it be fixed from inputs known before ignition? |
|---|---|---|
| Scripted short constant stage, then ramp to zero | 24 | Yes for 20: the script gives the planned flow change, and the test flows match an included test (a "sibling") |
| Scripted ignition-to-test ramp, normal flames | 22 | Partly: flows are still changing at the cutoffs, and most have no sibling |
| Table says RAMP, but the script shows a plain constant stage | 7 | No: the ramp was commanded live and only shows in the setpoint log, which also reveals the end |
| Inverse flames: dilution ramp (3) or multi-stage flows (5) | 8 | 3 multi-stage tests have siblings; the rest need a dilution law F2 doesn't have |
| "LOW O2" in the table (inverse) | 2 | No: there is no per-test O2 |
| Inside the condition range, but outside the F2 domain (inverse) | 9 | No: F2 refuses them; only the baselines could score them |
| Constant flow, only the stage-timing file (`_TP.txt`) missing | 3 | Yes, if `flame_design.py` may run without that file |
| Not in the NASA listing at all | 20 | No |

There are also 11 tests in the NASA files with no table row: 19115C1/D1/H1/P1, 19123H1, 19123L8, 19124H1, 19142B1, 19175B2, 19196N2 and 22024M1. The same blocker applies as for the CFI-G ones.

**s-Flame (PSI-23).** The three test days, 20142, 20170 and 20176, were never opened (61 test names). Their text files are already on disk, about 157 MB, unopened. They are already the frozen test partition in `docs/wsp/atlas/splits/PSI-23.json`, not a new batch. The adopted F2 releases can't score any of them:
- every test is helium-diluted, and F2 has no input for the diluent;
- day 20142 burns an H2/CH4 blend, which no release has a law for;
- days 20170 and 20176 mix about 21 % and 30 % O2 with no per-test O2 source.

Day 19245 has only logs and gas-analyser files.

**Proposed batch** (`docs/wsp/atlas/splits/ACME-batch-fresh.draft.json`): status DRAFT, grouped by test day, not committed. It holds 55 Flame Design tests on 16 days, with no endpoint measured or read:
- **A1 (20 tests):** the short-constant-then-ramp tests with siblings: 16 on train days (19115, 19137, 19156, 19189) and 4 on test day 19123 (L4–L7).
- **A2 (3 tests):** 19150H1, 21301C2 and 19206L6, where only the `_TP.txt` file is missing.
- **A3 (3 tests):** inverse multi-stage 22024B2, 22027B4 and 22027B8.
- **B1 (29 tests):** ignition ramps, inverse dilution ramps, and tests with no sibling. These need a flow conversion or a law F2 doesn't carry.

The 26 tier-A tests sit on 10 days, and 10 of them on day 19137. The file also lists every excluded group with its reason. The total is 204.7 MB of text (data, setpoints, stage timing and script files), all already on disk, so the download is 0 bytes.

**Things to know before freezing:**
- **The A1 label is close to fixed in advance.** The rule in `flame_design.py` calls the end "operator termination" whenever the scripted ramp starts first, 5–29 s after onset. So A1 mostly tests whether F2 respects the planned stop, not extinction physics.
- **I printed outcome-related columns by mistake.** I printed the after-test table columns (final O2 and pressure) for 7 rows. I dropped the three that were candidates (19123L3, 19206N1, 19142G1); the other four are train-day tests already outside the batch. I also printed the first CFI-G table rows, which include burn times. Those are cohort train tests already read, plus 21088C1, which is excluded anyway.
- **Not certified unread.** NASA also publishes per-day PDFs (test sheets, flow traces) and `_Comments.txt` files that may describe outcomes, and I can't tell whether anyone has read them for these tests.

I read only the experiment-table condition columns, the script parameter files and the listings. I opened no sensor channels, setpoint logs, stage-timing files, endpoint files, holdout or vault data.

My helper scripts (`fd_excl.py`, `fd_dom.py`, `make_draft.py`) are in `D:\Caches\Temp\claude\c2f4976cab650b4b\199e808b-35cd-44e2-918a-d476aca69199\scratchpad\`.