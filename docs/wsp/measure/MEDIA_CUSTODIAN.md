# Blind media custody

This is a standalone working pipeline, not a shared IGNIS contract. The custodian
selects the lexicographically first explicitly TRAIN run in the pinned existing FLEX
split, joins its condition-catalogue `cir_test_point` alias to the source catalogue,
and selects its backlit camera archive, index and instrument settings. No diameter
reference, outcome series, track values or model output is read. Original IDs,
partition, URLs and the redaction/alias map remain in
`runs/dev/data_astra/work/media_routing/`; that directory is not a measurer input.

`tools/wsp/analysis/lib.py` was inspected before using metadata. Its generic series
reader parses rows before selecting IDs, so it is not used to open any outcome
file here. The custodian's own gate verifies the split digest and TRAIN membership
before source access. Selection uses identity order, never visible quality or an
outcome. The source's camera settings corroborate the acquisition date in the
frozen identity; a differing folder date is not used to change its group.

The initial 5,982,225,940-byte reservation covered one archive, two instrument files
and transient workspace. The owner and download custodian subsequently replaced it
with a **15,000,000,000-byte total media reservation**, including the same original,
all native copies, both observers' outputs and evidence, and all workspace. The
coordinated shared accounted peak is 29,899,938,697 bytes, below the owner's 40 GB ceiling;
10 GiB real free space is reserved. Media is separate from the concurrent
document/sensor download area. Every source has listed and actual byte counts,
URL, original file identity and SHA-256 in the private fetch receipt. A locally
computed digest pins received content; it is not an independently supplied NASA
checksum. No second archive is fetched while this one is retained without new
storage accounting.

Blind packets use unpredictable aliases, a strict field allowlist, a frozen
method digest and opaque lossless image filenames. No run, day, fuel, partition,
expected value, reference result or model enters a packet. Images preserve native
pixel coordinates; any crop or mask must be recorded privately and represented
explicitly in instrument geometry. No generated reconstruction, rescaling,
interpolation or content-aware filling is permitted. Original audio, captions,
titles and embedded metadata never enter a packet. A condition overlay over the
measurement region makes that region unavailable.

The initial manual method records liquid silhouette horizontal and vertical
extents with visible edge-ambiguity intervals in pixels. It does not fit a circle,
normalize intensity, choose a threshold from reference values, or reconstruct an
occluded perimeter. Absent/ambiguous features produce missing values with reasons.
The frame sample is systematic across native index order and explicitly partial;
the complete archive remains retained. Native frame-index observations are useful
evidence but cannot yet qualify dimensional diameter or time-dependent predictions.
Independent physical scale and verified acquisition-clock interpretation are
required before that qualification. Instrument codes without a manual are not
silently converted into physical exposure, gain or length.

The first packet receives independent A/B observations because the one-in-ten
schedule begins with its first item. Equivalent image bytes have different packet
aliases. Each measurer receives only its packet and a minimal instrument/method
instruction; it declares prior familiarity, sees no peer output, and writes sealed
observations. The parent coordinator receives completion/path receipts only. These
are discipline-enforced roles on a shared filesystem, not OS access isolation.

Source deletion is blocked until both independent measurement receipts are sealed
against that source digest and retained evidence is rehashed successfully. The
verification primitive is tested with synthetic files; no downloaded NASA source
has been deleted. Measurement qualification against authorized TRAIN references
belongs to a separate comparator after measurements and method are frozen.

Reproduce acquisition (uses the private pinned routing record):

```powershell
python tools/wsp/astra/media_custodian.py
python tools/wsp/astra/media_custodian.py --execute
python tools/wsp/astra/media_custodian.py --stage --execute
# Custodian screens every selected frame and writes its private overlay-review receipt.
python tools/wsp/astra/media_custodian.py --release-reviewed --execute
python -m unittest discover -s tools/wsp/astra -p test_media_custodian.py
```

The synthetic suite covers path escape, gate-before-source access, split hash drift,
blind-manifest field/value denial, and retention verification before deletion. Its
passing result establishes these mechanics only, not NASA measurement accuracy.

## First release, 2026-11-14

The single archive and two instrument records completed acquisition: 1,982,225,940
listed and received bytes. The archive's embedded frame index matches the separately
fetched index by SHA-256. Eleven frames selected by `round(i*(N-1)/10), i=0..10`
were decoded without rescaling, saved as lossless 16-bit PNG, and checked for exact
array equality. Each selected frame was visually screened for identifying overlays;
none required masking. This is a partial sample, with opaque full-artifact references
resolved only in the private routing store.

Two independently aliased packets are ready under
`runs/dev/data_astra/work/blind_tasks/`. Each contains `manifest.json`, `METHOD.md`,
the eleven image files and an empty `output/` directory for a fresh observer.
Delivered manifest, method and image hashes were rechecked after copying. Their
identities are recorded in the private release receipt; no original run identity or
condition enters either packet. Acquisition reports 12-bit sampling; stored TIFF
and PNG values are 16-bit with low four bits zero. The fixed display recipe is
`floor(stored_value/256)`, never per-frame normalization.

Retained logical bytes at release, including the original, private review copies
and both packets: 2,028,898,643. The full reservation remains held. No original was
deleted. No dimensional/time qualification, reference comparison, model score or
pixel measurement was performed by this custodian. The six synthetic tests passed
with resource warnings treated as errors:

```powershell
python -W error::ResourceWarning -m unittest discover -s tools/wsp/astra -p test_media_custodian.py
```

## Sealed pilot comparison and continuation

Both pilot submissions and every evidence artifact were hash-pinned before
comparison, then reverified afterward. A frozen descriptive protocol matches source
pixel digest, native frame index, method, pixel unit and observable. It retains A
and B separately, reports missingness and per-observable differences, and propagates
their operator-bound intervals conservatively. These are bounded annotations, not
confidence intervals. No averaging replaces either measurement and no NASA reference
was opened. The protocol, immutable comparison, output seals and implementation
snapshot remain in the private routing store. Distinct observer contexts are
corroborated by the controller's agent inventory; identical declared model families
and unknown exact builds limit independence claims. One run is descriptive
repeatability, not NASA accuracy or between-run qualification.

The complete native inventory contains 1,468 frames, all joined to the instrument
index. The eleven pilot observations remain sealed. A 16 MiB staging budget selected
the first seven pending native indices, without outcome or quality filtering.
Native arrays are decoded one at a time and checked for lossless equality. Subsequent
advancement requires both exact-coverage submissions and verified retained evidence.
The total media ceiling and 10 GiB free-space reserve are checked before each batch.
No original archive is deleted while intended coverage or qualification is unresolved.

The first continuation dispatch reused the pilot observers. Both recognized the
pictured acquisition from their earlier opaque packet and stopped before measuring.
The frozen rule is: "Declare prior familiarity with the pictured acquisition before
beginning; if familiar, stop and return only an exposure-declaration receipt."
Their receipts report within-task familiarity, not outside test identification,
conditions, partition, predictions or peer exposure. The literal rule applies and
was not relaxed retrospectively. Stopped outputs are preserved and hash-pinned;
unchanged screened pixels were re-aliased for genuinely fresh contexts. Future
dispatch must respect this same context eligibility rule.

Independent scale research inspected exact instrument settings/indexes, seventeen
small calibration-only text objects, and screened apparatus pages. No usable
experiment-specific length coefficient or target geometry was established. Nominal
field-of-view/resolution specifications and a different binned camera configuration
are not substituted for calibration. Both native time fields are retained; their
association, synchronization and exposure-time meaning remain unresolved. The
frozen pixel/frame-index method is unchanged. The source-reading ledger records the
precise pages and scope privately; mixed document bodies are not released to authors.

The owner subsequently prioritized acquisition and page qualification. Both fresh
replacement observers completed all seven staged frames. Input, method, evidence,
output seals, stopped receipts and the retained source digest were verified. The
checkpoint now records 18 of 1,468 native frames sealed, with 1,450 remaining, and
is explicitly paused. No further batch starts until controller resumption.

The replacement observers used descriptive runtime-context prose rather than a
shared machine identifier. Their sealed prose and exposure declarations remain
unchanged. A separate hash-bound controller receipt records their actual distinct
fresh task identities and `fork_turns=none`. It also corrects the frozen comparison
protocol's stale pilot-context list without modifying that protocol or its metrics.
The comparator accepts this explicit binding, rejects mismatched packet/submission
identities and affirmative familiarity, and preserves both original declarations.

Retained logical bytes at this checkpoint were 2,119,813,129 before writing the
small storage receipt. The 15 GB reservation remains held; originals, stopped
attempts, both completed submissions and private comparison remain retained. No
NASA reference values were opened and qualification remains pixel/frame-index only.

Additional executed checks: eight custody tests and six comparator tests passed.
The comparator tests reject mismatched units, methods, source pixels and frame
coverage, invalid bounds, shared observer context, and sealed-output mutation.

```powershell
python -W error::ResourceWarning -m unittest discover -s tools/wsp/astra -p test_compare_blind_measurements.py
python tools/wsp/astra/compare_blind_measurements.py
# After explicit controller resumption and completion-receipt verification:
python tools/wsp/astra/media_custodian.py --stage-continuation --workspace-bytes 16777216 --execute
```

The comparison command uses exclusive creation and refuses to overwrite an existing
comparison. A paused continuation fails closed until the controller explicitly
resumes it. Generated records are working artifacts, not replacements for shared
IGNIS contracts.
