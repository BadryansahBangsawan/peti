# Kontribusi

Dokumen ini menjelaskan cara mengubah kode yang ada. Perilaku aplikasi ditentukan oleh `src/` dan `src-tauri/`, bukan catatan di `rencana/` atau `pencil/`.

Belum ada berkas `LICENSE`. Mengirim perubahan tidak menetapkan izin redistribusi. Lihat [README](README.md).

## Melaporkan bug

Buka issue dan sertakan:

- versi yang tampil di Settings (`Peti 0.1.0` selama katalog bahasa belum diubah)
- sistem operasi
- langkah yang memicu masalah
- kalimat kesalahan yang tampil

Jangan tempel token GitHub, client secret, isi keyring, atau isi `peti.db`.

## Menyiapkan lingkungan

Prasyarat dan perintah ada di [Instalasi](docs/installation.md) dan [Pengembangan](docs/development.md). Dari akar repositori:

```bash
npm install
npm run tauri dev
```

## Sebelum pull request

Tidak ada skrip lint. `.github/workflows/ci.yml` menjalankan `npm test` dan `cargo test` pada push ke `main` dan pada pull request. Workflow itu belum dijalankan di GitHub. Pemeriksaan yang sama, dari akar repositori:

```bash
npm test
npx tsc --noEmit
cargo test --manifest-path src-tauri/Cargo.toml
```

`npm run build` menjalankan `tsc && vite build`. Tes di `src/**/*.test.ts` dikecualikan dari `tsc` lewat `tsconfig.json` dan dijalankan oleh `npm test`.

Yang perlu dijaga:

- Versi dependensi di `package.json` tetap, tanpa rentang `^`.
- TypeScript memakai `strict`, `noUnusedLocals`, dan `noUnusedParameters`.
- Token GitHub tidak masuk ke SQLite. Tes `schema_has_no_token_column` menjaga nama kolom.
- Batas unduhan bersamaan tetap 1–8. Jangan menghapus kolom atau pemeriksaan `concurrency`.
- Teks bawaan Bahasa Indonesia yang dikunci tes (`statusLabel`, `transferLine`, `detailTakes`) jangan diubah hanya untuk terjemahan. Terjemahan antarmuka ada di `src/i18n.ts`.
- Jangan mengirim `.env`, `dist/`, `src-tauri/target/`, atau basis data lokal.

## Gaya

Ikuti gaya berkas yang sedang diubah. Frontend membangun DOM secara imperatif di `src/main.ts`. Bukan React.

Pesan kesalahan yang tampil ke pengguna ditulis dalam Bahasa Indonesia di backend. Jangan mengganti kalimat itu tanpa memperbarui tes yang menguncinya.

## Pull request

Jelaskan perilaku yang berubah dan perintah yang sudah dijalankan. Template ada di `.github/PULL_REQUEST_TEMPLATE.md`.
