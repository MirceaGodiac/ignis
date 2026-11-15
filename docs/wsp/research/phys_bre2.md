# ACME BRE (PSI-20): the second law (`d2-fable`): put the held class into what is scored

Author: `claude-fable-5-2`, 15 November 2026. **Data-informed** (train and validation days as study
data, PROTOCOL 13.4). Read for it: the bre section of `docs/wsp/feedback/gen7_failure_analysis.md`,
`docs/wsp/feedback/bre_train_anatomy.md`, the coordinator's message. No test day, no git command, no
program run. Parent: the W0 release of `docs/wsp/research/phys_bre_w0.md`
(`sha256:aa0eef5b...2b4d`), **not** D1. Same slug `bre`, same declaration, no new feature.

## 1. What went wrong

The W0 law has three outcomes (no flame, held, goes out at `t_ext`). The scored quantity was taken
from `flame_end`, a clock with one shape and no mass at infinity, so the held class was not in the
score at all; W0 section 7.5 said so and named `lasts` as the target to score for the yes/no
question, and `lasts` was never scored. Where flames are held (65 to 85 kPa: 20 % out by 60 s
observed, 72 % forecast) the clock loses to the planned-stop baseline; at 1 atm it ties or beats it.
My D1 step tried to repair a missing class with a pressure exponent, on the clock. That was the
wrong tool.

## 2. The scored quantity

**Rule (taken at scoring time, no weight):** for a horizon `h` seconds after onset and a cutoff `c`
(seconds after onset, at least 1 s),

```text
P(extinguished by h | burning at c) = 1 - lasts(stated_time = h) / lasts(stated_time = c)      h < stop
                                    = 1 - lasts(stated_time = stop) / lasts(stated_time = c)   h >= stop
P(operator termination by h)        = lasts(stated_time = stop) / lasts(stated_time = c)       h >= stop, else 0
P(active at h)                      = lasts(stated_time = h) / lasts(stated_time = c)          h < stop, else 0
```

`stop` is the planned stop. Every other input of the prefix is unchanged between the calls; only
`stated_time` moves. All horizons 6 to 210 s are inside its declared domain (1 to 3000 s).

**Why this is the physically right one.** `lasts(t)` is the law's survival function itself:
`logit = 4.0 (H(ln t - base) - b)`, the probability that the uncertain size ratio leaves the flame
alive at `t`. Its limit for long times is the probability that the flame is held, so the mass at
infinity is in it by construction.

**Is it monotone in `stated_time` as typed?** Yes. `stated_time` enters only through rows 0 to 5 of
`lasts.logit.W1` with weight -1.0 and positive output weights (0.71 to 1.18); `skip[6] = 0`; rows 6
and 7 do not read it. The logit is therefore non-increasing in `stated_time`, flat beyond the last
knot (`ln t - base > 3.46`), so `1 - lasts` is a proper distribution function with an atom at
infinity and the ratios above lie in [0, 1].

**The alternative, a held fraction times the clock** (`(1 - lasts(3000 s)) x F_clock(h)`), is wrong
for these numbers: the `flame_end` median was deliberately typed long near the boundary to imitate
the mixture with one shape (W0 section 7.2), so the product counts the held class twice. At 34 %
oxygen, 0.57 atm, 3 g/m2/s it gives 9 % out by 60 s where the law says 26 %.

**Where each fails.**
- `1 - lasts`: it is too diffuse far from the boundary, because one constant (4.0) carries both the
  boundary uncertainty and the clock uncertainty. In air at 1 atm it will say about 0.7 to 0.75 out
  by 60 s where the clock said 0.89 and the days show 1.00. That is a known cost of this rule, about
  45 train cases. It also inherits the small-flame corner of W0 section 7.5.
- The clock alone: right in air, wrong wherever a flame can be held.
- The product: better than `lasts` in air (0.83), worse near the boundary.

`flame_end` stays in the release for the time of flames that did go out and is no longer the source
of the scored probability.

## 3. The boundary: one damped step, on W0's numbers

From my W0 table, `1 - lasts(60 s)` is 0.19 to 0.35 at 34 % oxygen (observed 0.26 in 0.30 to 0.36)
and 0.35 to 0.52 at 26.5 % (observed 0.49 in 0.25 to 0.30): once the held class is scored, the
oxygen strata show no fault that asks for a number. These are estimates from the registered table,
not a run.

The reduced-pressure stratum still does: at 26.5 %, 0.69 atm the rule gives about 0.45 against
0.20 observed (49 cases, train and validation together), while flames in the same oxygen at 1 atm
mostly go out. The only number through which pressure moves the held boundary is the exponent `a`
of the critical radiative flux (`q_x ~ p^a`, W0 section 1.3). The data ask for more than any
radiation law allows (about 2); the optically thin limit is 1.0. One damped step, half-way to that
ceiling:

**`a`: 0.5 -> 0.8** (the blue, soot-free flames radiate nearer the thin limit than I judged; the
thinning of the outer hot layer does not relieve the sheet).

| line | old (W0) | new | derivation |
|---|---|---|---|
| `lasts.logit.skip[3]` | -1.08 | -1.72 | `-4.0 x 0.541 a` |
| `lasts.logit.W1[0:6, 3]` | 0.46 | 0.13 | `1 - 1.082 a` |
| `lasts.logit.W1[6:8, 3]` | 0.5 | 0.8 | small-flame margin |
| `flame_end.extinction.median.skip[3]` | 0.28 | -0.15 | `base - 0.65 b` |
| `flame_end.extinction.median.W1[0:5, 3]` | -0.27 | -0.43 | `-0.541 a` |
| `flame_end.extinction.median.W1[5:7, 3]` | 0.5 | 0.8 | small-flame margin |
| `height_at_end.median.skip[3]` | -0.27 | -0.43 | `-0.541 a` |

These are the values of D1; what differs is that they now act through the scored `lasts`. At 1 atm
the step is nil by construction (`z_p = 0`); it cannot help or hurt a 1 atm day. Its size on the
logit of `lasts` is `0.64 |ln(p / 1 atm)|` plus the shift of `base`: about +0.4 at 0.69 atm and +0.6
at 0.57 atm.

**Expected** (estimates, not runs): 65 to 85 kPa, out by 60 s: 0.72 (clock) to about 0.45 by the rule
and about 0.35 with the step, against 0.20 observed; oxygen 0.30 to 0.36: 0.51 to about 0.25 to
0.30, against 0.26; air: 0.89 to about 0.72, against 1.00 (worse). The remaining gap at 65 to 85 kPa
is not closed and should not be forced through `a`.

## For the engineer

- **Slug and parent.** Revision `d2-fable` of slug `bre`. Parent: the W0 release
  `sha256:aa0eef5b...2b4d`. Declaration unchanged; no new feature.
- **Scoring-time rule.** Section 2, exactly: for each case call target `lasts` once per horizon with
  `stated_time = h`, once with `stated_time = c` (the cutoff, not below 1 s) and once with
  `stated_time = stop`; extinction by `h` is `1 - lasts(min(h, stop)) / lasts(c)`; operator
  termination is `lasts(stop) / lasts(c)` from the stop on. If a ratio exceeds 1 by rounding, clip to
  1. `flame_end` is not used for the score.
- **Check before scoring:** that `lasts` is non-increasing over the horizons for every case (it
  must be, section 2); a violation means the edits were not applied as written.

### Edits

Parent: W0 (`configs/models/law/bre/<w0>/edits.wsp`), all its lines unchanged except:

```text
# bre d2-fable: pressure exponent of the critical flux a = 0.5 -> 0.8; scored through lasts
lasts.logit.skip[3] = -1.72
lasts.logit.W1[0:6, 3] = 0.13
lasts.logit.W1[6:8, 3] = 0.8
flame_end.extinction.median.skip[3] = -0.15
flame_end.extinction.median.W1[0:5, 3] = -0.43
flame_end.extinction.median.W1[5:7, 3] = 0.8
height_at_end.median.skip[3] = -0.43
```
