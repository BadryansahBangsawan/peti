import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { matchAsset, versionLabel } from "../site/release.js";

const dmg = (name: string) => ({ name, browser_download_url: `https://example.test/${name}` });

test("release assets follow the selected installer", () => {
  const assets = [
    dmg("Peti_0.2.0_aarch64.dmg"),
    dmg("Peti_0.2.0_x64.dmg"),
    dmg("Peti_0.2.0_x64-setup.exe"),
    dmg("Peti_0.2.0_arm64-setup.exe"),
    dmg("Peti_0.2.0_amd64.AppImage"),
    dmg("Peti_0.2.0_amd64.deb"),
    dmg("Peti_0.2.0_aarch64.app.tar.gz"),
  ];
  assert.equal(matchAsset(assets, ".dmg", "arm")?.name, "Peti_0.2.0_aarch64.dmg");
  assert.equal(matchAsset(assets, ".dmg", "x64")?.name, "Peti_0.2.0_x64.dmg");
  assert.equal(matchAsset(assets, ".exe", "x64")?.name, "Peti_0.2.0_x64-setup.exe");
  assert.equal(matchAsset(assets, ".exe", "arm")?.name, "Peti_0.2.0_arm64-setup.exe");
  assert.equal(matchAsset(assets, ".appimage", "any")?.name, "Peti_0.2.0_amd64.AppImage");
  assert.equal(matchAsset(assets, ".deb", "any")?.name, "Peti_0.2.0_amd64.deb");
  assert.equal(matchAsset([dmg("Peti_0.2.0_aarch64.dmg")], ".dmg", "x64"), null);
  assert.equal(matchAsset([dmg("Peti.dmg")], ".dmg", "arm")?.name, "Peti.dmg");
  assert.equal(versionLabel({ tag_name: "v0.2.0" }), "v0.2.0");
  assert.equal(versionLabel(null), "");
  assert.equal(versionLabel({}), "");
});

test("the download page stays quiet before a release", () => {
  const html = readFileSync(new URL("../site/unduh.html", import.meta.url), "utf8");
  const css = readFileSync(new URL("../site/styles.css", import.meta.url), "utf8");
  const script = readFileSync(new URL("../site/release.js", import.meta.url), "utf8");
  const page = `${html}\n${css}\n${script}`;
  for (const phrase of ["Data contoh", "Akun contoh", "belum ada rilis", "unduh ulang", "unduh lagi", "client id", "Masuk GitHub belum"]) {
    assert.equal(page.includes(phrase), false, phrase);
  }
  assert.match(page, /api\.github\.com\/repos\/BadryansahBangsawan\/peti\/releases\/latest/);
});
