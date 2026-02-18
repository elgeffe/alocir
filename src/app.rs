use eframe::egui;
use eframe::egui::{Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind};
use eframe::emath::{Align2, pos2};
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::context_menu::{DeferredAction, build_context_menu};
use crate::file_ops::{
    ClipEntry, copy_dir_recursive, open_path, open_terminal, reveal_in_file_manager,
};
use crate::scanner::{FileNode, ScanProgress, format_size};
use crate::settings::{SettingsState, show_settings_window};
use crate::theme::ThemeColors;
use crate::treemap::squarify;

struct RenameState {
    path: PathBuf,
    new_name: String,
    error: Option<String>,
}

pub struct SpaceSnifferApp {
    root: Option<FileNode>,
    root_path: String,
    scan_path: PathBuf,
    excluded: HashSet<PathBuf>,
    nav_stack: Vec<usize>,
    scan_progress: Arc<ScanProgress>,
    scanning: bool,
    clipboard: Option<ClipEntry>,
    rename_state: Option<RenameState>,
    saved_nav_names: Option<Vec<String>>,
    scan_duration: Option<std::time::Duration>,
    settings: SettingsState,
}

impl SpaceSnifferApp {
    pub fn new(path: PathBuf, excluded: HashSet<PathBuf>, settings: SettingsState, ctx: &egui::Context) -> Self {
        let root_path = path.to_string_lossy().to_string();
        let progress = Arc::new(ScanProgress::new());
        FileNode::scan_async(path.clone(), Arc::clone(&progress), excluded.clone(), ctx.clone());
        SpaceSnifferApp {
            root: None,
            root_path,
            scan_path: path,
            excluded,
            nav_stack: Vec::new(),
            scan_progress: progress,
            scanning: true,
            clipboard: None,
            rename_state: None,
            saved_nav_names: None,
            scan_duration: None,
            settings,
        }
    }

    fn theme(&self) -> ThemeColors {
        self.settings.scheme.theme()
    }

    fn current_node(&self) -> Option<&FileNode> {
        let mut node = self.root.as_ref()?;
        for &idx in &self.nav_stack {
            if idx < node.children.len() {
                node = &node.children[idx];
            } else {
                return None;
            }
        }
        Some(node)
    }

    fn current_dir_path(&self) -> PathBuf {
        let mut path = self.scan_path.clone();
        if let Some(root) = &self.root {
            let mut node = root;
            for &idx in &self.nav_stack {
                if idx < node.children.len() {
                    node = &node.children[idx];
                    path = path.join(&node.name);
                }
            }
        }
        path
    }

    fn nav_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(root) = &self.root {
            let mut node = root;
            for &idx in &self.nav_stack {
                if idx < node.children.len() {
                    node = &node.children[idx];
                    names.push(node.name.clone());
                }
            }
        }
        names
    }

    fn restore_nav(root: &FileNode, names: &[String]) -> Vec<usize> {
        let mut stack = Vec::new();
        let mut node = root;
        for name in names {
            if let Some(idx) = node.children.iter().position(|c| &c.name == name) {
                stack.push(idx);
                node = &node.children[idx];
            } else {
                break;
            }
        }
        stack
    }

    fn trigger_rescan(&mut self, ctx: &egui::Context) {
        self.saved_nav_names = Some(self.nav_names());
        let progress = Arc::new(ScanProgress::new());
        self.scan_progress = Arc::clone(&progress);
        self.scanning = true;
        self.scan_duration = None;
        FileNode::scan_async(self.scan_path.clone(), progress, self.excluded.clone(), ctx.clone());
    }

    // -- File operations --

    fn do_paste(&mut self, target_dir: PathBuf, ctx: &egui::Context) {
        if let Some(clip) = self.clipboard.take() {
            let file_name = clip.path.file_name().unwrap_or_default().to_os_string();
            let dest = target_dir.join(&file_name);

            let result = if clip.is_cut {
                std::fs::rename(&clip.path, &dest).or_else(|_| {
                    if clip.path.is_dir() {
                        copy_dir_recursive(&clip.path, &dest)
                            .and_then(|_| std::fs::remove_dir_all(&clip.path))
                    } else {
                        std::fs::copy(&clip.path, &dest)
                            .and_then(|_| std::fs::remove_file(&clip.path))
                    }
                })
            } else if clip.path.is_dir() {
                copy_dir_recursive(&clip.path, &dest)
            } else {
                std::fs::copy(&clip.path, &dest).map(|_| ())
            };

            if let Err(e) = result {
                eprintln!("Paste failed: {}", e);
                self.clipboard = Some(clip);
            } else {
                self.trigger_rescan(ctx);
            }
        }
    }

    fn do_trash(&mut self, path: PathBuf, ctx: &egui::Context) {
        match trash::delete(&path) {
            Ok(()) => self.trigger_rescan(ctx),
            Err(e) => eprintln!("Move to trash failed: {}", e),
        }
    }

    // -- UI sections --

    fn show_scanning_ui(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let items = self
                .scan_progress
                .items_scanned
                .load(std::sync::atomic::Ordering::Relaxed);
            let bytes = self
                .scan_progress
                .bytes_scanned
                .load(std::sync::atomic::Ordering::Relaxed);
            let current = self.scan_progress.current_path.lock().unwrap().clone();

            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.3);
                ui.heading("Scanning...");
                ui.add_space(16.0);
                ui.add(egui::Spinner::new().size(32.0));
                ui.add_space(12.0);
                ui.label(format!(
                    "{} items scanned  |  {} processed",
                    items,
                    format_size(bytes),
                ));
                ui.add_space(8.0);
                let display_path = if current.len() > 80 {
                    format!("...{}", &current[current.len() - 77..])
                } else {
                    current
                };
                ui.label(egui::RichText::new(display_path).small().color(Color32::GRAY));
            });
        });
    }

    fn show_rename_dialog(&mut self, ctx: &egui::Context) {
        let mut should_close = false;
        let mut should_rename = false;

        if let Some(state) = &mut self.rename_state {
            let mut open = true;
            egui::Window::new("Rename")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("New name:");
                        let resp = ui.text_edit_singleline(&mut state.new_name);
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            should_rename = true;
                        }
                    });
                    if let Some(err) = &state.error {
                        ui.colored_label(Color32::RED, err);
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Rename").clicked() {
                            should_rename = true;
                        }
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }
                    });
                });
            if !open {
                should_close = true;
            }
        }

        if should_rename {
            if let Some(state) = &self.rename_state {
                if let Some(parent) = state.path.parent() {
                    let new_path = parent.join(&state.new_name);
                    match std::fs::rename(&state.path, &new_path) {
                        Ok(()) => {
                            self.rename_state = None;
                            self.trigger_rescan(ctx);
                        }
                        Err(e) => {
                            if let Some(s) = &mut self.rename_state {
                                s.error = Some(e.to_string());
                            }
                        }
                    }
                }
            }
        } else if should_close {
            self.rename_state = None;
        }
    }

    fn show_treemap_ui(&mut self, ctx: &egui::Context) {
        let theme = self.theme();

        egui::TopBottomPanel::top("breadcrumb").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Settings gear
                if ui.button("\u{2699}").on_hover_text("Settings").clicked() {
                    self.settings.open = !self.settings.open;
                }

                // New scan button
                if ui
                    .button("\u{1F4C2}")
                    .on_hover_text("New Scan (opens new window)")
                    .clicked()
                {
                    spawn_new_instance();
                }

                ui.separator();

                // Breadcrumb navigation
                if ui
                    .selectable_label(self.nav_stack.is_empty(), &self.root_path)
                    .clicked()
                {
                    self.nav_stack.clear();
                }

                if let Some(root) = &self.root {
                    let mut node = root;
                    let mut breadcrumb_click: Option<usize> = None;
                    for depth in 0..self.nav_stack.len() {
                        let idx = self.nav_stack[depth];
                        if idx < node.children.len() {
                            node = &node.children[idx];
                            ui.label(">");
                            let is_current = depth == self.nav_stack.len() - 1;
                            if ui.selectable_label(is_current, &node.name).clicked() {
                                breadcrumb_click = Some(depth + 1);
                            }
                        }
                    }
                    if let Some(len) = breadcrumb_click {
                        self.nav_stack.truncate(len);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(dur) = self.scan_duration {
                        let secs = dur.as_secs_f64();
                        let time_str = if secs >= 60.0 {
                            format!("{:.0}m {:.0}s", (secs / 60.0).floor(), secs % 60.0)
                        } else {
                            format!("{:.1}s", secs)
                        };
                        ui.label(
                            egui::RichText::new(format!("Scanned in {}", time_str))
                                .small()
                                .color(Color32::GRAY),
                        );
                        ui.separator();
                    }
                    if let Some(node) = self.current_node() {
                        ui.label(format!("Total: {}", format_size(node.size)));
                    }
                });
            });
        });

        let current_dir = self.current_dir_path();
        let has_clipboard = self.clipboard.is_some();

        egui::CentralPanel::default().show(ctx, |ui| {
            let children_info = {
                let node = match self.current_node() {
                    Some(n) => n,
                    None => {
                        ui.label("No data to display.");
                        return;
                    }
                };

                if node.children.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(format!("{}\n{}", node.name, format_size(node.size)));
                    });
                    return;
                }

                let bounds = ui.available_rect_before_wrap().shrink(2.0);
                let items: Vec<(usize, f64)> = node
                    .children
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.size > 0)
                    .map(|(i, c)| (i, c.size as f64))
                    .collect();
                let layout = squarify(&items, bounds);

                layout
                    .iter()
                    .map(|item| {
                        let child = &node.children[item.index];
                        let color =
                            theme.color_for_node(&child.name, child.is_dir);
                        (
                            item.rect,
                            color,
                            child.name.clone(),
                            format_size(child.size),
                            child.is_dir,
                            item.index,
                        )
                    })
                    .collect::<Vec<(Rect, Color32, String, String, bool, usize)>>()
            };

            let mut clicked_dir: Option<usize> = None;
            let deferred: RefCell<Option<DeferredAction>> = RefCell::new(None);

            struct RenderItem {
                inner: Rect,
                color: Color32,
                name: String,
                size_str: String,
                is_dir: bool,
                hovered: bool,
            }
            let mut render_items: Vec<RenderItem> = Vec::new();

            // Pass 1: allocate rects, clicks, context menus
            for (rect, color, name, size_str, is_dir, index) in &children_info {
                let inner = rect.shrink(1.5);
                if inner.width() < 1.0 || inner.height() < 1.0 {
                    continue;
                }

                let response = ui.allocate_rect(inner, Sense::click().union(Sense::hover()));
                if response.clicked() && *is_dir {
                    clicked_dir = Some(*index);
                }

                let response = response.on_hover_text(format!(
                    "{}\n{}{}",
                    name,
                    size_str,
                    if *is_dir { "\n(click to expand)" } else { "" }
                ));

                let item_path = current_dir.join(name);
                let item_name = name.clone();
                let item_is_dir = *is_dir;
                let deferred_ref = &deferred;
                response.context_menu(|ui| {
                    build_context_menu(
                        ui,
                        deferred_ref,
                        &item_path,
                        &item_name,
                        item_is_dir,
                        has_clipboard,
                        &current_dir,
                    );
                });

                render_items.push(RenderItem {
                    inner,
                    color: *color,
                    name: name.clone(),
                    size_str: size_str.clone(),
                    is_dir: *is_dir,
                    hovered: response.hovered(),
                });
            }

            // Pass 2: paint
            let painter = ui.painter();
            for item in &render_items {
                let bg = if item.hovered {
                    theme.hover_color(item.color)
                } else {
                    item.color
                };
                painter.rect_filled(item.inner, CornerRadius::same(3), bg);

                let border = if item.is_dir {
                    theme.dir_border
                } else {
                    theme.file_border
                };
                painter.rect_stroke(
                    item.inner,
                    CornerRadius::same(3),
                    Stroke::new(1.0, border),
                    StrokeKind::Outside,
                );

                if item.inner.width() >= 30.0 && item.inner.height() >= 20.0 {
                    let center = item.inner.center();
                    let fs =
                        (item.inner.width().min(item.inner.height()) * 0.12).clamp(9.0, 16.0);
                    let font = FontId::proportional(fs);

                    let max_chars = (item.inner.width() / (fs * 0.5)) as usize;
                    let label = if item.name.len() > max_chars && max_chars > 3 {
                        format!("{}...", &item.name[..max_chars - 3])
                    } else {
                        item.name.clone()
                    };

                    painter.text(
                        pos2(center.x, center.y - fs * 0.5),
                        Align2::CENTER_CENTER,
                        &label,
                        font,
                        theme.text_primary,
                    );

                    if item.inner.height() >= 40.0 {
                        painter.text(
                            pos2(center.x, center.y + fs * 0.6),
                            Align2::CENTER_CENTER,
                            &item.size_str,
                            FontId::proportional((fs * 0.85).max(8.0)),
                            theme.text_secondary,
                        );
                    }

                    if item.is_dir && item.inner.width() > 50.0 && item.inner.height() > 40.0 {
                        painter.text(
                            pos2(item.inner.right() - 8.0, item.inner.top() + 8.0),
                            Align2::RIGHT_TOP,
                            "+",
                            FontId::proportional(10.0),
                            theme.indicator,
                        );
                    }
                }
            }

            if let Some(idx) = clicked_dir {
                self.nav_stack.push(idx);
            }

            // Execute deferred context menu action
            if let Some(action) = deferred.into_inner() {
                match action {
                    DeferredAction::OpenFile(p) => open_path(&p),
                    DeferredAction::RevealInFinder(p) => reveal_in_file_manager(&p),
                    DeferredAction::CopyPath(s) => ctx.copy_text(s),
                    DeferredAction::Cut(p) => {
                        self.clipboard = Some(ClipEntry { path: p, is_cut: true });
                    }
                    DeferredAction::Copy(p) => {
                        self.clipboard = Some(ClipEntry { path: p, is_cut: false });
                    }
                    DeferredAction::Paste { target_dir } => self.do_paste(target_dir, ctx),
                    DeferredAction::StartRename { path, current_name } => {
                        self.rename_state = Some(RenameState {
                            path,
                            new_name: current_name,
                            error: None,
                        });
                    }
                    DeferredAction::MoveToTrash(p) => self.do_trash(p, ctx),
                    DeferredAction::OpenTerminal(p) => open_terminal(&p),
                }
            }
        });
    }
}

impl eframe::App for SpaceSnifferApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.scanning {
            let done = *self.scan_progress.done.lock().unwrap();
            if done {
                self.root = self.scan_progress.result.lock().unwrap().take();
                self.scan_duration = self.scan_progress.duration.lock().unwrap().take();
                self.scanning = false;
                if let Some(names) = self.saved_nav_names.take() {
                    if let Some(root) = &self.root {
                        self.nav_stack = Self::restore_nav(root, &names);
                    }
                }
            }
        }

        if self.scanning {
            self.show_scanning_ui(ctx);
        } else {
            self.show_treemap_ui(ctx);
            self.show_rename_dialog(ctx);
            show_settings_window(ctx, &mut self.settings);
        }
    }
}

/// Spawn a new instance of the app as a separate process.
fn spawn_new_instance() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::process::Command::new(exe).spawn();
    }
}
