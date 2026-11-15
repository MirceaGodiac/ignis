# BASS — the flame-front model (`bass_front.wsp`)

**State: development model, scored in sample widths on burns measured from the public videos.**
It is the first BASS model in this repository that has met real data. It forecasts where the tip of
the flame will be, given the first seconds of the burn and the fan history. Every number was typed
by the author; the validation burns were measured by agents that knew nothing of the model, into
files the author has not opened.

| Candidate | Release | How written |
|---|---|---|
| **D1** (carried) | `sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c` | data-informed: the four train burns and the published theory of concurrent spread (35 non-zero numbers of 159) |
| D2, not adopted | `sha256:0187941810a89347a9ea3953add9aa6058152bb643b009ca1ffa3e9471f216a7` | D1 without the oxygen boundary of its quench law; written after D1's validation table, so not blind |
| persistence | `sha256:ce53fe1fb3e6300c5e818dd7c9c5ec8e8f4c2cfd19673449d3936de05527de46` | reference: the tip keeps the speed the prefix shows |

## Result (validation, read once)

Twelve of the seventeen validation burns can be scored (40 cases; the measurers found no flame track
in two, too few stills in one, no top view in one, and one day has no fan calibration). Lengths are
sample widths; nothing here is in metres.

| | D1 | persistence |
|---|---|---|
| CRPS, mean over burns | **0.117** | 0.234 |
| Absolute error of the median | 0.16 | 0.32 |
| Coverage of the 50 % / 90 % band | 0.66 / 0.90 | 0.34 / 0.62 |
| Has the track ended? (Brier, 10–60 s ahead) | 0.178 | 0.277 |
| Spread or no spread, from the conditions alone (Brier, 16 burns) | 0.170 | 0.125 |

Persistence minus D1, paired over the 40 cases: CRPS +0.117 [+0.051, +0.182] (bootstrap over burns);
D1 is better in 10 of 12 burns. On train (three burns, the ones the numbers were typed from): 0.076
against 0.242.

**What failed.** The quench law. Saying "it will spread" for every burn scores 0.125 on the ignition
outcome; D1 scores 0.170. It was written from one train burn that never spread (16.4 % oxygen) and a
published boundary for another material, and it does not carry to another day. The tip forecast does
not depend on it.

By cutoff the error falls as the prefix grows (0.18, 0.12, 0.09, 0.04 at 5, 10, 20, 40 s); by lead
it grows (0.07, 0.15, 0.17 for 0–10, 10–30, 30–60 s ahead).

**D2: the boundary withdrawn, and what that showed.** D2 keeps only what needs no boundary (a
background rate, and a quench below 1 cm/s, where every published boundary puts it). It was
registered with its rule before its validation look: serve it only if the tip scores are D1's and
the track-ended score is not worse.

| Validation | D1 | D2 | persistence |
|---|---|---|---|
| Tip CRPS | 0.117 | 0.117 (identical case by case) | 0.234 |
| Has the track ended? | **0.178** | 0.183 | 0.277 |
| Spread or no spread | 0.170 | **0.121** | 0.125 |

The track-ended score is worse by 0.005, so by the rule **D2 is not adopted** and D1 stays the
carried release. The trade is on record: the oxygen boundary costs 0.05 on the question it was
written for and is worth 0.005 on a flame that is already burning. What the model lacks is a class,
not a number: a sample either carries a flame at its conditions or does not, and a flame seen
spreading has answered that. A hazard cannot say both.

## What the model says

A flame on a thin fuel in concurrent flow is a band that travels with the flow. Three places on the
sample describe it.

- **The front** (where the fuel starts to burn) advances at the spread rate and follows a change of
  flow at once: `v = (0.0215 + 0.0064 U) (X_O2 / 0.19)^1.5` sample widths per second, `U` in cm/s, for
  the 2 cm sample.
- **The tip** of the visible flame is ahead of the front by an overhang, `h = 0.065 U (X_O2 / 0.19)^3`
  widths, which follows the flow within about 2 s. A step down in flow pulls the tip back while the
  front goes on.
- **The base** repeats the front one burn-through time later (6.3 s). After a step down it keeps the
  old speed for that long, so the flame shortens from both ends.

The run's own speed is read from the prefix: the advance seen over the advance the law gives under
the same flow history. It is trusted little for a short prefix or one that ends just after a change
of flow, and almost fully after 35 s of settled spread.

| Number | Value | From |
|---|---|---|
| Spread rate at low flow | 0.037 w/s at 2.4 cm/s, 18.8 % O2 | T5, T6 after their fan steps; T16 at 17.6 % and 3.0 cm/s |
| Slope in flow | 0.0064 w/s per cm/s | T5 before its step (10.9 cm/s); linear in every published model |
| Oxygen power | 1.5 | the models say 1 to 2; two oxygen levels in train |
| Overhang | 0.065 w per cm/s | three fan steps: the tip fell back 0.21 w, stalled 3.5 s, stalled one still |
| Burn-through time | 6.3 s | steady length minus overhang over speed; the base after the T5 step |
| Narrow sample | same speed in mm/s, so 1.83 times in its own widths | no train burn; the published work disagrees on the sign of the width effect |
| Spread before a prefix | 25 % | three burns of one day agree within 5 %; other days and flows are outside them |

Flow speed comes from the day's fan calibration; below its lowest row it is the straight line to the
stall point, which is the model's assumption, not a calibration.

## What it is conditional on

The fan history up to each forecast time is an input. The forecast says what the flame does under a
fan schedule, not what the operator will do next. The operators step the fan in response to the
flame, so the schedule itself carries information about the burn; the model does not use it beyond
the flow speed.

## Limits

- **Units.** Sample widths. No run has a length calibration, so nothing here is in metres. The
  forward's metric route (a prefix with the conditions `x_o2` and `width` and the fan history as
  control stages) is implemented and tested on a synthetic fixture; with a width in metres it
  converts, without one it refuses.
- **Twelve burns, three sessions of one campaign.** The interval above is a bootstrap over burns.
- **Flows above 11 cm/s, 21 % oxygen and the narrow sample are outside train.** The validation burns
  hold all three; the result above includes them, and the author cannot say which carry the error.
- **The quench law is wrong** on spread-or-not (above). No ignition forecast should be read from D1.
- **Not predicted:** temperature, flame brightness, the base (a median is in the diagnostics, unscored).

## Measurement

Tracks (`wsp-bass-front-track/1`) hold the downstream tip and the upstream base of the blue flame in
the top-view stills, in sample widths, the fan history read from the video overlay, and the day's
fan calibration. The four train tracks are in `tools/wsp/bass_optical/tracks/train/`; how they were
measured is in [feedback/bass_train_measurement.md](../feedback/bass_train_measurement.md) and
[feedback/bass_train_tracks.md](../feedback/bass_train_tracks.md). The validation tracks are local
files outside git (`runs/dev/bass_front/val_sealed/`). The scoring rules were fixed in
[PROTOCOL.md](../PROTOCOL.md) section 9 before the validation score existed.

```bash
target/release/wsp-dev front-feedback --release sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c --release sha256:ce53fe1fb3e6300c5e818dd7c9c5ec8e8f4c2cfd19673449d3936de05527de46 --tracks tools/wsp/bass_optical/tracks/train --label train --out runs/dev/front_train.json
```

## Fifth generation: `bass_front.wsp2` P1 (not adopted)

Forward `front2.rs` (separate oxygen powers for speed and overhang, separate growth and shrink
times, a slow-down downstream, flow paths; all off by default, in which case it equals `front.rs`
exactly, by test). Three changes were typed and scored once on the train burns: a slower growth of
the flame (0.082 sample widths against 0.076), a weaker oxygen dependence with flow paths (0.085).
Both were worse and were dropped. What remained is P1 `sha256:d53e2944b62863014b875a8023b57b50ca28095f4d69c86e1a0c28fcf49d429b`: D1 with
the powers of the narrow sample corrected (`speed.pow = [1.5, -0.5]`, `over.pow = [3.0, 1.0]`) from
two burns of the earlier BASS campaign (PSI-26), which belongs to no partition; for the 2 cm sample
it is D1 number for number. On three burns of that campaign it scores 0.157 against 0.220.

Validation, one look on the sealed tracks (12 burns): tip CRPS 0.1137 against 0.1168 for D1 (paired
−0.0031 [−0.0096, +0.0032]); track ended 0.1976 against 0.1780; spread or not 0.1703 for both. The
rule written beforehand asked for a tip score not higher and a track-ended score not worse: **not
adopted**. D1 stays.

