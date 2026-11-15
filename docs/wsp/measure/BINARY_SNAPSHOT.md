# Executable identity independent of shared build outputs

`tools/wsp/astra/binary_snapshot.py` captures a stable executable into a
SHA-256-addressed directory outside Cargo's mutable build output. The Astra
candidate and TRAIN-feedback runners now launch that captured executable. A
subsequent build, replacement, or disappearance of the original executable does
not change the selected bytes for an already-preflighted job.

This is a reproducibility and concurrency change. It changes no candidate
weights, release definitions, partitions, scoring arguments, or scientific
interpretation. The existing candidates remain data-informed, not zero-shot.
Historical releases and execution receipts are not retroactively rewritten.

## Capture and selection

The default store is `runs/tooling/astra-binaries/`, with layout:

```text
<store>/<executable-sha256>/
    executable.exe       # executable on Windows; named executable elsewhere
    snapshot.json
```

The helper rejects a store beneath the repository's `target/`, Astra's
`runs/tooling/astra-target/`, and the current `CARGO_TARGET_DIR`, when configured.
A caller using the lower-level helper may
provide additional target roots explicitly. No build is triggered, and no build
directory is cleaned. Store paths are resolved before this containment check.

Capture opens the source once, streams its bytes into a unique temporary
directory within the store, hashes the copied stream, flushes and synchronizes
the copy, then rereads the same open source. Device/inode, size, and modification
time must agree before and after, and the source path must still identify that
file. Both source passes must have the same hash and byte length. The copy is
then reopened and hashed independently. This avoids hashing one source version
and silently copying another. A source changing during capture fails preflight;
it does not cause an automatic retry against a different build.

The helper publishes the complete directory with a same-filesystem rename.
Executable and manifest are made read-only. Concurrent identical captures verify
and reuse the existing complete directory; they never overwrite its executable
or manifest. A corrupt existing snapshot is an error, not a repair operation.
Only newly created staging directories are automatically removed. Existing
snapshot directories and binaries are never deleted by this helper.

The manifest records its schema, executable filename, SHA-256, byte length,
absolute original source path, and known build identity or explicit `null`.
Build identity is supplied by the operator, not guessed from the current Git
checkout, timestamps, or the existence of a fresh build. A supplied identity
should identify the actual build inputs; a dirty tree's current commit alone
does not establish those inputs.

When identical bytes already exist, their first-capture provenance stays
immutable. An explicitly supplied build identity conflicting with that manifest
(including a previously unknown identity) is refused. The helper cannot enrich
an old immutable manifest in place; a separate store can preserve a separately
documented capture if needed.

## Runner integration

Both runners accept these arguments:

- `--binary PATH`: capture that source before starting work.
- `--binary-snapshot DIRECTORY`: load and verify a previously captured hash
  directory. This is mutually exclusive with `--binary` and never falls back to
  a source executable.
- `--snapshot-directory DIRECTORY`: set the capture store; default shown above.
- `--build-identity TEXT`: record a known identity during capture, or require an
  exact manifest identity when selecting an existing snapshot. Omission means
  the caller supplies no additional build-identity assertion.

The helper can also preflight bytes without starting any candidate or feedback
work: `python tools/wsp/astra/binary_snapshot.py capture <source-executable>`.
Its `verify <snapshot-directory>` action checks an existing snapshot without
launching it. These actions emit the binary identity receipt as JSON.

Without either selection argument, `train_feedback.py` captures
`runs/tooling/astra-target/release/wsp-dev.exe` once before creating its worker pool.
`candidates.py` captures `runs/tooling/astra-target/release/ignis.exe` before its selected
action. Programmatic candidate CLI calls lazily capture once if the caller has
not already configured a snapshot. Threads in a TRAIN invocation share the
same verified snapshot object.

For a future authorized invocation, an existing snapshot can be selected using
`--binary-snapshot runs/tooling/astra-binaries/<sha256>`. Preparing this change
does not authorize rerunning historical feedback. Existing output/receipt
overwrite guards remain in force; this implementation did not rerun feedback.

Every child launch verifies executable SHA-256/size and manifest SHA-256 before
launch and in a `finally` block before returning, including unsuccessful child
launches. Shell or alternate-executable overrides are refused. Runner
finalization also verifies the snapshot. A changed or missing snapshot fails;
the runner never substitutes `target` or another available build.

New TRAIN execution receipts include `execution_binary`, containing the snapshot
and manifest paths, both hashes, size, original source path, and build identity.
Successful TRAIN receipts are written only after the existing TRAIN-only report
check and final snapshot verification. New assembly receipts include the same
identity block. Existing assembly receipts remain unchanged when inspected.
Both command-line entry points print the selected binary identity before work.

If an integrity failure occurs after a child has produced files, those files
are left for inspection; they are not automatically deleted, relabelled as a
successful execution, or rerun. A receipt records verified binary bytes, not a
claim that the build's source identity, scientific result, or complete execution
environment was independently established.

## Scope of protection

Read-only files plus before/after hashes protect this workflow against ordinary
shared-target rebuilds and detect changes to the captured file. They are not an
operating-system isolation boundary against an owner who can replace a snapshot
and restore it between checks. The process is launched by snapshot pathname;
this implementation does not claim an atomic kernel verify-and-execute primitive.
The executable is snapshotted, not its external DLLs, libraries, environment,
datasets, or model artifacts. Those remain separate reproducibility inputs.

On the tested Windows/Python installation, `stat` and `fstat` could report
different `ctime` meanings for the same unchanged file. Capture therefore uses
device/inode, size, modification time, and repeated content hashes, rather than
comparing those inconsistent `ctime` values. This does not infer a deletion or
modification actor from a timestamp.

For future builds needing independent build outputs, the owner may choose an
owned `CARGO_TARGET_DIR` before building and then capture its executable. That
can avoid build-output interference, but allocating or rebuilding a separate
large target tree is outside this change and was not done here.

## Static deletion audit and verification

Inspected the four owned Python sources: `binary_snapshot.py`,
`test_binary_snapshot.py`, `train_feedback.py`, and `candidates.py`. Neither
production runner nor the snapshot helper contains shared-target cleanup or
target-executable deletion. `candidates.py` already used `TemporaryDirectory`
for newly staged CLI outputs. The snapshot helper also removes only its own
new staging directories. The tests remove only checked temporary fixture files
and temporary test directories. This narrow audit does not establish which
process affected an earlier shared build directory; no actor is inferred from
mtime, a fresh build, or absent bytes.

Executed verification:

```powershell
$env:PYTHONDONTWRITEBYTECODE = '1'
python -m unittest discover -s tools/wsp/astra -p test_binary_snapshot.py -v
```

All nine tests passed. They cover hash-addressing/provenance, subsequent source
replacement, concurrent capture without overwrite, corruption without repair,
source changes during capture, prelaunch/finalization checks, invalid selection,
target containment, and forbidden execution overrides. A synthetic TRAIN-runner
integration fixture confirms that the new receipt contains the selected binary
identity and that existing-output guards prevent a second execution. Its child
call is a stub writing a temporary synthetic report, not an IGNIS scorer.
One test copies the
installed Python interpreter into a temporary source, captures it, deletes only
that temporary source, then runs a literal print statement using the snapshot.
The fixture supplies Python's runtime path explicitly; it is not a claim that
all executables are standalone.

The `--help` paths for all three command-line entry points also ran successfully.
No real IGNIS model, candidate assembly, scoring, feedback, dataset, or experimental
outcome was executed or read for these tests. No Cargo build or heavy new build
directory was created. The physical-core source/document were unchanged during
this task.
