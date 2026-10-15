# Langkah kerja

Urut. Jangan kerjakan Discover sebelum langkah 8 selesai, dan jangan kerjakan fork sebelum langkah 13 selesai. Tiap langkah selesai hanya jika "Selesai bila" di bagian yang dirujuk juga benar.

Inti unduhan, antrean, GitHub, dan penyimpanan di Rust. Tampilan di webview Tauri 2. Rincian keputusan ada di `00-mulai-dari-sini.md`.

## Langkah 1 — Proyek kosong

**Maksud.** Aplikasi Tauri 2 terbuka di macOS, ikon jendelanya tanda Peti.

**Pekerjaan.** Pasang rantai Rust yang memenuhi MSRV Tauri 2 (1.90 atau lebih baru pada lini 2.12). Buat proyek Tauri 2 dengan frontend Vite. Pasang tanda dari `brand/logo/slot-symbol-v1.svg` sebagai ikon. Belum ada halaman sungguhan.

**Selesai bila.** Jendela kosong terbuka di macOS. `cargo` membangun inti Rust-nya.

## Langkah 2 — Kerangka lima halaman

**Maksud.** Pengguna bisa pindah ke Home, Downloads, Discover, Library, dan Settings. Isinya masih kosong yang jujur ("belum ada"), bukan data palsu.

**Pekerjaan.** Sidebar kiri satu tingkat, isi di kanan, sesuai `04-desain.md` dan `05-halaman-dan-alur.md`. Lebar di bawah sekitar 960 piksel menyusutkan sidebar menjadi ikon.

**Selesai bila.** Lima tujuan bisa diklik, yang aktif terlihat, dan tidak ada halaman keenam.

## Langkah 3 — Tema

**Maksud.** Light, dark, dan ikut sistem. Dark mode solid.

**Pekerjaan.** Pasang token di `04-desain.md`: warna, dua radius, tiga tingkat teks, font sistem, satu set ikon garis. Tema bawaan ikut sistem. Belum perlu halaman Settings yang lengkap; yang ini harus sudah terlihat di kerangka.

**Selesai bila.** Berganti tema tidak mengubah tata letak. Tidak ada gradasi pada latar atau tombol. Gerak menghormati pengurangan gerak di sistem.

## Langkah 4 — Penyimpanan lokal

**Maksud.** Antrean, riwayat, dan pengaturan bertahan setelah aplikasi ditutup.

**Pekerjaan.** Satu SQLite di folder data aplikasi. Yang disimpan: item antrean (pemilik, repo, ref, folder tujuan, status, byte, ukuran total bila ada, pesan galat), riwayat selesai (pemilik, repo, ref, folder penuh, waktu), pengaturan (folder bawaan, jumlah bersamaan, tema). Token GitHub tidak masuk ke database ini.

**Selesai bila.** Menulis satu item, menutup aplikasi, membuka lagi, item itu masih ada. Tidak ada token di file database.

## Langkah 5 — Parser tautan

**Maksud.** Tempelan menjadi daftar repo, tanpa jaringan.

**Pekerjaan.** Terima `github.com/pemilik/repo`, akhiran `.git`, dan `/tree/ref`. Tolak gist, issue, host lain, dan pemilik tanpa repo. Pecah tempelan pada spasi dan baris baru. Jangan gagalkan satu tempelan karena satu token buruk; kembalikan yang valid dan yang ditolak beserta alasannya.

**Selesai bila.** Kasus di `01-pengunduh.md` untuk tautan valid, banyak tautan, dan satu sampah lulus tanpa mengunduh.

## Langkah 6 — Unduh satu repo

**Maksud.** Satu tautan menjadi folder isi repo.

**Pekerjaan.** Unduh arsip lewat HTTP, ikuti pengalihan hanya ke `github.com` dan `codeload.github.com`, ekstrak ke folder tujuan. Tolak seluruh ekstrak jika ada path yang keluar dari folder tujuan. Nama folder `pemilik-repo`, akhiran `-2` bila sudah ada. Progres: persen hanya jika ukuran total ada; jika tidak, byte dan kecepatan. Belum perlu antrean berganda.

**Selesai bila.** Satu repo publik kecil selesai diekstrak, bisa dibuka, dan tidak berisi `.git`. Uji juga arsip yang mencoba menulis ke luar folder: harus ditolak.

## Langkah 7 — Antrean

**Maksud.** Banyak tautan, batas 3 berjalan, sisanya menunggu.

**Pekerjaan.** Hubungkan parser ke antrean. Jalankan paling banyak sesuai pengaturan (bawaan 3, batas 8). Item keempat tetap menunggu sampai ada slot. Duplikat yang masih menunggu atau berjalan diabaikan. Tutup aplikasi: yang berjalan menjadi gagal dengan alasan ditutup, yang menunggu tetap menunggu.

**Selesai bila.** Sepuluh tautan dalam satu tempelan menjadi item. Tidak pernah ada empat unduhan HTTP sekaligus pada bawaan. Membuka ulang aplikasi mengembalikan antrean.

## Langkah 8 — Jeda, lanjut, batal, ulang

**Maksud.** Kendali di `01-pengunduh.md` jujur.

**Pekerjaan.** Jeda menyimpan bagian yang sudah ada. Lanjut memakai Range bila ukuran diketahui dan server menerima; jika tidak, katakan bahwa unduhan mulai dari awal lalu lakukan. Batal menghapus bagian yang belum selesai. Ulang hanya dari status gagal.

**Selesai bila.** Empat kendali itu berperilaku seperti kalimat di atas, termasuk kalimat jujur ketika Range tidak ada. Batal tidak meninggalkan folder setengah jadi yang terlihat sebagai sukses.

## Langkah 9 — Riwayat dan Library

**Maksud.** Yang selesai bisa ditemukan lagi, beserta foldernya.

**Pekerjaan.** Pindahkan item selesai ke riwayat. Halaman Library sesuai `05`. Buka folder memakai tindakan sistem. Home menampilkan beberapa riwayat terbaru, masih boleh tanpa angka berjalan yang hidup kalau angka itu baru lengkap di langkah 15.

**Selesai bila.** Unduhan selesai muncul di Library dengan path yang benar. Yang gagal tidak muncul di Library.

## Langkah 10 — Pesan galat unduhan

**Maksud.** Kegagalan yang sudah dikenal tidak menampilkan isi mentah sebagai satu-satunya teks.

**Pekerjaan.** Pasang tabel galat di `01-pengunduh.md` pada parser, unduhan, ekstrak, dan folder. Pesan duduk di baris antrean.

**Selesai bila.** Tautan sampah, repo yang tidak ada, dan folder yang tidak bisa ditulis masing-masing menghasilkan pesan dari tabel itu.

## Langkah 11 — Discover

**Maksud.** Daftar repo populer dari Search API.

**Pekerjaan.** Minta repositori publik, urut star, jendela minggu sebagai bawaan. Tampilkan nama, deskripsi, bahasa, star, fork. Saringan bahasa, topik, dan periode (hari, minggu, bulan). Tahan permintaan beruntun. Simpan hasil sebentar. Jika kuota habis, tampilkan pesan batas, bukan daftar kosong. Baca sisa kuota dari header, jangan dari angka yang ditulis mati.

**Selesai bila.** Butir selesai di `02-temukan.md` untuk daftar, saringan, dan kuota. Belum wajib bisa mengunduh dari sini.

## Langkah 12 — Detail, unduh langsung, pilih banyak

**Maksud.** Dari menemukan sampai masuk antrean, satu jalan.

**Pekerjaan.** Detail repo. Unduh satu baris dan tambah pilihan ke antrean yang sudah ada, bukan antrean baru. Jika folder bawaan belum ada, minta sekali. Batang pilihan menampilkan jumlah.

**Selesai bila.** Butir selesai di `02-temukan.md` untuk detail dan pilih banyak. Item yang masuk dari Discover tunduk pada batas bersamaan langkah 7.

## Langkah 13 — Masuk GitHub

**Maksud.** Akun tersambung tanpa client secret di aplikasi.

**Pekerjaan.** Device flow sesuai dokumentasi OAuth. Simpan token di penyimpanan rahasia sistem, bukan SQLite. Settings: satu baris masuk atau keluar, sesuai `05`. Keluar menghapus token dan tidak menghapus antrean atau riwayat.

**Selesai bila.** Masuk lalu keluar bisa diulang. File database tidak berisi token. Tanpa masuk, unduh publik tetap jalan.

## Langkah 14 — Tiga tindakan

**Maksud.** Hanya unduh, hanya fork, fork lalu unduh. Fork tidak pernah diam-diam.

**Pekerjaan.** Saat masuk, detail dan pengambilan repo menampilkan tiga pilihan di `03-github.md`. Hanya unduh tidak memanggil fork. Hanya fork tidak menulis folder. Fork lalu unduh menunggu fork ada, lalu mengunduh fork itu. Repo milik sendiri tidak di-fork; tawarkan unduh saja. Tanpa masuk, pilihan fork tidak ada. Pastikan scope OAuth App untuk fork repo publik dari dokumentasi saat mengerjakan langkah ini.

**Selesai bila.** Semua butir selesai di `03-github.md`.

## Langkah 15 — Home dan Settings utuh

**Maksud.** Dua halaman yang tersisa memakai data sungguhan.

**Pekerjaan.** Home: jumlah berjalan, menunggu, gagal, tempel singkat, riwayat terbaru, keadaan kosong. Settings: folder bawaan, jumlah bersamaan, tema, GitHub. Perubahan langsung dipakai, tanpa tombol simpan.

**Selesai bila.** Mengubah jumlah bersamaan mengubah langkah 7 tanpa membuka ulang dari nol. Mengubah tema mengubah langkah 3. Tempel dari Home berakhir di Downloads.

## Langkah 16 — Keadaan dan gerak

**Maksud.** Kosong, memuat, berhasil, dan gagal terasa sama di lima halaman.

**Pekerjaan.** Isi keadaan di `04-desain.md` dan `05-halaman-dan-alur.md`. Gerak singkat pada batang pilihan dan pergantian keadaan. Tidak ada modal yang mengunci jendela hanya karena unduhan sedang berjalan.

**Selesai bila.** Tiap halaman punya empat keadaan itu, dan unduhan yang berjalan tidak mengambil alih seluruh jendela.

## Langkah 17 — Windows dan Arch

**Maksud.** Aplikasi yang sama di dua sistem selain macOS.

**Pekerjaan.** Bangun di Windows (WebView2) dan di Arch. Di Arch, penuhi prasyarat Tauri 2, termasuk webkit2gtk 4.1, dari [dokumentasi prasyarat](https://v2.tauri.app/start/prerequisites/). Jangan menambah tata letak baru. Cek tiga alur di `05-halaman-dan-alur.md` di tiap sistem: unduh langsung, Discover sampai Library, dan tiga tindakan GitHub.

**Selesai bila.** Ketiga alur selesai di macOS, Windows, dan Arch. Dark mode tetap solid di ketiganya. Tidak ada logo atau nama baru di jalan.
