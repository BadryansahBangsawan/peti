# Antarmuka perintah

Peti tidak membuka API HTTP. Antarmuka di `src/main.ts` memanggil perintah Tauri yang didaftarkan di `src-tauri/src/lib.rs`. Nama di bawah adalah nama Rust. Argumen dan bidang balasan memakai camelCase karena struct-nya `rename_all = "camelCase"`.

Kesalahan perintah dikembalikan sebagai string. Kalimat yang dikunci kode disebut apa adanya.

## Antrean dan riwayat

### `queue_rows`

Tanpa argumen. Mengembalikan daftar `{ id, owner, repo, gitRef, status, error, bytes, total }`. `error` kosong jika tidak ada. `total` boleh `null`.

### `queue_paste`

Argumen: `paste` (string), `folder` (string).

Folder kosong memakai `default_folder`. Jika itu juga kosong, balasan `{ queued: 0, needFolder: true, rejected: [] }` dan tidak ada yang diantrekan. Folder yang tidak kosong disimpan sebagai folder bawaan.

Balasan: `{ queued, needFolder, rejected }`. `rejected` berisi `{ link, reason }`.

### `queue_pause`, `queue_cancel`

Argumen: `id` (bilangan). Tidak mengembalikan badan.

`queue_cancel` pada status `dibatalkan` tidak menghapus baris. Baris itu tetap di daftar.

### `queue_dismiss`

Argumen: `id`. Hanya menghapus baris berstatus `dibatalkan` dari tabel `queue`. Folder unduhan dan baris `history` tidak disentuh. Status lain: `hapus hanya untuk yang dibatalkan`. Id yang tidak ada: `item antrean tidak ada`.

### `queue_retry`

Argumen: `id`. Hanya untuk status `gagal`. Jika tidak, `ulang hanya untuk yang gagal`. Worker dibangunkan setelah status diubah.

### `queue_resume`

Argumen: `id`. Hanya untuk status `dijeda`. Jika tidak, `lanjut hanya untuk yang dijeda`. Jika jumlah yang `berjalan` sudah mencapai `concurrency`, `slot unduhan penuh`.

### `history_rows`

Tanpa argumen. Hanya baris yang foldernya lolos `reveal::real_directory`: ada, direktori, bukan symlink. Bidang: `{ id, owner, repo, gitRef, folder, finishedAt }`.

### `open_history_folder`

Argumen: `id`. Membuka folder itu. Gagal dengan `Folder tidak bisa dipakai, pilih yang lain` jika baris atau folder tidak layak.

## Pencarian

### `discover_search`

Argumen: `text`, `language`, `topic`, `period`.

Tidak melempar saat pencarian gagal. Balasan `{ repos, limited, failed }`. `repos` berisi `{ owner, repo, description, language, stars, forks }`. `limited: true` berarti batas GitHub. `failed: true` berarti pencarian tidak jalan. Keduanya tidak diisi bersamaan oleh kode saat ini.

`period`: `day`, `month`, selain itu diperlakukan sebagai `week`.

### `queue_discovered`

Argumen: `repos` (`{ owner, repo }[]`), `folder`.

Balasan `{ queued, needFolder }`. Folder kosong dan tidak ada folder bawaan menghasilkan `needFolder: true` dan `queued: 0`.

## Pengaturan

### `app_settings`

Balasan `{ defaultFolder, concurrency, githubClientId, language }`. String kosong jika folder atau client id belum ada.

### `set_download_settings`

Argumen: `folder`, `concurrency` (0–255 di batas tipe, ditolak di luar 1–8). Tema yang tersimpan tidak diubah.

### `saved_theme` / `set_saved_theme`

Membaca atau menulis `system`, `light`, atau `dark`. Nilai lain: `tema tidak dikenal`.

### `saved_language` / `set_saved_language`

Nilai yang diterima: `en`, `hi`, `zh`, `pt`, `en-GB`, `ja`, `id`, `de`, `ru`, `fr`. Selain itu: `bahasa tidak dikenal`.

### `set_github_client_id`

Argumen: `clientId`. Kosong menghapus nilai. Nilai tidak kosong yang gagal pemeriksaan: `Masuk GitHub belum disiapkan.` Nilai yang lolos juga dipasang ke sesi yang sedang berjalan.

## GitHub

Balasan status, mulai, poll, dan keluar berbentuk `{ connected, login, userCode, interval }`.

### `github_status`

Membaca keyring. Tidak memulai login.

### `github_begin`

Memulai Device Flow jika client id lolos dan belum ada sesi. Membuka `https://github.com/login/device` ketika kode baru diterima.

### `github_poll`

Melanjutkan poll sesuai interval GitHub.

### `github_logout`

Menghapus token dan login dari keyring.

Kesalahan login: `Masuk GitHub belum disiapkan.`, `Masuk dibatalkan.`, `Kode masuk kedaluwarsa. Coba lagi.`, `Masuk tidak bisa dijalankan.`

### `github_take`

Argumen: `owner`, `repo`, `action`, `folder`.

`action` hanya `download`, `fork`, atau `fork-download`. Nilai lain mengembalikan kesalahan `Fork tidak dibuat.`

Balasan `{ forked, queued, needFolder, ownRepo, failed }`. `ownRepo: true` berarti repositori milik akun yang masuk dan tidak di-fork. `needFolder: true` berarti tujuan belum ada.

## Unduhan

Kesalahan unduhan yang bisa tampil di antrean:

| Kalimat | Arti di kode |
| --- | --- |
| `Tautan tidak valid, {teks}` | Nama atau ref ditolak |
| `Repositori tidak tersedia` | Arsip tidak bisa diambil |
| `Folder tidak bisa dipakai, pilih yang lain` | Tujuan tidak layak |
| `Unduhan terputus. Ulang` | Transfer terhenti |
| `Unduhan tidak disimpan. Ulang` | Hasil tidak tertulis |
| `Batas GitHub tercapai, coba lagi nanti` | Batas API atau unduhan |
