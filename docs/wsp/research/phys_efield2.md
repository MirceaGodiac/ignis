# `efield2`: the E-FIELD law with the coflow as an input

Author: `claude-fable-5-2`, 15 November 2026. **Data-informed** (the coordinator's summary of the
train anatomy and of `docs/wsp/feedback/gen7_failure_analysis.md`; no test day, no per-case file;
I did not read the E-FIELD anatomy itself, for lack of time). Parent: the W0 release of `efield`
(`phys_efield_w0.md`). No fit: four numbers, damped.

## Physics of the coflow

W0 left the coflow out "by choice (no physical number to give it)". It does two things.

1. **Current.** In still gas a microgravity flame is large, cool and fed only by diffusion and by
   its own ion wind: less `CH` per carbon, a lower ion yield, and a cold gap to the mesh. With an
   air coflow the flame is compact and hot, and the stream carries a column of hot products to
   the mesh; ion mobility rises in proportion to temperature at constant pressure, so the
   space-charge limit `G` rises too. Both the source and the gap move the same way, so the whole
   current curve shifts. Stated error: low by 0.40 in ln with coflow, high without (size not
   given to me). Step: **+0.20** with coflow, **-0.10** without, relative to W0.
2. **Flame out.** Without coflow nothing renews the air around the flame. It grows, radiates and
   goes out in tens of seconds, as the spherical flames of the same rack do; a weak field delays
   this and does not prevent it. W0 held this in its "other" clock at a median of 600 s. Stated:
   54 % out by 60 s against 8 % forecast. Half-way is about 0.3 by 60 s, a median of **105 s**
   at shape 1.5. With coflow the 600 s stay.

The blow-off clock (ion-wind strain against the flame's extinction strain) is unchanged.

Not typed: the plain space-charge baseline beats the law on current (0.374 against 0.546). I
cannot say from the summary whether that is the level of `G`, the saturation cap or the fall after
the peak; moving `G` or removing the cap without seeing which would be a guess. The shift above
acts on the level only.

## For the engineer

New slug `efield2`: the declaration of `efield` with one feature appended (index 7), targets
unchanged.

| # | `name` | unit | transform | centre | scale | domain | Derived from |
|---|---|---|---|---|---|---|---|
| 7 | `has_coflow` | 1 | identity | 0 | 1 | 0 to 1 | 1 if `u_coflow_m_s` > 0, else 0 |

No scoring-time rule.

## Edits

All lines of the parent block (W0, `phys_efield_w0.md` section 7.4) unchanged except the three
`b0` lines below. Where the parent writes a `skip[0:7]` list, index 7 is zero unless written
below. New and changed lines:

```text
x.has_coflow.centre = 0.0
x.has_coflow.scale = 1.0
x.has_coflow.lo = 0.0
x.has_coflow.hi = 1.0
# current: 0.10 lower in still gas (was -12.939), 0.20 higher than W0 with coflow. The hinge
# positions are not moved, so the rising branch and the plateau shift together.
ion_current.median.b0 = -13.039
ion_current.median.skip[7] = 0.3
peak_ion_current.median.b0 = -13.039
peak_ion_current.median.skip[7] = 0.3
# flame out for lack of air: median 105 s in still gas (was 600 s), 600 s with coflow.
flame_end.other.median.b0 = 4.654
flame_end.other.median.skip[7] = 1.743
```
