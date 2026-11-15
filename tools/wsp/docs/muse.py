"""Document lane: a minimal client for Meta's Muse Spark 1.3 Contributor (OpenAI-compatible).

Public NASA data only: the Contributor tier may train Meta's models on its inputs. The key is read
from D:/downloads/api.env (META_API_KEY) and never printed or stored.

    python tools/wsp/docs/muse.py --probe      # lists models and tries one image request
"""
import base64
import json
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

BASE = "https://api.meta.ai/v1"
MODEL = "muse-spark-1.3-contributor"
ENV = Path("D:/downloads/api.env")


def key():
    for line in ENV.read_text(encoding="utf-8").splitlines():
        if line.strip().startswith("META_API_KEY"):
            return line.split("=", 1)[1].strip().strip('"')
    raise SystemExit("META_API_KEY not found")


def call(messages, response_format=None, effort=None, timeout=600, retries=5):
    body = {"model": MODEL, "messages": messages}
    if response_format:
        body["response_format"] = response_format
    if effort:
        body["reasoning_effort"] = effort
    data = json.dumps(body).encode()
    for attempt in range(retries):
        req = urllib.request.Request(BASE + "/chat/completions", data=data, method="POST",
                                     headers={"Authorization": "Bearer " + key(), "Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                return json.loads(r.read())
        except urllib.error.HTTPError as e:
            msg = e.read().decode(errors="replace")[:500]
            if e.code in (429, 500, 502, 503, 504) and attempt < retries - 1:
                time.sleep(5 * (attempt + 1)); continue
            raise RuntimeError(f"HTTP {e.code}: {msg}")
        except (urllib.error.URLError, TimeoutError):
            if attempt < retries - 1:
                time.sleep(5 * (attempt + 1)); continue
            raise


def image_part(png_bytes):
    return {"type": "image_url", "image_url": {"url": "data:image/png;base64," + base64.b64encode(png_bytes).decode()}}


def probe():
    req = urllib.request.Request(BASE + "/models", headers={"Authorization": "Bearer " + key()})
    with urllib.request.urlopen(req, timeout=60) as r:
        ids = [m["id"] for m in json.loads(r.read()).get("data", [])]
    print("models:", ids)
    sys.path.insert(0, "runs/dev/data_astra/work/document_pages/dependencies")
    import fitz
    doc = fitz.open()
    page = doc.new_page(width=300, height=150)
    page.draw_rect(fitz.Rect(20, 20, 280, 130), color=(1, 0, 0), fill=(0, 0, 1))
    page.insert_text((40, 80), "TEST 42", fontsize=28, color=(1, 1, 1))
    png = page.get_pixmap(dpi=72).tobytes("png")
    t = time.time()
    r = call([{"role": "user", "content": [{"type": "text", "text": "What text and colours do you see? One line."}, image_part(png)]}])
    print("image answer:", r["choices"][0]["message"]["content"][:200], f"({time.time() - t:.1f} s)", "usage:", r.get("usage"))


if __name__ == "__main__":
    if "--probe" in sys.argv:
        probe()
