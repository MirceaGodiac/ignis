"""Synthetic boundary tests; never open NASA condition/outcome payloads."""

from copy import deepcopy
import json
import unittest

import author_gate as gate
import screen_conditions as screen
import test_author_gate as fixtures


class ConditionsCustodyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        fixtures.ConditionsProjectionTests.setUpClass()

    def row(self):
        return fixtures.ConditionsProjectionTests().synthetic_row()

    def value(self, value, unit=None, **changes):
        result = deepcopy(self.row()["atmosphere"]["pressure"])
        result.update(value_si=value, unit_si=unit)
        result.update(changes)
        return result

    def project(self, row, schema_pin=None):
        fields, omitted = screen.screen_row(row)
        raw = screen.encode(row)
        schema_pin = schema_pin or fixtures.ConditionsProjectionTests.schema_pin
        prose_pin = fixtures.ConditionsProjectionTests.prose_pin
        receipt = gate.ConditionsApproval(gate.digest(raw), tuple(row[k] for k in
            ("atlas_id", "investigation_id", "test_id", "level")), fields, row["group_id"],
            "1" * 64, approved_species=tuple(sorted(screen.SPECIES)))
        approvals = {"synthetic": receipt}
        pin = gate.Pin("author-conditions-approvals/1", gate.digest(
            gate.conditions_approval_bytes(approvals, schema_pin, prose_pin)))
        adapter = gate.AtlasConditions(schema_pin, prose_pin, approvals, pin)
        result = adapter.read("synthetic", lambda _: raw)
        return result, omitted

    def test_raw_notes_and_descriptions_do_not_leak(self):
        row = self.row()
        row["fuel"]["name"] = self.value("kapton", raw_value="DO NOT EXPOSE synthetic post-test note")
        result, _ = self.project(row)
        self.assertEqual("allowed", result.decision.state)
        public = json.dumps(result.projection)
        self.assertIn("kapton", public)
        self.assertNotIn("DO NOT EXPOSE", public)
        self.assertNotIn("raw_value", public)
        self.assertNotIn('"notes"', public)

    def test_ignition_conditioned_and_conflicting_values_are_omitted(self):
        for flags in (["conditioned_on_ignition"], ["conflict"]):
            row = self.row()
            row["atmosphere"]["pressure"]["flags"] = flags
            fields, omitted = screen.screen_row(row)
            self.assertNotIn("atmosphere.pressure", fields)
            self.assertIn("atmosphere.pressure", omitted)

    def test_nominal_multiple_stays_multiple_and_matrix_stays_matrix(self):
        row = self.row()
        row["level"] = "matrix_row"
        row["atmosphere"]["pressure"].update(status="multiple", value_si=None,
            set={"kind": "range", "values": [50000, 101325]}, reason="synthetic planned range")
        result, _ = self.project(row)
        self.assertEqual("allowed", result.decision.state)
        value = result.projection["conditions"]["atmosphere.pressure"]
        self.assertEqual("multiple", value["status"])
        self.assertIsNone(value["value_si"])
        self.assertEqual({"kind": "range", "values": [50000, 101325]}, value["set"])
        self.assertEqual("matrix_row", result.projection["level"])

    def test_material_name_is_not_a_generic_free_text_escape(self):
        row = self.row()
        row["fuel"]["name"] = self.value("kapton_failed_after_ignition")
        fields, omitted = screen.screen_row(row)
        self.assertNotIn("fuel.name", fields)
        self.assertIn("fuel.name", omitted)

    def test_timing_requires_explicit_initial_provenance(self):
        row = self.row()
        row["atmosphere"]["pressure"]["basis"] = "measured"
        self.assertNotIn("atmosphere.pressure", screen.screen_row(row)[0])
        row["investigation_id"] = "PSI-10"
        row["atmosphere"]["pressure"].update(basis="unstated")
        row["atmosphere"]["pressure"]["source"]["column"] = "pinit (atm)"
        self.assertIn("atmosphere.pressure", screen.screen_row(row)[0])

    def test_nominal_tag_cannot_override_known_during_burn_source(self):
        row = self.row()
        row["investigation_id"] = "PSI-21"
        row["injection"]["x_fuel_injected"] = self.value(0.5, "1")
        row["injection"]["x_fuel_injected"]["source"]["column"] = "Fuel conc. Range"
        self.assertNotIn("injection.x_fuel_injected", screen.screen_row(row)[0])

    def test_geometry_label_survives_unreleased_analysis_measurement(self):
        row = self.row()
        row["geometry"]["sizes"] = [{"dimension": "initial_droplet_diameter",
            "value": self.value(0.001, "m", basis="measured", flags=["from_analysis_sheet"])}]
        fields, _ = screen.screen_row(row)
        self.assertIn("geometry.sizes.0.dimension", fields)
        self.assertNotIn("geometry.sizes.0.value", fields)

    def test_parent_conflict_does_not_disappear_in_component(self):
        row = self.row()
        row["fuel"]["name"] = self.value("pmma", flags=["conflict"])
        row["fuel"]["components"] = [{"species": "pmma", "role": "fuel",
            "fraction": self.value(1, "1", basis="derived", rule="PURE")}]
        fields, _ = screen.screen_row(row)
        self.assertFalse(any(path.startswith("fuel.") for path in fields))

    def test_schema_drift_refuses_release(self):
        result, _ = self.project(self.row(), gate.Pin(gate.ATLAS_VERSION, "0" * 64))
        self.assertEqual("unavailable", result.decision.state)
        self.assertIsNone(result.projection)


if __name__ == "__main__":
    unittest.main()
