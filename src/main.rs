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

use explorer::ExplorerApp;
use start_screen::StartScreen;

/// Top-level app that transitions between screens.
enum AppState {
    Start(StartScreen),
    Explorer(ExplorerApp),
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
        let mut transition: Option<AppState> = None;

        match &mut self.state {
            AppState::Start(start) => {
                if let Some(req) = start.show(ctx) {
                    ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Title(
                        format!("Alocir - {}", req.path.display()),
                    ));
                    // Go straight to explorer — no scan needed
                    transition = Some(AppState::Explorer(ExplorerApp::new(req.path)));
                }
            }
            AppState::Explorer(explorer) => {
                let switch_to_treemap = explorer.show(ctx);
                if switch_to_treemap {
                    // User wants treemap — trigger full scan
                    let path = explorer.current_path().to_path_buf();
                    ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Title(
                        format!("Alocir - {} (scanning)", path.display()),
                    ));
                    transition = Some(AppState::Scanning(app::SpaceSnifferApp::new(
                        path,
                        std::collections::HashSet::new(),
                        settings::SettingsState::new(),
                        ctx,
                    )));
                }
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
