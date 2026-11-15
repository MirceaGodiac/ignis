// How answers show numbers to a reader: SI series in a unit that reads at their magnitude, percentile
// axes drawn as one line with a band, and identifiers kept out of legends. Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import { displayUnit, looksLikeId, percentileBand, quantityFromText, quantityLabel, rangeLabel, scaled } from "../src/research/chartRows.ts";
import { seriesName } from "../src/research/viewSpec.ts";
import type { SeriesRef } from "../src/contracts/generated";

test("SI lengths read in the prefix that fits their magnitude", () => {
  assert.deepEqual(displayUnit("m", [3.4e-3, 2.9e-3, null]), { unit: "mm", factor: 1e3 });
  assert.equal(scaled(3.4e-3, 1e3), 3.4);
  assert.equal(displayUnit("m", [4.2e-5]).unit, "µm");
  assert.equal(displayUnit("m", [0.5]).unit, "mm");
  assert.equal(displayUnit("m", [2.5]).unit, "m");
  assert.equal(displayUnit("m", [1200]).unit, "km");
  // m² steps by 10⁶: 1e-7 m²/s (a burning-rate constant) is 0.1 mm²/s, not 1e5 µm²/s
  assert.equal(displayUnit("m^2/s", [1e-7, 6e-8]).unit, "mm²/s");
  assert.equal(displayUnit("m2", [3e-6]).unit, "mm²");
  assert.equal(displayUnit("Pa", [101325, 50000]).unit, "kPa");
  assert.equal(displayUnit("s", [0.004]).unit, "ms");
});

test("units the scaler does not know, and empty or zero series, are shown as given", () => {
  assert.deepEqual(displayUnit("K", [1800]), { unit: "K", factor: 1 });
  assert.deepEqual(displayUnit("1", [0.21]), { unit: "1", factor: 1 });
  assert.deepEqual(displayUnit("m", [0, null]), { unit: "m", factor: 1 });
  assert.deepEqual(displayUnit("mm", [3.4]), { unit: "mm", factor: 1 });
});

test("three percentile axes of one quantity become a median line with a shaded band", () => {
  const b = percentileBand([
    { field: "/median_mm", title: "Median", unit: "mm" },
    { field: "/p05_mm", title: "5th percentile", unit: "mm" },
    { field: "/p95_mm", title: "95th percentile", unit: "mm" },
  ]);
  assert.ok(b);
  assert.deepEqual([b.mid, b.lo, b.hi], [0, 1, 2]);
  assert.equal(b.range, "5th–95th percentile");
  assert.equal(b.midTitle, "Median");
  assert.equal(b.quantity, null);
  // the y title then names the quantity from the view's caption, not the three statistics
  assert.equal(quantityFromText("Forecast of droplet diameter for the methanol run."), "Droplet diameter");
  assert.equal(quantityFromText("Rows of the forecast"), null);
});

test("the band's quantity comes from what the fields share", () => {
  const b = percentileBand([
    { field: "/d_q0.05", title: "Lower", unit: "mm" },
    { field: "/d_q0.5", title: "Central", unit: "mm" },
    { field: "/d_q0.95", title: "Upper", unit: "mm" },
  ]);
  assert.ok(b);
  assert.deepEqual([b.lo, b.mid, b.hi], [0, 1, 2]);
  assert.equal(b.quantity, "Diameter");
  const c = percentileBand([
    { field: "/diameter_upper", title: "Upper bound", unit: "mm" },
    { field: "/diameter_mean", title: "Mean", unit: "mm" },
    { field: "/diameter_lower", title: "Lower bound", unit: "mm" },
  ]);
  assert.ok(c);
  assert.deepEqual([c.lo, c.mid, c.hi], [2, 1, 0]);
  assert.equal(c.range, "Range");
  assert.equal(c.midTitle, "Mean");
  assert.equal(c.quantity, "Diameter");
});

test("axes that are not bounds of one quantity stay separate series", () => {
  // different quantities
  assert.equal(percentileBand([{ field: "/p_kpa", title: "Pressure", unit: "kPa" }, { field: "/o2", title: "O2" }, { field: "/t", title: "Time", unit: "s" }]), null);
  // percentiles in different units
  assert.equal(percentileBand([{ field: "/a", title: "Median", unit: "mm" }, { field: "/b", title: "5th percentile", unit: "m" }, { field: "/c", title: "95th percentile", unit: "mm" }]), null);
  // two axes only
  assert.equal(percentileBand([{ field: "/a", title: "Median", unit: "mm" }, { field: "/b", title: "95th percentile", unit: "mm" }]), null);
  // a central line on its own band's edge is not a band (two medians, one upper)
  assert.equal(percentileBand([{ field: "/a", title: "Median", unit: "mm" }, { field: "/b", title: "Median (run B)", unit: "mm" }, { field: "/c", title: "95th percentile", unit: "mm" }]), null);
});

test("legend names come from the series role, never from an artifact id", () => {
  const ref = (r: Partial<SeriesRef>): SeriesRef => ({ artifact_id: "art_6629c9e5aa01", role: "predicted", time_pointer: "/times_s", value_pointer: "/mean", unit_pointer: "/unit", time_origin_pointer: "/time_origin", ...r });
  assert.equal(seriesName(ref({ label: "art_6629c9e5… (predicted)", value_pointer: "/quantiles/1" }), [0.05, 0.5, 0.95]), "Forecast (median)");
  assert.equal(seriesName(ref({ label: null }), []), "Forecast (mean)");
  assert.equal(seriesName(ref({ role: "observed", label: "art_3427dd…" }), []), "Measured");
  assert.equal(seriesName(ref({ role: "observed", label: "Run 12 (observed)" }), []), "Run 12 (observed)");
  assert.ok(looksLikeId("sha256:ddc6469cf444"));
  assert.ok(!looksLikeId("Experimental handwritten release (not qualified)"));
  assert.equal(quantityLabel("liquid_diameter"), "Droplet diameter");
  assert.equal(rangeLabel(0.05, 0.95), "5–95%");
});
