// The run panel lists each NASA file a record cites once, at a place (row/page) when it has one.
import test from "node:test";
import assert from "node:assert/strict";
import { runSources } from "../src/shell/runSources.ts";

const sheet = "99c5262ccb9034b57d592f76a918cae58f07291c176984448c065130f4c23d44";
const table = "37c88ce8ae9391b7544ff630a45ce87543d04d317cdbfbed104fe7d5c27e1b96";

test("one entry per file, preferring a located citation", () => {
  const files = runSources({
    aliases: [
      { name: "FLEX-001", scheme: "file_flex_number", locator: { asset_sha256: sheet, path: "FLEX-001_TestSheet.txt", row: null } },
      { name: "FLEX-001", scheme: "testsheet_header", locator: { asset_sha256: sheet, path: "FLEX-001_TestSheet.txt", row: 1 } },
      { name: "1", scheme: "summary_flex_number", locator: { asset_sha256: table, path: "PSI-69_Experimental table_FLEX.csv", row: 2, column: "FLEX Test #" } },
    ],
    conditions: { oxygen_mole_fraction: { locator: { asset_sha256: table, path: "PSI-69_Experimental table_FLEX.csv", row: 2 } }, pressure: null, bogus: { locator: { asset_sha256: "nothex" } } },
  });
  assert.equal(files.length, 2);
  assert.equal(files[0].sourceId, `sha256:${sheet}`);
  assert.equal(files[0].loc.row, 1);
  assert.deepEqual(files[1].uses, ["name 1", "oxygen mole fraction"]);
});
