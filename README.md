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
- **Cloud storage detection** — iCloud, Google Drive, OneDrive, Dropbox, MEGA, Box, pCloud, and more are auto-detected and can be skipped before scanning
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

#### Windows

1. Download and extract `alocir-windows-x86_64.zip`
2. Run `alocir.exe`

#### Linux

1. Download and extract `alocir-linux-x86_64.zip`
2. Make the binary executable and run it:
   ```bash
   chmod +x alocir
   ./alocir
   ```

#### macOS

1. Download and extract `alocir-macos-arm64.zip`
2. Move `Alocir.app` to `/Applications/`
3. On first launch, macOS may show an "unidentified developer" warning — right-click the app and choose **Open** to bypass it

### Build from source

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- **Linux only** — install system dependencies:
  ```bash
  sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libgtk-3-dev
  ```

#### Build and run

```bash
cargo run --release
```

The compiled binary will be at `target/release/alocir` (or `alocir.exe` on Windows).

#### macOS app bundle

To build a proper `.app` bundle with an icon:

```bash
./scripts/build-macos.sh
```

This creates `target/release/Alocir.app` with ad-hoc code signing. To install:

```bash
cp -r target/release/Alocir.app /Applications/
```

> **Note:** To eliminate the "unidentified developer" warning, the app must be signed with an [Apple Developer ID](https://developer.apple.com/developer-id/) and notarized:
>
> ```bash
> ./scripts/build-macos.sh --sign "Developer ID Application: Your Name (TEAMID)"
> xcrun notarytool submit target/release/Alocir.app --apple-id ... --team-id ... --password ...
> xcrun stapler staple target/release/Alocir.app
> ```

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
- Disk usage trends over time
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
