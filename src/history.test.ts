import assert from "node:assert/strict";
import { test } from "node:test";
import { historyName, readHistory, recentHistory, type HistoryRow } from "./history.ts";

const row = (id: number, gitRef = ""): HistoryRow => ({
  id,
  owner: "octocat",
  repo: `repo-${id}`,
  gitRef,
  folder: `/tmp/out/octocat-repo-${id}`,
  finishedAt: "2026-10-14 00:00:00 UTC",
});

test("a name shows the ref only when one was stored", () => {
  assert.equal(historyName(row(1)), "octocat/repo-1");
  assert.equal(historyName(row(1, "main")), "octocat/repo-1@main");
  assert.equal(historyName(row(1, "  ")), "octocat/repo-1");
});

test("blank folders and junk rows never become history", () => {
  const rows = readHistory([
    row(1),
    { ...row(2), folder: "  " },
    { ...row(3), owner: "" },
    { id: "1", owner: "a", repo: "b", gitRef: "", folder: "/tmp", finishedAt: "t" },
    null,
    "selesai",
  ]);
  assert.deepEqual(rows.map((item) => item.id), [1]);
  assert.deepEqual(readHistory({ folder: "/tmp" }), []);
});

test("recent rows keep the given order and stop at five", () => {
  const rows = [6, 5, 4, 3, 2, 1].map((id) => row(id));
  assert.deepEqual(
    recentHistory(rows).map((item) => item.id),
    [6, 5, 4, 3, 2],
  );
  assert.deepEqual(recentHistory([]), []);
});
