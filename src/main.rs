// Hide the console window on Windows for release builds (a native GUI app
// shouldn't flash a terminal behind it); debug builds keep it so the
// --export-test/--export-templates dev flags still print their output.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod assets;
mod bundled;
mod history;
mod model;
mod network;
mod render;
mod templates;

/// Self-test for the LAN collaboration code path: hosts and joins on
/// loopback within the same process, sends a project referencing a real
/// bundled image from the "client" side, and checks the "host" side
/// receives it with the image bytes correctly transplanted to a local path.
/// Not part of the normal app UI flow — a dev-only regression check.
fn net_test() -> eframe::Result<()> {
    use std::time::{Duration, Instant};

    let port = 17878u16;
    let photos = bundled::BundledPhotos::extract_all();
    let source_bytes = std::fs::read(&photos.coastline).expect("bundled coastline photo should exist");

    let host = network::host(port).expect("host() should bind");
    std::thread::sleep(Duration::from_millis(150));
    let join_rx = network::join(&format!("127.0.0.1:{port}"));
    let client = join_rx
        .recv_timeout(Duration::from_secs(6))
        .expect("join() should resolve within 6s")
        .expect("join() should connect");

    // Drain the PeerConnected events both sides get so they don't confuse
    // the assertions below.
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if let Ok(network::NetEvent::PeerConnected) = host.events.try_recv() {
            break;
        }
    }

    let mut project = model::Project::default();
    let layer_id = project.alloc_id();
    project.layers.push(model::Layer {
        id: layer_id,
        name: "Test Photo".to_string(),
        visible: true,
        kind: model::LayerKind::Image(model::ImageLayer {
            path: photos.coastline.clone(),
            crop: None,
            linked_frame_id: None,
            quad: None,
        }),
        transform: model::Transform {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            rotation_deg: 0.0,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: 0.0,
            shadow: model::ShadowStyle::default(),
        },
    });

    client.send_project(&project);

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut received: Option<model::Project> = None;
    while Instant::now() < deadline && received.is_none() {
        if let Ok(network::NetEvent::ProjectReceived(p)) = host.events.try_recv() {
            received = Some(p);
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    let received = received.expect("host should have received the client's project within 3s");
    let image_layer = received
        .layers
        .iter()
        .find_map(|l| match &l.kind {
            model::LayerKind::Image(img) => Some(img),
            _ => None,
        })
        .expect("received project should contain the image layer");

    assert_ne!(
        image_layer.path, photos.coastline,
        "received path should have been remapped to a local cache path, not left as the sender's own path"
    );
    let received_bytes = std::fs::read(&image_layer.path).expect("remapped path should be a real readable file");
    assert_eq!(received_bytes, source_bytes, "transferred image bytes should match the original exactly");

    println!("net-test PASS: hosted, joined, and synced a project with a remapped image asset");
    println!("  original path: {}", photos.coastline);
    println!("  remapped path: {}", image_layer.path);

    // Cursor sharing: the client's pointer position should reach the host
    // as its own lightweight message, distinct from a full project sync.
    client.send_cursor(123.5, 456.75, "Test User");
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut cursor = None;
    while Instant::now() < deadline && cursor.is_none() {
        if let Ok(network::NetEvent::CursorReceived { x, y, name }) = host.events.try_recv() {
            cursor = Some((x, y, name));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let (x, y, name) = cursor.expect("host should have received the client's cursor position within 3s");
    assert_eq!((x, y, name.as_str()), (123.5, 456.75, "Test User"));
    println!("net-test PASS: cursor position synced ({x}, {y}, {name})");

    // Leaving a hosted session ("drop the NetworkState") must actually
    // release the port, not just detach the app's UI from a background
    // listener thread that keeps running forever — that was the exact bug
    // reported ("if i host session and leave if i join again port is being
    // used"). Drop both sides, then require a fresh host() on the same
    // port to succeed immediately.
    drop(client);
    drop(host);
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut rehosted = None;
    while Instant::now() < deadline && rehosted.is_none() {
        if let Ok(net) = network::host(port) {
            rehosted = Some(net);
        } else {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    rehosted.expect("re-hosting on the same port right after leaving should succeed, not fail with 'address already in use'");
    println!("net-test PASS: leaving a hosted session frees the port for immediate re-hosting");

    Ok(())
}

/// Self-test for the Paint layer render path: builds a project with a blue
/// Shape background and a Paint layer on top, stamps a red brush stroke into
/// it (via the same `stamp_line` the live brush tool calls), renders the
/// whole project, and checks the stroke composited correctly over the shape
/// underneath it while leaving the rest of the shape untouched.
fn paint_test(out_path: &str) {
    let mut assets = assets::AssetCache::default();
    let fonts = assets::FontManager::load();

    let mut project = model::Project::default();
    project.layers.clear();
    project.canvas_width = 400;
    project.canvas_height = 400;
    project.background = model::Background::Color([255, 255, 255, 255]);

    let shape_id = project.alloc_id();
    project.layers.push(model::Layer {
        id: shape_id,
        name: "Shape".to_string(),
        visible: true,
        kind: model::LayerKind::Shape(model::ShapeLayer { fill_color: [40, 80, 220, 255] }),
        transform: model::Transform {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 400.0,
            rotation_deg: 0.0,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: 0.0,
            shadow: model::ShadowStyle::default(),
        },
    });

    let paint_id = project.alloc_id();
    let mut paint_layer = model::PaintLayer::new_transparent(400, 400);
    app::stamp_line(&mut paint_layer, (100.0, 200.0), (300.0, 200.0), 30.0, [220, 30, 30, 255]);
    project.layers.push(model::Layer {
        id: paint_id,
        name: "Paint".to_string(),
        visible: true,
        kind: model::LayerKind::Paint(paint_layer),
        transform: model::Transform {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 400.0,
            rotation_deg: 0.0,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: 0.0,
            shadow: model::ShadowStyle::default(),
        },
    });

    let img = render::render_project(&project, &mut assets, &fonts);
    img.save(out_path).expect("failed to save paint test export");

    let stroke_pixel = img.get_pixel(200, 200);
    assert_eq!(stroke_pixel.0, [220, 30, 30, 255], "brush stroke should be opaque red at its center");
    let untouched_pixel = img.get_pixel(20, 20);
    assert_eq!(untouched_pixel.0, [40, 80, 220, 255], "shape outside the stroke should show through unpainted");

    println!("paint-test PASS: brush stroke composited correctly over the layer beneath it");
    println!("  wrote {out_path}");
}

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

    if args.iter().any(|a| a == "--net-test") {
        return net_test();
    }

    if let Some(pos) = args.iter().position(|a| a == "--paint-test") {
        let out_path = args.get(pos + 1).cloned().unwrap_or_else(|| "paint_test.png".to_string());
        paint_test(&out_path);
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
