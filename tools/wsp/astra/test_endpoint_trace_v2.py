"""Boundary fixtures are synthetic acquisition records, not NASA observations."""
import base64
import gzip
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

import endpoint_trace_v2 as v2


HEADER = b"2021-01-01 001 12:00:00:000 ,Experiment: 5,Test Point: SYNTHETIC1\r\n"
COLUMNS = b"Time,PMT1,PMT2,PMT3,MFCA1StaticMixerFlow,MFCA2BypassFlow,MFCA3FuelFlow\n"


class TraceTests(unittest.TestCase):
    def read(self, raw, dates=(), test="SYNTHETIC1"):
        folder = tempfile.TemporaryDirectory()
        self.addCleanup(folder.cleanup)
        source = Path(folder.name)/"sensor.csv"
        ledger = Path(folder.name)/"rows.jsonl.gz"
        source.write_bytes(raw)
        summary, trace = v2.read_trace(source,hashlib.sha256(raw).hexdigest(),test,"sensor",ledger,dates)
        with gzip.open(ledger,"rt",encoding="utf-8") as stream:
            rows = [json.loads(line) for line in stream]
        return summary, trace, rows

    def test_lossless_all_rows_including_malformed_and_invalid_utf8(self):
        raw = HEADER+COLUMNS+b"12:00:01:000,1,2,3,4,5,6\r\n"+b"bad,row\n"+b"\xff\x00\n"+b"12:00:02:000,nan,2,3,4,5,6"
        summary, trace, rows = self.read(raw)
        self.assertEqual(b"".join(base64.b64decode(r["raw_base64"]) for r in rows),raw)
        self.assertEqual([r["line"] for r in rows],list(range(1,7)))
        self.assertEqual(summary["data_rows"],4)
        self.assertEqual(summary["valid_rows"],1)
        self.assertEqual(len(summary["parse_failures"]),3)
        self.assertEqual(len(trace["times"]),4)
        self.assertEqual(rows[-1]["byte_offset"]+rows[-1]["byte_length"],len(raw))

    def test_header_identity_refuses_before_body_parser(self):
        raw = HEADER+COLUMNS+b"DO NOT PARSE THIS BODY\n"
        summary, trace, rows = self.read(raw,test="UNKNOWN")
        self.assertEqual(summary["status"],"refused_identity")
        self.assertIsNone(trace)
        self.assertEqual(len(rows),1)

    def test_duplicate_backward_and_gap_preserved(self):
        raw = HEADER+COLUMNS+b"".join((t+",1,2,3,4,5,6\n").encode() for t in
              ("12:00:01:000","12:00:01:008","12:00:01:008","12:00:01:005","12:00:01:013","12:00:02:000"))
        summary,trace,_ = self.read(raw)
        self.assertEqual([a["kind"] for a in summary["clock_anomalies"]],["duplicate_clock","backward_clock"])
        self.assertEqual(len(trace["times"]),6)
        self.assertEqual(len(summary["missing_intervals"]),1)
        self.assertEqual(v2.segments(trace,.016),[(0,2),(2,3),(3,5),(5,6)])

    def test_documented_midnight_preserves_raw_clock(self):
        raw = HEADER.replace(b"12:00:00:000",b"23:59:59:000")+COLUMNS+b"23:59:59:992,1,2,3,4,5,6\n00:00:00:000,1,2,3,4,5,6\n"
        summary,trace,rows = self.read(raw,("2021-01-02",))
        self.assertEqual(trace["times"][1],86400)
        self.assertEqual(rows[-1]["source_clock_ms"],0)
        self.assertEqual(len(summary["calendar_rollovers"]),1)
        self.assertFalse(summary["clock_anomalies"])

    def test_first_sample_after_documented_midnight(self):
        raw=HEADER.replace(b"12:00:00:000",b"23:59:59:000")+COLUMNS+b"00:00:00:008,1,2,3,4,5,6\n"
        summary,trace,_=self.read(raw,("2021-01-02",))
        self.assertEqual(trace["times"][0],86400.008)
        self.assertEqual(len(summary["calendar_rollovers"]),1)

    def test_undocumented_midnight_stays_unresolved(self):
        raw=HEADER.replace(b"12:00:00:000",b"23:59:59:000")+COLUMNS+b"00:00:00:008,1,2,3,4,5,6\n"
        summary,trace,_=self.read(raw)
        self.assertEqual(trace["times"][0],.008)
        self.assertFalse(trace["valid"][0])
        self.assertEqual(summary["clock_anomalies"][0]["kind"],"uncorroborated_day_wrap")

    def test_clock_units_and_calendar_consistency(self):
        self.assertEqual(v2.clock_ms("01:02:03:004"),3723004)
        for token in ("01:02:03:4","24:00:00:000","12:60:00:000","12:00:00:1000"):
            with self.assertRaises(ValueError):v2.clock_ms(token)
        with self.assertRaises(ValueError):v2.metadata(HEADER.replace(b"001",b"002",1))

    def test_schema_errors_record_all_rows(self):
        raw=HEADER+b"Time,PMT1,PMT1\n12:00:00:001,1,2\n"
        summary,trace,rows=self.read(raw)
        self.assertIn("missing_time_or_duplicate_columns",summary["schema_errors"])
        self.assertEqual(summary["data_rows"],1)
        self.assertEqual(summary["valid_rows"],0)
        self.assertEqual(len(rows),3)

    def test_tp_alias_requires_unique_pretest_start(self):
        meta=v2.metadata(HEADER)
        tp=v2.stamp_text(b"Test Point: OLDNAME\nScript start: 2021-01-01 12:00:00\n")
        linked=v2.clock_link("SYNTHETIC1",meta,tp,{v2.start_key(meta):["SYNTHETIC1"]})
        self.assertEqual(linked["status"],"tp_alias_bound_by_unique_pretest_start")
        duplicate=v2.clock_link("SYNTHETIC1",meta,tp,{v2.start_key(meta):["SYNTHETIC1","OTHER"]})
        self.assertFalse(duplicate["aligned"])
        tp["stamps"]["Script start"]["clock_ms"]+=1000
        self.assertFalse(v2.clock_link("SYNTHETIC1",meta,tp,{v2.start_key(meta):["SYNTHETIC1"]})["aligned"])

    def test_eof_and_confirmation_are_different(self):
        intervals=v2.intervals([0,.1,.2],[False,True,True],[3,4,5],.5,0,"dark")
        self.assertFalse(intervals[0]["confirmed"])
        self.assertTrue(intervals[0]["confirmation_right_censored"])
        self.assertEqual(intervals[0]["transition_lower_clock_s"],0)
        confirmed=v2.intervals([0,.25,.5,.75],[False,True,True,True],[3,4,5,6],.5,0,"dark")[0]
        self.assertTrue(confirmed["confirmed"])
        self.assertTrue(confirmed["right_censored"])
        self.assertEqual(confirmed["confirmation_clock_s"],.75)

    def test_causal_ambiguity_not_forced_winner(self):
        dark={"transition_lower_clock_s":1,"transition_upper_clock_s":1.1}
        self.assertEqual(v2.diagnostic_order(dark,{"clock_s":1.15},True)["temporal_order"],"unresolved")
        self.assertEqual(v2.diagnostic_order(dark,{"clock_s":3},False)["temporal_order"],"unresolved")
        result=v2.diagnostic_order(dark,{"clock_s":3},True)
        self.assertEqual(result["temporal_order"],"dark_before_command")
        self.assertEqual(result["physical_cause"],"unresolved")

    def test_relighting_and_last_observed_loss_are_distinct(self):
        import numpy as np
        fd=v2.load_pinned_module("fixture_original_measurement","tools/wsp/ext/flame_design.py",v2.FD_SHA)
        times=np.arange(0,5.125,.125)
        values=np.zeros((len(times),6))
        values[:,0]=np.where(((times<1)|((times>=2)&(times<3))),1.,0.)
        trace={"times":times,"values":values,"valid":np.ones(len(times),bool),
               "lines":list(range(3,len(times)+3)),"columns":list(v2.SENSOR_REQUIRED)}
        result=v2.optical_diagnostics(trace,.125,0,fd)
        self.assertEqual(len(result["confirmed_later_relighting"]),1)
        self.assertLess(result["first_confirmed_dark_interval"]["first_clock_s"],2)
        self.assertGreater(result["last_observed_loss_candidate"]["first_clock_s"],2.9)
        self.assertFalse(result["qualified_physical_extinction"])
        self.assertEqual(result["record_end_censoring"]["state_beyond_record"],"unobserved")

    def test_unordered_samples_refuse_dark_estimator(self):
        import numpy as np
        trace={"times":np.array([1.,1.]),"values":np.zeros((2,6)),"valid":np.ones(2,bool),
               "lines":[3,4],"columns":list(v2.SENSOR_REQUIRED)}
        result=v2.optical_diagnostics(trace,.008,0,None)
        self.assertEqual(result["status"],"refused_global_dark_estimator_input")


if __name__=="__main__":
    unittest.main()
