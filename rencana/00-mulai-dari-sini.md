# Peti — mulai dari sini

Folder ini adalah rencana kerja aplikasi Peti. Belum ada kode aplikasi. Nama dan logo tidak dibuka lagi: nama yang dipakai Peti, tandanya file `brand/logo/slot-symbol-v1.svg` (kotak rounded dengan satu celah di tepi atas).

PRD di `.claude/prds/peti.prd.md` lebih lama. Pertanyaan yang di sana masih kosong, dan yang sudah kamu isi di spesifikasi ini, mengikuti folder ini.

## Isi folder

| File | Isinya |
|---|---|
| `00-mulai-dari-sini.md` | Yang sudah dikunci, dan keputusan yang membuat langkahnya bisa dikerjakan |
| `01-pengunduh.md` | Unduh satu, unduh banyak, folder, antrean, progres, jeda, riwayat, galat |
| `02-temukan.md` | Halaman repo populer, cari, saring, detail, pilih banyak |
| `03-github.md` | Masuk GitHub, tiga pilihan tindakan, fork tidak pernah diam-diam |
| `04-desain.md` | Tampilan, dan apa yang diambil atau ditolak dari referensi |
| `05-halaman-dan-alur.md` | Lima halaman dan tiga alur |
| `06-langkah-kerja.md` | Urutan pengerjaan |

Kerjakan `06` dari atas ke bawah. Jangan loncat ke Discover atau GitHub sebelum antrean unduhan jujur.

## Yang sudah kamu kunci

- Berjalan di macOS, Windows, dan Linux, termasuk Arch.
- Dibangun dengan Rust.
- Mengunduh satu repositori GitHub, atau banyak sekaligus, dari tautan yang ditempel.
- Pengguna menentukan folder penyimpanan.
- Ada antrean, progres (persen, kecepatan, sisa waktu), jeda, lanjut, batal, dan ulang jika gagal.
- Ada riwayat unduhan beserta lokasinya.
- Pesan galat jelas: repo tidak ada, tautan tidak valid, unduhan bermasalah.
- Ada halaman untuk menemukan repo yang sedang populer.
- Tiap repo menampilkan nama, deskripsi, bahasa, jumlah star, dan popularitas.
- Bisa mencari, menyaring (bahasa, kategori, periode), membuka detail, mengunduh langsung, memilih banyak, dan menambah ke antrean.
- Masuk GitHub bersifat opsional.
- Tiga tindakan, dipilih sendiri: hanya unduh, hanya fork, fork lalu unduh hasil fork.
- Tanpa masuk, unduh repo publik tetap bisa.
- Fork tidak jalan kecuali pengguna memilihnya.
- Tampilannya modern, minimal, rapi, ada ruang kosong, sudut rounded, light mode dan dark mode.
- Dark mode memakai warna solid, bukan gradasi yang berlebihan.
- Ada tipografi yang jelas, ikon yang tidak berlebihan, gerak yang halus, serta keadaan kosong, memuat, berhasil, dan gagal.
- Nyaman di berbagai ukuran jendela desktop.
- Halaman: Home, Downloads, Discover, Library, Settings.
- Tiga alur di `05-halaman-dan-alur.md`.

## Keputusan rencana

Ini bukan permintaan baru. Tanpa keputusan ini, langkah kerja tidak bisa ditulis dengan jujur. Ubah sebelum langkah yang bersangkutan dikerjakan, jangan di tengah.

1. **Kulit aplikasi Tauri 2, inti Rust.** Antrean, unduhan, GitHub, dan penyimpanan ditulis di Rust. Tampilannya halaman di webview sistem (WebKit di macOS, WebView2 di Windows, webkit2gtk 4.1 di Linux dan Arch). UI murni Rust (iced, egui, Slint) tidak dipakai: light/dark, gerak, dan keadaan halaman yang diminta tidak jujur dijanjikan dari situ pada urutan kerja ini. Kalau UI-nya harus murni Rust, berhenti sebelum langkah 2.
2. **Yang diunduh adalah snapshot isi repo, bukan `git clone`.** Arsip repositori diunduh lewat HTTP, lalu diekstrak ke folder tujuan. Jeda, lanjut, persen, dan kecepatan adalah perilaku unduhan HTTP. `git clone` tidak bisa dijeda di tengah dengan jujur, jadi histori git (folder `.git`) tidak termasuk di versi ini.
3. **Persen tidak dikarang.** Persen tampil hanya jika ukuran total diketahui. Jika tidak, yang tampil byte yang sudah masuk dan kecepatannya. Lanjut memakai HTTP Range bila server menerimanya. Jika tidak, lanjut berarti item itu diunduh ulang dari awal, dan tulisannya mengatakan begitu.
4. **Bersamaan, bawaannya 3.** Sisanya menunggu di antrean. Angka ini ada di Settings. Batas atas di pengaturan: 8.
5. **"Sedang populer" memakai GitHub Search API, bukan halaman trending.** Dokumentasi REST yang dicek tidak menyediakan endpoint trending. Search diurutkan menurut star, dalam jendela hari, minggu, atau bulan, plus bahasa. Ini bukan salinan github.com/trending. Menyalin HTML halaman itu tidak dipakai: mudah patah. Batas search: sampai 10 permintaan per menit tanpa masuk, 30 per menit setelah masuk (bukan code search). Sisa kuota dibaca dari header balasan, tidak ditebak.
6. **Kategori berarti topik GitHub.** GitHub tidak punya daftar kategori resmi. Saringan kategori mengisi qualifier `topic:`.
7. **Popularitas yang ditampilkan adalah star dan fork**, bukan skor buatan.
8. **Masuk memakai device flow.** Aplikasi desktop tidak menyimpan client secret. Token tidak ditulis di SQLite; tempatnya penyimpanan rahasia sistem.
9. **Tautan yang diterima** hanya `github.com/pemilik/repo`, dengan atau tanpa `.git`, plus `/tree/ref` bila ref disebut. Gist, issue, dan host lain ditolak satu per satu. Tautan buruk tidak menggagalkan tautan lain dalam tempelan yang sama.
10. **Folder yang sudah ada tidak ditimpa.** Jika `pemilik-repo` sudah ada di tujuan, nama berikutnya `pemilik-repo-2`, dan seterusnya. Riwayat mencatat folder yang benar-benar dipakai.
11. **Repo privat tidak termasuk.** Tanpa masuk maupun dengan masuk, versi ini mengunduh repo publik. Yang tidak ketemu dibalas dengan pesan "tidak tersedia", tanpa menebak privat atau memang tidak ada.

## Tidak termasuk

Menjalankan atau membangun repo yang sudah diunduh. Aplikasi ponsel. Kolaborasi. Host selain GitHub. Clone dengan histori git. Salinan persis halaman trending GitHub. Nama baru, logo baru, atau kit merek.

## Sumber

- [GitHub Search API](https://docs.github.com/en/rest/search/search) — batas search, dan tidak ada endpoint trending di halaman yang dicek
- [Device flow untuk aplikasi OAuth](https://docs.github.com/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps) — masuk tanpa client secret
- [Prasyarat Tauri 2](https://v2.tauri.app/start/prerequisites/) — webkit2gtk 4.1 di Linux
- [Tauri](https://github.com/tauri-apps/tauri) — lini 2.x yang dipakai
