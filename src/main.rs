mod app;
mod context_menu;
mod file_ops;
mod scanner;
mod settings;
mod theme;
mod treemap;

use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    // Show native directory picker
    let path: PathBuf = match rfd::FileDialog::new()
        .set_title("Select a directory to scan")
        .pick_folder()
    {
        Some(p) => p,
        None => {
            eprintln!("No directory selected. Exiting.");
            return Ok(());
        }
    };

    let title = format!("Alocir - {}", path.display());

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title(&title),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| Ok(Box::new(app::SpaceSnifferApp::new(path, &cc.egui_ctx)))),
    )
}
