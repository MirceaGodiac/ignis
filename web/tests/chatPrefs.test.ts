// The chat list: pinned first, then grouped by last activity; names and removals are the viewer's own.
import test from "node:test";
import assert from "node:assert/strict";
import type { ResearchEpisode } from "../src/contracts/generated.ts";
import { chatTitle, groupChats, rename, setHidden, setPinned, type ChatPrefs } from "../src/shell/chatPrefs.ts";

const now = new Date(2026, 10, 15, 12, 0, 0);
const ago = (days: number) => new Date(now.getTime() - days * 86_400_000).toISOString();
const ep = (id: string, days: number) => ({ episode_id: id, goal: `goal ${id}`, created_at: ago(days), updated_at: ago(days) }) as ResearchEpisode;
const EMPTY: ChatPrefs = { titles: {}, pinned: [], hidden: [] };
const list = [ep("a", 0), ep("b", 1), ep("c", 3), ep("d", 20), ep("e", 90)];
const shape = (p: ChatPrefs) => groupChats(list, p, now).map((g) => [g.label, g.items.map((e) => e.episode_id)]);

test("groups by last activity, newest first, each question once", () => {
  assert.deepEqual(shape(EMPTY), [
    ["Today", ["a"]],
    ["Yesterday", ["b"]],
    ["Previous 7 days", ["c"]],
    ["Previous 30 days", ["d"]],
    ["Older", ["e"]],
  ]);
});

test("a pinned question leads and leaves its date group; a removed one is gone and loses its pin", () => {
  let p = setPinned(EMPTY, "d", true);
  assert.deepEqual(shape(p)[0], ["Pinned", ["d"]]);
  assert.ok(!shape(p).some(([l, ids]) => l !== "Pinned" && (ids as string[]).includes("d")));
  p = setHidden(p, "d", true);
  assert.deepEqual(p.pinned, []);
  assert.ok(!shape(p).flatMap(([, ids]) => ids as string[]).includes("d"));
  assert.ok(shape(setHidden(p, "d", false)).flatMap(([, ids]) => ids as string[]).includes("d"), "undo restores it");
});

test("a name replaces the goal; an empty name restores the question", () => {
  const p = rename(EMPTY, "a", "  Moon-base tests  ");
  assert.equal(chatTitle(p, list[0]), "Moon-base tests");
  assert.equal(chatTitle(rename(p, "a", "   "), list[0]), "goal a");
});
