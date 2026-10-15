# GitHub

Masuk bersifat opsional. Tanpa akun, unduh repo publik tetap utuh: tempel tautan, Discover, antrean, riwayat.

## Masuk dan keluar

Masuk memakai [device flow](https://docs.github.com/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps). Peti menampilkan kode dan membuka halaman GitHub. Pengguna mengizinkan di browser. Aplikasi tidak menyimpan client secret.

Token tidak ditulis di database unduhan dan tidak ditulis di file teks. Tempatnya penyimpanan rahasia sistem. Keluar menghapus token dari sana dan mengembalikan aplikasi ke mode tanpa akun. Antrean dan riwayat tidak ikut terhapus.

Versi ini memakai OAuth App dengan scope yang cukup untuk membuat fork repo publik. Nama scope dipastikan saat langkah 14, dari dokumentasi saat itu, bukan ditebak di dokumen ini.

## Tiga tindakan

Saat pengguna mengambil satu repo dan sedang masuk, pilihannya persis tiga:

| Tindakan | Yang terjadi | Yang tidak terjadi |
|---|---|---|
| Hanya unduh | Arsip repo itu masuk ke folder tujuan | Tidak ada panggilan fork |
| Hanya fork | Fork dibuat di akun yang masuk | Tidak ada unduhan |
| Fork, lalu unduh | Fork dibuat, Peti menunggu repo fork itu ada, lalu yang diunduh adalah fork itu, bukan repo asalnya | Tidak mengunduh asal sambil fork diam-diam |

Tanpa masuk, pilihan fork tidak ditawarkan. Yang ada hanya unduh. Tidak ada fork yang "ikut" karena pengguna kebetulan pernah masuk lalu lupa.

Fork hanya jalan setelah pengguna menekan salah satu dari dua tindakan yang memang fork. Tidak ada saat tempel tautan, tidak ada saat tambah ke antrean, tidak ada saat unduh langsung dari daftar.

## Hasil fork

- Berhasil: sebut `pemilik-kamu/repo` yang terbentuk.
- Fork lalu unduh: item antrean menunjuk ke fork itu. Progresnya sama dengan unduhan lain.
- Gagal: pesan yang mengatakan fork tidak dibuat, dan tidak ada unduhan pengganti yang diam-diam mengambil repo asal.
- Repo milik sendiri tidak di-fork. Pesannya mengatakan itu repo kamu, lalu menawarkan unduh saja.

## Yang tidak termasuk

Repo privat. Mengunduh atas nama akun untuk repo yang tidak publik. Menulis issue, star, atau mengubah repo. Menyimpan client secret di dalam aplikasi yang dibagikan.

## Selesai bila

- Tanpa masuk, unduh publik dari tautan dan dari Discover tetap jalan.
- Hanya unduh, saat sedang masuk, tidak membuat fork. Ini dicek dari tidak adanya repo baru di akun.
- Hanya fork tidak menulis folder di disk.
- Fork lalu unduh menulis isi fork, dan riwayat menyebut pemilik akun itu, bukan pemilik asal.
- Keluar lalu unduh lagi tidak membuat fork.
