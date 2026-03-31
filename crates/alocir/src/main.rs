mod app;

use alocir_shared::settings::SettingsState;
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;

enum AppState {
    Start,
    Scanning(app::SpaceSnifferApp),
}

struct AlocirApp {
    state: AppState,
}

impl AlocirApp {
    fn new() -> Self {
        AlocirApp {
            state: AppState::Start,
        }
    }
}

thread_local! {
    static PENDING_PATH: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

impl eframe::App for AlocirApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut transition: Option<AppState> = None;

        match &mut self.state {
            AppState::Start => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.3);
                        ui.heading("Alocir");
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Disk space visualizer")
                                .color(egui::Color32::GRAY),
                        );
                        ui.add_space(32.0);

                        let btn = egui::Button::new(
                            egui::RichText::new("\u{1F4C2}  Select Folder to Scan").size(18.0),
                        )
                        .min_size(egui::vec2(240.0, 48.0));

                        if ui.add(btn).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Select a directory to scan")
                                .pick_folder()
                            {
                                PENDING_PATH.with(|p| *p.borrow_mut() = Some(path));
                            }
                        }
                    });
                });

                PENDING_PATH.with(|p| {
                    if let Some(path) = p.borrow_mut().take() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                            format!("Alocir - {}", path.display()),
                        ));
                        transition = Some(AppState::Scanning(app::SpaceSnifferApp::new(
                            path,
                            HashSet::new(),
                            SettingsState::new(),
                            ctx,
                        )));
                    }
                });
            }
            AppState::Scanning(app) => {
                app.update(ctx, frame);
            }
        }

        if let Some(new_state) = transition {
            self.state = new_state;
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Alocir")
            .with_icon(std::sync::Arc::new(alocir_shared::icon::app_icon())),
        ..Default::default()
    };

    eframe::run_native(
        "Alocir",
        options,
        Box::new(|_cc| Ok(Box::new(AlocirApp::new()))),
    )
}
