# bass1 D1: one revision of the plastics law from the train days

Author: `claude-fable-5-2`, 15 November 2026. Kind: **D1, data-informed** (not zero-shot). Parent:
the W0 release `sha256:97c0daf9…` of `docs/wsp/research/phys_bass1_w0.md`. Read for this step:
`docs/wsp/feedback/bass1_train_anatomy.md` (train days only) and the entry of
`flat_quench_speed.median.b0` in `runs/dev/gen7/sensitivity/bass1_97c0daf9de84482e.json`; the two
validation aggregates the coordinator quoted (quench 0.517 against 0.230 and 0.180; sustain 0.086
against 0.120). No validation case, no test day, no per-case file. Nothing was searched or fitted.

## 1. The fault

Train, quench speed: the opposed PMMA slabs are 8 of 39 scored cases and 73 % of the loss. The
forecast median (about 1 cm/s) lies 5.7 cm/s below the observed interval; 2 of 8 fall in the 50 %
band and 3 of 8 in the 90 % band. The films (4 cases, 1 % of the loss, all in the 90 % band) and the
spheres (11 cases, 10 in the 50 % band) are where the law put them.

The W0 law treats a slab of 1 to 5 mm as a film that does not burn out: warm through under a flame
that sits still (W0 sections 1.6 and 3.2; section 9 already says the flat pair has no conduction
term). The slab burns in BASS-II were flame-spread tests. A spreading edge never sits on warm
fuel: it meets cold solid all the time, and the solid under it has been heated only for the time
the edge takes to pass, `t_res = D_h / (U v_f)`. With the W0 numbers (`D_h = 1.7 cm2/s`,
`b_c = 110`, `q_r = 7.2`) and a spread rate of 0.1 to 0.2 mm/s for thick PMMA, `t_res` is 20 to 60 s
at 3 to 5 cm/s and the conduction into the slab is `b_c / sqrt(t_res)`, 14 to 27 kW/m2, two to
nearly four times the surface radiation. The edge then goes out at

```text
U_q = A_e (1 + (b_c / q_r) sqrt(U_q v_f / D_h)) / (X - x_s)        2.8 to 4.1 cm/s at 19 % O2
```

against 0.94 cm/s for the film. So physics alone, with no number of the train days, puts a
spreading slab flame at 3 to 4.4 times the film's quench speed. The train days ask for about
7 times (median error 5.7 cm/s on a median near 0.95).

**Why a film must not move with it.** A film 0.1 to 0.4 mm thick is at one temperature through its
thickness within a second; there is no cold interior to conduct into, whatever the edge does. Its
quench speed is set by surface radiation alone, and the four train films agree with that.

## 2. Can the forward tell a slab from a film? Not as the inputs are filled now

Thickness is not one of the five features. For every flat sample the W0 convention supplies
`nose_radius = 0.01`, `heat_time = 200`; oxygen, speed and direction do not separate the two. With
the inputs as they are, any change of `flat_quench_speed` moves films and slabs together.

`nose_radius` is idle for flat samples (weight zero in the three flat targets) and its domain is
0.002 to 0.03 m. The step below uses it as the conduction length of a flat sample in the crudest
form, a flag:

```text
flat PMMA thicker than 0.5 mm (slabs, thick flat samples):  nose_radius = 0.02
flat PMMA of 0.5 mm or less (films):                         nose_radius = 0.01   (unchanged)
```

This is a change of an input convention of W0 section 7.1, made by the data lane, not a weight;
the declaration is untouched. A hinge on `z_r = ln(nose_radius / 0.01)` is zero at 0.01, so every
film forecast is identical to W0, to the last digit. The proper form is thickness as a sixth
feature, which is a new slug.

If the data lane cannot supply the flag, the edits below are inert (no forecast changes) and the
fallback is one number, `flat_quench_speed.median.b0` from -4.601 to -4.43: a factor 1.19, half of
what the four films themselves ask for (their median error is -0.37 cm/s on about 0.9), with the
film's own spread speed of 0.05 to 0.2 cm/s, neglected in W0, as its reason. It does not repair the
slabs; nothing that leaves the films alone can, without the flag.

## 3. The step

| Number | Old | New | Reason | Expected effect |
|---|---|---|---|---|
| `flat_quench_speed.median.W1[0, 2]` | 0 | 1.0 | hinge on `z_r`, active only above `nose_radius = 0.01` | none on films |
| `flat_quench_speed.median.w2[0]` | 0 | 1.44 | at the flag (`z_r = 0.693`) adds 1.0 to ln(median): a factor 2.7. Conduction into cold fuel at a spreading edge gives 3 to 4.4 by section 1; the train days ask for about 7 (1.95 in ln); 1.0 is half of that and just under the physical estimate | slab median from 0.94 to 2.6 cm/s at 19 % O2 (0.83 to 2.3 in air); the slab error falls from about -5.7 to about -4 cm/s |
| `flat_quench_speed.spread.W1[0, 2]` | 0 | 1.0 | same hinge | none on films |
| `flat_quench_speed.spread.w2[0]` | 0 | 0.43 | ln-sd from 0.50 to 0.80 for slabs: a slab flame may rest on warm fuel (W0, about 1 cm/s; two or three of the eight train slabs did) or run on cold fuel (3 to 4 times that, or more against a cold holder); the theory cannot say which, and half of ln 3.2 added in quadrature to 0.50 is 0.77 | slab 90 % band 0.7 to 9.6 cm/s at 19 %; slabs that went out near 1 cm/s stay inside it |

Four numbers, one fault. Not touched: every film, rod and sphere forecast, the `sustain` pair
(slabs lit and held in every train case, which the flat pair already says with 0.93), all
constants of W0 section 4.

**Size of the effect, stated before it is scored.** With the slab loss near 5.7 cm/s per case and
a median moved from 1 to 2.6 cm/s under a wider band, the slab loss should fall to roughly 3.5 to
4, and the train quench score from 1.33 to about 1.0 to 1.1. That is still above the constant
baseline (0.84): the step is damped by design, and the rods, which the law puts near zero and
which went out near 1 cm/s (24 % of the loss, the diffusion term `D_c / R` of W0 being too strong
for them, as W0 named among its weakest assumptions), are left for the next step.

**What would say this step is wrong.** Slabs on unseen days going out at 1 cm/s or below in the
majority (the median was better where it was), or slab quench speeds that do not fall as the
slab is given time at low flow (then it is not conduction into cold fuel).

## Edits

```text
# bass1 D1: revision of sha256:97c0daf9… (W0). Data-informed: train days only.
# Input convention changed with it: flat PMMA thicker than 0.5 mm is supplied nose_radius = 0.02
# (films stay at 0.01, where the two hinges below are zero).
# A spreading edge on a thick slab meets cold fuel: conduction 110 / sqrt(t_res) on top of the
# surface radiation raises the quench speed 3 to 4 times; typed 2.7 times (half of the train days).
flat_quench_speed.median.W1[0, 2] = 1.0
flat_quench_speed.median.w2[0] = 1.44
# Resting on warm fuel or running on cold fuel: ln-sd 0.50 -> 0.80 for slabs.
flat_quench_speed.spread.W1[0, 2] = 1.0
flat_quench_speed.spread.w2[0] = 0.43
```
