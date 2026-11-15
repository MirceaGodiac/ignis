# 7. Limits, exposures and mistakes

What a reviewer should hold against the results before reading any score. Nothing here was found by
somebody else: it is the author's own ledger, so it is likely to be incomplete in the places where
he did not think to look.

## 7.1 Exposure ledger

Every known way in which information about validation or test runs could have reached a typed
number. "Reached a number?" is the author's own assessment and is exactly what a review should
test.

| Id | What happened | When, who | Reached a number? |
|---|---|---|---|
| **X-02** | `research/exp_flex.md` carries six `[AGG]` statistics computed by the agent from the summary table of the FLEX data report (284 tests of every partition); `research/lit_droplets.md` carries `[F]` trends from FLEX papers read in full | 13 and 14 Nov, research agents, then the author | Not to my knowledge: every droplet law cites a train table. The ranges were in the author's context when he chose which laws to write |
| **X-04** | The spherical-flame review read in full the CFI-G cool-flame paper (Kim et al. 2023), Waddell's thesis and the CFI-G final report; it reported one count over the whole campaign, which is withheld from the saved text. It also noticed files in a shared scratch folder that looked like per-test data and says it did not use them | 14 Nov, research agent | No number that I know of. The constants of the growth law should be traced to their test list (1.3 of the sources page) |
| **X-07** | The blend weights (1 : 1) were typed after the author had seen the aggregate validation scores of all four members; BASS D2 was written after the validation result of D1 | 14 Nov, author | Equal weights are what one types knowing nothing, and no other pair was tried; D2 is declared validation-informed and was refused |
| **X-08** | Accidental reads by agents: one listed the run ids of validation forecast artifacts and opened none; three printed rows of BASS tests outside their lists from the experiment table (B1 to B5; B1 and B2); the flame-spread reviewer's search results twice showed one-line outcomes of BASS-type tests and two table rows; the ACME analysis agent read the line of the clock card that says which fuel and pressure combinations the validation sessions hold; the flame-trace agent listed the name and size of `FLEX-238_LLUVData.txt` (a validation or test file) without fetching it, and the file names of the sample folder; the raw-signal agent printed twelve rows of the chamber calculator before filtering (cabin-air and vent operations and flow-controller checkout points, no science test) | 14 Nov, agents | No. None is in a report as a value, and the author did not see the values |
| **X-09** | The author read the **inputs** of validation runs: conditions of all four families (which is what the shift map is), and the inputs of the seventeen BASS validation track files (oxygen, sample label, band width, restrictor, igniter interval, calibration rows, the set of fan values) | 14 Nov, author | Inputs are allowed. But knowing where validation sits steers which regimes get attention: the hold scenario for inverse flames (D-26) addresses exactly the region where the validation session lies. It was built from train evidence (M-09) and is declared here |
| **X-10** | Pretraining: the author recalled that the first cool flames of CFI-G were reported around 23 June 2021, a validation session date, and reasoned with it | 14 Nov, author | No number was typed from it |
| **X-11** | A heptane candidate that was meant to stay unlooked-at (D4, flags only) was a member of the evaluation group when the results document was regenerated, so it was scored on validation | 14 Nov, author's mistake | Its row was removed by script without being read and the group was rescored. The score existed on disk for some minutes |
| **X-12** | The private data root holds the full flame traces of validation runs; `runs/dev/nasa/PSI-159/` holds "as run" files of validation tests | 14 Nov, author | Not read: the author uses them only through the prefix builder and through a tool that prints counts. Nothing technical enforces it |
| **X-13** | The physics dossiers of the fifth generation are told not to read per-test results; three of their literature helpers ran on a different model than the owner asked for (claude-opus-5-5 instead of claude-fable-5-2) | 14 Nov | Not an exposure; a process deviation, recorded in [registers/agents.md](registers/agents.md) |
| **X-14** | The BASS physics dossier of the fifth generation (claude-fable-5-2) read section F2 of `exp_bass.md` and the matching rows of its table H before the author's caution reached it. It also proposes to floor the prior of the class "carries a flame" at an aggregate of the validation burns (about two of sixteen did not spread), which it took from the protocol's description of the validation tracks | 14 Nov, agent | Not yet: nothing of it is typed. Its statement about 16 to 17 % oxygen may be anchored on F2, as it says itself. A class prior floored at a validation aggregate would be a validation-informed number; I do not intend to type it, and if anything like it is typed it will be declared |
| **X-15** | The n-heptane physics dossier (claude-fable-5-2) read one aggregate remark about measured ISS cool flames in a theory paper (standoff about 3, rate about 0.5 mm²/s) and skipped that paper's comparison with ISS data; it used a train run whose atmosphere is an unresolved conflict (H19H101) as its only evidence for a helium cool flame at 1 atm; several property values are from the model's memory and marked so; one constant (the cool-flame oxygen limit, 0.119 atm) is "algebra on two train points", which is as close to fitting as a hand calculation gets | 14 Nov, agent | Nothing typed yet. The memory-sourced properties must be looked up before any is typed; the two-point constant, if used, is a train-read number with a range (0.10 to 0.125), not a derived one |
| **X-16** | A literature helper of the inverse-flame physics dossier (claude-fable-5-2, web only) opened the CFI-G cool-flame paper (Kim et al. 2023) through a mislabelled search result; its reader returned the oscillation and extinction times of one representative flame, a range of cool-flame durations and the ranges of the test matrix. The helper says it discarded them and kept only model equations, the fitted `ρD` and qualitative statements; its report to the dossier's author contains none | 14 Nov, agent's helper | Not to my knowledge: the dossier's oscillation finding rests on train radiometer traces. The same paper had been read in full by the fourth-generation review (X-04) |
| **X-17** | The literature helper of the normal-flame physics dossier (claude-opus-5-5) searched the text of the CFI-G cool-flame paper for headings; lines from its results section and fragments of its Table 1 appeared in its tool output, and one keyword search of a conference paper surfaced a headline finding of the campaign. It passed none of that on. The dossier's author used published aggregate flame radii from the repository's own documents once "as a soft cross-check during reasoning" | 14 Nov, agent and helper | No number in the report rests on them, by its own statement |
| **X-18** | **Inputs of one test encode the outcome of the test before it.** Tests of one chamber fill share their atmosphere: the ambient oxygen (normal flames) or fuel (inverse flames) tabulated before a test is what the previous test left, so the drop from one test to the next measures how much the previous flame burned. Anyone who reads the conditions of consecutive validation tests can therefore estimate validation burn times. The author and three agents read validation conditions (X-09); the normal-flame agent noticed the channel and says it did not compute it | noticed 14 Nov | Not used: no forward sees another run, and nobody computed the differences. It bounds what "inputs only" means, and it applies to FLEX fills in the same way |
| **X-19** | Small reads by the independent checkers: the heptane checker displayed the first four rows of the flame-end table, which are methanol train runs; the inverse checker opened the script parameter file named for train test 21196M2_1 and found the script name 21196M3 inside, used nothing of it beyond the name; the methanol dossier's helpers saw a search-result abstract of the 2014 FLEX overview | 14 Nov, agents | No |
| **X-20** | The validation look of the fifth generation scored droplets on the private copy of the data root against a vault of their own (`runs/dev/holdout_ext/`), which the evaluator's deposit command wrote from that copy's validation runs. Depositing and scoring handle validation continuations inside the evaluator; the author read neither vault, only the aggregate tables. A first attempt to deposit into the shared vault was refused by its write-once check and changed nothing there | 14 Nov, author | No. It is a departure from the shared procedure that the evaluation lane has not seen |
| **X-21** | `tools/wsp/analysis/cmp_split.py` reads the per-case validation scores inside the evaluator's comparison file in order to split the burner-flame table by data root, and prints aggregates. The comparison files (`runs/dev/cmp_validation_*.json`) have always held per-case validation scores; the author has not read one per case. To write the script he printed the field names of one such file and one case id (a hash) | 14 Nov, author | No |
| **X-22** | To explain why the carried inverse model scores 0.161 in this look and 0.137 in the earlier table, the author compared the sealed forecasts of the carried releases with those of their carry releases on the validation prefixes (model outputs on validation inputs; no outcome): identical. The first run of the look had stopped half-way; the burner-flame comparison files it left were not read and were written again by the second run | 14 Nov, author | No |
| **X-23** | The transfer test showed the author aggregate scores of the frozen models over the train and validation days of Flame Design: one aggregate look at its validation days, before any model has been written on it. Its test days were not measured or scored. The agents that surveyed, catalogued and scored Flame Design, CFI and the other new investigations saw per-test outcomes of them; the outcome values are kept in files the author does not open (`tests_outcomes.jsonl`, the endpoint and case files of the transfer test) | 14 Nov, author and agents | No. Any later model written on Flame Design inherits this look |
| **X-25** | To register the Saffire transfer test the author read the condition rows of the 13 Saffire samples in the test catalogue (material, flow speed and direction, oxygen where it is a condition, igniter power and time), the transcribed Saffire II Readme table (conditions only), the file listings of the three flights (names and sizes), and the BASS reader and scorer code. No frame, sensor series, outcome cell, `tests_outcomes.jsonl` or protected page was opened. Earlier sessions: no Saffire result is recorded as read in this ledger; `ASTRA_PROPERTIES.md` cites a Saffire VI methods report (another campaign) and states that no Saffire burn result was used. The published BASS-II results of X-01 are of another campaign | 15 Nov, author | No: no number is typed in this test. The look itself spends the three held flight groups (D-49) |
| **X-26** | Saffire measurement and look. The measuring agent saw every frame of the four SIBAL burns and, after a full pass that included the sheets, changed one frame-handling rule (the brightness cutoff that marks backlit frames, and skipping two frames either side of them); its luminous threshold and geometry did not change, and it had seen no forecast. Its report to the author mentioned frame-rate drops "near the end" of Saffire I. The check agent saw 61 overlay images. The author then read the aggregate tables of the one scored run. From now on Saffire I, II and III are development data for the fabric-flame family | 15 Nov, agents and author | No: no weight was typed. Any later fabric model inherits this look |
| **X-27** | FLEX-035 (`PSI-69:181F001:2009-11-27`, n-heptane **validation**) was studied per run. For the comparison with Farouk and Dryer its 0 to 38 s trajectory was matched to a published figure and diagnosed (hot flame lost near 7 to 9 s, cool flame at 0.37 to 0.40 mm²/s, stall at 1.27 mm; [RESULTS_POINT_METRICS.md](../RESULTS_POINT_METRICS.md)); the identification of the droplet of Xi et al. had already read the first 10 s of every heptane run, validation included. The author of `flex_heptane.wsp2` D3 read that diagnosis before writing the candidate; he did not open the series. The same file prints aggregate point metrics of the spent heptane test and the nRMSE of one test run (H05H103) | 15 Nov, the comparison lane and the author of D3 | The diagnosis chose which law was looked at again (the cool-flame rate). It set no number: D3's five statements were typed on 14 November for `flex_heptane.wsp5` P0 and are copied unchanged, and the train tables carry them. The three validation cases of FLEX-035 are left out of D3's registered look ([PROTOCOL.md](../PROTOCOL.md) 13.7). The heptane validation days now have one per-run look |

The exposure records of the candidates (`meta.json`, collected in
[registers/candidates.json](registers/candidates.json)) were written before this ledger. Where the
ledger says more (X-01 for `bass_front.wsp`), the ledger is right and the record is incomplete. A
committed `meta.json` is part of a release's identity and is not edited; the correction is this
page.

## 7.2 How worn the validation partitions are

**Astra access disclosure X-23 (14 November):** a delegated methanol checker selected raw
files using train-record aliases, loaded them with `read_text()`, and displayed their
first four lines (header and up to three series rows). The matcher did
not establish a collision-free identity. After noticing the historical FLEX alias issue,
it stopped raw access. Path-only reconstruction lists 68 droplet and 64 UV trace candidate
paths and no FLEX-238 basename, but does not certify all source identities. Protected
outcome access is **not established**; absence of exposure is also not certified. F-004
records the method and path-only receipt. No uncertain raw numeric material was sent to
the model author or used in weights; only independent analytic property calculations are
retained. The checker writes no weights. This is separate from the historical X-19 incident.

**Astra source disclosure X-24 (14 November):** the property/transport collector opened
metadata, the full abstract and introductory/setup pages 1–2 of Zhao et al.,
*Concurrent flame growth, spread and extinction over composite fabric samples in low
speed purely forced flow in microgravity*, [NASA NTRS 20150008962](https://ntrs.nasa.gov/citations/20150008962).
The abstract contains a cohort aggregate. The source is class C, corrected from the
collector's initial class B label. No results section, per-test table or figure was
consulted, and the aggregate was not passed to the author or used as a coefficient.
The source-reading ledger retains the exposure; no claim of fresh blindness is made.

Every look is in [PROTOCOL.md](../PROTOCOL.md) (sections 4 and 7 to 10) and in the result files of
`runs/dev/RESULTS_*.md`. By my count of those sections, the fifth generation included:

| Family | Aggregate looks | Per-run look | What decided on them |
|---|---|---|---|
| FLEX n-heptane | about six, plus the mistaken scoring of X-11 | none | The third generation adopted; the fourth and the fifth refused |
| ACME normal | about eight; the fifth-generation look gave two tables of one release (with and without the planned stop) | none | The third-generation hinge law adopted; the growth clock refused on a tie; the blend adopted; the committee adopted |
| ACME inverse | about eight; the last gave two tables likewise | none | Two revisions of the hinge-law spec vetoed; the growth forward adopted; the blend refused; the committee adopted |
| BASS | three | none; the tracks are sealed | The first look was blind; D2 refused on the second; the fifth-generation P1 refused on the third |

Each family has two validation sessions per configuration (ACME) or seven to nine days (FLEX). After
this many looks the validation numbers are development numbers, and a difference of 0.004 in a
Brier score or 0.6 µm in a CRPS, on which adoptions and refusals turned, is within what the choice
of sessions alone could produce. The paired intervals in [RESULTS.md](../RESULTS.md) say the same.
The test partition is the only clean measure and has not been read by anything on this branch.

## 7.3 Mistakes, in the order they were made

1. Phase 1: revising on per-run validation feedback (X-05). Stopped in phase 2.
2. Fourth generation: narrowing the droplet bands because the tracker is precise. The band has to
   cover the error of the law on a new session, which the tracker's scatter says nothing about.
3. Fourth generation: a helium law typed as `exp(3.85 x_He)` from runs at 30 to 35 % helium with
   nothing bounding it at the hot burning rate. Wrong by inspection for higher fractions.
4. Fourth generation: three changes bundled in one candidate, so that validation could not say
   which one failed; the flags-only candidate was a second look that cost another use of the
   partition.
5. BASS: a boundary typed as a line from one burn on each side of it.
6. Regenerating the results document with a candidate in the group that was not to be scored (X-11).
7. Fifth generation, first camera rule: a per-frame detection probability with a prior floor, which
   could not overrule a class prior of 1e-8 coherently. Replaced before any score was taken.
8. Fifth generation: the committee roots type the flame-onset delay as 2.4 s after the ignition
   stamp, but the stored `programmed_stop` is the bare stage sum, for which the right offset is 0.4 s
   (M-10). Found by the author and corrected before anything was registered ([08](08_in_progress.md)).
9. Reading BASS-II cohort results into the experiment dossier a day before deciding that they must
   not be read (X-01).
10. Uneven rechecking of delegated analyses: "the author rechecked what he acted on" was true of the
    claims that became numbers and was sometimes a spot check ([03](03_datasets_and_measurements.md)).
11. Fifth generation: a train session table read with the candidate and the carried model in the
    wrong order, which reversed its sign. Noticed from a count that could not be right and rerun
    before anything was typed from it.
12. Fifth generation: the registered look was started without having checked that the evaluator had
    a vault for the private copy of the data root. The first run stopped half-way; nothing of it was
    read (X-22). A dry run of the tooling on train would have found it.
13. Fifth generation: this dossier's handoff page and four of its pages still said "not registered,
    not validated" for two commits after the candidates had been registered. Corrected with the
    outcome.
14. Freezing the groups of Flame Design: the first run of the script put normal and inverse flames
    into one stratum (it tested a field every row carries). Seen in the output, corrected and rerun
    before anything was committed or used; no outcome had been seen, so the two assignments could
    not be told apart by their results.
15. The first list of what the server would store counted rows of NASA's listings; the listings
    repeat 5,701 rows. Found by the file catalogue an hour later and corrected before the list was
    merged into the application's repository.

## 7.4 Limits that no revision removes

- **Droplets.** Which side a droplet near the critical diameter falls on, the exact end of a cool
  flame, disruption, and drift out of view cannot be read from a diameter prefix. A third or more of
  the scored error is tracker artefact in the target itself.
- **Burner flames.** With conditions alone as input, about 15 % of normal and 40 % of inverse
  outcomes on train were decided by things outside any model's inputs (operator and script stops,
  the limit setting, the fibre, igniter dwell). The score window is 6 to 120 s: a flame out at 123 s
  and one held for five minutes are the same case.
- **Flame spread.** Four train burns, one oxygen range, one sample width. No run has a length
  calibration; the model answers in sample widths. It refuses on stored shared runs, which carry
  neither a width in metres nor the fan history.
- **All.** The bands are assumed, not inferred. No release is NASA-trained and none claims a
  posterior. Every carried model is data-informed; only the W0 candidates and the reference
  baselines are zero-shot.

## 7.5 What a reviewer cannot see

The author's and the agents' session transcripts are local files on the author's machine and are
not in the repository. What they contain that matters is reproduced here and in the saved reports;
the briefs given to the agents are summarised in PROTOCOL and in the headers of the reports. If a
review needs the exact brief of an agent, ask for it in a finding and it will be added to the
dossier.

## 7.6 Astra implementation limits and process corrections

All Astra R1/joint candidates are data-informed. The complete exposure travels with
their patches; `ASTRA_PROPERTIES.md` states property-domain and reference-state limits.
No new validation look is part of this work. The alias uncertainty X-23 remains unresolved
and uncertain raw material is excluded rather than retroactively certified safe.

F-007 confirms an inherited BASS event row can decrease after a sample reaches the end.
Only the new active route changes that mechanism. Its joint position law combines
authors' distributions; its terminal events use a new coherent interpretation on both
members, not a mixture of the old invalid-CIF interpretation. Historical releases stay.

F-008 finds train filtering occurred after generic family-wide domain loading. The new
train route filters identities first. This reduces semantic access, but mixed JSONL files
are still streamed to identify rows; it is not filesystem isolation and establishes no
new fact about what earlier authors personally saw.

During implementation, a delegated `rustfmt` invocation recursively reformatted frozen
modules under the wrong edition. The changes were detected by diff, restored exactly
from the unchanged checkpoint, and excluded from release construction. A second status
check caught remaining formatter changes before the final gates. No old release was
re-identified. Formatting is now restricted to owned files with child traversal disabled.
The initial synthetic tests also caught an invalid physical-tensor fill declaration and
test-fixture/roundoff mistakes; these were fixed before any candidate score was read.

F-009 identifies same-model key collisions in generic paired summaries; Astra reconstructs
comparisons by full release ID. F-010 identifies case metrics copied before event scoring;
the exporter now copies final metrics and cluster labels. Original train receipts remain,
and a separate replay reproduces every aggregate exactly. No weight was selected from it.

The initial exposure metadata failed SourceId syntax because identifiers contained spaces.
Syntax was corrected before patch/release creation, retaining exposure and all coefficients.
Later Git status noise was LF/CRLF conversion, not continued numerical source drift.
No line-ending-only file change belongs in the checkpoint.

The all-NASA instruction adds frozen groups before author outcome access, separate blind
measurement and prospective registrations. First-extension train results do not qualify
that larger coverage. X-23/X-24 remain disclosed; new work does not erase earlier exposure.

## 7.7 All-NASA roles and incidental exposure

These records do not erase X-23/X-24 and do not qualify any new family. The
existing ten Astra releases remain data-informed and unchanged. No all-NASA
numerical candidate had been written at events X1-X6. The later independent
numerical W0 kernel is frozen; the coordinator remains data-informed.

| Record | What reached which context | Disposition |
|---|---|---|
| ASTRA-ALL-X2 | A methods-targeted search returned an unsolicited per-test BASS-II report snippet to the measurement designer; report and secondary page not opened | Result/run identity withheld from main; designer ineligible as fresh blind measurer or clean-room author; report locator and exact scope in ASTRA_MEASUREMENT_DESIGN.md |
| ASTRA-ALL-X3 | Later generic theory searches returned terrestrial BRE agreement snippets and a cited-by expansion containing an ISS spherical-flame cohort summary to the theory analyst | No papers opened from those hits, no numbers reproduced or used; subsequent reading limited to screened theory routes; analyst already exposed |
| ASTRA-ALL-X4 | Main author's read of the end of PROTOCOL.md to inspect new section 11.2 also returned 11.1's trial-method timing summaries and methodological outcome statements | No per-run identity/outcome pair or score table was opened; trial membership is unresolved in this context. Treat as development exposure, not a clean source or W0 premise; no new weight derived from it |
| ASTRA-ALL-X5 | The property auditor read the expressly permitted ASTRA_PROPERTIES.md, which mixes property limits with earlier TRAIN-derived model-choice discussion | Property auditor is not clean-room and authors no model numbers; only independently sourced physical-property facts enter the audit |
| ASTRA-ALL-X6 | The owner pasted Fable's registered Flame Design aggregate results and requested remediation; main then read the aggregate report through its recorded-method choices | Main is now explicitly informed by this registered transfer look, including validation aggregates. No per-case validation file or held/test outcome was opened. Existing Astra weights remain unchanged; this context cannot author a numerical W0 candidate |

New-investigation outcome access by model authors still requires committed,
hash-pinned Fable groups and complete TRAIN-only dependencies. Held groups and
unknown identities stay closed. The owner expressly keeps
`runs/dev/data_all/work/atlas/tests_outcomes.jsonl` closed. Page extraction
and protected annotation do not authorize their contents to reach an author.
No protected page count by outcome, tracker residual, failure pattern or label
distribution is used as informal model feedback.

The separate roles run in a shared checkout with procedural/tool restrictions,
not independent OS security principals. This is stated rather than claiming
filesystem isolation. The fresh measuring role receives instrument/pixels only;
the method designer's full research context is not its prompt.

### Continuation and transfer disclosures

- Reused blind operators stopped under the frozen familiarity clause after
  recognizing the acquisition from their own earlier task frames. Their receipts
  were retained and fresh operators completed the seven-frame continuation. No
  external test identity, model, condition or peer measurement was reported.
- The owner supplied TRAIN discrepancy allegations and registered aggregate
  findings again in the whole-goal instruction. These are claims to verify, not
  independent proof of their causes. The clean-room numerical author did not
  receive them. The coordinator remains data-informed.
- Repository transfer copied bytes without displaying protected payloads. The
  retained data store and sealed paths have exactly the same role restrictions.

- A fresh discrepancy checker followed AGENTS into the historical Iancu handoff
  and encountered aggregate model summaries before any source replay. That
  checker was retired from the independent role; no numerical source processing
  or implementation occurred. Its exposure receipt is retained privately.

- ASTRA-ALL-X7: the already data-informed six-family error-budget analyst read
  the expressly allowed `flex_heptane_d2_train_anatomy.md`. Its proposal 2 embeds
  a historical unseen-session coverage assertion. That assertion was excluded
  from evidence and ranking; no validation file, per-run validation outcome or
  held/test data was opened. The analyst authors no weights and is not a
  clean-room numerical author. Source extent is in the error-budget report.
