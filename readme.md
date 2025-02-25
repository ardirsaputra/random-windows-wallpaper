# Wallpaper Updater

Wallpaper Updater adalah aplikasi berbasis GUI untuk Windows yang secara otomatis mengunduh dan mengatur wallpaper dari sumber online dalam interval waktu tertentu. Aplikasi ini dibuat menggunakan Rust dengan eframe (egui) untuk antarmuka grafis.

## Fitur
- Mengganti wallpaper secara manual dengan satu klik.
- Mengunduh wallpaper dari berbagai sumber online.
- Menyimpan dan mengonversi wallpaper ke format BMP.
- Mengubah wallpaper dalam interval waktu tertentu secara otomatis.
- Menjalankan aplikasi secara otomatis saat startup.
- Meminimalkan aplikasi ke system tray.

## Prasyarat
Sebelum menjalankan atau mengompilasi aplikasi ini, pastikan Anda memiliki:
- Rust (https://www.rust-lang.org/)
- Cargo (termasuk dalam instalasi Rust)
- Paket-paket berikut yang diperlukan:
  ```sh
  cargo add eframe winapi winreg image reqwest rand
  ```

## Instalasi dan Penggunaan
### 1. Clone Repository
```sh
git clone https://github.com/ardirsaputra/random-windows-wallpaper.git
cd random-windows-wallpaper
```

### 2. Build dan Jalankan
```sh
cargo run --release
```

### 3. Menggunakan Aplikasi
- Klik tombol "Ganti Wallpaper" untuk mengganti wallpaper secara manual.
- Aktifkan opsi "Perbarui wallpaper secara otomatis" untuk mengatur wallpaper dalam interval tertentu.
- Gunakan opsi "Jalankan saat startup" agar aplikasi berjalan otomatis saat Windows dinyalakan.
- Pilih "Minimize ke tray" agar aplikasi berjalan di latar belakang.

## Konfigurasi Tambahan
### Mengubah Sumber Wallpaper
Sumber wallpaper dapat diubah dengan memodifikasi konstanta `WALLPAPER_URLS` di dalam kode:
```rust
const WALLPAPER_URLS: &[&str] = &[
    "https://minimalistic-wallpaper.demolab.com/?random",
    "https://source.unsplash.com/random/1920x1080",
    "https://picsum.photos/1920/1080",
    "https://images.pexels.com/photos/random/1920/1080",
];
```
Tambahkan atau ganti URL dengan sumber wallpaper favorit Anda.

## Lisensi
Proyek ini menggunakan lisensi MIT. Silakan lihat file `LICENSE` untuk informasi lebih lanjut.

## Kontribusi
Jika ingin berkontribusi, silakan buat pull request atau laporkan masalah di halaman repository proyek ini.

---
Dibuat dengan ❤️ menggunakan Rust dan egui.

