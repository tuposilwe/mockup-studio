// Hide the console window on Windows for release builds (a native GUI app
// shouldn't flash a terminal behind it); debug builds keep it so the
// --export-test/--export-templates dev flags still print their output.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod assets;
mod bundled;
mod history;
mod model;
mod render;
mod templates;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--export-test") {
        let out_path = args.get(pos + 1).cloned().unwrap_or_else(|| "test_export.png".to_string());
        let mut assets = assets::AssetCache::default();
        let fonts = assets::FontManager::load();
        let project = model::Project::default();
        let img = render::render_project(&project, &mut assets, &fonts);
        img.save(&out_path).expect("failed to save test export");
        println!("Wrote {out_path}");
        return Ok(());
    }

    if let Some(pos) = args.iter().position(|a| a == "--export-templates") {
        let out_dir = args.get(pos + 1).cloned().unwrap_or_else(|| ".".to_string());
        let mut assets = assets::AssetCache::default();
        let fonts = assets::FontManager::load();
        let photos = bundled::BundledPhotos::extract_all();
        for entry in templates::template_gallery() {
            let project = (entry.build)(&photos);
            let img = render::render_project(&project, &mut assets, &fonts);
            let path = format!("{out_dir}/{}.png", entry.name.replace(' ', "_"));
            img.save(&path).expect("failed to save template export");
            println!("Wrote {path}");
        }
        return Ok(());
    }

    // Without an explicit icon, eframe falls back to its own bundled default
    // ("e" logo) and overwrites the Dock/taskbar icon with it a few frames
    // after launch — which briefly replaces the correct icon set via the
    // .app bundle's Info.plist. Providing ours here keeps it consistent.
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/app_icon.png"))
        .expect("bundled app icon should decode");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 1000.0])
            .with_title("Mockup Studio")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Mockup Studio",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new()))),
    )
}
