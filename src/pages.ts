export type PageId = "home" | "downloads" | "discover" | "library" | "settings";

export type Page = {
  id: PageId;
  label: string;
  title: string;
  section: string;
  empty: string;
  action: { label: string; target: PageId };
};

export const pages: readonly Page[] = [
  {
    id: "home",
    label: "Home",
    title: "Selamat datang kembali.",
    section: "Workspace",
    empty: "Tidak ada unduhan yang berjalan.",
    action: { label: "Buka Downloads", target: "downloads" },
  },
  {
    id: "downloads",
    label: "Downloads",
    title: "Downloads",
    section: "Workspace",
    empty: "Belum ada unduhan. Tempel tautan untuk memulai.",
    action: { label: "Buka Home", target: "home" },
  },
  {
    id: "discover",
    label: "Discover",
    title: "Temukan repo berikutnya.",
    section: "Jelajahi",
    empty: "Belum ada repo.",
    action: { label: "Buka Downloads", target: "downloads" },
  },
  {
    id: "library",
    label: "Library",
    title: "Library",
    section: "Lokal",
    empty: "Belum ada unduhan yang selesai.",
    action: { label: "Buka Downloads", target: "downloads" },
  },
  {
    id: "settings",
    label: "Settings",
    title: "Settings",
    section: "Workspace",
    empty: "Belum ada pengaturan tersimpan.",
    action: { label: "Buka Home", target: "home" },
  },
];
