mod app;
mod context_menu;
mod explorer;
mod file_ops;
mod icon;
mod scanner;
mod settings;
mod start_screen;
mod theme;
mod treemap;

use start_screen::StartScreen;

/// Top-level app that transitions from the start screen to the scanner.
enum AppState {
    Start(StartScreen),
    Scanning(app::SpaceSnifferApp),
}

struct AlocirApp {
    state: AppState,
}

impl AlocirApp {
    fn new() -> Self {
        AlocirApp {
            state: AppState::Start(StartScreen::new()),
        }
    }
}

impl eframe::App for AlocirApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        match &mut self.state {
            AppState::Start(start) => {
                start.consume_pending_path();
                if let Some(req) = start.show(ctx) {
                    ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Title(
                        format!("Alocir - {}", req.path.display()),
                    ));

                    self.state = AppState::Scanning(app::SpaceSnifferApp::new(
                        req.path,
                        req.excluded,
                        settings::SettingsState::new(),
                        ctx,
                    ));
                }
            }
            AppState::Scanning(app) => {
                app.update(ctx, frame);
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Alocir")
            .with_icon(std::sync::Arc::new(icon::app_icon())),
        ..Default::default()
    };

    eframe::run_native(
        "Alocir",
        options,
        Box::new(|_cc| Ok(Box::new(AlocirApp::new()))),
    )
}
