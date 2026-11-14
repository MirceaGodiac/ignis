# ignis-vision

BASS visible-front metrology (V3.1 §3.4, §4.4–4.5, §9.1–9.3). Turns a NASA MP4 clip into
`ContourObservation` records for the BASS experts.

Pipeline: decode with **container PTS** → optional lens undistortion → admitted ROI → segmentation
(classical comparator in Rust, or SAM2.1 in the Python sidecar; both strictly causal) →
leading-luminous boundary on a fixed transverse grid → homography to the specimen plane →
`ContourObservation` with status and scale uncertainty. Topology changes (multivalued, detached,
occluded fronts) are reported as status, never smoothed away.

## Modules

| Module | Contents |
|---|---|
| `video` | probe, MP4/H.264 decode with real PTS, PNG frame cache (`data/cache/frames`, `IGNIS_CACHE`) |
| `imageio` | minimal PNG I/O for the cache and the sidecar exchange |
| `calibration` | pixel → specimen-plane calibration from cited reference points (homography) |
| `segment` | classical causal segmentation |
| `sam2` | client for `sidecar/sam2/ignis_sam2.py` (JSON lines over stdin/stdout, lock-stepped) |
| `contour` | leading-luminous boundary on the fixed grid |
| `pipeline` | end-to-end `contours` run, output under `artifacts/vision/` |
| `cli` | `ignis vision probe \| frames \| contours` |

Configs: `configs/vision/segmentation.toml`, `sam2.toml` (checkpoint, device, sidecar command),
`calibration.example.json` (illustrative only), `calibration.psi25_T12.pixel.json` (pixel-only, no
cited reference points yet).

## Commands

```bash
ignis vision probe <clip.mp4>
ignis vision frames <clip.mp4> [--every 30] [--max-frames 500] [--execute]
ignis vision contours <clip.mp4> --calibration <spec.json> [--method classical|sam2] \
  [--run-id <id>] [--time-origin-s <t0>] [--every 1] [--profile p] [--execute]
```

## Status (development laptop)

Identical output to the previous Python pipeline on a real BASS clip; 403 frames/s (previously 42).
Contours for the BASS cohort videos have not been produced yet.

## Tests

```bash
cargo test --release -p ignis-vision
```

`pts_come_from_the_container`, `contour_masks_are_causal`, `homography_maps_declared_points`.

See also: [GUIDE](../../docs/GUIDE.md#10-vision-metrology) · [SAM2 sidecar](../../sidecar/sam2/README.md) ·
[REPOMAP](../../docs/REPOMAP.md)
