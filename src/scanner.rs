use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct FileNode {
    pub name: String,
    pub size: u64,
    pub children: Vec<FileNode>,
    pub is_dir: bool,
}

pub struct ScanProgress {
    pub items_scanned: AtomicU64,
    pub bytes_scanned: AtomicU64,
    pub current_path: Mutex<String>,
    pub done: Mutex<bool>,
    pub result: Mutex<Option<FileNode>>,
    pub duration: Mutex<Option<Duration>>,
}

impl ScanProgress {
    pub fn new() -> Self {
        ScanProgress {
            items_scanned: AtomicU64::new(0),
            bytes_scanned: AtomicU64::new(0),
            current_path: Mutex::new(String::new()),
            done: Mutex::new(false),
            result: Mutex::new(None),
            duration: Mutex::new(None),
        }
    }
}

impl FileNode {
    pub fn scan_async(
        path: std::path::PathBuf,
        progress: Arc<ScanProgress>,
        excluded: HashSet<PathBuf>,
        ctx: eframe::egui::Context,
    ) {
        let excluded = Arc::new(excluded);
        std::thread::spawn(move || {
            let start = Instant::now();
            let result = FileNode::scan(&path, &progress, &excluded, &ctx);
            let elapsed = start.elapsed();
            *progress.duration.lock().unwrap() = Some(elapsed);
            *progress.result.lock().unwrap() = Some(result);
            *progress.done.lock().unwrap() = true;
            ctx.request_repaint();
        });
    }

    fn scan(
        path: &Path,
        progress: &Arc<ScanProgress>,
        excluded: &Arc<HashSet<PathBuf>>,
        ctx: &eframe::egui::Context,
    ) -> FileNode {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        // Use symlink_metadata to avoid following symlinks for the check
        let meta = match fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(_) => {
                return FileNode {
                    name,
                    size: 0,
                    children: Vec::new(),
                    is_dir: false,
                };
            }
        };

        // Skip symlinks
        if meta.is_symlink() {
            return FileNode {
                name,
                size: 0,
                children: Vec::new(),
                is_dir: false,
            };
        }

        if meta.is_file() {
            let size = meta.len();
            progress.items_scanned.fetch_add(1, Ordering::Relaxed);
            progress.bytes_scanned.fetch_add(size, Ordering::Relaxed);
            return FileNode {
                name,
                size,
                children: Vec::new(),
                is_dir: false,
            };
        }

        // Skip excluded directories.
        if excluded.contains(path) {
            return FileNode {
                name,
                size: 0,
                children: Vec::new(),
                is_dir: true,
            };
        }

        // Directory: update progress path (throttled — only if lock is free)
        progress.items_scanned.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut current) = progress.current_path.try_lock() {
            *current = path.to_string_lossy().to_string();
            ctx.request_repaint();
        }

        // Read directory entries
        let entries: Vec<_> = match fs::read_dir(path) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .filter(|e| {
                    // Skip symlinks early using the DirEntry's file_type (no extra syscall)
                    e.file_type().map(|ft| !ft.is_symlink()).unwrap_or(false)
                })
                .collect(),
            Err(_) => Vec::new(),
        };

        // Scan children in parallel with rayon
        let mut children: Vec<FileNode> = entries
            .par_iter()
            .map(|entry| FileNode::scan(&entry.path(), progress, excluded, ctx))
            .collect();

        let total_size: u64 = children.iter().map(|c| c.size).sum();

        // Sort children by size descending for better treemap layout
        children.sort_unstable_by(|a, b| b.size.cmp(&a.size));

        FileNode {
            name,
            size: total_size,
            children,
            is_dir: true,
        }
    }
}

pub fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        format!("{:.1} TB", b / TB)
    } else if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1), "1 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1023), "1023.0 KB");
    }

    #[test]
    fn format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 500), "500.0 MB");
    }

    #[test]
    fn format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_size(1024u64 * 1024 * 1024 * 10), "10.0 GB");
    }

    #[test]
    fn format_size_terabytes() {
        assert_eq!(format_size(1024u64 * 1024 * 1024 * 1024), "1.0 TB");
        assert_eq!(format_size(1024u64 * 1024 * 1024 * 1024 * 5), "5.0 TB");
    }

    #[test]
    fn file_node_construction() {
        let node = FileNode {
            name: "test.txt".to_string(),
            size: 1024,
            children: Vec::new(),
            is_dir: false,
        };
        assert_eq!(node.name, "test.txt");
        assert_eq!(node.size, 1024);
        assert!(!node.is_dir);
        assert!(node.children.is_empty());
    }

    #[test]
    fn file_node_directory_with_children() {
        let child1 = FileNode {
            name: "big.bin".to_string(),
            size: 2000,
            children: Vec::new(),
            is_dir: false,
        };
        let child2 = FileNode {
            name: "small.txt".to_string(),
            size: 100,
            children: Vec::new(),
            is_dir: false,
        };
        let dir = FileNode {
            name: "mydir".to_string(),
            size: 2100,
            children: vec![child1, child2],
            is_dir: true,
        };
        assert!(dir.is_dir);
        assert_eq!(dir.children.len(), 2);
        assert_eq!(dir.size, 2100);
    }

    #[test]
    fn scan_progress_initial_state() {
        let progress = ScanProgress::new();
        assert_eq!(progress.items_scanned.load(Ordering::Relaxed), 0);
        assert_eq!(progress.bytes_scanned.load(Ordering::Relaxed), 0);
        assert!(!*progress.done.lock().unwrap());
        assert!(progress.result.lock().unwrap().is_none());
        assert!(progress.duration.lock().unwrap().is_none());
    }
}
