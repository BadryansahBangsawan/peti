import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  applySearchResult,
  cardKey,
  descriptionOf,
  FAILED_MESSAGE,
  githubHref,
  LIMIT_MESSAGE,
  periodQuery,
  readDiscover,
  readQueueDiscover,
  repoTitle,
  SEARCH_WAIT_MS,
  selectionLabel,
  type DiscoverCard,
  type DiscoverNotice,
} from "./discover.ts";
import {
  CONNECTED,
  DENIED,
  DEVICE_PAGE,
  EXPIRED,
  FORK_FAILED,
  NOT_RUN,
  OWN_REPO,
  UNPREPARED,
  detailTakes,
  deviceHint,
  forkResultNote,
  githubMessage,
  pollDelayMs,
  readGithub,
  readTake,
  type GithubView,
  type TakeAction,
  type TakeView,
} from "./github.ts";
import { historyName, readHistory, recentHistory, type HistoryRow } from "./history.ts";
import { pages, type PageId } from "./pages.ts";
import {
  formatRate,
  formatSize,
  nextSpeed,
  percentOf,
  readQueue,
  rowMessage,
  statusLabel,
  transferLine,
  type QueueRow,
  type SpeedSample,
} from "./queue.ts";
import { applyTheme, parseSavedTheme, type Theme } from "./theme.ts";

function svg(inner: string, size = 18): string {
  return `<svg viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${inner}</svg>`;
}

const icons: Record<PageId, string> = {
  home: svg(
    `<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-6a2 2 0 0 1 2.582 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>`,
  ),
  downloads: svg(`<path d="M12 17V3"/><path d="m6 11 6 6 6-6"/><path d="M19 21H5"/>`),
  discover: svg(
    `<path d="m16.24 7.76-1.804 5.411a2 2 0 0 1-1.265 1.265L7.76 16.24l1.804-5.411a2 2 0 0 1 1.265-1.265z"/><circle cx="12" cy="12" r="10"/>`,
  ),
  library: svg(
    `<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/><path d="M2 10h20"/>`,
  ),
  settings: svg(`<path d="M14 17H5"/><path d="M19 7h-9"/><circle cx="17" cy="17" r="3"/><circle cx="7" cy="7" r="3"/>`),
};

const plusIcon = `<path d="M5 12h14"/><path d="M12 5v14"/>`;
const linkIcon = `<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>`;
const folderGitIcon = `<path d="M9 20H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H20a2 2 0 0 1 2 2v5"/><circle cx="13" cy="12" r="2"/><path d="M18 19c-2.8 0-5-2.2-5-5v8"/><circle cx="20" cy="19" r="2"/>`;
const pauseIcon = `<rect width="4" height="16" x="6" y="4" rx="1"/><rect width="4" height="16" x="14" y="4" rx="1"/>`;
const cancelIcon = `<path d="M18 6 6 18"/><path d="m6 6 12 12"/>`;
const playIcon = `<path d="M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z"/>`;
const retryIcon = `<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>`;
const downloadIcon = `<path d="M12 17V3"/><path d="m6 11 6 6 6-6"/><path d="M19 21H5"/>`;
const arrowDownIcon = `<path d="M12 5v14"/><path d="m19 12-7 7-7-7"/>`;
const folderCheckIcon = `<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/><path d="m9 13 2 2 4-4"/>`;
const folderOpenIcon = `<path d="m6 14 1.45-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.55 6a2 2 0 0 1-1.94 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"/>`;
const arrowRightIcon = `<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>`;
const arrowUpRightIcon = `<path d="M7 7h10v10"/><path d="M7 17 17 7"/>`;
const searchIcon = `<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>`;
const chevronDownIcon = `<path d="m6 9 6 6 6-6"/>`;
const starIcon = `<path d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"/>`;
const forkIcon = `<circle cx="12" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><circle cx="18" cy="6" r="3"/><path d="M18 9v2c0 .6-.4 1-1 1H7c-.6 0-1-.4-1-1V9"/><path d="M12 12v3"/>`;
const searchXIcon = `<path d="m13.5 8.5-5 5"/><path d="m8.5 8.5 5 5"/><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>`;
const circleCheckIcon = `<circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/>`;
const refreshIcon = `<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/>`;
const clockIcon = `<circle cx="12" cy="12" r="10"/><path d="M12 6v6h4.5"/>`;
const alertIcon = `<circle cx="12" cy="12" r="10"/><path d="M12 8v4"/><path d="M12 16h.01"/>`;
const arrowLeftIcon = `<path d="m12 19-7-7 7-7"/><path d="M19 12H5"/>`;
const folderIcon = `<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>`;
const clipboardPasteIcon = `<path d="M11 14h10"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><path d="M15 2H9a1 1 0 0 0-1 1v2c0 .6.4 1 1 1h6c.6 0 1-.4 1-1V3c0-.6-.4-1-1-1Z"/><path d="m17 10 4 4-4 4"/>`;
const githubMarkIcon = `<path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/>`;
const settingsIcon = `<path d="M14 17H5"/><path d="M19 7h-9"/><circle cx="17" cy="17" r="3"/><circle cx="7" cy="7" r="3"/>`;
const infoIcon = `<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>`;
const copyIcon = `<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>`;
const loaderIcon = `<path d="M21 12a9 9 0 1 1-6.219-8.56"/>`;
const shieldCheckIcon = `<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/><path d="m9 12 2 2 4-4"/>`;

const root = document.querySelector("#app") ?? document.body.appendChild(document.createElement("div"));
if (!root.id) {
  root.id = "app";
}

const shell = document.createElement("div");
shell.className = "app";

const nav = document.createElement("nav");
nav.className = "sidebar";
nav.setAttribute("aria-label", "Utama");

const content = document.createElement("main");
content.className = "content";

const surface = document.createElement("div");
surface.className = "surface";

const toolbar = document.createElement("header");
toolbar.className = "toolbar";

const railButton = document.createElement("button");
railButton.type = "button";
railButton.className = "icon-btn";
railButton.setAttribute("aria-label", "Sidebar");
railButton.innerHTML = svg(`<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/>`, 17);

const crumbSlash = document.createElement("span");
crumbSlash.className = "crumb-slash";
crumbSlash.textContent = "/";

const crumbSection = document.createElement("span");
crumbSection.className = "crumb-section";

const crumbChevron = document.createElement("span");
crumbChevron.className = "crumb-chevron";
crumbChevron.innerHTML = svg(`<path d="m9 18 6-6-6-6"/>`, 14);

const crumbPage = document.createElement("span");
crumbPage.className = "crumb-page";

toolbar.append(railButton, crumbSlash, crumbSection, crumbChevron, crumbPage);

const title = document.createElement("h1");
title.className = "page-title";

const empty = document.createElement("p");
empty.className = "empty";

const action = document.createElement("button");
action.type = "button";
action.className = "action";

const discoverFilters = document.createElement("form");
discoverFilters.className = "discover-page";
discoverFilters.hidden = true;

const searchInput = document.createElement("input");
searchInput.type = "search";
searchInput.autocomplete = "off";

const languageSelect = document.createElement("select");
const topicSelect = document.createElement("select");

const periodSelect = document.createElement("select");
for (const option of [
  { value: "day", label: "Hari ini" },
  { value: "week", label: "Minggu ini" },
  { value: "month", label: "Bulan ini" },
]) {
  const item = document.createElement("option");
  item.value = option.value;
  item.textContent = option.label;
  periodSelect.append(item);
}
periodSelect.value = "week";

const notice = document.createElement("p");
notice.className = "notice";
notice.hidden = true;

const discoverSearch = document.createElement("div");
discoverSearch.className = "discover-search";
const searchGlyph = document.createElement("span");
searchGlyph.className = "paste-icon";
searchGlyph.innerHTML = svg(searchIcon, 17);
searchInput.placeholder = "Cari nama repo, pemilik, atau topik…";
searchInput.setAttribute("aria-label", "Cari");
const searchClear = document.createElement("button");
searchClear.type = "button";
searchClear.className = "search-clear";
searchClear.hidden = true;
searchClear.setAttribute("aria-label", "Hapus pencarian");
searchClear.innerHTML = svg(cancelIcon, 15);
searchClear.addEventListener("click", () => {
  searchInput.value = "";
  if (currentId() !== "discover") {
    return;
  }
  paintPeriod();
  paintDiscoverFilters();
  scheduleDiscover(true);
});
discoverSearch.append(searchGlyph, searchInput, searchClear);

function filterSelect(
  select: HTMLSelectElement,
  label: string,
  empty: string,
  options: { value: string; label: string }[],
): HTMLLabelElement {
  const wrap = document.createElement("label");
  wrap.className = "filter-field";
  select.setAttribute("aria-label", label);
  const blank = document.createElement("option");
  blank.value = "";
  blank.textContent = empty;
  select.append(blank);
  for (const option of options) {
    const item = document.createElement("option");
    item.value = option.value;
    item.textContent = option.label;
    select.append(item);
  }
  select.value = "";
  const chevron = document.createElement("span");
  chevron.className = "filter-chevron";
  chevron.innerHTML = svg(chevronDownIcon, 16);
  wrap.append(select, chevron);
  return wrap;
}

const filterRow = document.createElement("div");
filterRow.className = "filter-row";
const periodBar = document.createElement("div");
periodBar.className = "period-bar";
const periodButtons = new Map<string, HTMLButtonElement>();
for (const option of [
  { value: "day", label: "Hari ini" },
  { value: "week", label: "Minggu ini" },
  { value: "month", label: "Bulan ini" },
]) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "period-btn";
  button.textContent = option.label;
  button.addEventListener("click", () => {
    periodSelect.value = option.value;
    periodSelect.dispatchEvent(new Event("change"));
    paintPeriod();
  });
  periodButtons.set(option.value, button);
  periodBar.append(button);
}
periodSelect.hidden = true;
filterRow.append(
  filterSelect(
    languageSelect,
    "Bahasa",
    "Semua bahasa",
    [
      "TypeScript",
      "JavaScript",
      "Python",
      "Go",
      "Rust",
      "Java",
      "C",
      "C++",
      "C#",
      "Ruby",
      "PHP",
      "Swift",
      "Kotlin",
      "Dart",
      "HTML",
      "CSS",
      "Shell",
      "Vue",
      "Scala",
      "Elixir",
    ].map((name) => ({ value: name, label: name })),
  ),
  filterSelect(topicSelect, "Topik", "Semua topik", [
    { value: "web", label: "Web" },
    { value: "cli", label: "CLI" },
    { value: "api", label: "API" },
    { value: "react", label: "React" },
    { value: "vue", label: "Vue" },
    { value: "android", label: "Android" },
    { value: "ios", label: "iOS" },
    { value: "docker", label: "Docker" },
    { value: "linux", label: "Linux" },
    { value: "database", label: "Database" },
    { value: "machine-learning", label: "Machine learning" },
    { value: "devops", label: "DevOps" },
    { value: "security", label: "Security" },
    { value: "game", label: "Game" },
    { value: "bot", label: "Bot" },
    { value: "library", label: "Library" },
  ]),
  periodBar,
  periodSelect,
);

const resultHead = document.createElement("div");
resultHead.className = "result-head";
resultHead.hidden = true;
const resultTitle = document.createElement("p");
resultTitle.className = "result-title";
const resultCount = document.createElement("span");
resultCount.className = "result-count";
resultCount.hidden = true;
const resultCriteria = document.createElement("p");
resultCriteria.className = "result-criteria";
resultCriteria.textContent = "·  berdasarkan star";
const resultFresh = document.createElement("span");
resultFresh.className = "result-fresh";
resultFresh.append(document.createRange().createContextualFragment(svg(refreshIcon, 13)));
const resultFreshText = document.createElement("span");
resultFreshText.textContent = "Baru diperbarui";
resultFresh.append(resultFreshText);
resultHead.append(resultTitle, resultCount, resultCriteria, resultFresh);

const activeFilters = document.createElement("div");
activeFilters.className = "active-filters";
activeFilters.hidden = true;
const resetFilters = document.createElement("button");
resetFilters.type = "button";
resetFilters.className = "reset-filters";
resetFilters.textContent = "Reset filter";
activeFilters.append(resetFilters);
const filterNote = document.createElement("p");
filterNote.className = "filter-note";
filterNote.hidden = true;
filterNote.append(document.createRange().createContextualFragment(svg(infoIcon, 15)));
const filterNoteText = document.createElement("span");
filterNoteText.textContent = "Hasil memakai kata kunci, bahasa, topik GitHub, dan periode yang dipilih.";
filterNote.append(filterNoteText);

function discoverFiltered(): boolean {
  return (
    searchInput.value.trim().length > 0 ||
    languageSelect.value.trim().length > 0 ||
    topicSelect.value.trim().length > 0 ||
    periodQuery(periodSelect.value) !== "week"
  );
}

function paintDiscoverFilters(): void {
  const filtered = discoverFiltered();
  const hasResults = discoverCards.length > 0;
  searchClear.hidden = searchInput.value.length === 0;
  resetFilters.hidden = !filtered || !hasResults;
  activeFilters.hidden = resetFilters.hidden;
  filterNote.hidden = !filtered || !hasResults || currentId() !== "discover" || detailCard !== null;
}

function resetDiscoverFilters(): void {
  searchInput.value = "";
  languageSelect.value = "";
  topicSelect.value = "";
  periodSelect.value = "week";
  paintPeriod();
  paintDiscoverFilters();
  if (currentId() === "discover") {
    scheduleDiscover(true);
  }
}

resetFilters.addEventListener("click", resetDiscoverFilters);

function paintPeriod(): void {
  const current = periodQuery(periodSelect.value);
  const filtered = discoverFiltered();
  resultTitle.textContent = filtered
    ? "Hasil pencarian"
    : current === "day"
      ? "Populer hari ini"
      : current === "month"
        ? "Populer bulan ini"
        : "Populer minggu ini";
  resultCount.hidden = !filtered;
  if (filtered) {
    resultCount.textContent = `${discoverCards.length} repo`;
  }
  resultCriteria.hidden = filtered;
  resultFresh.hidden = filtered;
  for (const [id, button] of periodButtons) {
    button.setAttribute("aria-pressed", id === current ? "true" : "false");
  }
}
paintPeriod();

function quietAction(icon: string, label: string): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "quiet-btn with-icon";
  button.append(document.createRange().createContextualFragment(svg(icon, 16)));
  const text = document.createElement("span");
  text.textContent = label;
  button.append(text);
  return button;
}

function stateCard(icon: string, eyebrow: string, heading: string, description: string, actionLabel: string, actionIcon: string): {
  card: HTMLElement;
  glyph: HTMLElement;
  eyebrowText: HTMLElement;
  title: HTMLElement;
  description: HTMLElement;
  button: HTMLButtonElement;
} {
  const card = document.createElement("section");
  card.className = "state-card";
  card.hidden = true;
  const brow = document.createElement("p");
  brow.className = "eyebrow";
  const glyph = document.createElement("span");
  glyph.innerHTML = svg(icon, 18);
  const eyebrowText = document.createElement("span");
  eyebrowText.textContent = eyebrow;
  brow.append(glyph, eyebrowText);
  const title = document.createElement("h2");
  title.textContent = heading;
  const body = document.createElement("p");
  body.textContent = description;
  const button = quietAction(actionIcon, actionLabel);
  card.append(brow, title, body, button);
  return { card, glyph, eyebrowText, title, description: body, button };
}

const discoverEmpty = stateCard(
  searchXIcon,
  "Discover / Tidak ada hasil",
  "Belum menemukan repo yang cocok.",
  "Coba kata kunci lain atau atur ulang filter bahasa dan topik.",
  "Reset filter",
  retryIcon,
);
discoverEmpty.button.addEventListener("click", resetDiscoverFilters);

const discoverStatus = stateCard(clockIcon, "Discover / Batas GitHub", "", "", "Coba lagi", refreshIcon);
discoverStatus.button.addEventListener("click", () => {
  if (currentId() === "discover") {
    scheduleDiscover(true);
  }
});

const selectionBar = document.createElement("div");
selectionBar.className = "selection bulk";
selectionBar.hidden = true;
const bulkIcon = document.createElement("span");
bulkIcon.className = "bulk-icon";
bulkIcon.innerHTML = svg(circleCheckIcon, 18);
const selectionCount = document.createElement("span");
selectionCount.className = "bulk-count";
const clearSelected = document.createElement("button");
clearSelected.type = "button";
clearSelected.className = "bulk-clear";
clearSelected.textContent = "Batalkan pilihan";
const addSelected = document.createElement("button");
addSelected.type = "button";
addSelected.className = "bulk-add";
addSelected.textContent = "Tambah ke antrean";
selectionBar.append(bulkIcon, selectionCount, clearSelected, addSelected);

const detail = document.createElement("section");
detail.className = "detail";
detail.hidden = true;

const discoverList = document.createElement("ul");
discoverList.className = "discover";
discoverList.hidden = true;

discoverFilters.append(
  discoverSearch,
  filterRow,
  activeFilters,
  resultHead,
  discoverStatus.card,
  discoverList,
  discoverEmpty.card,
  selectionBar,
  filterNote,
  notice,
);

let discoverTimer: ReturnType<typeof setTimeout> | undefined;
let discoverTicket = 0;
let discoverCards: DiscoverCard[] = [];
let discoverNotice: DiscoverNotice = "none";
const selected = new Map<string, DiscoverCard>();
let detailCard: DiscoverCard | null = null;
let pendingQueue: DiscoverCard[] = [];
let forkNote = "";
let forkResult: { action: Exclude<TakeAction, "download">; view: TakeView } | null = null;
type DetailChoice = TakeAction | "queue";
let detailChoice: DetailChoice = "download";
let addOpen = false;
let downloadDetailId: number | null = null;
let latestSpeeds = new Map<number, number>();
let addAction: TakeAction = "download";
let addAccepted: number | null = null;
let addReviewed = "";
let folderDialog: Promise<string | null> | null = null;
let downloadFolder = "";
let downloadConcurrency = 3;
let homeHistoryCount: number | null = null;
let homeQueueCount: number | null = null;
let queueTail: Promise<void> = Promise.resolve();

const queueList = document.createElement("ul");
queueList.className = "queue";
queueList.hidden = true;

const queueTabs = ["semua", "berjalan", "menunggu", "selesai", "gagal"] as const;
type QueueTab = (typeof queueTabs)[number];
const queueTabLabel: Record<QueueTab, string> = {
  semua: "Semua",
  berjalan: "Berjalan",
  menunggu: "Menunggu",
  selesai: "Selesai",
  gagal: "Gagal",
};
let queueTab: QueueTab = "semua";
let queueRows: QueueRow[] = [];
let doneRows: HistoryRow[] = [];

function inkButton(label: string): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "ink-btn";
  button.innerHTML = svg(plusIcon, 16);
  const text = document.createElement("span");
  text.textContent = label;
  button.append(text);
  return button;
}

function focusPaste(): void {
  pasteInput.focus();
}

const addRepo = inkButton("Tambah repo");
addRepo.classList.add("add-repo");
addRepo.hidden = true;
addRepo.addEventListener("click", () => {
  if (queueTab === "gagal") {
    const failed = queueRows.filter((row) => row.status === "gagal");
    if (failed.length > 0) {
      for (const row of failed) {
        runQueue("queue_retry", row.id);
      }
      return;
    }
  }
  addOpen = true;
  downloadDetailId = null;
  renderQueue(queueRows);
  addLinks.focus();
});

function paintHeadingAction(retryAll: boolean): void {
  const text = addRepo.querySelector("span");
  if (text) {
    text.textContent = retryAll ? "Ulang semua gagal" : "Tambah repo";
  }
  const icon = addRepo.querySelector("svg");
  if (icon) {
    icon.outerHTML = svg(retryAll ? retryIcon : plusIcon, 16);
  }
}

const downloads = document.createElement("section");
downloads.className = "downloads";
downloads.hidden = true;

const pasteForm = document.createElement("form");
pasteForm.className = "paste";
const pasteIcon = document.createElement("span");
pasteIcon.className = "paste-icon";
pasteIcon.innerHTML = svg(linkIcon, 17);
const pasteInput = document.createElement("input");
pasteInput.type = "text";
pasteInput.autocomplete = "off";
pasteInput.spellcheck = false;
pasteInput.placeholder = "Tempel tautan GitHub untuk menambahkan repo…";
pasteInput.setAttribute("aria-label", "Tautan GitHub");
const pasteKey = document.createElement("span");
pasteKey.className = "paste-key";
pasteKey.textContent = "⌘ V";
pasteForm.append(pasteIcon, pasteInput, pasteKey);
pasteForm.addEventListener("submit", (event) => {
  event.preventDefault();
  void submitPaste(pasteInput.value, "");
});

const pasteNote = document.createElement("p");
pasteNote.className = "paste-note";
pasteNote.hidden = true;

const queueEmpty = document.createElement("section");
queueEmpty.className = "queue-empty";
queueEmpty.hidden = true;
const emptyEyebrow = document.createElement("p");
emptyEyebrow.className = "eyebrow";
emptyEyebrow.innerHTML = svg(
  `<path d="M12 17V3"/><path d="m6 11 6 6 6-6"/><path d="M19 21H5"/>`,
  18,
);
const emptyEyebrowText = document.createElement("span");
emptyEyebrowText.textContent = "Downloads / Kosong";
emptyEyebrow.append(emptyEyebrowText);
const emptyTitle = document.createElement("h2");
emptyTitle.textContent = "Repo pertamamu dimulai di sini.";
const emptyDesc = document.createElement("p");
emptyDesc.textContent = "Tempel tautan GitHub untuk menyimpan isi repo ke perangkatmu.";
const emptyAdd = inkButton("Tambah repo");
emptyAdd.addEventListener("click", focusPaste);
queueEmpty.append(emptyEyebrow, emptyTitle, emptyDesc, emptyAdd);

const transfer = document.createElement("div");
transfer.className = "transfer";
transfer.hidden = true;
const transferHead = document.createElement("div");
transferHead.className = "transfer-head";
transferHead.hidden = true;
const liveDot = document.createElement("span");
liveDot.className = "live-dot";
const transferHeadline = document.createElement("p");
transferHeadline.className = "transfer-headline";
const transferSpeed = document.createElement("span");
transferSpeed.className = "transfer-speed";
transferSpeed.hidden = true;
transferSpeed.append(document.createRange().createContextualFragment(svg(arrowDownIcon, 15)));
const transferRate = document.createElement("span");
transferSpeed.append(transferRate);
transferHead.append(liveDot, transferHeadline, transferSpeed);
const transferFoot = document.createElement("div");
transferFoot.className = "transfer-foot";
const transferDetail = document.createElement("p");
transferDetail.className = "transfer-detail";
const pauseAll = quietAction(pauseIcon, "Jeda semua");
pauseAll.disabled = true;
pauseAll.addEventListener("click", () => {
  const ids = queueRows.filter((row) => row.status === "berjalan").map((row) => row.id);
  void Promise.all(
    ids.map((id) =>
      invoke("queue_pause", { id }).catch((err: unknown) => {
        if (typeof err === "string") {
          paintPasteNote(err);
        }
      }),
    ),
  ).then(() => {
    refreshVisibleQueue();
  });
});
transferFoot.append(transferDetail, pauseAll);
transfer.append(transferHead, transferFoot);

const queueTabBar = document.createElement("div");
queueTabBar.className = "queue-tabs";
queueTabBar.hidden = true;
const tabButtons = new Map<QueueTab, HTMLButtonElement>();
for (const id of queueTabs) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "queue-tab";
  const label = document.createElement("span");
  label.textContent = queueTabLabel[id];
  const count = document.createElement("span");
  count.className = "count";
  button.append(label, count);
  button.addEventListener("click", () => {
    queueTab = id;
    renderQueue(queueRows);
  });
  tabButtons.set(id, button);
  queueTabBar.append(button);
}

const doneNotice = document.createElement("div");
doneNotice.className = "done-notice";
doneNotice.hidden = true;
const doneTitle = document.createElement("p");
doneTitle.className = "done-title";
const doneLink = document.createElement("button");
doneLink.type = "button";
doneLink.className = "done-link";
doneLink.textContent = "Lihat semuanya di Library";
doneLink.append(document.createRange().createContextualFragment(svg(arrowRightIcon, 15)));
doneLink.addEventListener("click", () => {
  show("library");
});
doneNotice.append(document.createRange().createContextualFragment(svg(circleCheckIcon, 19)), doneTitle, doneLink);
const doneList = document.createElement("ul");
doneList.className = "done-list";
doneList.hidden = true;
const doneNote = document.createElement("p");
doneNote.className = "done-note";
doneNote.hidden = true;
doneNote.textContent = "Hasil unduhan tetap tersedia di Library setelah kamu berpindah halaman.";
const failBanner = document.createElement("div");
failBanner.className = "fail-banner";
failBanner.hidden = true;
const failTitle = document.createElement("p");
failTitle.className = "fail-banner-title";
const failNote = document.createElement("p");
failNote.className = "fail-banner-note";
failNote.textContent = "Setiap item ditangani sendiri. Mengulang mengunduh repo itu dari awal.";
failBanner.append(failTitle, failNote);
const failRunning = document.createElement("p");
failRunning.className = "fail-running";
failRunning.hidden = true;
downloads.append(pasteForm, queueEmpty, transfer, queueTabBar, failBanner, doneNotice, queueList, doneList, doneNote);
const queueFoot = document.createElement("footer");
queueFoot.className = "queue-foot";
const queueFootPath = document.createElement("span");
queueFootPath.className = "queue-foot-path";
queueFootPath.hidden = true;
queueFootPath.append(document.createRange().createContextualFragment(svg(downloadIcon, 14)));
const queueFootPathText = document.createElement("span");
queueFootPath.append(queueFootPathText);
queueFoot.append(queueFootPath);

const downloadDetail = document.createElement("section");
downloadDetail.className = "download-detail";
downloadDetail.hidden = true;

const addWorkspace = document.createElement("section");
addWorkspace.className = "add-workspace";
addWorkspace.hidden = true;
const addForm = document.createElement("form");
addForm.className = "add-form";
const addLinksLabel = document.createElement("label");
addLinksLabel.className = "add-links-label";
const addLinksTitle = document.createElement("span");
addLinksTitle.textContent = "Tautan repositori";
const addLinksHint = document.createElement("span");
addLinksHint.className = "add-links-hint";
addLinksHint.textContent = "Satu tautan per baris";
const addLinks = document.createElement("textarea");
addLinks.className = "add-links";
addLinks.rows = 6;
addLinks.spellcheck = false;
addLinks.autocomplete = "off";
addLinks.setAttribute("aria-label", "Tautan repositori");
addLinksLabel.append(addLinksTitle, addLinksHint, addLinks);
const addChoiceTitle = document.createElement("p");
addChoiceTitle.className = "add-choice-title";
addChoiceTitle.textContent = "Apa yang ingin kamu lakukan?";
const addChoices = document.createElement("div");
addChoices.className = "choices";
const addChoiceButtons = new Map<TakeAction, HTMLButtonElement>();
for (const choice of [
  { id: "download", label: "Hanya unduh", hint: "Simpan isi repo ke perangkatmu." },
  { id: "fork", label: "Hanya fork", hint: "Buat salinan repo di akun GitHub." },
  { id: "fork-download", label: "Fork, lalu unduh", hint: "Unduh dari fork yang baru dibuat." },
] as const) {
  const button = choiceRow(choice.label, choice.hint, choice.id === "download", () => {
    addAction = choice.id;
    addAccepted = null;
    addReviewed = "";
    paintAddChoices();
    paintAddButton();
  });
  addChoiceButtons.set(choice.id, button);
  addChoices.append(button);
}
const addSave = document.createElement("div");
addSave.className = "add-save";
const addSaveLabel = document.createElement("span");
addSaveLabel.className = "add-save-label";
addSaveLabel.textContent = "Simpan ke";
const addFolderIcon = document.createElement("span");
addFolderIcon.className = "composer-folder";
addFolderIcon.innerHTML = svg(folderIcon, 16);
const addFolderPath = document.createElement("span");
addFolderPath.className = "composer-path";
const addFolderChange = document.createElement("button");
addFolderChange.type = "button";
addFolderChange.className = "folder-change";
addFolderChange.textContent = "Ganti";
addFolderChange.addEventListener("click", () => {
  void chooseFolder();
});
addSave.append(addSaveLabel, addFolderIcon, addFolderPath, addFolderChange);
const addReview = document.createElement("aside");
addReview.className = "add-review";
const addReviewTitle = document.createElement("h2");
addReviewTitle.textContent = "Siap masuk antrean";
const addReviewCount = document.createElement("p");
addReviewCount.className = "add-review-count";
addReviewCount.hidden = true;
const addReviewReasons = document.createElement("div");
addReviewReasons.className = "add-review-reasons";
const addReviewNotes = document.createElement("div");
addReviewNotes.className = "add-review-notes";
for (const line of [
  "Isi repo diunduh sebagai snapshot, tanpa histori Git (.git).",
  "Folder yang sudah ada tetap aman. Peti memakai nama folder baru.",
  "Unduhan dimulai saat slot tersedia.",
]) {
  const note = document.createElement("p");
  note.textContent = line;
  addReviewNotes.append(note);
}
const addSubmit = document.createElement("button");
addSubmit.type = "submit";
addSubmit.className = "ink-btn";
const addSubmitLabel = document.createElement("span");
addSubmitLabel.textContent = "Tambahkan";
addSubmit.append(addSubmitLabel);
addReview.append(addReviewTitle, addReviewCount, addReviewReasons, addReviewNotes, addSubmit);
addForm.append(addLinksLabel, addChoiceTitle, addChoices, addSave);
addForm.addEventListener("submit", (event) => {
  event.preventDefault();
  void submitAdd();
});
const addBack = document.createElement("button");
addBack.type = "button";
addBack.className = "back";
addBack.append(document.createRange().createContextualFragment(svg(arrowLeftIcon, 15)));
const addBackLabel = document.createElement("span");
addBackLabel.textContent = "Kembali ke Downloads";
addBack.append(addBackLabel);
addBack.addEventListener("click", () => {
  addOpen = false;
  renderQueue(queueRows);
});
addWorkspace.append(addBack, addForm, addReview);
downloads.append(queueFoot, failRunning, downloadDetail, addWorkspace);

const home = document.createElement("section");
home.className = "home";
home.hidden = true;

const composer = document.createElement("section");
composer.className = "composer";
const composerTitle = document.createElement("h2");
composerTitle.textContent = "Mulai dengan tautan GitHub.";
const composerForm = document.createElement("form");
composerForm.className = "composer-field";
const composerIcon = document.createElement("span");
composerIcon.className = "paste-icon";
composerIcon.innerHTML = svg(linkIcon, 18);
const composerInput = document.createElement("input");
composerInput.type = "text";
composerInput.autocomplete = "off";
composerInput.spellcheck = false;
composerInput.placeholder = "github.com/pemilik/repo";
composerInput.setAttribute("aria-label", "Tautan GitHub");
const composerKey = document.createElement("span");
composerKey.className = "paste-key";
composerKey.textContent = "⌘ V";
composerForm.append(composerIcon, composerInput, composerKey);
composerForm.addEventListener("submit", (event) => {
  event.preventDefault();
  void submitPaste(composerInput.value, "");
});
const composerActions = document.createElement("div");
composerActions.className = "composer-actions";
const downloadRepo = document.createElement("button");
downloadRepo.type = "button";
downloadRepo.className = "ink-btn";
downloadRepo.innerHTML = svg(downloadIcon, 16);
const downloadRepoLabel = document.createElement("span");
downloadRepoLabel.textContent = "Unduh repo";
downloadRepo.append(downloadRepoLabel);
downloadRepo.addEventListener("click", () => {
  void submitPaste(composerInput.value, "");
});
const composerFolderIcon = document.createElement("span");
composerFolderIcon.className = "composer-folder";
composerFolderIcon.hidden = true;
composerFolderIcon.innerHTML = svg(folderIcon, 16);
const composerPath = document.createElement("span");
composerPath.className = "composer-path";
composerPath.hidden = true;
const composerChange = document.createElement("button");
composerChange.type = "button";
composerChange.className = "folder-change";
composerChange.textContent = "Ganti";
composerChange.addEventListener("click", () => {
  void chooseFolder();
});
const composerSpacer = document.createElement("span");
composerSpacer.className = "composer-spacer";
composerActions.append(composerFolderIcon, composerPath, composerChange, composerSpacer, downloadRepo);
const composerHint = document.createElement("p");
composerHint.className = "composer-hint";
composerHint.textContent = "Punya beberapa repo? Tempel semua tautannya sekaligus.";
composer.append(composerTitle, composerForm, composerActions, composerHint);

const homeStrip = document.createElement("div");
homeStrip.className = "home-strip";
homeStrip.hidden = true;
const homeStripIcon = document.createElement("span");
homeStripIcon.className = "home-strip-icon";
homeStripIcon.innerHTML = svg(downloadIcon, 18);
const homeHeadline = document.createElement("p");
homeHeadline.className = "home-strip-title";
const homeDetail = document.createElement("p");
homeDetail.className = "home-strip-detail";
const homeFail = document.createElement("span");
homeFail.className = "fail-chip";
homeFail.hidden = true;
const homeSpacer = document.createElement("span");
homeSpacer.className = "home-strip-spacer";
const homeOpen = document.createElement("button");
homeOpen.type = "button";
homeOpen.className = "text-link is-green";
const homeOpenLabel = document.createElement("span");
homeOpenLabel.textContent = "Lihat Downloads";
homeOpen.append(homeOpenLabel, document.createRange().createContextualFragment(svg(arrowRightIcon, 16)));
homeOpen.addEventListener("click", () => show("downloads"));
homeStrip.append(homeStripIcon, homeHeadline, homeDetail, homeFail, homeSpacer, homeOpen);

const recentHead = document.createElement("div");
recentHead.className = "recent-head";
const recentTitle = document.createElement("h2");
recentTitle.textContent = "Terakhir diunduh";
const recentSpacer = document.createElement("span");
recentSpacer.className = "recent-spacer";
const openLibrary = document.createElement("button");
openLibrary.type = "button";
openLibrary.className = "text-link";
const openLibraryLabel = document.createElement("span");
openLibraryLabel.textContent = "Buka Library";
openLibrary.append(openLibraryLabel, document.createRange().createContextualFragment(svg(arrowUpRightIcon, 15)));
openLibrary.addEventListener("click", () => show("library"));
recentHead.append(recentTitle, recentSpacer, openLibrary);

const recentList = document.createElement("ul");
recentList.className = "recent";
recentList.hidden = true;

const suggest = document.createElement("div");
suggest.className = "suggest";
const suggestIcon = document.createElement("span");
suggestIcon.className = "suggest-icon";
suggestIcon.innerHTML = svg(
  `<path d="m16.24 7.76-1.804 5.411a2 2 0 0 1-1.265 1.265L7.76 16.24l1.804-5.411a2 2 0 0 1 1.265-1.265z"/><circle cx="12" cy="12" r="10"/>`,
  16,
);
const suggestText = document.createElement("span");
suggestText.textContent = "Belum punya repo pilihan?";
const suggestLink = document.createElement("button");
suggestLink.type = "button";
suggestLink.className = "text-link is-strong";
const suggestLinkLabel = document.createElement("span");
suggestLinkLabel.textContent = "Jelajahi Discover";
suggestLink.append(suggestLinkLabel, document.createRange().createContextualFragment(svg(arrowRightIcon, 14)));
suggestLink.addEventListener("click", () => show("discover"));
suggest.append(suggestIcon, suggestText, suggestLink);

const firstRun = document.createElement("div");
firstRun.className = "first-home";
const firstComposer = document.createElement("section");
firstComposer.className = "first-composer";
const firstTitle = document.createElement("h2");
firstTitle.textContent = "Repo apa yang ingin kamu unduh?";
const firstForm = document.createElement("form");
firstForm.className = "composer-field";
const firstIcon = document.createElement("span");
firstIcon.className = "paste-icon";
firstIcon.innerHTML = svg(linkIcon, 18);
const firstInput = document.createElement("input");
firstInput.type = "text";
firstInput.autocomplete = "off";
firstInput.spellcheck = false;
firstInput.placeholder = "github.com/pemilik/repo";
firstInput.setAttribute("aria-label", "Tautan GitHub");
const firstKey = document.createElement("span");
firstKey.className = "paste-key";
firstKey.textContent = "⌘ V";
firstForm.append(firstIcon, firstInput, firstKey);
firstForm.addEventListener("submit", (event) => {
  event.preventDefault();
  void submitPaste(firstInput.value, "");
});
const firstActions = document.createElement("div");
firstActions.className = "first-actions";
const firstFolderIcon = document.createElement("span");
firstFolderIcon.className = "first-folder-icon";
firstFolderIcon.innerHTML = svg(folderIcon, 16);
const firstPath = document.createElement("span");
firstPath.className = "first-path";
const changeFolder = document.createElement("button");
changeFolder.type = "button";
changeFolder.className = "folder-change";
changeFolder.textContent = "Ganti folder";
changeFolder.addEventListener("click", () => {
  void chooseFolder();
});
const firstSpacer = document.createElement("span");
firstSpacer.className = "first-spacer";
const pasteClipboard = document.createElement("button");
pasteClipboard.type = "button";
pasteClipboard.className = "ink-btn";
pasteClipboard.append(document.createRange().createContextualFragment(svg(clipboardPasteIcon, 16)));
const pasteClipboardLabel = document.createElement("span");
pasteClipboardLabel.textContent = "Tempel dari clipboard";
pasteClipboard.append(pasteClipboardLabel);
pasteClipboard.addEventListener("click", () => {
  void readClipboard();
});
firstActions.append(firstFolderIcon, firstPath, changeFolder, firstSpacer, pasteClipboard);
const firstHint = document.createElement("p");
firstHint.className = "composer-hint";
firstHint.textContent =
  "Bisa satu atau banyak tautan sekaligus. Kamu akan melihat validasi sebelum mulai.";
firstComposer.append(firstTitle, firstForm, firstActions, firstHint);

const firstNext = document.createElement("div");
firstNext.className = "first-next";
const firstSteps = document.createElement("div");
firstSteps.className = "first-steps";
const stepsTitle = document.createElement("h2");
stepsTitle.textContent = "Dari tautan ke folder lokal.";
const stepList = document.createElement("div");
stepList.className = "first-step-list";
for (const step of [
  { icon: linkIcon, text: "Tempel tautan repo publik" },
  { icon: downloadIcon, text: "Pilih folder dan mulai unduh" },
  { icon: folderOpenIcon, text: "Buka hasilnya dari Library" },
]) {
  const row = document.createElement("div");
  row.className = "first-step";
  const icon = document.createElement("span");
  icon.innerHTML = svg(step.icon, 16);
  const label = document.createElement("span");
  label.textContent = step.text;
  row.append(icon, label);
  stepList.append(row);
}
firstSteps.append(stepsTitle, stepList);
const discoverCard = document.createElement("aside");
discoverCard.className = "discover-card";
const compass = document.createElement("span");
compass.className = "discover-card-icon";
compass.innerHTML = svg(
  `<path d="m16.24 7.76-1.804 5.411a2 2 0 0 1-1.265 1.265L7.76 16.24l1.804-5.411a2 2 0 0 1 1.265-1.265z"/><circle cx="12" cy="12" r="10"/>`,
  24,
);
const discoverCardTitle = document.createElement("h2");
discoverCardTitle.textContent = "Belum punya repo pilihan?";
const discoverCardText = document.createElement("p");
discoverCardText.textContent = "Cari proyek lewat bahasa, topik, dan repo populer di Discover.";
const discoverCardButton = document.createElement("button");
discoverCardButton.type = "button";
discoverCardButton.className = "line-btn";
const discoverCardLabel = document.createElement("span");
discoverCardLabel.textContent = "Jelajahi Discover";
discoverCardButton.append(
  discoverCardLabel,
  document.createRange().createContextualFragment(svg(arrowRightIcon, 16)),
);
discoverCardButton.addEventListener("click", () => show("discover"));
discoverCard.append(compass, discoverCardTitle, discoverCardText, discoverCardButton);
firstNext.append(firstSteps, discoverCard);

const firstRule = document.createElement("div");
firstRule.className = "first-rule";
const githubRow = document.createElement("div");
githubRow.className = "github-note";
const githubMark = document.createElement("span");
githubMark.className = "github-note-icon";
githubMark.innerHTML = svg(githubMarkIcon, 21);
const githubCopy = document.createElement("div");
githubCopy.className = "github-note-copy";
const githubLabel = document.createElement("p");
githubLabel.className = "github-note-label";
githubLabel.textContent = "Masuk GitHub hanya jika dibutuhkan.";
const githubDesc = document.createElement("p");
githubDesc.className = "github-note-desc";
githubDesc.textContent = "Unduh repo publik tanpa akun. Hubungkan akun nanti untuk membuat fork.";
githubCopy.append(githubLabel, githubDesc);
const githubSettings = document.createElement("button");
githubSettings.type = "button";
githubSettings.className = "line-btn";
const githubSettingsLabel = document.createElement("span");
githubSettingsLabel.textContent = "Buka Settings";
githubSettings.append(
  document.createRange().createContextualFragment(svg(settingsIcon, 16)),
  githubSettingsLabel,
);
githubSettings.addEventListener("click", () => show("settings"));
githubRow.append(githubMark, githubCopy, githubSettings);
firstRun.append(firstComposer, firstNext, firstRule, githubRow);

home.append(composer, homeStrip, recentHead, recentList, suggest, firstRun);

async function readClipboard(): Promise<void> {
  let text = "";
  try {
    text = await navigator.clipboard.readText();
  } catch {
    firstInput.focus();
    return;
  }
  if (text.trim().length === 0) {
    firstInput.focus();
    return;
  }
  firstInput.value = text;
  void submitPaste(text, "");
}

function paintDownloadFolder(path: string): void {
  downloadFolder = path;
  firstPath.textContent = path;
  composerPath.textContent = path;
  const folderKnown = path.trim().length > 0;
  composerPath.hidden = !folderKnown;
  composerFolderIcon.hidden = !folderKnown;
  changeFolder.textContent = "Ganti folder";
  addFolderPath.textContent = path;
  destinationPath.textContent = path;
  settingsFolderPath.textContent = path;
  queueFootPathText.textContent = path;
  queueFootPath.hidden = !folderKnown;
  if (!folderKnown) {
    queueFoot.hidden = true;
  } else if (currentId() === "downloads" && queueTab !== "gagal") {
    queueFoot.hidden = false;
  }
}

const historyList = document.createElement("ul");
historyList.className = "history";
historyList.hidden = true;

const libraryEmpty = document.createElement("section");
libraryEmpty.className = "library-empty";
libraryEmpty.hidden = true;
const libraryEmptyBody = document.createElement("div");
libraryEmptyBody.className = "library-empty-body";
const libraryEmptyIcon = document.createElement("span");
libraryEmptyIcon.className = "library-empty-icon";
libraryEmptyIcon.innerHTML = svg(folderOpenIcon, 38);
const libraryEmptyTitle = document.createElement("h2");
libraryEmptyTitle.textContent = "Library-mu masih kosong.";
const libraryEmptyText = document.createElement("p");
libraryEmptyText.textContent =
  "Selesaikan unduhan pertama untuk melihat repo, folder penyimpanan, dan waktu selesainya di sini.";
const libraryEmptyActions = document.createElement("div");
libraryEmptyActions.className = "library-empty-actions";
const libraryStart = inkButton("Mulai unduhan");
libraryStart.addEventListener("click", () => {
  show("downloads");
  focusPaste();
});
const libraryDiscover = document.createElement("button");
libraryDiscover.type = "button";
libraryDiscover.className = "line-btn";
libraryDiscover.append(
  document.createRange().createContextualFragment(
    svg(
      `<path d="m16.24 7.76-1.804 5.411a2 2 0 0 1-1.265 1.265L7.76 16.24l1.804-5.411a2 2 0 0 1 1.265-1.265z"/><circle cx="12" cy="12" r="10"/>`,
      16,
    ),
  ),
);
const libraryDiscoverLabel = document.createElement("span");
libraryDiscoverLabel.textContent = "Jelajahi Discover";
libraryDiscover.append(libraryDiscoverLabel);
libraryDiscover.addEventListener("click", () => show("discover"));
libraryEmptyActions.append(libraryStart, libraryDiscover);
libraryEmptyBody.append(libraryEmptyIcon, libraryEmptyTitle, libraryEmptyText, libraryEmptyActions);
const libraryNote = document.createElement("p");
libraryNote.className = "library-note";
libraryNote.append(document.createRange().createContextualFragment(svg(infoIcon, 15)));
const libraryNoteText = document.createElement("span");
libraryNoteText.textContent = "Unduhan yang berjalan, dijeda, atau gagal tetap berada di Downloads.";
libraryNote.append(libraryNoteText);
libraryEmpty.append(libraryEmptyBody, libraryNote);

let historyTicket = 0;

const githubBlock = document.createElement("div");
githubBlock.className = "settings-github";
githubBlock.hidden = true;
const githubGlyph = document.createElement("span");
githubGlyph.className = "settings-github-icon";
githubGlyph.innerHTML = svg(githubMarkIcon, 25);
const settingsGithubCopy = document.createElement("div");
settingsGithubCopy.className = "settings-github-copy";
const githubAccount = document.createElement("p");
githubAccount.className = "settings-github-title";
githubAccount.textContent = "Belum terhubung";
const githubState = document.createElement("p");
githubState.className = "state";
githubState.textContent = "Unduh publik tetap bisa. Masuk untuk membuat fork.";
const githubCode = document.createElement("p");
githubCode.className = "code";
const githubHint = document.createElement("p");
githubHint.className = "hint";
const githubNote = document.createElement("p");
githubNote.className = "note";
const githubButton = document.createElement("button");
githubButton.type = "button";
settingsGithubCopy.append(githubAccount, githubState, githubCode, githubHint, githubNote);
githubBlock.append(githubGlyph, settingsGithubCopy, githubButton);

let githubView: GithubView | null = null;
let githubTimer: ReturnType<typeof setTimeout> | undefined;
let githubTicket = 0;
let deviceFlowOpen = false;

function clearGithubTimer(): void {
  if (githubTimer !== undefined) {
    clearTimeout(githubTimer);
    githubTimer = undefined;
  }
}

paintGithubButton(false);

function paintGithubButton(connected: boolean): void {
  githubButton.className = connected ? "line-btn" : "ink-btn";
  githubButton.replaceChildren();
  if (!connected) {
    githubButton.append(document.createRange().createContextualFragment(svg(arrowUpRightIcon, 16)));
  }
  const label = document.createElement("span");
  label.textContent = connected ? "Keluar" : "Hubungkan GitHub";
  githubButton.append(label);
}

function renderGithub(view: GithubView | null, note: string): void {
  paintAccount();
  const onSettings = currentId() === "settings";
  githubBlock.hidden = !onSettings;
  if (!onSettings) {
    clearGithubTimer();
    paintDeviceFlow();
    return;
  }
  const connected = view?.connected === true;
  if (connected) {
    deviceFlowOpen = false;
  }
  githubBlock.classList.toggle("is-connected", connected);
  githubAccount.textContent = connected && view?.login ? view.login : "Belum terhubung";
  githubState.textContent = connected ? CONNECTED : "Unduh publik tetap bisa. Masuk untuk membuat fork.";
  githubCode.textContent = !connected && view ? view.userCode : "";
  githubHint.textContent = !connected && view?.userCode ? deviceHint() : "";
  githubNote.textContent = note;
  githubButton.hidden = !view && note === NOT_RUN;
  paintGithubButton(connected);
  paintClientRow();
  armGithubPoll(!connected && view?.userCode ? view : null);
  paintDeviceFlow();
}

function armGithubPoll(view: GithubView | null): void {
  clearGithubTimer();
  if (currentId() !== "settings" || !view || view.connected || !view.userCode) {
    return;
  }
  const ticket = githubTicket;
  githubTimer = setTimeout(() => {
    githubTimer = undefined;
    void invoke("github_poll")
      .then((loaded) => {
        if (ticket !== githubTicket || currentId() !== "settings") {
          return;
        }
        const next = readGithub(loaded);
        if (!next) {
          armGithubPoll(view);
          return;
        }
        githubView = next;
        renderGithub(next, "");
      })
      .catch((err: unknown) => {
        if (ticket !== githubTicket || currentId() !== "settings") {
          return;
        }
        applyGithubError(err, true);
      });
  }, pollDelayMs(view.interval));
}

function applyGithubError(err: unknown, keepCode: boolean): void {
  const message = githubMessage(err);
  const stop = message === DENIED || message === EXPIRED || message === UNPREPARED;
  if (stop || !keepCode || !githubView?.userCode) {
    if (stop) {
      githubView = { connected: false, login: "", userCode: "", interval: 0 };
      renderGithub(githubView, message);
      return;
    }
    githubView = null;
    renderGithub(null, message);
    return;
  }
  renderGithub(githubView, message);
}

function loadGithub(): void {
  const ticket = githubTicket;
  const page = currentId();
  void invoke("github_status")
    .then((loaded) => {
      if (ticket !== githubTicket || currentId() !== page) {
        return;
      }
      const view = readGithub(loaded);
      if (!view) {
        return;
      }
      githubView = view;
      paintAccount();
      if (currentId() === "settings") {
        renderGithub(view, "");
      }
      if (currentId() === "discover") {
        renderDetail();
      }
    })
    .catch((err: unknown) => {
      if (ticket !== githubTicket || currentId() !== page) {
        return;
      }
      if (currentId() === "settings") {
        applyGithubError(err, false);
        return;
      }
      githubView = null;
      paintAccount();
      if (currentId() === "discover") {
        renderDetail();
      }
    });
}

const themeGlyph: Record<Theme, string> = {
  system: `<rect width="20" height="14" x="2" y="3" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/>`,
  light: `<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>`,
  dark: `<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>`,
};

const themes: { id: Theme; label: string }[] = [
  { id: "system", label: "Ikuti sistem" },
  { id: "light", label: "Terang" },
  { id: "dark", label: "Gelap" },
];

let theme: Theme = "system";
let themeTouched = false;
applyTheme(theme, document.documentElement);

const themeBlock = document.createElement("fieldset");
themeBlock.className = "theme";
const legend = document.createElement("legend");
legend.textContent = "Tema";
themeBlock.append(legend);

for (const item of themes) {
  const label = document.createElement("label");
  const input = document.createElement("input");
  input.type = "radio";
  input.name = "theme";
  input.value = item.id;
  input.checked = item.id === theme;
  input.addEventListener("change", () => {
    if (!input.checked) {
      return;
    }
    themeTouched = true;
    theme = item.id;
    applyTheme(theme, document.documentElement);
    void invoke("set_saved_theme", { theme }).catch(() => undefined);
  });
  label.className = "theme-option";
  const glyph = document.createElement("span");
  glyph.className = "theme-glyph";
  glyph.innerHTML = svg(themeGlyph[item.id], 18);
  const name = document.createElement("span");
  name.textContent = item.label;
  label.append(glyph, name, input);
  themeBlock.append(label);
}

const pageIntro = document.createElement("div");
pageIntro.className = "page-intro";
pageIntro.append(title);
const pageHead = document.createElement("div");
pageHead.className = "page-head";
pageHead.append(pageIntro, addRepo);

const clientField = document.createElement("input");
clientField.type = "text";
clientField.autocomplete = "off";
clientField.spellcheck = false;
clientField.setAttribute("aria-label", "Client id GitHub");
const settingsNote = document.createElement("p");
settingsNote.className = "settings-note";

function settingsBlock(titleText: string): HTMLElement {
  const section = document.createElement("section");
  section.className = "settings-block";
  const head = document.createElement("div");
  head.className = "settings-head";
  const heading = document.createElement("h2");
  heading.textContent = titleText;
  head.append(heading);
  section.append(head);
  return section;
}

const settingsPage = document.createElement("div");
settingsPage.className = "settings-page";
settingsPage.hidden = true;

const folderSection = settingsBlock("Folder unduhan");
const settingsFolderRow = document.createElement("div");
settingsFolderRow.className = "settings-folder";
const settingsFolderIcon = document.createElement("span");
settingsFolderIcon.className = "settings-folder-icon";
settingsFolderIcon.innerHTML = svg(folderIcon, 18);
const settingsFolderPath = document.createElement("span");
settingsFolderPath.className = "settings-folder-path";
const settingsFolderButton = document.createElement("button");
settingsFolderButton.type = "button";
settingsFolderButton.className = "line-btn";
settingsFolderButton.append(document.createRange().createContextualFragment(svg(folderOpenIcon, 16)));
const settingsFolderLabel = document.createElement("span");
settingsFolderLabel.textContent = "Ganti folder";
settingsFolderButton.append(settingsFolderLabel);
settingsFolderButton.addEventListener("click", () => {
  void chooseFolder();
});
settingsFolderRow.append(settingsFolderIcon, settingsFolderPath, settingsFolderButton);
folderSection.append(settingsFolderRow);

const themeSection = settingsBlock("Tampilan");
themeSection.append(themeBlock);

const githubSection = settingsBlock("GitHub");
githubSection.append(githubBlock);

const clientRow = document.createElement("section");
clientRow.className = "settings-block";
clientRow.hidden = true;
const clientHead = document.createElement("div");
clientHead.className = "settings-head";
const clientTitle = document.createElement("h2");
clientTitle.textContent = "Client id GitHub";
clientHead.append(clientTitle);
clientRow.append(clientHead, clientField);

const settingsFoot = document.createElement("footer");
settingsFoot.className = "settings-foot";
const settingsVersion = document.createElement("span");
settingsVersion.textContent = "Peti 0.1.0";
const settingsLocal = document.createElement("span");
settingsLocal.textContent = "Dibuat untuk repo, disimpan di perangkatmu.";
settingsFoot.append(settingsVersion, settingsLocal);

const deviceFlow = document.createElement("section");
deviceFlow.className = "device-flow";
deviceFlow.hidden = true;
const deviceBack = document.createElement("button");
deviceBack.type = "button";
deviceBack.className = "device-back";
deviceBack.append(document.createRange().createContextualFragment(svg(arrowLeftIcon, 15)));
const deviceBackLabel = document.createElement("span");
deviceBackLabel.textContent = "Kembali ke Settings";
deviceBack.append(deviceBackLabel);
const deviceHead = document.createElement("div");
deviceHead.className = "device-flow-head";
const deviceTitle = document.createElement("h2");
deviceTitle.textContent = "Hubungkan akun GitHub";
deviceHead.append(deviceTitle);
const deviceBody = document.createElement("div");
deviceBody.className = "device-flow-body";
const deviceMain = document.createElement("div");
deviceMain.className = "device-flow-main";
const deviceMark = document.createElement("span");
deviceMark.className = "device-mark";
deviceMark.innerHTML = svg(githubMarkIcon, 38);
const deviceStepTitle = document.createElement("h3");
deviceStepTitle.className = "device-step-title";
deviceStepTitle.textContent = "Masuk lewat browsermu.";
const deviceStepText = document.createElement("p");
deviceStepText.className = "device-step-text";
deviceStepText.textContent = "Salin kode di bawah, buka GitHub, lalu izinkan Peti terhubung ke akunmu.";
const deviceCodeRow = document.createElement("div");
deviceCodeRow.className = "device-code";
const deviceCode = document.createElement("p");
deviceCode.className = "device-code-value";
const deviceCopy = document.createElement("button");
deviceCopy.type = "button";
deviceCopy.className = "line-btn";
deviceCopy.append(document.createRange().createContextualFragment(svg(copyIcon, 16)));
const deviceCopyLabel = document.createElement("span");
deviceCopyLabel.textContent = "Salin kode";
deviceCopy.append(deviceCopyLabel);
deviceCodeRow.append(deviceCode, deviceCopy);
const deviceHintLine = document.createElement("p");
deviceHintLine.className = "device-hint";
const deviceActions = document.createElement("div");
deviceActions.className = "device-actions";
const deviceOpen = document.createElement("a");
deviceOpen.className = "ink-btn";
deviceOpen.href = DEVICE_PAGE;
deviceOpen.target = "_blank";
deviceOpen.rel = "noreferrer";
deviceOpen.append(document.createRange().createContextualFragment(svg(arrowUpRightIcon, 16)));
const deviceOpenLabel = document.createElement("span");
deviceOpenLabel.textContent = "Buka GitHub";
deviceOpen.append(deviceOpenLabel);
deviceActions.append(deviceOpen);
const deviceWait = document.createElement("p");
deviceWait.className = "device-wait";
deviceWait.hidden = true;
deviceWait.append(document.createRange().createContextualFragment(svg(loaderIcon, 16)));
const deviceWaitLabel = document.createElement("span");
deviceWaitLabel.textContent = "Menunggu izin di browser…";
deviceWait.append(deviceWaitLabel);
deviceMain.append(
  deviceMark,
  deviceStepTitle,
  deviceStepText,
  deviceCodeRow,
  deviceHintLine,
  deviceActions,
  deviceWait,
);
const deviceAside = document.createElement("aside");
deviceAside.className = "device-aside";
const deviceAsideTitle = document.createElement("h3");
deviceAsideTitle.textContent = "Setelah terhubung";
function deviceInfo(icon: string, text: string): HTMLDivElement {
  const row = document.createElement("div");
  row.className = "device-aside-row";
  row.append(document.createRange().createContextualFragment(svg(icon, 17)));
  const label = document.createElement("span");
  label.textContent = text;
  row.append(label);
  return row;
}
const deviceAsideRule = document.createElement("div");
deviceAsideRule.className = "device-aside-rule";
const deviceAsideNote = document.createElement("p");
deviceAsideNote.className = "device-aside-note";
deviceAsideNote.textContent =
  "Kamu selalu bisa kembali dan melanjutkan unduhan tanpa menghubungkan akun.";
deviceAside.append(
  deviceAsideTitle,
  deviceInfo(forkIcon, "Buat fork ke akun GitHub-mu"),
  deviceInfo(downloadIcon, "Unduh hasil fork yang dipilih"),
  deviceInfo(shieldCheckIcon, "Pilih sendiri setiap tindakan fork"),
  deviceAsideRule,
  deviceAsideNote,
);
deviceBody.append(deviceMain, deviceAside);
deviceFlow.append(deviceBack, deviceHead, deviceBody);
deviceBack.addEventListener("click", () => {
  deviceFlowOpen = false;
  paintDeviceFlow();
});
deviceCopy.addEventListener("click", () => {
  const code = githubView?.connected === true ? "" : (githubView?.userCode ?? "");
  const clipboard = navigator.clipboard;
  if (!clipboard || code.length === 0 || typeof clipboard.writeText !== "function") {
    return;
  }
  void clipboard.writeText(code).catch(() => undefined);
});

function paintDeviceFlow(): void {
  const code = githubView?.connected === true ? "" : (githubView?.userCode ?? "");
  const open = deviceFlowOpen && code.length > 0 && currentId() === "settings";
  deviceFlow.hidden = !open;
  settingsPage.classList.toggle("is-device", open);
  deviceCode.textContent = open ? code : "";
  deviceHintLine.textContent = open ? deviceHint() : "";
  deviceWait.hidden = !open;
  if (currentId() === "settings") {
    pageHead.hidden = open;
  }
}

settingsPage.append(deviceFlow, folderSection, themeSection, githubSection, clientRow, settingsNote, settingsFoot);
clientField.addEventListener("change", saveClientId);

function paintClientRow(): void {
  clientRow.hidden =
    clientField.value.trim().length === 0 &&
    githubNote.textContent !== UNPREPARED &&
    settingsNote.textContent !== UNPREPARED;
}

content.append(
  pageHead,
  pasteNote,
  home,
  downloads,
  discoverFilters,
  detail,
  empty,
  action,
  libraryEmpty,
  historyList,
  settingsPage,
);
surface.append(toolbar, content);
shell.append(nav, surface);
root.append(shell);

let railForced: boolean | null = null;
const railQuery = window.matchMedia("(max-width: 1100px)");

function applyRail(): void {
  shell.classList.toggle("is-rail", railForced ?? railQuery.matches);
}

railQuery.addEventListener("change", () => {
  if (railForced === null) {
    applyRail();
  }
});
railButton.addEventListener("click", () => {
  railForced = !shell.classList.contains("is-rail");
  applyRail();
});
applyRail();

const buttons = new Map<PageId, HTMLButtonElement>();

function currentId(): PageId {
  const id = location.hash.slice(1);
  return pages.some((page) => page.id === id) ? (id as PageId) : "home";
}

function clearDiscoverTimer(): void {
  if (discoverTimer !== undefined) {
    clearTimeout(discoverTimer);
    discoverTimer = undefined;
  }
}

function renderSelection(): void {
  selectionCount.textContent = selectionLabel(selected.size);
  selectionBar.hidden = currentId() !== "discover" || selected.size === 0;
}

function detailRule(): HTMLDivElement {
  const rule = document.createElement("div");
  rule.className = "detail-rule";
  return rule;
}

function detailChip(text: string): HTMLSpanElement {
  const chip = document.createElement("span");
  chip.className = "chip";
  chip.textContent = text;
  return chip;
}

function detailMetric(icon: string, value: string): HTMLSpanElement {
  const span = document.createElement("span");
  span.className = "detail-metric";
  span.append(document.createRange().createContextualFragment(svg(icon, 15)));
  const text = document.createElement("span");
  text.textContent = value;
  span.append(text);
  return span;
}

function choiceRow(label: string, hint: string, on: boolean, onClick: () => void): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = on ? "choice is-on" : "choice";
  button.setAttribute("aria-pressed", on ? "true" : "false");
  const radio = document.createElement("span");
  radio.className = "radio";
  const copy = document.createElement("span");
  copy.className = "choice-copy";
  const name = document.createElement("span");
  name.className = "choice-label";
  name.textContent = label;
  const note = document.createElement("span");
  note.className = "choice-hint";
  note.textContent = hint;
  copy.append(name, note);
  button.append(radio, copy);
  button.addEventListener("click", onClick);
  return button;
}

function forkReady(view: TakeView): boolean {
  return view.forked.length > 0 && !view.failed && !view.ownRepo;
}

function forkSlugParts(slug: string): { owner: string; repo: string } | null {
  const slash = slug.indexOf("/");
  if (slash <= 0 || slash >= slug.length - 1) {
    return null;
  }
  return { owner: slug.slice(0, slash), repo: slug.slice(slash + 1) };
}

function forkPlace(slug: string): { folder: string; ref: string; waiting: boolean } {
  const parts = forkSlugParts(slug);
  if (!parts) {
    return { folder: "", ref: "", waiting: false };
  }
  const queued = queueRows.find(
    (row) =>
      row.owner === parts.owner &&
      row.repo === parts.repo &&
      (row.status === "menunggu" || row.status === "berjalan"),
  );
  if (!queued) {
    return { folder: "", ref: "", waiting: false };
  }
  return {
    folder: "",
    ref: queued.gitRef.trim(),
    waiting: queued.status === "menunggu",
  };
}

function renderForkResult(
  card: DiscoverCard,
  result: { action: Exclude<TakeAction, "download">; view: TakeView },
): void {
  const view = result.view;
  const back = document.createElement("button");
  back.type = "button";
  back.className = "back";
  back.append(document.createRange().createContextualFragment(svg(arrowLeftIcon, 15)));
  const backLabel = document.createElement("span");
  backLabel.textContent = "Kembali ke detail repo";
  back.append(backLabel);
  back.addEventListener("click", () => {
    forkResult = null;
    forkNote = "";
    renderDetail();
  });
  detail.append(back);
  if (view.queued || result.action === "fork") {
    const intro = document.createElement("div");
    intro.className = "detail-intro";
    const heading = document.createElement("h2");
    heading.textContent = view.queued ? "Fork berhasil. Unduhan siap dimulai." : "Fork berhasil.";
    const desc = document.createElement("p");
    desc.className = "desc";
    const actionLabel = result.action === "fork" ? "Hanya fork" : "Fork, lalu unduh";
    desc.textContent = `Hasil tindakan ${actionLabel} untuk ${repoTitle(card)}.`;
    intro.append(heading, desc);
    detail.append(intro);
  }
  const banner = document.createElement("div");
  banner.className = "fork-banner";
  banner.append(document.createRange().createContextualFragment(svg(circleCheckIcon, 22)));
  const bannerCopy = document.createElement("div");
  bannerCopy.className = "fork-banner-copy";
  const bannerTitle = document.createElement("p");
  bannerTitle.className = "fork-banner-title";
  bannerTitle.textContent = `${view.forked} berhasil dibuat`;
  bannerCopy.append(bannerTitle);
  if (view.queued) {
    const bannerText = document.createElement("p");
    bannerText.className = "fork-banner-text";
    bannerText.textContent = "Unduhan memakai fork milikmu. Repo asal tidak ikut diunduh.";
    bannerCopy.append(bannerText);
  }
  banner.append(bannerCopy);
  const columns = document.createElement("div");
  columns.className = "fork-columns";
  const facts = document.createElement("div");
  facts.className = "fork-facts";
  const factsTitle = document.createElement("h3");
  factsTitle.textContent = "Fork yang dibuat";
  const rows = document.createElement("div");
  rows.className = "fork-rows";
  const place = forkPlace(view.forked);
  const info: [string, string][] = [
    ["Repositori asal", repoTitle(card)],
    ["Fork milikmu", view.forked],
  ];
  if (place.ref.length > 0) {
    info.push(["Branch", place.ref]);
  }
  for (const [label, value] of info) {
    const row = document.createElement("div");
    row.className = "fork-row";
    const name = document.createElement("span");
    name.className = "fork-row-label";
    name.textContent = label;
    const text = document.createElement("span");
    text.className = "fork-row-value";
    text.textContent = value;
    row.append(name, text);
    rows.append(row);
  }
  facts.append(factsTitle, rows);
  const parts = forkSlugParts(view.forked);
  const href = parts ? githubHref(parts.owner, parts.repo) : null;
  const seeDownloads = (): HTMLButtonElement => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "ink-btn";
    button.append(document.createRange().createContextualFragment(svg(arrowRightIcon, 16)));
    const label = document.createElement("span");
    label.textContent = "Lihat Downloads";
    button.append(label);
    button.addEventListener("click", () => {
      show("downloads");
    });
    return button;
  };
  if (href) {
    const actions = document.createElement("div");
    actions.className = "fork-actions";
    const open = document.createElement("a");
    open.className = "line-btn fork-open";
    open.href = href;
    open.rel = "noreferrer";
    open.target = "_blank";
    open.append(document.createRange().createContextualFragment(svg(arrowUpRightIcon, 16)));
    const openLabel = document.createElement("span");
    openLabel.textContent = "Buka fork di GitHub";
    open.append(openLabel);
    actions.append(open);
    facts.append(actions);
  }
  columns.append(facts);
  if (view.queued) {
    const cardSide = document.createElement("aside");
    cardSide.className = "fork-queue";
    const identity = document.createElement("div");
    identity.className = "fork-queue-id";
    identity.append(document.createRange().createContextualFragment(svg(folderGitIcon, 22)));
    const identityName = document.createElement("span");
    identityName.textContent = view.forked;
    identity.append(identityName);
    cardSide.append(identity);
    if (place.waiting) {
      const waiting = document.createElement("span");
      waiting.className = "status-chip";
      waiting.textContent = "Menunggu slot";
      cardSide.append(waiting);
    }
    if (place.folder.length > 0) {
      const folder = document.createElement("div");
      folder.className = "fork-folder";
      const folderLabel = document.createElement("span");
      folderLabel.className = "fork-folder-label";
      folderLabel.textContent = "Folder tujuan";
      const folderPath = document.createElement("span");
      folderPath.className = "fork-folder-path";
      folderPath.textContent = place.folder;
      folder.append(folderLabel, folderPath);
      cardSide.append(folder);
    }
    const rule = document.createElement("div");
    rule.className = "fork-queue-rule";
    const note = document.createElement("p");
    note.className = "fork-queue-note";
    note.textContent =
      "Unduhan dimulai saat slot tersedia. Progres muncul di antrean yang sama dengan repo lainnya.";
    cardSide.append(rule, note, seeDownloads());
    columns.append(cardSide);
  }
  detail.append(banner, columns);
  if (view.queued) {
    const persist = document.createElement("p");
    persist.className = "fork-persist";
    persist.textContent =
      "Hasil unduhan baru masuk Library setelah proses unduh dan ekstraksi selesai.";
    detail.append(persist);
  }
}

function renderDetail(): void {
  detail.replaceChildren();
  const card = detailCard;
  if (!card || currentId() !== "discover") {
    detail.hidden = true;
    pageHead.hidden = false;
    if (currentId() === "discover") {
      discoverFilters.hidden = false;
    }
    return;
  }
  pageHead.hidden = true;
  discoverFilters.hidden = true;
  if (forkResult && forkReady(forkResult.view)) {
    renderForkResult(card, forkResult);
    detail.hidden = false;
    return;
  }
  const back = document.createElement("button");
  back.type = "button";
  back.className = "back";
  back.append(document.createRange().createContextualFragment(svg(arrowLeftIcon, 15)));
  const backLabel = document.createElement("span");
  backLabel.textContent = "Kembali ke Discover";
  back.append(backLabel);
  back.addEventListener("click", () => {
    detailCard = null;
    forkNote = "";
    forkResult = null;
    detailChoice = "download";
    renderDetail();
    renderDiscover(discoverCards, discoverNotice);
  });
  const intro = document.createElement("div");
  intro.className = "detail-intro";
  const heading = document.createElement("h2");
  heading.textContent = repoTitle(card);
  const desc = document.createElement("p");
  desc.className = "desc";
  desc.textContent = descriptionOf(card);
  intro.append(heading, desc);
  const head = document.createElement("div");
  head.className = "detail-head";
  head.append(intro);
  const href = githubHref(card.owner, card.repo);
  if (href) {
    const link = document.createElement("a");
    link.className = "ink-btn";
    link.href = href;
    link.rel = "noreferrer";
    link.target = "_blank";
    link.append(document.createRange().createContextualFragment(svg(arrowUpRightIcon, 16)));
    const linkLabel = document.createElement("span");
    linkLabel.textContent = "Buka di GitHub";
    link.append(linkLabel);
    head.append(link);
  }
  const meta = document.createElement("div");
  meta.className = "detail-meta";
  meta.append(detailChip("Public"));
  const language = card.language.trim();
  if (language.length > 0) {
    meta.append(detailChip(language));
  }
  meta.append(detailMetric(starIcon, `${card.stars} star`), detailMetric(forkIcon, `${card.forks} fork`));
  const workspace = document.createElement("div");
  workspace.className = "detail-workspace";
  const overview = document.createElement("div");
  overview.className = "overview";
  const aboutTitle = document.createElement("h3");
  aboutTitle.className = "about-title";
  aboutTitle.textContent = "Tentang repo";
  const about = document.createElement("p");
  about.className = "desc";
  about.textContent = descriptionOf(card);
  const gets = document.createElement("div");
  gets.className = "gets";
  const getsTitle = document.createElement("h3");
  getsTitle.textContent = "Yang akan kamu dapatkan";
  const getsList = document.createElement("ul");
  for (const [icon, text] of [
    [folderGitIcon, "Snapshot isi repo pada branch yang dipilih"],
    [folderIcon, "Hasil ekstraksi di folder unduhanmu"],
    [infoIcon, "Histori Git dan folder .git tidak disertakan"],
  ] as const) {
    const item = document.createElement("li");
    item.append(document.createRange().createContextualFragment(svg(icon, 16)));
    const label = document.createElement("span");
    label.textContent = text;
    item.append(label);
    getsList.append(item);
  }
  gets.append(getsTitle, getsList);
  overview.append(aboutTitle, about, detailRule(), gets);
  const panel = document.createElement("aside");
  panel.className = "take-panel";
  const panelTitle = document.createElement("h3");
  panelTitle.textContent = "Bawa ke ruang kerjamu";
  const choices = document.createElement("div");
  choices.className = "choices";
  const connected = githubView?.connected === true;
  if (connected) {
    if (detailChoice === "queue") {
      detailChoice = "download";
    }
  } else if (detailChoice === "fork" || detailChoice === "fork-download") {
    detailChoice = "download";
  }
  const hints: Record<DetailChoice, string> = {
    download: "Simpan ke perangkat",
    queue: "Masukkan ke antrean unduhan",
    fork: "Salin ke akun GitHub",
    "fork-download": "Unduh hasil fork milikmu",
  };
  const rows: { id: DetailChoice; label: string }[] = detailTakes(connected).map((take) => ({
    id: take.id,
    label: take.label,
  }));
  if (!connected) {
    rows.push({ id: "queue", label: "Tambah ke antrean" });
  }
  for (const row of rows) {
    choices.append(
      choiceRow(row.label, hints[row.id], detailChoice === row.id, () => {
        chooseDetailAction(card, row.id);
      }),
    );
  }
  const permit = document.createElement("p");
  permit.className = "permit";
  permit.textContent = "Fork hanya dibuat jika kamu memilih tindakan fork.";
  panel.append(panelTitle, detailRule(), choices, detailRule(), permit);
  if (forkNote) {
    const note = document.createElement("p");
    note.className = "fork-note";
    note.textContent = forkNote;
    panel.append(note);
  }
  workspace.append(overview, panel);
  detail.append(back, head, meta, detailRule(), workspace);
  detail.hidden = false;
}

function stillThisCard(card: DiscoverCard): boolean {
  return detailCard !== null && cardKey(detailCard) === cardKey(card) && currentId() === "discover";
}

async function runTake(
  card: DiscoverCard,
  action: Exclude<TakeAction, "download">,
  folder: string,
): Promise<void> {
  if (!stillThisCard(card)) {
    return;
  }
  let loaded: unknown;
  try {
    loaded = await invoke("github_take", {
      owner: card.owner,
      repo: card.repo,
      action,
      folder,
    });
  } catch {
    if (!stillThisCard(card)) {
      return;
    }
    forkResult = null;
    forkNote = FORK_FAILED;
    renderDetail();
    return;
  }
  if (!stillThisCard(card)) {
    return;
  }
  const body = readTake(loaded);
  if (!body) {
    forkResult = null;
    forkNote = FORK_FAILED;
    renderDetail();
    return;
  }
  if (body.needFolder) {
    if (folder.trim().length > 0) {
      return;
    }
    const path = await chooseFolder();
    if (path) {
      await runTake(card, action, path);
    }
    return;
  }
  if (forkReady(body)) {
    forkResult = { action, view: body };
    forkNote = "";
  } else {
    forkResult = null;
    forkNote = forkResultNote(body);
  }
  renderDetail();
}

async function chooseFolder(): Promise<string | null> {
  if (folderDialog) {
    return null;
  }
  const current = downloadFolder.trim();
  const pending = (async (): Promise<string | null> => {
    let picked: string | string[] | null = null;
    try {
      picked = await open({
        directory: true,
        multiple: false,
        title: "Folder unduhan",
        ...(current.length > 0 ? { defaultPath: current } : {}),
      });
    } catch {
      return null;
    }
    if (typeof picked !== "string") {
      return null;
    }
    const folder = picked.trim();
    if (folder.length === 0) {
      return null;
    }
    const concurrency =
      Number.isInteger(downloadConcurrency) && downloadConcurrency >= 1 && downloadConcurrency <= 8
        ? downloadConcurrency
        : 3;
    try {
      await invoke("set_download_settings", { folder, concurrency });
    } catch (err: unknown) {
      const message = typeof err === "string" ? err : "";
      if (currentId() === "settings") {
        settingsNote.textContent = message;
      } else {
        paintPasteNote(message);
      }
      return null;
    }
    downloadConcurrency = concurrency;
    paintDownloadFolder(folder);
    if (currentId() === "settings") {
      settingsNote.textContent = "";
    }
    return folder;
  })();
  folderDialog = pending;
  try {
    return await pending;
  } finally {
    folderDialog = null;
  }
}

function paintPasteNote(text: string): void {
  pasteNote.textContent = text;
  const page = currentId();
  pasteNote.hidden = text.length === 0 || (page !== "home" && page !== "downloads");
}

function readPaste(value: unknown): { needFolder: boolean; queued: number; rejected: string[] } | null {
  if (value === null || typeof value !== "object") {
    return null;
  }
  const body = value as Record<string, unknown>;
  if (typeof body.needFolder !== "boolean") {
    return null;
  }
  const queued =
    typeof body.queued === "number" && Number.isSafeInteger(body.queued) && body.queued >= 0 ? body.queued : 0;
  const rejected: string[] = [];
  if (Array.isArray(body.rejected)) {
    for (const item of body.rejected) {
      if (item !== null && typeof item === "object") {
        const reason = (item as Record<string, unknown>).reason;
        if (typeof reason === "string" && reason.trim().length > 0) {
          rejected.push(reason.trim());
        }
      }
    }
  }
  return { needFolder: body.needFolder, queued, rejected };
}

async function submitPaste(paste: string, folder: string): Promise<void> {
  const text = paste.trim();
  if (text.length === 0) {
    return;
  }
  let loaded: unknown;
  try {
    loaded = await invoke("queue_paste", { paste: text, folder });
  } catch (err: unknown) {
    paintPasteNote(typeof err === "string" ? err : "");
    return;
  }
  const body = readPaste(loaded);
  if (!body) {
    return;
  }
  if (body.needFolder) {
    if (folder.trim().length > 0) {
      return;
    }
    const path = await chooseFolder();
    if (!path) {
      return;
    }
    await submitPaste(text, path);
    return;
  }
  if (pasteInput.value.trim() === text) {
    pasteInput.value = "";
  }
  if (composerInput.value.trim() === text) {
    composerInput.value = "";
  }
  if (firstInput.value.trim() === text) {
    firstInput.value = "";
  }
  paintPasteNote(body.rejected.join("\n"));
  refreshVisibleQueue();
}

const samples = new Map<number, SpeedSample>();
let queueTicket = 0;
let queuePoll: ReturnType<typeof setTimeout> | undefined;

function clearQueuePoll(): void {
  if (queuePoll !== undefined) {
    clearTimeout(queuePoll);
    queuePoll = undefined;
  }
}

function armQueuePoll(rows: QueueRow[]): void {
  clearQueuePoll();
  const page = currentId();
  if (page !== "home" && page !== "downloads") {
    return;
  }
  if (!rows.some((row) => row.status === "menunggu" || row.status === "berjalan")) {
    return;
  }
  const ticket = queueTicket;
  queuePoll = setTimeout(() => {
    queuePoll = undefined;
    if (ticket !== queueTicket) {
      return;
    }
    refreshVisibleQueue();
  }, 1000);
}

function refreshVisibleQueue(): void {
  const page = currentId();
  if (page !== "home" && page !== "downloads") {
    return;
  }
  const ticket = queueTicket;
  void invoke("queue_rows")
    .then((loaded) => {
      if (ticket !== queueTicket) {
        return;
      }
      const now = currentId();
      if (now !== "home" && now !== "downloads") {
        return;
      }
      const rows = readQueue(loaded);
      if (now === "downloads") {
        renderQueue(rows);
      }
      if (now === "home") {
        renderHomeStrip(rows);
      }
      if (now === "home" || now === "downloads") {
        void invoke("history_rows")
          .then((history) => {
            if (ticket !== queueTicket || currentId() !== now) {
              return;
            }
            renderHistory(now, readHistory(history));
          })
          .catch(() => undefined);
      }
      armQueuePoll(rows);
    })
    .catch(() => {
      if (ticket !== queueTicket) {
        return;
      }
      if (currentId() === "downloads") {
        renderQueue([]);
      }
      if (currentId() === "home") {
        renderHomeStrip([]);
      }
    });
}

function runQueue(command: string, id: number): void {
  void invoke(command, { id })
    .then(() => {
      refreshVisibleQueue();
    })
    .catch((err: unknown) => {
      paintPasteNote(typeof err === "string" ? err : "");
      refreshVisibleQueue();
    });
}

function rowSpeed(row: QueueRow, now: number): number | null {
  const next = nextSpeed(row.status, row.bytes, now, samples.get(row.id));
  if (next.sample) {
    samples.set(row.id, next.sample);
  } else {
    samples.delete(row.id);
  }
  return next.speed;
}

let settingsTicket = 0;

function readAppSettings(
  value: unknown,
): { defaultFolder: string; concurrency: number; githubClientId: string } | null {
  if (value === null || typeof value !== "object") {
    return null;
  }
  const body = value as Record<string, unknown>;
  if (typeof body.defaultFolder !== "string") {
    return null;
  }
  if (typeof body.concurrency !== "number" || !Number.isInteger(body.concurrency)) {
    return null;
  }
  if (typeof body.githubClientId !== "string") {
    return null;
  }
  return {
    defaultFolder: body.defaultFolder,
    concurrency: body.concurrency,
    githubClientId: body.githubClientId,
  };
}

function loadAppSettings(): void {
  const ticket = ++settingsTicket;
  void invoke("app_settings")
    .then((loaded) => {
      if (ticket !== settingsTicket) {
        return;
      }
      const body = readAppSettings(loaded);
      if (!body) {
        return;
      }
      downloadConcurrency = body.concurrency;
      paintDownloadFolder(body.defaultFolder);
      if (currentId() !== "settings") {
        return;
      }
      if (document.activeElement !== clientField) {
        clientField.value = body.githubClientId;
      }
      paintClientRow();
    })
    .catch(() => undefined);
}

function saveClientId(): void {
  settingsNote.textContent = "";
  void invoke("set_github_client_id", { clientId: clientField.value })
    .then(() => {
      paintClientRow();
    })
    .catch((err: unknown) => {
      settingsNote.textContent = typeof err === "string" ? err : UNPREPARED;
      paintClientRow();
    });
}

function rememberPending(cards: readonly DiscoverCard[]): void {
  const merged = new Map(pendingQueue.map((card) => [cardKey(card), card]));
  for (const card of cards) {
    merged.set(cardKey(card), { ...card });
  }
  pendingQueue = [...merged.values()];
}

type QueueSend = { queued: number } | { needFolder: true } | { failed: true };

async function sendQueue(cards: DiscoverCard[], chosen: string | null): Promise<QueueSend> {
  if (cards.length === 0) {
    return { queued: 0 };
  }
  if (folderDialog && chosen === null) {
    rememberPending(cards);
    return { needFolder: true };
  }
  if (chosen !== null) {
    rememberPending(cards);
    cards = pendingQueue;
  }
  const folder = chosen ?? downloadFolder.trim();
  let loaded: unknown;
  try {
    loaded = await invoke("queue_discovered", {
      repos: cards.map((card) => ({ owner: card.owner, repo: card.repo })),
      folder,
    });
  } catch {
    return { failed: true };
  }
  const body = readQueueDiscover(loaded);
  if (!body) {
    return { failed: true };
  }
  if (body.needFolder) {
    rememberPending(cards);
    if (chosen !== null) {
      return { needFolder: true };
    }
    const saved = await chooseFolder();
    if (!saved) {
      return { needFolder: true };
    }
    return sendQueue(pendingQueue, saved);
  }
  pendingQueue = [];
  for (const card of cards) {
    selected.delete(cardKey(card));
  }
  renderSelection();
  if (currentId() === "discover" && detailCard === null) {
    renderDiscover(discoverCards, discoverNotice);
  }
  return { queued: body.queued };
}

function queueCards(cards: DiscoverCard[], chosen: string | null = null): Promise<QueueSend> {
  const run = queueTail.then(() => sendQueue(cards, chosen));
  queueTail = run.then(
    () => undefined,
    () => undefined,
  );
  return run;
}

function chooseDetailAction(card: DiscoverCard, choice: DetailChoice): void {
  detailChoice = choice;
  forkNote = "";
  renderDetail();
  if (choice === "fork" || choice === "fork-download") {
    void runTake(card, choice, "");
    return;
  }
  void queueCards([card]).then((outcome) => {
    if (!stillThisCard(card) || detailChoice !== choice) {
      return;
    }
    if ("queued" in outcome) {
      if (choice === "download") {
        detailCard = null;
        detailChoice = "download";
        forkNote = "";
        show("downloads");
        return;
      }
      forkNote = outcome.queued > 0 ? "Masuk antrean unduhan." : "Repo ini sudah ada di antrean.";
      renderDetail();
      return;
    }
    if ("failed" in outcome) {
      forkNote = "Tidak bisa masuk antrean.";
      renderDetail();
    }
  });
}

function renderDiscover(cards: DiscoverCard[], kind: DiscoverNotice): void {
  if (currentId() !== "discover") {
    return;
  }
  discoverNotice = kind;
  discoverList.replaceChildren();
  for (const card of cards) {
    const item = document.createElement("li");
    const box = document.createElement("input");
    box.type = "checkbox";
    box.className = "check";
    const key = cardKey(card);
    box.setAttribute("aria-label", `Pilih ${repoTitle(card)}`);
    box.checked = selected.has(key);
    box.addEventListener("change", () => {
      if (box.checked) {
        selected.set(key, card);
      } else {
        selected.delete(key);
      }
      renderSelection();
    });
    const name = document.createElement("button");
    name.type = "button";
    name.className = "name";
    name.textContent = repoTitle(card);
    name.addEventListener("click", () => {
      forkNote = "";
      forkResult = null;
      detailChoice = "download";
      detailCard = card;
      renderDetail();
    });
    const info = document.createElement("div");
    info.className = "repo-info";
    const desc = document.createElement("p");
    desc.className = "desc";
    desc.textContent = descriptionOf(card);
    const meta = document.createElement("div");
    meta.className = "repo-meta";
    const language = card.language.trim();
    if (language.length > 0) {
      const lang = document.createElement("span");
      lang.className = "lang";
      const dot = document.createElement("span");
      dot.className = "lang-dot";
      const langText = document.createElement("span");
      langText.textContent = language;
      lang.append(dot, langText);
      meta.append(lang);
    }
    meta.append(metric(starIcon, String(card.stars)), metric(forkIcon, String(card.forks)));
    info.append(name, desc, meta);
    const download = quietAction(downloadIcon, "Unduh");
    download.addEventListener("click", () => {
      void queueCards([card]);
    });
    item.append(box, info, download);
    discoverList.append(item);
  }
  discoverList.hidden = cards.length === 0;
  resultHead.hidden = cards.length === 0;
  paintPeriod();
  paintDiscoverFilters();
  notice.textContent = "";
  notice.hidden = true;
  const limited = kind === "limited";
  const failed = kind === "failed";
  discoverStatus.card.hidden = !limited && !failed;
  if (limited || failed) {
    discoverStatus.glyph.innerHTML = svg(limited ? clockIcon : alertIcon, 18);
    discoverStatus.eyebrowText.textContent = limited ? "Discover / Batas GitHub" : "Discover";
    discoverStatus.title.textContent = limited ? LIMIT_MESSAGE : FAILED_MESSAGE;
    discoverStatus.description.textContent = limited
      ? "Coba setelah kuota direset. Unduhan dalam antrean tetap berjalan."
      : "";
    discoverStatus.description.hidden = !limited;
  }
  discoverEmpty.card.hidden = kind !== "empty";
  empty.hidden = true;
  action.hidden = true;
}

function metric(icon: string, value: string): HTMLSpanElement {
  const span = document.createElement("span");
  span.className = "metric";
  span.append(document.createRange().createContextualFragment(svg(icon, 14)));
  const text = document.createElement("span");
  text.textContent = value;
  span.append(text);
  return span;
}

function paintDiscover(ticket: number, incoming: ReturnType<typeof readDiscover>): void {
  if (currentId() !== "discover" || ticket !== discoverTicket) {
    return;
  }
  const result = applySearchResult(discoverCards, ticket, discoverTicket, incoming);
  discoverCards = result.cards;
  renderDiscover(result.cards, result.notice);
}

function scheduleDiscover(immediate: boolean): void {
  clearDiscoverTimer();
  const ticket = ++discoverTicket;
  const start = () => {
    discoverTimer = undefined;
    const text = searchInput.value;
    const language = languageSelect.value;
    const topic = topicSelect.value;
    const period = periodQuery(periodSelect.value);
    void invoke("discover_search", { text, language, topic, period })
      .then((loaded) => {
        paintDiscover(ticket, readDiscover(loaded));
      })
      .catch(() => {
        paintDiscover(ticket, { cards: [], limited: false, failed: true });
      });
  };
  if (immediate) {
    start();
    return;
  }
  discoverTimer = setTimeout(start, SEARCH_WAIT_MS);
}

function rowIconButton(inner: string, label: string, onClick?: () => void): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "row-icon";
  button.disabled = onClick === undefined;
  button.setAttribute("aria-label", label);
  button.innerHTML = svg(inner, 16);
  if (onClick) {
    button.addEventListener("click", onClick);
  }
  return button;
}

function paintDone(open: boolean): void {
  doneNotice.hidden = !open;
  doneList.hidden = !open;
  doneNote.hidden = !open;
  doneList.replaceChildren();
  if (!open) {
    return;
  }
  doneTitle.textContent = `${doneRows.length} repo siap dibuka`;
  for (const row of doneRows) {
    const item = document.createElement("li");
    const icon = document.createElement("span");
    icon.className = "done-icon";
    icon.innerHTML = svg(folderCheckIcon, 23);
    const info = document.createElement("div");
    info.className = "done-info";
    const name = document.createElement("span");
    name.className = "name";
    name.textContent = `${row.owner} / ${row.repo}`;
    const path = document.createElement("span");
    path.className = "path";
    path.textContent = row.folder;
    const meta = document.createElement("span");
    meta.className = "meta";
    const ref = row.gitRef.trim();
    meta.textContent = ref.length > 0 ? `${recentWhen(row.finishedAt)}  ·  ${ref}` : recentWhen(row.finishedAt);
    info.append(name, path, meta);
    const status = document.createElement("span");
    status.className = "status-chip is-selesai";
    status.textContent = "Selesai";
    const openFolder = quietAction(folderOpenIcon, "Buka folder");
    const note = document.createElement("span");
    note.className = "open-error";
    openFolder.addEventListener("click", () => {
      note.textContent = "";
      void invoke("open_history_folder", { id: row.id }).catch((err: unknown) => {
        note.textContent =
          typeof err === "string" && err.length > 0 ? err : "Folder tidak bisa dipakai, pilih yang lain";
      });
    });
    item.append(icon, info, status, openFolder, note);
    doneList.append(item);
  }
}

function failQueueRow(row: QueueRow): HTMLLIElement {
  const item = document.createElement("li");
  item.className = "fail-row";
  const body = document.createElement("div");
  body.className = "repo-body";
  const name = document.createElement("button");
  name.type = "button";
  name.className = "name";
  name.textContent = `${row.owner} / ${row.repo}`;
  name.addEventListener("click", () => {
    openDownloadDetail(row.id);
  });
  const reason = document.createElement("span");
  reason.className = "error is-failed";
  const message = rowMessage(row);
  reason.textContent = message;
  body.append(name, reason);
  const status = document.createElement("span");
  status.className = "status-chip is-gagal";
  status.textContent = statusLabel(row.status);
  const actions = document.createElement("span");
  actions.className = "row-actions";
  const inspect = message === "Repositori tidak tersedia";
  const button = document.createElement("button");
  button.type = "button";
  button.className = "retry-btn";
  button.append(document.createRange().createContextualFragment(svg(inspect ? linkIcon : retryIcon, 16)));
  const label = document.createElement("span");
  label.textContent = inspect ? "Periksa tautan" : "Ulang";
  button.append(label);
  button.addEventListener("click", () => {
    if (inspect) {
      pasteForm.hidden = false;
      focusPaste();
      return;
    }
    runQueue("queue_retry", row.id);
  });
  actions.append(button);
  item.dataset.key = queueRowKey(row);
  item.append(body, status, actions);
  return item;
}

function pasteName(value: string): boolean {
  if (value.length === 0 || value === "." || value === ".." || value.endsWith(".")) {
    return false;
  }
  const first = value[0] ?? "";
  if (!/^[A-Za-z0-9]$/.test(first)) {
    return false;
  }
  return /^[A-Za-z0-9._-]+$/.test(value);
}

function splitPaste(input: string): { accepted: { owner: string; repo: string }[]; rejected: string[] } {
  const accepted: { owner: string; repo: string }[] = [];
  const rejected: string[] = [];
  for (const token of input.split(/\s+/)) {
    if (token.length === 0) {
      continue;
    }
    const invalid = `Tautan tidak valid, ${token}`;
    const withoutFragment = token.split("#", 1)[0] ?? token;
    const withoutQuery = withoutFragment.split("?", 1)[0] ?? withoutFragment;
    const schemeAt = withoutQuery.indexOf("://");
    let rest = withoutQuery;
    if (schemeAt >= 0) {
      const scheme = withoutQuery.slice(0, schemeAt);
      if (scheme.toLowerCase() !== "http" && scheme.toLowerCase() !== "https") {
        rejected.push(invalid);
        continue;
      }
      rest = withoutQuery.slice(schemeAt + 3);
    } else if (withoutQuery.startsWith("git@")) {
      rejected.push(invalid);
      continue;
    }
    const slash = rest.indexOf("/");
    if (slash <= 0) {
      rejected.push(invalid);
      continue;
    }
    const host = rest.slice(0, slash);
    if (host.toLowerCase() !== "github.com" && host.toLowerCase() !== "www.github.com") {
      rejected.push(invalid);
      continue;
    }
    const segments = rest
      .slice(slash + 1)
      .split("/")
      .filter((segment) => segment.length > 0);
    if (segments.length < 2) {
      rejected.push(invalid);
      continue;
    }
    const owner = segments[0] ?? "";
    let repo = segments[1] ?? "";
    if (repo.endsWith(".git")) {
      repo = repo.slice(0, -4);
    }
    if (!pasteName(owner) || !pasteName(repo)) {
      rejected.push(invalid);
      continue;
    }
    const extra = segments.slice(2);
    const refOk =
      extra.length === 0 ||
      (extra[0] === "tree" &&
        extra.length > 1 &&
        extra.slice(1).every((segment) => segment !== "." && segment !== ".." && !/[\u0000-\u001f]/.test(segment)));
    if (!refOk) {
      rejected.push(invalid);
      continue;
    }
    accepted.push({ owner, repo });
  }
  return { accepted, rejected };
}

function paintAddChoices(): void {
  for (const [id, button] of addChoiceButtons) {
    const on = id === addAction;
    button.className = on ? "choice is-on" : "choice";
    button.setAttribute("aria-pressed", on ? "true" : "false");
  }
}

function paintAddButton(): void {
  const reviewed = addLinks.value === addReviewed && addAccepted !== null && addAccepted > 0;
  addSubmitLabel.textContent = reviewed ? `Tambahkan ${addAccepted} repo` : "Tambahkan";
}

function paintAddReview(queued: number, reasons: readonly string[]): void {
  addReviewCount.hidden = queued <= 0;
  if (queued > 0) {
    addReviewCount.textContent = `${queued} tautan siap ditambahkan`;
  }
  addReviewReasons.replaceChildren();
  for (const reason of reasons) {
    if (reason.trim().length === 0) {
      continue;
    }
    const line = document.createElement("p");
    line.textContent = reason.trim();
    addReviewReasons.append(line);
  }
  paintAddButton();
}

function rememberAddReview(text: string, queued: number, reasons: readonly string[]): void {
  addReviewed = text;
  addAccepted = queued;
  paintAddReview(queued, reasons);
}

async function submitAdd(asked = false): Promise<void> {
  const text = addLinks.value.trim();
  if (text.length === 0 || currentId() !== "downloads") {
    return;
  }
  const folder = downloadFolder.trim();
  if (addAction === "download") {
    let loaded: unknown;
    try {
      loaded = await invoke("queue_paste", { paste: text, folder });
    } catch (err: unknown) {
      rememberAddReview(addLinks.value, 0, [typeof err === "string" ? err : ""]);
      return;
    }
    const body = readPaste(loaded);
    if (!body) {
      return;
    }
    if (body.needFolder) {
      if (asked) {
        rememberAddReview(addLinks.value, 0, body.rejected);
        return;
      }
      const path = await chooseFolder();
      if (!path) {
        return;
      }
      await submitAdd(true);
      return;
    }
    rememberAddReview(addLinks.value, body.queued, body.rejected);
    refreshVisibleQueue();
    return;
  }
  const parsed = splitPaste(text);
  const reasons = [...parsed.rejected];
  let accepted = 0;
  for (const link of parsed.accepted) {
    let loaded: unknown;
    try {
      loaded = await invoke("github_take", {
        owner: link.owner,
        repo: link.repo,
        action: addAction,
        folder,
      });
    } catch {
      reasons.push(FORK_FAILED);
      continue;
    }
    const body = readTake(loaded);
    if (!body) {
      reasons.push(FORK_FAILED);
      continue;
    }
    if (body.needFolder) {
      if (asked) {
        reasons.push(FORK_FAILED);
        continue;
      }
      const path = await chooseFolder();
      if (!path) {
        rememberAddReview(addLinks.value, accepted, reasons);
        return;
      }
      await submitAdd(true);
      return;
    }
    if (body.ownRepo) {
      reasons.push(OWN_REPO);
      continue;
    }
    if (body.failed || body.forked.length === 0 || (addAction === "fork-download" && !body.queued)) {
      reasons.push(FORK_FAILED);
      continue;
    }
    if (body.queued) {
      accepted += 1;
      continue;
    }
    reasons.push(`${body.forked} dibuat`);
  }
  rememberAddReview(addLinks.value, accepted, reasons);
  if (addAction === "fork-download") {
    refreshVisibleQueue();
  }
}

function factRow(label: string, value: string): HTMLDivElement {
  const row = document.createElement("div");
  row.className = "fact";
  const name = document.createElement("span");
  name.className = "fact-label";
  name.textContent = label;
  const text = document.createElement("span");
  text.className = "fact-value";
  text.textContent = value;
  row.append(name, text);
  return row;
}

function openDownloadDetail(id: number): void {
  addOpen = false;
  downloadDetailId = id;
  renderQueue(queueRows);
}

function downloadFigures(row: QueueRow): string {
  const percent = percentOf(row.bytes, row.total);
  const speed = latestSpeeds.get(row.id) ?? null;
  const figureParts: string[] = [];
  if (percent !== null) {
    figureParts.push(`${percent}%`);
  }
  if (percent !== null && row.total !== null) {
    figureParts.push(`${formatSize(row.bytes)} / ${formatSize(row.total)}`);
  } else if (row.bytes > 0) {
    figureParts.push(`${formatSize(row.bytes)} terunduh`);
  }
  if (row.status === "berjalan" && speed !== null && speed >= 1) {
    figureParts.push(formatRate(speed));
    if (percent !== null && row.total !== null && row.total > row.bytes) {
      const seconds = Math.ceil((row.total - row.bytes) / speed);
      figureParts.push(`sisa ${seconds} detik`);
    }
  }
  return figureParts.join("  ·  ");
}

function paintDownloadDetail(row: QueueRow): void {
  const key = `${row.id}\t${row.status}\t${row.error}`;
  const text = downloadFigures(row);
  const existing = downloadDetail.querySelector(".download-figures");
  if (downloadDetail.dataset.key === key && (text.length === 0) === !existing) {
    if (existing) {
      existing.textContent = text;
    }
    downloadDetail.hidden = false;
    return;
  }
  downloadDetail.dataset.key = key;
  downloadDetail.replaceChildren();
  const back = document.createElement("button");
  back.type = "button";
  back.className = "back";
  back.append(document.createRange().createContextualFragment(svg(arrowLeftIcon, 15)));
  const backLabel = document.createElement("span");
  backLabel.textContent = "Kembali ke Downloads";
  back.append(backLabel);
  back.addEventListener("click", () => {
    downloadDetailId = null;
    renderQueue(queueRows);
  });
  const heading = document.createElement("h2");
  heading.textContent = `${row.owner} / ${row.repo}`;
  const progress = document.createElement("div");
  progress.className = "download-progress";
  const status = document.createElement("span");
  status.className = "status-chip";
  if (row.status === "berjalan" || row.status === "gagal" || row.status === "selesai") {
    status.classList.add(`is-${row.status}`);
  }
  status.textContent = statusLabel(row.status);
  progress.append(status);
  const figures = document.createElement("p");
  figures.className = "download-figures";
  figures.textContent = text;
  if (text.length > 0) {
    progress.append(figures);
  }
  const message = rowMessage(row);
  if (message.length > 0) {
    const error = document.createElement("p");
    error.className = "error";
    error.textContent = message;
    progress.append(error);
  }
  const actions = document.createElement("div");
  actions.className = "download-actions";
  if (row.status === "berjalan") {
    const pause = quietAction(pauseIcon, "Jeda unduhan");
    pause.addEventListener("click", () => {
      runQueue("queue_pause", row.id);
    });
    actions.append(pause);
  } else if (row.status === "dijeda") {
    const resume = quietAction(playIcon, "Lanjut");
    resume.addEventListener("click", () => {
      runQueue("queue_resume", row.id);
    });
    actions.append(resume);
  }
  if (row.status === "berjalan" || row.status === "dijeda" || row.status === "menunggu") {
    const cancel = quietAction(cancelIcon, "Batalkan");
    cancel.addEventListener("click", () => {
      runQueue("queue_cancel", row.id);
    });
    actions.append(cancel);
  }
  if (actions.childElementCount > 0) {
    progress.append(actions);
  }
  const facts = document.createElement("div");
  facts.className = "download-facts";
  const href = githubHref(row.owner, row.repo);
  if (href) {
    facts.append(factRow("Sumber", href.replace(/^https:\/\//, "")));
  }
  const ref = row.gitRef.trim();
  if (ref.length > 0) {
    facts.append(factRow("Branch / ref", ref));
  }
  facts.append(factRow("Format", "Snapshot repo, tanpa histori Git"));
  const stages = document.createElement("aside");
  stages.className = "download-stages";
  const stagesTitle = document.createElement("h3");
  stagesTitle.textContent = "Tahap unduhan";
  const stageList = document.createElement("ol");
  for (const [label, description] of [
    ["Repo diperiksa", "Tautan repo publik valid"],
    ["Mengunduh arsip", "Menerima isi repositori"],
    ["Mengekstrak", "Dimulai setelah unduhan selesai"],
    ["Siap dibuka", "Folder muncul di Library"],
  ] as const) {
    const item = document.createElement("li");
    const name = document.createElement("span");
    name.textContent = label;
    const copy = document.createElement("span");
    copy.textContent = description;
    item.append(name, copy);
    stageList.append(item);
  }
  const rule = document.createElement("p");
  rule.textContent =
    "Selesai berarti arsip sudah diekstrak utuh. Folder belum bisa dibuka selama proses ini.";
  stages.append(stagesTitle, stageList, rule);
  const workspace = document.createElement("div");
  workspace.className = "download-workspace";
  workspace.append(progress, facts);
  downloadDetail.append(back, heading, workspace, stages);
  downloadDetail.hidden = false;
}

function hideQueueChrome(): void {
  pasteForm.hidden = true;
  queueEmpty.hidden = true;
  transfer.hidden = true;
  queueTabBar.hidden = true;
  failBanner.hidden = true;
  failRunning.hidden = true;
  doneNotice.hidden = true;
  doneList.hidden = true;
  doneNote.hidden = true;
  queueList.hidden = true;
  queueFoot.hidden = true;
}

function applyDownloadsOverlay(): boolean {
  if (currentId() !== "downloads") {
    downloadDetail.hidden = true;
    addWorkspace.hidden = true;
    return false;
  }
  if (addOpen) {
    hideQueueChrome();
    downloadDetail.hidden = true;
    addWorkspace.hidden = false;
    pageHead.hidden = false;
    addRepo.hidden = true;
    title.textContent = "Tambah repo";
    addFolderPath.textContent = downloadFolder;
    paintAddChoices();
    paintAddButton();
    return true;
  }
  const row =
    downloadDetailId === null ? null : (queueRows.find((item) => item.id === downloadDetailId) ?? null);
  if (!row) {
    if (downloadDetailId !== null) {
      downloadDetailId = null;
    }
    addWorkspace.hidden = true;
    downloadDetail.hidden = true;
    pageHead.hidden = false;
    addRepo.hidden = false;
    return false;
  }
  hideQueueChrome();
  addWorkspace.hidden = true;
  pageHead.hidden = true;
  addRepo.hidden = true;
  paintDownloadDetail(row);
  return true;
}

function queueRowKey(row: QueueRow): string {
  return `${row.id}\t${row.status}\t${row.gitRef}`;
}

function paintQueueProgress(item: HTMLElement, row: QueueRow): void {
  const note = item.querySelector(".repo-body > .error");
  if (note) {
    note.textContent = transferLine({
      status: row.status,
      bytes: row.bytes,
      total: row.total,
      speed: latestSpeeds.get(row.id) ?? null,
      error: rowMessage(row),
    });
  }
  const body = item.querySelector(".repo-body");
  if (!(body instanceof HTMLElement)) {
    return;
  }
  const percent = percentOf(row.bytes, row.total);
  const show = percent !== null && (row.status === "berjalan" || row.status === "dijeda");
  const track = body.querySelector(":scope > .track");
  if (!show) {
    track?.remove();
    return;
  }
  const fill = track?.firstElementChild;
  if (track && fill instanceof HTMLElement) {
    fill.style.width = `${percent}%`;
    return;
  }
  track?.remove();
  const created = document.createElement("span");
  created.className = "track";
  const bar = document.createElement("span");
  bar.style.width = `${percent}%`;
  created.append(bar);
  body.append(created);
}

function renderQueue(rows: QueueRow[]): void {
  queueRows = rows;
  paintQueueCount(rows);
  const counts: Record<QueueTab, number> = {
    semua: rows.length,
    berjalan: 0,
    menunggu: 0,
    selesai: 0,
    gagal: 0,
  };
  let dijeda = 0;
  for (const row of rows) {
    if (row.status === "berjalan" || row.status === "menunggu" || row.status === "gagal") {
      counts[row.status] += 1;
    } else if (row.status === "dijeda") {
      dijeda += 1;
    }
  }
  counts.selesai = doneRows.length;
  for (const [id, button] of tabButtons) {
    const count = button.querySelector(".count");
    if (count) {
      count.textContent = String(counts[id]);
    }
    button.setAttribute("aria-selected", id === queueTab ? "true" : "false");
  }
  const now = Date.now();
  const live = new Set(rows.map((row) => row.id));
  for (const id of samples.keys()) {
    if (!live.has(id)) {
      samples.delete(id);
    }
  }
  latestSpeeds = new Map();
  for (const row of rows) {
    const speed = rowSpeed(row, now);
    if (speed !== null && speed >= 1) {
      latestSpeeds.set(row.id, speed);
    }
  }
  if (applyDownloadsOverlay()) {
    return;
  }
  const parts: string[] = [];
  if (counts.menunggu > 0) {
    parts.push(`${counts.menunggu} menunggu`);
  }
  if (dijeda > 0) {
    parts.push(`${dijeda} dijeda`);
  }
  if (counts.gagal > 0) {
    parts.push(`${counts.gagal} perlu dicoba ulang`);
  }
  transferHead.hidden = counts.berjalan === 0;
  transferHeadline.textContent = counts.berjalan > 0 ? `${counts.berjalan} repo sedang diunduh` : "";
  transferDetail.textContent = parts.join("  ·  ");
  pauseAll.disabled = counts.berjalan === 0;
  const noRows = rows.length === 0;
  const showDone = queueTab === "selesai" && doneRows.length > 0;
  const failAttention = queueTab === "gagal" && counts.gagal > 0;
  queueEmpty.hidden = showDone || !noRows;
  const pasteOpen = failAttention && document.activeElement === pasteInput;
  pasteForm.hidden = showDone || (failAttention && !pasteOpen);
  transfer.hidden = showDone || failAttention || noRows || (counts.berjalan === 0 && parts.length === 0);
  queueTabBar.hidden = noRows && doneRows.length === 0;
  empty.hidden = true;
  action.hidden = true;
  failBanner.hidden = !failAttention;
  if (failAttention) {
    failTitle.textContent = `${counts.gagal} unduhan perlu perhatian`;
  }
  const othersRunning = failAttention && counts.berjalan > 0;
  failRunning.hidden = !othersRunning;
  if (othersRunning) {
    failRunning.textContent = `${counts.berjalan} unduhan lain masih berjalan`;
  }
  queueFoot.hidden = failAttention || downloadFolder.trim().length === 0;
  paintHeadingAction(failAttention);
  paintDone(showDone);
  const shown = queueTab === "semua" ? rows : rows.filter((row) => row.status === queueTab);
  if (showDone || shown.length === 0) {
    queueList.replaceChildren();
    queueList.hidden = true;
    return;
  }
  const running = rows.filter((row) => row.status === "berjalan");
  let rateSum = 0;
  let rateCount = 0;
  for (const row of running) {
    const speed = latestSpeeds.get(row.id);
    if (speed !== undefined) {
      rateSum += speed;
      rateCount += 1;
    }
  }
  transferSpeed.hidden = rateCount === 0;
  if (rateCount > 0) {
    transferRate.textContent = formatRate(rateSum);
  }
  const keys = shown.map((row) => queueRowKey(row));
  const current = [...queueList.children];
  const sameShape =
    current.length === shown.length &&
    current.every((node, index) => node instanceof HTMLElement && node.dataset.key === keys[index]);
  if (sameShape) {
    if (queueTab !== "gagal") {
      for (let index = 0; index < shown.length; index += 1) {
        const node = current[index];
        const row = shown[index];
        if (node instanceof HTMLElement && row) {
          paintQueueProgress(node, row);
        }
      }
    }
    queueList.hidden = false;
    return;
  }
  queueList.replaceChildren();
  for (const row of shown) {
    if (queueTab === "gagal") {
      queueList.append(failQueueRow(row));
      continue;
    }
    const item = document.createElement("li");
    const tile = document.createElement("span");
    tile.className = "tile";
    tile.innerHTML = svg(folderGitIcon, 20);
    const body = document.createElement("div");
    body.className = "repo-body";
    const identity = document.createElement("span");
    identity.className = "repo-id";
    const name = document.createElement("button");
    name.type = "button";
    name.className = "name";
    name.textContent = `${row.owner} / ${row.repo}`;
    name.addEventListener("click", () => {
      openDownloadDetail(row.id);
    });
    identity.append(name);
    const ref = row.gitRef.trim();
    if (ref.length > 0) {
      const chip = document.createElement("span");
      chip.className = "ref-chip";
      chip.textContent = ref;
      identity.append(chip);
    }
    const note = document.createElement("span");
    note.className = "error";
    if (row.status === "gagal") {
      note.classList.add("is-failed");
    }
    note.textContent = transferLine({
      status: row.status,
      bytes: row.bytes,
      total: row.total,
      speed: latestSpeeds.get(row.id) ?? null,
      error: rowMessage(row),
    });
    body.append(identity, note);
    const percent = percentOf(row.bytes, row.total);
    if (percent !== null && (row.status === "berjalan" || row.status === "dijeda")) {
      const track = document.createElement("span");
      track.className = "track";
      const fill = document.createElement("span");
      fill.style.width = `${percent}%`;
      track.append(fill);
      body.append(track);
    }
    const status = document.createElement("span");
    status.className = "status-chip";
    if (row.status === "berjalan" || row.status === "gagal" || row.status === "selesai") {
      status.classList.add(`is-${row.status}`);
    }
    status.textContent = statusLabel(row.status);
    const actions = document.createElement("span");
    actions.className = "row-actions";
    if (row.status === "gagal") {
      const retry = document.createElement("button");
      retry.type = "button";
      retry.className = "retry-btn";
      retry.innerHTML = svg(retryIcon, 16);
      const retryLabel = document.createElement("span");
      retryLabel.textContent = "Ulang";
      retry.append(retryLabel);
      retry.addEventListener("click", () => {
        runQueue("queue_retry", row.id);
      });
      actions.append(retry);
    } else if (row.status === "dijeda") {
      actions.append(
        rowIconButton(playIcon, "Lanjut", () => {
          runQueue("queue_resume", row.id);
        }),
        rowIconButton(cancelIcon, "Batal", () => {
          runQueue("queue_cancel", row.id);
        }),
      );
    } else if (row.status === "berjalan") {
      actions.append(
        rowIconButton(pauseIcon, "Jeda", () => {
          runQueue("queue_pause", row.id);
        }),
        rowIconButton(cancelIcon, "Batal", () => {
          runQueue("queue_cancel", row.id);
        }),
      );
    } else if (row.status === "menunggu") {
      actions.append(
        rowIconButton(pauseIcon, "Jeda"),
        rowIconButton(cancelIcon, "Batal", () => {
          runQueue("queue_cancel", row.id);
        }),
      );
    } else {
      actions.append(rowIconButton(pauseIcon, "Jeda"), rowIconButton(cancelIcon, "Batal"));
    }
    item.dataset.key = queueRowKey(row);
    item.append(tile, body, status, actions);
    queueList.append(item);
  }
  queueList.hidden = false;
}

function recentWhen(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  const clock = `${hours}.${minutes}`;
  const now = new Date();
  const sameDay =
    date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate();
  if (sameDay) {
    return `Hari ini, ${clock}`;
  }
  const day = String(date.getDate()).padStart(2, "0");
  const month = String(date.getMonth() + 1).padStart(2, "0");
  return `${day}.${month}.${date.getFullYear()}, ${clock}`;
}

function paintQueueCount(rows: QueueRow[]): void {
  const count = rows.length;
  queueCount.hidden = count === 0;
  queueCount.textContent = count === 0 ? "" : String(count);
}

function homeIsFirst(): boolean {
  return (homeHistoryCount ?? 0) === 0 && (homeQueueCount ?? 0) === 0;
}

function applyHomeMode(): void {
  if (currentId() !== "home") {
    return;
  }
  const first = homeIsFirst();
  home.classList.toggle("is-first", first);
  if (first) {
    title.textContent = "Selamat datang di Peti.";
    return;
  }
  const page = pages.find((item) => item.id === "home");
  if (page) {
    title.textContent = page.title;
  }
}

function renderHomeStrip(rows: QueueRow[]): void {
  if (currentId() !== "home") {
    return;
  }
  homeQueueCount = rows.length;
  paintQueueCount(rows);
  applyHomeMode();
  let berjalan = 0;
  let menunggu = 0;
  let gagal = 0;
  for (const row of rows) {
    if (row.status === "berjalan") {
      berjalan += 1;
    } else if (row.status === "menunggu") {
      menunggu += 1;
    } else if (row.status === "gagal") {
      gagal += 1;
    }
  }
  homeHeadline.textContent = berjalan > 0 ? `${berjalan} unduhan berjalan` : "";
  homeDetail.textContent = menunggu > 0 ? `${menunggu} repo menunggu` : "";
  homeFail.textContent = gagal > 0 ? `${gagal} gagal` : "";
  homeFail.hidden = gagal === 0;
  homeStrip.hidden = berjalan === 0 && menunggu === 0 && gagal === 0;
}

function renderRecent(rows: HistoryRow[]): void {
  recentList.replaceChildren();
  if (currentId() !== "home") {
    recentList.hidden = true;
    return;
  }
  homeHistoryCount = rows.length;
  applyHomeMode();
  const shown = recentHistory(rows);
  if (homeIsFirst() || shown.length === 0) {
    recentList.hidden = true;
    return;
  }
  for (const row of shown) {
    const item = document.createElement("li");
    const icon = document.createElement("span");
    icon.className = "recent-icon";
    icon.innerHTML = svg(folderCheckIcon, 22);
    const info = document.createElement("div");
    info.className = "recent-info";
    const name = document.createElement("span");
    name.className = "name";
    name.textContent = historyName(row);
    const path = document.createElement("span");
    path.className = "recent-path";
    path.textContent = row.folder;
    info.append(name, path);
    const when = document.createElement("span");
    when.className = "recent-when";
    when.textContent = recentWhen(row.finishedAt);
    const open = document.createElement("button");
    open.type = "button";
    open.className = "open-folder";
    open.append(document.createRange().createContextualFragment(svg(folderOpenIcon, 16)));
    const openLabel = document.createElement("span");
    openLabel.textContent = "Buka folder";
    open.append(openLabel);
    const note = document.createElement("span");
    note.className = "open-error";
    open.addEventListener("click", () => {
      note.textContent = "";
      void invoke("open_history_folder", { id: row.id }).catch((err: unknown) => {
        note.textContent =
          typeof err === "string" && err.length > 0 ? err : "Folder tidak bisa dipakai, pilih yang lain";
      });
    });
    item.append(icon, info, when, open, note);
    recentList.append(item);
  }
  recentList.hidden = false;
}

function renderHistory(id: PageId, rows: HistoryRow[]): void {
  if (id === "home") {
    renderRecent(rows);
    return;
  }
  if (id === "downloads") {
    doneRows = rows;
    if (currentId() === "downloads") {
      renderQueue(queueRows);
    }
    return;
  }
  recentList.hidden = true;
  historyList.replaceChildren();
  const shown = rows;
  if (shown.length === 0) {
    historyList.hidden = true;
    if (id === "library") {
      libraryEmpty.hidden = false;
      empty.hidden = true;
      action.hidden = true;
    }
    return;
  }
  if (id === "library") {
    libraryEmpty.hidden = true;
    empty.hidden = true;
    action.hidden = true;
  }
  for (const row of shown) {
    const item = document.createElement("li");
    const name = document.createElement("span");
    name.className = "name";
    name.textContent = historyName(row);
    const folder = document.createElement("span");
    folder.className = "meta folder";
    folder.textContent = row.folder;
    const when = document.createElement("span");
    when.className = "meta when";
    when.textContent = row.finishedAt;
    const open = document.createElement("button");
    open.type = "button";
    open.className = "open";
    open.textContent = "Buka folder";
    const note = document.createElement("span");
    note.className = "open-error";
    open.addEventListener("click", () => {
      note.textContent = "";
      void invoke("open_history_folder", { id: row.id }).catch((err: unknown) => {
        note.textContent =
          typeof err === "string" && err.length > 0
            ? err
            : "Folder tidak bisa dipakai, pilih yang lain";
      });
    });
    item.append(name, folder, when, open, note);
    historyList.append(item);
  }
  historyList.hidden = false;
  if (id === "library") {
    empty.hidden = true;
    action.hidden = true;
  }
}

function show(id: PageId): void {
  const page = pages.find((item) => item.id === id);
  if (!page) {
    return;
  }
  if (location.hash !== `#${id}`) {
    history.replaceState(null, "", `#${id}`);
  }
  title.textContent = page.title;
  crumbSection.textContent = page.section;
  crumbPage.textContent = page.label;
  empty.textContent = page.empty;
  action.textContent = page.action.label;
  action.onclick = () => show(page.action.target);
  queueTicket += 1;
  clearQueuePoll();
  const isSettings = page.id === "settings";
  const isDiscover = page.id === "discover";
  const isDownloads = page.id === "downloads";
  const isHome = page.id === "home";
  const isLibrary = page.id === "library";
  if (isDownloads) {
    addOpen = false;
    downloadDetailId = null;
  }
  pageHead.hidden = false;
  empty.hidden = isSettings || isDiscover || isDownloads || isHome || isLibrary;
  action.hidden = isSettings || isDiscover || isDownloads || isHome || isLibrary;
  libraryEmpty.hidden = !isLibrary;
  addRepo.hidden = !isDownloads;
  downloads.hidden = !isDownloads;
  home.hidden = !isHome;
  if (isHome) {
    applyHomeMode();
  }
  homeStrip.hidden = true;
  settingsPage.hidden = !isSettings;
  themeBlock.hidden = !isSettings;
  githubTicket += 1;
  clearGithubTimer();
  githubBlock.hidden = !isSettings;
  discoverFilters.hidden = !isDiscover;
  notice.hidden = true;
  notice.textContent = "";
  resultHead.hidden = true;
  discoverStatus.card.hidden = true;
  discoverEmpty.card.hidden = true;
  discoverList.hidden = true;
  discoverList.replaceChildren();
  selectionBar.hidden = true;
  detail.hidden = true;
  clearDiscoverTimer();
  discoverTicket += 1;
  historyList.hidden = true;
  historyList.replaceChildren();
  recentList.hidden = true;
  recentList.replaceChildren();
  queueList.hidden = true;
  queueList.replaceChildren();
  const ticket = ++historyTicket;
  if (isDiscover) {
    renderSelection();
    renderDetail();
    renderDiscover(discoverCards, discoverNotice);
    scheduleDiscover(true);
  }
  if (id === "downloads" || id === "home") {
    refreshVisibleQueue();
  }
  if (isSettings || isDiscover) {
    loadGithub();
  }
  if (isSettings || isHome) {
    loadAppSettings();
  }
  if (id === "home" || id === "library" || id === "downloads") {
    void invoke("history_rows")
      .then((loaded) => {
        if (ticket !== historyTicket) {
          return;
        }
        renderHistory(id, readHistory(loaded));
      })
      .catch(() => {
        if (ticket !== historyTicket) {
          return;
        }
        renderHistory(id, []);
      });
  }
  paintPasteNote(pasteNote.textContent ?? "");
  paintDeviceFlow();
  for (const [pageId, button] of buttons) {
    if (pageId === id) {
      button.setAttribute("aria-current", "page");
    } else {
      button.removeAttribute("aria-current");
    }
  }
}

const brand = document.createElement("div");
brand.className = "brand";
brand.innerHTML = `<svg class="logo" viewBox="0 0 256 256" width="28" height="28" aria-hidden="true"><path fill="currentColor" fill-rule="evenodd" d="M 88 44 H 168 A 44 44 0 0 1 212 88 V 168 A 44 44 0 0 1 168 212 H 88 A 44 44 0 0 1 44 168 V 88 A 44 44 0 0 1 88 44 Z M 104 44 V 96 A 24 24 0 0 0 152 96 V 44 Z"/></svg>`;
const wordmark = document.createElement("span");
wordmark.className = "wordmark";
wordmark.textContent = "peti";
brand.append(wordmark);

const navMain = document.createElement("div");
navMain.className = "nav-main";
const grow = document.createElement("div");
grow.className = "grow";

const destination = document.createElement("div");
destination.className = "destination";
const destinationTitle = document.createElement("div");
destinationTitle.className = "destination-title";
destinationTitle.innerHTML = svg(
  `<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>`,
  16,
);
const destinationLabel = document.createElement("span");
destinationLabel.textContent = "Folder unduhan";
destinationTitle.append(destinationLabel);
const destinationPath = document.createElement("span");
destinationPath.className = "destination-path";
destination.append(destinationTitle, destinationPath);

const divider = document.createElement("div");
divider.className = "divider";

const queueCount = document.createElement("span");
queueCount.className = "nav-count";
queueCount.hidden = true;

const sideAccount = document.createElement("button");
sideAccount.type = "button";
sideAccount.className = "side-account";
const avatar = document.createElement("span");
avatar.className = "avatar";
const identity = document.createElement("span");
identity.className = "identity";
const accountName = document.createElement("span");
accountName.className = "account-name";
const accountState = document.createElement("span");
accountState.className = "account-state";
identity.append(accountName, accountState);
const accountChevron = document.createElement("span");
accountChevron.className = "account-chevron";
accountChevron.innerHTML = svg(`<path d="m9 18 6-6-6-6"/>`, 14);
sideAccount.append(avatar, identity, accountChevron);
sideAccount.addEventListener("click", () => show("settings"));

function paintAccount(): void {
  const login = githubView?.connected === true ? githubView.login : "";
  const connected = login.length > 0;
  accountName.textContent = connected ? login : "Tanpa akun";
  accountState.textContent = connected ? "GitHub terhubung" : "Hubungkan GitHub";
  avatar.replaceChildren();
  if (connected) {
    avatar.textContent = login.slice(0, 1).toUpperCase();
  } else {
    avatar.innerHTML = svg(githubMarkIcon, 16);
  }
}

let settingsButton: HTMLButtonElement | null = null;
for (const page of pages) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "nav-btn";
  if (page.id === "settings") {
    button.classList.add("settings");
  }
  button.innerHTML = `${icons[page.id]}<span class="nav-label"></span>`;
  const label = button.querySelector(".nav-label");
  if (label) {
    label.textContent = page.label;
  }
  if (page.id === "downloads") {
    button.append(queueCount);
  }
  button.addEventListener("click", () => show(page.id));
  buttons.set(page.id, button);
  if (page.id === "settings") {
    settingsButton = button;
  } else {
    navMain.append(button);
  }
}
nav.append(brand, navMain, grow, destination, divider);
if (settingsButton) {
  nav.append(settingsButton);
}
nav.append(sideAccount);
paintAccount();

function onDiscoverFilter(): void {
  if (currentId() !== "discover") {
    return;
  }
  paintPeriod();
  paintDiscoverFilters();
  scheduleDiscover(false);
}

searchInput.addEventListener("input", onDiscoverFilter);
languageSelect.addEventListener("change", onDiscoverFilter);
topicSelect.addEventListener("change", onDiscoverFilter);
periodSelect.addEventListener("change", onDiscoverFilter);
discoverFilters.addEventListener("submit", (event) => {
  event.preventDefault();
  if (currentId() === "discover") {
    scheduleDiscover(true);
  }
});

addSelected.addEventListener("click", () => {
  void queueCards([...selected.values()]);
});

clearSelected.addEventListener("click", () => {
  selected.clear();
  renderSelection();
  if (currentId() === "discover") {
    renderDiscover(discoverCards, discoverNotice);
  }
});

githubButton.addEventListener("click", () => {
  if (currentId() !== "settings" || githubButton.hidden) {
    return;
  }
  const leaving = githubButton.textContent === "Keluar";
  if (leaving) {
    deviceFlowOpen = false;
  } else {
    deviceFlowOpen = true;
    paintDeviceFlow();
  }
  const ticket = ++githubTicket;
  clearGithubTimer();
  void invoke(leaving ? "github_logout" : "github_begin")
    .then((loaded) => {
      if (ticket !== githubTicket || currentId() !== "settings") {
        return;
      }
      const view = readGithub(loaded);
      if (!view) {
        return;
      }
      githubView = view;
      renderGithub(view, "");
    })
    .catch((err: unknown) => {
      if (ticket !== githubTicket || currentId() !== "settings") {
        return;
      }
      if (leaving && githubView?.connected) {
        renderGithub(githubView, githubMessage(err));
        return;
      }
      applyGithubError(err, !leaving);
    });
});

window.addEventListener("hashchange", () => show(currentId()));
show(currentId());

void invoke("saved_theme")
  .then((saved) => {
    if (themeTouched) {
      return;
    }
    theme = parseSavedTheme(saved);
    applyTheme(theme, document.documentElement);
    for (const input of themeBlock.querySelectorAll("input")) {
      if (input instanceof HTMLInputElement) {
        input.checked = input.value === theme;
      }
    }
  })
  .catch(() => undefined);
