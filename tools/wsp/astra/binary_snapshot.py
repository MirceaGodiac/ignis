"""Capture and verify executable bytes independently of mutable Cargo target directories.

Snapshots are immutable by convention and read-only on disk, not a security boundary
against the owning user. Acquisition checks the open source twice and reads back the
copy. Execution verifies the snapshot before launch and before returning any result.
"""
from __future__ import annotations

import argparse
from dataclasses import dataclass
import errno
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[3]
DEFAULT_STORE = ROOT / "runs/tooling/astra-binaries"
MANIFEST = "snapshot.json"
EXECUTABLE = "executable.exe" if os.name == "nt" else "executable"
SCHEMA = "ignis.executable-snapshot.v1"


class SnapshotError(RuntimeError):
    """Binary provenance or representability could not be established."""


def _identity(info):
    # Windows Python can expose different ctime semantics through stat and fstat;
    # inode/device, length, modification time and byte rehash establish this check.
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns)


def _digest(stream):
    digest = hashlib.sha256()
    size = 0
    # An I/O buffer, not a file-size limit.
    for block in iter(lambda: stream.read(1024 * 1024), b""):
        digest.update(block)
        size += len(block)
    return digest.hexdigest(), size


def _checked_digest(path: Path):
    if path.is_symlink():
        raise SnapshotError(f"snapshot file must not be a symlink: {path}")
    with path.open("rb") as stream:
        before = os.fstat(stream.fileno())
        if not stat.S_ISREG(before.st_mode):
            raise SnapshotError(f"not a regular file: {path}")
        result = _digest(stream)
        after = os.fstat(stream.fileno())
        if _identity(before) != _identity(after) or _identity(after) != _identity(path.stat()):
            raise SnapshotError(f"file changed during verification: {path}")
    return result


def build_target_roots():
    roots = [ROOT / "target", ROOT / "runs/tooling/astra-target"]
    if configured := os.environ.get("CARGO_TARGET_DIR"):
        path = Path(configured)
        roots.append(path if path.is_absolute() else ROOT / path)
    return tuple(path.resolve() for path in roots)


def _outside_targets(path: Path, target_roots):
    for target in target_roots:
        if path.is_relative_to(Path(target).resolve()):
            raise SnapshotError(f"executable snapshot must be outside build target: {path}")


@dataclass(frozen=True)
class BinarySnapshot:
    directory: Path
    sha256: str
    size_bytes: int
    source_path: str
    build_identity: str | None
    manifest_sha256: str

    @property
    def path(self):
        return self.directory / EXECUTABLE

    def receipt(self):
        return {
            "schema": SCHEMA,
            "snapshot_path": str(self.path),
            "manifest_path": str(self.directory / MANIFEST),
            "manifest_sha256": self.manifest_sha256,
            "sha256": self.sha256,
            "size_bytes": self.size_bytes,
            "source_path": self.source_path,
            "build_identity": self.build_identity,
        }

    def verify(self):
        if self.directory.resolve() != self.directory or self.directory.name != self.sha256:
            raise SnapshotError("snapshot directory identity changed")
        digest, size = _checked_digest(self.path)
        manifest_digest, _ = _checked_digest(self.directory / MANIFEST)
        if (digest, size) != (self.sha256, self.size_bytes):
            raise SnapshotError(f"snapshot executable changed: {self.path}")
        if manifest_digest != self.manifest_sha256:
            raise SnapshotError(f"snapshot provenance manifest changed: {self.directory}")

    def run(self, arguments, **kwargs):
        """Return only after final integrity verification, including failed launches.

        Never execute the mutable source path. shell/executable overrides are refused.
        A post-run integrity failure raises even if the child returned success.
        """
        if kwargs.get("shell") or "executable" in kwargs:
            raise SnapshotError("shell/executable overrides bypass snapshot identity")
        self.verify()
        try:
            return subprocess.run([str(self.path), *map(str, arguments)], **kwargs)
        finally:
            self.verify()


def load_snapshot(directory, *, target_roots=None):
    """Explicit selection requires a complete, correctly addressed snapshot; no fallback."""
    directory = Path(directory).resolve(strict=True)
    _outside_targets(directory, build_target_roots() if target_roots is None else target_roots)
    manifest_path = directory / MANIFEST
    manifest_digest, _ = _checked_digest(manifest_path)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if set(manifest) != {"schema", "sha256", "size_bytes", "executable", "source_path", "build_identity"}:
        raise SnapshotError("unexpected or missing snapshot manifest fields")
    if (manifest["schema"] != SCHEMA or manifest["executable"] != EXECUTABLE
            or not isinstance(manifest["sha256"], str)
            or len(manifest["sha256"]) != 64
            or any(c not in "0123456789abcdef" for c in manifest["sha256"])
            or type(manifest["size_bytes"]) is not int or manifest["size_bytes"] <= 0
            or not isinstance(manifest["source_path"], str)
            or not Path(manifest["source_path"]).is_absolute()
            or (manifest["build_identity"] is not None
                and (not isinstance(manifest["build_identity"], str) or not manifest["build_identity"].strip()))):
        raise SnapshotError("invalid snapshot manifest")
    snapshot = BinarySnapshot(directory, manifest["sha256"], manifest["size_bytes"],
                              manifest["source_path"], manifest["build_identity"], manifest_digest)
    snapshot.verify()
    return snapshot


def capture_binary(source, store=DEFAULT_STORE, *, build_identity=None, target_roots=None):
    """Copy one stable source into a complete atomically published hash directory.

    Existing snapshots/manifests are verified and reused, never overwritten or deleted.
    First-capture provenance remains authoritative when identical bytes are reused.
    A contradictory explicitly supplied build identity is refused, not silently replaced.
    """
    source = Path(source).resolve(strict=True)
    store = Path(store).resolve()
    targets = build_target_roots() if target_roots is None else target_roots
    _outside_targets(store, targets)
    if source.is_relative_to(store):
        raise SnapshotError("use explicit snapshot selection for a binary already in the store")
    if build_identity is not None and (not isinstance(build_identity, str) or not build_identity.strip()):
        raise SnapshotError("build identity must be nonempty or explicitly unknown")
    store.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".capture-", dir=store) as temporary:
        stage = Path(temporary)
        executable = stage / EXECUTABLE
        with source.open("rb") as original, executable.open("xb") as copied:
            before = os.fstat(original.fileno())
            if not stat.S_ISREG(before.st_mode) or before.st_size <= 0:
                raise SnapshotError("source executable must be a nonempty regular file")
            digest = hashlib.sha256()
            size = 0
            for block in iter(lambda: original.read(1024 * 1024), b""):
                copied.write(block)
                digest.update(block)
                size += len(block)
            copied.flush()
            os.fsync(copied.fileno())
            sha256 = digest.hexdigest()
            original.seek(0)
            reread = _digest(original)
            if (reread != (sha256, size) or _identity(before) != _identity(os.fstat(original.fileno()))
                    or _identity(before) != _identity(source.stat())):
                raise SnapshotError(f"source changed during snapshot capture: {source}")
        if _checked_digest(executable) != (sha256, size):
            raise SnapshotError("staged copy does not match captured source bytes")
        manifest = {"schema": SCHEMA, "sha256": sha256, "size_bytes": size,
                    "executable": EXECUTABLE, "source_path": str(source), "build_identity": build_identity}
        with (stage / MANIFEST).open("x", encoding="utf-8", newline="\n") as metadata:
            json.dump(manifest, metadata, indent=2)
            metadata.write("\n")
            metadata.flush()
            os.fsync(metadata.fileno())
        executable.chmod(stat.S_IRUSR | stat.S_IXUSR | stat.S_IRGRP | stat.S_IXGRP | stat.S_IROTH | stat.S_IXOTH)
        (stage / MANIFEST).chmod(stat.S_IRUSR | stat.S_IRGRP | stat.S_IROTH)
        destination = store / sha256
        try:
            # Both directories are on the store filesystem. Existing nonempty directories
            # cannot be replaced; competing identical captures verify the winner below.
            os.rename(stage, destination)
        except OSError as error:
            if error.errno not in (errno.EEXIST, errno.ENOTEMPTY, errno.EACCES) or not destination.is_dir():
                raise
        snapshot = load_snapshot(destination, target_roots=targets)
        if (snapshot.sha256, snapshot.size_bytes) != (sha256, size):
            raise SnapshotError("existing snapshot differs from captured executable")
        if build_identity is not None and snapshot.build_identity != build_identity:
            raise SnapshotError("existing immutable snapshot has a different or unknown build identity")
        return snapshot


def add_snapshot_arguments(parser: argparse.ArgumentParser):
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--binary", type=Path, help="source executable to snapshot before work")
    selection.add_argument("--binary-snapshot", type=Path, help="existing SHA-256 snapshot directory")
    parser.add_argument("--snapshot-directory", type=Path, default=DEFAULT_STORE)
    parser.add_argument("--build-identity", help="known source/build identity; omitted means unknown")


def select_snapshot(args, default_source):
    if args.binary_snapshot is not None:
        snapshot = load_snapshot(args.binary_snapshot)
        if args.build_identity is not None and snapshot.build_identity != args.build_identity:
            raise SnapshotError("selected snapshot does not match the requested build identity")
        return snapshot
    return capture_binary(args.binary if args.binary is not None else default_source,
                          args.snapshot_directory, build_identity=args.build_identity)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    actions = parser.add_subparsers(dest="action", required=True)
    capture = actions.add_parser("capture", help="snapshot bytes without launching the executable")
    capture.add_argument("source", type=Path)
    capture.add_argument("--snapshot-directory", type=Path, default=DEFAULT_STORE)
    capture.add_argument("--build-identity")
    verify = actions.add_parser("verify", help="verify an existing snapshot without launching it")
    verify.add_argument("directory", type=Path)
    args = parser.parse_args()
    selected = (capture_binary(args.source, args.snapshot_directory, build_identity=args.build_identity)
                if args.action == "capture" else load_snapshot(args.directory))
    print(json.dumps(selected.receipt(), indent=2))
