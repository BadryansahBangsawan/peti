export type Theme = "system" | "light" | "dark";

export function parseSavedTheme(value: unknown): Theme {
  if (value === "light" || value === "dark" || value === "system") {
    return value;
  }
  return "system";
}

export function applyTheme(theme: Theme, root: HTMLElement): void {
  if (theme === "system") {
    root.removeAttribute("data-theme");
    return;
  }
  root.setAttribute("data-theme", theme);
}
