"""Offline synthetic tests of acquisition boundaries; no NASA payload is read."""
import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import fetch_public as fetch
from astra_download_recover_alias import eligible_aliases


class Response:
    def __init__(self, content, status=200, headers=None):
        self.content, self.status_code = content, status
        self.headers = headers or {"Content-Length": str(len(content)), "ETag": '"synthetic"'}

    def __enter__(self):
        return self

    def __exit__(self, *_):
        pass

    def raise_for_status(self):
        pass

    def iter_content(self, **_):
        yield self.content


class Client:
    def __init__(self, response):
        self.response = response
        self.headers = None

    def get(self, _, **kwargs):
        self.headers = kwargs["headers"]
        return self.response


class IntegrityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.row = dict(identity="synthetic", investigation="PSI-69", size_bytes=6,
                        file_name="synthetic.txt", stable_url="https://example.invalid/synthetic",
                        local_path="corpus/incoming/PSI-69/synthetic.txt")
        self.client = fetch.Fetcher(self.root, 1000000, 0)
        self.dest = self.root / self.row["local_path"]

    def run_fetch(self, response):
        remote = Client(response)
        with patch.object(fetch, "session", return_value=remote), patch.object(fetch.time, "sleep"):
            result = self.client.fetch(self.row)
        return result, remote

    def test_integrity_and_repeated_readback(self):
        result, _ = self.run_fetch(Response(b"abcdef"))
        self.assertEqual(result, "verified_download")
        receipt = self.client.latest["synthetic"]
        self.assertEqual(receipt["sha256"], hashlib.sha256(b"abcdef").hexdigest())
        self.assertEqual(self.client.fetch(self.row), "verified_existing")
        self.dest.write_bytes(b"abcdeg")
        self.assertEqual(self.client.fetch(self.row), "failed")

    def test_resume_requires_range_and_validator(self):
        self.dest.parent.mkdir(parents=True)
        self.dest.with_suffix(".txt.part").write_bytes(b"abc")
        self.client.record(self.row, state="downloading", response_headers={"ETag": '"synthetic"'})
        result, remote = self.run_fetch(Response(b"def", 206, {"Content-Length": "3", "Content-Range": "bytes 3-5/6"}))
        self.assertEqual(result, "verified_download")
        self.assertEqual(remote.headers, {"Range": "bytes=3-", "If-Range": '"synthetic"'})
        self.assertEqual(self.dest.read_bytes(), b"abcdef")

    def test_server_ignores_range_restart_is_safe(self):
        self.dest.parent.mkdir(parents=True)
        self.dest.with_suffix(".txt.part").write_bytes(b"abc")
        self.client.record(self.row, state="downloading", response_headers={"ETag": '"synthetic"'})
        result, _ = self.run_fetch(Response(b"abcdef"))
        self.assertEqual(result, "verified_download")
        self.assertEqual(self.dest.read_bytes(), b"abcdef")

    def test_size_mismatch_never_promoted(self):
        result, _ = self.run_fetch(Response(b"short"))
        self.assertEqual(result, "failed")
        self.assertFalse(self.dest.exists())

    def test_bad_content_range_never_promoted(self):
        self.dest.parent.mkdir(parents=True)
        self.dest.with_suffix(".txt.part").write_bytes(b"abc")
        self.client.record(self.row, state="downloading", response_headers={"ETag": '"synthetic"'})
        result, _ = self.run_fetch(Response(b"def", 206, {"Content-Length": "3", "Content-Range": "bytes 2-4/6", "ETag": '"synthetic"'}))
        self.assertEqual(result, "failed")
        self.assertFalse(self.dest.exists())

    def test_peak_budget_blocks_before_network(self):
        self.client.peak = 1
        self.assertEqual(self.client.fetch(self.row), "failed")
        self.assertEqual(self.client.latest["synthetic"]["state"], "budget_blocked")

    def test_oversized_failed_partial_is_logged_and_restarted(self):
        self.dest.parent.mkdir(parents=True)
        self.dest.with_suffix(".txt.part").write_bytes(b"synthetic oversized failed transfer")
        result, _ = self.run_fetch(Response(b"abcdef"))
        self.assertEqual(result, "verified_download")
        self.assertIn('"disposition": "deleted_invalid_partial"', self.client.log.read_text())
        self.assertEqual(self.dest.read_bytes(), b"abcdef")

    def test_full_partial_without_verification_receipt_restarts(self):
        self.dest.parent.mkdir(parents=True)
        self.dest.with_suffix(".txt.part").write_bytes(b"broken")
        self.client.record(self.row, state="downloading", response_headers={"ETag": '"synthetic"'})
        result, remote = self.run_fetch(Response(b"abcdef"))
        self.assertEqual(result, "verified_download")
        self.assertEqual(remote.headers, {})
        self.assertEqual(self.dest.read_bytes(), b"abcdef")

    def test_inconsistent_range_end_is_rejected(self):
        self.dest.parent.mkdir(parents=True)
        self.dest.with_suffix(".txt.part").write_bytes(b"abc")
        self.client.record(self.row, state="downloading", response_headers={"ETag": '"synthetic"'})
        result, _ = self.run_fetch(Response(b"def", 206, {"Content-Length": "3", "Content-Range": "bytes 3-3/6", "ETag": '"synthetic"'}))
        self.assertEqual(result, "failed")
        self.assertFalse(self.dest.exists())

    def test_connection_failure_discards_failed_session(self):
        class BrokenClient:
            closed = False

            def get(self, *_args, **_kwargs):
                raise fetch.requests.ConnectionError("synthetic connection interruption")

            def close(self):
                self.closed = True

        broken, healthy = BrokenClient(), Client(Response(b"abcdef"))
        fetch.LOCAL.session = broken
        try:
            with patch.object(fetch, "session", side_effect=[broken, healthy]), patch.object(fetch.time, "sleep"):
                self.assertEqual(self.client.fetch(self.row), "verified_download")
            self.assertTrue(broken.closed)
        finally:
            if hasattr(fetch.LOCAL, "session"):
                del fetch.LOCAL.session


class AliasBoundaries(unittest.TestCase):
    def test_only_exact_metadata_key_version_host_and_length_are_eligible(self):
        row = dict(identity="synthetic", investigation="PSI-69", category="Reports",
                   file_name="part.pdf", key="psi-data/PSI-69/Reports/full/part.pdf",
                   file_version=2, size_bytes=6, stable_url="https://example.invalid/listing")
        target = fetch.stable_url(dict(row, file_name="full/part.pdf"))
        probe = dict(identity="synthetic", variant="metadata_key_filename", requested_url=target,
                     http_status=200, final_host="synthetic.amazonaws.com", response_headers={"Content-Length": "6"})
        self.assertEqual(eligible_aliases({"synthetic": row}, [probe])[0]["identity"], "synthetic")
        for changed in (dict(probe, requested_url=target.replace("version=2", "version=1")),
                        dict(probe, requested_url=target.replace("psi.nasa.gov", "example.invalid"))):
            with self.assertRaises(ValueError):
                eligible_aliases({"synthetic": row}, [changed])
        for changed in (dict(probe, final_host="amazonaws.com.example.invalid"),
                        dict(probe, response_headers={"Content-Length": "7"})):
            self.assertEqual(eligible_aliases({"synthetic": row}, [changed]), [])


if __name__ == "__main__":
    unittest.main()
