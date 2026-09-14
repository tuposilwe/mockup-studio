# Mockup Studio

A native macOS desktop app for designing App Store screenshot mockups — device frames, backgrounds, text, and shapes on a resizable canvas, exported to PNG at 1242×2688. Built in Rust with [egui](https://github.com/emilk/egui).

## Features

- **Template gallery** — 8 prepared mockups (gradient/solid backgrounds, panorama-style dual frames, badge accents) with live thumbnails; click one to start editing
- **Real device frame** — a genuine transparent-screen iPhone frame image by default, with screen photos precisely inset to the actual screen area (not stretched to the frame's outer edges); swap in your own frame PNG per layer, or fall back to a built-in procedural frame with selectable colors
- **Layers** — device frame, image, text, and shape (solid rounded-rect), each with position/size/rotation/opacity, corner radius, drop shadow, and mirroring
- **Canvas interaction** — drag to move, corner handles to resize from any corner, a rotate handle (hold Shift to snap to 15°), Figma-style alignment snap guides, double-click to edit text inline or swap a photo
- **Text** — multi-line, font size/weight (100–900)/line-height/letter-spacing/color/alignment, rendered via system fonts
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

## Bundled assets

`assets/bundled/` contains photography used as default screen content in templates (royalty-free, via [picsum.photos](https://picsum.photos)/Unsplash, free-to-use license) and one real device frame PNG. These are embedded into the binary at compile time via `include_bytes!`, so the app is self-contained.

## Project layout

- `src/model.rs` — project/layer data model
- `src/render.rs` — software rasterizer (rounded-rect SDFs, text layout, image compositing) shared by the live preview and PNG export
- `src/app.rs` — egui UI: canvas interaction, layers/properties panels, gallery screen
- `src/templates.rs` — built-in template definitions
- `src/assets.rs` — image/font loading with a resize cache
- `src/bundled.rs` — extracts embedded assets to a cache dir on first run
- `src/history.rs` — undo/redo stack
