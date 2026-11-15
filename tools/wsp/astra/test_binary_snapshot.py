"""Synthetic temporary-file/subprocess checks; no IGNIS model or feedback is invoked."""
from concurrent.futures import ThreadPoolExecutor
from dataclasses import replace
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import binary_snapshot as snapshots
import train_feedback


class BinarySnapshotTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="ignis-snapshot-test-")
        self.root = Path(self.temporary.name).resolve()
        self.target = self.root / "target"
        self.target.mkdir()
        self.source = self.target / "synthetic.exe"
        self.source.write_bytes(b"synthetic executable bytes, never a model")
        self.store = self.root / "snapshots"

    def tearDown(self):
        self.temporary.cleanup()

    def capture(self, **kwargs):
        return snapshots.capture_binary(self.source, self.store,
                                        target_roots=(self.target,), **kwargs)

    def mutate(self, path, content):
        self.assertTrue(path.is_relative_to(self.root))
        path.chmod(stat.S_IRUSR | stat.S_IWUSR | stat.S_IXUSR)
        path.write_bytes(content)

    def test_hash_address_receipt_and_source_replacement_are_independent(self):
        original = self.source.read_bytes()
        snapshot = self.capture(build_identity="synthetic-build-only")
        self.assertEqual(snapshot.sha256, hashlib.sha256(original).hexdigest())
        self.assertEqual(snapshot.size_bytes, len(original))
        self.assertEqual(snapshot.directory.name, snapshot.sha256)
        self.assertEqual(snapshot.receipt()["source_path"], str(self.source))
        self.assertEqual(snapshot.receipt()["build_identity"], "synthetic-build-only")
        self.source.write_bytes(b"different synthetic build")
        snapshot.verify()
        later = self.capture()
        self.assertNotEqual(later.directory, snapshot.directory)
        self.assertEqual(snapshot.path.read_bytes(), original)

    def test_concurrent_identical_capture_never_overwrites_existing_snapshot(self):
        with ThreadPoolExecutor(max_workers=4) as pool:
            results = list(pool.map(lambda _: self.capture(), range(4)))
        self.assertEqual(len({item.directory for item in results}), 1)
        first = results[0]
        manifest = first.directory / snapshots.MANIFEST
        before = (manifest.read_bytes(), manifest.stat().st_mtime_ns, first.path.stat().st_mtime_ns)
        self.capture()
        self.assertEqual(before, (manifest.read_bytes(), manifest.stat().st_mtime_ns, first.path.stat().st_mtime_ns))
        with self.assertRaises(snapshots.SnapshotError):
            self.capture(build_identity="cannot-retroactively-change-manifest")

    def test_corrupt_existing_snapshot_fails_without_repair_or_fallback(self):
        snapshot = self.capture()
        self.mutate(snapshot.path, b"corrupt")
        with self.assertRaises(snapshots.SnapshotError):
            self.capture()
        self.assertEqual(snapshot.path.read_bytes(), b"corrupt")
        with patch.object(snapshots.subprocess, "run") as launch:
            with self.assertRaises(snapshots.SnapshotError):
                snapshot.run(["unused"])
            launch.assert_not_called()

    def test_source_changed_during_capture_is_refused_and_not_published(self):
        original_digest = snapshots._digest
        changed = False

        def mutate_before_source_reread(stream):
            nonlocal changed
            if not changed and Path(stream.name) == self.source:
                changed = True
                self.source.write_bytes(b"changed while capture held the source open")
            return original_digest(stream)

        with patch.object(snapshots, "_digest", side_effect=mutate_before_source_reread):
            with self.assertRaises(snapshots.SnapshotError):
                self.capture()
        self.assertFalse(any(self.store.iterdir()))

    def test_finalize_refuses_child_success_if_snapshot_or_manifest_changes(self):
        for artifact in ("executable", "manifest"):
            with self.subTest(artifact=artifact):
                self.source.write_bytes(f"unique synthetic {artifact}".encode())
                snapshot = self.capture()

                def successful_but_changed(command, **kwargs):
                    self.assertEqual(command[0], str(snapshot.path))
                    target = snapshot.path if artifact == "executable" else snapshot.directory / snapshots.MANIFEST
                    self.mutate(target, b"changed during synthetic subprocess")
                    return subprocess.CompletedProcess(command, 0, "ok", "")

                with patch.object(snapshots.subprocess, "run", side_effect=successful_but_changed):
                    with self.assertRaises(snapshots.SnapshotError):
                        snapshot.run(["unused"])

    def test_load_rejects_missing_wrong_address_and_changed_manifest(self):
        snapshot = self.capture()
        with self.assertRaises(FileNotFoundError):
            snapshots.load_snapshot(self.root / "missing", target_roots=(self.target,))
        forged = replace(snapshot, sha256="0" * 64)
        with self.assertRaises(snapshots.SnapshotError):
            forged.verify()
        manifest = snapshot.directory / snapshots.MANIFEST
        modified = json.loads(manifest.read_text())
        modified["size_bytes"] += 1
        self.mutate(manifest, json.dumps(modified).encode())
        with self.assertRaises(snapshots.SnapshotError):
            snapshots.load_snapshot(snapshot.directory, target_roots=(self.target,))

    def test_store_under_build_target_and_launch_overrides_are_refused(self):
        with self.assertRaises(snapshots.SnapshotError):
            snapshots.capture_binary(self.source, self.target / "snapshots", target_roots=(self.target,))
        snapshot = self.capture()
        for override in ({"shell": True}, {"executable": str(self.source)}):
            with self.assertRaises(snapshots.SnapshotError):
                snapshot.run([], **override)

    def test_actual_subprocess_uses_snapshot_after_temporary_source_disappears(self):
        # Only a copied Python interpreter executes a literal print statement. This is
        # a process-launch fixture, not a model/build/feedback operation.
        shutil.copyfile(sys.executable, self.source)
        snapshot = self.capture(build_identity="synthetic-subprocess-fixture")
        self.assertTrue(self.source.is_relative_to(self.root))
        self.source.unlink()
        env = os.environ.copy()
        env["PYTHONHOME"] = sys.base_prefix
        env["PYTHONDONTWRITEBYTECODE"] = "1"
        env["PATH"] = str(Path(sys.executable).parent) + os.pathsep + env.get("PATH", "")
        result = snapshot.run(["-c", "print('synthetic-snapshot-ok')"],
                              capture_output=True, text=True, env=env, cwd=self.root)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), "synthetic-snapshot-ok")
        self.assertEqual(result.args[0], str(snapshot.path))

    def test_train_runner_records_snapshot_identity_and_keeps_overwrite_guard(self):
        snapshot = self.capture()
        for relative, release_id in (("old/release.json", "synthetic-parent"),
                                     ("new/r1-astra/release.json", "synthetic-r1"),
                                     ("new/joint-astra/release.json", "synthetic-joint")):
            path = self.root / "configs/models" / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps({"id": release_id}), encoding="utf-8")
        out = self.root / "runs/dev/astra"
        out.mkdir(parents=True)

        def synthetic_report(arguments, **kwargs):
            # A stub report writes only a temporary fixture. No model process is run.
            output = Path(arguments[arguments.index("--out") + 1])
            self.assertTrue(output.is_relative_to(self.root))
            output.write_text(json.dumps({"partition": "train"}), encoding="utf-8")
            return subprocess.CompletedProcess(arguments, 0, "synthetic-only", "")

        with patch.object(train_feedback, "ROOT", self.root), patch.object(train_feedback, "OUT", out):
            with patch.object(snapshots.BinarySnapshot, "run", side_effect=synthetic_report) as launch:
                receipt = train_feedback.run_case(("synthetic-case", "old", "new"), snapshot)
                self.assertEqual(receipt["execution_binary"], snapshot.receipt())
                self.assertEqual(receipt["command"][0], str(snapshot.path))
                self.assertEqual(receipt["candidate_status"], "data-informed; not zero-shot")
                saved = out / "train_synthetic-case_execution.json"
                before = saved.read_bytes()
                with self.assertRaises(RuntimeError):
                    train_feedback.run_case(("synthetic-case", "old", "new"), snapshot)
                self.assertEqual(saved.read_bytes(), before)
                self.assertEqual(launch.call_count, 1)

    def test_receipt_guard_resolves_real_junction_and_still_rejects_escape(self):
        checkout = self.root / "synthetic-checkout"
        checkout.mkdir()
        shared_runs = self.root / "synthetic-shared-runs"
        (shared_runs / "dev/astra").mkdir(parents=True)
        junction = checkout / "runs"
        if os.name == "nt":
            # Both junction and target stay within this checked temporary fixture.
            quote = lambda value: "'" + str(value).replace("'", "''") + "'"
            command = ("New-Item -ItemType Junction -Path " + quote(junction)
                       + " -Target " + quote(shared_runs) + " -ErrorAction Stop | Out-Null")
            try:
                created = subprocess.run(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", command],
                                         capture_output=True, text=True)
            except FileNotFoundError:
                self.skipTest("PowerShell unavailable for temporary Windows junction")
            if created.returncode:
                self.skipTest("Temporary Windows junction unavailable: " + created.stderr)
        else:
            junction.symlink_to(shared_runs, target_is_directory=True)
        self.assertEqual(junction.resolve(), shared_runs.resolve())
        with patch.object(train_feedback, "ROOT", checkout):
            self.assertEqual(train_feedback.resolve_receipt_directory(junction / "dev/astra"),
                             (shared_runs / "dev/astra").resolve())
            self.assertEqual(train_feedback.resolve_receipt_directory(junction / "dev/astra/new"),
                             (shared_runs / "dev/astra/new").resolve())
            with self.assertRaises(ValueError):
                train_feedback.resolve_receipt_directory(junction / "dev/astra/../outside")
            with self.assertRaises(ValueError):
                train_feedback.resolve_receipt_directory(self.root / "unrelated")


if __name__ == "__main__":
    unittest.main()
