# SAM2 sidecar

A small Python process that runs **SAM2.1** video segmentation for `ignis vision contours --method sam2`.
It is the only Python in the repository. The Rust side (`crates/ignis-vision/src/sam2.rs`) starts it,
feeds frames one at a time and receives one mask per frame.

## Setup

Requires Python 3.11–3.13 and [uv](https://docs.astral.sh/uv/).

```bash
uv sync --project sidecar/sam2          # installs sam2 (from GitHub), torch, numpy, pillow
mkdir -p data/models/sam2
curl -L -o data/models/sam2/sam2.1_hiera_small.pt \
  https://dl.fbaipublicfiles.com/segment_anything_2/092824/sam2.1_hiera_small.pt
```

Settings live in [`configs/vision/sam2.toml`](../../configs/vision/sam2.toml): `checkpoint`
(default `data/models/sam2/sam2.1_hiera_small.pt`, or env `IGNIS_SAM2_CHECKPOINT`), `model_cfg`,
`device` (`cpu` by default; `cuda` on a GPU pod) and `sidecar_cmd`. `data/models/` is not gitignored:
do not commit the checkpoint.

## Use

```bash
ignis vision contours <clip.mp4> --calibration <spec.json> --method sam2              # plan
ignis vision contours <clip.mp4> --calibration <spec.json> --method sam2 --execute    # runs the sidecar
```

## Protocol

JSON lines on stdin, one reply per request on stdout:

| Request | Reply |
|---|---|
| `{"op":"init","checkpoint","model_cfg","device","width","height","work_dir"}` | `{"ok":true,"sam2_version","checkpoint_sha256"}` |
| `{"op":"frame","frame":t,"image":<png>,"prompt_box_px":[x0,y0,x1,y1] \| null}` | `{"ok":true,"frame":t,"mask":<png>,"dependency_frames":[...],"state":"prompted\|tracked"}` |
| `{"op":"close"}` | `{"ok":true,"manifest":{...}}` (also written to `<work_dir>/sam2_manifest.json`) |

Any failure returns `{"ok":false,"error":"..."}` and the Rust side stops the run.

## Causality

Frames arrive one at a time in increasing order and enter the predictor's frame store only when they
arrive. Propagation is forward only from the prompt frame, so the memory bank holds only frames in
[p0, t]; the mask at *t* is reported as depending on every fed frame in [p0, t]. A prompt is accepted
only on the frame being sent, and only once. This is what lets a SAM2 contour enter a causal prefix.

Changing the protocol means changing `ignis_sam2.py` and `crates/ignis-vision/src/sam2.rs` together.

See also: [ignis-vision](../../crates/ignis-vision/README.md) · [GUIDE §10](../../docs/GUIDE.md#10-vision-metrology)
