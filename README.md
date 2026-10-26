# Lieve

Dynamic Island untuk Windows, dengan mata terinspirasi Liebe yang bisa dikustomisasi. Dibangun dengan **Rust native + Win32/GDI**, tanpa Electron, Tauri, atau WebView.

> **Status: alpha dalam pengembangan.** Belum semua fitur selesai dan belum ada installer resmi. Target RAM sekitar 30 MiB, bukan jaminan batas maksimum.

## Fitur saat ini

| Fitur | Status |
|---|---|
| Notch always-on-top, hover, pin, floating, minimal | Tersedia |
| Mata di tengah, preset/custom warna, glow | Cursor-follow, sleepy, dan Devil grin bergigi lancip (gigi selalu putih, tidak mengikuti warna mata) opsional |
| Panel Appearance di dalam notch | Tersedia |
| Animasi notch | Spring seperti web, fade konten, indikator tab bergerak, kedipan mata, debounce auto-close |
| Musik Windows: metadata, foto album, play/pause, skip, seek | Terintegrasi SMTC; dukungan bergantung player |
| Lirik sinkron di notch kecil dan panel | LRCLIB; judul lagu menjadi fallback |
| Timer fokus 25 menit + notifikasi | Berlanjut lewat Restart Lieve; tidak disimpan setelah Exit biasa |
| Catatan minum harian | Tersimpan lokal, reset berdasarkan tanggal |
| File shelf | Referensi lokal, picker/drop handler, buka/hapus referensi; drag-out belum tersedia |
| To-do | Tambah/edit lewat dialog native, selesai/belum, hapus, simpan lokal |
| Tray menu | Pin, Appearance, Devil grin, Restart Lieve, Exit; ikon masih default Windows |
| Status baterai | Persentase/AC di menu tray; notifikasi baterai rendah masih memerlukan uji perangkat |
| Control Center: Wi-Fi, Bluetooth, volume, brightness | Tersedia; tombol lanjutan membuka Settings Windows |
| HUD volume/brightness eksternal (OSD) | Dibuat dengan spring, auto-hide sekitar 2 detik |
| Kaca frosted + tingkat transparansi | Tersedia via Appearance → Glass; efek blur bergantung Windows |
| Reminder task/minum, autostart, fullscreen auto-hide | Belum selesai |
| Installer dan penyempurnaan visual | Belum selesai |

Foto album kini diambil dari thumbnail pemutar Windows. Jika player belum menyediakan gambar, Music menampilkan `Loading...` atau `No artwork`; ilustrasi hanya dipakai pada demo. Rendering belum identik dengan prototipe web. Beberapa alur native masih membutuhkan pengujian interaktif; lihat [catatan validasi](native/VALIDATION.md).

## Menjalankan versi native

Windows 10/11 64-bit. Untuk build, install:

- [Rust](https://rustup.rs/) dengan target MSVC.
- Visual Studio Build Tools: **Desktop development with C++** dan Windows SDK.

```powershell
git clone https://github.com/zenarl-frl/Lieve.git
cd Lieve/native
cargo build --release
.\target\release\lieve.exe
```

Hover notch untuk membuka. Pada **Music**, ikon gelembung kutipan di kanan baris playback membuka/menutup **Lyrics** di dalam player yang sama; progress dan playback tetap terlihat. Ikon gear di kanan navigasi membuka **Appearance/Settings**. Menutup Settings mengembalikan tampilan sebelumnya, termasuk Music dengan Lyrics aktif. Untuk pin, klik kiri ikon tray atau pilih **Pin open** dari menu klik kanan. Menu yang sama menyediakan **Exit Lieve**. Floating dapat digeser melalui area kosong/tepi atas navigasi. Pada menu **Motion**, pilih **web spring** untuk gerak seperti prototipe, **follow Windows** untuk mengikuti preferensi sistem (default), atau **reduced** untuk perpindahan langsung. Pilihan disimpan lokal. Jika animasi Windows dimatikan, mode follow Windows akan berpindah langsung; pilih web spring agar animasi Lieve tetap berjalan.

Panel terbuka berukuran **400 × 220 piksel logis**, dengan lima tab dan Settings di atas. Music memakai satu kartu album: artwork 72 px, metadata berhierarki, progress selebar konten, waktu rata kiri/kanan, dan playback tepat di tengah. Ikon vektor dirender supersampled, tombol memiliki hover feedback, dan album yang namanya sama dengan judul lagu tidak diulang. Hitungan To-do/Shelf berada pada tombol tambah; empat item daftar tetap terlihat sekaligus. Ukuran fisik mengikuti skala DPI Windows.

Notch memakai spring dengan parameter prototipe web dan mempertahankan momentum saat hover dibalik di tengah gerakan. Konten keluar dahulu lalu memudar masuk pada ukuran aslinya, sehingga teks dan mata tidak melar ketika shell mengembang. Radius sudut ikut bergerak, indikator tab meluncur, dan mata berkedip berkala saat terlihat.

Frame aktif dijadwalkan dengan timer high-resolution mengikuti refresh monitor (maksimum 240 Hz), lalu berhenti saat gerak selesai. Buffer gambar dipakai ulang, dan mata dirender supersampled agar pergeseran kecil lebih halus. Pada pengukuran lokal, median interval pembaruan ukuran jendela turun dari 27,01 ms ke 8,30 ms; ini pengukuran event geometri, bukan jaminan FPS pada semua perangkat.

Gambar album memakai area resampling saat diperkecil dan bilinear saat diperbesar. Ikon, mata, serta tepi kartu memakai perataan coverage supersampled dalam buffer 32-bit, menggantikan pengecilan HALFTONE yang dapat terlihat berbintik. Kualitas detail foto tetap mengikuti thumbnail yang disediakan player. Teks tombol Focus/Water dipusatkan dari batas tombol, sementara label tambah dan hitungan daftar memakai padding serta alignment yang konsisten.

Saat notch idle, mata mengikuti arah kursor dengan pergeseran kecil dan kemiringan seperti web. Setelah 26 detik tanpa input Windows, mata menyipit dan miring ke ekspresi mengantuk. Input berikutnya membangunkan mata; saat mengantuk, pembacaan aktivitas dapat membutuhkan hingga sekitar 500 ms sebelum transisi mulai. Panel yang dibuka mengembalikan mata ke pose netral. Mode reduced menghentikan gerak dan tracking.

Aktifkan **Appearance → Devil grin** atau centang **Devil grin** di menu tray untuk ekspresi senyum bergigi lancip. Preview langsung memainkan senyum membuka, menahan pose, lalu menutup; ekspresi diulang sekitar setiap 12 detik saat mascot terlihat idle atau di preview Appearance. Mode ini tersimpan dan default-nya mati. Notch idle menjadi 144 × 48 saat aktif agar mulut muat; Music/panel tetap compact. Pada mode reduced, grin ditampilkan statis.

Klik kanan ikon tray → **Restart Lieve** untuk menjalankan ulang aplikasi. Settings/data lokal dimuat kembali, dan timer Focus yang berjalan atau dipause diteruskan. Restart memakai deadline timer sehingga waktu startup tetap dihitung. Exit biasa tidak menyimpan sesi Focus.

Menu klik kanan/tray menampilkan status daya: persentase baterai, `AC` saat terhubung listrik, `No battery` pada perangkat tanpa baterai, atau `Power —` saat status tidak diketahui. Pembacaan menggunakan API lokal Windows saat mulai dan sekitar setiap 30 detik. Saat memakai baterai pada ≤20%, Lieve mengirim satu notifikasi tray; notifikasi dapat muncul lagi setelah AC terdeteksi atau baterai pulih ke ≥25%. Pembatas ini berlaku selama aplikasi berjalan. Tampilan notifikasi mengikuti pengaturan Windows.

Pada **To-do**, pilih Add task; gunakan Enter/Add untuk menyimpan atau Escape/Cancel untuk batal. Klik task untuk menandai selesai, `edit` untuk mengubah teks, dan `x` untuk menghapus. Dialog edit menampilkan teks sebelumnya; Enter/Save menyimpan tanpa mengubah status selesai, Escape/Cancel membatalkan perubahan tanpa menulis ulang file task. Teks kosong ditolak. Jika penyimpanan gagal, perubahan dikembalikan dan pesan error ditampilkan. Scroll untuk melihat item tambahan. Reminder belum diimplementasikan.

Pada **Shelf**, Add file memilih file tanpa menyalin isinya. Klik referensi untuk membuka dengan Windows; buka hanya file tepercaya. `x` tidak menghapus file asli. Drag/drop Explorer masih perlu pengujian manual pada device pengguna.

Ikon kontrol di kanan bilah navigasi (atau **Control Center** dari menu) membuka panel Wi-Fi, Bluetooth, Sound, dan Brightness. Kartu Wi-Fi menampilkan SSID saat ini dan membuka `ms-settings:network-wifi`; kartu Bluetooth membuka daftar perangkat tersambung (hingga empat nama) dan tautan `ms-settings:bluetooth`. Baris Sound menyediakan tombol mute beserta slider volume; baris Brightness menampilkan `N/A` pada perangkat tanpa WMI brightness. Volume dan brightness eksternal (misalnya tombol hardware) memunculkan HUD kecil di bawah notch dengan animasi spring yang tetap meluncur saat nilai berubah lagi dan hilang otomatis sekitar dua detik; HUD tidak muncul sementara slider di panel sedang digeser. Tombol volume/mute di keyboard Windows mengubah endpoint audio master sehingga HUD yang sama ikut muncul; tombol brightness yang tidak lewat WMI tidak dapat dibaca tanpa hook global. Panel **Appearance → Glass** memuat toggle *Frosted backdrop* dan slider **Transparency** 0–100% (0% penuh warna, 100% latar jelas); keduanya tersimpan di `settings.txt`, dan tampilan status menunjukkan apakah blur native aktif atau tersedia pada Windows/display ini. Transparansi hanya aktif saat notch bergerak atau panel terbuka; notch idle kembali penuh warna tanpa flash di tengah animasi.

## Prototipe desain web

```powershell
cd prototype
npm ci
npm run dev
```

Buka alamat Vite yang muncul di terminal (default port 5180). Ini prototipe desain berbasis React dengan data simulasi, bukan aplikasi desktop native.

## Struktur

```text
native/       Rust native, build dan validasi
prototype/    Referensi desain React + TypeScript + Vite
```

Pengaturan dan data pengguna disimpan di `%APPDATA%\Lieve`: `settings.txt`, `water.txt`, `shelf.txt`, `tasks.txt`. Data ini tidak perlu dimasukkan ke Git. Referensi shelf dapat tidak valid setelah pindah perangkat.

## Privasi

Metadata judul lagu, artis dan durasi dikirim lewat HTTPS ke **LRCLIB** untuk mencari lirik. Tidak ada akun atau analytics. Pengaturan, task, catatan minum dan referensi file disimpan lokal. Shelf membuka file hanya saat diminta pengguna.

## Pengujian dan RAM

```powershell
cd native
cargo test
# Opsional: desktop Windows, player aktif dengan artwork, dan koneksi LRCLIB
cargo test -- --include-ignored --nocapture
```

Uji interaksi To-do pada aplikasi yang sudah berjalan (Windows PowerShell):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\smoke-todo.ps1
```

Jalankan dari `native/`, biarkan pointer di luar notch, dan jangan gunakan UI selama tes. Skrip menambah satu task uji unik, menguji penyimpanan/toggle/edit/hapus/batal, lalu menghapus task uji dan melepas pin yang diaktifkannya. Cek edit mencakup teks awal, penolakan teks kosong, status selesai yang tetap tersimpan, serta pembatalan tanpa menulis ulang file. Alur ini sudah lulus pada build Windows GNU dengan APPDATA sementara; lihat catatan validasi. Jika gagal, periksa peringatan tentang task uji yang mungkin tertinggal; jangan hapus task pribadi.

Untuk menguji gerak notch, gunakan mode docked tanpa pin, pilih **Motion: web spring**, dan biarkan pointer di luar notch:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\smoke-animation.ps1 -ExpectAnimated
```

Skrip memeriksa ukuran antara, posisi tengah, buka/tutup, dan pembalikan arah di tengah gerak. `-CaptureDirectory <folder-yang-sudah-ada>` menyimpan dua screenshot jendela Lieve untuk inspeksi frame. Pengujian ini tidak mengukur FPS.

Untuk membandingkan pacing pembaruan ukuran (mode web spring, docked, tanpa pin, pointer di luar):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\measure-motion.ps1
```

Skrip membuka/menutup notch empat kali dan melaporkan median/p95 interval event geometri selama 300 ms awal pembukaan. Ini bukan pengukuran presentasi compositor/GPU.

Working set build terdahulu teramati sekitar **29–30 MiB** setelah integrasi WinRT, bukan hanya shell. Angka ini bukan pengukuran peak atau jaminan pada semua device. Ukur versi release tanpa memaksa working-set trimming:

```powershell
Get-Process lieve | Select-Object Id, CPU,
  @{n='WorkingSetMiB';e={[math]::Round($_.WorkingSet64/1MB,2)}},
  @{n='PrivateMiB';e={[math]::Round($_.PrivateMemorySize64/1MB,2)}}
```

## Melanjutkan pengembangan

Baca [panduan native](native/README.md) dan [hasil validasi](native/VALIDATION.md). Prioritas berikutnya: validasi UI dan persistensi To-do/Shelf, status koneksi lanjutan, reminder, aksesibilitas, visual polish, dan packaging. Jangan menganggap placeholder atau alur yang belum diuji sebagai fitur yang selesai.
