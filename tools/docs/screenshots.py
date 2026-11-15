"""Capture the README screenshots from a running IGNIS UI into docs/images/.

    python tools/docs/screenshots.py [--base https://ignis-ardealu.app] [--profile DIR]

Opens a visible Chromium with a persistent profile: the first time, sign in through Cloudflare
Access in that window; the script waits (up to 10 minutes) until the app shell is there, then
visits each view and saves a 1600x1000 light-theme screenshot. Requires `pip install playwright`
and `python -m playwright install chromium`.
"""
import argparse
import pathlib
import sys

from playwright.sync_api import sync_playwright

SHOTS = [
    ("home", "#/"),
    ("research-answer", "#/research/ep_2105cece292b4fa9915a6e8415f8e461"),
    ("sources", "#/sources?q=oxygen%20concentration%20flammability%20limit"),
    ("runs", "#/runs"),
    ("conditions", "#/conditions"),
    ("atmosphere-map", "#/conditions?tab=map"),
    ("forecast-lab", "#/lab/PSI-69%3A192F002%3A2009-10-24"),
    ("compare", "#/compare"),
    ("models", "#/models"),
]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", default="https://ignis-ardealu.app")
    ap.add_argument("--profile", default=str(pathlib.Path.home() / ".ignis-screenshots-profile"))
    ap.add_argument("--only", nargs="*")
    a = ap.parse_args()
    out = pathlib.Path(__file__).resolve().parents[2] / "docs" / "images"
    out.mkdir(parents=True, exist_ok=True)
    with sync_playwright() as p:
        ctx = p.chromium.launch_persistent_context(
            a.profile, headless=False, viewport={"width": 1600, "height": 1000},
            color_scheme="light", device_scale_factor=1,
        )
        page = ctx.pages[0] if ctx.pages else ctx.new_page()
        page.goto(a.base + "/#/")
        print("waiting for the app (sign in in the browser window if asked)...", flush=True)
        page.wait_for_selector("nav >> text=Runs", timeout=600_000)
        for name, route in SHOTS:
            if a.only and name not in a.only:
                continue
            page.goto(a.base + "/" + route)
            page.wait_for_load_state("networkidle")
            page.wait_for_timeout(2500)
            page.screenshot(path=str(out / f"{name}.png"))
            print("saved", name, flush=True)
        ctx.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
