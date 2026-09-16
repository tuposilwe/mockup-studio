use anyhow::{Context, Result};
use image::RgbaImage;
use std::collections::HashMap;
use std::sync::Arc;

type CropKey = Option<(u32, u32, u32, u32)>;
type ResizeKey = (String, u32, u32, CropKey);

const RESIZE_CACHE_LIMIT: usize = 64;

#[derive(Default)]
pub struct AssetCache {
    images: HashMap<String, Arc<RgbaImage>>,
    resized: HashMap<ResizeKey, Arc<RgbaImage>>,
}

impl AssetCache {
    pub fn get_or_load(&mut self, path: &str) -> Option<Arc<RgbaImage>> {
        if let Some(img) = self.images.get(path) {
            return Some(img.clone());
        }
        match image::open(path) {
            Ok(img) => {
                let rgba = Arc::new(img.to_rgba8());
                self.images.insert(path.to_string(), rgba.clone());
                Some(rgba)
            }
            Err(_) => None,
        }
    }

    /// Cropped + Lanczos3-resized image, cached by (path, target size, crop).
    /// A move-drag holds size constant across many frames, so this turns what
    /// would be a full re-resize every frame into a cache hit after the first.
    pub fn get_or_resize(
        &mut self,
        path: &str,
        crop: Option<crate::model::CropRect>,
        target_w: u32,
        target_h: u32,
    ) -> Option<Arc<RgbaImage>> {
        let crop_key = crop.map(|c| (c.x.to_bits(), c.y.to_bits(), c.w.to_bits(), c.h.to_bits()));
        let key: ResizeKey = (path.to_string(), target_w, target_h, crop_key);
        if let Some(img) = self.resized.get(&key) {
            return Some(img.clone());
        }

        let src = self.get_or_load(path)?;
        let (sw, sh) = src.dimensions();
        let cropped = if let Some(c) = crop {
            let cx = (c.x * sw as f32).round() as u32;
            let cy = (c.y * sh as f32).round() as u32;
            let cw = (c.w * sw as f32).round().max(1.0) as u32;
            let ch = (c.h * sh as f32).round().max(1.0) as u32;
            let cw = cw.min(sw.saturating_sub(cx).max(1));
            let ch = ch.min(sh.saturating_sub(cy).max(1));
            image::imageops::crop_imm(src.as_ref(), cx, cy, cw, ch).to_image()
        } else {
            (*src).clone()
        };
        let resized = Arc::new(image::imageops::resize(
            &cropped,
            target_w,
            target_h,
            image::imageops::FilterType::Lanczos3,
        ));

        if self.resized.len() >= RESIZE_CACHE_LIMIT {
            self.resized.clear();
        }
        self.resized.insert(key, resized.clone());
        Some(resized)
    }

    #[allow(dead_code)]
    pub fn invalidate(&mut self, path: &str) {
        self.images.remove(path);
        self.resized.retain(|k, _| k.0 != path);
    }
}

/// Static-weight Roboto variants (Apache 2.0, Google) embedded into the
/// binary so text rendering works identically on macOS, Windows, and Linux
/// without depending on OS-specific system font paths being present.
pub struct FontManager {
    pub light: Option<ab_glyph::FontVec>,
    pub regular: Option<ab_glyph::FontVec>,
    pub bold: Option<ab_glyph::FontVec>,
    pub black: Option<ab_glyph::FontVec>,
}

fn load_font(bytes: &'static [u8]) -> Option<ab_glyph::FontVec> {
    ab_glyph::FontVec::try_from_vec(bytes.to_vec()).ok()
}

impl FontManager {
    pub fn load() -> Self {
        FontManager {
            light: load_font(include_bytes!("../assets/fonts/Roboto-Light.ttf")),
            regular: load_font(include_bytes!("../assets/fonts/Roboto-Regular.ttf")),
            bold: load_font(include_bytes!("../assets/fonts/Roboto-Bold.ttf")),
            black: load_font(include_bytes!("../assets/fonts/Roboto-Black.ttf")),
        }
    }

    pub fn font_for_weight(&self, weight: u16) -> Option<&ab_glyph::FontVec> {
        let candidate = if weight <= 300 {
            self.light.as_ref()
        } else if weight <= 500 {
            self.regular.as_ref()
        } else if weight <= 700 {
            self.bold.as_ref()
        } else {
            self.black.as_ref()
        };
        candidate.or(self.regular.as_ref())
    }
}

#[allow(dead_code)]
pub fn load_rgba(path: &str) -> Result<RgbaImage> {
    Ok(image::open(path)
        .with_context(|| format!("failed to open image {path}"))?
        .to_rgba8())
}
