fn main() {
    // Compile the resource file (icon.rc)
    embed_resource::compile("icon.rc", embed_resource::NONE);

    // Optionally, use winres to set the icon for the executable
    // let mut res = winres::WindowsResource::new();
    // res.set_icon("icon.ico"); // Ensure this matches your icon file name
    // res.compile().expect("Failed to compile resource");
}