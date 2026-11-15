# FLEX droplets, fifth generation: laws from mechanism (`flex_methanol.wsp5`, `flex_heptane.wsp5`)

**State: experimental, hand-written, data-informed. Not adopted: the carried models stay
(`flex_methanol.wsp2` D3, `flex_heptane.wsp2` D2).** Forward `droplet6.rs`: the fifth droplet forward
(flame cameras as evidence of the state, a heavy-tailed radiative clock) plus three laws, all off by
default, in which case it returns exactly what `droplet5.rs` returns (tested). Not NASA-trained; no
Bayesian posterior is claimed.

| Model | Candidate | Release | Non-zero / authored | Train CRPS, mean / median (µm) | Validation CRPS (µm) |
|---|---|---|---|---|---|
| `flex_methanol.wsp5` | P1: the laws below, cameras, heavy tail | `sha256:eafcb57cb78250628cf44a9b3b41887a4202f013e0fe65366a49dd46ee5d58e9` | 92 / 406 | 28.7 / 17.5 | 42.5 |
| | P0: the laws without cameras | `sha256:4d873aa696d22cea9d8c6bba816d5702c261402669d5b523ab8b14174219df8a` | 90 / 406 | 29.7 / 18.2 | not scored: it failed the train rule on the median |
| | carry of D3 (the carried numbers) | `sha256:a8e957efdb5df6c3cdb2732f31c02a6ee6de7d5711759e9202a35a8313aa91a5` | 77 / 406 | 29.9 / 17.7 | 39.5 |
| `flex_heptane.wsp5` | P0: the laws below | `sha256:9a108f1a7fbf6fe56161f5108bff1a66b824a0d0171ad70060a44cb5185fac0b` | 107 / 603 | 43.9 / 19.8 | 47.2 |
| | P1: P0, cameras, heavy tail | `sha256:57fa63b20d00e53c9e16186fec09845039b70659bd5d0ef7a3896907e02f78ce` | 109 / 603 | 44.6 / 21.0 | 48.4 |
| | carry of D2 (the carried numbers) | `sha256:b6bc5a9a502f8b8559d05ce03a8e62bb195c53ee159a735ae4ab3bc99fb20f2c` | 102 / 603 | 46.2 / 21.8 | 46.1 |

Validation: 65 methanol and 43 n-heptane cases over nine and seven days, on the private copy of the
data root (every release read the same prefixes, the carry included). Paired differences, carry
minus candidate: methanol P1 −3.1 [−12.1, +4.6] µm; heptane P1 −2.3 [−7.3, +2.4]. The rule written
beforehand (replace when the validation CRPS is not higher) refuses all three.

## The laws that were written

**Methanol.**

- *An unlit droplet evaporates on its stored heat.* When the flame leaves, the liquid is near
  boiling; the latent heat comes out of the liquid, which cools, and the rate falls as
  `K0 / (1 + t/t0)` with `t0 = T* c_l D² / (1.5 K0 L)`. Constants from property tables, not from
  train: `K0 = 0.30 mm²/s` (methanol in nitrogen at 330 K), `T* = 16 K`, `c_l = 2650 J/kg/K`,
  `L = 1.118 MJ/kg`. It replaces three numbers the carried model typed from train, at an equal train
  score.
- *The critical diameter stops growing above 18 % oxygen* (5.8 mm in air where the carried law
  extrapolates to 9 mm and physics says about 4.5) *and falls with pressure above 1 atm* (radiation
  grows with pressure; a 3.0 mm train droplet at 3 atm lost its flame early).
- *No pressure term in the water extinction*: the carried term was typed from that same run.

**n-Heptane.**

- *The cool-flame burning rate has no oxygen or pressure dependence below 1 atm* (0.345 mm²/s on
  train with no trend); above 1 atm a warmer flame burns faster, from one train run.
- *The cool flame ends where the oxygen partial pressure says*:
  `D_ce / D0 = 0.47 sqrt((0.18 − β) / (pO2 − β))`, β = 0.12 atm.
- *A flame born below its end diameter dies away* over 3.5 s, in place of the helium patch of the
  fourth generation; helium enters the radiative clock.

What was typed, tried once on train and dropped is listed in section 10 of
[PROTOCOL.md](../PROTOCOL.md) and in `dossier/08_in_progress.md`.

## What the refusal means

The candidates are better on train and 1 to 3 µm worse on validation, inside intervals of ±5 to
±10 µm. Validation cannot tell the two sets of laws apart; it shows no gain, and a model is not
replaced for no gain. What the rebuild did achieve is not a score: three train-read numbers of the
methanol model became property-table constants at an equal train score, and two patches of the
heptane model (the oxygen and pressure terms of the cool-flame rate, the helium after-burn law)
became laws with a stated mechanism. The 90 % band of the heptane camera candidate covered 0.88 on
validation; the band widths were not revised.

## Exposure

Data-informed. The author had read the train partition, the aggregate validation tables of every
earlier candidate and the conditions of the validation runs. No validation outcome was read per
run. The physics dossier and the property look-up were made by delegated agents
(claude-fable-5-2); the author typed every number.

## Reproduce

```bash
python tools/wsp/reassemble.py --only flex/wsp5
```

```bash
IGNIS_DATA=runs/dev/data_ext target/release/wsp-dev feedback --release sha256:eafcb57cb78250628cf44a9b3b41887a4202f013e0fe65366a49dd46ee5d58e9 --partition train --out runs/dev/gen5/card_fb_methanol_p1.json
```
