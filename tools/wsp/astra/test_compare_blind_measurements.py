import copy
import tempfile
import unittest
from pathlib import Path
import compare_blind_measurements as cm


def observer(context, delta=0):
    return {"method_sha256": "fixture", "identity": {"task_context": context},
            "unit": "pixel", "time_axis": "native_frame_index",
            "rows": {0: {"source_sha256": "synthetic_fixture",
                         "observables": {n: {"value": 10+delta, "bounds": [8+delta, 12+delta], "status": "observed"} for n in cm.OBSERVABLES}}}}


class CompareTests(unittest.TestCase):
    def test_controller_binding_preserves_prose_and_rejects_exposure(self):
        doc = {"packet_id":"synthetic_fixture", "observer":{"task_context":"observer prose"},
               "exposure_declaration":{"task_context":"different prose", "prior_familiarity":False}}
        binding = {"packet_id":doc["packet_id"], "observer_declaration":copy.deepcopy(doc["observer"]),
                   "exposure_declaration":copy.deepcopy(doc["exposure_declaration"]),
                   "task_context":"fresh-context-a", "fork_turns":"none"}
        original = copy.deepcopy(doc)
        with self.assertRaises(ValueError): cm.observer_identity(doc)
        resolved = cm.observer_identity(doc, binding)
        self.assertEqual(resolved["task_context"], "fresh-context-a")
        self.assertEqual(resolved["original_task_context"], "observer prose")
        self.assertEqual(doc, original)
        for field in ("packet_id", "fork_turns"):
            bad=copy.deepcopy(binding);bad[field]="wrong"
            with self.assertRaises(ValueError):cm.observer_identity(doc,bad)
        doc["exposure_declaration"]["prior_familiarity"]=True
        binding["exposure_declaration"]=copy.deepcopy(doc["exposure_declaration"])
        with self.assertRaises(ValueError):cm.observer_identity(doc,binding)

    def test_conservative_difference_not_measurement_average(self):
        result = cm.compare(observer("a"), observer("b", 1))
        row = result["records"][0]
        self.assertEqual(row["difference_b_minus_a"], 1)
        self.assertEqual(row["difference_bounds"], [-3, 5])
        self.assertTrue(row["bounds_overlap"])
        self.assertEqual(row["a"]["value"], 10)
        self.assertEqual(row["b"]["value"], 11)

    def test_no_zero_for_missing_and_no_silent_exclusion(self):
        a, b = observer("a"), observer("b")
        a["rows"][0]["observables"]["left"] = {"value": None, "bounds": None, "status": "occluded"}
        s = cm.compare(a,b)["summary"]["left"]
        self.assertEqual(s["available_pairs"], 0)
        self.assertEqual(s["one_sided_missing"], 1)
        self.assertIsNone(s["mean_absolute_difference_px"])

    def test_incompatible_identity_units_and_frame_rejected(self):
        for change in ("method_sha256", "unit", "time_axis"):
            b = observer("b")
            b[change] = "different"
            with self.assertRaises(ValueError): cm.compare(observer("a"), b)
        b = observer("b")
        b["rows"][0]["source_sha256"] = "different"
        with self.assertRaises(ValueError): cm.compare(observer("a"), b)
        b = observer("b")
        b["rows"][1] = b["rows"].pop(0)
        with self.assertRaises(ValueError): cm.compare(observer("a"), b)
        with self.assertRaises(ValueError): cm.compare(observer("a"), observer("a"))

    def test_invalid_operator_bounds_rejected(self):
        for value,bounds in ((float("nan"),[0,1]),(2,[1,0]),(3,[0,2]),(None,[0,2])):
            with self.assertRaises(ValueError):cm.checked_value(value,bounds)

    def test_seal_detects_mutation_without_rewriting(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);p=root/'measurement.json';p.write_bytes(b'synthetic_fixture')
            seal={"packets":[{"path":str(root),"files":[{"path":p.name,"bytes":p.stat().st_size,"sha256":cm.sha256(p)}]}]}
            cm.verify_seal(seal)
            p.write_bytes(b'changed')
            with self.assertRaises(ValueError):cm.verify_seal(seal)
            self.assertEqual(p.read_bytes(),b'changed')


if __name__ == '__main__':unittest.main()
