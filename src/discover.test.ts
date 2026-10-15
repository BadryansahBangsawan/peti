import assert from "node:assert/strict";
import { test } from "node:test";
import {
  applySearchResult,
  descriptionOf,
  EMPTY_DESCRIPTION,
  FAILED_MESSAGE,
  githubHref,
  LIMIT_MESSAGE,
  metaLine,
  periodQuery,
  readDiscover,
  readQueueDiscover,
  repoTitle,
  SEARCH_WAIT_MS,
  selectionLabel,
  type DiscoverCard,
} from "./discover.ts";

const kept: DiscoverCard = {
  owner: "octocat",
  repo: "Hello-World",
  description: " hi ",
  language: "Rust",
  stars: 12,
  forks: 3,
};

const zero: DiscoverCard = {
  owner: "octocat",
  repo: "zero",
  description: "",
  language: "",
  stars: 0,
  forks: 0,
};

test("a search payload keeps real counts and drops invented ones", () => {
  const read = readDiscover({
    repos: [
      kept,
      zero,
      { owner: "octocat", repo: "text", stars: "12", forks: 1 },
      { owner: "octocat", repo: "neg", stars: -1, forks: 1 },
      { owner: "octocat", repo: "float", stars: 1.5, forks: 1 },
      { owner: " ", repo: "blank", stars: 1, forks: 1 },
    ],
    limited: false,
    failed: false,
  });
  assert.equal(read.failed, false);
  assert.equal(read.limited, false);
  assert.deepEqual(read.cards, [kept, zero]);
  assert.equal(repoTitle(kept), "octocat/Hello-World");
  assert.equal(descriptionOf(kept), "hi");
  assert.equal(descriptionOf(zero), EMPTY_DESCRIPTION);
  assert.equal(metaLine(kept), "Rust · 12 star · 3 fork");
  assert.equal(metaLine(zero), "0 star · 0 fork");
  assert.equal(metaLine(kept).includes("skor"), false);
  assert.equal(/\d\s*k\b/i.test(metaLine(kept)), false);
  assert.equal(/\d\s*k\b/i.test(metaLine(zero)), false);
});

test("a broken payload is a failed search, not an empty success", () => {
  for (const value of [null, undefined, "repos", 0]) {
    const read = readDiscover(value);
    assert.equal(read.failed, true, String(value));
    assert.equal(read.limited, false);
    assert.deepEqual(read.cards, []);
  }
  assert.equal(FAILED_MESSAGE, "Pencarian tidak bisa dijalankan.");
  assert.equal(LIMIT_MESSAGE, "Batas GitHub tercapai, coba lagi nanti");
  assert.equal(SEARCH_WAIT_MS, 300);
});

test("quota and failure keep the rows already on screen", () => {
  const current = [kept];
  const stale = applySearchResult(current, 1, 2, {
    cards: [zero],
    limited: false,
    failed: false,
  });
  assert.equal(stale.notice, "none");
  assert.deepEqual(stale.cards, current);

  const limited = applySearchResult(current, 2, 2, {
    cards: [],
    limited: true,
    failed: false,
  });
  assert.equal(limited.notice, "limited");
  assert.deepEqual(limited.cards, current);

  const failed = applySearchResult(current, 2, 2, {
    cards: [],
    limited: false,
    failed: true,
  });
  assert.equal(failed.notice, "failed");
  assert.deepEqual(failed.cards, current);

  const empty = applySearchResult(current, 2, 2, {
    cards: [],
    limited: false,
    failed: false,
  });
  assert.equal(empty.notice, "empty");
  assert.deepEqual(empty.cards, []);

  const fresh = applySearchResult(current, 2, 2, {
    cards: [zero],
    limited: false,
    failed: false,
  });
  assert.equal(fresh.notice, "none");
  assert.deepEqual(fresh.cards, [zero]);
});

test("period falls back to the week window", () => {
  assert.equal(periodQuery("day"), "day");
  assert.equal(periodQuery("month"), "month");
  assert.equal(periodQuery("week"), "week");
  assert.equal(periodQuery("year"), "week");
  assert.equal(periodQuery(""), "week");
});

test("a github link is built only from a real owner and repo", () => {
  assert.equal(githubHref("octocat", "Hello-World"), "https://github.com/octocat/Hello-World");
  assert.equal(githubHref("a", "b"), "https://github.com/a/b");
  for (const [owner, repo] of [
    ["../x", "nope"],
    ["octocat", "a."],
    ["octocat", ".."],
    ["", "Hello-World"],
    ["octo cat", "Hello-World"],
    ["https://evil.test", "Hello-World"],
  ] as const) {
    assert.equal(githubHref(owner, repo), null, `${owner}/${repo}`);
  }
  assert.equal(selectionLabel(0), "0 dipilih");
  assert.equal(selectionLabel(3), "3 dipilih");
});

test("queueing from discover asks for a folder instead of pretending it queued", () => {
  assert.deepEqual(readQueueDiscover({ queued: 2, needFolder: false }), {
    queued: 2,
    needFolder: false,
  });
  assert.deepEqual(readQueueDiscover({ queued: 0, needFolder: true }), {
    queued: 0,
    needFolder: true,
  });
  for (const value of [null, undefined, "queued", { queued: "2", needFolder: false }, { queued: -1, needFolder: false }, { queued: 1 }]) {
    assert.equal(readQueueDiscover(value), null);
  }
});
