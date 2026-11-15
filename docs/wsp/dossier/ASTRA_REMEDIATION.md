# Owner-requested remediation after the Flame Design report

Opened 2026-11-14 after the owner supplied Fable's report and explicitly asked
that its remaining problems be resolved. This extends the all-NASA work; it
does not relax the outcome-access, literal-authoring or no-push rules.
Existing Astra candidates remain **data-informed**, unchanged after feedback.

## Storage-plan drift: already resolved upstream, independently checked

Current `MirceaGodiac/ignis` main was verified at
`ce82e5467e04006e410d5c7dbbfa3781a121def5` with `git ls-remote`.
The plan/workboard correction originated in
`63e4226982045a3a91d6b2d3e79313621f74c780` (2026-11-14 16:22:33 UTC).
`LANES_V2.md` lines 171 and 358–364 and `WORKBOARD.md` line 47 agree;
the plan's storage section is now section 10, not the formerly cited section 7.

| Authoritative inventory | Unique files | Bytes |
|---|---:|---:|
| Documents/tables | 636 | 1,629,828,160 |
| Sensor/settings text | 24,482 | 12,255,895,308 |
| Combined | 25,118 | 13,885,723,468 |

The combined size is 13.885723468 decimal GB. Neither inventory nor their union
duplicates `(investigation, psi_file_id, file_version)` or a stable URL.
Their Git bytes match the inventories merged through PR 25 at
`52e7426f5beb5495de32287b5ec18e9826231a0c`.

- Document CSV SHA-256: `f993618eef9e00c5978d9052d09fff9011471b2ef87f43e00e8999cba1d897c0`.
- Compressed sensor CSV: `1cd284cffe898c9038b73e4e1ca215848914d6cb123746cc894eaca866804b79`.
- Uncompressed sensor CSV: `8c7ea8b729481c567f8f7d287218754632b835edb56dedf6936fefad9a3e1155`.

No redundant upstream edit or commit was made. The GitHub connector returned
404; configured local `gh`/Git read access succeeded. A temporary managed
verification worktree was archived successfully and remains recoverable;
the science checkout remains on `feat/handwritten-science`. A distinct `ignis`
fetch remote was initially added. The owner later requested the actual repository
transfer; the current checkout is based on IGNIS main, as recorded in
`ASTRA_ALL_NASA_PROGRESS.md`. Nothing was pushed.

## Local split prose: corrected without regrouping

F-011 distinguishes nine split investigations from ten split strata; six
investigations are held. PROTOCOL 11.2 prose is corrected at the owner's
request. No frozen JSON, group membership, seed, selection or old commit changed.

## Endpoint disagreements: investigate before assigning a winner

A new non-modelling audit is restricted to the complete 154-test CFI-G TRAIN
identity set, checked before payload access. It preserves the original endpoint
method and receipts. The new diagnostic protocol is frozen before raw replay.
The completed first replay reproduced 152 historical measurements to their
reported precision. One exact raw source is unavailable and one identity join
conflicts; neither is guessed. Seven traces have nonincreasing clocks and are
refused by the stricter diagnostic, leaving a 145-trace comparison population.
The nine discrepancies exceeding five seconds now have source-bound case
accounts: stabilization-stage origins, an abort, an absent log endpoint, a
conflicting reference, and nonterminal optical losses/reignitions are distinct
possibilities. They do not justify a global time offset.

New diagnostic helpers require the complete observed confirmation interval,
never accept a short EOF tail as confirmation, never bridge a timestamp gap,
preserve an extinction stamp separately from completion/abort status, and
classify cutoff intervals including unresolved and censored cases. All eleven
focused tests passed. The report is
[`ENDPOINT_TRAIN_AUDIT.md`](../measure/ENDPOINT_TRAIN_AUDIT.md).
Its event brackets describe observed processing support, not calibrated physical
confidence intervals. The completed lossless continuation reconstructed all 306 source ledgers
(726,514,735 bytes; 1,948,244 physical lines). It identifies four NUL-corrupt
rows, seven nonincreasing clocks and five gaps; 144 traces remain optically
eligible. It distinguishes first optical loss, returns and last observed loss
from an actual chemical endpoint. See `ENDPOINT_TRACE_V2.md`. These diagnostics
are not an independent blind confirmation or a corrected NASA table.
No new accuracy or corrected Flame Design score is claimed here.

The eighteen reported Flame Design disagreements are not automatically eighteen
bad sensor measurements. A missing distinction in a log parser can create an
apparent disagreement. Each source assertion, observable, clock and censoring
state must remain visible. A new method/endpoint definition gets a new version;
the historical registered result is retained. No protected per-test inspection
or unregistered validation look is authorized by this remediation.

## Executable continuity and the reported target deletion

Fable reports that the shared build directory disappeared mid-run and that a
saved executable reproduced all TRAIN artifacts. That report does not identify
the deleting actor. Astra has not established an actor from filesystem times
or the presence of a later rebuild and makes no attribution.

An immutable, hash-addressed executable snapshot helper and prelaunch/finalization
checks are implemented in Astra's two runners. Nine tests passed, including an
actual temporary executable subprocess, source replacement, snapshot tampering,
concurrent capture and the TRAIN runner's overwrite guard. Details and limits
are in [`BINARY_SNAPSHOT.md`](../measure/BINARY_SNAPSHOT.md). No model feedback
was rerun. Shared `target/`, another worker's binary, releases and feedback
receipts are not deleted or overwritten. Old run receipts remain historical;
they are not relabelled as having used the new snapshot route. Static inspection
of the changed production helpers found no shared-target cleanup operation;
that does not identify the actor behind an earlier deletion.

## Model interpretation and remaining scientific work

The registered pooled interval remains the primary statistic for the historical
transfer result; choosing another interval after seeing its sign would change
the experiment. A surrogate-fuel result does not establish physical ethylene
support, calibrated uncertainty, superiority to the previous model or equality
to the reference. These are distinct claims requiring distinct evidence.

The all-NASA work addresses physical fuel inputs and prospective family models.
New numerical W0 candidates need fresh screened contexts. Later data-informed
candidates use TRAIN evidence; releases and evaluation rules must be committed
before any new authorized aggregate look. Negative and undecided results stay
in the dossier rather than becoming targets for a retrospectively chosen test.
