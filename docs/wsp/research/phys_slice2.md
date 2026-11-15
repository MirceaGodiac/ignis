# `slice2`: lifting as a window, and a label that is not always written

Author: `claude-fable-5-2`, 15 November 2026. **Data-informed** (train and validation days, read
through `docs/wsp/feedback/gen7_failure_analysis.md`; no test day, no per-case file). Parent: the
W0 release of `slice` (`phys_slice_w0.md`). Blow-off and flame length are not touched: same
features, same numbers.

## Physics

W0 gave for "lifted" the chance that the condition is past the lifting limit (`Phi` > 1). Two
things are wrong with that as a forecast of the log.

1. **A flame past the blow-off limit is not there to be lifted.** Lifted is the window
   `1 < Phi < 1.3`. A line exists in the log only because the flame was still burning when it was
   set, so the forecast must also be conditional on not being past blow-off:

   ```text
   P(lifted | flame present) = (P(Phi > 1) - P(Phi > 1.3)) / (1 - P(Phi > 1.3))
   ```

   With the W0 width (0.5 in ln, logistic slope 3.4) this is 0.09 at `Phi` = 0.6, 0.30 at
   `Phi` = 1, and tends to 0.59 for large `Phi` (if the flame is there although the law says it
   should be gone, the law's limit is wrong and the flame is probably between the two limits).
2. **The crew wrote "lifted" only when they noted it.** A base a millimetre or two off the rim on
   a 0.4 mm tube, seen in a glovebox camera while stepping a flow, is easy to miss. I type the
   chance that a lifted flame is noted as **0.5**. The window takes the mean forecast down by
   about half; the residual over-forecast is then about 3.5; I go half-way in ln.

Expected: mean forecast on train from 0.28 to about 0.07 (observed 0.03); on the 0.4 mm tube from
0.64 to about 0.2 (observed 0.06). The remaining excess there is the jet term for 40 % methane
(`phys_slice_d1.md`), whose six lines can be applied to this slug as well.

The position of the line in the sequence and the step from the previous line are not added: the
window form already uses the fact that the flame survived, and the step size enters blow-off only,
which must not change.

## For the engineer

New slug `slice2`: the eight features of `slice`, unchanged. Targets:

| `name` | kind | What it is |
|---|---|---|
| `blow_off` | binary | as `slice` |
| `past_lift` | binary | the condition is past the lifting limit (the network W0 called `lifted`) |
| `past_blow` | binary | the condition is past the blow-off limit, cumulative (not the per-line hazard of `blow_off`) |
| `lift_noted` | binary | a lifted flame is written down as lifted; no feature dependence |
| `flame_length` | positive, m | as `slice` |

**Scoring-time rule for the log's `lifted` label**, with `p` the forecast probabilities of one
line:

```text
p_lifted = p_lift_noted * (p_past_lift - p_past_blow) / (1 - p_past_blow)
```

`p_past_lift >= p_past_blow` always (same network, lower offset), so the result is in [0, 1].
`blow_off` is scored as before from its own target.

## Edits

All lines of the parent block (W0, `phys_slice_w0.md` section 7.4) unchanged, except that every
line `lifted.logit.*` is written under the name `past_lift.logit.*` with the same value. New
lines:

```text
# past the blow-off limit: the past_lift network 0.892 lower (3.4 * ln 1.3).
past_blow.logit.b0 = -4.248
past_blow.logit.skip[0:8] = [3.4, 0.0, 0.0, -1.87, -10.2, -0.258, -0.258, 0.0]
past_blow.logit.W1[0, 0:8] = [-1.0, 1.0, -1.0, -1.3, 0.0, 0.065, 0.088, 0.0]
past_blow.logit.b1[0] = -0.285
past_blow.logit.w2[0] = 1.054
past_blow.logit.W1[1, 0:8] = [-1.0, 1.0, -1.0, -1.3, 0.0, 0.065, 0.088, 0.0]
past_blow.logit.b1[1] = -2.485
past_blow.logit.w2[1] = 1.292
past_blow.logit.W1[2, 0:8] = [-1.0, 1.0, -1.0, -1.3, 0.0, 0.065, 0.088, 0.0]
past_blow.logit.b1[2] = -4.685
past_blow.logit.w2[2] = 1.054
past_blow.logit.W1[3, 3] = -1.0
past_blow.logit.b1[3] = -0.693
past_blow.logit.w2[3] = 1.7
# a lifted flame is noted in the log half of the time.
lift_noted.logit.b0 = 0.0
lift_noted.logit.skip[0:8] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
```
