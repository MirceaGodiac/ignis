# Test-level atlas: the common schema

One row per test, in one schema, for the flight combustion investigations of NASA's PSI repository, so that
tests of different investigations can be placed on the same map (oxygen against pressure, against crew
atmospheres), filtered, clustered and linked to their files.

| File (under `runs/dev/data_all/work/atlas/`) | Content |
|---|---|
| `tests.jsonl` | one JSON object per row, in this schema: identity, conditions, hardware, provenance, group id, and the **names** of the outcome fields the source holds. No outcome value. |
| `tests_outcomes.jsonl` | the outcome values of the 15 investigations outside the project's partitions, keyed by `atlas_id`. Kept apart: the evaluation groups of these investigations are not frozen, and a model author reads `tests.jsonl`. |
| `tests_summary.json` | counts, coverage by field, table defects found, hashes of the inputs. |
| `tests.schema.json` | this schema as JSON Schema (draft 2020-12), written from `tools/wsp/atlas/atlas_schema.py`. |

Written by `tools/wsp/atlas/normalize_tests.py`; checked by `tools/wsp/atlas/check_atlas.py`. What each
investigation's table is, and what it lacks, is in `INVESTIGATIONS.md`. Schema identifier: `ignis.atlas.test/1`.

Two principles hold everywhere:

- **Nothing is guessed.** A value that is not in a source that was read is left empty, with the reason. A label
  is either copied from a source, with its locator, or derived by a rule that has an id in section 5.
- **Raw beside SI.** Every value keeps the source's own text and unit next to the SI value.

## 1. What a row is

A row is the finest unit the investigation's table gives. The field `level` says which:

| `level` | Meaning |
|---|---|
| `test` | one ignition (one flame, one droplet, one sample), with its own conditions |
| `test_point` | a part of a test that the table lists separately: a flow setting held inside one flame, a secondary aging run of one smoke test. `parent_test_id` names the test. |
| `test_day` | one day or session of testing; the row gives the conditions used that day, often as a set |
| `matrix_row` | one row of a planned test matrix: the set of values to be covered, not something that was run |

A `test_day` or `matrix_row` row is not a test and must not be counted as one. Its conditions are usually sets
(status `multiple`), and it never has a single point on the oxygen-pressure map unless both are single values.

## 2. The value object

Every condition is an object with the same thirteen keys.

| Key | Content |
|---|---|
| `status` | `resolved`, `multiple`, `varying`, `unknown`, `not_applicable`, `withheld_outcome` (below) |
| `value_si` | the value in SI, or the token of a closed list, or a canonical text. `null` unless `status` is `resolved`. |
| `unit_si` | `Pa`, `1` (mole fraction or other ratio), `m`, `m/s`, `kg/s`, `m3/s` (at the source's standard conditions), `K`, `s`, `W`, `date` (ISO `YYYY-MM-DD`), `time` (24-hour `HH:MM[:SS]`), or `null` for tokens and text |
| `raw_value` | what the source cell holds, as text or number, before any conversion |
| `raw_unit` | the unit of the raw value as the source writes it (`atm`, `mmHg`, `cm/s`, `%`, `est`, `GMT`, ...) |
| `basis` | `nominal`, `measured`, `derived`, `unstated` (below) |
| `scope` | `row`: the source gives the value for this row. `investigation`: a document gives it for the whole investigation and it is repeated on every row. |
| `source` | where the value is: `file`, `sheet`, `column`, `row`, `page`, `field`, `note`. `row` is the record number in a CSV (header is 1) or the row of a spreadsheet sheet. `field` names a field of a JSON metadata file. |
| `rule` | id of the rule of section 5 that produced the value, when it is not a plain copy with a unit conversion |
| `flags` | list from the closed list of flags (section 4), or a flag of a project record behind the prefix `rec:` |
| `reason` | why the value is empty; required whenever `status` is not `resolved` |
| `set` | `{kind, values}` when `status` is `multiple`: `kind` is `list` (these values) or `range` (from the first to the second) |
| `alternatives` | other values for the same quantity from another source, each with `value_si`, `raw_value`, `source`; present with the flags `conflict`, `malformed_in_table` or `blank_in_table` |

### Status

| `status` | Meaning |
|---|---|
| `resolved` | one value |
| `multiple` | the source gives several values or a range for this row (a test day, a matrix row, a two-part sample); see `set` |
| `varying` | the source says the quantity was changed during the test and gives no single value (a cell that reads "RAMP", an untimed sequence of fan settings) |
| `unknown` | the quantity exists for this test but is not in any source that was read. It is **not** zero and **not** a default. The reason says where it is expected. |
| `not_applicable` | the quantity has no meaning for this kind of test (the fuel fraction of the ambient gas for a flame whose fuel is injected; an injected flow for a droplet) |
| `withheld_outcome` | the source cell mixes information from before and after ignition (an oxygen reading given as a range over the burn). The cell is kept in `tests_outcomes.jsonl` and can be promoted once the groups are frozen. |

### Basis

| `basis` | Meaning | Assigned when |
|---|---|---|
| `nominal` | a planned, commanded or labelled value | bottle labels, set points, matrix values, round fill values ("1 atm", "~21% O2"), tokens read from a title or column |
| `measured` | a reading that the source reports as taken on the test | the source or its report says so (a calibrated sensor, a flow read from the video record, a diameter from the analysis sheet) |
| `derived` | computed by the source, or by a rule of this schema, from other values | dates from a day-of-year code, balance fractions, values a table note calls derived |
| `unstated` | the source gives a number and does not say which of the three it is | unrounded table numbers with no explanation, and conditions copied from project records |

`unstated` is used rather than a guess. `INVESTIGATIONS.md` lists the basis per column.

## 3. Row fields

| Field | Type | Content |
|---|---|---|
| `schema` | text | `ignis.atlas.test/1` |
| `atlas_id` | text | `<investigation_id>:<test_id>`, unique in the file |
| `investigation_id` | text | PSI accession, `PSI-<n>` |
| `acronym` | text | acronym of the investigation, from its PSI metadata |
| `test_id` | text | the identifier the source gives the row (test name, Flex number, sample number, test day); where the source gives none, a positional id (`matrix-3`, `d045-row0002`) |
| `aliases` | list of `{scheme, name}` | other names of the same test, for linking to files (CIR test point, file Flex number, camera record names, campaign day) |
| `level` | closed list | section 1 |
| `parent_test_id` | text or null | for a `test_point`: the test it belongs to |
| `project_partition` | closed list | `train` or `validation` for rows of the four project families; `outside_partitions` for the 15 others |
| `group_id` | text | stable group of the row: the test day or session as the source gives it. Splits are frozen by this field. Section 6. |
| `group_rule` | rule id | how `group_id` was formed (section 6) |
| `source_rows` | list of `{file, sheet, row}` | every table row the atlas row was built from |
| `date` | value, `date` | day of the test |
| `start_time` | value, `time` | time of day of the test as the source gives it (ignition time where the source says so); the time zone label is in `raw_unit` and no conversion between zones is made |
| `hardware` | object | `rack`, `insert` (values) and `raw` (the hardware string of the PSI metadata) |
| `fuel` | object | `name` (value: canonical fuel or sample material; several fuels joined by `+`; diluents are not part of the name), `components` (list of `{species, role, fraction}`; `role` is `fuel` or `diluent`; `fraction` is a value), `phase` (value) |
| `geometry` | object | `configuration` (value), `flame_orientation` (value), `sizes` (list of `{dimension, value}` in metres) |
| `atmosphere` | object | `x_o2` (ambient oxygen mole fraction), `pressure`, `temperature`, `diluents` (`status`, `reason`, `items` of `{species, fraction}`), `x_fuel_ambient` (fuel mole fraction of the chamber gas, for inverse flames) |
| `oxidizer_stream` | object | `x_o2`: oxygen mole fraction of the oxidiser side. The ambient value for every test except inverse flames, where oxygen is injected (rule `OX1`). |
| `flow` | object | `speed` (imposed speed of the ambient gas past the flame, m/s) and `direction` (value) |
| `injection` | object | reactant flow through a burner: `x_fuel_injected`, `fuel_mass_flow`, `oxidizer_mass_flow`, `inert_mass_flow`, `total_mass_flow` |
| `gravity` | value | gravity level |
| `extra_conditions` | object | commanded settings and conditions that have no common field; closed list of keys in section 4 |
| `varied` | list of field paths | what differs between the rows of this investigation (rule `VAR`); the same list on every row of an investigation |
| `measurements` | object | `status` (`not_filled` or `filled`) and `items` of `{kind, category, n_files, bytes}`. Defined here, filled later from the file catalogue; `not_filled` in this version. |
| `outcome_status` | closed list | `listed`: the source table holds outcome columns, named in the next field. `none_in_source`: the table has no outcome column. `not_read_protected`: a project family, whose outcomes this tool does not read. |
| `outcome_fields_available` | list of `{name, unit, source}` | names of the outcome columns of the row's table. A name is listed for every row of the table whether or not its own cell is filled, because which cells are filled is itself outcome information. |
| `labels` | object | `flame_type` (value), `studied` and `fire_safety_need` (lists of `{value, rule, matched, source}`) |
| `map` | object | for plotting: `x_o2`, `x_o2_role` (`ambient` or `injected`), `pressure_pa`, `crew_atmospheres`, `crew_atmospheres_o2_only`. Filled only from single resolved values (rule `CREW`). |
| `notes` | list of text | remarks about the row that are not conditions |

Conventions: mole fractions are fractions, not percent; a percentage "by volume" in a source is read as a mole
fraction. Species names are lower-case words (`ethylene`, `nitrogen`, `sibal_fabric`); the list is open.

## 4. Closed lists, in full

**`level`**: `test`, `test_point`, `test_day`, `matrix_row`.

**`project_partition`**: `train`, `validation`, `outside_partitions`.

**`status`**: `resolved`, `multiple`, `varying`, `unknown`, `not_applicable`, `withheld_outcome`.

**`basis`**: `nominal`, `measured`, `derived`, `unstated`.

**`scope`**: `row`, `investigation`.

**`set.kind`**: `list`, `range`.

**`hardware.rack`** (the facility the experiment ran in): `CIR` (Combustion Integrated Rack), `MSG` (Microgravity
Science Glovebox), `Cygnus` (the cargo vehicle, after it left the station).

**`hardware.insert`** (the experiment hardware inside it): `MDCA` (Multi-user Droplet Combustion Apparatus),
`ACME` (Advanced Combustion via Microgravity Experiments chamber insert), `Saffire` (the Saffire flow duct unit),
`CSD_SAME` (the Comparative Soot Diagnostic hardware used by SAME and SAME-R), `own_apparatus` (the hardware
string names no insert: the investigation's own apparatus).

**`fuel.phase`**: `gas`, `liquid_droplet`, `liquid_other` (a liquid that is not a free or tethered droplet, for
example held in a wick), `solid`.

**`geometry.configuration`**:

| Token | Meaning |
|---|---|
| `spherical_porous_burner` | gas issues from a porous sphere |
| `flat_porous_burner` | gas issues from a flat porous disc (burning-rate emulator) |
| `gas_jet_burner` | gas issues from a tube into still gas |
| `coflow_burner` | gas issues from a tube inside a parallel stream of oxidiser |
| `droplet_free` | free-floating liquid droplet |
| `droplet_fiber_supported` | droplet tethered on a fibre |
| `droplet_unspecified` | droplet, support not stated |
| `flat_sample` | flat solid sample (fabric, sheet or slab) in a flow duct |
| `heated_wire_sample` | sample heated by a wire to make smoke, without a flame |
| `aerosol_bag` | no fuel: a dust aerosol in a bag, for instrument tests |

**`geometry.flame_orientation`**: `normal` (fuel is injected into the oxidiser), `inverse` (oxidiser is injected
into fuel). `not_applicable` for anything that is not a burner flame.

**`geometry.sizes.dimension`**: `burner_diameter`, `nozzle_inner_diameter`, `initial_droplet_diameter`,
`sample_thickness`, `sample_length`, `sample_width`.

**`flow.direction`**: `quiescent` (no imposed flow), `concurrent` (flow in the direction of flame spread),
`opposed` (flow against flame spread), `coflow` (oxidiser stream parallel to the fuel jet), `forced` (an imposed
flow whose direction relative to a flame is not defined, as past a heated sample), `burner_injection` (the only
imposed flow is the reactant leaving the burner; see `injection`).

**`gravity`**: `microgravity`, `normal_gravity`, `partial_gravity`. Every row of this version is `microgravity`.

**`labels.flame_type`** (the phenomenon, by rule `L1` from the configuration): `spherical_diffusion_flame`,
`flat_burner_diffusion_flame`, `jet_diffusion_flame`, `coflow_jet_diffusion_flame`, `droplet_flame`,
`solid_surface_flame`, `overheat_smoke` (smoke from an overheated sample, no flame), `none` (no combustion).

**`labels.studied`** (what the investigation's title names, rule `L2`): `cool_flame`, `smoke_point`,
`extinction`, `electric_field`, `liftoff`, `burning_rate`, `aerosol`.

**`labels.fire_safety_need`** (rule `L3`): `prevention`, `detection`, `suppression`. An empty list means that no
keyword of the rule matched, not that the investigation is irrelevant to fire safety.

**`map.x_o2_role`**: `ambient`, `injected`.

**`map.crew_atmospheres`** and `crew_atmospheres_o2_only`: `earth_iss`, `moon_base_rover`, `spacesuit` (rule `CREW`).

**`outcome_status`**: `listed`, `none_in_source`, `not_read_protected`.

**`measurements.status`**: `not_filled`, `filled`.

**`measurements.items.kind`**: `color_imaging`, `backlit_imaging`, `uv_imaging`, `intensified_imaging`,
`still_images`, `radiometer`, `photomultiplier`, `thermocouple`, `pressure`, `gas_flow`, `gas_composition`,
`air_velocity`, `particle_instrument`, `electrical`, `test_log`.

**`flags`**:

| Flag | Meaning |
|---|---|
| `approximate` | the source marks the value as approximate ("~", "approx") |
| `conflict` | another source gives a different value; it is in `alternatives` |
| `malformed_in_table` | the table cell is damaged or is not a value ("3400%", "037 mm", "See Fig. 5"); the value comes from the document named in `source`, and the cell is in `alternatives` |
| `blank_in_table` | the table cell is blank; the value comes from the document named in `source`, or the token is the unspecified one |
| `fill_series_artifact` | the cell is a spreadsheet fill-series artefact and was read as the first cell of its run (rule `FILL`) |
| `fractions_sum_not_1` | the tabulated mole fractions of the atmosphere do not add up to 1 |
| `negative_value` | the source gives a negative number for a quantity that is physically non-negative; it is copied as given |
| `conditioned_on_ignition` | a table note says that only the values of tests that ignited are listed |
| `defect_noted` | the value is copied as given but a defect of the table affects it; see `INVESTIGATIONS.md` |
| `balance_gas` | the fraction is the remainder after the named components (rule `BAL`) |
| `from_analysis_sheet` | taken from an analysis sheet rather than from the test log |
| `basis_of_fraction_unstated` | the source does not say whether a mixture fraction is by mole, volume or mass |

**`extra_conditions` keys**:

| Key | Unit | Content |
|---|---|---|
| `campaign_day` | text | day index of the campaign, as the table numbers it |
| `description` | text | free-text description of the row |
| `test_type` | text | the table's test-type or test-round label |
| `fiber` | text | the table's fibre flag |
| `igniter_power` | `W` | igniter power |
| `igniter_time` | `s` | igniter on-time |
| `heater_resistance_ratio` | `1` | SAME: ratio of the heated wire's resistance to its room-temperature resistance, the temperature set point |
| `heating_duration` | `s` | SAME: duration of sample heating |
| `primary_aging_time` | `s` | SAME: commanded primary aging time |
| `primary_mixing_time` | `s` | SAME: commanded primary mixing time |
| `planned_aging_duration` | `s` | SAME: planned aging duration of the matrix row |
| `mesh_polarity` | text | E-FIELD: polarity of the high-voltage mesh |
| `fuel_mfc_full_scale` | `m3/s` | full scale of the fuel mass flow controller (standard, nitrogen basis) |
| `n2_mfc_full_scale` | `m3/s` | full scale of the nitrogen mass flow controller (standard, nitrogen basis) |
| `fuel_volume_flow_std` | `m3/s` | fuel stream volume flow at the source's standard conditions |
| `hardware_description` | text | DAFT: what the session's hardware contained |
| `sampled_location` | text | DAFT: where the air was sampled |
| `fan_display` | text | BASS: fan setting as displayed |
| `flow_restrictor` | text | BASS: flow restrictor setting |
| `x_co` | `1` | carbon monoxide mole fraction of the ambient before the test |
| `zst` | `1` | stoichiometric mixture fraction, as the project record holds it |
| `tad` | `K` | adiabatic flame temperature, as the project record holds it |

## 5. Rules

Each derived value names its rule in `rule`. A value without a rule is a copy of the source cell, with a unit
conversion at most. Conversions: 1 atm = 101325 Pa, 1 bar = 100000 Pa, 1 mmHg = 133.322387415 Pa,
1 psi = 6894.757293168 Pa, 1 sccm = 1e-6/60 m3/s, 1 slpm = 1e-3/60 m3/s. Converted values are rounded to 12
significant digits.

| Rule | Statement |
|---|---|
| `D1` | A five-digit test-day code `YYDDD` is year 20YY, day DDD of the year (GMT). The date is 1 January plus DDD - 1 days. |
| `D2` | A GMT day of year with a year stated by the source (in the cell, or in a report named in `source.note`) gives the date the same way. |
| `D3` | A day-month cell with the year stated by a report named in `source.note` gives the date. |
| `DAYCOL` | Flame Design: when the day code in a test's name differs from the day code shared by the other tests of the same value of the table's Day column, the Day column decides the date and the group. The date from the name is kept in `alternatives`, and the name under which the listing has files is added to `aliases` when it exists there and not in the table. |
| `T1` | A time of day is written `HH:MM:SS` on a 24-hour clock. "pm" adds 12 hours; the separators ":" and "." are equivalent. The zone label is copied to `raw_unit`; no conversion to GMT is made. A cell that is not a complete time is `unknown`. |
| `H1` | Rack and insert are the tokens whose names appear in the hardware string of the PSI metadata (Combustion Integrated Rack, Microgravity Science Glovebox, Cygnus; Droplet Combustion Apparatus, ACME, Saffire, Soot Diagnostic). |
| `H2` | Where the hardware string names no rack, the rack is taken from the document cited in `source`. |
| `H3` | Where the hardware string is not empty and names no insert, the insert is `own_apparatus`. |
| `G1` | `microgravity` when `projectType` of the PSI metadata is "Microgravity Investigation". |
| `G2` | For the four project investigations, whose full metadata this tool does not open: `microgravity` when the rack is `CIR` or `MSG`, both on the space station. |
| `L1` | `flame_type` follows the configuration: spherical burner, flat burner, gas jet and coflow burner give the four diffusion-flame tokens; any droplet gives `droplet_flame`; `flat_sample` gives `solid_surface_flame`; `heated_wire_sample` gives `overheat_smoke`; `aerosol_bag` gives `none`. |
| `L2` | `studied` holds a token when the investigation title matches its pattern, case-insensitive: "cool flame", "smoke point", "extinguish" or "suppress" (`extinction`), "electric", "liftoff" or "lift-off", "burning rate", "aerosol". The matched text and the field are recorded. |
| `L3` | `fire_safety_need`: `suppression` when the title matches "extinguish" or "suppress"; `detection` when the title matches "aerosol"; `prevention` when "flammab" occurs in the title or, for the 15 investigations whose metadata is on disk, in the objective or the approach. A keyword rule: coarse by construction, and it records what it matched. |
| `P1` | Phase is `liquid_droplet` when the configuration is a droplet or the insert is `MDCA`. |
| `P2` | Phase is `gas` when every fuel species is one of methane, ethylene, ethane, propane, propylene, butane, hydrogen, which are gases at room conditions and are fed from bottles. |
| `P3` | Phase is `solid` when the configuration is `flat_sample` or `heated_wire_sample`. |
| `P5` | Phase is `liquid_other` when a cited document says the sample is a liquid held in a wick. |
| `F1` | For a burner without a coflow, the flow direction is `burner_injection` and the flow speed is `not_applicable`. |
| `F2` | For a heated sample, an inlet velocity of zero is `quiescent` and any other is `forced`. |
| `OX1` | The oxidiser side is the ambient unless the source says oxygen is injected. |
| `OR1` | A flame whose table gives a fuel bottle or a fuel flow through the burner or nozzle, in an oxidising ambient, is `normal`. |
| `BAL` | In a stream of named components, the last named gas is the balance: its fraction is 1 minus the others. Not computed when another fraction is a set or varying. |
| `PURE` | A fuel named without a fraction and without a diluent has fraction 1. |
| `XF` | The fuel mole fraction of an injected mixture is the sum of its fuel components. |
| `REF` | A matrix row that says "same conditions as rows a-b" takes the set of values of those rows. |
| `DOC` | Where a table cell is blank or damaged and a table of a document of the same investigation gives the value, the document's value is used; the table cell is kept in `alternatives`. The document tables used exist only as pictures and were transcribed by hand into `tools/wsp/atlas/transcribed/`. |
| `FILL` | A spreadsheet fill-series artefact is a run of consecutive cells whose text is the first cell's text with its trailing integer raised by one per row ("40% CH4", "40% CH5", ...). Every cell of the run is read as the first. |
| `CFG-DROPLET` | The configuration is `droplet_unspecified` when the hardware is the droplet apparatus and the support is blank or not tabulated. |
| `CFG-SHEET` | The configuration is `flat_sample` when the table gives a sample thickness, length and width. |
| `REC` | A condition of a project record is copied with its raw value, SI value, unit, locator and flags (prefixed `rec:`). Basis is `derived` when the record's evidence class is `source_derived`, else `unstated` unless the record's own note says how the value was obtained. An untimed sequence of settings is `varying`. The date is the day of the record's session id. |
| `VAR` | A field is listed in `varied` when its resolved values differ between the rows of the investigation, or when any row holds a set or a varying value for it. For the four project investigations only train and validation rows are compared. |
| `CREW` | Section 7. |

A number that starts with 0 followed by a digit ("037 mm") is treated as damaged and is not read.

## 6. Group id

`group_id` is the test day or session as the source gives it. Tests of one group share a chamber fill, a bottle,
a crew session or a flight, so a split must keep a group whole.

| Rule | Form | Used for | Derivation |
|---|---|---|---|
| `GR-DAY` | `PSI-<n>:day:<YYDDD>` | PSI-10 Flame Design, PSI-20 BRE, PSI-21 CLD Flame, PSI-22 E-FIELD Flames, PSI-23 s-Flame | first five characters of the test name, or the table's test-day code |
| `GR-DAY` | `PSI-<n>:day:<YYDDD>` | PSI-10, one test | where the name's day code disagrees with the table's Day column, the Day column decides (rule `DAYCOL`) |
| `GR-CIRDAY` | `PSI-39:day:<date>` | PSI-39 CFI | the day of year written in the CIR test-point name (three digits after the first letter), in the year of the Test Date. It equals the Test Date for all but two tests, whose date cells are typing errors; `GR-DAY` on the Test Date is the fallback when a row has no CIR name. |
| `GR-DAY` | `PSI-101:day:<date>` | PSI-101 SAME-R | Date of the table with the year 2010 |
| `GR-PARENT` | the parent's group | PSI-101 SAME-R, secondary aging runs | a test point stays in the group of its test even when the table dates it a day later |
| `GR-DAY` | `PSI-106:day:<date>` | PSI-106 SLICE | Test Date of the table |
| `GR-DAY` | `PSI-107:gmt:2009-<DDD>` | PSI-107 SPICE | GMT date of the table (day of year) |
| `GR-SESSION` | `PSI-47:session:<name>` | PSI-47 DAFT | Session Name |
| `GR-FLIGHT` | `PSI-<n>:flight` | PSI-98, PSI-99, PSI-100 Saffire | all samples of one Saffire unit burned in one flight, one after another in the same atmosphere: one group |
| `GR-MATRIX` | `PSI-<n>:matrix` | PSI-68 FLEX-2, PSI-102 SAME | the table is a planned matrix with no day; all its rows form one group |
| `GR-SPLIT` | as the split file gives it | PSI-69 FLEX, PSI-159 CFI-G, PSI-25 BASS-II | the group of the project's frozen split, read through `tools/wsp/analysis/lib.py`; not recomputed |
| `GR-ROW` | `PSI-<n>:test:<test_id>` | fallback | a row whose day cannot be read forms its own group; not used in this version |

## 7. Crew atmospheres (rule `CREW`)

A row is placed in a crew atmosphere only when its **ambient** oxygen and its pressure are both single resolved
values. An injected oxidiser stream is not an atmosphere and is never classified.

| Token | Target | Oxygen mole fraction | Pressure |
|---|---|---|---|
| `earth_iss` | Earth and the space station: 21 % at 14.7 psi | 0.20 to 0.22 | 14.2 to 15.2 psi (14.7 plus or minus 0.5) |
| `moon_base_rover` | planned Moon base and rovers | at least 0.30 | at most 10.2 psi |
| `spacesuit` | 100 % at 4.3 psi | at least 0.99 | 3.8 to 4.8 psi (4.3 plus or minus 0.5) |

The tolerances are choices of this schema and are stated in `tests_summary.json`. `crew_atmospheres_o2_only`
lists the atmospheres whose oxygen band the row is in when its pressure is unknown: a candidate, not a member.

## 8. Outcome separation

An outcome is anything known only after ignition: burn time, flame-off or extinction time, burn or spread length
and rate, weight-loss rate, final composition or pressure, the number of flames that ignited, anything measured
during or after the test, and any property computed from those. Date, start time and commanded settings are
conditions.

- `tests.jsonl` never holds an outcome value. It lists the names of the outcome columns of the row's table.
- `tests_outcomes.jsonl` holds, per `atlas_id`, the non-empty outcome cells: `name`, `raw_value`, `raw_unit`,
  `source` (file, sheet, column, row). Raw values only; nothing is converted or interpreted.
- For the four project investigations (PSI-69, PSI-159, PSI-25, PSI-26) no outcome is read at all, and only
  conditions of train and validation runs are copied, through `tools/wsp/analysis/lib.py`.

## 9. Checks

`python tools/wsp/atlas/check_atlas.py` runs the writer twice in separate processes and compares the bytes of
the four files with each other and with the files on disk; validates every row against the JSON Schema and
against the status rules of section 2; checks that `atlas_id` is unique; checks that no value of `tests.jsonl`
comes from a cell held in `tests_outcomes.jsonl`; and checks that this document names every token of every
closed list, every rule id in use and every row field.
