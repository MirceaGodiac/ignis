"""Synthetic-only tests for condition supplement boundaries."""

import io
from hashlib import sha256
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

import condition_supplements as c


def trace_line(time, temperature="24.0"):
    return (f"2016-04-18 {time}\t2016-04-18 {time}\t0 0 0 0\t0 0 0 0 0 0 0 0\t14.7 {temperature}\n").encode()


def header(point="A109D04"):
    return [f"Test Point:\t{point}\n".encode(), b"AS Started:\t2016-04-18 15:48:52.276\n",
        b"AS Burn to Completion/Igniter On:\t2016-04-18 15:49:10.557\n",
        b"MDCA MP Time\tRadiometer Time\tAve PSIA  Ave DegC\n"]


class SupplementTests(unittest.TestCase):
    def test_parser_stops_before_post_igniter_numeric_payload(self):
        lines = header() + [trace_line("15:49:09.000"), trace_line("15:49:10.557", "FORBIDDEN")]
        def source():
            yield from lines
            raise AssertionError("post-igniter read")
        result = c.summary_prefix(source(), "A109D04")
        self.assertEqual("preigniter_observation", result["state"])
        self.assertEqual(24, result["initial"]["temperature_c"])
        self.assertEqual(1.557, result["initial"]["seconds_before_igniter"])

    def test_missing_or_mismatched_clock_never_releases_value(self):
        for lines in (header("OTHER"), header()[:1] + header()[3:]):
            result = c.summary_prefix(iter(lines + [trace_line("15:49:09.000")]), "A109D04")
            self.assertNotIn("initial", result)

    def test_reversed_timestamp_prefix_is_not_silently_sorted(self):
        lines = header() + [trace_line("15:49:09.000"), trace_line("15:49:08.000")]
        self.assertEqual("nonmonotonic_prefix", c.summary_prefix(iter(lines), "A109D04")["state"])

    def test_mixture_parser_returns_only_timestamp(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "synthetic.txt"
            path.write_text("Date/Time,O2,OUTCOME\n04/18/2016 17:30,FORBIDDEN,FORBIDDEN\n")
            with path.open("rb") as stream:
                self.assertEqual("2016-04-18T17:30:00", c.mixture_timestamp(stream))

    def test_ambiguous_mixture_clock_is_not_repaired(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "synthetic.txt"
            for stamp in ("9/10/2012 13:49:00 PM", "4/12/12 23:18"):
                path.write_text("Date/Time,O2\n" + stamp + ",FORBIDDEN\n")
                with path.open("rb") as stream:
                    self.assertIsNone(c.mixture_timestamp(stream))

    def test_header_order_maps_temperature_instead_of_relabelling_pressure(self):
        lines = header()
        lines[-1] = b"MDCA MP Time\tRadiometer Time\tAve DegC  Ave PSIA\n"
        lines.append(trace_line("15:49:09.000").replace(b"14.7 24.0", b"24.0 14.7"))
        result = c.summary_prefix(iter(lines), "A109D04")
        self.assertEqual("preigniter_observation", result["state"])
        self.assertEqual(24.0, result["initial"]["temperature_c"])
        self.assertEqual(14.7, result["initial"]["pressure_psia"])

    def test_ambiguous_or_unsupported_channel_units_are_rejected(self):
        for channels in ("Ave PSIA  Ave K", "Ave DegC  Ave DegC", "Ave PSIA  Ave DegC  Ave DegC",
                         "Ave DegC  Ave kPa", "Ave PSIA  Ave DegC Other"):
            lines = header()
            lines[-1] = ("MDCA MP Time\tRadiometer Time\t" + channels + "\n").encode()
            result = c.summary_prefix(iter(lines + [trace_line("15:49:09.000")]), "A109D04")
            self.assertNotIn("initial", result)

    def synthetic_source(self, root, raw):
        path = root / "corpus/incoming/synthetic.txt"
        path.parent.mkdir(parents=True)
        path.write_bytes(raw)
        meta = {"identity": "synthetic", "investigation": "PSI-999", "psi_file_id": "synthetic",
                "file_name": "synthetic.txt", "file_version": "1", "local_path": "corpus/incoming/synthetic.txt"}
        sources = c.Sources.__new__(c.Sources)
        sources.receipts = {"synthetic": {"state": "verified_download", "actual_bytes": len(raw),
                                           "sha256": sha256(raw).hexdigest()}}
        sources.verified, sources.coverage = {}, {}
        return sources, meta, path

    def test_replaced_source_cannot_reuse_an_old_cached_verification(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sources, meta, path = self.synthetic_source(root, b"synthetic old source")
            with patch.object(c, "BASE", root):
                sources.verified_path(meta)
                path.write_bytes(b"synthetic new source")
                with self.assertRaises(ValueError):
                    sources.verified_path(meta)
                with self.assertRaises(ValueError):
                    with sources.open_verified(meta):
                        self.fail("changed payload reached parser")

    def test_parser_gets_verified_snapshot_and_mutation_fails_before_release(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            original = b"synthetic original bytes"
            sources, meta, path = self.synthetic_source(root, original)
            with patch.object(c, "BASE", root):
                with self.assertRaises(ValueError):
                    with sources.open_verified(meta) as snapshot:
                        path.write_bytes(b"synthetic changed bytes!")
                        self.assertEqual(original, snapshot.read())
                with self.assertRaises(ValueError):
                    sources.verify_used_sources()

    def test_final_source_check_detects_change_after_successful_parse(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            original = b"synthetic source bytes"
            sources, meta, path = self.synthetic_source(root, original)
            with patch.object(c, "BASE", root):
                with sources.open_verified(meta) as snapshot:
                    self.assertEqual(original, snapshot.read())
                path.write_bytes(b"synthetic source drift")
                with self.assertRaises(ValueError):
                    sources.verify_used_sources()

    def test_fuel_folder_requires_explicit_reviewed_label(self):
        self.assertEqual(("decane",), c.parse_folder("Reduced Gravity/Quiescent 2016 - Decane")["species"])
        self.assertIsNone(c.parse_folder("Legacy/A109D04"))
        self.assertIsNone(c.parse_folder("Reduced Gravity/Unknown - flame_failure"))
        self.assertEqual(("octane",), c.parse_folder("Reduced Gravity/100% Octane")["species"])

    def test_no_material_inferred_from_geometry_alone(self):
        self.assertIsNone(c.material_token("2 cm sphere"))
        self.assertEqual("pmma", c.material_token("PMMA sphere 2 cm"))

    def test_missing_condition_is_unknown_not_zero(self):
        value = c.atlas_value(unit="K", reason="no established observation")
        self.assertEqual("unknown", value["status"])
        self.assertIsNone(value["value_si"])


if __name__ == "__main__":
    unittest.main()
