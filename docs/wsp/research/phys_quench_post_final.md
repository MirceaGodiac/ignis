# Fabric flame, ignition and quench: the one revision after the final run (PROTOCOL 13.8)

Author: `claude-fable-5-2`, 15 November 2026. **Typed after the final run with every partition
seen; in-sample on all data; no held-out score exists.** Parent: `bass_front.wsp5` Q0
(`sha256:00a8bc35…5e72`, law of [phys_quench_w0.md](phys_quench_w0.md)). Read: the 23 fabric tracks
(4 train, 17 validation, 2 test) with their fan histories, and
`runs/dev/gen7/front_whatif/noop_check2/burns.md`. One candidate, run once
(`runs/dev/gen7/front_whatif/post_final_q1/`, logged with its expectation).

**Result in two lines.** One number changed: the oxygen floor of the fabric, 14.8 % to 14.1 %. It
does what was predicted on every partition and it is **not served**: the ignition score stays
above the constant on validation and test, and the track-ended score above D1 on train. The burns
bracket the boundary; they do not place it well enough to answer "will it light and stay lit".

## 1. Where Q0 was wrong

Flows are the scorer's reading of the fan display. "P(no spread)" is Q0's forecast at 20 s.

| Burn | O2, flow | What happened | Q0 P(no spread) | Term at fault |
|---|---|---|---:|---|
| T14 (test) | 16.9 %, 3.7 then 2.5 to 3.3 cm/s | lit, burned 95 s to the end | 0.80 | floor, and the low-flow term (limit 17.5 % plus 0.4 % ignition margin) |
| T11 (test) | 17.1 %, 4.6 | lit, burned to the end | 0.66 | floor |
| T16 (train) | 17.6 %, 4.7 then 3.0 | lit, burned to the end | 0.56 | floor |
| T18 (val) | 17.4 %, 5.6 | lit; went out later when the flow was cut | 0.46 | floor |
| T19, T22 (val, 1 cm) | 17.2 %, 17.1 %, 11 | lit | 0.45, 0.51 | floor (the width term adds 0.1 % here) |
| T21 (val, 1 cm) | 17.2 %, 11 then 47 | lit, spread, blown off at 12.5 s | 0.83 | floor (0.43); the rest is the blow-off, which Q0 had right |
| T5, T6 (train) | 18.8 %, cut to 2.4 | burned 80 to 90 s at 2.4 | 0.26, 0.38 | low-flow term: Q0's limit at 2.4 cm/s is 18.7 % |
| T28a (val, 1 cm) | 20.7 %, 4.6 | never lit | 0.01 | none: T27 and T29 lit at the same reading |

**The largest fault is the oxygen limit, not the flow limit and not the width loss.** Every fresh
sample at 16.9 % or more lit (seven between 16.9 and 17.6 %), where Q0 gave 43 to 76 % for "will
not light". At 16.4 % one lit (T9, 11 cm/s; its track is refused by the scorer) and one did not
(T10, 6.4 cm/s). So the 50 % point of ignition is about 16.4 % at 6 to 11 cm/s; Q0 had it at 16.9
to 17.15 %.

## 2. Is there a quench event to anchor the boundary?

Yes, four, and they agree with Q0's quench branch better than its ignition did.

| Event | Reading | Same reading, other outcome |
|---|---|---|
| T4 went out after 30 s at 2.4 cm/s | 18.7 % | T5 and T6 burned 80 to 90 s at 2.4 cm/s, 18.8 % |
| T18 went out as the flow fell 3.8 to 2.8 cm/s | 17.4 % | T16 burned 85 s at 3.0 cm/s, 17.6 %; T14 burned at 2.5 to 3.3, 16.9 % |
| T22 (1 cm) went out within 10 s at 6.0 cm/s | 17.1 % | no second 1 cm burn near it; T19 burned at 11 cm/s, 17.2 % |
| T21 (1 cm) blown off at 47 cm/s | 17.2 % | none |

On the low-flow branch each event has a survivor at the same reading. That fixes where the 50 %
line runs (18.7 to 18.8 % at 2.4 cm/s, where Q0 has 18.7 %; about 17.5 % at 2.9 cm/s, where Q0
has 18.1 %) and shows that these readings cannot separate the outcomes. Two reasons I can name:
the flow below the first calibration row is a straight line to an assumed zero at fan 38, which
differs between days by about 1 cm/s, worth about 1 % of oxygen at 3 cm/s; and near-limit flames
are sporadic. T14 against T10 shows the same: by the law's own margin T14 (lower flow) is the
harder of the two, and it burned.

## 3. The change

```text
# floor of the oxygen limit, SIBAL: 0.148 -> 0.141 (PMMA unchanged)
sus.x0[0:2] = [0.141, 0.136]
```

Physics: the floor is the one coefficient the dossier could only bracket (D4: 1700 K rule, plus
judgements for heavy cotton and glass, sd 1.0 %). The ignition burns put the limit at 6 to 11 cm/s
at 16.4 % minus the 0.4 % kernel margin, and the pair T18 / T16 puts the sustain limit at 2.9 cm/s
at 17.5 %; with Q0's flow terms both give a floor of 14.2 %. The 2.4 cm/s pair and the 1 cm quench
ask for 14.9 to 15.0 %. I type 14.1 %, inside one sd of Q0. The flow terms, the width term and
every width stay: nothing in the burns moves them without fitting a pair of burns apiece.

## 4. Expectation, written before the run, and the result

| Score | Q0 | Expected | Result | Baseline to pass |
|---|---:|---:|---:|---|
| Ignition, train (4) | 0.1469 | 0.10 | **0.1082** | constant 0.2500 ("every sample lights"), 0.1875 (train share): passes |
| Ignition, validation (15) | 0.1928 | 0.15 | **0.1612** | 0.1333, 0.1250: fails |
| Ignition, test (2) | 0.5347 | 0.26 | **0.2912** | 0.0000, 0.0625: fails |
| Track ended, train (3) | 0.0807 | 0.04 to 0.06 | **0.0403** | D1 0.0034: fails |
| Track ended, validation (12) | 0.1356 | 0.145 to 0.16 | **0.1445** | D1 0.1780: passes |
| Track ended, test (2) | 0.0291 | 0.02 to 0.03 | **0.0250** | D1 0.2066: passes |
| Tip CRPS, all partitions | unchanged | unchanged | unchanged | |

Every direction as predicted; validation ignition improved less than I estimated (T4 kept more
early quench than my hand count: 0.28 where I had 0.05). Per burn the movement is where it was aimed: T14 0.80
to 0.62, T11 0.66 to 0.44, T16 0.56 to 0.36, T18 0.46 to 0.25; and the price was as expected: T10
(never lit) 0.75 to 0.53, T23 (never lit) 0.42 to 0.21.

**Verdict by the rule: not served. The refusal of the quench question for fabric stands.**

## 5. What the law cannot do with these burns, plainly

No typed number would pass the rule honestly, and I did not look for one that passes dishonestly.

- The test partition is two burns that both lit, so the constant scores 0 there. A law passes
  only by giving 16.9 % at 3.7 cm/s no chance of failure, while a sample 0.5 % lower at higher
  flow (T10) failed. That needs a boundary 0.1 to 0.2 % wide drawn between two burns, which is
  what D1 did and what failed on validation.
- Two of the three samples that never lit (T23, T28a) are re-used samples at readings where
  fresh ones lit. No law of oxygen, flow and width can earn credit on them.
- On train, D1's track-ended score (0.0034) is in-sample and means "no flame goes out at 18.8 %,
  2.4 cm/s". A validation flame did, at that reading (T4). A law cannot say both.
- The ignition target counts a flame that spread and was then blown off within 20 s (T21) as
  "lit", while the forecast counts it as gone. Being right about the blow-off costs about 0.03
  on validation, as much as the whole gap to the constant (0.028).

What the 23 burns do support: the fabric lights at 16.9 % and above at 4 to 12 cm/s; the limit
of ignition is near 16.4 %; a 2 cm flame is at even odds at 18.7 % and 2.4 cm/s and at 17.5 %
and 2.9 cm/s; a 1 cm flame needs about twice the flow of a 2 cm one at 17 % (one burn); 47 cm/s
blows a 1 cm flame off at 17.2 % (one burn). The floor of 14.1 % is the value that agrees with
these; its sd of 1.0 % is not reduced.

## For the engineer

1. **The typed lines** are in section 3 (also `runs/dev/gen7/front_whatif/post_final_q1/edits.wsp`,
   child `sha256:01fc1530…7ff7`). If a release is assembled it carries the label at the top of
   this file and is not on the served list.
2. **Ignition target.** Forecast `p_quenched` at 20 s against "never spread" mixes two questions.
   Scoring `1 - p_lit_nodes` (already in `Boundary`) against "never spread", and leaving every
   flame that goes out to the track-ended score, would stop T21 and T28b from being read backwards.
   Not changed here; it is a change of protocol.
3. **Low-end flow.** Blocks whose first non-zero calibration row is at fan 55 (T1, T2, T4; T11,
   T14) or 46.5 (T5, T6, T16) get their flow below that row from a line to zero at fan 38; the two
   blocks that have a row at fan 42 read 3.8 cm/s (T10, T18) and 0.5 cm/s (T19, T22, T23) there.
   The quench pairs of section 2 differ by less than this. One low-end curve per restrictor, or a
   flow uncertainty in the input, is the missing piece.
4. **Fresh or re-used sample** is a missing input (track labels "moved igniter", "top side ign";
   the published test list marks T23 to T29 as re-used). The law is for fresh samples.
5. **Two refused tracks.** T9 (frame-width positions) is the only burn that lit at 16.4 %; T15 has
   no calibration row. Both would count for the ignition score if readable.
6. **Igniter on-time** follows the outcome (48 s for T23: the operator held the wire because
   nothing lit). It must not become a predictor beyond the short-time factor it is now.
