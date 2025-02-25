#![windows_subsystem = "windows"]

use eframe::egui;
use eframe::IconData;
use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use reqwest::blocking;
use image::io::Reader as ImageReader;
use image::{ImageFormat};
use winapi::um::winuser::{SystemParametersInfoW, SPI_SETDESKWALLPAPER};
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE, KEY_READ};
use std::process::Command;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use rand::seq::SliceRandom;

#[derive(PartialEq, Clone, Copy)]
enum UpdateInterval {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    Custom(u64),
}

fn load_icon() -> Option<IconData> {
    if let Ok(reader) = ImageReader::open("icon.png") {
        if let Ok(image) = reader.decode() {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            return Some(IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            });
        }
    }
    None
}

impl UpdateInterval {
    fn as_seconds(&self) -> u64 {
        match self {
            UpdateInterval::OneMinute => 60,
            UpdateInterval::FiveMinutes => 5 * 60,
            UpdateInterval::FifteenMinutes => 15 * 60,
            UpdateInterval::ThirtyMinutes => 30 * 60,
            UpdateInterval::OneHour => 60 * 60,
            UpdateInterval::Custom(seconds) => *seconds,
        }
    }
    
    fn display_name(&self) -> String {
        match self {
            UpdateInterval::OneMinute => "1 menit".to_string(),
            UpdateInterval::FiveMinutes => "5 menit".to_string(),
            UpdateInterval::FifteenMinutes => "15 menit".to_string(),
            UpdateInterval::ThirtyMinutes => "30 menit".to_string(),
            UpdateInterval::OneHour => "1 jam".to_string(),
            UpdateInterval::Custom(seconds) => format!("{} detik (kustom)", seconds),
        }
    }
    
    fn all_intervals() -> Vec<UpdateInterval> {
        vec![
            UpdateInterval::OneMinute,
            UpdateInterval::FiveMinutes,
            UpdateInterval::FifteenMinutes,
            UpdateInterval::ThirtyMinutes,
            UpdateInterval::OneHour,
        ]
    }
}

struct MyApp {
    status: String,
    log: Arc<Mutex<Vec<String>>>,
    auto_update: bool,
    update_interval: UpdateInterval,
    run_at_startup: bool,
    minimized_to_tray: bool,
    custom_interval: String,
    window_visible: bool,
    last_window_pos: Option<egui::Pos2>,
    wallpaper_counter: u32,
    is_processing: Arc<Mutex<bool>>, // Ubah ke Arc<Mutex> untuk thread safety
    last_update_time: Instant,
    auto_update_handle: Option<JoinHandle<()>>, // Tambahkan handle untuk thread auto-update
}

impl Default for MyApp {
    fn default() -> Self {
        let run_at_startup = is_configured_to_run_at_startup();
        
        Self {
            status: "Klik tombol untuk mengganti wallpaper".to_owned(),
            log: Arc::new(Mutex::new(vec![])),
            auto_update: false,
            update_interval: UpdateInterval::FiveMinutes,
            run_at_startup,
            minimized_to_tray: false,
            custom_interval: "300".to_string(),
            window_visible: true,
            last_window_pos: None,
            wallpaper_counter: load_last_counter(),
            is_processing: Arc::new(Mutex::new(false)),
            last_update_time: Instant::now(),
            auto_update_handle: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if !self.window_visible {
            ctx.request_repaint_after(Duration::from_secs(1));
            if self.minimized_to_tray {
                if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.window_visible = true;
                    self.minimized_to_tray = false;
                    frame.set_visible(true);
                    if let Some(pos) = self.last_window_pos {
                        frame.set_window_pos(pos);
                    }
                }
            }
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Wallpaper Updater");
            
            let is_processing = *self.is_processing.lock().unwrap();
            ui.add_enabled_ui(!is_processing, |ui| {
                if ui.button("Ganti Wallpaper").clicked() {
                    self.wallpaper_counter += 1;
                    save_counter(self.wallpaper_counter);
                    let counter = self.wallpaper_counter;
                    let log = Arc::clone(&self.log);
                    let is_processing = Arc::clone(&self.is_processing);
                    let ctx_clone = ctx.clone();
                    
                    let mut urls = WALLPAPER_URLS.to_vec();
                    let mut rng = rand::thread_rng();
                    urls.shuffle(&mut rng);
                    
                    *is_processing.lock().unwrap() = true;
                    thread::spawn(move || {
                        match download_and_set_wallpaper(counter, log.clone(), &urls) {
                            Ok(_) => log.lock().unwrap().push(format!("Wallpaper {} berhasil diterapkan", counter)),
                            Err(e) => log.lock().unwrap().push(format!("Gagal memproses wallpaper {}: {}", counter, e)),
                        }
                        *is_processing.lock().unwrap() = false;
                        ctx_clone.request_repaint();
                    });
                    self.status = format!("Mengganti ke wallpaper_{}", self.wallpaper_counter);
                }
            });
            
            if is_processing {
                ui.spinner();
                ui.label("Sedang memproses wallpaper...");
            }
            
            ui.separator();
            ui.heading("Pengaturan");
            
            ui.checkbox(&mut self.auto_update, "Perbarui wallpaper secara otomatis")
                .on_hover_text("Aktifkan untuk mengganti wallpaper secara berkala");
            
            if self.auto_update {
                ui.horizontal(|ui| {
                    ui.label("Interval:");
                    egui::ComboBox::from_label("")
                        .selected_text(self.update_interval.display_name())
                        .show_ui(ui, |ui| {
                            for interval in UpdateInterval::all_intervals() {
                                ui.selectable_value(&mut self.update_interval, interval, interval.display_name());
                            }
                            ui.selectable_value(&mut self.update_interval, 
                                UpdateInterval::Custom(self.custom_interval.parse().unwrap_or(300)), 
                                "Kustom...");
                        });
                });
                
                if matches!(self.update_interval, UpdateInterval::Custom(_)) {
                    ui.horizontal(|ui| {
                        ui.label("Detik:");
                        ui.text_edit_singleline(&mut self.custom_interval);
                    });
                }
                
                if ui.button("Terapkan Interval").clicked() {
                    self.restart_auto_update(ctx);
                }
            }
            
            if ui.checkbox(&mut self.run_at_startup, "Jalankan saat startup").clicked() {
                configure_run_at_startup(self.run_at_startup);
            }
            
            if ui.checkbox(&mut self.minimized_to_tray, "Minimize ke tray").clicked() {
                if self.minimized_to_tray {
                    self.window_visible = false;
                    self.last_window_pos = Some(frame.info().window_info.position.unwrap_or(egui::Pos2::new(100.0, 100.0)));
                    frame.set_visible(false);
                }
            }
            
            if ui.button("Minimize").clicked() {
                if self.minimized_to_tray {
                    self.window_visible = false;
                    self.last_window_pos = Some(frame.info().window_info.position.unwrap_or(egui::Pos2::new(100.0, 100.0)));
                    frame.set_visible(false);
                } else {
                    frame.set_minimized(true);
                }
            }
            
            ui.separator();
            ui.label(&self.status);
            
            ui.separator();
            ui.heading("Log");
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                let log = self.log.lock().unwrap();
                for entry in log.iter() {
                    ui.label(entry);
                }
            });
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }

    fn on_close_event(&mut self) -> bool {
        if self.minimized_to_tray {
            self.window_visible = false;
            self.last_window_pos = None;
            false
        } else {
            true
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(handle) = self.auto_update_handle.take() {
            handle.join().unwrap(); // Tunggu thread selesai
        }
        configure_run_at_startup(self.run_at_startup);
    }
}

impl MyApp {
    fn restart_auto_update(&mut self, ctx: &egui::Context) {
        if let Some(handle) = self.auto_update_handle.take() {
            handle.join().unwrap();
        }
        
        if self.auto_update {
            let log = Arc::clone(&self.log);
            let interval = if let UpdateInterval::Custom(_) = self.update_interval {
                UpdateInterval::Custom(self.custom_interval.parse().unwrap_or(300))
            } else {
                self.update_interval
            };
            let seconds = interval.as_seconds();
            self.wallpaper_counter += 1;
            let counter = self.wallpaper_counter;
            let is_processing = Arc::clone(&self.is_processing);
            let ctx_clone = ctx.clone();
            
            let mut urls = WALLPAPER_URLS.to_vec();
            let mut rng = rand::thread_rng();
            urls.shuffle(&mut rng);
            
            self.auto_update_handle = Some(thread::spawn(move || {
                loop {
                    if !*is_processing.lock().unwrap() {
                        *is_processing.lock().unwrap() = true;
                        match download_and_set_wallpaper(counter, log.clone(), &urls) {
                            Ok(_) => log.lock().unwrap().push(format!("Wallpaper {} berhasil diterapkan (auto)", counter)),
                            Err(e) => log.lock().unwrap().push(format!("Gagal (auto) wallpaper {}: {}", counter, e)),
                        }
                        *is_processing.lock().unwrap() = false;
                        ctx_clone.request_repaint();
                    }
                    thread::sleep(Duration::from_secs(seconds));
                }
            }));
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 600.0)),
        icon_data: load_icon(),
        centered: true,
        resizable: true,
        ..Default::default()
    };
    
    eframe::run_native(
        "Wallpaper Updater",
        options,
        Box::new(|_cc| {
            let app = MyApp::default();
            if app.run_at_startup {
                configure_run_at_startup(true);
            }
            Box::new(app)
        })
    )
}

const WALLPAPER_URLS: &[&str] = &[
    "https://minimalistic-wallpaper.demolab.com/?random",
    "https://source.unsplash.com/random/1920x1080",
    "https://picsum.photos/1920/1080",
    "https://images.pexels.com/photos/random/1920/1080",
];

fn download_and_set_wallpaper(counter: u32, log: Arc<Mutex<Vec<String>>>, urls: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let wallpaper_dir = get_wallpaper_directory();
    
    let mut attempts = 0;
    const MAX_ATTEMPTS: i32 = 3;
    let bytes = 'outer: loop {
        attempts += 1;
        for &url in urls {
            log.lock().unwrap().push(format!("Mencoba mengambil gambar dari {} (attempt {})", url, attempts));
            match blocking::get(url) {
                Ok(response) => {
                    let bytes = response.bytes()?;
                    if bytes.len() > 2 && &bytes[0..2] == &[0xFF, 0xD8] {
                        log.lock().unwrap().push(format!("Berhasil mendapatkan JPEG dari {}", url));
                        break 'outer bytes;
                    } else {
                        log.lock().unwrap().push(format!("Gambar dari {} bukan JPEG valid", url));
                    }
                }
                Err(e) => {
                    log.lock().unwrap().push(format!("Gagal download dari {}: {}", url, e));
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
        if attempts >= MAX_ATTEMPTS {
            return Err(format!("Gagal mendapatkan JPEG valid setelah {} percobaan dari semua URL", MAX_ATTEMPTS).into());
        }
    };

    let filename = format!("wallpaper_{}.jpg", counter);
    let mut path_jpg = wallpaper_dir.clone();
    path_jpg.push(&filename);
    
    {
        let mut file = File::create(&path_jpg)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }
    log.lock().unwrap().push(format!("Gambar tersimpan di {:?}", path_jpg));
    
    let path_bmp = convert_to_bmp(&path_jpg, log.clone())?;
    set_wallpaper(&path_bmp, log.clone());
    update_wallpaper_registry(log.clone());
    
    cleanup_old_files(&wallpaper_dir, counter, log.clone());
    
    Ok(())
}

fn get_wallpaper_directory() -> PathBuf {
    let path = PathBuf::from("C:\\Wallpapers");
    fs::create_dir_all(&path).expect("Gagal membuat folder Wallpapers");
    path
}

fn convert_to_bmp(path_jpg: &PathBuf, log: Arc<Mutex<Vec<String>>>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let img = ImageReader::open(path_jpg)?.decode()?;
    let path_bmp = path_jpg.with_extension("bmp");
    let mut file = File::create(&path_bmp)?;
    img.write_to(&mut file, ImageFormat::Bmp)?;
    log.lock().unwrap().push(format!("Gambar dikonversi ke {:?}", path_bmp));
    Ok(path_bmp)
}

fn set_wallpaper(path: &PathBuf, log: Arc<Mutex<Vec<String>>>) {
    let path_str = path.to_str().unwrap();
    let mut path_wide: Vec<u16> = path_str.encode_utf16().collect();
    path_wide.push(0);
    unsafe { SystemParametersInfoW(SPI_SETDESKWALLPAPER, 0, path_wide.as_ptr() as *mut _, 3); }
    log.lock().unwrap().push(format!("Wallpaper diubah ke {:?}", path));
}

fn update_wallpaper_registry(log: Arc<Mutex<Vec<String>>>) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Control Panel\\Desktop", KEY_WRITE).expect("Gagal membuka registry");
    key.set_value("WallpaperStyle", &"2").expect("Gagal mengatur WallpaperStyle");
    key.set_value("TileWallpaper", &"0").expect("Gagal mengatur TileWallpaper");
    Command::new("RUNDLL32.EXE").arg("user32.dll,UpdatePerUserSystemParameters").output().expect("Gagal memperbarui sistem.");
    log.lock().unwrap().push("Registry wallpaper diperbarui".to_owned());
}

fn configure_run_at_startup(enable: bool) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    
    let app_name = "WallpaperUpdater";
    let exe_path = std::env::current_exe().unwrap();
    let exe_path_str = exe_path.to_str().unwrap();
    
    if enable {
        if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_WRITE) {
            let _ = key.set_value(app_name, &exe_path_str);
        }
    } else {
        if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_WRITE) {
            let _ = key.delete_value(app_name);
        }
    }
}

fn is_configured_to_run_at_startup() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    
    let app_name = "WallpaperUpdater";
    let exe_path = std::env::current_exe().unwrap();
    let exe_path_str = exe_path.to_str().unwrap();
    
    if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_READ) {
        if let Ok(value) = key.get_value::<String, _>(app_name) {
            return value == exe_path_str;
        }
    }
    false
}

fn save_counter(counter: u32) {
    if let Ok(mut file) = File::create("wallpaper_counter.txt") {
        let _ = write!(file, "{}", counter);
    }
}

fn load_last_counter() -> u32 {
    if let Ok(content) = fs::read_to_string("wallpaper_counter.txt") {
        content.trim().parse().unwrap_or(0)
    } else {
        0
    }
}

fn cleanup_old_files(dir: &PathBuf, current_counter: u32, log: Arc<Mutex<Vec<String>>>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if let Some(file_name) = path.file_stem() {
                if let Some(name) = file_name.to_str() {
                    if name.starts_with("wallpaper_") {
                        if let Ok(number) = name.strip_prefix("wallpaper_").unwrap().parse::<u32>() {
                            if number < current_counter {
                                if let Err(e) = fs::remove_file(&path) {
                                    log.lock().unwrap().push(format!("Gagal menghapus file lama {:?}: {}", path, e));
                                } else {
                                    log.lock().unwrap().push(format!("Menghapus file lama: {:?}", path));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}