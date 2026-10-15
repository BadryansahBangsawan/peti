import assert from "node:assert/strict";
import { test } from "node:test";
import { formatSize, nextSpeed, percentOf, readQueue, rowMessage, statusLabel, transferLine } from "./queue.ts";

test("a failed row keeps the stored table message and a waiting row stays blank", () => {
  const rows = readQueue([
    {
      id: 1,
      owner: "octocat",
      repo: "missing",
      gitRef: "",
      status: "gagal",
      error: "Repositori tidak tersedia",
    },
    {
      id: 2,
      owner: "octocat",
      repo: "Hello-World",
      gitRef: "",
      status: "gagal",
      error: "Folder tidak bisa dipakai, pilih yang lain",
    },
    {
      id: 3,
      owner: "octocat",
      repo: "waiting",
      gitRef: "",
      status: "menunggu",
      error: "",
    },
  ]);
  assert.deepEqual(
    rows.map((row) => rowMessage(row)),
    [
      "Repositori tidak tersedia",
      "Folder tidak bisa dipakai, pilih yang lain",
      "",
    ],
  );
});

test("junk, a raw status, and a blank name never become a queue row", () => {
  const rows = readQueue([
    null,
    "404",
    { id: 1, owner: "", repo: "a", gitRef: "", status: "gagal", error: "Repositori tidak tersedia" },
    { id: 2, owner: "a", repo: "b", gitRef: "", status: "HTTP 404", error: "404" },
    { id: "3", owner: "a", repo: "b", gitRef: "", status: "gagal", error: "Repositori tidak tersedia" },
    { folder: "/tmp/out" },
  ]);
  assert.deepEqual(rows, []);
  assert.deepEqual(readQueue({ error: "Repositori tidak tersedia" }), []);
});

test("whitespace is not a message", () => {
  const rows = readQueue([
    { id: 4, owner: "octocat", repo: "paused", gitRef: "main", status: "dijeda", error: "   " },
  ]);
  assert.equal(rows.length, 1);
  assert.equal(rowMessage(rows[0]), "");
  assert.equal(rows[0].status, "dijeda");
  assert.equal(rows[0].bytes, 0);
  assert.equal(rows[0].total, null);
  assert.equal(percentOf(rows[0].bytes, rows[0].total), null);
});

test("percent comes only from a known total and is never a k abbreviation", () => {
  assert.equal(percentOf(10, null), null);
  assert.equal(percentOf(10, 0), null);
  assert.equal(percentOf(-1, 100), null);
  assert.equal(percentOf(50, 100), 50);
  assert.equal(percentOf(1000, 100), 100);
  const shown = String(percentOf(96200, 100000));
  assert.equal(shown.includes("k"), false);
  assert.equal(shown, "96");
  assert.equal(formatSize(96200).includes("k"), false);
  assert.equal(formatSize(96200), "96,2 KB");
});

test("a transfer line uses a real total, a real rate, and never invents either", () => {
  assert.equal(
    transferLine({ status: "berjalan", bytes: 500000, total: 1000000, speed: 50000, error: "" }),
    "50%  ·  500,0 KB / 1,0 MB  ·  50,0 KB/s  ·  sisa 10 detik",
  );
  assert.equal(
    transferLine({ status: "berjalan", bytes: 500000, total: 1000000, speed: null, error: "" }),
    "50%  ·  500,0 KB / 1,0 MB",
  );
  const unknown = transferLine({ status: "berjalan", bytes: 2500, total: null, speed: 1500, error: "" });
  assert.equal(unknown.includes("%"), false);
  assert.equal(unknown, "2,5 KB terunduh  ·  1,5 KB/s  ·  ukuran total belum tersedia");
  assert.equal(
    transferLine({ status: "dijeda", bytes: 500000, total: 1000000, speed: 50000, error: "" }),
    "50%  ·  500,0 KB / 1,0 MB  ·  progres tersimpan",
  );
});

test("a running download keeps its last rate when a poll sees no new bytes", () => {
  const first = nextSpeed("berjalan", 0, 1_000, undefined);
  assert.equal(first.speed, null);
  assert.ok(first.sample);
  const second = nextSpeed("berjalan", 50_000, 2_000, first.sample ?? undefined);
  assert.equal(second.speed, 50_000);
  const stalled = nextSpeed("berjalan", 50_000, 3_000, second.sample ?? undefined);
  assert.equal(stalled.speed, 50_000);
  assert.equal(stalled.sample?.at, 2_000);
  const paused = nextSpeed("dijeda", 50_000, 4_000, stalled.sample ?? undefined);
  assert.equal(paused.speed, null);
  assert.equal(paused.sample, stalled.sample);
  const resumed = nextSpeed("berjalan", 50_000, 5_000, paused.sample ?? undefined);
  assert.equal(resumed.speed, 50_000);
  const later = nextSpeed("berjalan", 80_000, 6_000, resumed.sample ?? undefined);
  assert.equal(later.speed, 7_500);
  const done = nextSpeed("selesai", 80_000, 7_000, later.sample ?? undefined);
  assert.equal(done.sample, null);
  assert.equal(done.speed, null);
});

test("waiting, failed, and saved rows keep their own sentence", () => {
  assert.equal(
    transferLine({ status: "menunggu", bytes: 0, total: null, speed: null, error: "" }),
    "Menunggu slot unduhan tersedia",
  );
  assert.equal(
    transferLine({ status: "menunggu", bytes: 0, total: null, speed: null, error: "Folder tidak bisa dipakai, pilih yang lain" }),
    "Folder tidak bisa dipakai, pilih yang lain",
  );
  assert.equal(
    transferLine({ status: "gagal", bytes: 10, total: 100, speed: 5, error: "Koneksi terputus. Coba unduh lagi." }),
    "Koneksi terputus. Coba unduh lagi.",
  );
  assert.equal(transferLine({ status: "selesai", bytes: 10, total: 10, speed: null, error: "" }), "");
  assert.equal(statusLabel("berjalan"), "Mengunduh");
  assert.equal(statusLabel("dijeda"), "Dijeda");
  assert.equal(statusLabel("menunggu"), "Menunggu");
  assert.equal(statusLabel("gagal"), "Gagal");
  assert.equal(statusLabel("selesai"), "Selesai");
});
