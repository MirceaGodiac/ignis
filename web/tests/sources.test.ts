// T7 part 2: how a citation's original is opened from a K1.18 SourceAccess. Run: npm --prefix web test
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { SourceAccess } from "../src/contracts/generated";
import { accessExpired, contentRangeLength, rangeHeader, viewerFor, videoTimeS } from "../src/research/sourceAccess.ts";

const fixture = <T>(p: string): T => JSON.parse(readFileSync(fileURLToPath(new URL(`../../fixtures/contracts/${p}`, import.meta.url)), "utf8")) as T;
const direct = fixture<SourceAccess>("evidence/source_access_direct.json");
const proxy = fixture<SourceAccess>("evidence/source_access_proxy.json");
const linkOut = fixture<SourceAccess>("evidence/source_access_link_out.json");
const loc = { asset_sha256: "0".repeat(64) };

test("a link-out or non-CORS source is never streamed; the locator picks PDF or video", () => {
  assert.equal(viewerFor(linkOut, { ...loc, page: 3 }), "link");
  assert.equal(viewerFor({ ...proxy, cors: false }, { ...loc, page: 3 }), "link");
  assert.equal(viewerFor(proxy, { ...loc, page: 3 }), "pdf");
  assert.equal(viewerFor(direct, { ...loc, pts_s: 1.5 }), "video");
  assert.equal(viewerFor(direct, null), "link");
});

test("a signed URL past expires_at is not used", () => {
  const exp = Date.parse(direct.expires_at!);
  assert.equal(accessExpired(direct, exp - 1), false);
  assert.equal(accessExpired(direct, exp), true);
  assert.equal(accessExpired(proxy, Date.now()), false, "no expiry: proxy URLs do not expire");
});

test("range arithmetic: half-open spans and Content-Range lengths", () => {
  assert.equal(rangeHeader(0, 262144), "bytes=0-262143");
  assert.equal(contentRangeLength("bytes 0-0/49152000"), 49152000);
  assert.equal(contentRangeLength("bytes */1234"), 1234);
  assert.equal(contentRangeLength("bytes 0-0/*"), null, "unknown length");
  assert.equal(contentRangeLength(null), null, "header not exposed by CORS");
  assert.equal(videoTimeS({ ...loc, pts_s: 2.5 }), 2.5);
  assert.equal(videoTimeS({ ...loc, frame: 75 }), null, "a frame index needs a known rate");
});
