# Pengunduh

Ini perilaku unduhan. Cara menyimpannya dan urutan mengerjakannya ada di langkah 4 sampai 10.

## Satu tautan

Pengguna menempel satu tautan repositori. Peti mengenali pemilik, nama repo, dan ref bila ada. Tanpa ref, yang diunduh cabang bawaan repo itu. Unduhan dimulai setelah folder tujuan jelas: folder yang dipilih saat itu, atau folder bawaan dari Settings jika sudah ada.

Hasil di disk adalah isi repo yang sudah diekstrak, di dalam folder `pemilik-repo`. Bukan folder `.git`.

## Banyak tautan

Tempelan boleh berisi banyak tautan, dipisah spasi atau baris baru. Peti memecahnya sendiri. Pengguna tidak memasukkan satu per satu.

- Tautan valid masuk antrean sebagai item terpisah.
- Tautan tidak valid tidak masuk, dan disebut satu per satu. Tautan valid di tempelan yang sama tetap masuk.
- Tautan yang sama dan ref yang sama, yang masih menunggu atau sedang berjalan, tidak diduplikasi.

## Folder

- Setiap mulai unduh, pengguna bisa memilih folder. Settings menyimpan folder bawaan supaya kali berikutnya tidak wajib dipilih lagi.
- Folder bawaan yang hilang atau tidak bisa ditulis tidak dipakai diam-diam. Peti meminta folder lain.
- Isi folder yang sudah ada tidak ditimpa. Bentrok nama memakai akhiran `-2`, `-3`, dan riwayat mencatat nama yang dipakai.

## Antrean

Status tiap item hanya salah satu dari: menunggu, berjalan, dijeda, selesai, gagal, dibatalkan.

- Paling banyak 3 item berjalan sekaligus. Sisanya menunggu. Angka ini diubah di Settings, paling banyak 8.
- Menutup aplikasi tidak menghapus antrean. Yang sedang berjalan menjadi gagal dengan alasan "ditutup saat berjalan", dan bisa diulang. Yang dijeda tetap dijeda.
- Selesai artinya arsip sudah masuk dan ekstraknya utuh. Ekstrak yang gagal di tengah tidak ditandai selesai, dan folder setengahnya tidak dibiarkan tanpa tanda.

## Progres

Tiap item yang berjalan menampilkan:

- persen, hanya bila ukuran total diketahui
- kecepatan unduhan
- sisa waktu, hanya bila persen diketahui (ukuran tersisa dibagi kecepatan)

Di atas daftar ada ringkasan kumpulan yang sedang berjalan: berapa item, persen gabungan bila semua ukuran yang berjalan diketahui, kecepatan gabungan, dan sisa waktu gabungan. Jika ada item tanpa ukuran total, ringkasan tidak memalsukan satu persen untuk semuanya; yang disebut jumlah item dan kecepatan.

## Jeda, lanjut, batal, ulang

- **Jeda** menghentikan unduhan dan menyimpan bagian yang sudah masuk.
- **Lanjut** meneruskan dari bagian itu bila server menerima Range dan ukuran total diketahui. Jika tidak, Peti mengatakan item diunduh ulang dari awal, lalu melakukannya.
- **Batal** menghentikan item dan menghapus bagian yang belum selesai. Tidak menghapus unduhan yang sudah selesai.
- **Ulang** hanya untuk yang gagal. Item itu kembali menunggu, dari awal.

## Riwayat

Unduhan yang selesai masuk riwayat: pemilik, nama, ref, folder penuh, dan waktu selesai. Riwayat tidak berisi yang gagal atau dibatalkan. Yang gagal tetap di antrean sampai diulang, dibatalkan, atau dibuang dari daftar.

Dari riwayat, pengguna membuka folder itu di pengelola file sistem.

## Galat

Pesannya mengatakan apa yang gagal dan apa yang bisa dilakukan. Tidak ada kode mentah sebagai satu-satunya teks.

| Kejadian | Pesan yang dipakai |
|---|---|
| Tautan bukan repo GitHub | Tautan tidak valid, plus tautannya |
| Repo tidak ketemu, atau tidak publik | Repositori tidak tersedia |
| Folder tidak bisa ditulis | Folder tidak bisa dipakai, pilih yang lain |
| Jaringan putus | Unduhan terputus. Ulang |
| Ekstrak rusak atau path di dalam arsip keluar dari folder tujuan | Unduhan tidak disimpan. Ulang |
| Kuota GitHub habis | Batas GitHub tercapai, coba lagi nanti |

Path di dalam arsip yang menunjuk ke luar folder tujuan ditolak seluruh ekstraknya. Pengalihan unduhan hanya diikuti ke `github.com` dan `codeload.github.com`.

## Selesai bila

- Satu tautan valid berakhir sebagai folder isi repo di tempat yang dipilih.
- Sepuluh tautan dalam satu tempelan menjadi sepuluh item, tanpa diketik ulang.
- Satu tautan sampah di antara yang valid hanya menolak yang sampah.
- Tiga unduhan bisa berjalan bersama, yang keempat menunggu.
- Persen tidak muncul ketika ukuran total tidak ada.
- Jeda lalu lanjut tidak mengulang dari nol bila Range didukung, dan mengatakannya bila harus mengulang.
- Batal tidak meninggalkan arsip setengah jadi.
- Setelah aplikasi ditutup dan dibuka, antrean dan riwayat masih ada.
- Dua unduh ke nama folder yang sama tidak saling menimpa.
