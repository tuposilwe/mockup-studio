# Mockup Studio

A native desktop app (macOS + Windows) for designing App Store / Play Store screenshot mockups — device frames, backgrounds, text, and shapes on a resizable canvas, exported to PNG. Built in Rust with [egui](https://github.com/emilk/egui).

## Features

- **Template gallery** — 12 prepared mockups (iPhone, MacBook, two Android styles, iPad; gradient/solid backgrounds, panorama-style dual frames, badge accents) with live thumbnails; click one to start editing
- **Real device frames** — genuine transparent-screen mockup images (iPhone, MacBook, Android x2, iPad) by default, with screen photos precisely inset to each frame's actual measured screen area (not stretched to the frame's outer edges); swap in your own frame PNG per layer, or fall back to a built-in procedural frame with selectable colors
- **Layers** — device frame, image, text, and shape (solid rounded-rect), each with position/size/rotation/opacity, corner radius, drop shadow, and mirroring
- **Canvas interaction** — drag to move, corner handles to resize from any corner, a rotate handle (hold Shift to snap to 15°), Figma-style alignment snap guides, double-click to edit text inline or swap a photo, full-screen preview, resizable/collapsible side panels
- **Text** — multi-line, font size/weight (100–900)/line-height/letter-spacing/color/alignment, rendered with an embedded Roboto (Apache 2.0) font family — no OS font dependency
- **Non-AI image tools** — crop, Lanczos3 resize/upscale, aspect-correct center-cropping
- **Undo/redo** (Ctrl/Cmd+Z, Ctrl/Cmd+Shift+Z), save/load projects as JSON, PNG export

## Running

Always use a **release build** — a debug build renders slowly enough (1–3s/frame) to feel unresponsive while dragging.

```bash
cargo run --release
```

or, once built:

```bash
./target/release/mockup_studio
```

## Building installers

### macOS (.app + .dmg)

```bash
./packaging/build_macos.sh
```

Produces `packaging/dist/Mockup Studio.app` (ad-hoc signed) and `packaging/dist/MockupStudio-macOS.dmg` (drag-to-Applications installer).

### Windows (.exe installer)

Cross-compiled from macOS. One-time setup:

```bash
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64 makensis
```

Then:

```bash
./packaging/build_windows.sh
```

Produces `packaging/MockupStudio-Setup.exe` (NSIS installer — installs to Program Files, adds Start Menu/Desktop shortcuts and an uninstaller). The app icon is embedded into the `.exe` at build time via `build.rs` (`winresource`).

> Built and verified to compile/package on macOS. The Windows binary and installer haven't been run on an actual Windows machine — there's no way to do that from this environment. If you hit a Windows-specific issue, it's likely either a missing runtime DLL (the GNU target statically links the C++ runtime, so this should be rare) or a rendering quirk in the `glow`/OpenGL backend on a particular GPU driver.

## Bundled assets

`assets/bundled/` contains photography used as default screen content in templates (royalty-free, via [picsum.photos](https://picsum.photos)/Unsplash, free-to-use license) and real device frame PNGs (iPhone, MacBook, two Android styles, iPad — used under a Vecteezy Pro license where applicable). `assets/fonts/` contains Roboto (Apache 2.0, Google). All of this is embedded into the binary at compile time via `include_bytes!`, so the app is fully self-contained on both platforms.

## Project layout

- `src/model.rs` — project/layer data model
- `src/render.rs` — software rasterizer (rounded-rect SDFs, text layout, image compositing) shared by the live preview and PNG export
- `src/app.rs` — egui UI: canvas interaction, layers/properties panels, gallery screen
- `src/templates.rs` — built-in template definitions
- `src/assets.rs` — image/font loading with a resize cache
- `src/bundled.rs` — extracts embedded assets to a cache dir on first run
- `src/history.rs` — undo/redo stack
- `build.rs` — embeds the app icon into the Windows `.exe` (no-op on other platforms)
- `packaging/` — icons, `build_macos.sh`, `build_windows.sh`, `installer.nsi`
