# Pengembangan

Direktori kerja adalah akar repositori.

## Perintah

| Perintah | Efek |
| --- | --- |
| `npm install` | Memasang dependensi terkunci |
| `npm run dev` | Vite saja, port 1420, `strictPort: true` |
| `npm run tauri dev` | Vite lalu jendela Tauri. Tidak dijalankan pada penulisan dokumen ini |
| `npm test` | `node --test src/*.test.ts` |
| `npx tsc --noEmit` | Pemeriksaan tipe frontend. Tes dikecualikan `tsconfig.json` |
| `npm run build` | `tsc && vite build`, hasil di `dist/` |
| `npm run preview` | Pratinjau hasil Vite, tanpa backend Rust |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Tes Rust |
| `npm run tauri build` | Build frontend lalu bundel. Lihat [Build distribusi](deployment.md) |
| tag `v*` | `.github/workflows/release.yml` membangun DMG dan mengunggahnya. Belum dijalankan |

`src-tauri` diabaikan oleh watch Vite. `TAURI_DEV_HOST` dijelaskan di [Konfigurasi](configuration.md).

## Tata letak

| Jalur | Isi |
| --- | --- |
| `src/main.ts` | Antarmuka, DOM imperatif |
| `src/i18n.ts` | Katalog 10 bahasa |
| `src/pages.ts` | Definisi lima halaman |
| `src/*.test.ts` | Tes frontend |
| `src-tauri/src/lib.rs` | Perintah Tauri dan worker antrean |
| `src-tauri/src/store.rs` | SQLite |
| `src-tauri/src/links.rs` | Parser tautan |
| `src-tauri/src/queue.rs` | Antrean |
| `src-tauri/src/download.rs` | Unduh zip |
| `src-tauri/src/archive.rs` | Ekstraksi |
| `src-tauri/src/discover.rs` | Pencarian GitHub |
| `src-tauri/src/github.rs` | Device Flow, keyring, fork |
| `src-tauri/src/reveal.rs` | Membuka folder |
| `index.html` | `<html lang="id">`, memuat `src/main.ts` |
| `rencana/`, `pencil/`, `brand/` | Catatan dan eksplorasi desain, bukan runtime |

## Konvensi

- Dependensi di `package.json` memakai versi tetap.
- `strict`, `noUnusedLocals`, dan `noUnusedParameters` menyala.
- Jangan menyimpan token di SQLite.
- Jangan menaikkan atau menghapus batas `concurrency` 1–8.
- Terjemahan baru masuk ke `src/i18n.ts`. Placeholder `{nama}` harus sama urutannya dengan katalog `id`.
- Pesan kesalahan backend yang sudah tampil ke pengguna dikunci tes.

## Pemeriksaan yang sudah dijalankan

Pada 15 Oktober 2026, dari akar repositori, di macOS:

- `npm test`: 32 lulus, 0 gagal.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 88 tes pustaka lulus.

`npm run tauri dev` tidak dijalankan. `npm run tauri build` lulus pada 15 Oktober 2026 dan menulis `Peti.app` serta `Peti_0.1.0_aarch64.dmg`. Lihat [Build distribusi](deployment.md).
