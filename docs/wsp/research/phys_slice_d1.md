# SLICE (PSI-106): one revision of the W0 law from train days (D1)

Author: `claude-fable-5-2`, 15 November 2026. **Data-informed**: typed after reading
`docs/wsp/feedback/slice_train_anatomy.md` (train days only) and the coordinator's summary of the
blind look. No validation outcome, no test day, no per-case file was opened. No fit, no search:
one physical number, removed in the six places where the network carries it. Parent: the W0
release of `phys_slice_w0.md`.

## What the train days show

| Stratum (train) | Blow-off forecast | Observed | Lifted forecast | Observed |
|---|---:|---:|---:|---:|
| Methane 40 % (39 lines, day 061, 0.4 mm tube) | 0.670 | 0.077 | 0.773 | 0.077 |
| Methane 100 % (117) | 0.055 | 0.060 | 0.227 | 0.043 |
| Ethane (79) | 0.056 | 0.025 | 0.195 | 0.000 |
| Tubes of 0.8 mm and more (187) | 0.02 to 0.04 | 0.00 to 0.03 | | |
| Jet above 15 m/s (22) | 0.754 | 0.136 | | |

Blow-off is calibrated for pure methane on every tube, the 0.4 mm tube included (day 058: 0.096
against 0.128), and for all forecasts below 0.5. Nine tenths of the loss is one day: 40 % methane
on the 0.4 mm tube, where the law says the flame cannot be there and it is.

## The step: the jet term does not grow with dilution

In W0 the reach of the jet's wall shear at the flame base, `l_j`, was multiplied by
`Z_st / Z_st(methane)`: 3.2 for 40 % methane, on the argument that the stoichiometric surface of a
diluted fuel lies closer to the jet (W0 section 1.3; stated range of the exponent 0 to 1.5). That
is true along the flame, not at its base: the kernel stands outside the rim in the wake of the
tube wall, on the air side, whatever the fuel (the computation the law rests on finds it lean, at
an equivalence ratio of 0.8). The pure-methane lines, which do not feel this factor, confirm
`l_j` itself; only the factor is contradicted. I set its exponent to 0.

- `lifted.logit.W1[0..2, 3]` and `blow_off.logit.W1[0..2, 3]`: -1.3 to **0.0**.

Effect: for 40 % methane the jet part of the base speed falls by a factor 3.2; where the jet
dominates, `ln Phi` falls by 1.15. Day 061: blow-off from about 0.67 to about 0.3 per line, lifted
from 0.77 to about 0.5 (observed 0.08 both). Pure methane, ethane, propane and every line on the
large tubes: unchanged. This is roughly half of what the day asks for; the other half would need
the burning speed of 40 % methane not to fall below that of pure methane, which I do not believe.

Why no more than this: the fuel of day 061 is in doubt. The engineer's note records that the crew
log says "100 % Methane" for that day while the bottle table says 40 %. If the log is right, the
W0 law was already near calibration there (pure methane on the 0.4 mm tube is), and a larger step
would be fitted to a mislabel.

## Not typed

- **`l_j`** (the wall gradient of 1.6e5 1/s recalled from memory): the pure-methane lines on the
  0.4 mm tube are calibrated, so the train days give no reason to move it.
- **`x.x_o2.centre`, `x.x_fuel.centre`** (the most sensitive numbers): they are reference points
  of the inputs, not physics; moving them would be a fit.
- **Lifted** stays over-forecast for pure fuels (0.23 against 0.04). Two causes, neither a
  number: the label is only written when the crew noted it, and the law gives the chance that the
  flame is past the lifting limit, while a flame past the blow-off limit is no longer there to be
  lifted. The right form is a window (between `Phi` = 1 and 1.3), which is a new network for this
  target, not a damped step.
- Sixteen train lines are refused for missing cabin oxygen or coflow: an input matter.

## Edits

```text
lifted.logit.W1[0, 3] = 0.0
lifted.logit.W1[1, 3] = 0.0
lifted.logit.W1[2, 3] = 0.0
blow_off.logit.W1[0, 3] = 0.0
blow_off.logit.W1[1, 3] = 0.0
blow_off.logit.W1[2, 3] = 0.0
```
