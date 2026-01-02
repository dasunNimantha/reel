fn main() {
    // Only run on Windows
    #[cfg(target_os = "windows")]
    {
        // Only embed icon if the file exists
        if std::path::Path::new("assets/icon.ico").exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_icon("assets/icon.ico");
            res.set("ProductName", "Reel");
            res.set("FileDescription", "Media file organizer and renamer");
            res.set("LegalCopyright", "Copyright © 2024 Dasun P");
            if let Err(e) = res.compile() {
                eprintln!("Warning: Failed to compile Windows resources: {}", e);
            }
        }
    }
}
