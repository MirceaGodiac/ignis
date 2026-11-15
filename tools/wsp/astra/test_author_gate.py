"""Synthetic metadata only; no NASA files, outcomes or network calls."""

from copy import deepcopy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import runpy

from author_gate import (ATLAS_VERSION, SPLIT_VERSION, AtlasConditions, AuthorGate, ConditionsApproval,
                         Pin, conditions_approval_bytes, digest, outcome_approval_bytes)


def encode(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def fixture(payloads=None):
    schema = {"origin": "synthetic_fixture", "version": "fixture-schema/1"}
    split = {
        "origin": "synthetic_fixture", "version": "fixture-split/1", "frozen_by": "Fable",
        "schema_sha256": digest(encode(schema)),
        "runs": {
            "synthetic-train": {"group": "a", "partition": "train"},
            "synthetic-validation": {"group": "b", "partition": "validation"},
            "synthetic-test": {"group": "c", "partition": "test"},
        },
    }
    mapping = {
        "origin": "synthetic_fixture", "version": "fixture-mapping/1",
        "schema_sha256": digest(encode(schema)), "split_sha256": digest(encode(split)),
        "payloads": payloads or {},
    }
    docs = {"schema": schema, "split": split, "mapping": mapping}
    return docs, {name: Pin(doc["version"], digest(encode(doc))) for name, doc in docs.items()}


def row(kind="measurement", runs=("synthetic-train",), pages=(), **changes):
    result = {
        "kind": kind, "runs": list(runs), "pages": list(pages),
        "sha256": digest(b"synthetic payload"), "screened": True,
        "dependencies_complete": True,
    }
    result.update(changes)
    return result


def gate_for(payloads):
    docs, pins = fixture(payloads)
    return AuthorGate.synthetic_fixture({name: encode(doc) for name, doc in docs.items()}, pins)


class AuthorBoundaryTests(unittest.TestCase):
    def assert_no_read(self, gate, key="target", state="sealed"):
        with tempfile.TemporaryDirectory() as directory:
            protected = Path(directory) / "synthetic-protected.bin"
            protected.write_bytes(b"synthetic fixture, never open on this route")

            def read_trap(_key):
                raise AssertionError(f"protected payload reader invoked: {protected}")

            result = gate.read_author(key, read_trap)
            self.assertEqual(state, result.decision.state)
            self.assertIsNone(result.payload)

    def test_live_adapter_is_unavailable_without_any_file_open(self):
        with patch.object(Path, "read_bytes", side_effect=AssertionError("file open")):
            self.assert_no_read(AuthorGate.live(), state="unavailable")

    def test_pending_or_invalid_metadata_cannot_read_payload(self):
        docs, pins = fixture({"target": row()})
        metadata = {name: encode(doc) for name, doc in docs.items()}
        for name in metadata:
            with self.subTest(missing=name):
                self.assert_no_read(AuthorGate.synthetic_fixture(
                    {key: raw for key, raw in metadata.items() if key != name}, pins,
                ), state="unavailable")
            with self.subTest(tampered=name):
                changed = dict(metadata)
                changed[name] += b" "
                self.assert_no_read(AuthorGate.synthetic_fixture(changed, pins), state="unavailable")
            with self.subTest(version=name):
                changed = dict(pins)
                changed[name] = Pin("wrong-version", pins[name].sha256)
                self.assert_no_read(AuthorGate.synthetic_fixture(metadata, changed), state="unavailable")

    def test_invalid_trust_bindings_refuse_even_when_rehashed(self):
        docs, _ = fixture({"target": row()})
        changes = [
            ("schema", "origin", "NASA"),
            ("split", "frozen_by", "author"),
            ("split", "schema_sha256", "0" * 64),
            ("mapping", "schema_sha256", "0" * 64),
            ("mapping", "split_sha256", "0" * 64),
            ("mapping", "extra_grant", "train"),
        ]
        for name, key, value in changes:
            with self.subTest(name=name, key=key):
                changed = deepcopy(docs)
                changed[name][key] = value
                raw = {part: encode(doc) for part, doc in changed.items()}
                pins = {part: Pin(doc["version"], digest(raw[part])) for part, doc in changed.items()}
                self.assert_no_read(AuthorGate.synthetic_fixture(raw, pins), state="unavailable")

    def test_screened_conditions_and_public_methods_can_be_read(self):
        for item in (row("conditions", ("synthetic-test",)), row("public_method", ())):
            gate = gate_for({"target": item})
            self.assertEqual("allowed", gate.decide("target").state)
            with tempfile.TemporaryDirectory() as directory:
                source = Path(directory) / "synthetic.bin"
                source.write_bytes(b"synthetic payload")
                self.assertEqual(b"synthetic payload", gate.read_author("target", lambda _: source.read_bytes()).payload)

    def test_every_outcome_kind_requires_only_train_dependencies(self):
        for kind in ("measurement", "test_label", "outcome_page", "interpretation"):
            self.assertEqual("allowed", gate_for({"target": row(kind)}).decide("target").state)
            for runs in ((), ("synthetic-validation",), ("synthetic-test",),
                         ("missing",), ("synthetic-train", "synthetic-test")):
                with self.subTest(kind=kind, runs=runs):
                    self.assert_no_read(gate_for({"target": row(kind, runs)}))

    def test_unknown_incomplete_unscreened_and_cyclic_dependencies_stay_sealed(self):
        cases = [
            {"target": row(pages=("missing-page",))},
            {"target": row(pages=("target",))},
            {"target": row(pages=("child",)), "child": row(pages=("target",))},
            {"target": row(dependencies_complete=False)},
            {"target": row(screened=False)},
            {"target": row("public_method", (), screened=False)},
            {"target": row("conditions", ("missing-run",))},
            {"target": row("interpretation", (), ("child",)), "child": row("public_method", ())},
        ]
        for case in cases:
            with self.subTest(case=case):
                self.assert_no_read(gate_for(case))

    def test_nested_page_dependencies_inherit_train_requirement(self):
        for kind in ("conditions", "public_method", "outcome_page"):
            payloads = {"target": row("interpretation", (), ("page",)),
                        "page": row(kind, ("synthetic-test",))}
            self.assert_no_read(gate_for(payloads))
        gate = gate_for({"target": row("interpretation", (), ("page",)), "page": row("outcome_page")})
        self.assertEqual("allowed", gate.decide("target").state)

    def test_payload_hash_mismatch_never_releases_bytes(self):
        gate = gate_for({"target": row()})
        calls = []

        def reader(key):
            calls.append(key)
            return b"changed synthetic payload"

        result = gate.read_author("target", reader)
        self.assertEqual(["target"], calls)
        self.assertEqual("payload_identity_mismatch", result.decision.reason)
        self.assertIsNone(result.payload)

    def test_blind_manifest_has_no_host_identity_or_partition(self):
        gate = gate_for({"source-test-model-hypothesis": row(runs=("synthetic-test",))})
        method = Pin("host-only-method-version", digest(b"synthetic method"))
        for operation in ("measure", "annotate"):
            task = gate.blind_task(["source-test-model-hypothesis"], method, operation)
            public = json.dumps(task.manifest)
            for forbidden in ("source-test", "synthetic-test", "host-only", "partition", "hypothesis", "expected", "model"):
                self.assertNotIn(forbidden, public)
            self.assertEqual({"origin", "task_id", "operation", "method_sha256", "inputs", "output"}, set(task.manifest))
            path = task.manifest["inputs"][0]["path"]
            self.assertRegex(path, r"^tasks/[0-9a-f]{32}/inputs/00000000\.bin$")
            self.assertEqual("source-test-model-hypothesis", task.host_routes["inputs"][path])
        with self.assertRaises(ValueError):
            AuthorGate.live().blind_task(["anything"], method, "measure")


class ConditionsProjectionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        root = Path(__file__).resolve().parents[3]
        source = root / "tools/wsp/atlas/atlas_schema.py"
        prose = root / "docs/wsp/atlas/SCHEMA.md"
        cls.schema_pin = Pin(ATLAS_VERSION, digest(source.read_bytes()))
        cls.prose_pin = Pin(ATLAS_VERSION, digest(prose.read_bytes()))
        cls.schema_module = runpy.run_path(str(source))

    def synthetic_row(self):
        def example(schema):
            if "const" in schema:
                return schema["const"]
            if "enum" in schema:
                return schema["enum"][0]
            kind = schema.get("type")
            if isinstance(kind, list):
                if "null" in kind:
                    return None
                kind = kind[0]
            if kind == "object":
                if set(schema.get("required", [])) == set(self.schema_module["VALUE_KEYS"]):
                    return {"status": "unknown", "value_si": None, "unit_si": None,
                            "raw_value": None, "raw_unit": None, "basis": None,
                            "scope": "row", "source": None, "rule": None, "flags": [],
                            "reason": "synthetic missing condition", "set": None, "alternatives": []}
                return {key: example(schema["properties"][key]) for key in schema.get("required", [])}
            if kind == "array":
                return [example(schema["items"]) for _ in range(schema.get("minItems", 0))]
            if kind == "string":
                return "synthetic"
            if kind in ("number", "integer"):
                return 1
            raise AssertionError(f"unsupported fixture shape: {schema}")

        row = example(self.schema_module["row_schema"]())
        row.update(atlas_id="PSI-999:synthetic", investigation_id="PSI-999", test_id="synthetic")
        source = {key: None for key in self.schema_module["SOURCE_KEYS"]}
        source["file"] = "synthetic-conditions.csv"
        row["atmosphere"]["pressure"].update(
            status="resolved", value_si=101325, unit_si="Pa", basis="nominal",
            source=source, reason=None, raw_value="DO NOT EXPOSE synthetic outcome note",
        )
        row["notes"] = ["DO NOT EXPOSE synthetic note"]
        row["hardware"]["raw"] = "DO NOT EXPOSE synthetic hardware note"
        return row

    def setup_adapter(self, row, fields=("atmosphere.pressure",), schema_pin=None, mapping_pin=None,
                      approved_species=(), group_id=None, catalogue_sha256=None):
        schema_pin = schema_pin or self.schema_pin
        raw = encode(row)
        receipt = ConditionsApproval(digest(raw), tuple(row[key] for key in
                                     ("atlas_id", "investigation_id", "test_id", "level")), fields,
                                     group_id, catalogue_sha256, approved_species)
        approvals = {"opaque-row": receipt}
        mapping_pin = mapping_pin or Pin("author-conditions-approvals/1", digest(
            conditions_approval_bytes(approvals, schema_pin, self.prose_pin)))
        adapter = AtlasConditions(schema_pin, self.prose_pin, approvals, mapping_pin)
        return adapter, raw, approvals

    def trap(self, _key):
        raise AssertionError("unscreened row reader called")

    def test_conditions_projection_keeps_only_structured_allowlist(self):
        adapter, raw, _ = self.setup_adapter(self.synthetic_row())
        result = adapter.read("opaque-row", lambda _: raw)
        self.assertEqual("allowed", result.decision.state)
        self.assertEqual(101325, result.projection["conditions"]["atmosphere.pressure"]["value_si"])
        self.assertTrue(result.projection["partial"])
        public = json.dumps(result.projection)
        for forbidden in ("DO NOT EXPOSE", "raw_value", "notes", "partition", "outcome", '"source":'):
            self.assertNotIn(forbidden, public)

    def test_free_text_or_outcome_projection_requests_refuse_before_read(self):
        for field in ("notes", "fuel.name", "extra_conditions.description", "labels.flame_type",
                      "atmosphere.pressure.raw_value", "outcome_fields_available", "project_partition"):
            with self.subTest(field=field):
                adapter, _, _ = self.setup_adapter(self.synthetic_row(), (field,))
                self.assertEqual("unavailable", adapter.read("opaque-row", self.trap).decision.state)

    def test_closed_vocabulary_and_status_validation(self):
        changes = [
            lambda row: row["gravity"].update(status="invented-status"),
            lambda row: row["atmosphere"]["pressure"].update(unit_si="invented-unit"),
            lambda row: row["atmosphere"]["pressure"].update(status="unknown"),
            lambda row: row["atmosphere"]["pressure"].update(flags=["conditioned_on_ignition"]),
            lambda row: row["atmosphere"]["pressure"].update(flags=["invented-flag"]),
            lambda row: row.update(unrecognised_outcomes="synthetic"),
            lambda row: row["atmosphere"]["pressure"].update(value_si=float("inf")),
        ]
        for change in changes:
            row = self.synthetic_row()
            change(row)
            adapter, raw, _ = self.setup_adapter(row)
            result = adapter.read("opaque-row", lambda _: raw)
            self.assertEqual("sealed", result.decision.state)
            self.assertIsNone(result.projection)

    def test_changed_schema_and_mapping_refuse_before_read(self):
        for pin in (Pin(ATLAS_VERSION, "0" * 64), Pin("ignis.atlas.test/999", self.schema_pin.sha256)):
            adapter, _, _ = self.setup_adapter(self.synthetic_row(), schema_pin=pin)
            self.assertEqual("unavailable", adapter.read("opaque-row", self.trap).decision.state)
        adapter, _, _ = self.setup_adapter(self.synthetic_row(), mapping_pin=Pin("author-conditions-approvals/1", "0" * 64))
        self.assertEqual("unavailable", adapter.read("opaque-row", self.trap).decision.state)

    def test_unapproved_rows_and_changed_bytes_never_project(self):
        adapter, raw, approvals = self.setup_adapter(self.synthetic_row())
        approvals["unapproved"] = approvals["opaque-row"]
        self.assertEqual("sealed", adapter.read("unapproved", self.trap).decision.state)
        changed = adapter.read("opaque-row", lambda _: raw + b" ")
        self.assertEqual("row_identity_mismatch", changed.decision.reason)
        self.assertIsNone(changed.projection)

    def test_reviewed_materials_geometry_and_missing_fraction_stay_separate(self):
        record = self.synthetic_row()
        safe = deepcopy(record["atmosphere"]["pressure"])
        safe.update(value_si="methane+nitrogen", unit_si=None)
        record["fuel"]["name"] = safe
        missing_fraction = deepcopy(record["atmosphere"]["x_o2"])
        record["fuel"]["components"] = [
            {"species": "methane", "role": "fuel", "fraction": missing_fraction},
            {"species": "nitrogen", "role": "diluent", "fraction": missing_fraction},
        ]
        size = deepcopy(record["atmosphere"]["pressure"])
        size.update(value_si=0.01, unit_si="m")
        record["geometry"]["sizes"] = [{"dimension": "burner_diameter", "value": size}]
        record["atmosphere"]["diluents"]["items"] = [{"species": "nitrogen", "fraction": missing_fraction}]
        fields = ("fuel.name", "fuel.components.0.species", "fuel.components.0.role",
                  "fuel.components.1.species", "geometry.sizes.0.dimension", "geometry.sizes.0.value",
                  "atmosphere.diluents.items.0.species")
        adapter, raw, _ = self.setup_adapter(record, fields, approved_species=("methane", "nitrogen"),
                                            group_id=record["group_id"], catalogue_sha256="a" * 64)
        result = adapter.read("opaque-row", lambda _: raw)
        self.assertEqual("allowed", result.decision.state)
        self.assertEqual("methane+nitrogen", result.projection["conditions"]["fuel.name"]["value_si"])
        self.assertEqual("burner_diameter", result.projection["conditions"]["geometry.sizes.0.dimension"]["value_si"])
        self.assertEqual(0.01, result.projection["conditions"]["geometry.sizes.0.value"]["value_si"])
        self.assertEqual(record["group_id"], result.projection["group_id"])
        self.assertEqual("a" * 64, result.projection["catalogue_sha256"])
        self.assertEqual(set(fields), set(result.projection["conditions"]))
        self.assertNotIn("DO NOT EXPOSE", json.dumps(result.projection))
        # Approving a safe species does not automatically approve its unavailable fraction.
        adapter, raw, _ = self.setup_adapter(record, fields + ("fuel.components.0.fraction",),
                                            approved_species=("methane", "nitrogen"))
        self.assertEqual("sealed", adapter.read("opaque-row", lambda _: raw).decision.state)

    def test_unreviewed_species_and_index_aliases_fail_closed(self):
        record = self.synthetic_row()
        record["fuel"]["components"] = [{"species": "unreviewed_material", "role": "fuel",
                                            "fraction": deepcopy(record["atmosphere"]["x_o2"])}]
        adapter, raw, _ = self.setup_adapter(record, ("fuel.components.0.species",), approved_species=("methane",))
        self.assertEqual("sealed", adapter.read("opaque-row", lambda _: raw).decision.state)
        for field in ("fuel.components.-1.species", "fuel.components.00.species",
                      "fuel.components.*.species", "geometry.sizes.0.value.raw_value"):
            adapter, _, _ = self.setup_adapter(record, (field,), approved_species=("methane",))
            self.assertEqual("unavailable", adapter.read("opaque-row", self.trap).decision.state)
        adapter, raw, _ = self.setup_adapter(record, ("fuel.components.1.species",), approved_species=("methane",))
        self.assertEqual("sealed", adapter.read("opaque-row", lambda _: raw).decision.state)

    def test_nominal_matrix_sets_preserve_kind_values_and_status(self):
        for kind, values in (("list", [50000, 75000, 100000]), ("range", [50000, 100000])):
            record = self.synthetic_row()
            record["level"] = "matrix_row"
            record["atmosphere"]["pressure"].update(status="multiple", value_si=None,
                reason="DO NOT EXPOSE planned matrix text", set={"kind": kind, "values": values})
            adapter, raw, _ = self.setup_adapter(record)
            result = adapter.read("opaque-row", lambda _: raw)
            self.assertEqual("allowed", result.decision.state)
            self.assertEqual("multiple", result.projection["conditions"]["atmosphere.pressure"]["status"])
            self.assertIsNone(result.projection["conditions"]["atmosphere.pressure"]["value_si"])
            self.assertEqual({"kind": kind, "values": values}, result.projection["conditions"]["atmosphere.pressure"]["set"])
            self.assertNotIn("DO NOT EXPOSE", json.dumps(result.projection))

    def test_measured_flagged_or_malformed_sets_cannot_project(self):
        for change in ({"basis": "measured"}, {"flags": ["conditioned_on_ignition"]},
                       {"set": {"kind": "range", "values": [2, 1]}},
                       {"set": {"kind": "list", "values": ["source words"]}},
                       {"set": {"kind": "list", "values": []}}):
            record = self.synthetic_row()
            record["atmosphere"]["pressure"].update(status="multiple", value_si=None,
                reason="synthetic nominal planned conditions", set={"kind": "list", "values": [50000, 100000]})
            record["atmosphere"]["pressure"].update(change)
            adapter, raw, _ = self.setup_adapter(record)
            result = adapter.read("opaque-row", lambda _: raw)
            self.assertEqual("sealed", result.decision.state)
            self.assertIsNone(result.projection)

    def test_component_fraction_cannot_lose_parent_flags(self):
        record = self.synthetic_row()
        name = deepcopy(record["atmosphere"]["pressure"])
        name.update(value_si="methane", unit_si=None, flags=["fill_series_artifact"])
        record["fuel"]["name"] = name
        fraction = deepcopy(record["atmosphere"]["pressure"])
        fraction.update(value_si=1, unit_si="1")
        record["fuel"]["components"] = [{"species": "methane", "role": "fuel", "fraction": fraction}]
        adapter, raw, _ = self.setup_adapter(record, ("fuel.components.0.fraction",))
        result = adapter.read("opaque-row", lambda _: raw)
        self.assertEqual("sealed", result.decision.state)
        self.assertIsNone(result.projection)


class CommittedSplitAdapterTests(unittest.TestCase):
    setUpClass = classmethod(ConditionsProjectionTests.setUpClass.__func__)
    synthetic_row = ConditionsProjectionTests.synthetic_row

    def build(self, payloads=None, status="split", change_split=None, changed_bytes=False,
              committed=True, recorder=None, unverified=False):
        root = Path(__file__).resolve().parents[3]
        catalogue_pin = Pin(ATLAS_VERSION, digest(b"synthetic catalogue snapshot"))
        raws, approvals = {}, {}
        for name, group in (("train", "a"), ("validation", "b"), ("test", "c"), ("new", "new")):
            record = self.synthetic_row()
            record.update(atlas_id=f"PSI-999:{name}", test_id=name, group_id=f"PSI-999:day:{group}")
            raw = encode(record)
            raws[name] = raw
            approvals[name] = ConditionsApproval(digest(raw),
                tuple(record[key] for key in ("atlas_id", "investigation_id", "test_id", "level")),
                (), record["group_id"], catalogue_pin.sha256)
        approval_pin = Pin("author-conditions-approvals/1", digest(
            conditions_approval_bytes(approvals, self.schema_pin, self.prose_pin)))
        conditions = AtlasConditions(self.schema_pin, self.prose_pin, approvals, approval_pin)
        if not unverified:
            for name in raws:
                self.assertEqual("allowed", conditions.read(name, lambda key: raws[key]).decision.state)
        groups = {"train": ["PSI-999:day:a"], "validation": ["PSI-999:day:b"], "test": ["PSI-999:day:c"]}
        if status != "split":
            groups = {"held": ["PSI-999:day:a"]}
        split = {
            "schema": SPLIT_VERSION, "investigation_id": "PSI-999", "acronym": "synthetic",
            "rule": "synthetic fixed groups; not a NASA freeze", "seed": 1,
            "group_key": "group_id", "catalogue_level": {"test": len(groups)},
            "strata": {"all": {"status": status, "groups": groups,
                                "rows_in_catalogue": {part: len(members) for part, members in groups.items()}}},
            "frozen_on": "2026-11-14", "use": {p: "synthetic" for p in ("train", "validation", "test", "held")},
            "note": "synthetic new groups held",
        }
        if change_split:
            change_split(split)
        raw_split = encode(split)
        split_pins = {"PSI-999": Pin(SPLIT_VERSION, digest(raw_split))}
        code_pins = {"catalogue": Pin(ATLAS_VERSION, digest(b"synthetic catalogue code")),
                     "freeze": Pin(SPLIT_VERSION, digest(b"synthetic freeze code"))}
        commit = "a" * 40
        payloads = payloads or {"target": row(runs=("PSI-999:train",))}
        mapping = outcome_approval_bytes(conditions, payloads, split_pins, catalogue_pin, code_pins, commit)
        mapping_pin = Pin("author-outcome-approvals/1", digest(mapping))
        committed_files = {
            "tools/wsp/atlas/atlas_schema.py": (root / "tools/wsp/atlas/atlas_schema.py").read_bytes(),
            "docs/wsp/atlas/SCHEMA.md": (root / "docs/wsp/atlas/SCHEMA.md").read_bytes(),
            "tools/wsp/atlas/normalize_tests.py": b"synthetic catalogue code",
            "tools/wsp/atlas/freeze_groups.py": b"synthetic freeze code",
            "docs/wsp/atlas/splits/PSI-999.json": raw_split if committed else b"uncommitted",
        }
        metadata = {"PSI-999": raw_split + (b" " if changed_bytes else b"")}
        with patch("author_gate._committed_metadata", side_effect=lambda _commit, path: committed_files[path]):
            return AuthorGate.from_atlas(conditions, metadata, split_pins, catalogue_pin, code_pins,
                                         mapping, mapping_pin, commit, recorder)

    def trap(self, _key):
        raise AssertionError("protected payload reader called")

    def test_exact_verified_atlas_identity_can_route_train(self):
        events = []
        gate = self.build(recorder=events.append)
        self.assertEqual("allowed", gate.decide("target").state)
        result = gate.read_author("target", lambda _: b"synthetic payload")
        self.assertEqual(b"synthetic payload", result.payload)
        self.assertEqual("allowed", events[0]["state"])
        self.assertIn("mapping_sha256", events[0])

    def test_held_and_new_groups_are_never_author_outcomes(self):
        for status in ("held_too_few_groups", "held_until_test_list"):
            gate = self.build(status=status, recorder=lambda _: None)
            self.assertEqual("sealed", gate.read_author("target", self.trap).decision.state)
        gate = self.build({"target": row(runs=("PSI-999:new",))}, recorder=lambda _: None)
        self.assertEqual("sealed", gate.read_author("target", self.trap).decision.state)

    def test_changed_or_uncommitted_frozen_bytes_fail_closed(self):
        for options in ({"changed_bytes": True}, {"committed": False}):
            gate = self.build(**options)
            self.assertEqual("unavailable", gate.read_author("target", self.trap).decision.state)

    def test_aliases_and_unverified_catalogue_ids_are_not_guessed(self):
        for identity in ("train", "PSI-999:TRAIN", " PSI-999:train", "PSI-999:alias"):
            gate = self.build({"target": row(runs=(identity,))}, recorder=lambda _: None)
            self.assertEqual("sealed", gate.read_author("target", self.trap).decision.state)
        gate = self.build(unverified=True, recorder=lambda _: None)
        self.assertEqual("sealed", gate.read_author("target", self.trap).decision.state)

    def test_mixed_pages_unresolved_pages_and_cycles_stay_sealed(self):
        cases = [
            {"target": row("interpretation", (), ("page",)),
             "page": row("outcome_page", ("PSI-999:train", "PSI-999:test"))},
            {"target": row("interpretation", (), ("missing",))},
            {"target": row("interpretation", ("PSI-999:train",), ("page",)),
             "page": row("outcome_page", ("PSI-999:train",), ("target",))},
        ]
        for payloads in cases:
            gate = self.build(payloads, recorder=lambda _: None)
            self.assertEqual("sealed", gate.read_author("target", self.trap).decision.state)

    def test_overlapping_strata_and_unknown_status_refuse_authority(self):
        def overlap(split):
            split["strata"]["other"] = deepcopy(split["strata"]["all"])
            split["catalogue_level"]["test"] *= 2

        def unknown(split):
            split["strata"]["all"]["status"] = "approved"

        for mutation in (overlap, unknown):
            gate = self.build(change_split=mutation)
            self.assertEqual("unavailable", gate.read_author("target", self.trap).decision.state)

    def test_missing_or_failed_reveal_record_prevents_open(self):
        self.assertEqual("unavailable", self.build().read_author("target", self.trap).decision.state)

        def failed(_event):
            raise OSError("synthetic ledger unavailable")

        result = self.build(recorder=failed).read_author("target", self.trap)
        self.assertEqual("reveal_record_failed", result.decision.reason)


if __name__ == "__main__":
    unittest.main()
