//! Nexus Desktop entry point.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod access;
mod ai;
mod app;
mod ide;
mod infra;
mod keys;
mod lock;
mod settings;
mod win;

fn icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../assets/nexus.png");
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Some(egui::IconData { rgba: img.into_raw(), width: w, height: h })
}

fn main() -> eframe::Result {
    // Launched by the com.ljcatherine.Nexus: URL scheme: hand off and exit.
    if access::handle_launch_args() {
        return Ok(());
    }
    // Crash log for diagnosis (no secrets are ever in panic messages we create).
    std::panic::set_hook(Box::new(|info| {
        let dir = settings::local_dir();
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("last-crash.txt"), format!("{info}\n"));
    }));
    let s = settings::Settings::load();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Nexus")
        .with_app_id("com.ljcatherine.NexusDesktop")
        .with_inner_size([1480.0, 920.0])
        .with_min_inner_size([960.0, 600.0]);
    if let Some(i) = icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(i));
    }
    let _ = s;
    let options = eframe::NativeOptions { viewport, persist_window: true, ..Default::default() };
    eframe::run_native("Nexus Desktop", options, Box::new(|cc| Ok(Box::new(app::NexusApp::new(cc)))))
}
