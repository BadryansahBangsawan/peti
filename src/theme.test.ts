import assert from "node:assert/strict";
import { test } from "node:test";
import { applyTheme, parseSavedTheme } from "./theme.ts";

function root(): HTMLElement {
  return {
    setAttribute(name: string, value: string) {
      this.theme = name === "data-theme" ? value : this.theme;
    },
    removeAttribute(name: string) {
      if (name === "data-theme") {
        this.theme = null;
      }
    },
    theme: null as string | null,
  } as HTMLElement & { theme: string | null };
}

test("system follows the OS and does not pin a theme", () => {
  const el = root();
  el.theme = "dark";
  applyTheme("system", el);
  assert.equal(el.theme, null);
});

test("an unknown saved theme follows the system", () => {
  assert.equal(parseSavedTheme("blue"), "system");
  assert.equal(parseSavedTheme(null), "system");
  assert.equal(parseSavedTheme("dark"), "dark");
});

test("light and dark pin that theme", () => {
  const el = root();
  applyTheme("light", el);
  assert.equal(el.theme, "light");
  applyTheme("dark", el);
  assert.equal(el.theme, "dark");
});
