# 1. Documents: what was read, how far, under which rule

Every document set behind the models, who read it, how deep, and what it was used for. The
per-source lists (author, title, link, "full / abstract / cited only") are the reference sections of
the files named below; they are not copied here. What is here is the classification, the places
where it was applied unevenly, and the sources a reviewer should open first.

## 1.1 The classes

| Class | What it is | Rule | Risk it guards against |
|---|---|---|---|
| **A. Free science** | Theory, numerical models, property data, experiments on other fuels, drop towers, other campaigns | Read in full | None for the partitions; ordinary risk of misreading |
| **B. Campaign, general** | Apparatus, procedure, instruments, data reduction of the modelled campaign; file-format documents | Read | None if the reader stops at methods |
| **C. Campaign, mixed** | Papers and reports that hold mechanism or correlations *and* per-test results of the modelled series | Read for mechanism and correlations; skip per-test tables, figures and paragraphs; say what was skipped | A per-test value of a validation or test run entering a typed number |
| **D. Per-test results** | Tables or figures that give outcomes of individual tests of the modelled series | Not read | The same, directly |
| **E. NASA archive files** | The PSI datasets themselves | By partition: [02_data_classification.md](02_data_classification.md) | Reading a protected run |
| **F. Method sources** | Weight-space programming: the MIT work, the Legora experiment, replications | Read in full | None |
| **G. Pretraining** | What the author model already knew | Cannot be audited; declared in every `meta.json` | Published FLEX and CFI-G results may be in it |

## 1.2 Document sets

Reader models are the ids recorded on the agents' own messages ([registers/agents.md](registers/agents.md)).
"Saved" says whether the file is the agent's text verbatim or was reformatted when saved.

| File | Date | Reader | Rule in force | Depth, by the file's own statement | Used for |
|---|---|---|---|---|---|
| `research/lit_droplets.md` | 14 Nov | agent, claude-opus-5-5; verbatim | C and D as above | 13 sources in full (NASA NTRS papers, Williams and Nayagam 2019, the 2014 overview, the methods sections of the data report); the rest as abstracts or citations, marked. Numbers tagged `[T]` theory, `[N]` numerical, `[F]` FLEX correlation or trend, `[O]` other experiments, `[E]` the agent's estimate | Choice of laws in the fourth-generation droplet candidates (all refused); the reading of the helium after-burn as a cool flame |
| `research/lit_spherical_flames.md` | 14 Nov | agent, claude-opus-5-5; verbatim except one withheld count | C and D | Four sources in full: Santa et al. 2007, Kim et al. 2023 (the CFI-G cool-flame paper), Waddell's 2023 thesis, the NASA ACME and CFI-G reports. Irace 2021 and 2023, Tse 2001, Mills and Matalon, Nayagam 2019, Christiansen 2003 as abstracts. Tags `[T]`, `[N]`, `[ISS]`, `[DT]`, `[D]` derived by the agent | The growth-clock forward `flame3.rs`: quasi-steady radius, critical flux, the constants |
| `research/lit_flame_spread.md` | 14 Nov | agent, claude-opus-5-5; verbatim | C and D, plus: no measured result of the BASS-II concurrent SIBAL tests from any source | 21 numbered sources; a list of nine sources or parts "deliberately not read for results" | The front model `front.rs`: linear spread law, overhang, quench boundary of a thin fuel |
| Physics dossiers of the fifth generation: `research/phys_bass.md`, `phys_flex_heptane.md`, `phys_flex_methanol.md`, `phys_acme_inverse.md`, `phys_acme_normal.md` | 14 Nov | agents, claude-fable-5-2, high effort; saved verbatim. Three of them used literature helpers on claude-opus-5-5, launched before the instruction to use Fable reached them | C and D; each must list every source with the depth read | Each report ends with a source table. Known lapses: X-14 to X-17 and X-19. Four of the five have an independent check on train (M-19); property values are being looked up in standard sources | Mechanistic forwards of the fifth generation; [10_extension_map.md](10_extension_map.md) |
| `research/properties_verified.md` and the tables in `research/properties/` | 14 Nov | agent, claude-fable-5-2, high effort; verbatim | A only: it was forbidden to open any experimental file | NIST Chemistry WebBook fluid and thermochemistry pages, Marrero and Mason 1972 (two tables from page images), McGivern and Manion 2012 (full), Tang et al. 2014 and 2015 (sections), Kadoya et al. 1985 (two tables), the TNF radiation page, the NIST ITS-90 thermocouple tables; several handbooks could not be reached and are listed | Every property value the physics dossiers quote, 374 rows with source and verdict; the constants of the self-cooling law of the methanol candidate |

Agents that did not read documents (train audits, measurements, code) are in
[03_datasets_and_measurements.md](03_datasets_and_measurements.md).

## 1.3 The mixed sources: open these first

A reviewer who wants to test the classification should start with the class C sources that were
read in full, and compare what they contain with the typed numbers.

| Source | Where read | What it holds beyond mechanism | What entered a model, to my knowledge |
|---|---|---|---|
| Dietrich et al., NASA/TP-2015-216046, the FLEX data report (first 284 tests) | `exp_flex.md` (whole report, aggregates computed from its summary table); `lit_droplets.md` (methods sections only) | Per-test tables for runs of every partition | Procedure, time origin, instrument resolutions. The `[AGG]` ranges were read by the author before the phase-2 weights. No typed number cites them; the edits files cite train tables. A reviewer can check each droplet law against `runs/dev/analysis/` (train only) to see whether train alone supports the typed value |
| Dietrich et al. 2014 (Microgravity Sci. Technol.), Nayagam et al. 2013, 2014 | both FLEX files | Figures of individual FLEX tests | Regime descriptions, the cool-flame extinction scaling used by the zero-shot heptane W0 |
| Kim et al. 2023, Proc. Combust. Inst. 39 | `exp_acme.md`, `lit_spherical_flames.md` (full) | Results of CFI-G tests, including the cool flames | That cool flames follow hot flames labelled `SE`, only for n-butane at pressure; no number |
| Waddell, PhD thesis 2023 | `lit_spherical_flames.md` (full) | Near-limit spherical flames, some from CFI-G | The growth law and critical-flux picture. **Check whether the constants `ρD = 0.0334 g m⁻¹ s⁻¹`, critical fuel flux `0.2 g m⁻² s⁻¹`, `D = 28 mm²/s` were derived from ethylene flames of the Flame Design experiment (another campaign, as the review's `[ISS]` tag suggests) or from CFI-G tests.** I typed them as published and did not trace them to their test list |
| ACME/CFI-G Final Report (2022), ACME Engineering Implementation (2022) | `exp_acme.md`, `lit_spherical_flames.md`, the raw-signal analysis (engineering report only, as a channel dictionary) | The final report discusses the campaign's results | Script logic, extinction detection, burner thermocouple, the two limit values quoted by sources |
| NASA/TM-20210011385, the BASS-II Summary Report; ICES-2019-101 | `exp_bass.md` | Per-test tables and the flammability boundary measured on the SIBAL samples | See X-01: the quench boundary of the BASS model |
| The 2012 and 2013 ASGSR posters of the earlier BASS campaign | `exp_bass.md` | Results of PSI-26 burns (outside every partition): the linear spread correlation, 2.55 mm/s at 10 cm/s | The linear form of the spread law; legitimately, since PSI-26 is external |

## 1.4 Published numbers that were typed as they stand

Numbers a model carries that come from a document and not from a train table. The edits file of
each candidate gives the reason beside the statement.

| Number | Model | From | Kind |
|---|---|---|---|
| `ρD`, critical fuel flux, diffusivity (above); the form `r_ss = m_tot / (4π ρD L)` | `acme_*.wsp3` baseline-theory and D1 | `lit_spherical_flames.md` | published theory and its measured constants |
| Quench velocities of tissue paper, 1.0, 1.5, 1.8, 2.7 cm/s at 21, 19, 17, 15 % oxygen | `bass_front.wsp` D1 (then shifted by 3 % oxygen from two train burns) | `lit_flame_spread.md` | numerical model and drop-tower work on another material |
| Areal density of the fabric, 18.05 mg/cm² of which 13.54 cotton | BASS card, scaling arguments | `lit_flame_spread.md` | material property |
| Type-K thermocouple table, cold junction 23.5 °C | the raw-signal measurement only, not a model | NIST table; the cold junction is an assumption | instrument |

## 1.5 Not read, on purpose

- The sources listed under "Sources deliberately not read for results" in `lit_flame_spread.md`
  (Zhao et al. 2017, the 2013 national-meeting paper, Olson et al. ICES-2019, Ferkul et al. 2013,
  Link et al. 2015, and the BASS parts of four more). Note that `exp_bass.md` had already quoted
  ICES-2019 and the summary report the day before: the rule came after the exposure.
- Per-test tables and figures of the FLEX ISS droplet tests and of the CFI-G tests, from the fourth
  generation on.
- The ACME session logs and daily test-point logs of PSI-159 (they mix train tests with others).
- Anything about the test burns of BASS-II. The author of the front model must not measure them.

## 1.6 Pretraining

**Astra additions (14 November).** Source-by-source primary links, reading extents and
limits are in `../research/astra_weight_space_research.md`,
`astra_physical_properties_and_transport.md`, `astra_uncertainty_and_freedom.md`,
`astra_research_analysis.md` and `astra_physics_analysis.md`. The last two are independent
analyses explicitly assigned to `gpt-6-astra` at the user's request; the earlier three
are collector reports with inherited model selection, not independently attested model
build IDs. No collector authors candidate weights. Sources supporting a coefficient are
checked again by the author before transcription; reading a source report is not claimed
as opening its cited primary source. See F-004 for the separate raw-file access uncertainty.

The author model may hold published FLEX and CFI-G results from pretraining; this cannot be
measured, and every `meta.json` says so. One concrete instance is known: while thinking about the
n-butane validation session of 23 June 2021, I recalled that the first cool flames of CFI-G were
reported around that date and reasoned with it about whether those flames cycle between hot and
cool. No number was typed from it, and no candidate carries a statement about that session. It is
recorded as X-10.

## 1.7 How to redo this

1. For each class C source in 1.3, open it and mark what it gives per test. Compare with the typed
   numbers of the model it informed (`configs/models/**/edits.wsp`; the reason for each number is
   the comment above it).
2. For each typed number that cites train, rerun the train-only table it cites
   ([09_reproduce.md](09_reproduce.md)) and see whether train alone supports the value and its
   stated precision.
3. Where a number is supported by train only loosely and sits close to a published per-test or
   whole-campaign value, file it ([review/](review/README.md)). X-01 is the case I found myself.

## 1.8 Astra implementation sources

[ASTRA_PROPERTIES.md](ASTRA_PROPERTIES.md) is the coefficient-level source ledger: NIST
WebBook correlations and property tables, Tang's evaluated diffusivities, Lemmon/Jacobsen,
the Methanol Institute sheet, the Saffire VI material-description excerpt, and a separately
executed CoolProp 8.0.0 reference-state check. These are class A property/theory or
different-campaign apparatus sources; none supplies a modeled-series per-test outcome.
The ledger distinguishes primary pages opened by the main author, collector queries,
failed accesses, and shared underlying experiments. [Astra's data/input audit](../research/astra_data_and_input_audit.md)
records the nine classification dispositions and why no new archive field or outside
campaign score is claimed.

## 1.9 All-NASA continuation sources

`ASTRA_FIRST_PRINCIPLES_ALL_NASA.md` identifies the main author's direct
Cantera 3.1 and NIST uncertainty reads and their limited extent. The companion
`research/astra_all_nasa_dimensional_basis.md` and
`measure/ASTRA_MEASUREMENT_DESIGN.md` retain their independent source ledgers,
partial reads and incidental exposures. Their proposals are not automatically
verified numerical sources. Page 07 section 7.7 records the contexts that must
not be reused for clean-room authorship or fresh blind measurement.

Acquired document/trace bytes are hashed in a non-modelling role and remain
unread by this author until screened and routed. A successful fetch, text
extraction or page count is not a claim to have read or interpreted that source.
