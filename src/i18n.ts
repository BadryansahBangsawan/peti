export const localeOptions = [
  { id: "en", label: "English", htmlLang: "en" },
  { id: "hi", label: "हिन्दी", htmlLang: "hi" },
  { id: "zh", label: "中文", htmlLang: "zh-Hans" },
  { id: "pt", label: "Português", htmlLang: "pt-BR" },
  { id: "en-GB", label: "English (UK)", htmlLang: "en-GB" },
  { id: "ja", label: "日本語", htmlLang: "ja" },
  { id: "id", label: "Bahasa Indonesia", htmlLang: "id" },
  { id: "de", label: "Deutsch", htmlLang: "de" },
  { id: "ru", label: "Русский", htmlLang: "ru" },
  { id: "fr", label: "Français", htmlLang: "fr" },
] as const;

export type LocaleId = (typeof localeOptions)[number]["id"];

const localeIds = new Set<string>(localeOptions.map((item) => item.id));

export function parseLocale(value: unknown): LocaleId {
  return typeof value === "string" && localeIds.has(value) ? (value as LocaleId) : "id";
}

export function htmlLang(id: LocaleId): string {
  return localeOptions.find((item) => item.id === id)?.htmlLang ?? "id";
}

export function fill(template: string, values: Record<string, string | number>): string {
  return template.replace(/\{([a-z]+)\}/g, (token, key: string) =>
    Object.prototype.hasOwnProperty.call(values, key) ? String(values[key]) : token,
  );
}

export type Messages = {
  navAria: string;
  sidebarAria: string;
  folderDownloads: string;
  noAccount: string;
  accountConnected: string;
  accountConnect: string;
  navHome: string;
  navDownloads: string;
  navDiscover: string;
  navLibrary: string;
  navSettings: string;
  sectionWorkspace: string;
  sectionExplore: string;
  sectionLocal: string;
  homeTitle: string;
  homeWelcome: string;
  homeEmpty: string;
  homeAction: string;
  downloadsTitle: string;
  downloadsEmpty: string;
  downloadsAction: string;
  discoverTitle: string;
  discoverEmpty: string;
  discoverAction: string;
  libraryTitle: string;
  libraryEmptyPage: string;
  libraryAction: string;
  settingsTitle: string;
  settingsEmpty: string;
  settingsAction: string;
  running: string;
  waitingRepos: string;
  failedCount: string;
  recentTitle: string;
  openFolder: string;
  openLibrary: string;
  seeDownloads: string;
  noPick: string;
  exploreDiscover: string;
  whatRepo: string;
  repoPlaceholder: string;
  linkAria: string;
  changeFolder: string;
  pasteClipboard: string;
  pasteMany: string;
  stepsTitle: string;
  stepPaste: string;
  stepFolder: string;
  stepOpen: string;
  discoverCardText: string;
  githubOnly: string;
  githubPublic: string;
  openSettings: string;
  todayAt: string;
  folderUnusable: string;
  composerTitle: string;
  composerHint: string;
  downloadRepo: string;
  changeShort: string;
  addRepo: string;
  retryAllFailed: string;
  pastePlaceholder: string;
  emptyEyebrow: string;
  emptyTitle: string;
  emptyDesc: string;
  pauseAll: string;
  tabAll: string;
  tabRunning: string;
  tabWaiting: string;
  tabDone: string;
  tabFailed: string;
  seeAllLibrary: string;
  doneNote: string;
  failNote: string;
  linksTitle: string;
  linksHint: string;
  linksAria: string;
  whatNext: string;
  saveTo: string;
  readyQueue: string;
  addSubmit: string;
  addSubmitCount: string;
  linksReady: string;
  backDownloads: string;
  pause: string;
  cancel: string;
  resume: string;
  retry: string;
  pauseDownload: string;
  cancelDownload: string;
  inspectLink: string;
  source: string;
  branchRef: string;
  formatLabel: string;
  formatValue: string;
  stagesTitle: string;
  stageChecked: string;
  stageCheckedDesc: string;
  stageArchive: string;
  stageArchiveDesc: string;
  stageExtract: string;
  stageExtractDesc: string;
  stageReady: string;
  stageReadyDesc: string;
  stageRule: string;
  unavailable: string;
  backDetail: string;
  forkBanner: string;
  forkMade: string;
  waitingSlotShort: string;
  destFolder: string;
  forkOk: string;
  forkOkDownload: string;
  forkActionResult: string;
  forkCreated: string;
  yourFork: string;
  branch: string;
  forkQueueNote: string;
  forkPersist: string;
  backDiscover: string;
  openOnGithub: string;
  openFork: string;
  aboutRepo: string;
  youGet: string;
  getSnapshot: string;
  getExtract: string;
  getNoGit: string;
  origin: string;
  statusRunning: string;
  statusPaused: string;
  statusWaiting: string;
  statusFailed: string;
  statusDone: string;
  statusCancelled: string;
  waitingSlot: string;
  downloaded: string;
  remaining: string;
  totalUnknown: string;
  progressSaved: string;
  searchPlaceholder: string;
  searchAria: string;
  clearSearch: string;
  languageAria: string;
  allLanguages: string;
  topicAria: string;
  allTopics: string;
  periodDay: string;
  periodWeek: string;
  periodMonth: string;
  byStars: string;
  justUpdated: string;
  resetFilters: string;
  filterNote: string;
  searchResults: string;
  popularDay: string;
  popularWeek: string;
  popularMonth: string;
  repoCount: string;
  emptyDiscoverEyebrow: string;
  emptyDiscoverHeading: string;
  emptyDiscoverHint: string;
  limitEyebrow: string;
  discoverEyebrow: string;
  tryAgain: string;
  clearSelection: string;
  addSelected: string;
  selected: string;
  limitMessage: string;
  failedSearch: string;
  limitHint: string;
  noDescription: string;
  takeTitle: string;
  hintDownload: string;
  hintQueue: string;
  hintFork: string;
  hintForkDownload: string;
  download: string;
  onlyDownload: string;
  onlyFork: string;
  forkThenDownload: string;
  addToQueue: string;
  permit: string;
  queuedNote: string;
  alreadyQueued: string;
  queueFailed: string;
  ownRepo: string;
  forkFailed: string;
  publicChip: string;
  pickRepo: string;
  folderTitle: string;
  appearance: string;
  themeLegend: string;
  themeSystem: string;
  themeLight: string;
  themeDark: string;
  language: string;
  githubTitle: string;
  clientId: string;
  tagline: string;
  version: string;
  notConnected: string;
  publicStill: string;
  connected: string;
  logout: string;
  connect: string;
  unprepared: string;
  denied: string;
  expired: string;
  notRun: string;
  deviceHint: string;
  deviceBack: string;
  deviceTitle: string;
  deviceStep: string;
  deviceBody: string;
  copyCode: string;
  openGithub: string;
  waitingBrowser: string;
  afterConnect: string;
  asideFork: string;
  asideDownload: string;
  asideChoose: string;
  asideNote: string;
  libraryEmptyTitle: string;
  libraryEmptyText: string;
  startDownload: string;
  libraryNote: string;
  reposDownloading: string;
  reposReady: string;
  failAttention: string;
  otherStillRunning: string;
  waitingCount: string;
  pausedCount: string;
  retryCount: string;
  addHintDownload: string;
  addHintFork: string;
  addHintForkDownload: string;
  reviewSnapshot: string;
  reviewFolder: string;
  reviewStarts: string;
  forkMadeName: string;
};

export const id: Messages = {
  navAria: "Utama",
  sidebarAria: "Sidebar",
  folderDownloads: "Folder unduhan",
  noAccount: "Tanpa akun",
  accountConnected: "GitHub terhubung",
  accountConnect: "Hubungkan GitHub",
  navHome: "Home",
  navDownloads: "Downloads",
  navDiscover: "Discover",
  navLibrary: "Library",
  navSettings: "Settings",
  sectionWorkspace: "Workspace",
  sectionExplore: "Jelajahi",
  sectionLocal: "Lokal",
  homeTitle: "Selamat datang kembali.",
  homeWelcome: "Selamat datang di Peti.",
  homeEmpty: "Tidak ada unduhan yang berjalan.",
  homeAction: "Buka Downloads",
  downloadsTitle: "Downloads",
  downloadsEmpty: "Belum ada unduhan. Tempel tautan untuk memulai.",
  downloadsAction: "Buka Home",
  discoverTitle: "Temukan repo berikutnya.",
  discoverEmpty: "Belum ada repo.",
  discoverAction: "Buka Downloads",
  libraryTitle: "Library",
  libraryEmptyPage: "Belum ada unduhan yang selesai.",
  libraryAction: "Buka Downloads",
  settingsTitle: "Settings",
  settingsEmpty: "Belum ada pengaturan tersimpan.",
  settingsAction: "Buka Home",
  running: "{count} unduhan berjalan",
  waitingRepos: "{count} repo menunggu",
  failedCount: "{count} gagal",
  recentTitle: "Terakhir diunduh",
  openFolder: "Buka folder",
  openLibrary: "Buka Library",
  seeDownloads: "Lihat Downloads",
  noPick: "Belum punya repo pilihan?",
  exploreDiscover: "Jelajahi Discover",
  whatRepo: "Repo apa yang ingin kamu unduh?",
  repoPlaceholder: "github.com/pemilik/repo",
  linkAria: "Tautan GitHub",
  changeFolder: "Ganti folder",
  pasteClipboard: "Tempel dari clipboard",
  pasteMany: "Bisa satu atau banyak tautan sekaligus. Kamu akan melihat validasi sebelum mulai.",
  stepsTitle: "Dari tautan ke folder lokal.",
  stepPaste: "Tempel tautan repo publik",
  stepFolder: "Pilih folder dan mulai unduh",
  stepOpen: "Buka hasilnya dari Library",
  discoverCardText: "Cari proyek lewat bahasa, topik, dan repo populer di Discover.",
  githubOnly: "Masuk GitHub hanya jika dibutuhkan.",
  githubPublic: "Unduh repo publik tanpa akun. Hubungkan akun nanti untuk membuat fork.",
  openSettings: "Buka Settings",
  todayAt: "Hari ini, {clock}",
  folderUnusable: "Folder tidak bisa dipakai, pilih yang lain",
  composerTitle: "Mulai dengan tautan GitHub.",
  composerHint: "Punya beberapa repo? Tempel semua tautannya sekaligus.",
  downloadRepo: "Unduh repo",
  changeShort: "Ganti",
  addRepo: "Tambah repo",
  retryAllFailed: "Ulang semua gagal",
  pastePlaceholder: "Tempel tautan GitHub untuk menambahkan repo…",
  emptyEyebrow: "Downloads / Kosong",
  emptyTitle: "Repo pertamamu dimulai di sini.",
  emptyDesc: "Tempel tautan GitHub untuk menyimpan isi repo ke perangkatmu.",
  pauseAll: "Jeda semua",
  tabAll: "Semua",
  tabRunning: "Berjalan",
  tabWaiting: "Menunggu",
  tabDone: "Selesai",
  tabFailed: "Gagal",
  seeAllLibrary: "Lihat semuanya di Library",
  doneNote: "Hasil unduhan tetap tersedia di Library setelah kamu berpindah halaman.",
  failNote: "Setiap item ditangani sendiri. Mengulang mengunduh repo itu dari awal.",
  linksTitle: "Tautan repositori",
  linksHint: "Satu tautan per baris",
  linksAria: "Tautan repositori",
  whatNext: "Apa yang ingin kamu lakukan?",
  saveTo: "Simpan ke",
  readyQueue: "Siap masuk antrean",
  addSubmit: "Tambahkan",
  addSubmitCount: "Tambahkan {count} repo",
  linksReady: "{count} tautan siap ditambahkan",
  backDownloads: "Kembali ke Downloads",
  pause: "Jeda",
  cancel: "Batal",
  resume: "Lanjut",
  retry: "Ulang",
  pauseDownload: "Jeda unduhan",
  cancelDownload: "Batalkan",
  inspectLink: "Periksa tautan",
  source: "Sumber",
  branchRef: "Branch / ref",
  formatLabel: "Format",
  formatValue: "Snapshot repo, tanpa histori Git",
  stagesTitle: "Tahap unduhan",
  stageChecked: "Repo diperiksa",
  stageCheckedDesc: "Tautan repo publik valid",
  stageArchive: "Mengunduh arsip",
  stageArchiveDesc: "Menerima isi repositori",
  stageExtract: "Mengekstrak",
  stageExtractDesc: "Dimulai setelah unduhan selesai",
  stageReady: "Siap dibuka",
  stageReadyDesc: "Folder muncul di Library",
  stageRule: "Selesai berarti arsip sudah diekstrak utuh. Folder belum bisa dibuka selama proses ini.",
  unavailable: "Repositori tidak tersedia",
  backDetail: "Kembali ke detail repo",
  forkBanner: "Unduhan memakai fork milikmu. Repo asal tidak ikut diunduh.",
  forkMade: "Fork yang dibuat",
  waitingSlotShort: "Menunggu slot",
  destFolder: "Folder tujuan",
  forkOk: "Fork berhasil.",
  forkOkDownload: "Fork berhasil. Unduhan siap dimulai.",
  forkActionResult: "Hasil tindakan {action} untuk {repo}.",
  forkCreated: "{name} berhasil dibuat",
  yourFork: "Fork milikmu",
  branch: "Branch",
  forkQueueNote: "Unduhan dimulai saat slot tersedia. Progres muncul di antrean yang sama dengan repo lainnya.",
  forkPersist: "Hasil unduhan baru masuk Library setelah proses unduh dan ekstraksi selesai.",
  backDiscover: "Kembali ke Discover",
  openOnGithub: "Buka di GitHub",
  openFork: "Buka fork di GitHub",
  aboutRepo: "Tentang repo",
  youGet: "Yang akan kamu dapatkan",
  getSnapshot: "Snapshot isi repo pada branch yang dipilih",
  getExtract: "Hasil ekstraksi di folder unduhanmu",
  getNoGit: "Histori Git dan folder .git tidak disertakan",
  origin: "Repositori asal",
  statusRunning: "Mengunduh",
  statusPaused: "Dijeda",
  statusWaiting: "Menunggu",
  statusFailed: "Gagal",
  statusDone: "Selesai",
  statusCancelled: "Dibatalkan",
  waitingSlot: "Menunggu slot unduhan tersedia",
  downloaded: "{size} terunduh",
  remaining: "sisa {seconds} detik",
  totalUnknown: "ukuran total belum tersedia",
  progressSaved: "progres tersimpan",
  searchPlaceholder: "Cari nama repo, pemilik, atau topik…",
  searchAria: "Cari",
  clearSearch: "Hapus pencarian",
  languageAria: "Bahasa",
  allLanguages: "Semua bahasa",
  topicAria: "Topik",
  allTopics: "Semua topik",
  periodDay: "Hari ini",
  periodWeek: "Minggu ini",
  periodMonth: "Bulan ini",
  byStars: "·  berdasarkan star",
  justUpdated: "Baru diperbarui",
  resetFilters: "Reset filter",
  filterNote: "Hasil memakai kata kunci, bahasa, topik GitHub, dan periode yang dipilih.",
  searchResults: "Hasil pencarian",
  popularDay: "Populer hari ini",
  popularWeek: "Populer minggu ini",
  popularMonth: "Populer bulan ini",
  repoCount: "{count} repo",
  emptyDiscoverEyebrow: "Discover / Tidak ada hasil",
  emptyDiscoverHeading: "Belum menemukan repo yang cocok.",
  emptyDiscoverHint: "Coba kata kunci lain atau atur ulang filter bahasa dan topik.",
  limitEyebrow: "Discover / Batas GitHub",
  discoverEyebrow: "Discover",
  tryAgain: "Coba lagi",
  clearSelection: "Batalkan pilihan",
  addSelected: "Tambah ke antrean",
  selected: "{count} dipilih",
  limitMessage: "Batas GitHub tercapai, coba lagi nanti",
  failedSearch: "Pencarian tidak bisa dijalankan.",
  limitHint: "Coba setelah kuota direset. Unduhan dalam antrean tetap berjalan.",
  noDescription: "Tidak ada deskripsi",
  takeTitle: "Bawa ke ruang kerjamu",
  hintDownload: "Simpan ke perangkat",
  hintQueue: "Masukkan ke antrean unduhan",
  hintFork: "Salin ke akun GitHub",
  hintForkDownload: "Unduh hasil fork milikmu",
  download: "Unduh",
  onlyDownload: "Hanya unduh",
  onlyFork: "Hanya fork",
  forkThenDownload: "Fork, lalu unduh",
  addToQueue: "Tambah ke antrean",
  permit: "Fork hanya dibuat jika kamu memilih tindakan fork.",
  queuedNote: "Masuk antrean unduhan.",
  alreadyQueued: "Repo ini sudah ada di antrean.",
  queueFailed: "Tidak bisa masuk antrean.",
  ownRepo: "Ini repo kamu. Unduh saja.",
  forkFailed: "Fork tidak dibuat.",
  publicChip: "Public",
  pickRepo: "Pilih {name}",
  folderTitle: "Folder unduhan",
  appearance: "Tampilan",
  themeLegend: "Tema",
  themeSystem: "Ikuti sistem",
  themeLight: "Terang",
  themeDark: "Gelap",
  language: "Bahasa",
  githubTitle: "GitHub",
  clientId: "Client id GitHub",
  tagline: "Dibuat untuk repo, disimpan di perangkatmu.",
  version: "Peti 0.1.0",
  notConnected: "Belum terhubung",
  publicStill: "Unduh publik tetap bisa. Masuk untuk membuat fork.",
  connected: "Terhubung",
  logout: "Keluar",
  connect: "Hubungkan GitHub",
  unprepared: "Masuk GitHub belum disiapkan.",
  denied: "Masuk dibatalkan.",
  expired: "Kode masuk kedaluwarsa. Coba lagi.",
  notRun: "Masuk tidak bisa dijalankan.",
  deviceHint: "Buka {url} lalu masukkan kode ini.",
  deviceBack: "Kembali ke Settings",
  deviceTitle: "Hubungkan akun GitHub",
  deviceStep: "Masuk lewat browsermu.",
  deviceBody: "Salin kode di bawah, buka GitHub, lalu izinkan Peti terhubung ke akunmu.",
  copyCode: "Salin kode",
  openGithub: "Buka GitHub",
  waitingBrowser: "Menunggu izin di browser…",
  afterConnect: "Setelah terhubung",
  asideFork: "Buat fork ke akun GitHub-mu",
  asideDownload: "Unduh hasil fork yang dipilih",
  asideChoose: "Pilih sendiri setiap tindakan fork",
  asideNote: "Kamu selalu bisa kembali dan melanjutkan unduhan tanpa menghubungkan akun.",
  libraryEmptyTitle: "Library-mu masih kosong.",
  libraryEmptyText: "Selesaikan unduhan pertama untuk melihat repo, folder penyimpanan, dan waktu selesainya di sini.",
  startDownload: "Mulai unduhan",
  libraryNote: "Unduhan yang berjalan, dijeda, atau gagal tetap berada di Downloads.",
  reposDownloading: "{count} repo sedang diunduh",
  reposReady: "{count} repo siap dibuka",
  failAttention: "{count} unduhan perlu perhatian",
  otherStillRunning: "{count} unduhan lain masih berjalan",
  waitingCount: "{count} menunggu",
  pausedCount: "{count} dijeda",
  retryCount: "{count} perlu dicoba ulang",
  addHintDownload: "Simpan isi repo ke perangkatmu.",
  addHintFork: "Buat salinan repo di akun GitHub.",
  addHintForkDownload: "Unduh dari fork yang baru dibuat.",
  reviewSnapshot: "Isi repo diunduh sebagai snapshot, tanpa histori Git (.git).",
  reviewFolder: "Folder yang sudah ada tetap aman. Peti memakai nama folder baru.",
  reviewStarts: "Unduhan dimulai saat slot tersedia.",
  forkMadeName: "{name} dibuat",
};

export const en: Messages = {
  navAria: "Main",
  sidebarAria: "Sidebar",
  folderDownloads: "Download folder",
  noAccount: "No account",
  accountConnected: "GitHub connected",
  accountConnect: "Connect GitHub",
  navHome: "Home",
  navDownloads: "Downloads",
  navDiscover: "Discover",
  navLibrary: "Library",
  navSettings: "Settings",
  sectionWorkspace: "Workspace",
  sectionExplore: "Explore",
  sectionLocal: "Local",
  homeTitle: "Welcome back.",
  homeWelcome: "Welcome to Peti.",
  homeEmpty: "No download is running.",
  homeAction: "Open Downloads",
  downloadsTitle: "Downloads",
  downloadsEmpty: "No downloads yet. Paste a link to start.",
  downloadsAction: "Open Home",
  discoverTitle: "Find the next repo.",
  discoverEmpty: "No repos yet.",
  discoverAction: "Open Downloads",
  libraryTitle: "Library",
  libraryEmptyPage: "No finished downloads yet.",
  libraryAction: "Open Downloads",
  settingsTitle: "Settings",
  settingsEmpty: "No saved settings yet.",
  settingsAction: "Open Home",
  running: "{count} downloads running",
  waitingRepos: "{count} repos waiting",
  failedCount: "{count} failed",
  recentTitle: "Recently downloaded",
  openFolder: "Open folder",
  openLibrary: "Open Library",
  seeDownloads: "See Downloads",
  noPick: "No repo in mind yet?",
  exploreDiscover: "Explore Discover",
  whatRepo: "Which repo do you want to download?",
  repoPlaceholder: "github.com/owner/repo",
  linkAria: "GitHub link",
  changeFolder: "Change folder",
  pasteClipboard: "Paste from clipboard",
  pasteMany: "One link or many at once. You will see them checked before it starts.",
  stepsTitle: "From a link to a local folder.",
  stepPaste: "Paste a public repo link",
  stepFolder: "Choose a folder and start the download",
  stepOpen: "Open the result from Library",
  discoverCardText: "Find projects by language, topic, and popular repos in Discover.",
  githubOnly: "Sign in to GitHub only when you need it.",
  githubPublic: "Download public repos without an account. Connect later to create a fork.",
  openSettings: "Open Settings",
  todayAt: "Today, {clock}",
  folderUnusable: "That folder cannot be used, choose another",
  composerTitle: "Start with a GitHub link.",
  composerHint: "More than one repo? Paste every link at once.",
  downloadRepo: "Download repo",
  changeShort: "Change",
  addRepo: "Add repo",
  retryAllFailed: "Retry all failed",
  pastePlaceholder: "Paste a GitHub link to add a repo…",
  emptyEyebrow: "Downloads / Empty",
  emptyTitle: "Your first repo starts here.",
  emptyDesc: "Paste a GitHub link to save the repo onto your device.",
  pauseAll: "Pause all",
  tabAll: "All",
  tabRunning: "Running",
  tabWaiting: "Waiting",
  tabDone: "Done",
  tabFailed: "Failed",
  seeAllLibrary: "See everything in Library",
  doneNote: "Finished downloads stay in Library after you leave this page.",
  failNote: "Each item is handled on its own. Retry downloads that repo from the start.",
  linksTitle: "Repository links",
  linksHint: "One link per line",
  linksAria: "Repository links",
  whatNext: "What do you want to do?",
  saveTo: "Save to",
  readyQueue: "Ready for the queue",
  addSubmit: "Add",
  addSubmitCount: "Add {count} repos",
  linksReady: "{count} links ready to add",
  backDownloads: "Back to Downloads",
  pause: "Pause",
  cancel: "Cancel",
  resume: "Resume",
  retry: "Retry",
  pauseDownload: "Pause download",
  cancelDownload: "Cancel",
  inspectLink: "Check link",
  source: "Source",
  branchRef: "Branch / ref",
  formatLabel: "Format",
  formatValue: "Repo snapshot, without Git history",
  stagesTitle: "Download stages",
  stageChecked: "Repo checked",
  stageCheckedDesc: "Public repo link is valid",
  stageArchive: "Downloading archive",
  stageArchiveDesc: "Receiving the repository contents",
  stageExtract: "Extracting",
  stageExtractDesc: "Starts after the download finishes",
  stageReady: "Ready to open",
  stageReadyDesc: "Folder shows up in Library",
  stageRule: "Done means the archive was extracted in full. The folder cannot be opened during this.",
  unavailable: "Repository is not available",
  backDetail: "Back to repo details",
  forkBanner: "The download uses your fork. The original repo is not downloaded.",
  forkMade: "Fork created",
  waitingSlotShort: "Waiting for a slot",
  destFolder: "Destination folder",
  forkOk: "Fork created.",
  forkOkDownload: "Fork created. The download is ready to start.",
  forkActionResult: "Result of {action} for {repo}.",
  forkCreated: "{name} was created",
  yourFork: "Your fork",
  branch: "Branch",
  forkQueueNote: "The download starts when a slot is free. Progress shows in the same queue as other repos.",
  forkPersist: "A finished download appears in Library only after the download and extraction are done.",
  backDiscover: "Back to Discover",
  openOnGithub: "Open on GitHub",
  openFork: "Open fork on GitHub",
  aboutRepo: "About this repo",
  youGet: "What you will get",
  getSnapshot: "A snapshot of the repo on the selected branch",
  getExtract: "Extracted files in your download folder",
  getNoGit: "Git history and the .git folder are not included",
  origin: "Original repository",
  statusRunning: "Downloading",
  statusPaused: "Paused",
  statusWaiting: "Waiting",
  statusFailed: "Failed",
  statusDone: "Done",
  statusCancelled: "Cancelled",
  waitingSlot: "Waiting for a free download slot",
  downloaded: "{size} downloaded",
  remaining: "{seconds} seconds left",
  totalUnknown: "total size not available yet",
  progressSaved: "progress saved",
  searchPlaceholder: "Search repo name, owner, or topic…",
  searchAria: "Search",
  clearSearch: "Clear search",
  languageAria: "Language",
  allLanguages: "All languages",
  topicAria: "Topic",
  allTopics: "All topics",
  periodDay: "Today",
  periodWeek: "This week",
  periodMonth: "This month",
  byStars: "·  by stars",
  justUpdated: "Just updated",
  resetFilters: "Reset filters",
  filterNote: "Results use the keyword, language, GitHub topic, and period you chose.",
  searchResults: "Search results",
  popularDay: "Popular today",
  popularWeek: "Popular this week",
  popularMonth: "Popular this month",
  repoCount: "{count} repos",
  emptyDiscoverEyebrow: "Discover / No results",
  emptyDiscoverHeading: "No matching repo yet.",
  emptyDiscoverHint: "Try another keyword or reset the language and topic filters.",
  limitEyebrow: "Discover / GitHub limit",
  discoverEyebrow: "Discover",
  tryAgain: "Try again",
  clearSelection: "Clear selection",
  addSelected: "Add to queue",
  selected: "{count} selected",
  limitMessage: "GitHub limit reached, try again later",
  failedSearch: "Search could not run.",
  limitHint: "Try again after the quota resets. Queued downloads keep going.",
  noDescription: "No description",
  takeTitle: "Bring it into your workspace",
  hintDownload: "Save to this device",
  hintQueue: "Put it on the download queue",
  hintFork: "Copy to your GitHub account",
  hintForkDownload: "Download your own fork",
  download: "Download",
  onlyDownload: "Download only",
  onlyFork: "Fork only",
  forkThenDownload: "Fork, then download",
  addToQueue: "Add to queue",
  permit: "A fork is created only if you choose a fork action.",
  queuedNote: "Added to the download queue.",
  alreadyQueued: "This repo is already in the queue.",
  queueFailed: "Could not add it to the queue.",
  ownRepo: "This is your repo. Just download it.",
  forkFailed: "The fork was not created.",
  publicChip: "Public",
  pickRepo: "Select {name}",
  folderTitle: "Download folder",
  appearance: "Appearance",
  themeLegend: "Theme",
  themeSystem: "Use system",
  themeLight: "Light",
  themeDark: "Dark",
  language: "Language",
  githubTitle: "GitHub",
  clientId: "GitHub client id",
  tagline: "Built for repos, kept on your device.",
  version: "Peti 0.1.0",
  notConnected: "Not connected",
  publicStill: "Public downloads still work. Sign in to create a fork.",
  connected: "Connected",
  logout: "Log out",
  connect: "Connect GitHub",
  unprepared: "GitHub sign-in is not set up.",
  denied: "Sign-in was cancelled.",
  expired: "The sign-in code expired. Try again.",
  notRun: "Sign-in could not run.",
  deviceHint: "Open {url} and enter this code.",
  deviceBack: "Back to Settings",
  deviceTitle: "Connect a GitHub account",
  deviceStep: "Sign in with your browser.",
  deviceBody: "Copy the code below, open GitHub, and allow Peti to connect to your account.",
  copyCode: "Copy code",
  openGithub: "Open GitHub",
  waitingBrowser: "Waiting for permission in the browser…",
  afterConnect: "After you connect",
  asideFork: "Create a fork on your GitHub account",
  asideDownload: "Download the fork you chose",
  asideChoose: "Choose each fork action yourself",
  asideNote: "You can always go back and keep downloading without connecting an account.",
  libraryEmptyTitle: "Your Library is still empty.",
  libraryEmptyText: "Finish a first download to see the repo, the folder it was saved in, and when it finished.",
  startDownload: "Start a download",
  libraryNote: "Running, paused, and failed downloads stay in Downloads.",
  reposDownloading: "{count} repos downloading",
  reposReady: "{count} repos ready to open",
  failAttention: "{count} downloads need attention",
  otherStillRunning: "{count} other downloads are still running",
  waitingCount: "{count} waiting",
  pausedCount: "{count} paused",
  retryCount: "{count} to retry",
  addHintDownload: "Save the repo onto your device.",
  addHintFork: "Make a copy of the repo on your GitHub account.",
  addHintForkDownload: "Download from the fork just created.",
  reviewSnapshot: "The repo is downloaded as a snapshot, without Git history (.git).",
  reviewFolder: "Folders that already exist stay safe. Peti uses a new folder name.",
  reviewStarts: "The download starts when a slot is free.",
  forkMadeName: "{name} created",
};

export const catalogs: Record<LocaleId, Messages> = {
  en,
  hi: en,
  zh: en,
  pt: en,
  "en-GB": en,
  ja: en,
  id,
  de: en,
  ru: en,
  fr: en,
};
