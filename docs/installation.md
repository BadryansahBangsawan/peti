# Instalasi

Direktori kerja semua perintah di halaman ini adalah akar repositori, tempat `package.json` berada.

## Penginstal macOS

Belum ada rilis yang diunggah. Setelah tag `v*` di-push, `.github/workflows/release.yml` menjalankan `npm run tauri build` di macOS dan mengunggah `Peti.app.zip` serta `.dmg` dari `src-tauri/target/release/bundle/dmg/`.

DMG bawaan Tauri memuat `Peti.app` dan tautan Applications. `tauri.conf.json` tidak memuat identitas Developer ID dan tidak ada notarization. Buka pertama, setelah berkas itu ada:

```bash
xattr -cr /Applications/Peti.app
open /Applications/Peti.app
```

Cask Homebrew belum dibuat. Cask membutuhkan URL rilis dan sha256. Keduanya belum ada.

Sampai rilis pertama terbit, jalankan dari sumber seperti di bawah.

## Prasyarat

| Perkakas | Yang tertulis di repositori | Yang dipakai sesi ini |
| --- | --- | --- |
| Node.js | Tidak ada field `engines` | v26.9.0 |
| npm | Mengunci dependensi lewat `package-lock.json` | bersama Node di atas |
| Rust | `rust-version = "1.90"`, edition 2024 | `rustc 1.99.0-nightly (1a98b1e13 2026-08-07)` |
| Tauri 2 | CLI `@tauri-apps/cli` 2.12.2 | `npm run tauri build` lulus pada 15 Oktober 2026 |

**Perlu dikonfirmasi:** versi Node.js minimum. Tauri 2 juga membutuhkan kompiler C dan, di macOS, perkakas Apple. Paket sistem per OS mengikuti dokumentasi Tauri 2. Mesin bersih, Windows, dan Linux tidak diuji pada penulisan dokumen ini.

## Pengguna

```bash
git clone https://github.com/BadryansahBangsawan/peti.git
cd peti
npm install
npm run tauri dev
```

`npm install` memasang dependensi dari `package-lock.json`. `npm run tauri dev` menjalankan `beforeDevCommand` (`npm run dev`) lalu membuka jendela desktop yang memuat `http://localhost:1420`.

Jangan memakai `npm run dev` sendirian jika Anda ingin mengunduh atau mencari. Perintah itu hanya menyalakan Vite.

## Kontributor

Langkahnya sama dengan bagian pengguna. Pemeriksaan sebelum mengubah kode ada di [Pengembangan](development.md).

## Yang tidak dipasang

- Tidak ada skrip lint.
- Tidak ada updater di `src-tauri/tauri.conf.json`.
- `.github/workflows/ci.yml` menjalankan `npm test` dan `cargo test` pada push ke `main` dan pada pull request. Workflow itu belum dijalankan di GitHub pada penulisan dokumen ini.
- `.github/workflows/release.yml` hanya berjalan pada tag `v*`. Tag itu belum di-push.
