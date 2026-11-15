# BASS — what can be measured, and what a model would have to say

## The question

The shared contracts give every BASS front quantity in metres, and no BASS run has a calibration.
Is there a measurable target in the train data, in whatever units it honestly has?

## What was measured (train tests T5, T6, T16, T10 only)

**Duct camera** (720×480 video, edge-on to the sample). Frames are registered by the position of the
overlay text (the analog capture shifts the whole picture; T16 is shifted 0.18 of the frame width),
averaged over 1 s, and each pixel is thresholded against its own noise (median and MAD over time,
z > 4). The flame is the largest luminous region on the sample plane. Brightness beats the blue
channel. Positions are fractions of the frame width.

**Top camera** (the D300s stills, available only as the 324×216 thumbnails of NASA's proof-sheet
PDFs, one per 1.12 s). The leading edge of the blue flame is found the same way. The lit sample band
is 56 px wide in all four tests, which gives positions in *sample widths* without any length
calibration.

**Overlay.** `FAN`, `AIR`, `RAD` and the clock are read from every frame with a glyph matcher
(2 errors in 112 hand-checked frames) and converted with the day's fan calibration where the table
covers the setting.

| Test | O2 | Fan setting (converted) | Tracked | End |
|---|---|---|---|---|
| T5 | 18.8 % | 61 (10.9 cm/s) → 42 at 64.6–71.1 s | 56.5–148.6 s | extinction (confirmed in the top view) |
| T6 | 18.8 % | 47 (5.2 cm/s) → 42 at 18–21 s | 12.5–124.6 s | lost in noise near the end of the sample |
| T16 | 17.6 % | 46 → 43 at 67.6–69.6 s | 57–158 s | leaves the captured picture |
| T10 | 16.4 % | 47–48 (6.0–6.4 cm/s), no step | — | never spread: char only at the igniter end |

Fan settings of 46 and below on GMT 100 lie under the lowest usable row of that day's calibration
and are **not converted**. The `AIR` display lags a fan step by more than 8 s, so it cannot time one.

**Leading-edge speed** (top view; fraction of the still width per second, and sample widths per second):

| Test | Before the step | After the step |
|---|---|---|
| T5 | 0.0153 ± 0.0016 (0.087 w/s) at fan 61 | 0.00623 ± 0.00004 (0.0356 w/s) at fan 42 |
| T6 | 0.0092 ± 0.0010 (0.054 w/s) at fan 47, seven stills after the igniter | 0.00624 ± 0.00003 (0.0364 w/s) at fan 42 |
| T16 | 0.00757 ± 0.00026 (0.0437 w/s) at fan 46 | 0.00635 ± 0.00002 (0.0366 w/s) at fan 43 |

The duct camera gives the same numbers within its scatter (0.0058–0.0062 after the step).

## What the measurements say

- **The front follows the flow, at once.** At each fan step the flame dims and shortens within the
  0.5 s resolution of the video and settles in 2–8 s, longer for the larger step. It shortens from
  its downstream tip. In T5 the speed falls 2.5-fold with the flow.
- **At the lowest flow the three flames spread at one rate**: 0.036 sample widths per second, within
  2 %, at 18.8 % and 17.6 % oxygen alike. Area and brightness keep decaying slowly through that
  phase (T5 goes out; the other two leave the field).
- **16.4 % oxygen at about 6 cm/s does not spread at all** (T10).
- **Direction.** The igniter and the upstream end are on the right in both cameras and the flame
  moves away from them: concurrent spread.

## What a legitimate target is

| | |
|---|---|
| Quantity | position of the leading (downstream) edge of the blue flame against time |
| Units | sample widths from the downstream end of the window, or fraction of the still width — declared image-plane units, not metres |
| Source | top-view stills at 1.12 s; the duct view as a second, noisier measure (scatter 0.01 against under 0.003) |
| Known controls | the fan setting against time at 0.5 s, the step times, the flow speed where the calibration covers the setting, initial oxygen, the igniter interval |
| Runs | three tracks of about 100 s and one no-spread outcome in train |

A forecast must take the fan history as a known control, and with these four runs it can be scored
on two things only: the response to a step, and spread against no spread. A rate-against-flow law
cannot be written from them: the low settings are uncalibrated and there are three points before
the steps, two of them inside the ignition transient.

## What was not done, on purpose

- No pixel-to-metre conversion. The matrix says "2 cm SIBAL", the dossier 1.2 or 2.2 cm, and whether
  the lit band is the fabric or the window opening is not documented. Inventing a scale would be a
  fabricated calibration.
- No new quantity in the shared contracts and nothing written into the shared records: both belong
  to the data lane.
- No model on three tracks whose only common regime is one flow setting.
- No test outside the train partition was opened. One search of the experiment table printed three
  rows of other tests by accident; they were not used.

What the data lane would have to provide is listed in
[EXPERIMENTAL_SCIENCE.md](../EXPERIMENTAL_SCIENCE.md), section 5.3.
