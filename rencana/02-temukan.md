# Temukan

Halaman ini untuk melihat repositori yang sedang populer, mencari, menyaring, membuka detail, lalu mengunduh atau memasukkan ke antrean. Perpindahan dari menemukan ke mengunduh tidak boleh terasa seperti aplikasi lain.

## Dari mana daftarnya

Bukan salinan github.com/trending. Dokumentasi REST yang dicek tidak punya endpoint untuk halaman itu.

Daftar bawaan memakai [GitHub Search API](https://docs.github.com/en/rest/search/search): repositori publik, diurutkan menurut jumlah star, dalam jendela waktu yang dipilih. Jendela bawaannya minggu ini.

Batas search: sampai 10 permintaan per menit tanpa masuk, 30 per menit setelah masuk. Peti tidak menembak ulang saat pengguna menggeser saringan dengan cepat. Permintaan terakhir yang menang, hasil sebelumnya tetap di layar sampai yang baru datang, dan hasil itu disimpan sebentar supaya membuka halaman lagi tidak langsung menghabiskan kuota. Sisa kuota dibaca dari header balasan. Kalau habis, halamannya mengatakan batas tercapai, bukan daftar kosong.

## Isi satu baris

- nama `pemilik/repo`
- deskripsi, atau "Tidak ada deskripsi" bila kosong
- bahasa, bila repo menyebutnya
- jumlah star
- jumlah fork, sebagai popularitas selain star

Bukan skor gabungan buatan. Bukan grid poster.

## Cari

Kotak cari mencari nama dan kata kunci lewat search yang sama. Kosong berarti daftar populer sesuai saringan, bukan halaman kosong.

## Saringan

Tiga saringan, semuanya opsional:

| Saringan | Arti di Peti |
|---|---|
| Bahasa | Bahasa utama repo |
| Kategori | Topik GitHub (`topic:`). GitHub tidak punya daftar kategori resmi |
| Periode | Hari ini, minggu ini, bulan ini |

Mengubah saringan memuat ulang daftar. Menghapus saringan kembali ke bawaan.

## Detail

Memilih satu repo membuka detail sebelum mengunduh: nama, deskripsi, bahasa, star, fork, dan tautan ke GitHub. Dari detail ada unduh langsung dan tambah ke antrean. Kalau sudah masuk GitHub, detail juga menawarkan tiga tindakan di `03-github.md`. Kalau belum masuk, yang ada hanya unduh.

## Unduh dari daftar

Tiap baris punya tindakan unduh. Itu memasukkan satu repo ke antrean dan memakai folder bawaan. Jika folder bawaan belum ada, Peti meminta folder dulu, sekali, lalu melanjutkan.

## Pilih banyak

- Tiap baris bisa dicentang.
- Begitu ada yang dicentang, muncul batang: jumlah yang dipilih, tambah ke antrean, dan bersihkan pilihan.
- Tambah ke antrean tidak langsung mengunduh di luar aturan antrean. Item masuk sebagai menunggu, lalu antrean yang menjalankannya.
- Pilihan tidak mengunci halaman. Pengguna tetap bisa mencari dan menyaring; centang yang sudah ada tetap sampai dibersihkan atau ditambahkan.

## Selesai bila

- Membuka Discover menampilkan daftar repo publik dengan nama, deskripsi, bahasa, star, dan fork.
- Mengubah periode atau bahasa mengubah daftar, tanpa menghabiskan kuota oleh ketukan beruntun.
- Kuota habis menampilkan pesan batas, bukan daftar kosong yang pura-pura "tidak ada repo".
- Detail kebuka sebelum unduh, dan unduh dari detail masuk antrean yang sama dengan unduhan dari tautan.
- Beberapa repo yang dicentang masuk antrean sekaligus, tanpa unduhan liar di luar batas bersamaan.
