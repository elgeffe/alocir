<p align="center">
  <img src="assets/logo.svg" alt="Alocir" width="128" height="128">
</p>

<h1 align="center">Alocir</h1>

<p align="center">
  A cross-platform disk space visualizer built with Rust.<br>
  Inspired by <a href="https://www.uderzo.it/main_products/space_sniffer/">SpaceSniffer</a>.
</p>

<p align="center">
  <a href="https://github.com/elgeffe/alocir/actions/workflows/ci.yml">
    <img src="https://github.com/elgeffe/alocir/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
</p>

---

## Features

- **Interactive treemap** — Visualize disk usage with a squarified treemap layout
- **Parallel scanning** — Fast directory scanning powered by Rayon
- **Folder exclusion** — Deselect any folder in the pre-scan tree view to skip it
- **Drill-down navigation** — Click directories to explore, breadcrumb trail to navigate back
- **File operations** — Cut, copy, paste, rename, and move to trash via right-click context menu
- **6 color schemes** — Dark Mode, SNES, Mac OS X, Windows 98, Windows XP, Matrix
- **Cross-platform** — Runs on Windows, Linux, and macOS

## Installation

### Download a release

Pre-built binaries are available on the [Releases](https://github.com/elgeffe/alocir/releases) page:

| Platform | Artifact |
|----------|----------|
| Windows (x86_64) | `alocir-windows-x86_64.zip` |
| Linux (x86_64) | `alocir-linux-x86_64.zip` |
| macOS (Apple Silicon) | `alocir-macos-arm64.zip` |

### Build from source

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

#### Build and run

```bash
cargo run --release
```

The compiled binary will be at `target/release/alocir` (or `alocir.exe` on Windows).

#### Run tests

```bash
cargo test
```

## Usage

1. Launch the app — the **start screen** appears with a folder picker and color scheme selector
2. Select a directory to scan
3. Review the **folder tree** — cloud storage folders are auto-unchecked. Toggle any folder to include or skip it
4. Click **Start Scan**
5. The treemap fills the window once scanning completes
6. **Click** a directory block to drill down into it
7. Use the **breadcrumb bar** at the top to navigate back
8. **Right-click** any item for file operations (open, copy, rename, trash, etc.)
9. Click the **gear icon** to change color schemes

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
- Export reports (CSV / HTML)
- Custom color scheme editor

## Dependencies

| Crate | Purpose |
|-------|---------|
| [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) | GUI framework (egui) |
| [rayon](https://github.com/rayon-rs/rayon) | Parallel iteration |
| [rfd](https://github.com/PolyMeilex/rfd) | Native file dialogs |
| [trash](https://github.com/Byron/trash-rs) | Cross-platform trash/recycle bin |

## License

Apache-2.0
