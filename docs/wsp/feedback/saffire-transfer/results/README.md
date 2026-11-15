# Saffire transfer test: result

One scored run, 15 November 2026, by `tools/wsp/saffire/score_transfer.py --execute` on the protocol
[../../../SAFFIRE_TRANSFER.md](../../../SAFFIRE_TRANSFER.md) (scorer frozen in
). The author read the aggregates below and nothing else; the track files were not opened.
No weight changed and nothing is run again. This look spends the three held Saffire flight groups.

## The answer

**The narrow-strip model transfers to the wide sheet, on two burns.** On both 40.6 cm Saffire sheets
burned with the flow (Saffire I S1 at 20 cm/s, Saffire III S1 at 30 cm/s), the adopted
`bass_front.wsp` D1, typed from three 2 cm BASS-II burns, forecasts the flame tip over the next
minute better than persistence: tip CRPS 0.058 sample widths against 0.250 over 8 cases, lower on
2 of 2 burns. By the sentence of section 7 of the protocol, that is "transfers". Two burns from two
flights say what the model did there, not how far it generalises.

## Wide sheets (the decision)

4 tracks; 2 scoreable; 2 opposed sheets refused by the reader at the flow condition (the model's
answer for opposed spread); 8 cases (cutoffs 5, 10, 20, 40 s on each burn).

| Release | Tip CRPS (sample widths) | Median abs. error | Coverage 50 % / 90 % | Minus persistence | Burns lower | Track-ended Brier |
|---|---|---|---|---|---|---|
| persistence | 0.250 | 0.386 | 0.26 / 0.58 | | | 0.035 |
| **D1 (adopted)** | **0.058** | 0.075 | 0.93 / 1.00 | −0.192 | 2 of 2 | 0.001 |
| joint Astra (`wsp3`) | 0.061 | 0.081 | 0.93 / 1.00 | −0.189 | 2 of 2 | 0.001 |
| `wsp4` operator | 0.061 | 0.081 | 0.93 / 1.00 | −0.189 | 2 of 2 | 0.001 |

- On BASS-II validation D1 scored 0.117 against persistence 0.234; on the sheets the gap is wider.
- **The bands are too wide, not too narrow:** 93 % of targets fall inside the 50 % band. The median is
  right and the stated uncertainty is generous at this width: the model is better than it says.
- Ignition: neither sheet failed to spread, and every release said so (Brier < 0.001). With no
  quenched sheet, the test says nothing about the quench boundary, the model's known weakness.
- The two Astra releases transfer by the same sentence; `wsp3` and `wsp4` are identical to the third
  decimal here and D1 is ahead by 0.003, inside any noise two burns can carry.

## Saffire II strips (5 cm; reported, decide nothing; tracks unreliable)

| Release | Tip CRPS | Coverage 50 % / 90 % | Minus persistence | Burns lower |
|---|---|---|---|---|
| persistence | 0.852 | 0.01 / 0.20 | | |
| D1 | 0.654 | 0.44 / 0.58 | −0.198 | 2 of 2 |
| joint Astra | 0.698 | 0.40 / 0.58 | −0.155 | 2 of 2 |
| `wsp4` | 0.697 | 0.40 / 0.58 | −0.155 | 2 of 2 |

The visual check accepted 6 of 31 strip images (the tip line short of a dim, diffuse glow), and the
two cameras disagree by 0.43 widths on one strip. These numbers are scores against a measured tip
that is probably wrong; they are not evidence about the model.

## Measurement, in short

Blind, by a script written and run by a measuring agent (claude-opus-5-5) from NASA's mapped JPEG
frames; checked by a second agent (claude-sonnet) on a fixed stride of overlays: sheets 30 of 30
accepted. Sheets: 402 and 334 flame stills, about 890 to 905 pixels per sample width, camera
agreement 0.024 and 0.007 widths. Camera 1 does not see the downstream part of either sheet; camera 2
fills it through the nominal length. The luminous region includes glowing char. The measurer changed
its frame handling once after a pass that included the sheets (the cutoff that classifies backlit
frames), with threshold and geometry unchanged and no knowledge of any forecast (dossier X-26).

## Files

`summary.json` (the wrapper's aggregate tables, release ids, protocol commit, track hashes),
`wide.json` and `strip.json` (scorer aggregates, `--quiet-cases`), `*.log` (scorer output). The chain
check reproduced the committed matched-window TRAIN aggregate to 1e-12 before Saffire was scored.
