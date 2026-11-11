fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/nexus.ico")
            .set("ProductName", "Nexus Desktop")
            .set("FileDescription", "Nexus Desktop")
            .set("CompanyName", "Liam Catherine")
            .set("LegalCopyright", "© 2026 Liam Catherine")
            .set("OriginalFilename", "NexusDesktop.exe");
        if let Err(e) = res.compile() {
            println!("cargo:warning=resource compile failed: {e}");
        }
    }
}
