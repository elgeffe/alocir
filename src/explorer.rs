use eframe::egui;
use eframe::egui::Color32;
use std::cell::RefCell;
use std::path::PathBuf;

use crate::context_menu::{DeferredAction, build_context_menu};
use crate::file_ops::open_path;
use crate::scanner::{FileNode, format_size};

/// Which column the explorer view is sorted by.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Size,
    Type,
    Modified,
}

/// Sort direction.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

pub struct ExplorerState {
    pub sort_col: SortColumn,
    pub sort_dir: SortDir,
    pub address_bar: String,
    pub address_bar_active: bool,
}

impl ExplorerState {
    pub fn new() -> Self {
        ExplorerState {
            sort_col: SortColumn::Name,
            sort_dir: SortDir::Asc,
            address_bar: String::new(),
            address_bar_active: false,
        }
    }
}

/// Render the explorer list view. Returns:
/// - `clicked_dir`: Some(child_index) if user double-clicked a directory
/// - `deferred`: optional context menu action to execute
/// - `address_navigate`: Some(path) if user typed a path and hit Enter
pub fn show_explorer_view(
    ui: &mut egui::Ui,
    node: &FileNode,
    current_dir: &PathBuf,
    state: &mut ExplorerState,
) -> (Option<usize>, Option<DeferredAction>, Option<PathBuf>) {
    let mut clicked_dir: Option<usize> = None;
    let deferred: RefCell<Option<DeferredAction>> = RefCell::new(None);
    let mut address_navigate: Option<PathBuf> = None;

    // Address bar
    ui.horizontal(|ui| {
        ui.label("Path:");
        let resp = ui.add(
            egui::TextEdit::singleline(&mut state.address_bar)
                .desired_width(ui.available_width() - 60.0)
                .hint_text("Type a path and press Enter..."),
        );
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let path = PathBuf::from(&state.address_bar);
            if path.is_dir() {
                address_navigate = Some(path);
            }
        }
        if ui.button("Go").clicked() {
            let path = PathBuf::from(&state.address_bar);
            if path.is_dir() {
                address_navigate = Some(path);
            }
        }
    });

    ui.add_space(4.0);

    // Build sorted index
    let mut indices: Vec<usize> = (0..node.children.len()).collect();
    sort_indices(&node.children, &mut indices, state.sort_col, state.sort_dir);

    // Column headers
    ui.horizontal(|ui| {
        let col_name_w = ui.available_width() * 0.40;
        let col_size_w = ui.available_width() * 0.15;
        let col_type_w = ui.available_width() * 0.15;
        let col_mod_w = ui.available_width() * 0.28;

        if column_header(ui, "Name", col_name_w, state.sort_col == SortColumn::Name, state.sort_dir) {
            toggle_sort(state, SortColumn::Name);
        }
        if column_header(ui, "Size", col_size_w, state.sort_col == SortColumn::Size, state.sort_dir) {
            toggle_sort(state, SortColumn::Size);
        }
        if column_header(ui, "Type", col_type_w, state.sort_col == SortColumn::Type, state.sort_dir) {
            toggle_sort(state, SortColumn::Type);
        }
        if column_header(ui, "Modified", col_mod_w, state.sort_col == SortColumn::Modified, state.sort_dir) {
            toggle_sort(state, SortColumn::Modified);
        }
    });

    ui.separator();

    // Rows in a scroll area
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let total_w = ui.available_width();
            let col_name_w = total_w * 0.40;
            let col_size_w = total_w * 0.15;
            let col_type_w = total_w * 0.15;
            let _col_mod_w = total_w * 0.28;

            for &idx in &indices {
                let child = &node.children[idx];
                let item_path = current_dir.join(&child.name);
                let is_dir = child.is_dir;

                let resp = ui.horizontal(|ui| {
                    // Name column
                    let icon = if is_dir { "\u{1F4C1}" } else { "\u{1F4C4}" };
                    let name_text = format!("{} {}", icon, child.name);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&name_text)
                                .color(if is_dir { Color32::from_rgb(100, 180, 255) } else { Color32::LIGHT_GRAY })
                        )
                        .truncate()
                        .sense(egui::Sense::click())
                    );
                    // Pad to column width
                    let used = ui.min_rect().width();
                    if used < col_name_w {
                        ui.add_space(col_name_w - used);
                    }

                    // Size column
                    let size_str = if is_dir {
                        format_size(child.size)
                    } else {
                        format_size(child.size)
                    };
                    ui.add(egui::Label::new(
                        egui::RichText::new(&size_str).color(Color32::GRAY)
                    ).truncate());
                    let used2 = ui.min_rect().width();
                    if used2 < col_name_w + col_size_w {
                        ui.add_space(col_name_w + col_size_w - used2);
                    }

                    // Type column
                    let type_str = if is_dir {
                        "Folder".to_string()
                    } else {
                        let ext = child.extension();
                        if ext.is_empty() {
                            "File".to_string()
                        } else {
                            format!("{} file", ext.to_uppercase())
                        }
                    };
                    ui.add(egui::Label::new(
                        egui::RichText::new(&type_str).color(Color32::GRAY)
                    ).truncate());
                    let used3 = ui.min_rect().width();
                    if used3 < col_name_w + col_size_w + col_type_w {
                        ui.add_space(col_name_w + col_size_w + col_type_w - used3);
                    }

                    // Modified column
                    let mod_str = match child.modified {
                        Some(epoch) => format_epoch(epoch),
                        None => "-".to_string(),
                    };
                    ui.label(egui::RichText::new(&mod_str).color(Color32::GRAY));
                });

                let row_resp = resp.response.interact(egui::Sense::click());

                // Double-click to navigate into directory or open file
                if row_resp.double_clicked() {
                    if is_dir {
                        clicked_dir = Some(idx);
                    } else {
                        open_path(&item_path);
                    }
                }

                // Context menu
                let item_name = child.name.clone();
                let deferred_ref = &deferred;
                row_resp.context_menu(|ui| {
                    build_context_menu(ui, deferred_ref, &item_path, &item_name, is_dir);
                });
            }
        });

    (clicked_dir, deferred.into_inner(), address_navigate)
}

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

    let btn = ui.add_sized([width, 18.0], egui::Button::new(rich).frame(false));
    btn.clicked()
}

fn toggle_sort(state: &mut ExplorerState, col: SortColumn) {
    if state.sort_col == col {
        state.sort_dir = match state.sort_dir {
            SortDir::Asc => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        };
    } else {
        state.sort_col = col;
        state.sort_dir = match col {
            SortColumn::Name => SortDir::Asc,
            SortColumn::Size => SortDir::Desc,
            SortColumn::Type => SortDir::Asc,
            SortColumn::Modified => SortDir::Desc,
        };
    }
}

fn sort_indices(children: &[FileNode], indices: &mut [usize], col: SortColumn, dir: SortDir) {
    indices.sort_by(|&a, &b| {
        let ca = &children[a];
        let cb = &children[b];

        // Directories always first
        let dir_ord = cb.is_dir.cmp(&ca.is_dir);
        if dir_ord != std::cmp::Ordering::Equal {
            return dir_ord;
        }

        let ord = match col {
            SortColumn::Name => ca.name.to_lowercase().cmp(&cb.name.to_lowercase()),
            SortColumn::Size => ca.size.cmp(&cb.size),
            SortColumn::Type => {
                let ea = ca.extension().to_lowercase();
                let eb = cb.extension().to_lowercase();
                ea.cmp(&eb)
            }
            SortColumn::Modified => {
                ca.modified.unwrap_or(0).cmp(&cb.modified.unwrap_or(0))
            }
        };

        match dir {
            SortDir::Asc => ord,
            SortDir::Desc => ord.reverse(),
        }
    });
}

fn format_epoch(epoch: u64) -> String {
    let secs_since_epoch = epoch;
    let days = secs_since_epoch / 86400;
    let remaining = secs_since_epoch % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;

    // Calculate year/month/day from days since epoch (1970-01-01)
    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        year, month, day, hours, minutes
    )
}

fn days_to_ymd(days_since_epoch: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
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
