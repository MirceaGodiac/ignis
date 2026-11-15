# 2. The NASA data: how each file was classified

A forecast model may use what is known before a run and what is observed up to the cutoff. Whether
a model is honest therefore depends on how each file and column was classified. This page gives the
classification, who made it, and the calls a reviewer should make again.

## 2.1 Datasets and partitions

| Dataset | Experiment | Manifest | Used as |
|---|---|---|---|
| PSI-69 | FLEX droplet combustion (methanol, n-heptane) | `data/manifests/PSI-69.listing.jsonl` | train, validation, test |
| PSI-159 | ACME CFI-G spherical burner flames (normal, inverse) | `data/manifests/PSI-159.listing.jsonl` | train, validation, test |
| PSI-25 | BASS-II flame spread; the cohort is the concurrent-flow SIBAL fabric tests | `data/manifests/PSI-25.listing.jsonl` | train, validation, test |
| PSI-26 | The earlier BASS campaign (2011 to 2013) | `data/manifests/PSI-26.listing.jsonl` | outside every partition: external data |

The archive is not downloaded whole. Files are fetched one by one from the `stable_url` of the
listing, with a generic client name.

| Family | Split file (`data/splits/`) | Train / validation / test runs | Groups | Frozen |
|---|---|---|---|---|
| FLEX methanol | `flex-660b7d9ede629b30.json` | 68 / 24 / 22 | day-sessions, 27 / 9 / (not counted here) | by the shared split rule, before this lane |
| FLEX n-heptane | `flex-c43bbe3580ffce2a.json` | 49 / 15 / 16 | day-sessions, 22 / 7 | by this lane on 13 November, from metadata only, before any heptane outcome was read (`wsp-dev heptane-cohort`); 80 free-floating runs of 113 heptane records |
| ACME normal | `acme_normal-c77bc5baee98d3e3.json` | 81 / 28 / 20 | test days, 7 / 2 | shared split rule |
| ACME inverse | `acme_inverse-747032c921c8c8f2.json` | 73 / 24 / 34 | test days, 7 / 2 | shared split rule |
| BASS | `bass-4083156d5b18dab4.json` | 4 / 17 / 4 | tests | shared split rule |

Partition ids are read through `tools/wsp/analysis/lib.py` (`partition_ids(family, "train" |
"validation")`); that module has no way to list the test partition. A whole day is in one partition,
so no chamber fill is split.

**Where the validation days sit** (inputs only, [feedback/shift_map_validation_inputs.md](../feedback/shift_map_validation_inputs.md)):
normal flames 2021-05-17 (30 % propane, 1 atm) and 2021-06-23 (30 % n-butane, 0.5 to 2 atm);
inverse flames 2021-08-23 (propane ambient near 20 %, 0.5 and 1.1 atm) and 2021-09-22 (n-butane
ambient at 5.7 %, 3.3 atm). None of the 24 inverse validation runs is in a condition cell that train
fills. FLEX validation days are spread through 2009 to 2011.

## 2.2 Classes

| Class | Meaning | May a forward read it? |
|---|---|---|
| **P** prospective condition | Set or known before the run starts | Yes, when the model's `ProspectivePolicy` lists it |
| **C** control | A commanded action with its time; known at the cutoff if it started at or before it | Yes (`t_start_s <= cutoff`); BASS is the exception described in 2.5 |
| **O** observation | Measured during the run | Only values with a time stamp at or before the cutoff |
| **T** target | What is scored | Never |
| **R** post-run fact | Recorded or derived after the run | Never. Read by the author for train runs only, as measurement |
| **X** not used | | |

## 2.3 FLEX (PSI-69)

| File or column | Class | Notes and who decided |
|---|---|---|
| `…_DropletData.txt`: `DropletTime_sec`, `DropletDiameter_mm` | O up to the cutoff, T after it | Shared ingest (the data lane). Time zero is the moment the igniters are energised (data report; the train files agree). One train sheet says its recorder missed the start (C01H101) |
| `…_DropletData.txt`: `ZeroedDropletXpos_mm`, `ZeroedDropletYpos_mm` | X | In the file, not ingested. I did not add them: it would change the input hash of every FLEX prefix (D-16) |
| `…_TestSheet.txt`: atmosphere (O2, diluents, pressure) | **P, with a caveat** | Several droplets are burned in one chamber fill; the tabulated atmosphere is computed after the fact from the fill and the combustion history. It does not depend on the run's own outcome, only on earlier burns, so I treat it as known before the run (C-04). A reviewer may disagree |
| `…_TestSheet.txt`: `D0 (mm)` | P in the fourth-generation forward only (`initial_diameter`); not used by the carried models | Defined as the diameter at time zero, so it is known at any cutoff. It is derived by the investigators from the same series the model reads |
| `…_TestSheet.txt`: `kavg`, `Dext`, the narrative | R | `kavg` is the investigators' fitted rate, `Dext` the diameter at visible-flame extinction, extrapolated when the droplet had left the view. Read for train runs, to adjudicate what happened |
| `…_LLUVData.txt` (UV camera), `…_ColorCamData.txt`: flame diameter against time | O up to the cutoff, **in the private data root only**; the time the trace ends is an outcome | Classified by this lane on 14 November. The shared records do not carry them. In the private root they are `UvFlameDiameter` and `VisibleFlameDiameter` observations (quantities that exist in the shared contracts) for the 156 train and validation runs, and the prefix builder admits only values at or before the cutoff. See 2.6 for the risk |
| `PSI-69_Experimental table_FLEX.csv` | P for conditions; the rest R | One row per test of every partition. Read through the ingested records, by partition |
| Radiometers | X | Not usable in 2009 to 2011 according to the data report |

Records refused or flagged (train): FLEX-114 / C11M201 (pressure and temperature stored as zero;
refused), H19H101 and H19H201 (table and sheets disagree on the atmosphere; marked
`unresolved_conflict`, refused), C08H502 (20 % CO2 in the table, none in the sheet), FLEX-124 and
FLEX-141 (one video under two test ids, both train, C-10), 240R003 (a gas bubble; its rate is not a
burning rate), recorder gaps in five series.

## 2.4 ACME CFI-G (PSI-159)

| File or column | Class | Notes and who decided |
|---|---|---|
| Table: fuel, `approx p (bar)`, `XO2 before test`, `Xf`, `mHC (mg/s)` | P | Set before the test. The ambient is tabulated before and after each test of a fill. For inverse flames the stored flow is the oxygen alone, not the injected stream (C-23) |
| Table: `Tad (K)`, `Zst` | P | Derived from the mixture |
| Table: `burn time (sec)`, `Self-ext or flow term` | T | `SE` is the end of the hot flame, not of combustion (C-16); the burn time is to 1 s; stored endpoints carry ±0.5 s |
| Table: `Peak burner T (C)`, `XO2 after test`, `TFP`, `Cool flame?`, comments | R | Not predictors. The thin filament is recorded after the test although it is mounted before it; I treat it as not known |
| `…_data.csv`: 125 Hz channels (burner thermocouple, radiometers, photomultipliers, flow readbacks, pressure) | O in principle; **not ingested anywhere**; used only to measure train tests | Channel identification is M-08. A model that read the first seconds of these channels would need them in a prefix; none does |
| `…_TP.txt`: the script timeline with stamps | R | Its stamps (`Extinction detected`, `MFCs off`) are the outcome. The `Constant flow` stamp is overwritten with the end time when the stage completes |
| `…_mfcSetpoints.csv`: commanded flows | P | Not ingested. They expose errors of the table (2.7) |
| `PSI-159_Raw Data_as run<test>.txt`: script and camera parameters | **P, by this lane's judgment (C-21)** | The file gives the stage durations of the script. Their sum up to the end of the last constant-flow stage is written as the condition `programmed_stop` in the private root by `tools/wsp/ext/acme_script.py`, for train and validation runs. Evidence that it is a plan and not a record of what happened is in M-10. The name "as run" is the reason to doubt it |
| Chamber calculator workbook, session logs, daily test-point logs | X | The calculator was read for train rows only; the logs were left alone because they mix tests |

## 2.5 BASS-II (PSI-25) and the earlier campaign (PSI-26)

| File or column | Class | Notes and who decided |
|---|---|---|
| Top-view still sequences and proof sheets; duct video | O up to the cutoff, T after it, **as measured by this lane** | No shared BASS observation of a flame front exists. Track files (`wsp-bass-front-track/1`) hold, per still, the tip and the base of the blue flame in sample widths. Train tracks are in git (`tools/wsp/bass_optical/tracks/train/`); the seventeen validation tracks were measured by two agents that knew nothing of the model and are sealed in `runs/dev/bass_front/val_sealed/` |
| Video overlay `FAN` (fan voltage × 10) with times | **C, given up to each forecast time** | The operator changes the fan during a burn. The front model is told the fan history up to the time it forecasts for, including steps after the cutoff. This makes the forecast conditional on the operator's later actions; it is declared in [PROTOCOL.md](../PROTOCOL.md) section 9 and on the card. A reviewer who holds that only the history up to the cutoff is legitimate should score the model that way; persistence, the baseline, does not use the schedule |
| Oxygen fraction of the work volume | P | Measured before and after each burn; the value before is used |
| Sample label and width class | P | The class (2 cm or 1 cm sample) is read from the measured band width in pixels, below 44 px meaning the narrow sample (`front_tools::rules`) |
| `PSI-25_Reference Data_Fan calibration data.xlsx` | P | Flow speed from the day's rows by linear interpolation; below the lowest row a straight line to the stall point, declared as an assumption |
| `PSI-25_Experimental table_BASS-II.csv`: `Air display` | R | A sequence of flow steps without times; it reveals what the operator did. Rows of tests outside the train list were printed by three agents by accident (X-08) |
| Overlay `AIR` (anemometer), `RAD` (radiometer) | X | About one minute of response; uncalibrated |
| PSI-26: everything | external | Burns of the same kind of fabric that belong to no partition. Measured from the public stills (M-11): ten tracks in git under `tools/wsp/bass_optical/tracks/external_psi26/`. There is no video of the 2012 fabric burns, so no overlay and no recorded fan history: flows are as-run readings and fan-change times are inferred from the flame. Same duct, fan and fabric by the documents; cabin air only (21.2 to 22.7 % oxygen); millimetres not established |

## 2.6 Shared records and the private data root

The shared records (`data/work/records/<family>/`: `experiments`, `observations`, `endpoints`,
`post`, `sessions`, `review`) are what every lane's models read. This lane did not change them or
the quantity contracts.

The private root `runs/dev/data_ext/` (set with the `IGNIS_DATA` environment variable) is a copy of
the shared root to which this lane added, with the data lane's own fetch and ingest commands:

- flame-camera series for the 156 train and validation droplet runs: 283 files; after ingest,
  49,525 `uv_flame_diameter` observations on 163 runs and 34,675 `visible_flame_diameter`
  observations on 127 runs. The liquid-diameter observations (143,640) and the files `experiments`,
  `endpoints`, `post`, `sessions`, `review` are byte-identical to the shared root.
- the condition `programmed_stop` on 108 normal-flame runs and 97 inverse-flame runs (the tool was
  rerun for the inverse family on 14 November).

**Risk a reviewer should weigh.** The private root holds the full flame traces of validation runs,
and the end of a trace is close to the outcome. The safeguard is procedural: the author reads that
root only through the prefix builder and `wsp-dev feedback --partition train`. Nothing technical
stops a careless read. The same holds for the "as run" files of validation tests in
`runs/dev/nasa/PSI-159/`: the tool reads them and prints counts only.

## 2.7 Table errors found on train

From the raw channels of train tests (M-08); each should be re-derived before it is relied on.

| Tests | Table says | Commands and readbacks say |
|---|---|---|
| 21125K1, K2, K3 | burner fuel fraction 0.031 | the 30 % butane bottle undiluted: 0.30 |
| 21106G6, G7 | 0.8 mg/s | 0.5 mg/s |
| 21106G9 | | about 0.71 mg/s |
| 21196M2_1 | oxygen 5.59 mg/s (the "as run" file agrees with the table) | the setpoints and readbacks of 21196M2: 2.79 mg/s |
| 21228C4, 21228K2, 21256C2 | injected oxygen fraction 0.5 | a nitrogen command of 0.068 slpm shows no readback, so it may have been 0.85. Not settled; the comment of 21256C2 says "No nitrogen flowed" |
| 21291A1, 21291C1 | burn time 167 and 246 s | the photomultipliers go dark at 41 and 89 s: hot and cool flames alternating |
| 21125D2 | burner fuel fraction 0.045 | 0.037 (a seventh normal-flame row, found by the independent check) |
| 21270H3 | flow stopped at 43 s | the flame went out by itself at 93.3 s with the flow still on; the other tests of 27 September also carry burn times that differ from the measured end |

The models read the table values. The readback-derived flow is 1.083 times the table flow across
train (5th to 95th percentile 1.058 to 1.089), a constant factor that no model corrects.

## 2.8 Calls a reviewer should make again

1. Is the tabulated FLEX atmosphere a prospective condition (2.3)?
2. Is `programmed_stop` prospective (2.4, C-21)? Could the operators have chosen the constant-flow
   time of a test from what the previous test of the fill did, and does that matter?
3. Is the fan schedule after the cutoff a legitimate input (2.5, D-30)?
4. Flame cameras: is "no flame frame in the last quarter second" evidence that the flame is out, when
   in six train runs the flame left the field of view first (C-12)? Are frame times in the trace
   files causal, or were frames dropped or added in processing in a way that depends on what came
   later?
5. The `SE` label and the burn-time clock (C-16): should a flame that alternates between hot and
   cool count as extinguished?
6. The width class of a BASS sample from a pixel threshold.
7. The script parameter file also says whether a thin filament was moved into the flame
   (`fiberTestPosition`). Is that a prospective input (C-42)? This page treated the filament as a
   fact recorded after the test.
8. Conditions of consecutive tests in one chamber fill encode the previous test's outcome (X-18).
   Is reading validation conditions in sequence still "inputs only"? Should a reviewer be given
   them shuffled?
9. The heptane cohort: 80 of 113 records were taken as free-floating from metadata (the structured
   field `Fiber = No`, with the fibre-usage field and a narrative screen not contradicting it;
   `crates/ignis-wsp-dev/src/heptane.rs`). The 33 left out are fibre-supported or unresolved; I did
   not examine them further.

## 2.9 Astra additions

No new NASA channel or shared record is admitted. [Astra's input audit](../research/astra_data_and_input_audit.md)
disposes of the nine questions above without treating unresolved provenance as settled.
The ACME condition-node values are **model outputs/internal hypotheses**, never new
measurements: raw value, raw unit and locator from the unperturbed measurement are removed
on the internal copy, and the scenario prefix has its own input hash. The artifact retains
the user's original prefix identity. No hypothetical condition is written to the data root.

Property coefficients and assumed thermal initial states are release parameters with
source/closure provenance, not observed liquid temperatures. The BASS SI width is an
inherited apparatus calibration. Future fan controls remain the explicitly conditional
forecast task. Train scoring uses the new identity-filtered loader described in F-008.
