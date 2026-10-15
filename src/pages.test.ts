import assert from "node:assert/strict";
import { test } from "node:test";
import { pages, type PageId } from "./pages.ts";

const ids: PageId[] = ["home", "downloads", "discover", "library", "settings"];

test("exactly five pages, in the spec order", () => {
  assert.deepEqual(
    pages.map((page) => page.id),
    ids,
  );
  assert.deepEqual(
    pages.map((page) => page.label),
    ["Home", "Downloads", "Discover", "Library", "Settings"],
  );
});

test("each empty state names what is missing and one real next page", () => {
  for (const page of pages) {
    assert.ok(page.empty.length > 0, page.id);
    assert.equal(page.empty.includes("contoh"), false, page.id);
    assert.equal(/\d/.test(page.empty), false, page.id);
    assert.ok(ids.includes(page.action.target), page.id);
    assert.ok(page.action.label.length > 0, page.id);
  }
});
