# BASS-II train tests T5, T6, T16, T10: what the video record can measure

> Train partition only; nothing fitted. Written by a delegated analysis agent (claude-opus-5-5) on 14 November 2026;
> the author rechecked the claims it acted on against the train tables. Paths under `scratchpad` are the agent's
> working folder and are not part of the repository.

## Summary (≤400 words)

- **Top view:** for these four tests the only usable top-view images are the 324 × 216 px proof-sheet thumbnails. There are no original stills. `T5_Still_Sequence.mp4` is just the camera's playback screen. `T10 original videos.zip` holds the two T10 video channels as `.ts` files, not stills.
- **Flame fronts in the top view:** the front can be measured in T5, T6 and T16, at about 1.12 s per still. T10 never spread: the post-test still shows char only at the igniter end.
  - **Scale:** the lit sample band measures 55.6–56.7 px (about 0.17 of the frame width) in all four tests. It gives positions in sample widths. Millimetres would need the data team to confirm the window width (2.0 or 2.2 cm).
- **Duct view:** averaging over 1 s and thresholding each pixel against its own noise recovers whole burns (the first pass found 12 s in T5 only).
  - T5: tracked 56.5–148.6 s, ending in extinction.
  - T6: tracked 12.5–124.6 s.
  - T16: tracked 57–158 s, until the flame leaves the captured picture.
  - T10: nothing after the igniter.
  - Brightness (R+G+B) beats the blue channel alone.
  - T16's whole picture is shifted 130 px to the left; positions are corrected using the overlay's position.
- **Direction:** the right side of the image is upstream in both cameras. The igniter is at the right, the flame tail points left, and the front moves right to left, which is consistent with concurrent flow.
- **Overlay:** read with a glyph matcher and checked by eye on 112 frames; 2 errors were found.
  - Fan steps: T5 FAN 61→42 at 64.6–71.1 s; T6 47→42 at 18–21 s; T16 46→43 at 67.6–69.6 s. T10 has no step during the burn.
  - Calibration (2-restrictor block): GMT 100 uses columns W–Y, rows 26–32; GMT 175 uses AE–AG, rows 26–33; speed U = 1.0066 × AIR.
  - Converted: T5 at 61 is 10.9 cm/s, T6 at 47 is 5.2 cm/s, T10 at 47–48 is 6.0–6.4 cm/s.
  - FAN 46, 43 and 42 on GMT 100 lie below the lowest usable row (46.5), so they are not converted.
- **At each fan step:** the flame dims and shortens within the 0.5 s resolution of the duct view. In T5 brightness halves in about 1.5 s and the flame settles in 7–8 s. The flame shortens from its downstream tip, which briefly moves back.
  - T5: front speed falls from 0.0153 to 0.0062 of the frame width per second.
  - T16: 0.0076 to 0.0064.
  - After the step, all three tests spread at 0.0062–0.0064 of the frame width per second, about 0.036 sample widths/s.
  - The AIR reading lags by more than 8 s, so it cannot time the steps.
- **Downloads:** 4 files, 39.6 MB.
- **Rules:** one grep of the experimental-table file accidentally printed three rows of other tests (B8, B11, B19). I did not use them.

---

## Report

# BASS-II train tests T5, T6, T16, T10: what the video record can measure

This is a measurement study; no model was built. Only these four tests were opened: T5 (sample 131, GMT 100), T6 (132, GMT 100), T16 (139, GMT 100) and T10 (134M, GMT 175).

## Units and conventions

**Duct video**
- 720 × 480 px, 29.97 fps; x is a fraction of the frame width, y a fraction of the frame height.
- The analog capture shifts the whole picture sideways, and the overlay text shifts with it. I checked this in T16: across the shift, the scene moved −120 px and the overlay moved −120 to −121 px.
- Positions are therefore registered to the overlay, in "ref" coordinates where the F of the FUEL label sits at 166 px (as in T5 and T6).
- Conversion: x_ref = x_recorded + (166 − offset)/720.
- Offsets during the burns: T5 and T6 165–169 px; T10 159 px; T16 35–36 px, so the whole T16 burn is shifted 0.18 of the frame width to the left.

**Top view**
- The images are the D300s stills as embedded in NASA's proof-sheet PDFs, rendered at 300 dpi and resampled to their native 324 × 216 px.
- The originals were 4288 × 2848 px (per the camera's playback screen), so one thumbnail pixel is about 13.2 original pixels.

**Clocks**
- The overlay clock was fitted to video time over all readings:

| test | clock at t = 0 | rms of fit |
|---|---|---|
| T5 | 14:26:55.0 | 0.32 s |
| T6 | 14:57:26.3 | 0.26 s |
| T16 | 18:24:01.8 | 0.25 s |
| T10 | 14:51:06.7 | 0.25 s |

- The still stamps are the camera's own whole seconds. At igniter onset they read 0–2.3 s earlier than the overlay clock: T5 14:27:44 vs 14:27:45.0; T6 14:57:30 vs 14:57:31.0; T10 14:51:20 vs 14:51:21.1; T16 18:24:46 vs 18:24:48.2.

No lengths are given in millimetres anywhere (see section 4).

## 1. Timeline of each duct video

Read by eye from frame contact sheets, and cross-checked against brightness and overlay statistics. Full list in `timeline.json`.

**T5** (255.3 s)

| t (s) | clock | event |
|---|---|---|
| 0–9.7 | from 14:26:55 | dark |
| 9.74–32.57 | to 14:27:27.6 | lit: holder underside and sample edge line at y ≈ 0.63; no hand or object |
| 32.6–50.0 | | dark (null period) |
| 50.0 | 14:27:45.0 | igniter glows (x ≈ 0.92) |
| 51.0 | | flame visible: pink chevron opening to the left |
| ≈55.5 | | igniter glow no longer distinguishable |
| 64.6–71.1 | 14:27:59.6–14:28:06.1 | FAN 61→42 |
| 115.6–123.1 | | FAN readings shift 42/43 → 41/42 (sub-unit change) |
| 148.6 | 14:29:23.6 | last flame detection (x = 0.23); top view shows the flame gone by 14:29:31 |
| 172–178 | | FAN →122 |
| 217.1–248.6 | | Nikon D300s playback screen (images 95–110/110) |
| 248.6–255.3 | | lit |

**T6** (166.3 s)

| t (s) | clock | event |
|---|---|---|
| 0–4.7 | | dark (only 4.5 s of null data) |
| 4.7 | 14:57:31.0 | igniter glows |
| ≈8.5 | | flame visible |
| ≈11.5 | | igniter glow gone |
| 18.0–21.0 | 14:57:44.3–47.3 | FAN 47→42 |
| 55.1–55.6 | | sub-unit change (last 43, then 41s) |
| 124.6 | 14:59:30.7 | last detection (x = 0.21); the stills had already stopped at 14:59:20 with the flame still burning |
| 129–146 | | FAN →122 |
| 154.7–166.3 | | lit; sample edge line bent after the burn |

**T16** (276.6 s)

| t (s) | clock | event |
|---|---|---|
| 0–14.0 | | lit |
| 14.0–15.2 | | signal glitch, then the whole picture stays shifted left about 120–131 px with coloured edge bands |
| 15.2–37.6 | | lit |
| 37.7–46.5 | | dark (null period) |
| 46.5 | 18:24:48.2 | igniter glows; colours flicker frame to frame |
| ≈47.2 | | flame visible |
| ≈55.8 | | igniter glow gone |
| 67.6–69.6 | 18:25:09.3–11.3 | FAN 46→43 |
| 158.1 | 18:26:39.7 | last detection, at recorded x = 0.026: the flame leaves the captured picture, it is not seen to go out; stills stop at 18:26:35 with the flame 0.18 sample widths from the window end |
| 181–189 | | FAN →122 |
| 191.5–202.5 | | lit |
| 202.5–276.6 | | camera playback (images 113–157/157) |

**T10** (from the `.ts` file: 3063 frames decoded, 3103 nominal, 102.2 s)

| t (s) | clock | event |
|---|---|---|
| 0–0.67 | | end of a lit period |
| 0.7–14.4 | | dark (null period) |
| 14.4 | 14:51:21.1 | igniter glows |
| 14.95–22.0 | | igniter fully on, lighting the whole section; chevron-shaped glow at x 0.74–0.97 |
| 22.0–23.5 | | glowing remnant fades |
| 23.5–76 | | nothing above noise |
| 76.1–82.1 | | FAN 47→122 |
| 76.5–102 | | faint grey diffuse patch above the sample plane (x 0.57–0.79, y 0.33–0.57); not flame-coloured, cause unknown, excluded |

No crew hand and no sample change appear in any of the four duct videos.

## 2. Flow history from the overlay

**How it was read**
- The overlay is a fixed character grid (pitch 28.67 px). Each frame's grid was located by correlating the label row against a reference.
- Each cell was matched against 20 glyph templates. The templates came from clustering 6000 cells and labelling the leaders by eye.
- Readings every 0.5 s are in `overlay_raw_T*.json`.

**Check by eye:** 112 frames (28 per test) compared with enlarged crops. Two errors:
- T16 at 18.0 s (a lit frame): AIR read "1 5". Fixed by keeping the right-aligned digits.
- T10 at 20.5 s: FAN read 46 where the display shows 48.

The tables use 2-s medians, which removes single misreads.

**RAD:** in T5, T6 and T10 the last digit is cut by the frame edge (11 of 24 px visible), so it is uncertain by ±0.02. In T16 RAD is fully visible and was verified. Radiometer gain is 50 for T5, T6 and T16 and 250 for T10, so T10's RAD is not comparable with the others.

**Calibration** (`PSI-25_Reference Data_Fan calibration data.xlsx`, sheet "Fan Calibration data", 2-restrictor block; all four tests used restrictor 2):
- **GMT 2014-100** (columns W/X/Y, rows 26–32): FAN 122→AIR 37, 105→30, 88.5→22, 72→15, 55→8.5, 46.5→5, 38→0.
- **GMT 2014-175** (columns AE/AF/AG, rows 26–33): 122→38, 105→31, 88.5→23, 72→16, 55→9, 46.5→5.75, 42→3.75, 38→0.
- **Speed:** U [cm/s] = 1.0066 × AIR (cell A33).
- **Interpolation:** linear in FAN between rows. The sheet advises against the 38 V×10 zero-flow row, so on GMT 100 nothing below the 46.5 row is converted.

| test | FAN before step | U | after step |
|---|---|---|---|
| T5 | 61 | 10.87 cm/s (overlay AIR read 10–11) | 42 then 41/42: not convertible |
| T6 | 47 | 5.24 cm/s (AIR read 5) | 42: not convertible |
| T16 | 46 | not convertible (0.5 below the 46.5 row; U at 46.5 would be 5.03) | 43: not convertible |
| T10 | 47–48 | 5.98–6.37 cm/s | no step until the post-burn ramp |

For context only: other days do have a FAN 42 row (GMT 174 gives 42→3, GMT 175 gives 42→3.75). They are not applied here.

**Comparison with the test matrix**
- **T5** pot "2.72 / 0.50 / 0.42" matches FAN 61 → 42 → a sub-unit shift to 41/42 at 115.6–123.1 s.
- **T6** air "5 2 1 <1" and **T16** air "5 3" match the AIR display slowly decaying after a single fan step:
  - T6 AIR: 5, then 4 from ~34 s, 3 from ~44 s, 2 from ~66–74 s, 1 from ~84 s, 0 from ~92 s.
  - T16 AIR: 4 from ~88 s, 3 from ~114 s.
  - This is the transducer's roughly 1-minute response, not further fan steps [inference].
- In T5 and T6, AIR falls to 0 while FAN stays at 41–42. The flow may have been under about 1 cm/s, or the fan may have been near its cut-off voltage; the record cannot separate these from the sensor lag.

**Tables:** 

| t (s) | overlay clock | FAN | AIR | RAD (last digit partly cut) | U from FAN (cm/s) |
|---|---|---|---|---|---|
| 48 | 14:27:44 | 61 | 10 | 0.11 | 10.9 |
| 50 | 14:27:46 | 61 | 10 | 0.16 | 10.9 |
| 52 | 14:27:48 | 61 | 10 | 0.33 | 10.9 |
| 54 | 14:27:50 | 61 | 10 | 0.36 | 10.9 |
| 56 | 14:27:52 | 61 | 10 | 0.29 | 10.9 |
| 58 | 14:27:54 | 61 | 10 | 0.31 | 10.9 |
| 60 | 14:27:56 | 61 | 10 | 0.35 | 10.9 |
| 62 | 14:27:58 | 61 | 11 | 0.40 | 10.9 |
| 64 | 14:28:00 | 60 (57/59/61) | 11 | 0.44 | 10.5 |
| 66 | 14:28:02 | 51.5 (49/50/53/55) | 11 | 0.46 | 7.1 |
| 68 | 14:28:04 | 46.5 (44/46/47/48) | 10 | 0.41 | 5.0 |
| 70 | 14:28:06 | 42.5 (42/43/44) | 10 | 0.38 | n/a |
| 72 | 14:28:08 | 42 | 10 | 0.34 | n/a |
| 74 | 14:28:10 | 42 | 9 | 0.32 | n/a |
| 76 | 14:28:12 | 42 | 8 | 0.32 | n/a |
| 78 | 14:28:14 | 42 (41/42) | 8 | 0.31 | n/a |
| 80 | 14:28:16 | 42 (42/43) | 6 | 0.31 | n/a |
| 82 | 14:28:18 | 42 (41/42/43) | 6 | 0.31 | n/a |
| 84 | 14:28:20 | 42 | 6 | 0.32 | n/a |
| 86 | 14:28:22 | 42 | 5 | 0.32 | n/a |
| 88 | 14:28:24 | 42 | 5 | 0.33 | n/a |
| 90 | 14:28:26 | 42 | 5 | 0.34 | n/a |
| 92 | 14:28:28 | 42 | 4 | 0.35 | n/a |
| 94 | 14:28:30 | 42 | 4 | 0.36 | n/a |
| 96 | 14:28:32 | 42 (42/43) | 4 | 0.37 | n/a |
| 98 | 14:28:34 | 42 (42/43) | 4 | 0.39 | n/a |
| 100 | 14:28:36 | 42 (42/43) | 4 | 0.40 | n/a |
| 102 | 14:28:38 | 42 | 3 | 0.41 | n/a |
| 104 | 14:28:40 | 42 | 3 | 0.43 | n/a |
| 106 | 14:28:42 | 42 | 3 | 0.45 | n/a |
| 108 | 14:28:44 | 42 | 3 | 0.46 | n/a |
| 110 | 14:28:46 | 42 | 3 | 0.48 | n/a |
| 112 | 14:28:48 | 42 | 3 | 0.50 | n/a |
| 114 | 14:28:50 | 42 (42/43) | 3 | 0.52 | n/a |
| 116 | 14:28:52 | 42 | 3 | 0.53 | n/a |
| 118 | 14:28:54 | 42 | 3 | 0.55 | n/a |
| 120 | 14:28:56 | 42 | 3 | 0.56 | n/a |
| 122 | 14:28:58 | 42 (41/42) | 3 | 0.56 | n/a |
| 124 | 14:29:00 | 42 (41/42) | 3 | 0.55 | n/a |
| 126 | 14:29:02 | 42 | 3 | 0.55 | n/a |
| 128 | 14:29:04 | 42 (41/42) | 2 | 0.54 | n/a |
| 130 | 14:29:06 | 42 | 2 | 0.54 | n/a |
| 132 | 14:29:08 | 42 (41/42) | 2 | 0.52 | n/a |
| 134 | 14:29:10 | 41.5 (41/42) | 2 | 0.51 | n/a |
| 136 | 14:29:12 | 42 | 2 | 0.49 | n/a |
| 138 | 14:29:14 | 42 | 2 | 0.47 | n/a |
| 140 | 14:29:16 | 42 (41/42) | 2 | 0.43 | n/a |
| 142 | 14:29:18 | 42 | 2 | 0.40 | n/a |
| 144 | 14:29:20 | 42 (41/42) | 1 | 0.35 | n/a |
| 146 | 14:29:22 | 42 (41/42) | 2 | 0.30 | n/a |
| 148 | 14:29:24 | 41.5 (41/42) | 1 | 0.25 | n/a |
| 150 | 14:29:26 | 42 | 1 | 0.18 | n/a |
| 152 | 14:29:28 | 42 | 0 | 0.15 | n/a |
| 154 | 14:29:30 | 41.5 (41/42) | 0 | 0.11 | n/a |
| 156 | 14:29:32 | 42 | 0 | 0.10 | n/a |
| 158 | 14:29:34 | 42 | 0 | 0.09 | n/a |

 

| t (s) | overlay clock | FAN | AIR | RAD (last digit partly cut) | U from FAN (cm/s) |
|---|---|---|---|---|---|
| 2 | 14:57:29 | 47 | 5 | 0.06 | 5.2 |
| 4 | 14:57:31 | 47 | 5 | 0.11 | 5.2 |
| 6 | 14:57:33 | 47 | 5 | 0.15 | 5.2 |
| 8 | 14:57:35 | 47 | 5 | 0.26 | 5.2 |
| 10 | 14:57:37 | 47 | 5 | 0.24 | 5.2 |
| 12 | 14:57:39 | 47 | 5 | 0.15 | 5.2 |
| 14 | 14:57:41 | 47 | 5 | 0.17 | 5.2 |
| 16 | 14:57:43 | 47 (46/47) | 5 | 0.20 | 5.2 |
| 18 | 14:57:45 | 45.5 (44/45/46/47) | 5 | 0.21 | n/a |
| 20 | 14:57:47 | 42.5 (42/43) | 5 | 0.21 | n/a |
| 22 | 14:57:49 | 42 | 5 | 0.20 | n/a |
| 24 | 14:57:51 | 42 | 5 | 0.20 | n/a |
| 26 | 14:57:53 | 42 (42/43) | 5 | 0.21 | n/a |
| 28 | 14:57:55 | 42 | 5 | 0.21 | n/a |
| 30 | 14:57:57 | 42 | 5 | 0.21 | n/a |
| 32 | 14:57:59 | 42 | 4 | 0.22 | n/a |
| 34 | 14:58:01 | 42 | 4 | 0.22 | n/a |
| 36 | 14:58:03 | 42 (42/43) | 4 | 0.23 | n/a |
| 38 | 14:58:05 | 42 | 4 | 0.23 | n/a |
| 40 | 14:58:07 | 42 | 4 | 0.24 | n/a |
| 42 | 14:58:09 | 42 (42/43) | 3 | 0.25 | n/a |
| 44 | 14:58:11 | 42 (42/43) | 3 | 0.26 | n/a |
| 46 | 14:58:13 | 42 | 3 | 0.26 | n/a |
| 48 | 14:58:15 | 42 (42/43) | 3 | 0.27 | n/a |
| 50 | 14:58:17 | 42 (42/43) | 3 | 0.28 | n/a |
| 52 | 14:58:19 | 42 | 3 | 0.29 | n/a |
| 54 | 14:58:21 | 42.5 (41/42/43) | 3 | 0.29 | n/a |
| 56 | 14:58:23 | 42 | 3 | 0.30 | n/a |
| 58 | 14:58:25 | 42 | 3 | 0.32 | n/a |
| 60 | 14:58:27 | 42 | 3 | 0.32 | n/a |
| 62 | 14:58:29 | 42 | 3 | 0.33 | n/a |
| 64 | 14:58:31 | 41.5 (41/42) | 3 | 0.33 | n/a |
| 66 | 14:58:33 | 42 (41/42) | 2 | 0.33 | n/a |
| 68 | 14:58:35 | 42 (41/42) | 3 | 0.34 | n/a |
| 70 | 14:58:37 | 42 | 2 | 0.34 | n/a |
| 72 | 14:58:39 | 42 | 2 | 0.35 | n/a |
| 74 | 14:58:41 | 42 | 2 | 0.36 | n/a |
| 76 | 14:58:43 | 42 (41/42) | 2 | 0.38 | n/a |
| 78 | 14:58:45 | 42 (41/42) | 2 | 0.39 | n/a |
| 80 | 14:58:47 | 42 | 2 | 0.40 | n/a |
| 82 | 14:58:49 | 42 | 2 | 0.41 | n/a |
| 84 | 14:58:51 | 42 (41/42) | 2 | 0.43 | n/a |
| 86 | 14:58:53 | 42 | 1 | 0.44 | n/a |
| 88 | 14:58:55 | 42 (41/42) | 1 | 0.45 | n/a |
| 90 | 14:58:57 | 42 | 1 | 0.46 | n/a |
| 92 | 14:58:59 | 42 | 0 | 0.47 | n/a |
| 94 | 14:59:01 | 41.5 (41/42) | 0 | 0.49 | n/a |
| 96 | 14:59:03 | 42 | 0 | 0.50 | n/a |
| 98 | 14:59:05 | 41.5 (41/42) | 0 | 0.51 | n/a |
| 100 | 14:59:07 | 42 | 0 | 0.50 | n/a |
| 102 | 14:59:09 | 41 (41/42) | 0 | 0.48 | n/a |
| 104 | 14:59:11 | 41.5 (41/42) | 0 | 0.46 | n/a |
| 106 | 14:59:13 | 42 (41/42) | 0 | 0.46 | n/a |
| 108 | 14:59:15 | 42 (41/42) | 0 | 0.42 | n/a |
| 110 | 14:59:17 | 41 (41/42) | 0 | 0.40 | n/a |
| 112 | 14:59:19 | 42 (41/42) | 0 | 0.36 | n/a |
| 114 | 14:59:21 | 41 (41/42) | 0 | 0.33 | n/a |
| 116 | 14:59:23 | 42 (41/42) | 0 | 0.29 | n/a |
| 118 | 14:59:25 | 42 (41/42) | 0 | 0.23 | n/a |
| 120 | 14:59:27 | 41 (41/42) | 0 | 0.18 | n/a |
| 122 | 14:59:29 | 41.5 (41/42) | 0 | 0.13 | n/a |
| 124 | 14:59:31 | 42 (41/42) | 0 | 0.10 | n/a |
| 126 | 14:59:33 | 42 (41/42) | 0 | 0.09 | n/a |
| 128 | 14:59:35 | 42 (41/42) | 0 | 0.09 | n/a |
| 130 | 14:59:37 | 41 (41/42) | 0 | 0.08 | n/a |

 

| t (s) | overlay clock | FAN | AIR | RAD | U from FAN (cm/s) |
|---|---|---|---|---|---|
| 44 | 18:24:46 | 46 | 5 | 0.11 | n/a |
| 46 | 18:24:48 | 46 | 5 | 0.16 | n/a |
| 48 | 18:24:50 | 46 | 5 | 0.27 | n/a |
| 50 | 18:24:52 | 46 | 5 | 0.32 | n/a |
| 52 | 18:24:54 | 46 | 5 | 0.34 | n/a |
| 54 | 18:24:56 | 46 | 5 | 0.38 | n/a |
| 56 | 18:24:58 | 46 | 5 | 0.27 | n/a |
| 58 | 18:25:00 | 46 | 5 | 0.21 | n/a |
| 60 | 18:25:02 | 46 | 5 | 0.21 | n/a |
| 62 | 18:25:04 | 46 | 5 | 0.22 | n/a |
| 64 | 18:25:06 | 46 | 5 | 0.22 | n/a |
| 66 | 18:25:08 | 46 | 5 | 0.23 | n/a |
| 68 | 18:25:10 | 44.5 (43/44/45) | 5 | 0.23 | n/a |
| 70 | 18:25:12 | 43 | 5 | 0.22 | n/a |
| 72 | 18:25:14 | 43 | 5 | 0.22 | n/a |
| 74 | 18:25:16 | 43 | 5 | 0.23 | n/a |
| 76 | 18:25:18 | 43 | 5 | 0.23 | n/a |
| 78 | 18:25:20 | 43 | 5 | 0.23 | n/a |
| 80 | 18:25:22 | 43 | 4 | 0.23 | n/a |
| 82 | 18:25:24 | 43 | 4 | 0.24 | n/a |
| 84 | 18:25:26 | 43 | 4 | 0.24 | n/a |
| 86 | 18:25:28 | 43 | 4 | 0.24 | n/a |
| 88 | 18:25:30 | 43 | 4 | 0.25 | n/a |
| 90 | 18:25:32 | 43 | 4 | 0.25 | n/a |
| 92 | 18:25:34 | 43 | 4 | 0.26 | n/a |
| 94 | 18:25:36 | 43 (42/43) | 4 | 0.27 | n/a |
| 96 | 18:25:38 | 43 | 4 | 0.27 | n/a |
| 98 | 18:25:40 | 43 (43/44) | 4 | 0.28 | n/a |
| 100 | 18:25:42 | 43 | 4 | 0.29 | n/a |
| 102 | 18:25:44 | 43 | 4 | 0.29 | n/a |
| 104 | 18:25:46 | 43 | 4 | 0.30 | n/a |
| 106 | 18:25:48 | 43 | 4 | 0.32 | n/a |
| 108 | 18:25:50 | 43 | 4 | 0.33 | n/a |
| 110 | 18:25:52 | 43 | 4 | 0.33 | n/a |
| 112 | 18:25:54 | 43 | 4 | 0.34 | n/a |
| 114 | 18:25:56 | 43 | 4 | 0.35 | n/a |
| 116 | 18:25:58 | 43 | 3 | 0.36 | n/a |
| 118 | 18:26:00 | 43 | 3 | 0.37 | n/a |
| 120 | 18:26:02 | 43 | 4 | 0.39 | n/a |
| 122 | 18:26:04 | 43 | 3 | 0.41 | n/a |
| 124 | 18:26:06 | 43 | 3 | 0.41 | n/a |
| 126 | 18:26:08 | 43 | 3 | 0.42 | n/a |
| 128 | 18:26:10 | 43 | 3 | 0.44 | n/a |
| 130 | 18:26:12 | 43 | 3 | 0.45 | n/a |
| 132 | 18:26:14 | 43 | 3 | 0.47 | n/a |
| 134 | 18:26:16 | 43 | 3 | 0.48 | n/a |
| 136 | 18:26:18 | 43 | 3 | 0.48 | n/a |
| 138 | 18:26:20 | 43 | 3 | 0.48 | n/a |
| 140 | 18:26:22 | 43 | 3 | 0.48 | n/a |
| 142 | 18:26:24 | 43 | 3 | 0.47 | n/a |
| 144 | 18:26:26 | 43 | 4 | 0.47 | n/a |
| 146 | 18:26:28 | 43 (42/43) | 3 | 0.45 | n/a |
| 148 | 18:26:30 | 43 | 3 | 0.43 | n/a |
| 150 | 18:26:32 | 43 | 3 | 0.40 | n/a |
| 152 | 18:26:34 | 43 | 3 | 0.35 | n/a |
| 154 | 18:26:36 | 43 | 3 | 0.32 | n/a |
| 156 | 18:26:38 | 43 | 3 | 0.27 | n/a |
| 158 | 18:26:40 | 43 | 3 | 0.23 | n/a |
| 160 | 18:26:42 | 43 | 3 | 0.17 | n/a |

 

| t (s) | overlay clock | FAN | AIR | RAD (last digit partly cut) | U from FAN (cm/s) |
|---|---|---|---|---|---|
| 12 | 14:51:19 | 48 (47/48) | 6 | 0.12 | 6.4 |
| 14 | 14:51:21 | 48 | 6 | 0.28 | 6.4 |
| 16 | 14:51:23 | 48 | 6 | 0.77 | 6.4 |
| 18 | 14:51:25 | 48 | 6 | 0.94 | 6.4 |
| 20 | 14:51:27 | 48 (46/48) | 6 | 0.98 | 6.4 |
| 22 | 14:51:29 | 47.5 (47/48) | 5 | 0.64 | 6.2 |
| 24 | 14:51:31 | 47 | 5 | 0.40 | 6.0 |
| 26 | 14:51:33 | 47 (47/48) | 6 | 0.24 | 6.0 |
| 28 | 14:51:35 | 47 | 6 | 0.16 | 6.0 |
| 30 | 14:51:37 | 47 | 6 | 0.15 | 6.0 |
| 32 | 14:51:39 | 47 | 6 | 0.13 | 6.0 |
| 34 | 14:51:41 | 47 | 6 | 0.13 | 6.0 |
| 36 | 14:51:43 | 47 (47/48) | 6 | 0.12 | 6.0 |
| 38 | 14:51:45 | 47 | 6 | 0.11 | 6.0 |



## 3. Duct-view flame measurement

**Method**
- Use all unlit frames that have the overlay. Bin 2 × 2 and register each frame by its overlay offset.
- Mask the overlay rows, the 10 px left edge, the encoder band beyond x = 686 px, and anything outside the captured picture.
- Average over 0.5 s and 1 s windows.
- For each pixel, take the median over time as background and 1.4826 × MAD as its noise (floor 0.25). A pixel counts as luminous at z > 4. Dilate by 9 binned pixels so the two flame sheets join; the flame is the largest region.
- **Detection threshold:** summed z must exceed 1.5 × the maximum found in pre-ignition dark windows, with area ≥ 40 px.

| test | null windows (1 s) | null max | threshold |
|---|---|---|---|
| T5 | 16 | 58.5 | 87.8 |
| T6 | 4 (weak null) | 112.3 | 168.5 |
| T16 | 7 | 46.4 | 69.6 |
| T10 | 12 | 21.7 | 32.6 |

- A track point must also lie on the sample plane (0.45 ≤ y_top ≤ 0.75 and y_bottom ≥ 0.55), come after the igniter glow ends, and come before the next lit or playback period.
- **Channel comparison** (1-s windows tracked): brightness (R+G+B) 93 / 109 / 102 vs blue-only 65 / 97 / 68 for T5 / T6 / T16. The colour signal in this recording is low-resolution, and in T16 it flickers.

**Where the flame is detected and where it is lost**

| test | tracked | 1-s windows detected | end |
|---|---|---|---|
| T5 | 56.5–148.6 s | 93 / 93 | extinction (confirmed in top view) |
| T6 | 12.5–124.6 s | 109 / 113 (one gap 118.6–122.6 s) | lost in noise at x = 0.21 |
| T16 | 57.0–158.1 s | 102 / 102 | leaves the captured picture |
| T10 | none | — | — (one window at 102.1 s on the grey patch, rejected) |

**Shape:** two sheets, at y ≈ 0.55–0.62 and y ≈ 0.64–0.72, either side of the sample edge (y ≈ 0.63). They meet at the right and open to the left, a "<" shape. The fitted box sometimes holds only one sheet, which leaves a residual scatter of about 0.01 in x_left.

**Tracks:** 

| t (s) | overlay clock | x left (ref) | x right (ref) | x left (as recorded) | y top | y bottom | area (px) | signal |
|---|---|---|---|---|---|---|---|---|
| 56.5 | 14:27:51.56 | 0.764 | 0.936 | 0.764 | 0.512 | 0.700 | 5240 | 140841 |
| 60.5 | 14:27:55.56 | 0.700 | 0.883 | 0.700 | 0.512 | 0.713 | 5052 | 95479 |
| 64.5 | 14:27:59.57 | 0.636 | 0.836 | 0.636 | 0.517 | 0.717 | 5404 | 88501 |
| 68.6 | 14:28:03.57 | 0.656 | 0.831 | 0.656 | 0.525 | 0.704 | 2424 | 17370 |
| 72.6 | 14:28:07.58 | 0.661 | 0.803 | 0.661 | 0.537 | 0.696 | 1060 | 4238 |
| 76.6 | 14:28:11.58 | 0.631 | 0.733 | 0.631 | 0.537 | 0.692 | 836 | 2715 |
| 80.6 | 14:28:15.59 | 0.592 | 0.667 | 0.593 | 0.537 | 0.608 | 372 | 701 |
| 84.6 | 14:28:19.59 | 0.558 | 0.639 | 0.560 | 0.604 | 0.692 | 404 | 1039 |
| 88.6 | 14:28:23.60 | 0.544 | 0.617 | 0.544 | 0.621 | 0.696 | 392 | 1170 |
| 92.6 | 14:28:27.60 | 0.528 | 0.592 | 0.528 | 0.625 | 0.696 | 324 | 798 |
| 96.6 | 14:28:31.61 | 0.492 | 0.564 | 0.492 | 0.542 | 0.613 | 364 | 732 |
| 100.6 | 14:28:35.61 | 0.475 | 0.542 | 0.475 | 0.629 | 0.696 | 316 | 715 |
| 104.6 | 14:28:39.62 | 0.419 | 0.517 | 0.419 | 0.542 | 0.617 | 400 | 886 |
| 108.6 | 14:28:43.62 | 0.411 | 0.492 | 0.411 | 0.537 | 0.613 | 452 | 1093 |
| 112.6 | 14:28:47.63 | 0.403 | 0.472 | 0.403 | 0.625 | 0.700 | 348 | 964 |
| 116.6 | 14:28:51.63 | 0.386 | 0.461 | 0.386 | 0.625 | 0.692 | 276 | 423 |
| 120.6 | 14:28:55.64 | 0.311 | 0.417 | 0.311 | 0.542 | 0.617 | 476 | 1043 |
| 124.6 | 14:28:59.64 | 0.317 | 0.392 | 0.318 | 0.550 | 0.608 | 284 | 475 |
| 128.6 | 14:29:03.65 | 0.306 | 0.367 | 0.306 | 0.546 | 0.613 | 252 | 427 |
| 132.6 | 14:29:07.65 | 0.286 | 0.344 | 0.286 | 0.554 | 0.617 | 196 | 451 |
| 136.6 | 14:29:11.65 | 0.253 | 0.342 | 0.253 | 0.554 | 0.621 | 276 | 454 |
| 140.6 | 14:29:15.66 | 0.239 | 0.294 | 0.239 | 0.554 | 0.625 | 232 | 371 |
| 144.6 | 14:29:19.66 | 0.222 | 0.283 | 0.222 | 0.558 | 0.621 | 228 | 433 |
| 148.6 | 14:29:23.67 | 0.214 | 0.256 | 0.214 | 0.567 | 0.617 | 72 | 98 |

 

| t (s) | overlay clock | x left (ref) | x right (ref) | x left (as recorded) | y top | y bottom | area (px) | signal |
|---|---|---|---|---|---|---|---|---|
| 12.5 | 14:57:38.76 | 0.831 | 0.931 | 0.831 | 0.546 | 0.692 | 1072 | 7891 |
| 16.5 | 14:57:42.75 | 0.792 | 0.897 | 0.792 | 0.537 | 0.625 | 1084 | 8354 |
| 20.5 | 14:57:46.75 | 0.781 | 0.864 | 0.782 | 0.537 | 0.617 | 584 | 1488 |
| 24.5 | 14:57:50.75 | 0.781 | 0.836 | 0.779 | 0.546 | 0.617 | 288 | 649 |
| 28.5 | 14:57:54.75 | 0.753 | 0.808 | 0.753 | 0.554 | 0.617 | 236 | 365 |
| 32.5 | 14:57:58.75 | 0.717 | 0.781 | 0.717 | 0.550 | 0.617 | 284 | 448 |
| 36.5 | 14:58:02.75 | 0.675 | 0.758 | 0.675 | 0.550 | 0.621 | 460 | 862 |
| 40.5 | 14:58:06.75 | 0.661 | 0.739 | 0.661 | 0.554 | 0.625 | 388 | 831 |
| 44.5 | 14:58:10.75 | 0.633 | 0.708 | 0.633 | 0.637 | 0.708 | 360 | 1044 |
| 48.5 | 14:58:14.75 | 0.611 | 0.681 | 0.611 | 0.637 | 0.704 | 288 | 584 |
| 52.5 | 14:58:18.75 | 0.586 | 0.661 | 0.586 | 0.550 | 0.625 | 480 | 1520 |
| 56.5 | 14:58:22.75 | 0.553 | 0.633 | 0.551 | 0.633 | 0.704 | 404 | 936 |
| 60.5 | 14:58:26.75 | 0.528 | 0.611 | 0.528 | 0.550 | 0.704 | 1072 | 3982 |
| 64.5 | 14:58:30.74 | 0.503 | 0.592 | 0.503 | 0.554 | 0.704 | 772 | 1765 |
| 68.6 | 14:58:34.74 | 0.478 | 0.553 | 0.478 | 0.642 | 0.708 | 228 | 396 |
| 72.6 | 14:58:38.74 | 0.453 | 0.533 | 0.453 | 0.550 | 0.621 | 488 | 1084 |
| 76.6 | 14:58:42.74 | 0.442 | 0.511 | 0.442 | 0.554 | 0.625 | 428 | 1085 |
| 80.6 | 14:58:46.74 | 0.414 | 0.486 | 0.412 | 0.550 | 0.629 | 400 | 983 |
| 84.6 | 14:58:50.74 | 0.397 | 0.464 | 0.397 | 0.558 | 0.625 | 284 | 626 |
| 88.6 | 14:58:54.74 | 0.358 | 0.439 | 0.358 | 0.562 | 0.625 | 404 | 853 |
| 92.6 | 14:58:58.74 | 0.344 | 0.414 | 0.344 | 0.554 | 0.629 | 440 | 893 |
| 96.6 | 14:59:02.74 | 0.300 | 0.392 | 0.300 | 0.558 | 0.629 | 536 | 1881 |
| 100.6 | 14:59:06.74 | 0.297 | 0.367 | 0.297 | 0.562 | 0.625 | 340 | 1439 |
| 104.6 | 14:59:10.74 | 0.286 | 0.344 | 0.286 | 0.562 | 0.625 | 232 | 799 |
| 109.6 | 14:59:15.73 | 0.233 | 0.311 | 0.233 | 0.567 | 0.629 | 264 | 517 |
| 113.6 | 14:59:19.73 | 0.225 | 0.289 | 0.225 | 0.567 | 0.629 | 292 | 855 |
| 117.6 | 14:59:23.73 | 0.217 | 0.267 | 0.217 | 0.575 | 0.633 | 160 | 321 |
| 124.6 | 14:59:30.73 | 0.200 | 0.242 | 0.201 | 0.608 | 0.662 | 112 | 617 |

 

| t (s) | overlay clock | x left (ref) | x right (ref) | x left (as recorded) | y top | y bottom | area (px) | signal |
|---|---|---|---|---|---|---|---|---|
| 57.0 | 18:24:58.74 | 0.825 | 0.900 | 0.643 | 0.575 | 0.725 | 856 | 2706 |
| 61.0 | 18:25:02.74 | 0.789 | 0.867 | 0.607 | 0.575 | 0.642 | 308 | 518 |
| 65.0 | 18:25:06.74 | 0.761 | 0.872 | 0.579 | 0.575 | 0.729 | 1004 | 3820 |
| 69.1 | 18:25:10.74 | 0.744 | 0.817 | 0.562 | 0.654 | 0.729 | 348 | 720 |
| 73.1 | 18:25:14.74 | 0.725 | 0.792 | 0.543 | 0.637 | 0.717 | 252 | 400 |
| 77.1 | 18:25:18.74 | 0.703 | 0.747 | 0.521 | 0.579 | 0.637 | 132 | 226 |
| 81.1 | 18:25:22.74 | 0.664 | 0.736 | 0.482 | 0.575 | 0.633 | 128 | 150 |
| 85.1 | 18:25:26.74 | 0.631 | 0.706 | 0.449 | 0.646 | 0.721 | 320 | 552 |
| 89.1 | 18:25:30.74 | 0.611 | 0.683 | 0.429 | 0.650 | 0.721 | 316 | 649 |
| 93.1 | 18:25:34.74 | 0.581 | 0.656 | 0.399 | 0.650 | 0.717 | 264 | 420 |
| 97.1 | 18:25:38.74 | 0.561 | 0.628 | 0.379 | 0.571 | 0.637 | 276 | 459 |
| 101.1 | 18:25:42.74 | 0.539 | 0.608 | 0.357 | 0.650 | 0.717 | 268 | 545 |
| 105.1 | 18:25:46.74 | 0.514 | 0.583 | 0.332 | 0.571 | 0.717 | 700 | 1546 |
| 109.1 | 18:25:50.73 | 0.478 | 0.556 | 0.296 | 0.650 | 0.717 | 380 | 764 |
| 113.1 | 18:25:54.73 | 0.467 | 0.533 | 0.285 | 0.650 | 0.717 | 308 | 701 |
| 117.1 | 18:25:58.73 | 0.433 | 0.511 | 0.251 | 0.650 | 0.713 | 336 | 763 |
| 121.1 | 18:26:02.73 | 0.414 | 0.481 | 0.232 | 0.571 | 0.637 | 284 | 583 |
| 125.1 | 18:26:06.73 | 0.392 | 0.458 | 0.210 | 0.571 | 0.637 | 268 | 537 |
| 129.1 | 18:26:10.73 | 0.369 | 0.436 | 0.188 | 0.650 | 0.717 | 268 | 457 |
| 133.1 | 18:26:14.73 | 0.325 | 0.411 | 0.143 | 0.654 | 0.717 | 376 | 714 |
| 137.1 | 18:26:18.73 | 0.317 | 0.383 | 0.135 | 0.571 | 0.637 | 276 | 617 |
| 141.1 | 18:26:22.73 | 0.297 | 0.361 | 0.115 | 0.567 | 0.633 | 232 | 704 |
| 145.1 | 18:26:26.73 | 0.275 | 0.339 | 0.093 | 0.571 | 0.633 | 216 | 494 |
| 149.1 | 18:26:30.73 | 0.242 | 0.308 | 0.060 | 0.571 | 0.637 | 252 | 511 |
| 153.1 | 18:26:34.73 | 0.222 | 0.292 | 0.040 | 0.567 | 0.633 | 328 | 1193 |
| 157.1 | 18:26:38.73 | 0.208 | 0.261 | 0.026 | 0.575 | 0.637 | 176 | 353 |
| 158.1 | 18:26:39.73 | 0.203 | 0.258 | 0.021 | 0.575 | 0.637 | 92 | 126 |

 

| t (s) | overlay clock | x left (ref) | x right (ref) | x left (as recorded) | y top | y bottom | area (px) | signal |
|---|---|---|---|---|---|---|---|---|
| 102.1 | 14:52:48.78 | 0.728 | 0.769 | 0.718 | 0.483 | 0.550 | 64 | 214 |

 (the single T10 row is the rejected grey-patch window).

**Speeds** (fraction of duct frame width per second, ± standard error):

| test | segment | n | downstream tip | upstream end | mean length | mean signal |
|---|---|---|---|---|---|---|
| T5 | FAN 61, 56.5–64.5 s | 8 | −0.0158 ± 0.0005 | −0.0117 ± 0.0007 | 0.186 | 108 500 |
| T5 | FAN 42, 75–147 s | 72 | −0.0058 ± 0.0001 | −0.0063 ± 0.0001 | 0.073 | 875 |
| T6 | FAN 47, 12.5–18 s | 5 | −0.0081 ± 0.0008 | −0.0058 ± 0.0004 | 0.103 | 8 760 |
| T6 | FAN 42, 24–118 s | 93 | −0.00606 ± 0.00004 | −0.00614 ± 0.00003 | 0.073 | 1 020 |
| T16 | FAN 46, 57–67.5 s | 11 | −0.0071 ± 0.0008 | −0.0029 ± 0.0006 | 0.092 | 2 120 |
| T16 | FAN 43, 73–157 s | 84 | −0.00621 ± 0.00004 | −0.00613 ± 0.00002 | 0.069 | 662 |

**Direction:** upstream is on the right in both cameras.
- The igniter is at the right: x 0.90–0.93 in the duct view, 0.88–0.95 in the top view.
- The luminous zone is anchored at its right end and opens or trails to the left.
- In every burn it moves right to left, away from the igniter.

This is consistent with concurrent flow: for concurrent tests the igniter sits near the inlet (ICES19). Supporting [inference]: the radiometer sits at the downstream end looking upstream, and in T5 its reading rises from 0.31 to 0.56 while the shrinking flame approaches the left end. Neither view shows the fan inlet.

## 4. Top view

**What exists in the archive**
- T5: still sequence and proof sheet.
- T6: proof sheet only.
- T16: proof sheet only.
- T10: proof sheet, plus the `original videos.zip` (video files only).
- None of the four tests has original NEF or JPEG stills.

**What the extra products turned out to be**
- **`T5_Still_Sequence.mp4`** is a 720 × 480 recording (563 frames, 18.8 s) of the camera's playback screen, showing images 95–110 as thumbnails of about 130 × 85 px. Unusable for measurement.
- **The T10 zip** contains `BASS_GMT175_SAMPLE_134_TEST_T10.ts` (22.5 MB, about 2.3 times the bit-rate of the posted mp4, used for the T10 analysis) and the `2ND_VIEW .ts`, which I did not open.

**Stills per test**

| test | stills on sheet | during test | shot (matrix) | mean cadence |
|---|---|---|---|---|
| T5 | 107 | 100 (14:27:42–14:29:33) + 7 post-test | 100 | 1.121 s |
| T6 | 106 | 100 (to 14:59:20) + 6 post-test | 117 | 1.131 s |
| T16 | 103 | 100 (to 18:26:35) + 3 post-test | 100 | 1.121 s |
| T10 | 22 | 18 + 4 post-test | 43 | 1.235 s |

**What the view shows**
- A face-on view of the dark holder: a rounded window with gold tabs and screws. The sample strip runs across at y ≈ 0.56, the igniter coil is at the right end of the window, and the window's left end is at x ≈ 0.16–0.17.
- While the igniter glows, the whole window is lit orange.
- The flame is a blue-violet "D": flat and edged with red glowing char on its right, rounded on its left. Before the T5 step it is a long cup with a tail to the left.
- **Post-test lit stills:**
  - T6 and T16: charred over the whole window except unburnt strips along both long edges and around the left end.
  - T10: clean white sample with char only at the igniter end, roughly 0.5–1 sample width by eye.
  - T5: mostly overexposed and not interpretable with confidence.

**Measurability**
- **Leading edge: yes.**
  - Method: subtract the per-pixel median of dark stills, apply a 3 × 3 median filter, threshold blue at 40 above background, take the largest region ≥ 12 px, with a 3 px border mask.
  - Resolution: 1 px = 0.0031 of the frame width = 0.018 sample widths. Standard errors of the low-flow fits are 0.3–0.7 %.
- **Glowing char edge:** measurable only in the first 10–20 stills after ignition (recorded as ember_x_min/max).
- **Dark charred region:** not visible in the dark stills.
- **Sample band width** (a measurement, not a calibration): measured at half maximum at 9 columns in every igniter-lit still.

| test | median width (px of 324) | p10–p90 | profiles |
|---|---|---|---|
| T5 | 56.7 | 56.2–72.1 | 36 |
| T6 | 55.6 | 53.8–56.4 | 25 |
| T16 | 56.2 | 55.2–79.6 | 63 |
| T10 | 55.6 | 53.7–73.5 | 45 |

- The high p90 values are profiles where the edge search caught a holder rail.
- The T10 post-test still independently gives about 56.5 px for the window opening.
- **Window downstream (left) end:** 54.0 px (T5), 53.7 (T16), 51.9 (T10). T6's band was not lit to its end, so for T6 only displacements in sample widths are given.
- **Why no millimetres:** the matrix says "2 cm SIBAL"; the dossier says 1.2 or 2.2 cm. It is not documented whether the band is the fabric or the window opening, and the window's right end is hidden behind the igniter.
- **Consistency check only:** if the band were 2.2 cm, the originals would be about 34 px/mm, which is the SR18 value. At ignition the flame sits about 4.1–4.2 sample widths from the window end, compatible with a 10 cm window (4.5 widths).

**Tracks:** 

| still | clock (camera) | x left | x right | y top | y bottom | left end, sample widths from window end | area (px of 324x216) | blue sum |
|---|---|---|---|---|---|---|---|---|
| 2 | 14:27:44 | 0.892 | 0.954 | 0.454 | 0.676 | 4.15 | 519 | 44174 |
| 10 | 14:27:53 | 0.719 | 0.923 | 0.426 | 0.708 | 3.16 | 3067 | 270614 |
| 13 | 14:27:56 | 0.673 | 0.895 | 0.426 | 0.708 | 2.89 | 3098 | 265512 |
| 16 | 14:28:00 | 0.636 | 0.806 | 0.426 | 0.704 | 2.68 | 2687 | 225218 |
| 19 | 14:28:03 | 0.660 | 0.768 | 0.431 | 0.694 | 2.82 | 1519 | 102393 |
| 22 | 14:28:06 | 0.670 | 0.735 | 0.449 | 0.681 | 2.88 | 806 | 47852 |
| 25 | 14:28:10 | 0.639 | 0.704 | 0.454 | 0.671 | 2.70 | 749 | 45564 |
| 28 | 14:28:13 | 0.605 | 0.676 | 0.454 | 0.667 | 2.50 | 723 | 44327 |
| 31 | 14:28:17 | 0.583 | 0.648 | 0.458 | 0.667 | 2.38 | 692 | 41069 |
| 34 | 14:28:20 | 0.559 | 0.624 | 0.458 | 0.667 | 2.24 | 686 | 40763 |
| 37 | 14:28:23 | 0.537 | 0.599 | 0.458 | 0.662 | 2.12 | 653 | 38619 |
| 40 | 14:28:27 | 0.515 | 0.577 | 0.458 | 0.662 | 1.99 | 642 | 37322 |
| 43 | 14:28:30 | 0.491 | 0.562 | 0.458 | 0.657 | 1.85 | 705 | 41540 |
| 46 | 14:28:33 | 0.463 | 0.537 | 0.458 | 0.657 | 1.69 | 729 | 43815 |
| 49 | 14:28:37 | 0.448 | 0.515 | 0.463 | 0.657 | 1.61 | 690 | 41012 |
| 52 | 14:28:40 | 0.429 | 0.497 | 0.463 | 0.657 | 1.50 | 629 | 37339 |
| 55 | 14:28:43 | 0.407 | 0.475 | 0.458 | 0.657 | 1.38 | 645 | 37286 |
| 58 | 14:28:47 | 0.386 | 0.454 | 0.458 | 0.657 | 1.25 | 636 | 36273 |
| 61 | 14:28:50 | 0.364 | 0.429 | 0.458 | 0.657 | 1.13 | 624 | 35848 |
| 64 | 14:28:54 | 0.343 | 0.410 | 0.458 | 0.657 | 1.01 | 658 | 37647 |
| 67 | 14:28:57 | 0.327 | 0.386 | 0.463 | 0.653 | 0.92 | 536 | 29262 |
| 70 | 14:29:00 | 0.309 | 0.364 | 0.468 | 0.648 | 0.81 | 514 | 27568 |
| 73 | 14:29:04 | 0.290 | 0.346 | 0.477 | 0.643 | 0.71 | 483 | 24279 |
| 76 | 14:29:07 | 0.269 | 0.321 | 0.477 | 0.643 | 0.58 | 450 | 21751 |
| 79 | 14:29:11 | 0.247 | 0.296 | 0.481 | 0.643 | 0.46 | 365 | 16914 |
| 82 | 14:29:14 | 0.225 | 0.272 | 0.481 | 0.643 | 0.34 | 311 | 14606 |
| 85 | 14:29:17 | 0.204 | 0.259 | 0.481 | 0.643 | 0.21 | 365 | 17900 |
| 88 | 14:29:21 | 0.185 | 0.241 | 0.491 | 0.634 | 0.11 | 399 | 20528 |
| 91 | 14:29:24 | 0.170 | 0.216 | 0.532 | 0.607 | 0.02 | 187 | 11841 |
| 94 | 14:29:27 | 0.194 | 0.219 | 0.556 | 0.588 | 0.16 | 39 | 2994 |
| 95 | 14:29:29 | 0.222 | 0.272 | 0.579 | 0.588 | 0.32 | 32 | 1954 |

 

| still | clock (camera) | x left | x right | y top | y bottom | left end, sample widths from window end | area (px of 324x216) | blue sum |
|---|---|---|---|---|---|---|---|---|
| 2 | 14:57:30 | 0.883 | 0.941 | 0.449 | 0.681 |  | 407 | 26665 |
| 8 | 14:57:37 | 0.858 | 0.944 | 0.468 | 0.681 |  | 961 | 75562 |
| 11 | 14:57:41 | 0.815 | 0.910 | 0.449 | 0.694 |  | 1297 | 103296 |
| 14 | 14:57:44 | 0.787 | 0.895 | 0.435 | 0.699 |  | 1402 | 101593 |
| 17 | 14:57:47 | 0.790 | 0.861 | 0.454 | 0.681 |  | 792 | 48015 |
| 20 | 14:57:51 | 0.765 | 0.836 | 0.454 | 0.681 |  | 827 | 50849 |
| 23 | 14:57:54 | 0.747 | 0.812 | 0.458 | 0.681 |  | 753 | 47090 |
| 26 | 14:57:57 | 0.719 | 0.784 | 0.458 | 0.676 |  | 740 | 46335 |
| 29 | 14:58:01 | 0.704 | 0.765 | 0.454 | 0.676 |  | 739 | 45446 |
| 32 | 14:58:04 | 0.676 | 0.741 | 0.454 | 0.676 |  | 752 | 46022 |
| 35 | 14:58:08 | 0.657 | 0.719 | 0.454 | 0.671 |  | 736 | 45695 |
| 38 | 14:58:11 | 0.636 | 0.698 | 0.454 | 0.671 |  | 715 | 43517 |
| 41 | 14:58:14 | 0.614 | 0.676 | 0.454 | 0.667 |  | 708 | 43429 |
| 44 | 14:58:18 | 0.586 | 0.651 | 0.454 | 0.667 |  | 747 | 45714 |
| 47 | 14:58:21 | 0.568 | 0.626 | 0.454 | 0.667 |  | 691 | 42750 |
| 50 | 14:58:24 | 0.543 | 0.608 | 0.458 | 0.671 |  | 792 | 48074 |
| 53 | 14:58:28 | 0.518 | 0.583 | 0.458 | 0.667 |  | 703 | 41103 |
| 56 | 14:58:31 | 0.497 | 0.562 | 0.463 | 0.657 |  | 645 | 37390 |
| 59 | 14:58:35 | 0.478 | 0.540 | 0.463 | 0.653 |  | 607 | 35504 |
| 62 | 14:58:38 | 0.457 | 0.518 | 0.468 | 0.653 |  | 639 | 37199 |
| 65 | 14:58:41 | 0.435 | 0.497 | 0.463 | 0.653 |  | 621 | 36919 |
| 68 | 14:58:45 | 0.417 | 0.475 | 0.463 | 0.653 |  | 590 | 34569 |
| 71 | 14:58:48 | 0.398 | 0.460 | 0.463 | 0.648 |  | 579 | 32931 |
| 74 | 14:58:51 | 0.377 | 0.435 | 0.458 | 0.648 |  | 584 | 33407 |
| 77 | 14:58:55 | 0.355 | 0.414 | 0.468 | 0.648 |  | 534 | 30409 |
| 80 | 14:58:58 | 0.336 | 0.398 | 0.468 | 0.648 |  | 545 | 30870 |
| 83 | 14:59:02 | 0.321 | 0.380 | 0.468 | 0.648 |  | 563 | 30553 |
| 86 | 14:59:05 | 0.296 | 0.355 | 0.468 | 0.643 |  | 509 | 25465 |
| 89 | 14:59:08 | 0.281 | 0.327 | 0.477 | 0.639 |  | 418 | 20148 |
| 92 | 14:59:12 | 0.265 | 0.315 | 0.481 | 0.634 |  | 331 | 14955 |
| 95 | 14:59:15 | 0.241 | 0.278 | 0.486 | 0.630 |  | 196 | 8753 |
| 98 | 14:59:18 | 0.219 | 0.262 | 0.481 | 0.630 |  | 278 | 12671 |
| 99 | 14:59:20 | 0.213 | 0.259 | 0.481 | 0.630 |  | 275 | 12848 |

 

| still | clock (camera) | x left | x right | y top | y bottom | left end, sample widths from window end | area (px of 324x216) | blue sum |
|---|---|---|---|---|---|---|---|---|
| 3 | 18:24:47 | 0.880 | 0.951 | 0.421 | 0.685 | 4.12 | 758 | 84662 |
| 14 | 18:25:00 | 0.821 | 0.895 | 0.449 | 0.657 | 3.78 | 870 | 56812 |
| 17 | 18:25:03 | 0.796 | 0.864 | 0.454 | 0.657 | 3.64 | 755 | 50416 |
| 20 | 18:25:06 | 0.772 | 0.846 | 0.444 | 0.662 | 3.49 | 850 | 56773 |
| 23 | 18:25:10 | 0.741 | 0.821 | 0.449 | 0.657 | 3.32 | 759 | 50420 |
| 26 | 18:25:13 | 0.722 | 0.787 | 0.458 | 0.653 | 3.21 | 584 | 37169 |
| 29 | 18:25:17 | 0.710 | 0.772 | 0.458 | 0.648 | 3.14 | 584 | 36363 |
| 32 | 18:25:20 | 0.685 | 0.744 | 0.463 | 0.648 | 3.00 | 560 | 35326 |
| 35 | 18:25:23 | 0.660 | 0.722 | 0.468 | 0.648 | 2.85 | 550 | 34438 |
| 38 | 18:25:27 | 0.639 | 0.701 | 0.468 | 0.648 | 2.73 | 584 | 36440 |
| 41 | 18:25:30 | 0.617 | 0.679 | 0.472 | 0.648 | 2.60 | 562 | 35048 |
| 44 | 18:25:33 | 0.593 | 0.654 | 0.472 | 0.639 | 2.46 | 550 | 34777 |
| 47 | 18:25:37 | 0.574 | 0.633 | 0.477 | 0.643 | 2.35 | 526 | 33564 |
| 50 | 18:25:40 | 0.556 | 0.614 | 0.481 | 0.643 | 2.25 | 490 | 31013 |
| 53 | 18:25:44 | 0.525 | 0.590 | 0.481 | 0.643 | 2.07 | 510 | 31173 |
| 56 | 18:25:47 | 0.497 | 0.568 | 0.477 | 0.643 | 1.91 | 588 | 35310 |
| 59 | 18:25:50 | 0.485 | 0.549 | 0.477 | 0.643 | 1.84 | 535 | 33521 |
| 62 | 18:25:54 | 0.463 | 0.528 | 0.481 | 0.643 | 1.71 | 528 | 33017 |
| 65 | 18:25:57 | 0.441 | 0.503 | 0.481 | 0.639 | 1.59 | 499 | 31063 |
| 68 | 18:26:00 | 0.420 | 0.481 | 0.481 | 0.643 | 1.47 | 502 | 30998 |
| 71 | 18:26:04 | 0.401 | 0.463 | 0.481 | 0.639 | 1.36 | 490 | 29568 |
| 74 | 18:26:07 | 0.377 | 0.438 | 0.486 | 0.639 | 1.22 | 477 | 28407 |
| 77 | 18:26:11 | 0.358 | 0.420 | 0.491 | 0.639 | 1.11 | 464 | 27936 |
| 80 | 18:26:14 | 0.340 | 0.398 | 0.486 | 0.643 | 1.00 | 470 | 27740 |
| 83 | 18:26:17 | 0.321 | 0.377 | 0.491 | 0.643 | 0.90 | 464 | 26505 |
| 86 | 18:26:21 | 0.296 | 0.355 | 0.491 | 0.639 | 0.75 | 470 | 24780 |
| 89 | 18:26:24 | 0.278 | 0.330 | 0.486 | 0.639 | 0.65 | 446 | 22771 |
| 92 | 18:26:27 | 0.259 | 0.315 | 0.486 | 0.639 | 0.54 | 428 | 19970 |
| 95 | 18:26:31 | 0.232 | 0.284 | 0.481 | 0.639 | 0.38 | 300 | 14022 |
| 98 | 18:26:34 | 0.207 | 0.262 | 0.477 | 0.634 | 0.24 | 365 | 17984 |
| 99 | 18:26:35 | 0.198 | 0.256 | 0.481 | 0.634 | 0.18 | 386 | 19284 |

 

| still | clock (camera) | x left | x right | y top | y bottom | left end, sample widths from window end | area (px of 324x216) | blue sum |
|---|---|---|---|---|---|---|---|---|
| 11 | 14:51:30 | 0.883 | 0.910 | 0.444 | 0.662 | 4.21 | 194 | 15662 |
| 14 | 14:51:34 | 0.864 | 0.876 | 0.556 | 0.579 | 4.11 | 14 | 1253 |

. For T10 there is only a region at the igniter shrinking from 194 to 14 px within 4 s of igniter-off; a brief flame cannot be told from the cooling coil.

**Speeds** (leading edge; fraction of still width per second and sample widths per second; trailing end in fraction of width per second):

| test | segment (camera clock) | n | lead (fw/s) | lead (w/s) | trail (fw/s) | mean length | mean area | mean blue sum |
|---|---|---|---|---|---|---|---|---|
| T5 | FAN 61, 14:27:51–14:28:00 | 9 | −0.0153 ± 0.0016 | −0.087 | −0.0131 ± 0.0022 | 0.202 | 2950 | 258 000 |
| T5 | ramp, 14:28:00–08 | 8 | +0.0030 ± 0.0015 | +0.017 | −0.0105 ± 0.0006 | 0.170→0.068 | 2687→782 | 107 000 |
| T5 | FAN 42, 14:28:08–14:29:23 | 68 | −0.00623 ± 0.00004 | −0.0356 | −0.00650 ± 0.00003 | 0.062 | 586 | 33 300 |
| T6 | FAN 47, 14:57:37–44 | 7 | −0.0092 ± 0.0010 | −0.054 | −0.0060 ± 0.0008 | 0.098 | 1237 | 95 400 |
| T6 | ramp, 14:57:44–50 | 6 | −0.0024 ± 0.0009 | −0.014 | −0.0102 ± 0.0018 | 0.108→0.065 | 1402→792 | 63 000 |
| T6 | FAN 42, 14:57:50–14:59:20 | 81 | −0.00624 ± 0.00003 | −0.0364 | −0.00646 ± 0.00002 | 0.060 | 614 | 35 900 |
| T16 | FAN 46, 18:24:59–18:25:09 | 10 | −0.00757 ± 0.00026 | −0.0437 | −0.0081 ± 0.0006 | 0.074 | 830 | 55 100 |
| T16 | ramp, 18:25:09–15 | 7 | −0.0060 ± 0.0006 | −0.034 | −0.0089 ± 0.0006 | 0.074→0.059 | 838→563 | 42 600 |
| T16 | FAN 43, 18:25:15–18:26:35 | 72 | −0.00635 ± 0.00002 | −0.0366 | −0.00645 ± 0.00002 | 0.060 | 499 | 29 800 |

## 5. Flame and flow at the fan steps

**T5** (FAN 61→42, 64.6–71.1 s)
- Duct brightness (0.5-s windows) is about 87 000 before. It falls to 73 500 in the first window with FAN below 61 (65.3 s), 44 500 at 66.3 s, about 6 000 at 70.8 s, and 1 700–5 900 over 72–74 s.
- Duct area falls from about 5 200 to about 1 160 px by 71.3 s.
- Top-view area: 3093 / 2687 / 2249 / 1841 / 1519 / 1164 / 917 / 806 / 782 px at 14:27:58 through 14:28:08; blue sum 264 000 → 46 000.
- Delay: no lag beyond the 0.5-s window; the flame settles in 7–8 s.
- The flame shortens from its downstream tip, from about 0.20 to about 0.07 (top: 0.17 → 0.068). The tip moves back about 0.02 (duct 0.636 → 0.664; top 0.636 → 0.670) while the upstream end keeps moving.
- Speed drops about 2.5×: 0.0153 → 0.0062 fw/s (top) and 0.0158 → 0.0058 (duct).
- AIR stays 10–11 until about 74 s, then falls (9 at 74 s, 6 at 80 s, 3 at 102 s): more than 8 s of lag.
- RAD peaks at 0.46 at 66 s, then falls to 0.31 by about 78 s.

**T6** (47→42, 18.0–21.0 s)
- Duct brightness: 8 500 → 6 600 (18.8 s) → 2 400 (19.8 s) → 1 200 (20.8 s) → about 500 from 21.8 s.
- Top-view area 1429 → 760 px from 14:57:43 to 14:57:48.
- Delay is at most 0.5 s.
- Speed 0.0092 → 0.0062 fw/s; the "before" value comes from 7 stills just after the igniter, so it is not a steady rate.
- RAD changes little (0.21 → about 0.20).

**T16** (46→43, 67.6–69.6 s)
- Duct brightness: about 2 400 → 1 000 (67.8 s) → 450 (69.3 s) → 200–500.
- Duct area 860 → 130–250 px.
- Top-view area 838 → 584 px from 18:25:09 to 18:25:13.
- The first drop is coincident with the ramp start within the 0.5-s sampling.
- Speed −16 %: 0.00757 → 0.00635 fw/s.
- RAD 0.23 → 0.22.

**Across the three tests**
- After the step, all three spread at 0.0062–0.0064 fw/s (0.036 widths/s): T5 and T6 at FAN 42/41 with 18.8 % O2, T16 at FAN 43 with 17.6 % O2.
- No step shows a delay beyond the sampling (0.5 s duct, about 1.1 s top).
- Adjustment takes about 2–8 s, longer for the bigger step.
- During every low-flow phase, area and brightness keep decaying slowly (T5 782 → 315 px over 75 s).

## Files downloaded

All into the new folder `D:\Caches\Temp\claude\23fcab7049aa850b\bd40257b-5cbd-4ff9-b44f-69568de1670b\scratchpad\bass_train_dl2\`, using curl with a generic user agent and the PSI-25 download URL.

| archive file | saved as | bytes |
|---|---|---|
| `PSI-25_Raw Data_BASS_GMT100_SAMPLE_131_TEST_T5_Still_Sequence.mp4` | `T5_Still_Sequence.mp4` | 1 859 994 |
| `PSI-25_Raw Data_GMT2014_100_BASS_S132M_TPT6_ProofSheet.pdf` | `T6_ProofSheet.pdf` | 561 256 |
| `PSI-25_Raw Data_GMT2014_100_BASS_S139M_TPT16_ProofSheet.pdf` | `T16_ProofSheet.pdf` | 503 414 |
| `PSI-25_Raw Data_BASS_GMT175_SAMPLE_134_TEST_T10 original videos.zip` | extracted to `T10_orig/` (two `.ts`, 22 538 944 and 22 516 196 bytes) | 36 687 937 |
| **total** | | **39 612 601 (37.8 MiB)** |

Nothing of any other test was downloaded or opened. A grep of the experimental-table file showed rows B8, B11 and B19 by accident (because "131" appears in their CO column); they were not used.

## What a forecasting target could honestly be

**Target**
- **Quantity:** position of the leading (downstream) edge of the blue flame versus time.
- **Camera and units:** the top-view proof-sheet stills (about 1.12 s apart), in sample widths from the downstream window end (no mm needed), or as a fraction of the still width.
- **Second, independent measure:** the duct-view downstream tip in 1-s windows. It is noisier (scatter 0.01 against under 0.003) but extends T6 by about 10 s and T16 to the edge of the picture.
- **Tests:** T5, T6 and T16 give tracks of about 100 s each. T10 should be scored as "no spread", at 16.4 % O2 and about 6 cm/s.

**Known controls**
- The FAN time series and step times, at 0.5-s resolution.
- U only where the day's table covers the setting.
- O2 initial and final values.
- "2 cm SIBAL" and restrictor 2.
- The igniter-on interval.
- Orientation: igniter and upstream at the right.

**Limits**
- Each tracked test has a short pre-step phase (7–10 stills) and a long low-flow phase. In the low-flow phases all three tests spread at the same rate within 2 %.
- The record cannot support a rate-versus-flow law: the low settings are uncalibrated and there are only three pre-step points.
- No absolute mm/s, and no physical brightness or length.

## What the data team would have to provide

1. The exposed width of the "2 cm SIBAL" window (2.0 or 2.2 cm), plus the holder drawing from PSI-26, to turn the measured 56 px (about 740 px in the originals) into a mm scale. The duct view has no object of documented size.
2. The original D300s stills (4288 × 2848, about 13× the proof-sheet resolution), including the stills missing from the sheets: T6 has 117 shot vs 100 shown, plausibly the end of the burn; T10 has 43 vs 18.
3. The still-camera clock offset relative to the overlay; here it is 0–2.3 s.
4. Fan calibration for FAN 41–46 on GMT 100 (lowest usable row there is 46.5 → 5.0), or a ruling on borrowing the GMT 174/175 rows.
5. O2 history within each burn, or a ruling that a linear decay may be assumed.
6. Radiometer calibration and gains.
7. Confirmation of the flow orientation from hardware documents; here it is inferred from the images and ICES19.

## Scripts and outputs

All in `D:\Caches\Temp\claude\23fcab7049aa850b\bd40257b-5cbd-4ff9-b44f-69568de1670b\scratchpad\analysis\bass\`:
- **Overlay:** `ovl_*.py`, `clockmap.py`, `overlay_tables.py` → `overlay_raw_T*.json`, `flow_history.json`.
- **Duct view:** `duct_pass.py` / `duct_detect.py` / `duct_final.py` / `duct_fits.py` → `duct_tracks.json`, `duct_fits.json`, `duct_track_*.json`.
- **Top view:** `proof_*.py`, `band_width.py`, `topview_track.py`, `topview_fits.py` → `topview_track.json`, `topview_band_width.json`, `proof/thumbs/` (338 thumbnails) with `proof/thumbs.json`.
- **Timeline and tables:** `timeline_events.py` → `timeline.json`; `report_tables.py` → `report_chunks.json`.
- **Images:** `img/` holds the contact sheets, overlay verification strips and mosaics.
- The intermediate `duct_T*.pkl` files (about 800 MB) were deleted; `duct_pass.py` rebuilds them.
