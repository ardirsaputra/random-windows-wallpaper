# Wallpaper Changer for Windows (Rust)

## 📌 Deskripsi
Program ini adalah aplikasi **Rust** yang secara otomatis mengunduh wallpaper dari internet dan mengatur wallpaper di Windows setiap **1 jam sekali**. Jika program dibuka kembali, wallpaper akan langsung diperbarui dengan gambar baru.

## 🛠 Fitur
- Mengunduh wallpaper dari **https://minimalistic-wallpaper.demolab.com/?random**
- Menyimpan gambar tanpa mengganti file sebelumnya
- Mengonversi gambar ke **format BMP** agar kompatibel dengan Windows
- Mengatur wallpaper secara otomatis
- Menggunakan ikon khusus untuk aplikasi `.exe`

## 🚀 Cara Instalasi
### 1️⃣ **Persyaratan**
Pastikan sistem sudah memiliki **Rust dan Cargo**. Jika belum, instal dengan:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Selain itu, pastikan juga menginstal **MSYS2** dan `mingw-w64-gcc`:
```sh
pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-binutils
```

### 2️⃣ **Clone Repository**
```sh
git clone https://github.com/username/repo-wallpaper.git
cd repo-wallpaper
```

### 3️⃣ **Build Program**
Jalankan perintah berikut untuk membangun aplikasi:
```sh
cargo build --release
```
Hasil build akan ada di folder `target/release/`.

## 📌 Menjalankan Program
Setelah build selesai, jalankan program dengan:
```sh
target/release/nama_program.exe
```
Program akan mulai mengunduh dan mengganti wallpaper setiap **1 jam sekali**.

## 🎨 Mengubah Ikon Aplikasi
1. Pastikan ada file ikon **`icon.ico`** dalam folder proyek.
2. Tambahkan `build.rs` dengan kode berikut:
   ```rust
   fn main() {
       let mut res = winres::WindowsResource::new();
       res.set_icon("icon.ico");
       res.compile().expect("Gagal meng-compile resource");
   }
   ```
3. Edit `Cargo.toml` dan tambahkan:
   ```toml
   [build-dependencies]
   winres = "0.1"
   ```
4. Build ulang dengan:
   ```sh
   cargo clean
   cargo build --release
   ```

## ❗ Troubleshooting
- **Ikon tidak berubah?** Jalankan:
  ```sh
  ie4uinit.exe -ClearIconCache
  taskkill /IM explorer.exe /F & start explorer
  ```
- **Wallpaper tidak berubah?** Pastikan program dijalankan dengan izin **Administrator**.
- **Gagal build `windres`?** Coba instal ulang `mingw-w64-x86_64-binutils` dengan:
  ```sh
  pacman -S mingw-w64-x86_64-binutils
  ```

## 📜 Lisensi
Proyek ini dirilis di bawah lisensi **MIT**.

---

✅ **Proyek selesai!** Jika ada pertanyaan atau masalah, silakan buat _issue_ di GitHub. 🚀

