# Desain

Nama Peti dan tandanya sudah ada. File ini tidak mengusulkan tanda baru. Tanda di `brand/logo/slot-symbol-v1.svg` dipakai sebagai ikon jendela. Warna tanda di dalam aplikasi mengikuti tema: tinta gelap di atas latar terang, tinta terang di atas latar gelap. Bukan biru GitHub.

## Yang harus terasa

Rapi, sedikit, dan sama di setiap halaman. Ruang kosong lebih penting daripada kartu di dalam kartu. Sudut rounded satu ukuran untuk tombol dan bidang, satu ukuran sedikit lebih besar untuk kartu. Dark mode warna solid: latar, permukaan satu tingkat lebih terang, teks dua tingkat, satu aksen. Tidak ada gradasi sebagai latar atau sebagai tombol.

Light mode memakai pasangan solid yang sama, bukan tema terpisah dengan bentuk lain.

Tipografi tiga tingkat saja: judul halaman, nama repo, keterangan. Font bawaan sistem, supaya Windows, macOS, dan Arch tidak bergantung pada font yang disisipkan.

Ikon satu set, garis, ukuran kecil, hanya untuk navigasi dan tindakan. Tidak untuk menghias baris.

Gerak singkat, sekitar sepersepuluh detik, pada munculnya batang pilihan dan pergantian keadaan. Tidak ada pantulan. Jika sistem meminta gerak dikurangi, perpindahan langsung jadi.

## Jendela

Navigasi di kiri, isi di kanan. Lima tujuan, satu tingkat: Home, Downloads, Discover, Library, Settings. Yang sedang dibuka terlihat dari warna permukaan, bukan dari gradasi.

Lebar nyaman dimulai sekitar 960 piksel. Di bawah itu, kiri menyusut menjadi ikon. Tidak ada tata letak ketiga untuk tablet atau ponsel.

## Keadaan yang wajib ada di setiap halaman

| Keadaan | Isinya |
|---|---|
| Kosong | Apa yang belum ada, dan satu tindakan berikutnya |
| Memuat | Bentuk baris yang tenang, bukan spinner sebesar halaman |
| Berhasil | Hasilnya sendiri. Pesan sukses hanya untuk tindakan yang tidak pindah halaman, seperti akun terhubung |
| Gagal | Apa yang salah dan satu tindakan perbaiki, sesuai tabel galat di `01` dan `02` |

## Yang diambil dari referensi

Dicari di Mobbin, lalu disaring. Yang dipakai polanya, bukan kulitnya.

- [Proton Drive](https://mobbin.com/screens/12f6528d-e78a-4540-9087-784c0baa9172) — daftar transfer yang tidak menutup halaman, tab status, tiap baris punya ukuran dan kecepatan. Untuk Downloads.
- [AWS, status unggah](https://mobbin.com/screens/d0b2a1fa-99a9-4c29-ba83-f77a3f850d40) — ringkasan di atas (persen, sisa waktu, kecepatan, batal) lalu daftar per item dengan status. Susunan informasinya dipakai. Biru penuh dan tabel rapatnya tidak.
- [Browserbase](https://mobbin.com/screens/654392d0-9063-4db4-8987-6b7fc6742537) — sidebar kiri pendek, satu tingkat, konten kanan yang lapang. Untuk kerangka jendela.
- [Sana](https://mobbin.com/screens/77f9f4bc-2d72-40bd-a645-9c1260fa43e5) — cari di atas, saringan sebagai chip, hasil berupa baris. Untuk Discover.
- [GoFundMe](https://mobbin.com/screens/d08c8ea3-a8e0-4f30-a55e-427c6a8b2b5f) — periode sebagai pilihan terpisah (hari, minggu, bulan). Grid kartunya tidak dipakai.
- [Dropbox Dash](https://mobbin.com/screens/2b07de77-af07-4067-b7b2-c92886e4fbe8) dan [SoundCloud](https://mobbin.com/screens/4fffddb0-26ac-42b2-90cc-7b8c16b32ff5) — tema sebagai radio: ikut sistem, terang, gelap. Satu blok, bukan halaman pengaturan yang panjang.
- [Klaviyo](https://mobbin.com/screens/32aa0033-4f4f-4394-9f35-abea3c3442db) — akun sebagai satu baris dengan status terhubung dan satu tindakan.

## Yang tidak ditiru

- Jendela modal yang mengunci aplikasi hanya untuk menunggu unduhan. Antrean harus tetap kelihatan.
- Dashboard penuh grafik. Home Peti ringkasan unduhan, bukan analitik.
- Grid produk. Repo adalah baris.
- Gradasi, dan biru merek GitHub.

## Titik awal warna, bukan hukum merek

Dipilih saat langkah 3, boleh diganti di sana, tidak diganti di setiap halaman satu per satu.

- Gelap: latar `#161616`, permukaan `#222222`, teks `#f4f4f4`, teks sekunder `#a3a3a3`, aksen `#f4f4f4` untuk tindakan utama (tombol tinta terang, teks gelap).
- Terang: latar `#f7f7f7`, permukaan `#ffffff`, teks `#161616`, teks sekunder `#5c5c5c`, aksen `#161616`.
- Radius bidang 8, radius kartu 12.
- Tema bawaan: ikut sistem. Settings juga menyediakan terang dan gelap, sesuai radio di referensi.

Aksen bukan warna pelangi. Tindakan utama dibedakan oleh tinta solid, bukan oleh gradasi.
