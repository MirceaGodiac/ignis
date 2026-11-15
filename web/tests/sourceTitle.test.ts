// Search hits read as documents: human titles from PSI stems, marked query terms, one card per file.
import test from "node:test";
import assert from "node:assert/strict";
import type { SearchHit } from "../src/contracts/generated";
import { groupHits, humanSourceName, markTerms, queryTerms } from "../src/research/sourceTitle.ts";

test("PSI file stems become document titles with the investigation in front", () => {
  assert.deepEqual(humanSourceName("PSI-20_Reports_Dissertation-PD-revised_"), { label: "PSI-20 · Dissertation (revised)", title: "Dissertation (revised)", investigation: "PSI-20" });
  assert.deepEqual(humanSourceName("PSI-107_Engineering Documents_SPICE_Phase III Flight Safety Data Package"), {
    label: "SPICE · Phase III Flight Safety Data Package",
    title: "Phase III Flight Safety Data Package",
    investigation: "SPICE",
  });
  assert.equal(humanSourceName("PSI-69_Experimental table_FLEX.csv").label, "FLEX · Experimental table");
  assert.equal(humanSourceName("data/PSI-12_Publications_ACME_Burner-Tube-final.pdf").label, "ACME · Burner Tube (final)");
});

test("hyphens that bind a letter or a number stay; prose titles are kept", () => {
  assert.equal(humanSourceName("PSI-3_Reports_n-heptane-droplet-2-D").title, "n-heptane droplet 2-D");
  assert.equal(humanSourceName("Synthetic fixture publication (not a real paper)").label, "Synthetic fixture publication (not a real paper)");
  assert.equal(humanSourceName("PSI-20").label, "PSI-20");
  assert.equal(humanSourceName("PSI-5_Reports_constructor-toString").title, "constructor toString", "no prototype keys as qualifiers");
  assert.equal(humanSourceName(null).label, "Untitled source");
  assert.equal(humanSourceName("   ").label, "Untitled source");
});

test("query terms mark whole words they start, case-insensitively", () => {
  const terms = queryTerms("Oxygen concentration and the flammability limit");
  assert.deepEqual(terms, ["concentration", "flammability", "oxygen", "limit"]);
  const parts = markTerms("Limiting oxygen: flammability limits at low O2.", terms);
  assert.deepEqual(
    parts.filter((p) => p.match).map((p) => p.text),
    ["Limiting", "oxygen", "flammability", "limits"],
  );
  assert.equal(parts.map((p) => p.text).join(""), "Limiting oxygen: flammability limits at low O2.");
  assert.deepEqual(markTerms("deoxygenated", ["oxygen"]), [{ text: "deoxygenated", match: false }], "only at a word start");
  assert.deepEqual(markTerms("a (b)", queryTerms("(b) [c]")), [{ text: "a (b)", match: false }], "short and special terms are safe");
});

const hit = (rank: number, source: string, page: number): SearchHit => ({
  rank,
  snippet: `s${rank}`,
  title: `PSI-1_Reports_${source}`,
  evidence: { evidence_id: `e${rank}`, source_id: source, evidence_class: "source_derived", origin: "system", locator: { asset_sha256: "0".repeat(64), page } } as SearchHit["evidence"],
});

test("hits of one document gather under its best rank", () => {
  const g = groupHits([hit(1, "A", 3), hit(2, "B", 1), hit(3, "A", 9), hit(4, "A", 3)]);
  assert.deepEqual(
    g.map((x) => [x.sourceId, x.hits.map((h) => h.rank)]),
    [
      ["A", [1, 3, 4]],
      ["B", [2]],
    ],
  );
  assert.equal(g[0].name.title, "A");
});
