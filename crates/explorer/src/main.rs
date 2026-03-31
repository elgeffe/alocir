mod explorer;

use eframe::egui;
use explorer::ExplorerApp;
use std::path::PathBuf;

enum AppState {
    Start,
    Exploring(ExplorerApp),
}

struct App {
    state: AppState,
}

impl App {
    fn new() -> Self {
        App {
            state: AppState::Start,
        }
    }
}

thread_local! {
    static PENDING_PATH: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut transition: Option<AppState> = None;

        match &mut self.state {
            AppState::Start => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.3);
                        ui.heading("Alocir Explorer");
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Fast file explorer")
                                .color(egui::Color32::GRAY),
                        );
                        ui.add_space(32.0);

                        let btn = egui::Button::new(
                            egui::RichText::new("\u{1F4C2}  Select Folder").size(18.0),
                        )
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
                });

                PENDING_PATH.with(|p| {
                    if let Some(path) = p.borrow_mut().take() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                            format!("Alocir Explorer - {}", path.display()),
                        ));
                        transition = Some(AppState::Exploring(ExplorerApp::new(path)));
                    }
                });
            }
            AppState::Exploring(explorer) => {
                let _switch_to_treemap = explorer.show(ctx);
                // Treemap switch is handled by the alocir binary, not here
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
            .with_title("Alocir Explorer")
            .with_icon(std::sync::Arc::new(alocir_shared::icon::app_icon())),
        ..Default::default()
    };

    eframe::run_native(
        "Alocir Explorer",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}
