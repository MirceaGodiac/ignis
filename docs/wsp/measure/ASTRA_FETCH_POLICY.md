# Astra public-corpus acquisition policy

Owner-authorized task, 14 November 2026: acquire document/table and sensor/settings files for all
19 flight combustion investigations, without exposing their bodies to model authors. This work
changes no model, measurement, split, shared contract or existing ingestion implementation.

## Scope and separation

`tools/wsp/astra/fetch_public.py` reads PSI listing metadata and downloads opaque byte streams.
The source endpoint and request shape follow `crates/ignis-data/src/sources/psi.rs`; existing shared
CLI/source code is unchanged. Downloads require `--execute`. Optional live refresh accesses only
investigation version and file-list metadata. Cached listing inputs are the four known
`data/manifests/PSI-*.listing.jsonl` files and the fifteen corresponding listing files under
`runs/dev/data_all/manifests/`. No experiment tables or outcomes are parsed.

Investigations: PSI-69, 159, 25, 26, 68, 39, 10, 23, 20, 21, 22, 107, 106, 98, 99, 100, 102, 101, 47.
Selected categories: Science Documents, Reports, Presentations, Engineering Documents, Analyzed
Data, Reference Data, Experimental table, Raw Data. All use the explicit case-insensitive suffix
filter `pdf|doc|docx|ppt|pptx|xls|xlsx|csv|txt|rtf|pxp`. Restricted entries are not fetched.

Payloads live only in `runs/dev/data_astra/corpus/incoming/PSI-<id>/`, with identity-hash filenames.
Original names and identities remain in metadata/receipts. This incoming area is an opaque corpus,
**not an author-readable dataset**. Downloading does not grant permission to read a payload. Model
authors must not inspect these bodies. New-investigation outcomes remain inaccessible until the
owner's split file exists and the applicable train/validation/test rules admit access. Measurement
handoff must separately enforce blinding and partition isolation; this script does not implement
that handoff. Nothing here reads a holdout, validation series or per-run validation outcome.

## Integrity, resumption and budgets

The full selected listing is written before payload requests. Exact duplicate identities are
collapsed; conflicting metadata for one identity fails. Identity includes investigation, remote
key, file version, declared byte size and PSI file id. Stable PSI URLs are recorded; signed redirect
URLs and HTTP error bodies are not logged.

An append-only receipt includes metadata identity, declared bytes, actual bytes, SHA-256 when
available, selected response headers, state, timestamp and kept/partial/not-downloaded disposition.
Incomplete `.part` files are preserved. Range resume requires an ETag and verified Content-Range;
an ignored Range safely restarts the partial. Downloads enforce listing/HTTP lengths, hash the
stream, independently reread and hash the completed file, and atomically promote it only when both
checks agree. A rerun rereads each completed file and compares length/hash against its receipt.
Mismatching existing completed files remain kept for review and are not overwritten. Interrupted
or failed files remain explicitly incomplete. Local SHA-256 establishes identity and readback
integrity; it is not an independent publisher checksum or semantic validation.

The execution uses sixteen workers, a 37 GiB ceiling (39.73 decimal GB, below the owner's 40 GB peak)
and at least 10 GiB free disk reserve. In-flight payload reservations, retained bytes and actual
free space are checked. Other agents can consume disk concurrently; free space is rechecked while
streaming. Receipts/metadata are small additional overhead inside the 0.27 GB margin. No bulk
images, video or calibration archives are selected. Completed payloads are never automatically
deleted. An oversized failed `.part` owned by this downloader is path-checked, hashed, removed and
logged as `deleted_invalid_partial` before restarting its transfer.

## Coverage accounting and excluded archives

The prompt's 636 files / 1.63 GB is a historical estimate, not a completeness criterion. The initial
cached extension-filter plan found 538 document-category rows and 24,754 Raw Data rows. Exact
identity deduplication removed 483 rows: 24,809 unique files / 13,810,402,622 declared bytes. Live
metadata refresh may change these totals; the current `metadata/fetch_plan.json` is authoritative
for a particular execution, and `logs/acquisition_inventory.json` reports verified acquisition.

The category-wide metadata also contains 104 ZIP and 15 MP4 entries excluded by the explicit
filter. `metadata/excluded_document_category_files.jsonl` accounts for them.
`astra_download_inventory.py` writes counts by category/extension and a filename-only
`metadata/archive_triage.jsonl`: likely document bundles (drawings, instrument/data-interpretation
documents, heater technical notes), likely media archives (images/photos), uncertain archives
(including generic fiber and imaging names), and direct media files. These classifications are
unverified; no member list or payload was opened. Pending ZIPs mean acquisition must not be called
complete coverage of every document/analysed-data asset.

## One-archive-at-a-time lifecycle (designed, not executed)

1. Select one remote archive from metadata and reserve its declared compressed bytes plus a bounded
   extraction workspace within the same peak/free-space budgets. Keep its stable identity and hash.
2. Download it into a separate opaque incoming archive slot. Verify length and stream/readback hash.
3. A separately authorized blind measurement/curation worker checks archive member metadata,
   traversal paths, symlinks and declared unpacked sizes before extraction; no bulk extraction.
4. Stream only required members into the bounded workspace. A measurer receives instrument data
   without a model, hypothesis, expected value or partition. Author access remains a separate gate.
5. Keep measurement artifacts, method/version/uncertainty provenance and only explicitly selected
   evidence frames. Verify their readback and provenance before removing the source archive or
   extracted workspace. Any deletion must verify absolute paths remain inside this archive slot,
   use the same native filesystem implementation throughout, and append a deletion receipt with
   prior content hash/bytes and retained-artifact references.
6. Confirm the slot is empty and account for retained evidence before taking the next archive.

An archive requiring more transient space than the budget supports is blocked, not silently
truncated. Unknown decompressed size stays unknown until member metadata is inspected. Sequential
media processing is not implemented or run by this document/text acquisition task.

## Reproduction and evidence

```powershell
python tools/wsp/astra/astra_download_test.py
python tools/wsp/astra/fetch_public.py
python tools/wsp/astra/fetch_public.py --refresh --metadata-only --execute --peak-gib 37
python tools/wsp/astra/fetch_public.py --execute --peak-gib 37 --workers 16
python tools/wsp/astra/astra_download_inventory.py
python tools/wsp/astra/astra_download_probe.py --execute
python tools/wsp/astra/astra_download_quarantine.py --execute
```

The offline eleven-test synthetic suite passed: completed-file corruption, resume validators/range,
server ignoring Range, bad Content-Range, length mismatch, resource refusal and oversized failed
partial recovery, restart of a full partial lacking a verification receipt, inconsistent range ends,
and connection-session replacement after a transport error. No NASA payload is
used in tests. The alias test also rejects wrong versions, hosts and lengths. Python compilation
and dry-run planning also passed. No Rust/shared contracts were
changed, and no repository-wide build/test gate is claimed. Live completion/failures are recorded
under `runs/dev/data_astra/logs/`; this policy does not assert a finished fetch before its receipts.

The complete live metadata refresh finished successfully on 14 November 2026. It selects 24,807
unique files / 13,809,295,236 bytes under the explicit filter, with 119 document-category archive/
media exclusions. All 19 `metadata/PSI-<id>.refresh_status.json` files record the refreshed listing
hash and investigation version. Initial duplicate acquisition processes were stopped; a Windows
single-process lock now prevents overlap, and the resumed run verifies completed files against
successful receipts. Two oversized partials from that overlap are handled by the logged recovery
rule above. Some stable PSI download URLs return 404 or a length that disagrees with their listing;
these remain failed rather than being accepted as verified corpus files.
Header-only diagnostics in `logs/header_probes.json` read zero response-body bytes. Repeating the
seven document failures both with their declared version and without a version reproduced six
404s and the same PSI-20 size disagreement (5,794,678 listing bytes versus 6,843,254 HTTP bytes).
The redirected object path and safe headers are recorded without signed URL query credentials.
The size-disagreeing PSI-20 response was subsequently retained as opaque bytes in
`corpus/quarantine/`, with its own transport/readback SHA-256 and 6,843,254 actual bytes in
`logs/size_mismatch_quarantine.json`. It remains ineligible as a verified listing file; the
5,794,678-byte declaration is retained alongside the disagreement. No semantic content was read.

The active fetch lock was also checked with a second execution: it refused before payload network
access with `another acquisition process owns this incoming corpus`, as intended.

A separate media custodian was subsequently authorized by the parent task to acquire one confirmed
TRAIN source. Shared resource accounting reserves the entire document/sensor selection
(13,809,295,236 bytes), quarantined PDF (6,843,254), overhead margin (1,000,000,000), and that
custodian's single archive/index/settings plus at most 4,000,000,000 workspace bytes
(5,982,225,940 total). The combined accounted peak is 20,798,364,430 bytes, below 40 decimal GB.
This does not authorize bulk media; the separate custodian owns its source and measurement handoff.

## Authoritative server-list reconciliation

The owner subsequently named `docs/wsp/atlas/server_documents.csv` and
`server_sensor_text.csv.gz` as authoritative acquisition metadata. Only their metadata rows were
read, together with the catalogue's `FILES.md` classification/count explanation. The reconciliation
script records input SHA-256 values, exact file ids/versions/names/sizes, original-filter overlap,
duplicates and every additional/deferred identity under `metadata/server_reconciliation*.json`.

The original 24,807-object selection is an exact subset of the 25,118 unique authoritative objects.
The 311-object / 76,428,232-byte difference is fully accounted:

| Reason | Files | Declared bytes |
|---|---:|---:|
| Calibration category excluded by original category filter: 97 TXT + 1 PDF | 98 | 10,127,367 |
| Raw Data `.log` excluded by original extension filter | 211 | 65,193,479 |
| Two PSI-26 report version-2 identities in the authoritative CSV and historical listing, absent from the refreshed listing | 2 | 1,107,386 |

The absent reports are `PSI-26_Reports_BASS_FinalReport.pdf` and
`PSI-26_Reports_SPICE_FinalReport.pdf`, each declared 553,693 bytes; no alternate current version of
either name appears in the fresh listing. They remain requested by explicit version, not silently
replaced. The authoritative CSVs are already deduplicated: 636 documents / 1,629,828,160 bytes and
24,482 sensor/text files / 12,255,895,308 bytes. The combined target is 13,885,723,468 bytes.
The original filter collapsed 483 repeated listing rows. The fresh full listing has 66,499 rows /
60,798 distinct metadata identities versus the catalogue's historical 66,501 / 60,800, explained
by the two absent PSI-26 reports.

All 311 additions are safe document/text suffixes and are written to
`metadata/server_extra_files.jsonl`. After the active original pass finishes, execute:

```powershell
python tools/wsp/astra/astra_download_reconcile.py
python tools/wsp/astra/fetch_public.py --include-server-additions --execute --peak-gib 37 --workers 4
```

The combined reservation rises by only 76,428,232 bytes to 20,874,792,662 bytes. The 119 excluded
ZIP/MP4 category entries are absent from both authoritative CSVs, so they do not explain the
311-object difference. Their separate filename-only triage remains pending the sequential archive
workflow; matching the CSVs does not establish completeness of all documents inside those archives.

The media custodian later requested retaining all native frames and both observers' evidence from
the same single FLEX source. Its reservation was increased to **15,000,000,000 bytes total**,
including originals, frames, all working copies and evidence; it replaces the earlier 5.98 GB
reservation. With the authoritative corpus, current quarantined responses (14,215,229 bytes), and
the 1 GB overhead margin, the accounted peak is 29,899,938,697 bytes. The custodian must enforce its
total cap and 10 GiB free-space reserve; this accounting authorizes no second media archive.

## External-service interruption and exact resumption

On 14 November 2026, the original pass verified **24,701 files / 13,618,490,964 bytes**. Its last
successful receipt was at 17:14:52 UTC. Subsequent requests accumulated HTTP 504 gateway timeouts;
independent zero-body probes of PSI-69 and PSI-101 both timed out. The owned sixteen-worker process
was stopped at 17:23:30 UTC to avoid continuing concurrent requests into the outage. Partial files,
receipts, quarantine and every verified payload were preserved.

`logs/fetch_interruption.json` records that interruption. Of the complete authoritative 25,118-file
target, 417 remain unresolved: 78 exhausted failures, six interrupted retries and 333 unattempted
files. The latter comprise 22 original-pass files and the 311 authoritative additions. The four
quarantined responses total 14,215,229 actual bytes and remain ineligible as verified listing files.
These are operational acquisition states, not scientific validation or complete-corpus claims.

`astra_download_availability.py --execute` permits one header-only probe at a time, guarded by its
own Windows lock and persisted 60/120/240/300-second failure backoff. Each probe has bounded
connection/read timeouts, follows redirects sequentially, and reads no response body. A successful
response must reach the expected object-store host with the expected small-object length. Probe
receipts are in `logs/availability_checks.jsonl`; the next permitted time is in
`logs/availability_state.json`.

When service responds again, resume at lower concurrency:

```powershell
python tools/wsp/astra/astra_download_availability.py --execute
python tools/wsp/astra/fetch_public.py --include-server-additions --execute --peak-gib 37 --workers 4
python tools/wsp/astra/astra_download_inventory.py
```

The second command is executed only after an availability probe reports healthy. It verifies the
exact authoritative identity-set digest, then locally rechecks existing length/SHA-256 receipts;
already verified matching files cause no payload request. It retries incomplete/failed transfers
and adds the previously unattempted identities. `logs/fetch_run_status.json` distinguishes active,
finished and externally interrupted acquisition. The outage does not reduce the requested scope.

NASA recovered for the zero-body probe at 18:32 UTC. After an interrupted agent session, the
previous process was confirmed absent rather than inferred live from its stale `running` record.
A second healthy probe at 18:39 UTC preceded the four-worker restart from the transferred IGNIS
checkout; its `runs` junction retains the same corpus and receipts. These checkpoints do not
change any source identity or hash.

The inventory command also writes `logs/acquisition_objects.jsonl`, one metadata-only record for
every selected identity, and `logs/acquisition_unresolved.jsonl`, its unresolved subset. Each row
includes the latest acquisition state, receipt timestamp, declared/actual bytes, hash when verified,
kept/deleted disposition, and an explicit unresolved reason (not yet attempted, HTTP status,
transport error type, integrity disagreement, or resource refusal). The manifests are snapshots,
not proof that a process is live; check the recorded PID against actual processes before restart.

## Recovered acquisition checkpoint

The resumed pass ended at 18:43:54 UTC. Follow-up sequential zero-body diagnostics checked every
remaining failure both with its declared version and without a version. One additional schematic
was recovered using the exact full filename from its catalogue key: the listing's basename had
omitted a prefix containing a slash. `astra_download_recover_alias.py --execute` admits only a
successful probe whose URL exactly matches that metadata key and selected version, whose final
host is the object store, and whose HTTP length matches the declaration. It retains the same
selected identity, original listing URL and version and uses the normal lock/readback verifier.

At 18:49 UTC, **25,088 of 25,118 identities were verified**, totaling **13,824,894,644 bytes**.
Of the original 417 unresolved objects, 387 are now verified and 30 remain unresolved:

- 19 return HTTP 404 for both declared-version and unversioned URLs.
- 11 return bytes whose HTTP lengths disagree with the listings. All eleven responses are retained
  separately in quarantine, totaling 32,197,350 actual bytes, with independent readback hashes;
  none is promoted to a verified listing identity.

`logs/interruption_417_resolution.jsonl` maps each original pending identity to its current state.
`logs/acquisition_unresolved.jsonl` contains the remaining reasons and separate quarantine
locators/hashes where applicable. `logs/acquisition_checkpoint.json` records the aggregate snapshot.
No acquisition worker remained live after these operations. The accounted peak reservation,
including the unchanged 15 GB media reservation and 1 GB overhead, is 29,917,920,818 bytes;
free disk was 188,451,565,568 bytes. The 119 excluded ZIP/MP4 entries remain separately inventoried,
so this checkpoint is neither a complete-corpus claim nor permission to read scientific payloads.
