# `cld2`: the CLD loss law with the procedure as an input

Author: `claude-fable-5-2`, 15 November 2026. **Data-informed** (train and validation days, read
through `docs/wsp/feedback/gen7_failure_analysis.md`; no test day, no per-case file). Parent: the
D1 release of `cld` (`phys_cld_w0.md` with the three lines of `phys_cld_d1.md`). No fit: two
numbers, sized from the physics and damped.

## Physics of the two new inputs

A test whose script ramps the fuel fraction down, or ramps the velocities, is a test designed to
find the limit. The law so far asked "is the most severe condition beyond my estimate of the
limit?", with an uncertainty of 0.5 in ln on that estimate. For a ramp the right question is "does
the limit lie inside the range the script sweeps, and when is it crossed?". The person who wrote
the script knew roughly where the limit is and set the end of the ramp beyond it. That knowledge
is information the law did not have, and it acts exactly like a shift of the limit towards the
swept range.

- **Dilution ramp** (`dilutes`). The ramp walks down the burning-speed table at fixed speed. I
  take the designer's margin as a factor 2 in `Phi`: the loss clock runs twice as fast. A test the
  parent put at 0.39 goes to about 0.87, one at 0.09 to 0.5, one at 0.03 to about 0.2 (strata:
  81 % lost, parent forecast 39 %).
- **Velocity ramp** (`velocity_ramp`). The analysis notes these ramps go downwards. The law has a
  blow-off limit at high speed and no limit at low speed, where a small flame loses its heat to
  the tube and goes out (W0 section 8.2 lists it as not considered). I cannot type that limit
  from one flag; the flag carries only that a limit is being sought. Factor 1.57 on the clock,
  smaller than for dilution because the direction of the ramp is not the one the law models
  (strata: 58 % lost, parent forecast 25 %; expected about 0.4).

With both flags 0 every forecast of D1 is unchanged. The radiometer target is unchanged.

Structural, not typed: the low-speed limit itself (it needs the fuel power at the end of the ramp
as a feature and a second clock), and the timing inside a ramp (the clock still reads the window,
not the ramp's own start and end).

## For the engineer

New slug `cld2`: the declaration of `cld` with two features appended (indices 6 and 7), targets
unchanged.

| # | `name` | unit | transform | centre | scale | domain | Derived from |
|---|---|---|---|---|---|---|---|
| 6 | `dilutes` | 1 | identity | 0 | 1 | 0 to 1 | 1 if `x_fuel_final` < `x_fuel_initial`, else 0 |
| 7 | `velocity_ramp` | 1 | identity | 0 | 1 | 0 to 1 | 1 if `schedule_from_ignition_command` has a stage `ramp_1`, `ramp_2` or `ramp_3`, else 0 |

No scoring-time rule.

## Edits

All lines of the parent block (D1: the W0 block of `phys_cld_w0.md` section 7.4 with the three
changed lines of `phys_cld_d1.md`) unchanged. Where the parent writes a `skip[0:6]` list, indices
6 and 7 are zero unless written below. New lines:

```text
x.dilutes.centre = 0.0
x.dilutes.scale = 1.0
x.dilutes.lo = 0.0
x.dilutes.hi = 1.0
x.velocity_ramp.centre = 0.0
x.velocity_ramp.scale = 1.0
x.velocity_ramp.lo = 0.0
x.velocity_ramp.hi = 1.0
# a dilution ramp is run to cross the limit: the clock runs twice as fast (ln 2 = 0.69).
loss.blowoff.median.skip[6] = -0.7
# a velocity ramp seeks a limit the law does not model in that direction: factor 1.57.
loss.blowoff.median.skip[7] = -0.45
loss.other.median.skip[6] = 0.0
loss.other.median.skip[7] = 0.0
radiometer.median.skip[6] = 0.0
radiometer.median.skip[7] = 0.0
radiometer.spread.skip[6] = 0.0
radiometer.spread.skip[7] = 0.0
```
