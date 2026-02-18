# Alocir

Cross-platform disk space visualizer built with Rust and egui. Inspired by SpaceSniffer.

## Architecture

The app is a two-phase state machine defined in `src/main.rs`:

1. **Start screen** (`StartScreen` in `src/start_screen.rs`) — folder picker, lazy-loaded folder tree with cloud storage auto-detection, exclusion checkboxes
2. **Scan + treemap** (`SpaceSnifferApp` in `src/app.rs`) — parallel directory scan with progress UI, then interactive squarified treemap visualization

### Source files

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, `AlocirApp` state machine (`Start` / `Scanning`) |
| `src/app.rs` | `SpaceSnifferApp` — scanning, treemap rendering, breadcrumb nav, filesystem watching, file operations |
| `src/scanner.rs` | `FileNode` tree, parallel recursive scan with Rayon, `ScanProgress` for live UI updates |
| `src/treemap.rs` | Squarified treemap layout algorithm |
| `src/start_screen.rs` | Folder picker, `FolderTree` with lazy-loading, cloud storage detection |
| `src/theme.rs` | `ColorScheme` enum (6 themes), `ThemeColors` with hash-based node coloring |
| `src/settings.rs` | Settings window (color scheme picker) |
| `src/context_menu.rs` | Right-click context menu (open, reveal, copy path, rename, trash, open terminal) |
| `src/file_ops.rs` | Platform-specific commands (`open`, `xdg-open`, `cmd /C start`, etc.) |
| `src/icon.rs` | App icon generation |

### Key patterns

- **Platform conditionals** — `file_ops.rs` and `context_menu.rs` use `cfg!(target_os = ...)` for macOS/Windows/Linux behavior
- **Deferred actions** — Context menu actions are collected into a `RefCell<Option<DeferredAction>>` and executed after the UI pass to avoid borrowing conflicts
- **Live filesystem watching** — `notify` crate watches the current directory; create/remove/modify events update the in-memory tree without rescanning
- **Lazy tree loading** — The start screen folder tree loads children on first expand, not upfront
- **Cloud storage detection** — `is_cloud_storage_dir()` checks path patterns and folder names for known providers (iCloud, Google Drive, OneDrive, Dropbox, MEGA, Box, pCloud, etc.)

### Dependencies

| Crate | Purpose |
|-------|---------|
| eframe 0.31 | GUI framework (egui) |
| rayon 1 | Parallel directory scanning |
| rfd 0.15 | Native file dialogs |
| trash 5 | Cross-platform trash/recycle bin |
| notify 8 | Filesystem watching |

## Build

```bash
cargo run --release       # Build and run
cargo test                # Run tests
./scripts/build-macos.sh  # macOS .app bundle
```

Linux requires system dependencies:
```bash
sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libgtk-3-dev
```

## CI/CD

`.github/workflows/ci.yml` builds on all three platforms (Linux x86_64, Windows x86_64, macOS ARM64), runs tests, and creates a GitHub Release with zip artifacts on push to main.

## AI-assisted development

This project is developed with the assistance of [Claude Code](https://docs.anthropic.com/en/docs/claude-code). AI-generated commits are co-authored with `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>`.

### Guidelines

- Always review generated code before merging
- Verify tests pass on all target platforms (Windows, Linux, macOS)
- Keep the human in the loop for architectural decisions

## Roadmap

### Now
- Stabilize cross-platform builds and release packaging
- Bug fixes for filesystem watcher edge cases

### Next
- Scan progress bar (percentage-based)
- Persistent settings (remember last color scheme, window size)
- Keyboard navigation for the treemap
- Search / filter files by name or extension

### Future
- Duplicate file detection
- Disk usage trends over time
- Export reports (CSV / HTML)
- Custom color scheme editor
