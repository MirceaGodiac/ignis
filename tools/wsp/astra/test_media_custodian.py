"""Synthetic-only safety tests; never accesses NASA media or references."""
import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import media_custodian as mc


class CustodianTests(unittest.TestCase):
    def test_controller_pause_precedes_source_and_staging(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(mc, "PRIVATE", Path(tmp)), \
                patch.object(mc, "native_inventory") as inventory:
            continuation = Path(tmp) / "continuation"
            continuation.mkdir()
            (continuation / "checkpoint.json").write_text(json.dumps({"paused_by_controller":True}))
            with self.assertRaisesRegex(ValueError, "paused by controller"):
                mc.stage_continuation({}, 1)
            inventory.assert_not_called()
            self.assertEqual(sorted(p.name for p in continuation.iterdir()), ["checkpoint.json"])

    def test_continuation_budget_preserves_order_and_never_skips_by_quality(self):
        members = [{"native_frame_index": i, "uncompressed_bytes": 100 + i} for i in range(6)]
        allocation = 1024 * 1024 * 2 + 65536
        self.assertEqual([x["native_frame_index"] for x in mc.budget_indices(members, {0, 3}, allocation * 2)], [1, 2])
        self.assertEqual([x["native_frame_index"] for x in mc.budget_indices(members, {0, 1, 2, 3, 4}, allocation)], [5])
        with self.assertRaises(ValueError):mc.budget_indices(members, set(), allocation - 1)

    def test_path_escape_and_parent_denied(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for value in ("..", "../outside", root):
                with self.assertRaises(ValueError):
                    mc.contained(root, value)

    def test_train_gate_precedes_media_access(self):
        with patch.object(mc, "train_gate", side_effect=ValueError("not train")), \
                patch.object(mc.urllib.request, "urlopen") as opened:
            with self.assertRaises(ValueError):
                mc.fetch({}, execute=True)
            opened.assert_not_called()

    def test_split_hash_mutation_denied(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(mc, "ROOT", Path(tmp)):
            root = Path(tmp)
            split = root / "split.json"
            split.write_text("{}")
            with self.assertRaisesRegex(ValueError, "split changed"):
                mc.train_gate({"split_path": "split.json", "split_sha256": "0" * 64})

    def test_membership_and_catalogue_pins_fail_closed(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(mc, "ROOT", Path(tmp)):
            root = Path(tmp)
            source = {"psi_file_id": "synthetic", "investigation": "PSI-69", "test_id": "opaque",
                      "test_id_alternative": None, "latest_version": True, "url": "https://example.invalid/fixture"}
            split = {"split": {"run_to_group": {"fixture": "group"}, "train": ["group"],
                               "validation": [], "test": []}}
            atlas = {"atlas_id": "fixture", "project_partition": "train", "group_id": "group",
                     "aliases": [{"scheme": "cir_test_point", "name": "opaque"}]}
            for name, data in (("split.json", split), ("atlas.jsonl", atlas), ("files.jsonl", source)):
                (root / name).write_text(json.dumps(data) + "\n")
            route = {"run_id": "fixture", "group_id": "group", "partition": "train", "alias": "opaque",
                     "split_path": "split.json", "split_sha256": mc.sha256(root / "split.json"),
                     "atlas_path": "atlas.jsonl", "atlas_sha256": mc.sha256(root / "atlas.jsonl"),
                     "file_catalogue_path": "files.jsonl", "file_catalogue_sha256": mc.sha256(root / "files.jsonl"),
                     "sources": [source]}
            mc.train_gate(route)
            bad = copy.deepcopy(route)
            bad["sources"][0]["url"] = "https://example.invalid/different"
            with self.assertRaisesRegex(ValueError, "pinned catalogue"):
                mc.train_gate(bad)
            for partition in ("validation", "test"):
                changed = copy.deepcopy(split)
                changed["split"][partition] = ["group"]
                (root / "split.json").write_text(json.dumps(changed))
                route["split_sha256"] = mc.sha256(root / "split.json")
                with self.assertRaisesRegex(ValueError, "TRAIN membership"):
                    mc.train_gate(route)

    def manifest(self):
        return {"schema": "astra.blind-media/1", "packet_id": "1" * 32,
                "full_source_ref": "media_" + "2" * 32,
                "method_version": "manual-silhouette-pixel-v1", "method_sha256": "0" * 64,
                "instrument": "backlit_camera", "observable": "liquid_silhouette_horizontal_vertical_extents",
                "distance_unit": "pixel", "time_axis": "native_frame_index",
                "calibration_status": "dimensional_and_clock_unqualified", "partial": True,
                "coverage": "systematic native-index sample; full source retained privately",
                "pixel_transform": "none; metadata removed; lossless PNG",
                "frames": [{"path": "frame_000000.png", "sha256": "0" * 64,
                            "frame_index": 0, "width": 10, "height": 10, "bit_depth": 8}]}

    def test_blind_manifest_denies_identity_in_any_layer(self):
        mc.validate_blind_manifest(self.manifest())
        for key in ("partition", "run_id", "expected_value", "fuel", "model"):
            row = self.manifest()
            row[key] = "sensitive"
            with self.assertRaises(ValueError):
                mc.validate_blind_manifest(row)
            row = self.manifest()
            row["frames"][0][key] = "sensitive"
            with self.assertRaises(ValueError):
                mc.validate_blind_manifest(row)
        for key in ("instrument", "coverage", "pixel_transform"):
            row = self.manifest()
            row[key] = "condition-bearing free text"
            with self.assertRaises(ValueError):
                mc.validate_blind_manifest(row)
        row = self.manifest()
        row["frames"][0]["path"] = "../source_identity.png"
        with self.assertRaises(ValueError):
            mc.validate_blind_manifest(row)
        row = self.manifest()
        row["frames"].append(copy.deepcopy(row["frames"][0]))
        with self.assertRaises(ValueError):
            mc.validate_blind_manifest(row)

    def test_retention_verified_before_source_deleted(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root / "source.bin"
            evidence = root / "evidence.bin"
            source.write_bytes(b"synthetic_fixture_original")
            evidence.write_bytes(b"synthetic_fixture_retained")
            digest = mc.sha256(source)
            items = [{"path": evidence.name, "sha256": mc.sha256(evidence)}]
            receipts = [{"state": "sealed", "source_sha256": digest, "measurer_id": m} for m in ("a", "b")]
            with self.assertRaises(ValueError):
                mc.verify_retention_then_delete(root, source.name, "0" * 64, items, receipts, execute=True)
            self.assertTrue(source.exists())
            damaged = copy.deepcopy(items)
            damaged[0]["sha256"] = "0" * 64
            with self.assertRaises(ValueError):
                mc.verify_retention_then_delete(root, source.name, digest, damaged, receipts, execute=True)
            self.assertTrue(source.exists())
            with self.assertRaises(ValueError):
                mc.verify_retention_then_delete(root, source.name, digest, items, receipts[:1], execute=True)
            self.assertTrue(source.exists())
            result = mc.verify_retention_then_delete(root, source.name, digest, items, receipts, execute=True)
            self.assertTrue(result["deleted"])
            self.assertFalse(source.exists())
            self.assertTrue(evidence.exists())


if __name__ == "__main__":
    unittest.main()
