use eframe::egui;
use std::path::PathBuf;

// ─── Public types ────────────────────────────────────────────────────────────

/// Result returned when the user picks a folder.
pub struct ScanRequest {
    pub path: PathBuf,
}

// ─── StartScreen ─────────────────────────────────────────────────────────────

pub struct StartScreen;

impl StartScreen {
    pub fn new() -> Self {
        StartScreen
    }

    /// Draw the start screen. Returns `Some(ScanRequest)` when the user picks a folder.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<ScanRequest> {
        egui::CentralPanel::default().show(ctx, |ui| {
            Self::show_pick_folder(ui);
        });

        let mut result: Option<ScanRequest> = None;
        PENDING_PATH.with(|p| {
            if let Some(path) = p.borrow_mut().take() {
                result = Some(ScanRequest { path });
            }
        });
        result
    }

    fn show_pick_folder(ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.3);

            ui.heading("Alocir");
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Disk space visualizer & file explorer")
                    .color(egui::Color32::GRAY),
            );

            ui.add_space(32.0);

            let btn = egui::Button::new(egui::RichText::new("\u{1F4C2}  Select Folder").size(18.0))
                .min_size(egui::vec2(200.0, 48.0));

            if ui.add(btn).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Select a directory to explore")
                    .pick_folder()
                {
                    PENDING_PATH.with(|p| *p.borrow_mut() = Some(path));
                }
            }
        });
    }
}

// Thread-local to shuttle the picked path out of the closure.
thread_local! {
    static PENDING_PATH: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_screen_creates() {
        let _s = StartScreen::new();
    }
}
