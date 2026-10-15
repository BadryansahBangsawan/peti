# Pemecahan masalah

Hanya masalah yang terlihat dari kode atau tes. Tidak ada issue yang dirujuk karena sesi ini tidak membaca issue GitHub.

## Pencarian dan unduhan tidak bereaksi di browser

`npm run dev` hanya menjalankan Vite. Backend Rust tidak ikut. Jalankan `npm run tauri dev` dari akar repositori.

## Port 1420 sudah dipakai

`vite.config.ts` memakai `strictPort: true` pada port 1420. Vite menolak port lain. Bebaskan port itu, atau jangan menjalankan dua `npm run dev` sekaligus.

## `Masuk GitHub belum disiapkan.`

Client id kosong, atau tidak lolos 1–128 karakter ASCII (huruf, angka, `.`, `_`, `-`). Isi client id OAuth App yang memakai Device Flow. Peti tidak meminta client secret. Contoh bentuk, bukan nilai sungguhan: `Iv1.contoh-bukan-rahasia`.

Kolom itu tersembunyi sampai pesan ini muncul atau sampai sebuah nilai sudah tersimpan.

## Masuk gagal setelah kode tampil

| Kalimat | Arti di kode |
| --- | --- |
| `Masuk dibatalkan.` | GitHub menolak pemberian akses |
| `Kode masuk kedaluwarsa. Coba lagi.` | Kode perangkat kedaluwarsa |
| `Masuk tidak bisa dijalankan.` | Permintaan atau keyring gagal |

Mulai ulang dari tombol masuk. Jangan menyalin token ke dalam issue.

## `Batas GitHub tercapai, coba lagi nanti`

Pencarian mendapat HTTP 403, 429, atau status lain saat sisa kuota 0. Unduhan memakai kalimat yang sama untuk batasnya sendiri. Tunggu, lalu coba lagi. Pencarian tidak memakai token.

## `Pencarian tidak bisa dijalankan.`

Permintaan Discover gagal di luar kasus batas: jaringan, status bukan 200, atau badan yang tidak bisa dibaca. Tidak ada hasil yang disimpan untuk kegagalan itu.

## Tautan ditolak

`Tautan tidak valid, {token}` berarti token itu bukan `github.com` atau `www.github.com` dengan pemilik dan repositori yang lolos, atau path setelah repo bukan `/tree/<ref>` yang valid. SSH tidak diterima.

`Repositori tidak tersedia` berarti arsip zip tidak bisa diambil setelah tautannya lolos parser.

## Folder

`Folder tidak bisa dipakai, pilih yang lain` berarti tujuan bukan direktori yang bisa ditulis, nama bentrok melewati `-1000`, atau folder yang akan dibuka hilang, bukan direktori, atau berupa symlink.

## Unduhan terhenti

`Unduhan terputus. Ulang` adalah transfer yang putus. `Unduhan tidak disimpan. Ulang` adalah hasil yang tidak tertulis. Keduanya bisa diulang dari antrean.

Item yang masih `berjalan` saat aplikasi ditutup menjadi `gagal` dengan `ditutup saat berjalan` pada pembukaan berikutnya. Item `dijeda` tetap dijeda.

## Lanjut atau ulang tidak jalan

`lanjut hanya untuk yang dijeda` berarti statusnya bukan `dijeda`. `slot unduhan penuh` berarti jumlah yang `berjalan` sudah sama dengan `concurrency` (1–8, bawaan 3).

`ulang hanya untuk yang gagal` berarti statusnya bukan `gagal`. `batal tidak menghapus yang sudah selesai` berarti baris itu sudah `selesai`; folder hasil tidak dihapus.

`hapus hanya untuk yang dibatalkan` berarti tombol silang dipakai pada baris yang belum `dibatalkan`. Silang pada baris `dibatalkan` hanya menghapus baris antrean itu.

## Selesai tidak muncul di Library

Library menyembunyikan baris jika folder dihapus, bukan direktori, atau symlink. Baris `history` tidak dihapus. Mengembalikan direktori yang sama di jalur yang tersimpan membuatnya tampil lagi.

## Fork tidak terjadi

`Fork tidak dibuat.` dikembalikan untuk `action` selain `download`, `fork`, dan `fork-download`. Repositori milik akun yang masuk tidak di-fork. Nama fork yang berakhiran `.git` tidak diantrekan. Jika folder belum dipilih, balasan meminta folder dan belum mengantrekan.
