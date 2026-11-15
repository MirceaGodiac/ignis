"""Synthetic regression tests; none of these fixtures is a NASA observation."""
import json
import math
import unittest
from unittest.mock import Mock

from endpoint_train_audit import (at_risk_interval, cause_order, gated_record,
                                observed_sustained, raw_routes, stamp_account,
                                top_identity, train_scope)


class EndpointAuditTests(unittest.TestCase):
    def test_unknown_identity_never_decodes_payload(self):
        decode = Mock(side_effect=AssertionError("payload accessed"))
        line = '{"payload":{"run_id":"train","value":666},"run_id":"unknown"}'
        self.assertIsNone(gated_record(line, {"train": {}}, decode))
        decode.assert_not_called()
        with self.assertRaises(ValueError):
            raw_routes("unknown", {"train": {}})

    def test_nested_identity_does_not_grant_access(self):
        self.assertEqual(top_identity('{"notes":"\\\"run_id\\\": \\\"train\\\"",'
                                      '"nested":[{"run_id":"train"}],"run_id":"sealed"}'), "sealed")
        with self.assertRaises(ValueError):
            top_identity('{"run_id":"train","run_id":"sealed"}')

    def test_exact_split_has_all154_train_ids(self):
        scope = train_scope()
        self.assertEqual(len(scope), 154)
        self.assertEqual(sum(r["family"] == "acme_normal" for r in scope.values()), 81)

    def test_eof_is_not_half_second_confirmation(self):
        self.assertIsNone(observed_sustained([0, .1, .2], [False, True, True], 0, .5, .11))
        result = observed_sustained([0, .1, .2, .3, .4, .5, .6],
                                    [False, True, True, True, True, True, True], 0, .5, .11)
        self.assertEqual(result, {"lower_clock_s": 0.0, "upper_clock_s": .1,
                                  "confirmed_clock_s": .6})

    def test_gap_cannot_count_as_observed_hold(self):
        self.assertIsNone(observed_sustained([0, .1, 1, 1.1], [False, True, True, True], 0, .5, .11))
        result = observed_sustained([0, .1, 1, 1.2, 1.4, 1.6], [False, True, True, True, True, True], 0, .5, .21)
        self.assertIsNone(result["lower_clock_s"])
        self.assertEqual(result["upper_clock_s"], 1)

    def test_seconds_scaling_is_consistent(self):
        a = observed_sustained([0, 1, 2, 3], [False, True, True, True], 0, 2, 1)
        b = observed_sustained([0, .125, .25, .375], [False, True, True, True], 0, .25, .125)
        self.assertEqual(a["upper_clock_s"]*.125, b["upper_clock_s"])
        self.assertEqual(a["confirmed_clock_s"]*.125, b["confirmed_clock_s"])
        shifted = observed_sustained([86400, 86401, 86402, 86403],
                                     [False, True, True, True], 86400, 2, 1)
        self.assertEqual(shifted["upper_clock_s"]-86400, a["upper_clock_s"])

    def test_invalid_time_is_rejected(self):
        for times in ([0, 0], [1, 0], [0, math.nan]):
            with self.assertRaises(ValueError):
                observed_sustained(times, [True, True], 0, .5, .1)

    def test_event_interval_and_censoring_cutoff_semantics(self):
        def endpoint(kind, lo, hi):
            return {"censoring": kind, "last_confirmed_active_s": lo, "first_terminal_s": hi}
        self.assertEqual(at_risk_interval(endpoint("event", 3, 3), 3), "not_at_risk")
        self.assertEqual(at_risk_interval(endpoint("interval", 2.5, 3.5), 3), "unknown")
        self.assertEqual(at_risk_interval(endpoint("interval", 3, 4), 3), "at_risk")
        self.assertEqual(at_risk_interval(endpoint("right", 3, None), 3), "at_risk")
        self.assertEqual(at_risk_interval(endpoint("right", 2, None), 3), "unknown")
        self.assertEqual(at_risk_interval(endpoint("unknown", None, None), 3), "unknown")
        self.assertEqual(at_risk_interval(endpoint("event", 3.2, 3.2), 3), "at_risk")
        # A manufactured midpoint-0.5 would wrongly exclude this decimal timestamp.
        self.assertFalse(3.2-.5 >= 3)

    def test_invalid_interval_rejected(self):
        with self.assertRaises(ValueError):
            at_risk_interval({"censoring": "interval", "last_confirmed_active_s": 4,
                              "first_terminal_s": 3}, 1)

    def test_ambiguous_order_not_forced_operator(self):
        self.assertEqual(cause_order([2, 2.3], [2.2, 2.4]), "unresolved")
        self.assertEqual(cause_order([1, 1.1], [2, 2.1]), "extinction_precedes_flow")
        self.assertEqual(cause_order([2, 2.1], [1, 1.1]), "flow_precedes_darkening")
        self.assertEqual(cause_order(None, [1, 1.1]), "unresolved")

    def test_abort_preserves_extinction_stamp_without_claiming_cause(self):
        result = stamp_account({"Extinction detected": 4, "Test Point Failure": 4}, "EXTINCTION DETECTED")
        self.assertTrue(result["aborted"])
        self.assertEqual(result["extinction_detected_clock_s"], 4)
        self.assertEqual(result["cause"], "unresolved")


if __name__ == "__main__":
    unittest.main()
