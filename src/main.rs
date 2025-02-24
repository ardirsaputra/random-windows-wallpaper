use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageFormat};
use tokio::time::sleep;
use reqwest;
use winapi::um::winuser::{SystemParametersInfoW, SPI_SETDESKWALLPAPER};
use std::process::Command;
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Saat aplikasi dijalankan, langsung ambil wallpaper baru
    refresh_wallpaper().await?;

    loop {
        // Menunggu selama 1 jam sebelum memperbarui wallpaper lagi
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        refresh_wallpaper().await?;
    }
}

/// Mengembalikan path ke folder penyimpanan wallpaper
fn get_wallpaper_directory() -> PathBuf {
    let mut path = PathBuf::from("C:\\Wallpapers");
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Gagal membuat folder Wallpapers");
    }
    path
}

/// Konversi gambar ke BMP sebelum diterapkan sebagai wallpaper
fn convert_to_bmp(path_jpg: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path_jpg)?.decode()?;
    let path_bmp = path_jpg.with_extension("bmp");

    let mut file = File::create(&path_bmp)?;
    img.write_to(&mut file, ImageFormat::Bmp)?;
    
    println!("Gambar dikonversi ke {:?}", path_bmp);
    Ok(path_bmp)
}

/// Mengatur wallpaper menggunakan Windows API
fn set_wallpaper(path: &PathBuf) {
    let path_str = path.to_str().unwrap();
    let mut path_wide: Vec<u16> = path_str.encode_utf16().collect();
    path_wide.push(0); // null-terminate

    unsafe {
        if SystemParametersInfoW(SPI_SETDESKWALLPAPER, 0, path_wide.as_ptr() as *mut _, 3) == 0 {
            eprintln!("Gagal mengubah wallpaper.");
        } else {
            println!("Wallpaper berhasil diterapkan: {}", path_str);
        }
    }
}

/// Memperbarui registry agar wallpaper diterapkan dengan benar
fn update_wallpaper_registry() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Control Panel\Desktop";
    let key = hkcu.open_subkey_with_flags(path, KEY_WRITE).expect("Gagal membuka registry");

    key.set_value("WallpaperStyle", &"2").expect("Gagal mengatur WallpaperStyle"); // Fill
    key.set_value("TileWallpaper", &"0").expect("Gagal mengatur TileWallpaper");

    println!("Registry wallpaper diperbarui.");
    
    // Paksa refresh desktop agar wallpaper diterapkan
    Command::new("RUNDLL32.EXE")
        .arg("user32.dll,UpdatePerUserSystemParameters")
        .output()
        .expect("Gagal memperbarui sistem.");
}


/// Fungsi untuk mengambil dan menerapkan wallpaper baru
async fn refresh_wallpaper() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://minimalistic-wallpaper.demolab.com/?random";
    println!("Mengambil gambar dari {}", url);

    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    let mut path_jpg = get_wallpaper_directory();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    path_jpg.push(format!("wallpaper_{}.jpg", timestamp));

    let mut file = File::create(&path_jpg)?;
    file.write_all(&bytes)?;
    println!("Gambar tersimpan di {:?}", path_jpg);

    let path_bmp = convert_to_bmp(&path_jpg)?;
    set_wallpaper(&path_bmp);
    update_wallpaper_registry();

    Ok(())
}