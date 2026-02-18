use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ClipEntry {
    pub path: PathBuf,
    pub is_cut: bool,
}

/// Copy a directory tree recursively.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Open a file or directory with the OS default application.
pub fn open_path(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

/// Reveal a file in the native file manager (Finder / Explorer / etc).
pub fn reveal_in_file_manager(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.to_string_lossy()))
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
        }
    }
}

/// Open a terminal window at the given directory.
pub fn open_terminal(path: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .args(["-a", "Terminal", &path.to_string_lossy()])
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args([
                "/C",
                "start",
                "cmd",
                "/K",
                &format!("cd /d {}", path.to_string_lossy()),
            ])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let terminals = [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xterm",
        ];
        for term in &terminals {
            if std::process::Command::new(term)
                .current_dir(path)
                .spawn()
                .is_ok()
            {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn clip_entry_copy() {
        let entry = ClipEntry {
            path: PathBuf::from("/tmp/test.txt"),
            is_cut: false,
        };
        assert!(!entry.is_cut);
        assert_eq!(entry.path, PathBuf::from("/tmp/test.txt"));
    }

    #[test]
    fn clip_entry_cut() {
        let entry = ClipEntry {
            path: PathBuf::from("/tmp/test.txt"),
            is_cut: true,
        };
        assert!(entry.is_cut);
    }

    #[test]
    fn clip_entry_clone() {
        let entry = ClipEntry {
            path: PathBuf::from("/tmp/test.txt"),
            is_cut: true,
        };
        let cloned = entry.clone();
        assert_eq!(cloned.path, entry.path);
        assert_eq!(cloned.is_cut, entry.is_cut);
    }

    #[test]
    fn copy_dir_recursive_flat() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let dst_path = dst.path().join("copy");

        std::fs::write(src.path().join("a.txt"), "hello").unwrap();
        std::fs::write(src.path().join("b.txt"), "world").unwrap();

        copy_dir_recursive(src.path(), &dst_path).unwrap();

        assert_eq!(
            std::fs::read_to_string(dst_path.join("a.txt")).unwrap(),
            "hello"
        );
        assert_eq!(
            std::fs::read_to_string(dst_path.join("b.txt")).unwrap(),
            "world"
        );
    }

    #[test]
    fn copy_dir_recursive_nested() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let dst_path = dst.path().join("copy");

        std::fs::create_dir(src.path().join("sub")).unwrap();
        std::fs::write(src.path().join("sub/deep.txt"), "nested").unwrap();

        copy_dir_recursive(src.path(), &dst_path).unwrap();

        assert_eq!(
            std::fs::read_to_string(dst_path.join("sub/deep.txt")).unwrap(),
            "nested"
        );
    }

    #[test]
    fn copy_dir_recursive_empty_dir() {
        let src = TempDir::new().unwrap();
        let dst = TempDir::new().unwrap();
        let dst_path = dst.path().join("copy");

        copy_dir_recursive(src.path(), &dst_path).unwrap();

        assert!(dst_path.exists());
        assert!(dst_path.is_dir());
    }
}
