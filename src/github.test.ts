import assert from "node:assert/strict";
import { test } from "node:test";
import {
  CONNECTED,
  DENIED,
  DEVICE_PAGE,
  EXPIRED,
  NOT_RUN,
  UNPREPARED,
  deviceHint,
  detailTakes,
  FORK_FAILED,
  forkResultNote,
  githubMessage,
  OWN_REPO,
  pollDelayMs,
  readGithub,
  readTake,
} from "./github.ts";

test("a connected account keeps only a real login and drops a code", () => {
  assert.deepEqual(
    readGithub({
      connected: true,
      login: "octocat",
      userCode: "ABCD-1234",
      interval: 5,
    }),
    { connected: true, login: "octocat", userCode: "", interval: 0 },
  );
  assert.equal(CONNECTED, "Terhubung");
  assert.deepEqual(readGithub({ connected: true, login: "not a name", userCode: "", interval: 0 }), {
    connected: true,
    login: "",
    userCode: "",
    interval: 0,
  });
});

test("a pending code keeps a bounded interval and drops a login", () => {
  assert.deepEqual(
    readGithub({ connected: false, login: "octocat", userCode: "ABCD-1234", interval: 15 }),
    { connected: false, login: "", userCode: "ABCD-1234", interval: 15 },
  );
  assert.equal(readGithub({ connected: false, login: "", userCode: "ABCD-1234", interval: 4 })?.interval, 5);
  assert.equal(
    readGithub({ connected: false, login: "", userCode: "ABCD-1234", interval: 3601 })?.interval,
    5,
  );
  assert.equal(
    readGithub({ connected: false, login: "", userCode: "ABCD-1234", interval: 5.5 })?.interval,
    5,
  );
  assert.deepEqual(readGithub({ connected: false, login: "octocat", userCode: "", interval: 9 }), {
    connected: false,
    login: "",
    userCode: "",
    interval: 0,
  });
});

test("junk never becomes a github row and the device page is fixed", () => {
  assert.equal(readGithub(null), null);
  assert.equal(readGithub("terhubung"), null);
  assert.equal(readGithub([]), null);
  assert.equal(readGithub({ connected: "yes", login: "octocat" }), null);
  assert.equal(readGithub({ connected: false, userCode: "ab", interval: 5 })?.userCode, "");
  assert.equal(readGithub({ connected: false, userCode: "bad code", interval: 5 })?.userCode, "");
  assert.equal(deviceHint(), `Buka ${DEVICE_PAGE} lalu masukkan kode ini.`);
  assert.equal(DEVICE_PAGE, "https://github.com/login/device");
});

test("only the four login sentences pass through", () => {
  assert.equal(githubMessage(UNPREPARED), UNPREPARED);
  assert.equal(githubMessage(DENIED), DENIED);
  assert.equal(githubMessage(EXPIRED), EXPIRED);
  assert.equal(githubMessage(NOT_RUN), NOT_RUN);
  assert.equal(githubMessage("reqwest timeout"), NOT_RUN);
  assert.equal(githubMessage("no such table"), NOT_RUN);
  assert.equal(githubMessage(404), NOT_RUN);
  assert.equal(githubMessage({ message: UNPREPARED }), NOT_RUN);
});

test("detail offers three actions only when connected", () => {
  assert.deepEqual(
    detailTakes(true).map((item) => item.label),
    ["Hanya unduh", "Hanya fork", "Fork, lalu unduh"],
  );
  assert.deepEqual(
    detailTakes(false).map((item) => item.id),
    ["download"],
  );
  assert.equal(
    detailTakes(false).some((item) => item.id === "fork" || item.id === "fork-download"),
    false,
  );
});

test("a fork result is the two sentences or a real name", () => {
  assert.equal(OWN_REPO, "Ini repo kamu. Unduh saja.");
  assert.equal(FORK_FAILED, "Fork tidak dibuat.");
  assert.equal(
    forkResultNote({
      forked: "",
      queued: false,
      needFolder: false,
      ownRepo: true,
      failed: false,
    }),
    OWN_REPO,
  );
  assert.equal(
    forkResultNote({
      forked: "",
      queued: false,
      needFolder: false,
      ownRepo: false,
      failed: true,
    }),
    FORK_FAILED,
  );
  const hostile = readTake({
    forked: "https://evil.test/a/b",
    queued: true,
    needFolder: false,
    ownRepo: false,
    failed: false,
  });
  assert.equal(hostile?.forked, "");
  assert.equal(hostile?.failed, true);
  assert.equal(forkResultNote(hostile!), FORK_FAILED);
  assert.equal(
    readTake({
      forked: "me/Hello-World",
      queued: true,
      needFolder: false,
      ownRepo: false,
      failed: false,
    })?.forked,
    "me/Hello-World",
  );
  const rewritten = readTake({
    forked: "me/notes.git",
    queued: true,
    needFolder: false,
    ownRepo: false,
    failed: false,
  });
  assert.equal(rewritten?.forked, "");
  assert.equal(rewritten?.failed, true);
  assert.equal(forkResultNote(rewritten!), FORK_FAILED);
  assert.equal(readTake(null), null);
  assert.equal(readTake("forked"), null);
});

test("poll delay stays inside five seconds to an hour", () => {
  assert.equal(pollDelayMs(5), 5000);
  assert.equal(pollDelayMs(3600), 3_600_000);
  assert.equal(pollDelayMs(0), 5000);
  assert.equal(pollDelayMs(4), 5000);
  assert.equal(pollDelayMs(3601), 5000);
  assert.equal(pollDelayMs(5.5), 5000);
});
