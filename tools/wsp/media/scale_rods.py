"""Width of the deployment-needle arms, in pixels, on the first frame of FLEX and FLEX-2 archives.

The same pixel search window is used for every image (straight dark rods 20-80 px wide on a 1024 px
image), so the result does not depend on any assumed scale. Binned 512 px images are reported per
1024 px. Writes <dir>/rods.json.

    python -I tools/wsp/media/scale_rods.py runs/dev/media/droplet/scale_frames
"""
import json
import sys
from pathlib import Path

import cv2

sys.path.insert(0, str(Path(__file__).resolve().parent))
import droplet_diameter as dd  # noqa: E402  (only its rod_width_px, unchanged)


def main(root):
    root = Path(root)
    res = {}
    for d in sorted(p for p in root.iterdir() if p.is_dir()):
        f = sorted(d.glob("*.tif"))
        if not f:
            continue
        im = cv2.imread(str(f[0]), cv2.IMREAD_UNCHANGED)
        w = im.shape[1]
        r = dd.rod_width_px(im, w / 30.25)  # window 0.3-1.2 mm at 33.85 px/mm per 1024 px: 20-81 px
        res[d.name] = dict(frame=f[0].name, width_px=w, rod_px=r and r[0], sections=r and r[1],
                           rod_px_per_1024=r and r[0] * 1024 / w)
    (root / "rods.json").write_text(json.dumps(res, indent=1) + "\n", encoding="utf-8")
    for k, v in res.items():
        print(k, v["width_px"], v["rod_px_per_1024"] and round(v["rod_px_per_1024"], 2), v["sections"])


if __name__ == "__main__":
    main(sys.argv[1])
