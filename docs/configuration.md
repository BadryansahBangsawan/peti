# Konfigurasi

Peti menyimpan pengaturan di SQLite, berkas `peti.db`, di direktori data aplikasi Tauri untuk identifier `app.peti`. Pada macOS yang dipakai sesi ini, jalur itu adalah `~/Library/Application Support/app.peti/peti.db`. Jalur Windows dan Linux **perlu dikonfirmasi**; sesi ini tidak menjalankannya.

Hanya ada satu baris pengaturan, `id = 1`. Token akses tidak ada di tabel ini. Token dan login GitHub disimpan di keyring sistem, layanan `app.peti`, akun `github-token` dan `github-login`.

Jangan menyalin isi `peti.db` atau keyring ke issue.

## Pengaturan

| Nama | Kegunaan | Wajib | Bawaan | Contoh aman |
| --- | --- | --- | --- | --- |
| `default_folder` | Folder induk unduhan | Opsional sampai unduhan dimulai | kosong | `/tmp/peti-contoh` |
| `concurrency` | Jumlah unduhan yang boleh berjalan bersama | Wajib di basis data | `3` | `3` |
| `theme` | Tema antarmuka | Wajib di basis data | `system` | `system`, `light`, atau `dark` |
| `language` | Bahasa antarmuka | Wajib di basis data | `id` | `id` |
| `github_client_id` | Client id OAuth App untuk Device Flow | Opsional; wajib hanya untuk masuk GitHub | kosong | `Iv1.contoh-bukan-rahasia` |
| `PETI_GITHUB_CLIENT_ID` | Mengganti client id saat proses dimulai, tanpa menulis basis data | Opsional | kosong, lalu nilai di basis data | `Iv1.contoh-bukan-rahasia` |
| `TAURI_DEV_HOST` | Host server Vite saat pengembangan | Opsional | tidak disetel; Vite tidak mengikat host luar | `127.0.0.1` |

`concurrency` harus 1–8. Nilai di luar itu ditolak dengan `jumlah bersamaan di luar 1 sampai 8`. Halaman Settings tidak menampilkan pengatur angka ini. Backend tetap memakai nilai yang tersimpan.

`theme` hanya `system`, `light`, atau `dark`. Nilai lain ditolak dengan `tema tidak dikenal`.

`language` hanya `en`, `hi`, `zh`, `pt`, `en-GB`, `ja`, `id`, `de`, `ru`, atau `fr`. Nilai lain ditolak dengan `bahasa tidak dikenal`. `en-GB` tersimpan terpisah dari `en` dan memakai katalog teks yang sama.

Client id yang tidak kosong harus 1–128 karakter ASCII: huruf, angka, `.`, `_`, atau `-`. Nilai yang tidak lolos ditolak dengan `Masuk GitHub belum disiapkan.` String kosong menghapus nilai tersimpan. Peti tidak memakai client secret.

Variabel lingkungan dibaca sekali saat proses dimulai. Jika `PETI_GITHUB_CLIENT_ID` tidak kosong setelah dipangkas, nilai itu dipakai dan basis data tidak diubah. Jika kosong, Peti membaca `github_client_id`.

Kolom client id di Settings disembunyikan sampai nilainya sudah terisi, atau sampai pesan `Masuk GitHub belum disiapkan.` muncul.

`TAURI_DEV_HOST` hanya dibaca `vite.config.ts`. Jika disetel, Vite mengikat host itu dan HMR memakai WebSocket di port 1421. Port aplikasi tetap 1420 dan `strictPort` menyala.
