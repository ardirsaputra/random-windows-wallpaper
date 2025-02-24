fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("icon.ico"); // Sesuaikan dengan nama file ikon
    res.compile().expect("Gagal meng-compile resource");
}
