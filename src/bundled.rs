use std::io::Write;

/// Real royalty-free photos (Unsplash, via picsum.photos, free-to-use license)
/// bundled into the binary so templates always find them regardless of where
/// the app is run from. Extracted once to a cache dir on first use.
pub struct BundledPhotos {
    pub beach_horizon: String,
    pub sunset_pier: String,
    pub city_night: String,
    pub calm_lake: String,
    pub dunes: String,
    pub coastline: String,
    pub gold_interior: String,
    pub mono_pier: String,
    /// A real (user-supplied) transparent-screen iPhone frame PNG, used as
    /// the default device frame image instead of the procedural silhouette.
    pub iphone_frame: String,
    /// A real (user-supplied) transparent-screen MacBook Pro frame PNG.
    pub macbook_frame: String,
    /// A real (user-supplied) Android phone outline frame; its solid white
    /// screen fill was color-keyed to transparent so photos can show through.
    pub android_frame: String,
    /// A second real (user-supplied) Android frame, waterdrop-notch style,
    /// with a genuinely transparent screen area already baked in.
    pub android_waterdrop_frame: String,
    /// A real (user-supplied) tablet/iPad frame with a genuinely transparent
    /// screen area.
    pub ipad_frame: String,
    /// Uzasasa wordmark logo (rasterized from a user-supplied SVG), used in
    /// the Product Listing template.
    pub uzasasa_logo: String,
    /// Official "Get it on Google Play" badge artwork, used in the App
    /// Download template.
    pub play_store_badge: String,
    /// Official "Download on the App Store" badge artwork, used in the App
    /// Download template.
    pub app_store_badge: String,
    /// A real (user-supplied) photo of a phone held in a hand, with a
    /// genuinely transparent screen hole (traced via flood-fill on the
    /// source alpha channel, not color-keyed), used as a more lifelike
    /// alternative to the flat procedural phone frame.
    pub hand_phone_frame: String,
}

fn extract(dir: &std::path::Path, name: &str, bytes: &[u8]) -> String {
    let path = dir.join(name);
    let needs_write = std::fs::metadata(&path).ok().map(|m| m.len() as usize) != Some(bytes.len());
    if needs_write {
        if let Ok(mut f) = std::fs::File::create(&path) {
            let _ = f.write_all(bytes);
        }
    }
    path.to_string_lossy().to_string()
}

impl BundledPhotos {
    pub fn extract_all() -> Self {
        let dir = std::env::temp_dir().join("mockup_studio_bundled_assets");
        let _ = std::fs::create_dir_all(&dir);

        BundledPhotos {
            beach_horizon: extract(&dir, "beach_horizon.jpg", include_bytes!("../assets/bundled/beach_horizon.jpg")),
            sunset_pier: extract(&dir, "sunset_pier.jpg", include_bytes!("../assets/bundled/sunset_pier.jpg")),
            city_night: extract(&dir, "city_night.jpg", include_bytes!("../assets/bundled/city_night.jpg")),
            calm_lake: extract(&dir, "calm_lake.jpg", include_bytes!("../assets/bundled/calm_lake.jpg")),
            dunes: extract(&dir, "dunes.jpg", include_bytes!("../assets/bundled/dunes.jpg")),
            coastline: extract(&dir, "coastline.jpg", include_bytes!("../assets/bundled/coastline.jpg")),
            gold_interior: extract(&dir, "gold_interior.jpg", include_bytes!("../assets/bundled/gold_interior.jpg")),
            mono_pier: extract(&dir, "mono_pier.jpg", include_bytes!("../assets/bundled/mono_pier.jpg")),
            iphone_frame: extract(&dir, "iphone_frame.png", include_bytes!("../assets/bundled/iphone_frame.png")),
            macbook_frame: extract(&dir, "macbook_frame.png", include_bytes!("../assets/bundled/macbook_frame.png")),
            android_frame: extract(&dir, "android_frame.png", include_bytes!("../assets/bundled/android_frame.png")),
            android_waterdrop_frame: extract(
                &dir,
                "android_waterdrop_frame.png",
                include_bytes!("../assets/bundled/android_waterdrop_frame.png"),
            ),
            ipad_frame: extract(&dir, "ipad_frame.png", include_bytes!("../assets/bundled/ipad_frame.png")),
            uzasasa_logo: extract(&dir, "uzasasa_logo.png", include_bytes!("../assets/bundled/uzasasa_logo.png")),
            play_store_badge: extract(&dir, "play_store_badge.png", include_bytes!("../assets/bundled/play_store_badge.png")),
            app_store_badge: extract(&dir, "app_store_badge.png", include_bytes!("../assets/bundled/app_store_badge.png")),
            hand_phone_frame: extract(&dir, "hand_phone_frame.png", include_bytes!("../assets/bundled/hand_phone_frame.png")),
        }
    }
}
