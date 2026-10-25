# Lieve

Dynamic Island untuk Windows, dengan mata terinspirasi Liebe yang bisa dikustomisasi. Dibangun dengan **Rust native + Win32/GDI**, tanpa Electron, Tauri, atau WebView.

> **Status: alpha dalam pengembangan.** Belum semua fitur selesai dan belum ada installer resmi. Target RAM sekitar 30 MiB, bukan jaminan batas maksimum.

## Fitur saat ini

| Fitur | Status |
|---|---|
| Notch always-on-top, hover, pin, floating, minimal | Tersedia |
| Mata di tengah, preset/custom warna, glow | Tersedia |
| Panel Appearance di dalam notch | Tersedia |
| Animasi resize dan debounce auto-close | Tersedia |
| Musik Windows: metadata, play/pause, skip, seek | Terintegrasi SMTC; dukungan bergantung player |
| Lirik sinkron di notch kecil dan panel | LRCLIB; judul lagu menjadi fallback |
| Timer fokus 25 menit + notifikasi | Tersedia; tidak dipersist setelah keluar |
| Catatan minum harian | Tersimpan lokal, reset berdasarkan tanggal |
| File shelf | Referensi lokal, picker/drop handler, buka/hapus referensi; drag-out belum tersedia |
| To-do | Tambah lewat dialog native, selesai/belum, hapus, simpan lokal |
| Tray menu | Tersedia; ikon masih default Windows |
| HUD volume/brightness, status koneksi/baterai | Belum selesai |
| Reminder task/minum, autostart, fullscreen auto-hide | Belum selesai |
| Installer dan penyempurnaan visual | Belum selesai |

Artwork album masih ilustrasi, bukan gambar album asli. Rendering belum identik dengan prototipe web. Beberapa alur native masih membutuhkan pengujian interaktif; lihat [catatan validasi](native/VALIDATION.md).

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

Hover notch untuk membuka. Klik **P** agar tetap terbuka, **⋯** untuk Appearance. Klik kanan notch/ikon tray untuk menu dan **Exit Lieve**. Floating dapat digeser melalui header. Animasi mengikuti preferensi animasi Windows saat aplikasi dimulai.

Pada **To-do**, pilih Add task; gunakan Enter/Add untuk menyimpan atau Escape/Cancel untuk batal. Klik task untuk menandai selesai, `x` untuk menghapus. Scroll untuk melihat item tambahan. Reminder dan edit task belum diimplementasikan.

Pada **Shelf**, Add file memilih file tanpa menyalin isinya. Klik referensi untuk membuka dengan Windows; buka hanya file tepercaya. `x` tidak menghapus file asli. Drag/drop Explorer masih perlu pengujian manual pada device pengguna.

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
# Opsional: memerlukan desktop Windows dan koneksi ke LRCLIB
cargo test -- --include-ignored --nocapture
```

Working set build terdahulu teramati sekitar **29–30 MiB** setelah integrasi WinRT, bukan hanya shell. Angka ini bukan pengukuran peak atau jaminan pada semua device. Ukur versi release tanpa memaksa working-set trimming:

```powershell
Get-Process lieve | Select-Object Id, CPU,
  @{n='WorkingSetMiB';e={[math]::Round($_.WorkingSet64/1MB,2)}},
  @{n='PrivateMiB';e={[math]::Round($_.PrivateMemorySize64/1MB,2)}}
```

## Melanjutkan pengembangan

Baca [panduan native](native/README.md) dan [hasil validasi](native/VALIDATION.md). Prioritas berikutnya: validasi UI dan persistensi To-do/Shelf, HUD/status Windows, reminder, aksesibilitas, visual polish, dan packaging. Jangan menganggap placeholder atau alur yang belum diuji sebagai fitur yang selesai.
