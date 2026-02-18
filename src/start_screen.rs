use eframe::egui;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ─── Public types ────────────────────────────────────────────────────────────

/// Result returned when the user is ready to scan.
pub struct ScanRequest {
    pub path: PathBuf,
    pub excluded: HashSet<PathBuf>,
}

// ─── StartScreen ─────────────────────────────────────────────────────────────

/// State machine for the start screen.
enum Phase {
    PickFolder,
    Configure {
        path: PathBuf,
        tree: FolderTree,
    },
}

pub struct StartScreen {
    phase: Phase,
}

impl StartScreen {
    pub fn new() -> Self {
        StartScreen {
            phase: Phase::PickFolder,
        }
    }

    /// Draw the start screen. Returns `Some(ScanRequest)` when the user clicks
    /// "Start Scan", `None` while still configuring.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<ScanRequest> {
        let mut result: Option<ScanRequest> = None;
        let mut go_back = false;

        egui::CentralPanel::default().show(ctx, |ui| match &mut self.phase {
            Phase::PickFolder => {
                Self::show_pick_folder(ui);
            }
            Phase::Configure { path, tree } => {
                let (r, back) = Self::show_configure(ui, path, tree);
                result = r;
                go_back = back;
            }
        });

        if go_back {
            self.phase = Phase::PickFolder;
        }

        result
    }

    fn show_pick_folder(ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.3);

            ui.heading("Alocir");
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Disk space visualizer").color(egui::Color32::GRAY));

            ui.add_space(32.0);

            let btn = egui::Button::new(egui::RichText::new("\u{1F4C2}  Select Folder").size(18.0))
                .min_size(egui::vec2(200.0, 48.0));

            if ui.add(btn).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Select a directory to scan")
                    .pick_folder()
                {
                    PENDING_PATH.with(|p| *p.borrow_mut() = Some(path));
                }
            }
        });
    }

    fn show_configure(
        ui: &mut egui::Ui,
        path: &PathBuf,
        tree: &mut FolderTree,
    ) -> (Option<ScanRequest>, bool) {
        let mut result: Option<ScanRequest> = None;
        let mut go_back = false;

        ui.vertical(|ui| {
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.heading("Scan:");
                ui.label(
                    egui::RichText::new(path.to_string_lossy().as_ref())
                        .strong()
                        .size(16.0),
                );
            });

            ui.add_space(12.0);

            let available = ui.available_size();

            ui.group(|ui| {
                ui.set_min_height(available.y - 80.0);
                ui.heading("Folders to scan");
                ui.label(
                    egui::RichText::new("Uncheck folders to skip them during the scan.")
                        .small()
                        .color(egui::Color32::GRAY),
                );
                ui.add_space(8.0);

                egui::ScrollArea::both()
                    .max_height(available.y - 140.0)
                    .show(ui, |ui| {
                        tree.show(ui);
                    });
            });

            ui.add_space(12.0);

            // Bottom bar.
            ui.horizontal(|ui| {
                let start_btn =
                    egui::Button::new(egui::RichText::new("\u{25B6}  Start Scan").size(18.0))
                        .min_size(egui::vec2(160.0, 44.0));

                if ui.add(start_btn).clicked() {
                    result = Some(ScanRequest {
                        path: path.clone(),
                        excluded: tree.excluded_paths(),
                    });
                }

                ui.add_space(12.0);

                let back_btn =
                    egui::Button::new(egui::RichText::new("<  Back").size(18.0))
                        .min_size(egui::vec2(160.0, 44.0));

                if ui.add(back_btn).clicked() {
                    go_back = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let excluded_count = tree.excluded_count();
                    if excluded_count > 0 {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} folder{} will be skipped",
                                excluded_count,
                                if excluded_count == 1 { "" } else { "s" }
                            ))
                            .color(egui::Color32::GRAY),
                        );
                    }
                });
            });
        });

        (result, go_back)
    }
}

// Thread-local to shuttle the picked path out of the closure.
thread_local! {
    static PENDING_PATH: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Call this after `show()` to consume any pending folder selection.
/// We need this because `show_pick_folder` can't mutate `self.phase`
/// while it's pattern-matched.
impl StartScreen {
    pub fn consume_pending_path(&mut self) {
        PENDING_PATH.with(|p| {
            if let Some(path) = p.borrow_mut().take() {
                let tree = FolderTree::from_path(&path);
                self.phase = Phase::Configure { path, tree };
            }
        });
    }
}

// ─── FolderTree ──────────────────────────────────────────────────────────────

struct TreeNode {
    name: String,
    path: PathBuf,
    checked: bool,
    children: Vec<TreeNode>,
    expanded: bool,
    /// Whether children have been loaded from disk yet.
    loaded: bool,
    /// Whether this directory has subdirectories (for showing the expand arrow).
    has_subdirs: bool,
}

struct FolderTree {
    root_name: String,
    children: Vec<TreeNode>,
}

impl FolderTree {
    fn from_path(root: &Path) -> Self {
        let root_name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root.to_string_lossy().to_string());

        let children = read_subdirs_shallow(root);
        FolderTree { root_name, children }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("\u{1F4C1} {}", self.root_name))
                    .strong()
                    .size(14.0),
            );
        });

        ui.indent("root_children", |ui| {
            for child in &mut self.children {
                show_tree_node(ui, child);
            }
        });
    }

    fn excluded_paths(&self) -> HashSet<PathBuf> {
        let mut excluded = HashSet::new();
        collect_excluded(&self.children, &mut excluded);
        excluded
    }

    fn excluded_count(&self) -> usize {
        count_excluded(&self.children)
    }
}

// ─── Tree helpers ────────────────────────────────────────────────────────────

/// Read one level of subdirectories (no recursion).
fn read_subdirs_shallow(dir: &Path) -> Vec<TreeNode> {
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
                let name = e.file_name().to_string_lossy().to_string();
                let is_cloud = is_cloud_storage_dir(&path, &name);
                let has_subdirs = has_any_subdir(&path);
                TreeNode {
                    name,
                    path,
                    checked: !is_cloud,
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

/// Quick check: does this directory contain at least one subdirectory?
fn has_any_subdir(dir: &Path) -> bool {
    match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .any(|e| {
                e.file_type()
                    .map(|ft| ft.is_dir() && !ft.is_symlink())
                    .unwrap_or(false)
            }),
        Err(_) => false,
    }
}

fn show_tree_node(ui: &mut egui::Ui, node: &mut TreeNode) {
    let expandable = node.has_subdirs;

    ui.horizontal(|ui| {
        if expandable {
            let arrow = if node.expanded { "\u{25BC}" } else { "\u{25B6}" };
            if ui.add(egui::Button::new(arrow).frame(false)).clicked() {
                node.expanded = !node.expanded;
                // Lazy-load children on first expand.
                if node.expanded && !node.loaded {
                    node.children = read_subdirs_shallow(&node.path);
                    // Inherit parent's unchecked state.
                    if !node.checked {
                        set_checked_recursive(&mut node.children, false);
                    }
                    node.loaded = true;
                }
            }
        } else {
            ui.add_space(22.0);
        }

        let prev = node.checked;
        ui.checkbox(&mut node.checked, "");

        let icon = if node.checked { "\u{1F4C1}" } else { "\u{1F4C2}" };

        let label = if node.checked {
            egui::RichText::new(format!("{} {}", icon, node.name))
        } else {
            egui::RichText::new(format!("{} {} (skip)", icon, node.name))
                .color(egui::Color32::GRAY)
                .strikethrough()
        };
        ui.label(label);

        if prev && !node.checked {
            set_checked_recursive(&mut node.children, false);
        }
        if !prev && node.checked {
            set_checked_recursive(&mut node.children, true);
        }
    });

    if expandable && node.expanded {
        ui.indent(&node.path, |ui| {
            for child in &mut node.children {
                show_tree_node(ui, child);
            }
        });
    }
}

fn set_checked_recursive(nodes: &mut [TreeNode], checked: bool) {
    for node in nodes {
        node.checked = checked;
        set_checked_recursive(&mut node.children, checked);
    }
}

fn collect_excluded(nodes: &[TreeNode], excluded: &mut HashSet<PathBuf>) {
    for node in nodes {
        if !node.checked {
            excluded.insert(node.path.clone());
        } else {
            collect_excluded(&node.children, excluded);
        }
    }
}

fn count_excluded(nodes: &[TreeNode]) -> usize {
    let mut count = 0;
    for node in nodes {
        if !node.checked {
            count += 1;
        } else {
            count += count_excluded(&node.children);
        }
    }
    count
}

// ─── Cloud storage detection ─────────────────────────────────────────────────

/// Check if a directory is a well-known cloud storage location that should be
/// unchecked by default.
fn is_cloud_storage_dir(path: &Path, name: &str) -> bool {
    let name_lower = name.to_lowercase();
    let path_str = path.to_string_lossy();

    // ── macOS unified cloud mount points ─────────────────────────────
    if path_str.contains("/Library/CloudStorage/")
        || path_str.contains("/Library/Mobile Documents/")
        || path_str.ends_with("/Library/CloudStorage")
        || path_str.ends_with("/Library/Mobile Documents")
    {
        return true;
    }

    // ── iCloud ───────────────────────────────────────────────────────
    if name_lower == "icloud drive"
        || name_lower == "iclouddrive"
        || name_lower == "icloud~"
    {
        return true;
    }

    // ── Google Drive ─────────────────────────────────────────────────
    if name_lower == "google drive"
        || name_lower == "google drive file stream"
        || name_lower == "my drive"
        || name_lower.starts_with("googledrive-")
    {
        return true;
    }

    // ── OneDrive ─────────────────────────────────────────────────────
    if name_lower == "onedrive"
        || name_lower == "onedrive - personal"
        || name_lower.starts_with("onedrive - ")
        || name_lower.starts_with("onedrive-")
    {
        return true;
    }

    // ── Dropbox ──────────────────────────────────────────────────────
    if name_lower == "dropbox"
        || name_lower == "dropbox (personal)"
        || name_lower == "dropbox (business)"
        || name_lower.starts_with("dropbox-")
        || name_lower.starts_with("dropbox (")
    {
        return true;
    }

    // ── MEGA ─────────────────────────────────────────────────────────
    if name_lower == "mega"
        || name_lower == "megasync"
        || name_lower == "mega downloads"
    {
        return true;
    }

    // ── Box ──────────────────────────────────────────────────────────
    if name_lower == "box"
        || name_lower == "box sync"
        || name_lower == "box drive"
        || name_lower.starts_with("box-")
    {
        return true;
    }

    // ── pCloud ───────────────────────────────────────────────────────
    if name_lower == "pcloud"
        || name_lower == "pcloud drive"
        || name_lower == "pclouddrive"
        || name_lower.starts_with("pcloud-")
    {
        return true;
    }

    // ── Sync.com ─────────────────────────────────────────────────────
    if name_lower == "sync" || name_lower == "sync.com" {
        return true;
    }

    // ── Tresorit ─────────────────────────────────────────────────────
    if name_lower == "tresorit" {
        return true;
    }

    // ── SpiderOak ────────────────────────────────────────────────────
    if name_lower == "spideroak"
        || name_lower == "spideroak hive"
        || name_lower == "spideroakone"
    {
        return true;
    }

    // ── IDrive ───────────────────────────────────────────────────────
    if name_lower == "idrive" || name_lower == "idrive-sync" {
        return true;
    }

    // ── Nextcloud / ownCloud ─────────────────────────────────────────
    if name_lower == "nextcloud" || name_lower == "owncloud" {
        return true;
    }

    // ── Amazon Drive / Amazon Photos ─────────────────────────────────
    if name_lower == "amazon drive"
        || name_lower == "amazon photos"
        || name_lower == "amazondrive"
    {
        return true;
    }

    // ── Yandex Disk ──────────────────────────────────────────────────
    if name_lower == "yandex.disk"
        || name_lower == "yandexdisk"
        || name_lower == "yandex disk"
    {
        return true;
    }

    false
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_icloud_library_path() {
        let path = PathBuf::from("/Users/jeff/Library/Mobile Documents");
        assert!(is_cloud_storage_dir(&path, "Mobile Documents"));
    }

    #[test]
    fn detects_cloud_storage_path() {
        let path = PathBuf::from("/Users/jeff/Library/CloudStorage/GoogleDrive-user@gmail.com");
        assert!(is_cloud_storage_dir(&path, "GoogleDrive-user@gmail.com"));
    }

    #[test]
    fn detects_onedrive_by_name() {
        let path = PathBuf::from("/Users/jeff/OneDrive");
        assert!(is_cloud_storage_dir(&path, "OneDrive"));
    }

    #[test]
    fn detects_onedrive_business() {
        let path = PathBuf::from("/Users/jeff/OneDrive - Contoso");
        assert!(is_cloud_storage_dir(&path, "OneDrive - Contoso"));
    }

    #[test]
    fn detects_dropbox_by_name() {
        let path = PathBuf::from("/Users/jeff/Dropbox");
        assert!(is_cloud_storage_dir(&path, "Dropbox"));
    }

    #[test]
    fn detects_dropbox_business() {
        let path = PathBuf::from("/Users/jeff/Dropbox (Business)");
        assert!(is_cloud_storage_dir(&path, "Dropbox (Business)"));
    }

    #[test]
    fn detects_google_drive() {
        let path = PathBuf::from("/Users/jeff/Google Drive");
        assert!(is_cloud_storage_dir(&path, "Google Drive"));
    }

    #[test]
    fn detects_mega() {
        let path = PathBuf::from("/Users/jeff/MEGA");
        assert!(is_cloud_storage_dir(&path, "MEGA"));
    }

    #[test]
    fn detects_box() {
        let path = PathBuf::from("/Users/jeff/Box");
        assert!(is_cloud_storage_dir(&path, "Box"));
    }

    #[test]
    fn detects_pcloud() {
        let path = PathBuf::from("/Users/jeff/pCloud Drive");
        assert!(is_cloud_storage_dir(&path, "pCloud Drive"));
    }

    #[test]
    fn detects_amazon_drive() {
        let path = PathBuf::from("/Users/jeff/Amazon Drive");
        assert!(is_cloud_storage_dir(&path, "Amazon Drive"));
    }

    #[test]
    fn detects_yandex_disk() {
        let path = PathBuf::from("/Users/jeff/Yandex.Disk");
        assert!(is_cloud_storage_dir(&path, "Yandex.Disk"));
    }

    #[test]
    fn does_not_flag_normal_dir() {
        let path = PathBuf::from("/Users/jeff/Documents");
        assert!(!is_cloud_storage_dir(&path, "Documents"));
    }

    #[test]
    fn does_not_flag_src() {
        let path = PathBuf::from("/Users/jeff/src");
        assert!(!is_cloud_storage_dir(&path, "src"));
    }

    #[test]
    fn excluded_paths_collects_unchecked() {
        let tree = FolderTree {
            root_name: "root".to_string(),
            children: vec![
                TreeNode {
                    name: "keep".to_string(),
                    path: PathBuf::from("/root/keep"),
                    checked: true,
                    children: Vec::new(),
                    expanded: false,
                    loaded: false,
                    has_subdirs: false,
                },
                TreeNode {
                    name: "skip".to_string(),
                    path: PathBuf::from("/root/skip"),
                    checked: false,
                    children: Vec::new(),
                    expanded: false,
                    loaded: false,
                    has_subdirs: false,
                },
            ],
        };

        let excluded = tree.excluded_paths();
        assert_eq!(excluded.len(), 1);
        assert!(excluded.contains(&PathBuf::from("/root/skip")));
    }

    #[test]
    fn excluded_count_correct() {
        let tree = FolderTree {
            root_name: "root".to_string(),
            children: vec![
                TreeNode {
                    name: "a".to_string(),
                    path: PathBuf::from("/root/a"),
                    checked: false,
                    children: Vec::new(),
                    expanded: false,
                    loaded: false,
                    has_subdirs: false,
                },
                TreeNode {
                    name: "b".to_string(),
                    path: PathBuf::from("/root/b"),
                    checked: false,
                    children: Vec::new(),
                    expanded: false,
                    loaded: false,
                    has_subdirs: false,
                },
                TreeNode {
                    name: "c".to_string(),
                    path: PathBuf::from("/root/c"),
                    checked: true,
                    children: Vec::new(),
                    expanded: false,
                    loaded: false,
                    has_subdirs: false,
                },
            ],
        };

        assert_eq!(tree.excluded_count(), 2);
    }
}
