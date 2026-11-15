# bass2: the plastics law with a thickness feature and rods that quench

Author: `claude-fable-5-2`, 15 November 2026. Kind: **data-informed** (train and validation days of
bass1 are study data, PROTOCOL 13.4). Parent law: `phys_bass1_w0.md` with the step of
`phys_bass1_d1.md`. Read for this: the bass1 table of `docs/wsp/feedback/gen7_failure_analysis.md`.
No test day, no per-case file. Nothing fitted; three faults, each answered by the fewest numbers.

## 1. What the table shows

| Stratum | Cases | Observed midpoint | Law's median | Verdict |
|---|---|---|---|---|
| PMMA rod, opposed | 14 | 1.24 cm/s | 0.0006 cm/s | the law says "no quench while the fan turns"; wrong |
| PMMA rod, stagnation | 5 | 0.98 cm/s | 0.007 cm/s | same fault |
| PMMA film, opposed | 5 | 2.38 cm/s | 0.95 cm/s | low by a factor 2.5 (W0 already had the published "below 2 cm/s" at the edge of its band) |
| "kept burning" | | | | no fault shown; untouched |

## 2. Rods: why `D_c / R` was wrong for them

W0 fed every small body by diffusion at `rho D Y_O / R`, as a speed `D_c / R`, 2.8 cm/s for a
1/4 inch rod, more than any quench speed, so rods could not quench. That flux is the steady
diffusion to a **sphere**, which draws oxygen from the whole solid angle around it. A rod burns
along its length. (a) Diffusion to a long cylinder has no steady state: the flux is
`rho D Y_O / (R ln(L / R))` at best, with `L` the burning length (3 to 6 cm against R = 0.3 to
0.6 cm: a third of the sphere's value), and it decays as the surrounding gas is used. (b) The flame
along the rod stands between the tip, or the leading edge, and the fresh gas at the sides, and
takes that oxygen first; downstream of it there are only products. (c) In opposed spread the flame
is an edge moving onto cold rod, not a stagnation flame at all. So for a rod the diffusive speed is
dropped, and what remains of W0 is the forced part alone:

```text
U_q(rod) = (2R/3) a_q,      a_q = [(q_r + q_c) / (k_s (X - x_s))]^2
```

0.54 cm/s for R = 0.318 cm at 18 % and 200 s. Second, W0's own test 2 named the missing term: heat
conducted along a rod does not die away as `1 / sqrt(t)`; a rod is a fin. The flight paper's steady
analysis gives 1.5 to 2 kW/m2 into the rod for as long as it burns, on top of 7.2 radiated:
`a_q` rises by `(1 + 1.8 / 7.2)^2 = 1.56`. I type a factor 1.5 (0.40 in ln). Rod medians at 18 %:
0.81, 1.2, 1.6 cm/s for the 1/4, 3/8, 1/2 inch rods (1.33 times that at 16 %, 0.69 times in air).
The table's 0.98 and 1.24 cm/s ask for a little more for the smallest rod; I stop at what the two
physical corrections give. Spread for rods: 0.75 in ln (the widening that belonged to the
cancelled diffusion term is switched off with it).

Opposed rods keep this same form: an edge flame on a rod has the film's radiative critical speed,
raised by conduction into the rod and lowered by the supply a curved surface gets; the two are of
the same size and I do not separate them. One form for both configurations is what the 19 cases
support.

**Coverage, decided.** Rods in opposed flow are covered for both targets, with the same inputs
(`nose_radius` = diameter / 2, `is_rod` = 1): `quench_speed` for the quench speed, the smaller of
`holds_low_flow` and `holds_high_flow` for "kept burning". This replaces the refusal of W0 section
9 for a rod lit at its downstream end. The blow-off half of that pair describes a stagnation tip;
at ignition speeds below 10 cm/s it does not act, above that the answer for an opposed rod is
weaker than for a tip and should be flagged. Spheres are unchanged (`is_rod` = 0): the table shows
them where the law put them.

## 3. Flat samples: thickness as a feature, and the film coefficient

**Thickness.** The D1 flag on `nose_radius` is retired: every flat sample is again supplied
`nose_radius = 0.01`, where the D1 hinges are zero. The D1 physics moves to `flat_thickness`: a
spreading edge on a thick slab meets cold fuel and loses `b_c / sqrt(t_res)` into it; the heated
depth in the time the edge takes to pass is `sqrt(alpha_s t_res)`, 1.5 to 2.7 mm. A sample thinner
than about 0.5 mm has no cold interior (factor 1); one thicker than 2 mm has all of it (the D1
factor 2.7 and ln-sd 0.80); between them the factor rises linearly in ln(thickness).

**Films.** W0 took the gas speed at the flame edge as the duct speed. The edge sits in the
boundary layer of the sample holder, where the gas moves at half to two thirds of the duct speed at
1 to 3 cm/s, so the duct speed at quench is 1.4 to 2 times W0's value. The table asks for 2.5
(0.92 in ln); I type half of that, 0.46: a factor 1.58, inside the physical range. Film medians:
1.3 cm/s in air, 1.5 at 19 %, 2.0 at 15 %. Slabs of 2 mm and more ride on the same coefficient:
4.1 cm/s at 19 % (D1: 2.6; the train slabs were still under-forecast after D1). The flat
"kept burning" pair is not changed: the table shows no fault there, and I note that it now carries
the old film coefficient.

## For the engineer

Seven features; the first five as in `bass1` (same order, units, transforms, centres), two appended.

| # | `name` | SI unit | transform | centre | scale | domain | Derivation from the condition fields |
|---|---|---|---|---|---|---|---|
| 0 to 4 | as `bass1` | | | | | | unchanged, except: every flat sample `nose_radius` = 0.01 (no flag) |
| 5 | `is_rod` | 1 | identity | 0 | 1 | 0 to 1 | 1 if the sample is a rod (any flow configuration: stagnation, concurrent, opposed); 0 for spheres, spherical sections and flat samples |
| 6 | `flat_thickness` | m | ln | 0.0005 | 1 | 0.00005 to 0.03 | flat samples: the thickness condition of the sheet, film or slab, in metres, as installed (full thickness, one- or two-sided alike); rods, spheres and sections: 0.0005 (weight zero in their targets) |

Targets, kinds and units as `bass1`. Mapping: `quench_u` to `quench_speed` for rods (all
configurations), spheres and sections with the nose to the flow, and to `flat_quench_speed` for
films and slabs; `sustain` to the smaller of the pair of the same geometry, opposed rods included.
Still refused: wake-lit spheres, candles and wax, fabric, Nomex, Ultem, fan-off ignition.

## Edits

All lines of the W0 block (section 7.4 of `phys_bass1_w0.md`) and all lines of the D1 block
unchanged, except the one changed line marked below. They address columns 0 to 4; columns 5 and 6
are zero wherever not written here.

```text
# bass2: data-informed (train and validation days of bass1). Features 5 (is_rod), 6 (flat_thickness).
x.is_rod.centre = 0.0
x.is_rod.scale = 1.0
x.is_rod.lo = 0.0
x.is_rod.hi = 1.0
x.flat_thickness.centre = 0.0005
x.flat_thickness.scale = 1.0
x.flat_thickness.lo = 0.00005
x.flat_thickness.hi = 0.03

# --- rods: a rod is not fed from all sides. The four chords of ln(1 - exp(-v)), the diffusion
# term D_c / R, are switched off for rods (-20 puts each unit below zero over the whole domain).
quench_speed.median.W1[0, 5] = -20.0
quench_speed.median.W1[1, 5] = -20.0
quench_speed.median.W1[2, 5] = -20.0
quench_speed.median.W1[3, 5] = -20.0
# a rod is a fin: 1.5 to 2 kW/m2 conducted along it for as long as it burns: a_q times 1.5.
quench_speed.median.skip[5] = 0.40
# the widening that went with the cancelled diffusion term: off for rods (ln-sd 0.75).
quench_speed.spread.W1[0, 5] = -20.0
quench_speed.spread.W1[1, 5] = -20.0

# --- films: the edge sits in the holder's boundary layer (gas at 0.5 to 0.67 of the duct speed).
# CHANGED LINE: was -4.601. Factor 1.58, half of what the film cases ask for.
flat_quench_speed.median.b0 = -4.14

# --- slabs: conduction into cold fuel at a spreading edge, by thickness (replaces the D1 flag,
# which is inert now that flat samples are supplied nose_radius = 0.01).
# ln(median) rises by 1.0 (factor 2.7) between 0.5 mm and 2 mm, flat outside.
flat_quench_speed.median.W1[1, 6] = 1.0
flat_quench_speed.median.w2[1] = 0.72
flat_quench_speed.median.W1[2, 6] = 1.0
flat_quench_speed.median.b1[2] = -1.386
flat_quench_speed.median.w2[2] = -0.72
# ln-sd rises from 0.50 to 0.80 over the same range.
flat_quench_speed.spread.W1[1, 6] = 1.0
flat_quench_speed.spread.w2[1] = 0.216
flat_quench_speed.spread.W1[2, 6] = 1.0
flat_quench_speed.spread.b1[2] = -1.386
flat_quench_speed.spread.w2[2] = -0.216
```

Expected, stated before scoring: rods from a median near zero to 0.8 to 1.6 cm/s (the stratum that
carried the loss should fall to the level of the published correlation or a little above it, since
the smallest rod is still forecast low); films from 0.95 to about 1.5 cm/s; slabs of 2 mm and more
from 2.6 to 4.1 cm/s at 19 %; spheres and every "kept burning" forecast unchanged except that
opposed rods are now answered.
