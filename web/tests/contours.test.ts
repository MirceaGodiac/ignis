// T7 part 3: the contour drawn over a video is the one for that frame, in image space only.
// Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { ImageContourObservation, VisualObservationBundle } from "../src/contracts/generated";
import { contourAt, frameTolerance, overlayUnits, parseNdjson, polylines } from "../src/research/contours.ts";

const fixture = <T>(p: string): T => JSON.parse(readFileSync(fileURLToPath(new URL(`../../fixtures/contracts/${p}`, import.meta.url)), "utf8")) as T;
const records = fixture<ImageContourObservation[]>("data/image_contours_pixel_only.json");
const bundle = fixture<VisualObservationBundle>("data/visual_bundle_pixel_only.json");

test("NDJSON round trip of the fixture; a bad line is named", () => {
  const text = records.map((r) => JSON.stringify(r)).join("\n") + "\n";
  assert.deepEqual(parseNdjson<ImageContourObservation>(text), records);
  assert.throws(() => parseNdjson("{}\nnot json"), /line 2/);
});

test("a missing edge breaks the line instead of joining across it", () => {
  assert.deepEqual(polylines(records[0]), [[[655, 300]], [[648.5, 320]]], "row 310 has no edge");
  assert.deepEqual(polylines(records[1]), [], "out of field: nothing to draw");
});

test("the contour shown belongs to the painted frame, within half a frame", () => {
  const tol = frameTolerance(records);
  assert.ok(Math.abs(tol - 0.05005) < 1e-9);
  assert.equal(contourAt(records, 49.5495, tol)?.frame, 1485);
  assert.equal(contourAt(records, 49.62, tol)?.frame, 1488);
  assert.equal(contourAt(records, 49.0, tol), null, "before the first contoured frame");
  assert.equal(contourAt(records, 60, tol), null, "long after the last one");
});

test("only image-space bundles are drawn; a metric bundle is refused", () => {
  assert.equal(overlayUnits(bundle), "pixel_only");
  assert.equal(overlayUnits({ ...bundle, units_status: "metric" }), null);
});
