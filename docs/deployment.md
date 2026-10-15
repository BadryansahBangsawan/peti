# Build distribusi

Direktori kerja adalah akar repositori.

```bash
npm run tauri build
```

Perintah itu menjalankan `beforeBuildCommand` di `src-tauri/tauri.conf.json`, yaitu `npm run build` (`tsc && vite build`). Frontend yang dibundel adalah `../dist`. Jendela berjudul Peti, 1100×720, minimum 720×480. Identifier `app.peti`. Nama produk `Peti`. Versi `0.1.0`.

`bundle.targets` adalah `"all"`. Di Tauri 2 itu memilih format bundel yang didukung sistem operasi tempat perintah dijalankan, bukan installer untuk setiap sistem operasi sekaligus.

Ikon yang dirujuk konfigurasi:

- `src-tauri/icons/32x32.png`
- `src-tauri/icons/128x128.png`
- `src-tauri/icons/128x128@2x.png`
- `src-tauri/icons/icon.icns`
- `src-tauri/icons/icon.ico`

Profil rilis Rust di `src-tauri/Cargo.toml`: `lto = true`, `opt-level = 3`, `panic = "abort"`, `strip = true`, `codegen-units = 1`.

Tidak ada blok updater. Tidak ada artefak rilis yang sudah diunggah ke GitHub.

`.github/workflows/release.yml` berjalan saat tag `v*` di-push, di `macos-latest`, dengan Node.js 22. Langkahnya `npm ci` lalu `npm run tauri build`. Setelah itu workflow meng-zip `src-tauri/target/release/bundle/macos/*.app` menjadi `Peti.app.zip` dan mengunggah zip itu bersama `src-tauri/target/release/bundle/dmg/*.dmg` ke GitHub Release tag tersebut. Tidak ada identitas tanda tangan di konfigurasi, jadi hasilnya ad-hoc, bukan notarized.

`.github/workflows/ci.yml` tidak membuat DMG. Workflow itu menjalankan tes.

Pada 15 Oktober 2026, `npm run tauri build` dijalankan di Mac ini (Apple Silicon). Hasilnya:

- `src-tauri/target/release/bundle/macos/Peti.app` (arm64, 8.83 MiB)
- `src-tauri/target/release/bundle/dmg/Peti_0.1.0_aarch64.dmg` (4.27 MiB)

`Info.plist` hasil build memuat `CFBundleIdentifier` `app.peti` dan `LSMinimumSystemVersion` `10.13`. `codesign` pada biner melaporkan `Signature=adhoc`, tanpa Team ID. Bukan notarized.

Build Windows, Linux, dan Intel tidak dijalankan. Workflow rilis juga belum dijalankan di GitHub. Nama berkas di runner GitHub mengikuti versi di `tauri.conf.json` dan arsitektur runner. **Perlu dikonfirmasi** jika `macos-latest` suatu saat bukan Apple Silicon: nama `_aarch64` bisa berbeda.
