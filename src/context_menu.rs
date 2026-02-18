use eframe::egui::{self, Color32};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

pub enum DeferredAction {
    OpenFile(PathBuf),
    RevealInFinder(PathBuf),
    CopyPath(String),
    Cut(PathBuf),
    Copy(PathBuf),
    Paste { target_dir: PathBuf },
    StartRename { path: PathBuf, current_name: String },
    MoveToTrash(PathBuf),
    OpenTerminal(PathBuf),
}

pub fn build_context_menu(
    ui: &mut egui::Ui,
    deferred: &RefCell<Option<DeferredAction>>,
    item_path: &Path,
    item_name: &str,
    is_dir: bool,
    has_clipboard: bool,
    current_dir: &Path,
) {
    if ui.button("Open").clicked() {
        *deferred.borrow_mut() = Some(DeferredAction::OpenFile(item_path.to_path_buf()));
        ui.close_menu();
    }

    let reveal_label = if cfg!(target_os = "macos") {
        "Reveal in Finder"
    } else if cfg!(target_os = "windows") {
        "Show in Explorer"
    } else {
        "Show in File Manager"
    };
    if ui.button(reveal_label).clicked() {
        *deferred.borrow_mut() = Some(DeferredAction::RevealInFinder(item_path.to_path_buf()));
        ui.close_menu();
    }

    if is_dir {
        if ui.button("Open Terminal Here").clicked() {
            *deferred.borrow_mut() =
                Some(DeferredAction::OpenTerminal(item_path.to_path_buf()));
            ui.close_menu();
        }
    }

    ui.separator();

    if ui.button("Copy Path").clicked() {
        *deferred.borrow_mut() = Some(DeferredAction::CopyPath(
            item_path.to_string_lossy().to_string(),
        ));
        ui.close_menu();
    }

    let rename_label = if item_name.len() > 20 {
        format!("Rename \"{}...\"...", &item_name[..17])
    } else {
        format!("Rename \"{}\"...", item_name)
    };
    if ui.button(rename_label).clicked() {
        *deferred.borrow_mut() = Some(DeferredAction::StartRename {
            path: item_path.to_path_buf(),
            current_name: item_name.to_string(),
        });
        ui.close_menu();
    }

    ui.separator();

    if ui.button("Cut").clicked() {
        *deferred.borrow_mut() = Some(DeferredAction::Cut(item_path.to_path_buf()));
        ui.close_menu();
    }

    if ui.button("Copy").clicked() {
        *deferred.borrow_mut() = Some(DeferredAction::Copy(item_path.to_path_buf()));
        ui.close_menu();
    }

    if has_clipboard {
        let target = if is_dir {
            item_path.to_path_buf()
        } else {
            current_dir.to_path_buf()
        };
        if ui.button("Paste").clicked() {
            *deferred.borrow_mut() = Some(DeferredAction::Paste { target_dir: target });
            ui.close_menu();
        }
    }

    ui.separator();

    if ui
        .button(
            egui::RichText::new("Move to Trash").color(Color32::from_rgb(255, 100, 100)),
        )
        .clicked()
    {
        *deferred.borrow_mut() = Some(DeferredAction::MoveToTrash(item_path.to_path_buf()));
        ui.close_menu();
    }
}
