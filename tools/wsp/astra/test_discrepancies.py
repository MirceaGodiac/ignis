"""Synthetic invariant checks; no NASA payload is read by this suite."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import discrepancies as d


class BoundaryTests(unittest.TestCase):
    def test_train_identity_gate_precedes_payload_decoder(self):
        with patch.object(d.audit,"gated_record",side_effect=AssertionError("payload decoded")):
            with self.assertRaisesRegex(ValueError,"TRAIN"):
                d.gate_artifact(b'{"run_id":"held","secret":123}',{"train":{}},"held")
        with self.assertRaises(ValueError):
            d.gate_artifact(b'{"run_id":"train","run_id":"held"}',{"train":{}},"train")

    def test_protected_routes_and_traversal_fail_closed(self):
        for name in ("../a", "runs/holdout/a", "runs/holdout_ext/a", "runs/validation/a", "runs/artifacts/models/a", "runs/tests_outcomes.jsonl"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                d.checked_path(name)

    def test_bytes_are_verified_on_exact_consumed_snapshot(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(d,"ROOT",Path(directory)):
            p = Path(directory)/"source.csv"; p.write_bytes(b"old")
            pin = d.sha(b"old")
            self.assertEqual(d.pinned_bytes("source.csv",pin),b"old")
            p.write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError,"changed"):
                d.pinned_bytes("source.csv",pin)

    def test_no_unknown_uncertainty_normalization_or_self_confirmation(self):
        left=d.measure("1","s",{"row":1},si=1)
        right=d.measure("2","s",{"row":2},si=2)
        r=d.comparison("synthetic",159,"duration",left,right,"fixture",delta=1)
        self.assertIsNone(r["normalized_difference"])
        self.assertFalse(r["cause_established"])
        self.assertEqual([c["id"] for c in r["checks"]],list(d.CAUSE_CHECKS))
        r["method"]["code_sha256"]="same"
        receipt={"reader_id":"other","implementation_sha256":"same","protocol_sha256":"p",
                 "source_sha256s":[],"discrepancy_id":r["id"],"verdict":"table_value_wrong","independence_attestation":True}
        with self.assertRaisesRegex(ValueError,"same method"):
            d.independent_confirmation(r,receipt)
        with self.assertRaises(ValueError):
            d.comparison("synthetic",159,"duration",left,right,"fixture",status="confirmed_independent")

    def test_incommensurate_units_cannot_be_compared(self):
        with self.assertRaisesRegex(ValueError,"incommensurate"):
            d.comparison("fixture",159,"flow",d.measure(1,"V",{}),d.measure(1,"m3/s",{}),"fixture")

    def test_lossless_numeric_parser_refuses_duplicates_nonfinite_and_schema_drift(self):
        header=b"2020-01-01 001 00:00:00:000,Test Point: T1\n"
        cols=b"Time,MFCA1Setpoint(V),MFCA2Setpoint(V),MFCA3Setpoint(V)\n"
        row=b"00:00:00:001,1,2,3\n"
        meta,columns,rows=d.exact_rows(header+cols+row,"T1","commands")
        self.assertEqual(rows[1][0],3)
        self.assertEqual(rows[1][1][1],"1")
        for body in (row+row,row.replace(b",2,",b",nan,"),row.replace(b",2,",b",,")):
            with self.assertRaises(ValueError):d.exact_rows(header+cols+body,"T1","commands")
        with self.assertRaises(ValueError):d.exact_rows(header+cols.replace(b"MFCA1Setpoint(V)",b"unknown")+row,"T1","commands")
        with self.assertRaisesRegex(ValueError,"identity"):
            d.exact_rows(header+cols+b"secret malformed row", "other","commands")

    def test_exact_time_join_never_interpolates(self):
        header=b"2020-01-01 001 00:00:00:000,Test Point: T1\n"
        sensor=header+b"Time,MFCA1StaticMixerFlow,MFCA2BypassFlow,MFCA3FuelFlow\n00:00:00:001,2,2,3\n00:00:00:003,8,8,8\n"
        command=header+b"Time,MFCA1Setpoint(V),MFCA2Setpoint(V),MFCA3Setpoint(V)\n00:00:00:001,1,2,3\n00:00:00:002,9,9,9\n"
        entry={"run_id":"fixture","test":"T1","sources":{"data":{"path":"sensor","sha256":d.sha(sensor)},"setpoints":{"path":"command","sha256":d.sha(command)}}}
        with patch.object(d,"payload",side_effect=lambda _,name: {"sensor":sensor,"command":command}[name]):
            records,state,count=d.command_comparisons(entry,{})
        self.assertEqual(state,"compared");self.assertEqual(count,3)
        self.assertEqual(len(records),1)
        self.assertEqual(records[0]["difference_si"],[1.0])
        self.assertEqual(records[0]["sensor_value"]["locator"]["rows"],[3])

    def test_immutable_output_cannot_replace_existing_receipt(self):
        with tempfile.TemporaryDirectory() as directory:
            p=Path(directory)/"receipt"
            d.immutable(p,b"first");d.immutable(p,b"first")
            with self.assertRaisesRegex(ValueError,"immutable"):
                d.immutable(p,b"second")

    def test_script_duplicate_missing_and_nonfinite_not_zero(self):
        self.assertNotIn("absent",d.script_fields(b"testpointName: X\n"))
        with self.assertRaises(ValueError):d.script_fields(b"duration: 1\nduration: 2\n")
        for value in (None,True,"nan","infinity",""):
            with self.assertRaises(ValueError):d.number(value)


if __name__ == "__main__":
    unittest.main()
