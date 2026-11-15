# Source-bound condition supplements

This is a non-modelling custody result. It proposes additions for the atlas owner and
does not change the 1,612-row projection, the atlas, a split, a model or a release.
No `tests_outcomes.jsonl`, holdout directory, sealed validation series, old model or
result file was opened. Source mixtures and mixed documentation were screened in this
custody lane; those exposures prohibit a later move into model authoring.

Historical author-safe output (not reverified by the revised reader):

`runs/dev/data_astra/work/condition_supplements/author_safe/aafea65c9bd8f31afe23a198776d1f2b8d3aab4030da2a60f9f9643343a2848e/`

The earlier supplement version remains immutable. Private source-coverage and exposure
receipts are under the matching `condition_supplements/private/<version>/` directory.
Authors must not open that private tree or the source payloads linked by a receipt.

The post-review source snapshot and channel-mapping changes below were tested only with
synthetic inputs. **Existing NASA supplements are unverified against this revised code
until a separately authorized custody replay.** Their original manifests describe the
earlier execution; neither those artifacts nor their receipts were rewritten or
recertified during this fix. The historical recovery counts below are retained as the
prior run's record, not a new execution claim.

## Actual recovery

| Source work | Result | Limits |
|---|---:|---|
| PSI-26 NASA experimental table | 121 source-row identity proposals from 122 rows | One row has no test identifier; 121 is not a claimed independent-burn count. |
| BASS material identities | 109 resolved proposals | Unnamed materials remain unknown; sample identifiers, not positional proximity, support repeated-sample joins. |
| BASS geometry | 34 `flat_sample` proposals | Sphere and candle configurations are absent from the current atlas vocabulary. |
| BASS setup workbook | 120 nominal flow-restrictor settings | These are setup codes, not calibrated flow speeds. |
| BASS dates | Nine unresolved rows, eight with exact workbook alternatives | Source typos or disagreements are preserved, not silently repaired. |
| FLEX-2 file catalogue | 596 exact CIR test-point proposals | A source test point is not automatically a burn; classification and official identities remain the owner's task. |
| FLEX-2 fuel folders | 532 unambiguous point-to-fuel proposals | Folder mixture numbers do not establish fraction basis or purity. |
| FLEX-2 source clocks | 495 point dates with matching source calendar/day code | Missing or inconsistent clocks remain unknown. |
| FLEX-2 pre-igniter thermometry | 471 source observations | Before the controller igniter command; calibration and successful ignition are not independently established. |
| FLEX-2 mixture files | All 124 inventoried, 545 same-day candidates | Zero verified chamber-fill composition assignments. No composition numbers were released. |
| Temperature coverage | All 19 investigations inventoried | Header/document coverage is explicit; unestablished initial temperatures stay unknown. |

## Intake format and identity

`psi26_intake.jsonl` and `psi68_intake.jsonl` contain partial intake records, not full
atlas rows. Every condition uses the actual `ignis.atlas.test/1` field schema, value
status, unit and vocabulary. The helper validates against Fable's live metadata schema
and its status invariants; the manifest pins the exact schema bytes. No parallel JSON
Schema or shared contract is introduced.

Original BASS `Test #`, `As Run Test #` and `Sample #` values remain separate. Source
row hashes/locators distinguish repeated identifiers. The workbook join requires exact
agreement of all three identifiers and a unique matching row. FLEX-2 uses the catalogue's
exact `cir_test_point` identifiers and exact file ids. It neither repairs aliases nor
turns a day/fuel-folder name into a run id.

Every proposed `atlas_id` and `group_id` remains null with
`identity_assignment = pending_atlas_owner`. FLEX-2 remains held until the owner accepts
an actual test list and freezes the groups. No proposal grants outcome access.

## BASS sources and geometry limits

The verified `PSI-26_Experimental table_BASS.csv` has only date/GMT, as-run/test/sample
identifiers and material descriptions. `PSI-26_Science Documents_As Run_Test Matrix.xlsx`
is mixed: only the `as run matrix` sheet's identity/date/material/setup columns are
interpreted. Radiometer, image-count and other output columns are not exported.

Crew Training 2011, page 21, independently identifies SIBAL, Nomex and Ultem as flat
samples; the exact source PDF hash is
`11ad76194acf095886b0baf33ac3cb960f266d55dce0ab5b15c69de0db7afb62`.
These apparatus facts are joined to explicit material identities with both source
locators. The same page lists PMMA sphere samples and Japan/paraffin wax candles.
Its page 25 concerns post-burn access safety and was not used as an ambient-temperature
condition. A later safety temperature must never become a preignition default.

Current vocabulary has neither a spherical-solid/candle configuration nor a solid-sample
diameter field. `geometry.configuration` therefore stays unknown for those proposals.
The owner can review the page-21 evidence for a vocabulary addition. This custodian does
not invent enum values or substitute `burner_diameter` for a solid-sphere diameter. A
label such as “2 cm” also does not establish thickness or width without a dimension
locator. Initial oxygen, pressure, temperature and calibrated flow are not supplied by
the screened BASS table. An untimed environment value or changing air-display sequence
in the workbook is not promoted to an initial condition.

## FLEX-2 joins and pre-igniter temperatures

`files.jsonl` preserves the Reduced Gravity campaign/fuel folders. The supplement parser accepts
only inspected fuel labels, retains `octane` separately from `n_octane`, and requires a
unique fuel assignment per exact point. When both exist, the explicit campaign year must
agree with the source clock. Source folder file ids and the whole catalogue hash accompany
every proposal. Component identities are kept; fractions remain `unknown` because neither
purity nor mole/mass/volume basis follows automatically from a folder label.

The generic point-summary reader interprets only a whitelisted controller header and
the source `Ave DegC` column. It requires the expected exact point id, an automated-sequence
start, an `AS Burn to Completion/Igniter On` timestamp, the expected channel header and a
monotonic timestamp prefix. It keeps the last source sample after sequence start and
strictly before the same-file igniter command. At the first timestamp at or after that
command it stops **before parsing the numeric payload**, and never reads later records.
Source bytes are separately hashed opaquely for identity.

The revised parser binds the two trailing average channels by their exact declared
`PSIA` and `DegC` units. Reversed channel order is mapped correctly; duplicate channels,
unknown units, extra trailing columns or an ambiguous channel pair fail closed. It no
longer assumes that the final number is temperature merely because both names appear
somewhere in the header. The known eighteen-token numeric layout remains required.

Every source parser now consumes a private temporary snapshot whose complete byte count
and SHA-256 match the acquisition receipt before parsing begins. An open-file identity
check detects changes during copying. Source paths are never accepted from a cached
verification for a later reopen. The source is rechecked after parsing, and all used
sources are rehashed once more before publication. Replacement, disappearance or drift
aborts the run without updating the expected receipt. Snapshots are streamed through
temporary files one source at a time; this does not load the corpus into memory.

The observation retains its source line hash, raw Celsius value, Kelvin conversion,
source-controller timestamp and elapsed seconds before the command. No average, fit,
trajectory or extrapolated t=0 temperature is generated. Clock qualification means the
event and observation use the same source controller clock; it is not independent
certification of the physical timebase or proof that a flame ignited.

FLEX-2 Engineering Requirements pages 22, 28–29 and Science Requirements pages 42–43,
51 describe ambient chamber thermometry, initial-condition requirements and GMT
synchronization. Their specified operating ranges and accuracy are **requirements**, not
observations or demonstrated calibration error. They are not filled into missing runs.
Of 588 screened summary files, 471 yield the required prefix, 52 have a nonmonotonic
prefix, 61 have no qualifying pre-command sample, and four lack the required clock/channel
identification. No unusable file is silently sorted, repaired or treated as zero.

The 124 gas-mixture files have separate reading-point ids and timestamp cells. Ambiguous
AM/PM or two-digit-year strings remain unknown. Shared day/point prefixes and timestamp
order identify candidates, not a common chamber fill. Their clock zone and fill membership
are not documented by these files. A scan of 581 comment sources and one event-log
screenshot did not establish a usable explicit composition association. The mixtures'
numerical composition fields are never returned by the parser or written into author
outputs. `source_clock_order` is explicitly distinct from the unresolved `timing_status`.

## Temperature information outside FLEX-2

`temperature_coverage.json` records candidate-file counts, verified availability, the
exact representative header sources inspected and remaining requirements for all 19
investigations. It does not equate a filename or a thermocouple channel with ambient
temperature. Camera filename-index files found for SLICE/SPICE are not thermometry.

The verified ACME non-imaging data description, SHA-256
`684d321a26e81472f3c9ca7d320f531217b06cf863075a60ebb004a931bba06b`, pages 2, 6–7 and 9,
resolves important channel semantics: TC200A/B are thermocouples 200 mm from the axis;
TCBurner measures the burner; TCCJ measures the cold-junction assembly; rake channels
have BRE-specific variants. Analog conversion, cold-junction handling, ambient-location
qualification and a command-log ignition join remain prerequisites for numerical release.
Raw channel values must not be assumed to be degrees Celsius. The Saffire-III header
identifies `P@PCM_ETS_TEMP_SIG`; its calibration, location and sample/clock joins remain
unestablished. No 298 K, ISS default or post-burn reading is substituted.

## Reproduce and verify

```powershell
python -m unittest discover -s tools/wsp/astra -p test_condition_supplements.py -v
python tools/wsp/astra/condition_supplements.py
python tools/wsp/astra/condition_supplements.py --execute
```

The original eight synthetic tests checked the strict stop before post-command numeric
parsing, denied post-boundary reads, missing/mismatched clocks, nonmonotonic prefixes,
ambiguous mixture timestamps, exclusion of composition values, exact fuel-label policy
and unknown-versus-zero handling. The real dry run and `--execute` completed. All seven
published data-file hashes were independently re-read and matched the manifest; intake
identities remained unassigned and every released temperature had a strictly positive
pre-command interval. No model qualification or evaluation-budget use follows from this.

The private exposure ledger records the initial mixed telemetry format probe, limited
mixed workbook/header discovery, comment screening and event-log inspection. No private
numeric result or post-test content was sent to the author lane. The current role must
remain non-modelling.

Post-review verification: **13 synthetic supplement tests passed**. The added checks
reproduce source replacement after an earlier verification, mutation during snapshot
parsing, source drift after parsing, reversed channel order and ambiguous/unsupported
channel units. No real source was replayed for these checks, and the prior NASA output
remains unverified under the revised implementation until that replay occurs.
