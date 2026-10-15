import assert from "node:assert/strict";
import { test } from "node:test";
import { catalogs, fill, htmlLang, localeOptions, parseLocale, type LocaleId, type Messages } from "./i18n.ts";

const ids: LocaleId[] = ["en", "hi", "zh", "pt", "en-GB", "ja", "id", "de", "ru", "fr"];

function tokens(text: string): string {
  return [...text.matchAll(/\{([a-z]+)\}/g)].map((match) => match[1]).join(",");
}

test("parseLocale keeps ten distinct picker ids and falls back to Indonesian", () => {
  assert.deepEqual(
    localeOptions.map((item) => item.id),
    ids,
  );
  for (const id of ids) assert.equal(parseLocale(id), id);
  for (const junk of ["", "xx", "EN", "en-gb", "en_GB", null, undefined]) {
    assert.equal(parseLocale(junk), "id");
  }
  assert.notEqual(parseLocale("en-GB"), "en");
  assert.equal(htmlLang("zh"), "zh-Hans");
  assert.equal(htmlLang("pt"), "pt-BR");
  assert.equal(htmlLang("en-GB"), "en-GB");
  assert.equal(htmlLang("id"), "id");
});

test("every catalog matches the Indonesian placeholders and stays Peti 0.1.0", () => {
  const source: Messages = catalogs.id;
  const keys = Object.keys(source) as (keyof Messages)[];
  for (const id of ids) {
    const catalog = catalogs[id];
    assert.equal(catalog.version, "Peti 0.1.0", id);
    for (const key of keys) {
      assert.equal(typeof catalog[key], "string", `${id}.${key}`);
      assert.ok(catalog[key].length > 0, `${id}.${key}`);
      assert.equal(tokens(catalog[key]), tokens(source[key]), `${id}.${key}`);
    }
  }
});

test("translated catalogs are not the English object; en-GB shares it", () => {
  for (const id of ["hi", "zh", "pt", "ja", "id", "de", "ru", "fr"] as const) {
    assert.notEqual(catalogs[id], catalogs.en, id);
    assert.notEqual(catalogs[id].homeWelcome, catalogs.en.homeWelcome, id);
  }
  assert.equal(catalogs["en-GB"], catalogs.en);
});

test("fill replaces known tokens and leaves unknown ones", () => {
  assert.equal(fill("{count} dipilih", { count: 3 }), "3 dipilih");
  assert.equal(fill("buka {url} lalu {code}", { url: "github.com/login/device" }), "buka github.com/login/device lalu {code}");
});
