// Agent text: bold, safe links and known run names become parts; nothing else is interpreted.
import test from "node:test";
import assert from "node:assert/strict";
import { runNameKey, splitRich } from "../src/shell/richParts.ts";

const index = new Map([
  [runNameKey("FLEX-093"), "PSI-69:193F001:2010-03-23"],
  [runNameKey("PSI-69:20CAL08:2009-03-05"), "PSI-69:20CAL08:2009-03-05"],
  [runNameKey("21088 A1"), "PSI-159:acme_normal:21088A1"],
]);

test("bold, links, runs by NASA name and by id; unknown names stay text", () => {
  const p = splitRich("In **FLEX–093** the flame went out [report](https://ntrs.nasa.gov/x.pdf#page=3); FLEX-9999 and 21088 A1 and PSI-69:20CAL08:2009-03-05.", index);
  const kinds = p.map((x) => x.kind);
  assert.deepEqual(kinds, ["text", "bold", "text", "link", "text", "run", "text", "run", "text"]);
  const bold = p[1] as { kind: "bold"; parts: { kind: string; id?: string }[] };
  assert.equal(bold.parts[0].kind, "run");
  assert.equal(bold.parts[0].id, "PSI-69:193F001:2010-03-23");
  assert.equal((p[3] as { href: string }).href, "https://ntrs.nasa.gov/x.pdf#page=3");
  assert.ok((p[4] as { text: string }).text.includes("FLEX-9999"));
});

test("a non-http link stays literal text", () => {
  const p = splitRich("see [x](javascript:alert(1))", index);
  assert.ok(p.every((x) => x.kind === "text"));
});
