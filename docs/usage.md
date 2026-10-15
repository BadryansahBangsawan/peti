# Penggunaan

Peti punya lima halaman: Home, Downloads, Discover, Library, dan Settings. Nama navigasi itu tetap dalam bahasa Inggris. Teks lain mengikuti bahasa yang dipilih.

## Memilih folder

Unduhan pertama meminta folder tujuan. Folder itu disimpan sebagai `default_folder` dan dipakai lagi. Anda bisa menggantinya dari Settings.

Peti membuat subfolder `{pemilik}-{repo}` di dalam folder itu. Jika nama itu sudah ada, nama berikutnya adalah `{pemilik}-{repo}-2` sampai `-1000`. Isi folder yang sudah ada tidak ditimpa.

## Menempel tautan

Di Downloads, tempel satu atau banyak tautan yang dipisah spasi. Contoh yang diterima:

```text
https://github.com/octocat/Hello-World
http://www.github.com/octocat/Hello-World.git
github.com/octocat/Hello-World/tree/main
```

Query dan fragment diabaikan. Satu sufiks `.git` dibuang. `/tree/<ref>` boleh berisi garis miring. Tautan SSH (`git@github.com:...`) ditolak.

Token yang tidak lolos tidak masuk antrean. Alasannya berbentuk `Tautan tidak valid, {token}`.

Repositori yang sama, dengan ref yang sama, tidak dimasukkan lagi selama statusnya `menunggu` atau `berjalan`. Perbandingan pemilik dan nama repositori tidak membedakan huruf besar.

## Antrean

Status yang tersimpan: `menunggu`, `berjalan`, `dijeda`, `selesai`, `gagal`, `dibatalkan`.

- Jeda menyimpan byte yang sudah ada.
- Lanjut hanya untuk status `dijeda`. Jika slot penuh, pesannya `slot unduhan penuh`.
- Batal menghapus folder yang belum selesai dan mempertahankan folder yang sudah selesai. Baris yang sudah `dibatalkan` tetap di daftar sampai tombol silang ditekan.
- Tombol silang pada baris `dibatalkan` menghapus baris itu dari antrean. Folder hasil dan riwayat Library tidak dihapus.
- Ulang mengantrekan kembali item yang gagal.

Jika aplikasi ditutup saat sebuah baris masih `berjalan`, baris itu menjadi `gagal` dengan alasan `ditutup saat berjalan` saat basis data dibuka lagi. Status `dijeda` tetap.

Unduhan memakai `https://github.com/{pemilik}/{repo}/archive/{ref}.zip`. Ref kosong memakai `HEAD`. Pengalihan hanya diikuti ke `github.com` atau `codeload.github.com` pada HTTPS port 443, paling banyak lima kali. Lanjut unduhan memakai header `Range` hanya jika ada berkas parsial dan total yang cocok. Respons 200 yang mengabaikan `Range` mengganti berkas parsial.

Ekstraksi menolak arsip yang deklarasi totalnya melebihi 8 GiB, jalur yang keluar dari folder tujuan, atau tujuan yang tidak kosong. Jika zip berisi satu direktori pembungkus, direktori itu ditanggalkan.

## Discover

Filter bahasa, topik, dan periode (hari ini, minggu ini, bulan ini). Periode yang tidak dikenal diperlakukan sebagai minggu. Pencarian tidak diautentikasi.

Hasil kosong karena batas API menampilkan `Batas GitHub tercapai, coba lagi nanti`. Kegagalan lain menampilkan `Pencarian tidak bisa dijalankan.`

Memilih satu atau lebih repositori menampilkan tombol unduh mengambang di tengah bawah. Tombol itu hanya ada selama ada pilihan dan daftar Discover terbuka, bukan saat detail repositori terbuka. Menekannya memasukkan repo yang dipilih ke antrean. Dari detail repositori Anda bisa mengunduh, memasukkan antrean, atau meminta fork.

## Masuk GitHub dan fork

Masuk memakai [Device Flow](https://github.com/login/device). Peti membuka halaman itu dan menampilkan kode. Cakupan yang diminta hanya `public_repo`. Tidak ada client secret.

Tanpa client id yang lolos pemeriksaan, pesannya `Masuk GitHub belum disiapkan.` Masuk yang dibatalkan: `Masuk dibatalkan.` Kode yang kedaluwarsa: `Kode masuk kedaluwarsa. Coba lagi.` Kegagalan lain: `Masuk tidak bisa dijalankan.`

Tindakan fork:

- `download` mengunduh repositori yang dipilih.
- `fork` membuat fork ke akun yang masuk.
- `fork-download` membuat fork lalu mengunduh fork itu.

Repositori milik akun yang sedang masuk tidak di-fork. Nama fork yang berakhiran `.git` tidak diantrekan. Jika folder tujuan belum ada, Peti meminta folder dan belum mengantrekan.

Keluar menghapus entri keyring untuk token dan login.

## Library

Library hanya menampilkan baris yang foldernya masih ada, berupa direktori biasa, dan bukan symlink. Menghapus folder di luar Peti menyembunyikan entri itu. Baris di tabel `history` tetap ada.

Membuka folder memakai `open` di macOS, `explorer` di Windows, dan `xdg-open` di Unix lain. Perilaku Windows dan Linux **perlu dikonfirmasi** karena tidak dijalankan pada sesi ini.

## Bahasa dan tema

Keduanya langsung dipakai, tanpa memuat ulang jendela. Pilihan bahasa: English, हिन्दी, 中文, Português, English (UK), 日本語, Bahasa Indonesia, Deutsch, Русский, Français. Bawaan `id`. Teks versi di setiap bahasa adalah `Peti 0.1.0`.

`zh` memakai `lang` HTML `zh-Hans`. `pt` memakai `pt-BR`.
