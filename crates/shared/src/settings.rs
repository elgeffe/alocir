use eframe::egui;
use eframe::egui::Color32;

use crate::theme::ColorScheme;

pub struct SettingsState {
    pub open: bool,
    pub scheme: ColorScheme,
}

impl SettingsState {
    pub fn new() -> Self {
        SettingsState {
            open: false,
            scheme: ColorScheme::DarkMode,
        }
    }
}

pub fn show_settings_window(ctx: &egui::Context, state: &mut SettingsState) {
    if !state.open {
        return;
    }

    egui::Window::new("Settings")
        .collapsible(false)
        .resizable(false)
        .open(&mut state.open)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(320.0)
        .show(ctx, |ui| {
            // -- About --
            ui.heading("Alocir");
            ui.label(egui::RichText::new("v0.1.0").small().color(Color32::GRAY));
            ui.add_space(4.0);
            ui.label("Cross-platform disk space visualizer.");
            ui.label(
                egui::RichText::new("Inspired by SpaceSniffer")
                    .small()
                    .color(Color32::GRAY),
            );

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            // -- Color Scheme --
            ui.heading("Color Scheme");
            ui.add_space(4.0);

            for &scheme in ColorScheme::ALL {
                if ui
                    .radio_value(&mut state.scheme, scheme, scheme.name())
                    .changed()
                {
                    // Apply dark/light visuals immediately
                    if scheme.is_dark() {
                        ctx.set_visuals(egui::Visuals::dark());
                    } else {
                        ctx.set_visuals(egui::Visuals::light());
                    }
                }
            }
        });
}
