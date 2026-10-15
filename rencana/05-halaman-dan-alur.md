# Halaman dan alur

Lima halaman. Navigasi kiri tidak berubah urutannya.

## Home

Ringkasan, bukan laporan.

- Berapa unduhan yang sedang berjalan, menunggu, dan gagal.
- Bidang tempel singkat. Mengirimnya membuka Downloads dengan tautan itu sudah masuk antrean, setelah folder jelas.
- Beberapa riwayat terbaru, dengan folder dan waktu.
- Kosong: tidak ada yang berjalan, satu kalimat, dan tindakan ke Downloads atau Discover.

## Downloads

Tempat antrean hidup.

- Bidang tempel, pilih folder, mulai.
- Tab: semua, berjalan, selesai, gagal.
- Ringkasan di atas daftar saat ada yang berjalan: jumlah, persen bila jujur, kecepatan, sisa waktu, dan batal untuk yang sedang dipilih atau untuk yang berjalan.
- Tiap baris: `pemilik/repo`, status, persen atau byte, kecepatan, sisa waktu, serta jeda, lanjut, batal, atau ulang sesuai status.
- Kosong: ajakan menempel tautan.
- Memuat: baris tenang saat daftar dibaca dari penyimpanan.
- Gagal: pesan di baris itu, plus ulang.

## Discover

Sesuai `02-temukan.md`.

- Daftar populer, cari, tiga saringan.
- Tiap baris: centang, identitas repo, unduh.
- Batang pilihan bila ada yang dicentang.
- Detail repo sebelum unduh.
- Kosong hanya bila pencarian memang tidak ketemu. Kuota habis bukan keadaan kosong.

## Library

Riwayat yang selesai.

- Nama, folder, waktu selesai.
- Buka folder di sistem.
- Kosong: belum ada yang selesai diunduh, dengan tautan ke Downloads.
- Tidak ada jeda atau ulang di sini. Yang belum selesai tinggal di Downloads.

## Settings

Empat blok, berurutan, tanpa tab lain.

1. Folder bawaan. Tampilkan path, tombol ganti.
2. Unduhan. Jumlah yang berjalan bersamaan, 1 sampai 8.
3. Tema. Radio: ikut sistem, terang, gelap.
4. GitHub. Satu baris. Belum masuk: tombol masuk. Sudah masuk: nama akun, status terhubung, tombol keluar.

Perubahan folder, jumlah, dan tema langsung dipakai. Tidak ada tombol simpan terpisah.

## Alur unduh langsung

Buka Peti, tempel satu atau banyak tautan, pilih folder, mulai, lihat progres di Downloads, buka hasil dari Library atau dari baris yang selesai.

Jika ada tautan tidak valid, yang valid tetap berjalan dan yang tidak valid disebut.

## Alur Discover

Buka Discover, jelajah atau cari, pilih satu atau beberapa, tambah ke antrean, unduhan berjalan menurut batas bersamaan, hasilnya muncul di Library.

Unduh satu dari baris adalah jalan pintas alur yang sama, bukan antrean kedua.

## Alur GitHub

Masuk dari Settings atau dari pilihan tindakan saat sedang masuk. Temukan repo atau tempel tautan. Pilih hanya unduh, hanya fork, atau fork lalu unduh. Peti menjalankan yang dipilih dan menampilkan hasilnya: folder untuk unduhan, nama fork untuk fork, keduanya untuk fork lalu unduh.

Kalau belum masuk, alur ini tidak menyisipkan fork.

## Selesai bila

Seseorang yang belum pernah membuka Peti bisa menyelesaikan ketiga alur tanpa penjelasan di luar aplikasi. Halaman yang bukan gilirannya tidak mengubah status unduhan.
