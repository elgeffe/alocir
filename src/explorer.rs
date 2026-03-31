use eframe::egui;
use eframe::egui::Color32;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::context_menu::{DeferredAction, build_context_menu};
use crate::file_ops::{open_path, open_terminal, reveal_in_file_manager};
use crate::scanner::format_size;

// ─── Directory entry (live from filesystem) ─────────────────────────────────

struct DirEntry {
    name: String,
    size: u64,
    is_dir: bool,
    modified: Option<u64>,
}

impl DirEntry {
    fn extension(&self) -> &str {
        if self.is_dir {
            return "";
        }
        match self.name.rsplit_once('.') {
            Some((_, ext)) => ext,
            None => "",
        }
    }
}

fn read_dir_entries(path: &Path) -> Vec<DirEntry> {
    let rd = match std::fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };
    rd.filter_map(|e| e.ok())
        .filter_map(|entry| {
            let ft = entry.file_type().ok()?;
            if ft.is_symlink() {
                return None;
            }
            let meta = entry.metadata().ok()?;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            Some(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                size: if ft.is_file() { meta.len() } else { 0 },
                is_dir: ft.is_dir(),
                modified,
            })
        })
        .collect()
}

// ─── Folder tree (lazy-loaded) ──────────────────────────────────────────────

struct TreeNode {
    name: String,
    path: PathBuf,
    children: Vec<TreeNode>,
    expanded: bool,
    loaded: bool,
    has_subdirs: bool,
}

impl TreeNode {
    fn from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let has_subdirs = has_any_subdir(path);
        TreeNode {
            name,
            path: path.to_path_buf(),
            children: Vec::new(),
            expanded: false,
            loaded: false,
            has_subdirs,
        }
    }

    fn ensure_loaded(&mut self) {
        if self.loaded {
            return;
        }
        self.children = read_subdirs(&self.path);
        self.loaded = true;
    }
}

fn read_subdirs(dir: &Path) -> Vec<TreeNode> {
    let mut entries: Vec<TreeNode> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type()
                    .map(|ft| ft.is_dir() && !ft.is_symlink())
                    .unwrap_or(false)
            })
            .map(|e| {
                let path = e.path();
                let has_subdirs = has_any_subdir(&path);
                TreeNode {
                    name: e.file_name().to_string_lossy().to_string(),
                    path,
                    children: Vec::new(),
                    expanded: false,
                    loaded: false,
                    has_subdirs,
                }
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    entries
}

fn has_any_subdir(dir: &Path) -> bool {
    match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).any(|e| {
            e.file_type()
                .map(|ft| ft.is_dir() && !ft.is_symlink())
                .unwrap_or(false)
        }),
        Err(_) => false,
    }
}

struct FolderTree {
    root: TreeNode,
}

impl FolderTree {
    fn new(root_path: &Path) -> Self {
        let mut root = TreeNode::from_path(root_path);
        root.expanded = true;
        root.ensure_loaded();
        FolderTree { root }
    }

    /// Show the tree and return the path the user clicked, if any.
    fn show(&mut self, ui: &mut egui::Ui, current_dir: &Path) -> Option<PathBuf> {
        let mut clicked: Option<PathBuf> = None;
        Self::show_node(ui, &mut self.root, current_dir, &mut clicked);
        clicked
    }

    fn show_node(
        ui: &mut egui::Ui,
        node: &mut TreeNode,
        current_dir: &Path,
        clicked: &mut Option<PathBuf>,
    ) {
        let is_current = node.path == current_dir;
        let expandable = node.has_subdirs;

        ui.horizontal(|ui| {
            // Expand/collapse arrow
            if expandable {
                let arrow = if node.expanded { "\u{25BC}" } else { "\u{25B6}" };
                if ui.add(egui::Button::new(arrow).frame(false)).clicked() {
                    node.expanded = !node.expanded;
                    if node.expanded {
                        node.ensure_loaded();
                    }
                }
            } else {
                ui.add_space(20.0);
            }

            // Folder icon + name (clickable)
            let icon = if node.expanded && expandable {
                "\u{1F4C2}"
            } else {
                "\u{1F4C1}"
            };
            let label_text = format!("{} {}", icon, node.name);
            let color = if is_current {
                Color32::WHITE
            } else {
                Color32::from_rgb(100, 180, 255)
            };
            let label = egui::Label::new(
                egui::RichText::new(&label_text).color(color).strong(),
            )
            .sense(egui::Sense::click());

            let resp = ui.add(label);
            if resp.clicked() {
                *clicked = Some(node.path.clone());
            }

            // Highlight current directory
            if is_current {
                let rect = resp.rect.expand(2.0);
                ui.painter().rect_stroke(
                    rect,
                    egui::CornerRadius::same(2),
                    egui::Stroke::new(1.0, Color32::from_rgb(80, 140, 220)),
                    egui::StrokeKind::Outside,
                );
            }
        });

        // Show children if expanded
        if node.expanded && !node.children.is_empty() {
            ui.indent(&node.path, |ui| {
                for child in &mut node.children {
                    Self::show_node(ui, child, current_dir, clicked);
                }
            });
        }
    }

    /// Ensure the tree path to `target` is expanded so it's visible.
    fn reveal_path(&mut self, target: &Path) {
        Self::reveal_in_node(&mut self.root, target);
    }

    fn reveal_in_node(node: &mut TreeNode, target: &Path) -> bool {
        if target == node.path {
            return true;
        }
        if !target.starts_with(&node.path) {
            return false;
        }
        // target is under this node — expand and recurse
        node.expanded = true;
        node.ensure_loaded();
        for child in &mut node.children {
            if Self::reveal_in_node(child, target) {
                return true;
            }
        }
        false
    }
}

// ─── Sort ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Name,
    Size,
    Type,
    Modified,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortDir {
    Asc,
    Desc,
}

fn sort_entries(entries: &mut [DirEntry], col: SortColumn, dir: SortDir) {
    entries.sort_by(|a, b| {
        let dir_ord = b.is_dir.cmp(&a.is_dir);
        if dir_ord != std::cmp::Ordering::Equal {
            return dir_ord;
        }
        let ord = match col {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Size => a.size.cmp(&b.size),
            SortColumn::Type => a.extension().to_lowercase().cmp(&b.extension().to_lowercase()),
            SortColumn::Modified => a.modified.unwrap_or(0).cmp(&b.modified.unwrap_or(0)),
        };
        match dir {
            SortDir::Asc => ord,
            SortDir::Desc => ord.reverse(),
        }
    });
}

// ─── Rename state ───────────────────────────────────────────────────────────

struct RenameState {
    path: PathBuf,
    new_name: String,
    error: Option<String>,
}

// ─── ExplorerApp ────────────────────────────────────────────────────────────

pub struct ExplorerApp {
    current_dir: PathBuf,
    entries: Vec<DirEntry>,
    address_bar: String,
    history: Vec<PathBuf>,
    sort_col: SortColumn,
    sort_dir: SortDir,
    rename_state: Option<RenameState>,
    tree: FolderTree,
}

impl ExplorerApp {
    pub fn new(path: PathBuf) -> Self {
        let mut entries = read_dir_entries(&path);
        sort_entries(&mut entries, SortColumn::Name, SortDir::Asc);
        let address_bar = path.to_string_lossy().to_string();
        let tree = FolderTree::new(&path);
        ExplorerApp {
            current_dir: path,
            entries,
            address_bar,
            history: Vec::new(),
            sort_col: SortColumn::Name,
            sort_dir: SortDir::Asc,
            rename_state: None,
            tree,
        }
    }

    pub fn current_path(&self) -> &Path {
        &self.current_dir
    }

    fn navigate_to(&mut self, path: PathBuf) {
        if path == self.current_dir {
            return;
        }
        self.history.push(self.current_dir.clone());
        self.current_dir = path;
        self.reload();
        self.tree.reveal_path(&self.current_dir);
    }

    fn go_back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.current_dir = prev;
            self.reload();
            self.tree.reveal_path(&self.current_dir);
        }
    }

    fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let parent = parent.to_path_buf();
            self.history.push(self.current_dir.clone());
            self.current_dir = parent;
            self.reload();
            self.tree.reveal_path(&self.current_dir);
        }
    }

    fn reload(&mut self) {
        self.entries = read_dir_entries(&self.current_dir);
        sort_entries(&mut self.entries, self.sort_col, self.sort_dir);
        self.address_bar = self.current_dir.to_string_lossy().to_string();
    }

    fn toggle_sort(&mut self, col: SortColumn) {
        if self.sort_col == col {
            self.sort_dir = match self.sort_dir {
                SortDir::Asc => SortDir::Desc,
                SortDir::Desc => SortDir::Asc,
            };
        } else {
            self.sort_col = col;
            self.sort_dir = match col {
                SortColumn::Name => SortDir::Asc,
                SortColumn::Size => SortDir::Desc,
                SortColumn::Type => SortDir::Asc,
                SortColumn::Modified => SortDir::Desc,
            };
        }
        sort_entries(&mut self.entries, self.sort_col, self.sort_dir);
    }

    fn execute_deferred(&mut self, action: DeferredAction, ctx: &egui::Context) {
        match action {
            DeferredAction::OpenFile(p) => open_path(&p),
            DeferredAction::RevealInFinder(p) => reveal_in_file_manager(&p),
            DeferredAction::CopyPath(s) => ctx.copy_text(s),
            DeferredAction::StartRename { path, current_name } => {
                self.rename_state = Some(RenameState {
                    path,
                    new_name: current_name,
                    error: None,
                });
            }
            DeferredAction::MoveToTrash(p) => {
                if trash::delete(&p).is_ok() {
                    self.reload();
                }
            }
            DeferredAction::OpenTerminal(p) => open_terminal(&p),
        }
    }

    /// Returns true if the user wants to switch to treemap view.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut switch_to_treemap = false;

        // Top toolbar
        egui::TopBottomPanel::top("explorer_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let has_history = !self.history.is_empty();
                if ui
                    .add_enabled(has_history, egui::Button::new("\u{2190}"))
                    .on_hover_text("Back")
                    .clicked()
                {
                    self.go_back();
                }

                let has_parent = self.current_dir.parent().is_some();
                if ui
                    .add_enabled(has_parent, egui::Button::new("\u{2191}"))
                    .on_hover_text("Up")
                    .clicked()
                {
                    self.go_up();
                }

                if ui.button("\u{21BB}").on_hover_text("Refresh").clicked() {
                    self.reload();
                }

                if ui
                    .button("\u{25A6}")
                    .on_hover_text("Switch to Treemap (full scan)")
                    .clicked()
                {
                    switch_to_treemap = true;
                }

                ui.separator();

                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.address_bar)
                        .desired_width(ui.available_width())
                        .hint_text("Type a path and press Enter..."),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let path = PathBuf::from(&self.address_bar);
                    if path.is_dir() {
                        self.navigate_to(path);
                    }
                }
            });
        });

        // Left panel: folder tree
        let mut tree_nav: Option<PathBuf> = None;
        egui::SidePanel::left("folder_tree")
            .default_width(240.0)
            .min_width(150.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        tree_nav = self.tree.show(ui, &self.current_dir);
                    });
            });

        // Right panel: file list
        let deferred: RefCell<Option<DeferredAction>> = RefCell::new(None);
        let mut nav_to: Option<PathBuf> = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            // Column headers
            ui.horizontal(|ui| {
                let w = ui.available_width();
                if column_header(ui, "Name", w * 0.40, self.sort_col == SortColumn::Name, self.sort_dir) {
                    self.toggle_sort(SortColumn::Name);
                }
                if column_header(ui, "Size", w * 0.15, self.sort_col == SortColumn::Size, self.sort_dir) {
                    self.toggle_sort(SortColumn::Size);
                }
                if column_header(ui, "Type", w * 0.15, self.sort_col == SortColumn::Type, self.sort_dir) {
                    self.toggle_sort(SortColumn::Type);
                }
                if column_header(ui, "Modified", w * 0.28, self.sort_col == SortColumn::Modified, self.sort_dir) {
                    self.toggle_sort(SortColumn::Modified);
                }
            });
            ui.separator();

            if self.entries.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("Empty folder");
                });
                return;
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let total_w = ui.available_width();
                    let col_name_w = total_w * 0.40;
                    let col_size_w = total_w * 0.15;
                    let col_type_w = total_w * 0.15;

                    for entry in &self.entries {
                        let item_path = self.current_dir.join(&entry.name);
                        let is_dir = entry.is_dir;

                        let resp = ui.horizontal(|ui| {
                            let icon = if is_dir { "\u{1F4C1}" } else { "\u{1F4C4}" };
                            let name_text = format!("{} {}", icon, entry.name);
                            let color = if is_dir {
                                Color32::from_rgb(100, 180, 255)
                            } else {
                                Color32::LIGHT_GRAY
                            };
                            ui.add(
                                egui::Label::new(egui::RichText::new(&name_text).color(color))
                                    .truncate()
                                    .sense(egui::Sense::click()),
                            );
                            let used = ui.min_rect().width();
                            if used < col_name_w {
                                ui.add_space(col_name_w - used);
                            }

                            let size_str = if is_dir {
                                "-".to_string()
                            } else {
                                format_size(entry.size)
                            };
                            ui.add(
                                egui::Label::new(egui::RichText::new(&size_str).color(Color32::GRAY))
                                    .truncate(),
                            );
                            let used2 = ui.min_rect().width();
                            if used2 < col_name_w + col_size_w {
                                ui.add_space(col_name_w + col_size_w - used2);
                            }

                            let type_str = if is_dir {
                                "Folder".to_string()
                            } else {
                                let ext = entry.extension();
                                if ext.is_empty() {
                                    "File".to_string()
                                } else {
                                    format!("{} file", ext.to_uppercase())
                                }
                            };
                            ui.add(
                                egui::Label::new(egui::RichText::new(&type_str).color(Color32::GRAY))
                                    .truncate(),
                            );
                            let used3 = ui.min_rect().width();
                            if used3 < col_name_w + col_size_w + col_type_w {
                                ui.add_space(col_name_w + col_size_w + col_type_w - used3);
                            }

                            let mod_str = match entry.modified {
                                Some(epoch) => format_epoch(epoch),
                                None => "-".to_string(),
                            };
                            ui.label(egui::RichText::new(&mod_str).color(Color32::GRAY));
                        });

                        let row_resp = resp.response.interact(egui::Sense::click());

                        if row_resp.double_clicked() {
                            if is_dir {
                                nav_to = Some(item_path.clone());
                            } else {
                                open_path(&item_path);
                            }
                        }

                        let item_name = entry.name.clone();
                        let deferred_ref = &deferred;
                        row_resp.context_menu(|ui| {
                            build_context_menu(ui, deferred_ref, &item_path, &item_name, is_dir);
                        });
                    }
                });
        });

        // Handle tree navigation
        if let Some(path) = tree_nav {
            self.navigate_to(path);
        }

        // Handle file list navigation
        if let Some(path) = nav_to {
            self.navigate_to(path);
        }

        if let Some(action) = deferred.into_inner() {
            self.execute_deferred(action, ctx);
        }

        self.show_rename_dialog(ctx);

        switch_to_treemap
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
                            self.reload();
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
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn column_header(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    is_active: bool,
    dir: SortDir,
) -> bool {
    let arrow = if is_active {
        match dir {
            SortDir::Asc => " \u{25B2}",
            SortDir::Desc => " \u{25BC}",
        }
    } else {
        ""
    };
    let text = format!("{}{}", label, arrow);
    let rich = if is_active {
        egui::RichText::new(text).strong()
    } else {
        egui::RichText::new(text)
    };
    ui.add_sized([width, 18.0], egui::Button::new(rich).frame(false))
        .clicked()
}

fn format_epoch(epoch: u64) -> String {
    let days = epoch / 86400;
    let remaining = epoch % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        year, month, day, hours, minutes
    )
}

fn days_to_ymd(days_since_epoch: u64) -> (u64, u64, u64) {
    let z = days_since_epoch + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
