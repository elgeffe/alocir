# Alocir

A cross-platform disk space visualizer built with Rust. Inspired by [SpaceSniffer](https://www.uderzo.it/main_products/space_sniffer/).

[![CI](https://github.com/elgeffe/alocir/actions/workflows/ci.yml/badge.svg)](https://github.com/elgeffe/alocir/actions/workflows/ci.yml)

## Features

- **Interactive treemap** - Visualize disk usage with a squarified treemap layout
- **Parallel scanning** - Fast directory scanning powered by Rayon
- **Drill-down navigation** - Click directories to explore, breadcrumb trail to navigate back
- **File operations** - Cut, copy, paste, rename, and move to trash via right-click context menu
- **Multiple color schemes** - Dark Mode, Retro, Mac OS X, Windows 98, Windows XP, Matrix
- **Cross-platform** - Runs on Windows, Linux, and macOS

## Building from source

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- On Linux, install the required system dependencies:
  ```bash
  sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libgtk-3-dev
  ```

### Build and run

```bash
cargo run --release
```

### Run tests

```bash
cargo test
```

## Usage

1. Launch the application - a directory picker dialog will appear
2. Select a directory to scan
3. The treemap fills the window once scanning completes
4. **Click** a directory block to drill down into it
5. Use the **breadcrumb bar** at the top to navigate back
6. **Right-click** any item for file operations (open, copy, rename, trash, etc.)
7. Click the **gear icon** to change color schemes

## Dependencies

- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - GUI framework (egui)
- [rayon](https://github.com/rayon-rs/rayon) - Parallel iteration
- [rfd](https://github.com/PolyMeilex/rfd) - Native file dialogs
- [trash](https://github.com/Byron/trash-rs) - Cross-platform trash/recycle bin

## License

MIT
