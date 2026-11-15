# Plastics (BASS, BASS-II): what the development cases show, and a third law

Author: `claude-fable-5-2`, 15 November 2026, under PROTOCOL 13.6. **Data-informed, in-sample**: train
and validation days of `bass1`. No test day was opened. Read case by case: every PMMA burn of
`runs/dev/data_all/work/transfer_bass1/` with the operators' words beside the coded outcome (rods 41,
spheres and section 60, films 15, slabs 24), the fan calibration workbook, `tools/wsp/ext/bass1_cases.py`,
`docs/wsp/feedback/bass1_feasibility.md`, the bass1 part of `gen7_failure_analysis.md`. Throwaway
scripts: `runs/dev/gen7/study/bass_show.py`, `bass_table.py`, `bass_pots.py`, `bass_loss.py`.
One candidate was run with the logged tool (section 3). The law proposed in section 4 needs two new
inputs and has **not** been run; its expected effects are hand estimates written before any run.

## 0. In one screen

1. **Below about 4 cm/s the outcome is a fan setting, not a speed.** Rods, spheres, films and thin
   slabs all go out at the same place on the fan knob, pot 0.27 to 0.40 (fan display 40 to 41, with
   the fan stopped at 38), whatever the oxygen (14.8 to 22 %), the diameter, the configuration, and
   whatever the day's calibration says that setting is worth. The cm/s written for it (0.5 to
   1.8) is the lane's straight line from the stopped fan to pot 1, times an anemometer reading that
   varies from 0 to 5.75 cm/s between days at the same display. No critical speed can be read from
   these burns, mine or the published one.
2. **Why the published correlation wins on films and spheres:** it is nearly a constant, 1.3 to
   1.7 cm/s, and that is what pot 0.3 to 0.4 is worth on the lane's scale on a typical day. Its
   oxygen trend is not in the rod data (22 burns, no trend from 15 to 21 %).
3. **Slabs are the only real extinction speeds** (anemometer readings of 5 to 18 cm/s), and they
   are not one critical speed. A slab either stays above its oxygen limit and burns down to the
   same fan floor as everything else, or it reaches that limit during a long burn and dies at
   whatever flow it was being held at, 8 to 18 cm/s. Which of the two is decided by thickness,
   by one or two burning sides, and by the starting oxygen. 85 % of the slab loss is four such burns.
4. **Checked:** a floor with no physics for rods, spheres and films (one median, no radius, oxygen
   or time term) lowers the quench score from 0.393 to 0.314, on 20 of the 22 days that change,
   as predicted in sign and roughly in size (K1, section 3).
5. **Proposed:** a new slug `bass3` with two inputs, the speed the burn's own fan curve gives at
   pot 1 and a one-sided indicator, carrying the floor on the lane's scale and a two-class slab
   law. Expected: rods 0.11 to about 0.05, slabs 2.06 to about 1.0, pooled 0.31 to about 0.15.
6. **The "kept burning" pair is left as it is**, flat pair included (section 5).

## 1. What the data showed

### 1.1 The setting at which the flame goes out

Scored burns that went out at a stated setting below pot 1 (the setting is recovered from the coded
speed through the burn's own curve; `bass_pots.py`):

| Sample | Burns | Pot at quench, median (range) | On the lane's scale |
|---|---:|---|---|
| rods, all | 22 | 0.312 (0.270 to 0.380) | 0.51 to 1.49 cm/s |
| spheres, nose to the flow | 4 | 0.305 (fan stopped to 0.32) | 0 to 1.33 |
| films | 3 | 0.400 (0.355 to 0.400) | 1.34 to 1.80 |
| slabs that reached low settings | 4 | 0.35 (0.30 to 0.48) | 1.4 to 5.0 |

Ten more spheres, one rod, two films and two slabs held their lowest setting (pot 0.30 to 0.50 for
all but three) and went out only when the fan was stopped or the sample was gone. The 4 cm
spherical section went out at pot 0.28.

Rods, split three ways:

| Split | Burns | Pot at quench, median (range) | On the lane's scale |
|---|---:|---|---|
| day's curve gives 2.0 cm/s at pot 1 | 8 | 0.328 (0.27 to 0.38) | 0.51 to 0.72 cm/s |
| 3.5 to 4.0 cm/s at pot 1 | 10 | 0.310 (0.275 to 0.36) | 1.04 to 1.36 |
| 5.0 cm/s at pot 1 | 4 | 0.305 (0.30 to 0.315) | 1.42 to 1.49 |
| oxygen 14 to 16 % | 3 | 0.310 | |
| 16 to 18 % | 10 | 0.307 | |
| 18 to 20 % | 5 | 0.320 | |
| 20 to 23 % | 4 | 0.315 | |
| 6.35 mm | 13 | 0.310 | |
| 9.5 mm | 9 | 0.320 | |
| stagnation / concurrent / opposed | 5 / 4 / 13 | 0.310 / 0.304 / 0.315 | |

If the rods had a critical speed, the pot at quench would be 2.5 times higher on the days whose
curve is 2.5 times lower. It is the same. The speed in cm/s differs between those days by exactly
the ratio of the curves. So the variation of the rod outcome is the day's calibration, and the
quantity that is constant is the setting.

### 1.2 What the calibration says about that setting

Anemometer readings in the calibration workbook, two restrictors, same fan display:

| Pot (fan display) | Readings over the calibration days, cm/s |
|---|---|
| 0 (38) | 0 on every day: fan stopped |
| 0.5 (42) | 0, 0, 0, 0, 0.5, 1, 1, 2, 2, 3, 3, 3.75 |
| 1.0 (46.5 to 47) | 0, 2, 2.5, 3, 3.5, 4, 4, 4, 4, 4.75, 4.75, 5, 5, 5, 5, 5, 5.75 |
| 2.0 (55) | 3, 4, 5, 5, 6, 6, 7 to 9 on fifteen days |

The fan and the restrictors are the same on all these days; a reading of 0 and one of 5.75 at one
display are the instrument, not the flow (the probe took a minute to respond and was near the
bottom of its range). The lane's rule, speed linear in the display from the stopped fan to the
day's reading at pot 1, therefore puts a number between 0.5 and 1.8 cm/s on a setting whose true
speed is unknown: it could be 0 (the fan reads 38 stopped and may well not turn at 40.5) or 1 to
2 cm/s. The operators wrote "finite low flow" beside several of these; one wrote that the flame
"doesn't go out until fan goes to 38".

**How much of the error is the scale?** For rods, spheres, films and thin slabs: all of what can be
resolved. The intervals are 0.02 to 0.1 cm/s wide at about 1 cm/s, and their position is the day's
curve. For the slabs that carry the loss: none. Their quench speeds are anemometer readings of 5
to 18 cm/s written by the operators (M3, M11, M14, M20: 17.6 of the 20.7 total slab loss of bass2).

### 1.3 The slabs, one by one

All opposed spread, PSI-25. "End" is from the operators' words; starred tests are not scored by the
lane's rules but the words state the end.

| Test | Thick. mm | Sides | Width cm | O2 start to end, % | End | Class |
|---|---:|---:|---:|---|---|---|
| M1 | 1 | 1 | 2 | 22.2 to 21.9 | weak at 8 cm/s; put out by stopping the fan | (below 8) |
| M14 | 2 | 1 | 2 | 22.2 to 21.6 | went out by itself at 9 cm/s | high |
| M2* | 3 | 1 | 2 | 22.2 to 21.2 | very dim at 13, raised to 14, went out | high |
| M3 | 5 | 1 | 2 | 20.8 to 19.7 | raised 10 to 20, out at 18 | high |
| M7 | 1 | 2 | 1 | 18.9 to 18.4 | burned the whole sample, lowest pot 0.5 | floor |
| M6* | 1 | 2 | 2 | 21.8 to 20.7 | plateaus at pot 0.4 and 0.36, reached the end | floor |
| M8 | 2 | 2 | 1 | 19.1 to 18.2 | 25 min; out at pot 0.4 | floor |
| M17* | 2 | 2 | 2 | 18.8 to 17.1 | 30 min at 10, 5, 3 cm/s; fan stopped | floor |
| M12* | 2 | 2 | 2 | 21.7 to 20.4 | plateaus at pot 0.35 to 0.4 | floor |
| M5 | 3 | 2 | 2 | 22.2 to 19.0 | no restrictor; out at 5 cm/s after flashing | between |
| M9* | 3 | 2 | 1 | 18.0 to 17.6 | weak at 10, raised, died after 15 min at 14 | high |
| M18* | 3 | 2 | 2 | 17.9 to 16.8 | 26 min; out at 8 | high |
| M13 | 4 | 2 | 2 | 20.8 to 19.0 | 6 min each at pot 1.7, 1.0, 0.55; out at pot 0.3 | floor |
| M4 | 5 | 2 | 2 | 21.8 to 19.8 | the same | floor |
| M11 | 5 | 2 | 1 | 20.0 to 18.5 | 29 min; raised to 20, lowered, out at 12 | high |
| M20 | 5 | 2 | 2 | 18.0 to 17.4 | 8 min at 10, out 3 min after going to 8 | high |

PSI-26, 2 mm one-sided, 1 cm wide, air: lit from the upstream edge it burned for minutes at 4 to
9 cm/s as a 1 cm blue flame; in opposed flow at 21 % it could not be kept alight at 9 or 15 cm/s
(tests 72, 73).

What decides the class:

- **Two-sided, 1 to 2 mm:** floor in all five, from 18.8 to 21.8 % oxygen.
- **Two-sided, 3 to 5 mm:** floor when the burn started at 20.8 % or more (M4, M13); high when it
  started at 20 % or less (M9, M18, M20, M11). At the end of the "high" burns the oxygen was 16.8
  to 18.5 %.
- **One-sided, 2 mm and more:** high in all three, even from 22.2 %, and higher with thickness
  (9, 13 to 14, 18 cm/s for 2, 3, 5 mm).
- **Not deciding, as far as 16 burns can say:** restrictor count and the day's curve (the high
  class is read from the anemometer). Width may matter (M11 and M9, both 1 cm wide, are the highest
  two-sided ones) but M7 and M8, also 1 cm, are floor; I do not use it.
- **History:** the high class went out during long burns (11 to 29 min) in which the oxygen fell
  by 0.4 to 1.5 points, twice after the flow had just been lowered and twice at a held flow. The
  operators raised the flow when the flame looked weak (M3, M9, M11) and kept these flames at 8 to
  20 cm/s; that is where they died. The speed recorded is the flow of the moment, not a property.

So the slab outcomes are **not** one critical speed per condition. They are two outcomes, "burned
down to the fan floor" and "reached its oxygen limit at the working flow", and the second has a
spread set by what the operators did.

### 1.4 Traces of the solid's history

They exist and are small on this scale. A rod burned again went out lower (O15a pot 0.32, O15b
0.27, "rod heatup effect?"); one "not preheated enough" went out higher (P22a 0.32, then 0.31
twice). A 2 cm sphere at 17 % took 3 to 4 minutes to light when new, 3, 2 and 1 minutes on three
successive burns of one day, and 15 s when "still hot". That is the heat-up clock of W0 section
1.4, seen in the ignition time, where the scale does not interfere.

### 1.5 "Kept burning"

101 burns answered, Brier 0.025 by case. The misses: a film at 15.0 % (forecast 0.70, did not
hold), a 9.5 mm rod at 14.5 % (0.69, did not hold), a sphere whose igniters broke (0.97, coded 0),
and slab M2 (0.42, held). Rods held at 14.8 % and above and failed at 13.6 and 14.5 %: the floor of
13.4 % with its 0.8 % width is where it should be for burns lit from cold. No stratum is off by
more than 0.07 in mean forecast against observed rate. No fault to repair.

### 1.6 For the data lane (found while reading)

- **Day curves below pot 2 put anemometer noise into the outcome** (1.2). One campaign curve for
  every day, or the setting itself as the outcome below pot 1, would remove a factor of up to 2.9
  that has no physical cause.
- M2: the ignition speed is coded 55.8 cm/s (fan 116, no restrictor, through the curve); the
  operators' own reading is 20 cm/s. The "kept burning" forecast is 0.42 for that reason alone.
- PSI-26 tests 72 and 73: the words say the flame could not be kept alight; coded as sustained.
- PSI-26 2013-05-15 test 53: "no ignition, broke 2 igniters" is an igniter failure, coded as not
  sustained.
- M2, M9, M17, M18: the end is stated in the words or cells ("self- extinguished", "died after
  15 min at 14 cm/s", "0.3ext", "extinguished 8 cm/s") and the rules miss it: four more slabs.
- The final oxygen is in the workbook and matters for slabs (it fell by up to 3.2 points in a burn).

## 2. Mechanism

**The floor.** A PMMA flame of any of these shapes survives every flow the fan can deliver and dies
where the fan stops delivering. This is W0's own statement for small bodies ("no quench while the
fan turns") and it turns out to hold for the 2 cm spheres and the films as well, on this
instrument. Films stop about 0.07 pot earlier than rods and spheres (0.36 to 0.40 against 0.31),
and two films at 17.4 % earlier than one at 20.8 % (0.40, 0.40 against 0.355): the order W0's
physics gives, and all that these data can say about it. In real speed the difference could be
anything from 0.1 to 0.5 cm/s.

**The slab's oxygen limit.** A flame spreading against the flow over a thick slab must heat fuel
in depth all the time, so it needs more oxygen than a film: its limit is 17 to 18.5 % for 3 to
5 mm burning on both sides (the oxygen at which M9, M18, M20, M11 died), below 17 % for 1 to 2 mm
(M17 alive at 17.1 %), and about 20 to 21.5 % for a slab burning on one side, which has no second
flame to warm it and a holder behind it (M3 died at 19.7 %, M14 at 21.6 %; the PSI-26 one-sided
slab would not burn in opposed flow at 21 %). Near that limit the flammable range of flow is a
narrow window around the speed at which the two branches of the boundary meet, `sqrt(A / s)`,
10 to 12 cm/s with W0's blow-off slope and the film coefficient: where these flames were kept and
where they went out. A sealed 250 L volume loses 0.6 % oxygen per gram burned, so a long burn that
starts within 1.5 to 2 points of the limit reaches it. A burn that starts higher does not, and the
operators then take it down to the fan floor.

## 3. Candidates

One candidate was logged. Two were considered and not run because I predicted they would lose.

### K1: the floor without the day's scale (logged, `runs/dev/gen7/study/bass_k1.wsp`, sha256 `2f794051…`)

Revision of `bass2`. Rods and spheres: one median, 1.2 cm/s (pot 0.31 at the campaign calibration),
ln-sd 0.30, the radius, oxygen, time and rod terms and the four diffusion chords set to zero.
Films: spread 0.50 to 0.30 (slabs of 2 mm and more kept at 0.80). 15 elements.

| Stratum | Expected (written in the log before the run) | Realised |
|---|---|---|
| rods | 0.264 to about 0.13; lower on every rod day except 2014-07-29 | opposed 0.186 to 0.091, stagnation 0.371 to 0.206, concurrent 0.122 to 0.042 (all 23 rods: 0.215 to 0.107) |
| spheres | 0.118 to about 0.03 | 0.091 to 0.024 |
| films | 0.061 to about 0.035 | 0.086 to 0.052 |
| slabs | unchanged | 2.066 to 2.060 |
| pooled, day mean | 0.4505 to about 0.37 | 0.393 to 0.314 (train 0.430 to 0.340, validation 0.249 to 0.214) |
| days | | lower on 20 of the 22 days whose forecasts differ |
| kept burning | identical | identical (0.0233) |

My "before" numbers were those of the 48-case comparison; the tool scores the 52 cases bass2
answers, hence the different bases. Sign right in every stratum, size right to about 20 %. On the
validation days alone the published correlation is still lower (0.180 against 0.214): two of the
nine validation cases are slabs M4 and M13, which K1 does not touch.

By the rule of 13.6 K1 can be kept as it stands (lower pooled, 20 of 22 days, other target
identical). It is the fallback if `bass3` is not built.

### Not run

- **A slab switch on thickness and oxygen inside `bass2`.** Without the number of burning sides the
  one-sided slabs (M3 at 20.8 %, M14 at 22.2 %) fall on the "floor" side of any rule that keeps M4
  and M13 there. Hand estimate: M3 from 8.3 to about 16, M14 from 3.0 to about 7, against gains of
  about 4 on the others. Predicted to lose; the missing input is named instead (13.6, structure
  before numbers).
- **Lowering the floor of "kept burning" for burns lit from cold (13.4 to 14.2 %).** About -0.09
  on one rod and +0.06 spread over six others that held at 14.8 to 15.4 %. No net effect expected.

## 4. Proposed: `bass3`, a new slug with two new inputs

New slug, not a revision: it needs `u_pot1` and `is_one_sided`. It has not been run. Expectations
below are hand estimates from the case table, made before any run; the first check belongs to the
logged tool once the slug exists, with this section as the note.

### 4.1 The law

```text
rods, spheres, sections:   U_q = 0.941 * 0.31 * u_pot1                     ln-sd 0.15
films and slabs, floor:    U_A = 0.941 * 0.39 * u_pot1 * (X / 0.18)^-0.6   ln-sd 0.15
slabs, at the oxygen limit: U_B = 11 cm/s * (thickness / 3 mm)^0.3 * 1.22^(one-sided)   ln-sd 0.35
flat samples:              ln U_q = clamp( ln U_A + 30 (z_c - ln(X / 0.18)),  ln U_A,  ln U_B )
z_c = ln(X_c / 0.18):      X_c = 18.0 % at 2 mm, 20.6 % at 5 mm (two sides), power law between;
                           one-sided: X_c times 1.32
ln-sd 0.80 half-way between the two levels
```

`u_pot1` is the speed the burn's fan curve gives at pot 1 (fan display 46.5); 0.941 is the lane's
own geometry (8 display units per pot below pot 0.5, 8.5 from the stopped fan to pot 1). 0.31 and
0.39 are the settings of section 1.1. The exponent -0.6 for flat samples is half of W0's -1.2, on
two films at 17.4 % and one at 20.8 %; it is the weakest number here. `X_c` is in starting oxygen
and so contains the fall during the burn. The slope 30 makes the change of class take 1.2 points
of oxygen (M17 against M18, M13 against M11 flip within 0.9). With `u_pot1` the same law holds
under the present coding and under a single campaign curve (it then becomes a constant).

What this law is: a statement about this duct. It says nothing about a critical speed below
4 cm/s and should not be quoted as one.

### 4.2 Expected effect, before running (base: `bass2`, 52 cases)

| Stratum | bass2 | K1 (run) | bass3 (expected) | Why |
|---|---:|---:|---:|---|
| rods (23) | 0.215 | 0.107 | about 0.05 | the day's curve removes the 0.5 to 0.7 against 1.0 to 1.5 split; left: pot 0.27 to 0.38 around 0.31 |
| spheres (14) | 0.091 | 0.024 | 0.01 to 0.02 | all on one curve already; narrower band |
| films (5) | 0.086 | 0.052 | about 0.03 | B16, B17 at 1.80 and B21 at 1.34 follow their curves |
| slabs (10) | 2.066 | 2.060 | about 1.0 | M3 8.3 to about 1.4, M14 3.0 to about 1.1, M20 1.7 to about 1.4, M4, M13, M8, M7 2.1 to about 0.1 together; M11 stays near 4.3 (forecast 4 cm/s, went out at 12 to 15); M5 0.9 to about 1.0 |
| pooled, day mean | 0.393 | 0.314 | about 0.15 | slab days 3.4, 1.3, 0.6, 0.6 to about 1.5, 0.7, 0.5, 0.0; the twenty other days to 0.03 to 0.06 |
| days | | 20 of 22 lower | lower than bass2 on every slab and rod day | |
| kept burning | 0.0233 | same | same | no line of the four holds targets changes |

Drop it if rods or slabs do not fall, or if the slab gain comes with M4 and M13 getting worse.

### 4.3 For the engineer

Nine features: the seven of `bass2` in the same order, two appended.

| # | `name` | SI unit | transform | centre | scale | domain | Derivation from the condition fields |
|---|---|---|---|---|---|---|---|
| 0 to 6 | as `bass2` | | | | | | unchanged (flat samples `nose_radius` = 0.01; `flat_thickness` as in bass2) |
| 7 | `u_pot1` | m/s | ln | 0.044 | 1 | 0.01 to 0.15 | the case's `fan_curve` evaluated at fan display 46.5 (linear interpolation, as `fan_to_u(46.5, fan_curve)` in `bass1_cases.py`), divided by 100. For every sample kind |
| 8 | `is_one_sided` | 1 | identity | 0 | 1 | 0 to 1 | flat samples: 1 if `sides_burning` is 1, else 0; rods, spheres, sections: 0 |

Targets, kinds, units and the mapping of `quench_u` and `sustain` as `bass2`. Coverage as `bass2`.

### 4.4 Edits

All lines of the `bass2` release (W0 block with its one changed line, D1 block, bass2 block)
unchanged, except the lines below. Columns 7 and 8 are zero wherever not written.

```text
# bass3: data-informed (train and validation days of bass1, PROTOCOL 13.6). Not yet run.
x.u_pot1.centre = 0.044
x.u_pot1.scale = 1.0
x.u_pot1.lo = 0.01
x.u_pot1.hi = 0.15
x.is_one_sided.centre = 0.0
x.is_one_sided.scale = 1.0
x.is_one_sided.lo = 0.0
x.is_one_sided.hi = 1.0

# --- rods, spheres, sections: out at pot 0.31 whatever radius, oxygen, time (22 rods 0.27-0.38,
# spheres 0.29-0.32). b0 = ln(0.941 * 0.31 * 0.044 m/s). The speed follows the burn's fan curve.
quench_speed.median.b0 = -4.356
quench_speed.median.skip[0] = 0.0
quench_speed.median.skip[2] = 0.0
quench_speed.median.skip[3] = 0.0
quench_speed.median.skip[5] = 0.0
quench_speed.median.skip[7] = 1.0
quench_speed.median.w2[0] = 0.0
quench_speed.median.w2[1] = 0.0
quench_speed.median.w2[2] = 0.0
quench_speed.median.w2[3] = 0.0
# the setting is reproducible to 0.03 pot in 0.31; with the conversion, ln-sd 0.15.
quench_speed.spread.b0 = 0.15
quench_speed.spread.w2[0] = 0.0
quench_speed.spread.w2[1] = 0.0

# --- flat samples, floor level: pot 0.39 at 18 % O2 (films 0.355-0.40), b0 = ln(0.941 * 0.39 * 0.044);
# -0.6 per e-fold of oxygen (two films at 17.4 %, one at 20.8 %).
flat_quench_speed.median.b0 = -4.126
flat_quench_speed.median.skip[0] = -0.6
flat_quench_speed.median.skip[7] = 1.0
# the D1 flag and the bass2 thickness ramp are replaced by the two-class law below.
flat_quench_speed.median.w2[0] = 0.0
flat_quench_speed.median.w2[1] = 0.0
flat_quench_speed.median.w2[2] = 0.0
# unit 3: g = 30 (z_c - z_x), z_c = -0.204 + 0.147 ln(thickness / 0.5 mm) + 0.28 one_sided:
# starting oxygen below which a slab reaches its limit during the burn (18.0 % at 2 mm, 20.6 % at
# 5 mm, two sides; 1.32 times that one-sided). Adds g to ln(median) above the floor level.
flat_quench_speed.median.W1[3, 0] = -30.0
flat_quench_speed.median.W1[3, 6] = 4.41
flat_quench_speed.median.W1[3, 8] = 8.4
flat_quench_speed.median.b1[3] = -6.12
flat_quench_speed.median.w2[3] = 1.0
# unit 4: the cap. Floor level + g never above the working flow of a slab at its limit,
# 11 cm/s (thickness / 3 mm)^0.3, 1.22 times one-sided: ln = -2.744 + 0.3 z_th + 0.2 one_sided.
flat_quench_speed.median.W1[4, 0] = -30.6
flat_quench_speed.median.W1[4, 4] = 0.405
flat_quench_speed.median.W1[4, 6] = 4.11
flat_quench_speed.median.W1[4, 7] = 1.0
flat_quench_speed.median.W1[4, 8] = 8.2
flat_quench_speed.median.b1[4] = -7.502
flat_quench_speed.median.w2[4] = -1.0
# spread: 0.15 at the floor level, 0.80 half-way between the classes, 0.35 at the limit level.
flat_quench_speed.spread.b0 = 0.15
flat_quench_speed.spread.w2[0] = 0.0
flat_quench_speed.spread.w2[1] = 0.0
flat_quench_speed.spread.w2[2] = 0.0
flat_quench_speed.spread.W1[3, 0] = -30.0
flat_quench_speed.spread.W1[3, 6] = 4.41
flat_quench_speed.spread.W1[3, 8] = 8.4
flat_quench_speed.spread.b1[3] = -6.12
flat_quench_speed.spread.w2[3] = 0.7
flat_quench_speed.spread.W1[4, 0] = -30.0
flat_quench_speed.spread.W1[4, 6] = 4.41
flat_quench_speed.spread.W1[4, 8] = 8.4
flat_quench_speed.spread.b1[4] = -7.05
flat_quench_speed.spread.w2[4] = -1.18
flat_quench_speed.spread.W1[5, 0] = -30.0
flat_quench_speed.spread.W1[5, 6] = 4.41
flat_quench_speed.spread.W1[5, 8] = 8.4
flat_quench_speed.spread.b1[5] = -7.99
flat_quench_speed.spread.w2[5] = 0.48
```

Fallback without a new slug (already run, section 3): the fifteen lines of
`runs/dev/gen7/study/bass_k1.wsp` on `bass2`.

## 5. The flat "kept burning" pair and the film coefficient

Settled: **the pair stays on W0's film coefficient.** The line changed in bass2
(`flat_quench_speed.median.b0`, a factor 1.58) was typed to match film quench speeds of about
1.8 cm/s. Those are pot 0.40 on a day whose curve gives 4.8 cm/s at pot 1: the fan floor on the
lane's scale, not a thermal limit of the film. In `bass3` that line becomes the floor and the
factor disappears. The pair describes ignition speeds of 2 to 7.5 cm/s, where the scale is the
anemometer's own; there the three films lit at 2 cm/s in air all held, which W0's coefficient
gives 0.97 and the bass2 factor would give 0.85. So there is no inconsistency left to carry over:
one target states the floor of the instrument, the other the flame.

## 6. What remains

**Measurement limits.**

- No speed below about 4 cm/s is known to better than a factor of two to three, and the quench
  setting may be a stopped fan. The quench speed of rods, spheres, films and thin slabs cannot be
  measured from these records. Neither W0's numbers (0.1 to 1 cm/s) nor the published critical
  speed (1.3 to 1.7 cm/s) is tested by them. What would test it: the fan's tachometer or a smoke
  trace at pot 0.3 to 0.5, or the overlay radiometer with the setting, on video.
- Oxygen on PSI-25 is known to 1.5 points; the slab classes flip within 0.9.

**Structural.**

- The slab outcome is a mixture of two endings chosen partly by the operators (how long the burn
  ran, whether the flow was raised when the flame looked weak). A single log-normal with a moving
  median is a poor shape for it; the forward has no two-class positive target. `bass3` puts a wide
  band at the boundary instead.
- M11 (5 mm, two sides, 1 cm wide, 20.0 %, out at 12 to 15 cm/s after 29 minutes) is on the wrong
  side of any rule that keeps M13 (4 mm, 20.8 %) at the floor. Width, or the 1.5 points the oxygen
  fell, would explain it; one case does not justify an input.
- The speed at which a slab at its limit dies is the flow the operators were holding. A forecast
  made before the burn can only say "the working flow of weak thick-slab flames, 8 to 18 cm/s".
- Slab burns with no restrictor (M5, M2) sit on a curve 2.5 times steeper; one scored case.

**Left of the physics of W0.** The floor of 13.4 % and the ignition clock stand on what was seen
(1.4, 1.5). The thermal limit of a slab in opposed flow was wrong in W0, D1 and bass2 for the same
reason: all three moved a film's quench speed up with thickness, where the data ask for an oxygen
limit that thickness raises and a flammable window around 10 cm/s.
