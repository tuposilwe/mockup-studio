use crate::assets::{AssetCache, FontManager};
use crate::model::{
    Background, DeviceFrameLayer, ImageLayer, Layer, LayerKind, PaintLayer, Project, ShadowStyle,
    ShapeLayer, TextAlign, TextLayer,
};
use ab_glyph::{Font, ScaleFont};
use image::{GenericImage, Rgba, RgbaImage};

/// Renders the full project to an RGBA image at native canvas resolution.
pub fn render_project(project: &Project, assets: &mut AssetCache, fonts: &FontManager) -> RgbaImage {
    let w = project.canvas_width;
    let h = project.canvas_height;
    let mut canvas = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));

    paint_background(&mut canvas, &project.background, assets);

    for layer in &project.layers {
        if !layer.visible {
            continue;
        }
        if let Some(buf) = render_layer(layer, assets, fonts) {
            let (bw, bh) = buf.dimensions();
            let center_x = layer.transform.x + layer.transform.width / 2.0;
            let center_y = layer.transform.y + layer.transform.height / 2.0;
            let dst_x = (center_x - bw as f32 / 2.0).round() as i32;
            let dst_y = (center_y - bh as f32 / 2.0).round() as i32;
            alpha_blit(&mut canvas, &buf, dst_x, dst_y);
        }
    }

    canvas
}

fn paint_background(canvas: &mut RgbaImage, bg: &Background, assets: &mut AssetCache) {
    let (w, h) = canvas.dimensions();
    match bg {
        Background::Color(c) => {
            let px = Rgba(*c);
            for p in canvas.pixels_mut() {
                *p = px;
            }
        }
        Background::Gradient { from, to, angle_deg } => {
            let theta = angle_deg.to_radians();
            let (dx, dy) = (theta.cos(), theta.sin());
            for y in 0..h {
                for x in 0..w {
                    let t = ((x as f32 / w as f32) * dx + (y as f32 / h as f32) * dy + 1.0) / 2.0;
                    let t = t.clamp(0.0, 1.0);
                    let px = lerp_rgba(*from, *to, t);
                    canvas.put_pixel(x, y, Rgba(px));
                }
            }
        }
        Background::Image { path } => {
            if let Some(img) = assets.get_or_load(path) {
                let resized = image::imageops::resize(
                    img.as_ref(),
                    w,
                    h,
                    image::imageops::FilterType::Lanczos3,
                );
                canvas.copy_from(&resized, 0, 0).ok();
            }
        }
    }
}

fn lerp_rgba(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    let mut out = [0u8; 4];
    for i in 0..4 {
        out[i] = (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8;
    }
    out
}

/// Builds the (unrotated-in-final-space) content buffer for one layer, including
/// corner-radius clipping and drop shadow, then applies mirror/rotation/opacity.
fn render_layer(layer: &Layer, assets: &mut AssetCache, fonts: &FontManager) -> Option<RgbaImage> {
    let t = &layer.transform;
    let w = t.width.max(1.0).round() as u32;
    let h = t.height.max(1.0).round() as u32;

    let content = match &layer.kind {
        LayerKind::Image(img_layer) => draw_image_content(img_layer, w, h, t.corner_radius, assets)?,
        LayerKind::Text(text_layer) => draw_text_content(text_layer, w, h, fonts),
        LayerKind::DeviceFrame(frame) => draw_device_frame_content(frame, w, h, assets),
        LayerKind::Shape(shape) => draw_shape_content(shape, w, h, t.corner_radius),
        LayerKind::Paint(paint) => draw_paint_content(paint, w, h),
    };

    let pad = shadow_padding(&t.shadow);
    let pw = w + pad * 2;
    let ph = h + pad * 2;

    let mut padded = RgbaImage::from_pixel(pw, ph, Rgba([0, 0, 0, 0]));
    if t.shadow.enabled {
        draw_shadow(&mut padded, w, h, pad, t.corner_radius, &t.shadow);
    }
    alpha_blit(&mut padded, &content, pad as i32, pad as i32);

    let mut buf = padded;
    if t.mirror_h {
        buf = image::imageops::flip_horizontal(&buf);
    }
    if t.mirror_v {
        buf = image::imageops::flip_vertical(&buf);
    }

    if t.rotation_deg.abs() > 0.01 {
        let (bw, bh) = buf.dimensions();
        let diag = ((bw as f32).hypot(bh as f32)).ceil() as u32 + 2;
        let mut square = RgbaImage::from_pixel(diag, diag, Rgba([0, 0, 0, 0]));
        let ox = (diag - bw) / 2;
        let oy = (diag - bh) / 2;
        alpha_blit(&mut square, &buf, ox as i32, oy as i32);
        buf = rotate_image(&square, t.rotation_deg.to_radians());
    }

    if t.opacity < 99.999 {
        let factor = (t.opacity / 100.0).clamp(0.0, 1.0);
        for p in buf.pixels_mut() {
            p[3] = (p[3] as f32 * factor).round() as u8;
        }
    }

    Some(buf)
}

fn shadow_padding(shadow: &ShadowStyle) -> u32 {
    if !shadow.enabled {
        return 0;
    }
    ((shadow.blur * 2.0) + shadow.offset_x.abs().max(shadow.offset_y.abs()) + 8.0).ceil() as u32
}

fn draw_shadow(buf: &mut RgbaImage, w: u32, h: u32, pad: u32, radius: f32, shadow: &ShadowStyle) {
    let (bw, bh) = buf.dimensions();
    let mut alpha = vec![0f32; (bw * bh) as usize];
    let ox = pad as i32 + shadow.offset_x.round() as i32;
    let oy = pad as i32 + shadow.offset_y.round() as i32;
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let px = ox + x;
            let py = oy + y;
            if px < 0 || py < 0 || px >= bw as i32 || py >= bh as i32 {
                continue;
            }
            let cov = rounded_rect_coverage(x as f32, y as f32, w as f32, h as f32, radius);
            alpha[(py as u32 * bw + px as u32) as usize] = cov;
        }
    }

    let sigma = (shadow.blur / 3.0).max(0.5);
    box_blur(&mut alpha, bw, bh, sigma);

    let [r, g, b, a] = shadow.color;
    for y in 0..bh {
        for x in 0..bw {
            let cov = alpha[(y * bw + x) as usize];
            if cov <= 0.001 {
                continue;
            }
            let out_a = (cov * (a as f32 / 255.0) * 255.0).round().clamp(0.0, 255.0) as u8;
            let existing = buf.get_pixel(x, y);
            let blended = blend_pixel(*existing, Rgba([r, g, b, out_a]));
            buf.put_pixel(x, y, blended);
        }
    }
}

/// Three-pass box blur approximates a Gaussian blur cheaply.
fn box_blur(data: &mut [f32], w: u32, h: u32, sigma: f32) {
    if sigma <= 0.01 {
        return;
    }
    let radius = ((sigma * 3.0).round() as i32).max(1);
    for _ in 0..3 {
        box_blur_pass_horizontal(data, w, h, radius);
        box_blur_pass_vertical(data, w, h, radius);
    }
}

fn box_blur_pass_horizontal(data: &mut [f32], w: u32, h: u32, radius: i32) {
    let w = w as i32;
    let h = h as i32;
    let mut out = vec![0f32; (w * h) as usize];
    let window = (2 * radius + 1) as f32;
    for y in 0..h {
        let mut acc = 0f32;
        for x in -radius..=radius {
            acc += sample(data, w, h, x, y);
        }
        for x in 0..w {
            out[(y * w + x) as usize] = acc / window;
            acc -= sample(data, w, h, x - radius, y);
            acc += sample(data, w, h, x + radius + 1, y);
        }
    }
    data.copy_from_slice(&out);
}

fn box_blur_pass_vertical(data: &mut [f32], w: u32, h: u32, radius: i32) {
    let w = w as i32;
    let h = h as i32;
    let mut out = vec![0f32; (w * h) as usize];
    let window = (2 * radius + 1) as f32;
    for x in 0..w {
        let mut acc = 0f32;
        for y in -radius..=radius {
            acc += sample(data, w, h, x, y);
        }
        for y in 0..h {
            out[(y * w + x) as usize] = acc / window;
            acc -= sample(data, w, h, x, y - radius);
            acc += sample(data, w, h, x, y + radius + 1);
        }
    }
    data.copy_from_slice(&out);
}

fn sample(data: &[f32], w: i32, h: i32, x: i32, y: i32) -> f32 {
    if x < 0 || y < 0 || x >= w || y >= h {
        0.0
    } else {
        data[(y * w + x) as usize]
    }
}

/// Signed-distance-based coverage for a rounded rectangle, antialiased over ~1px.
fn rounded_rect_coverage(x: f32, y: f32, w: f32, h: f32, radius: f32) -> f32 {
    let r = radius.min(w / 2.0).min(h / 2.0).max(0.0);
    let cx = w / 2.0;
    let cy = h / 2.0;
    let px = (x + 0.5 - cx).abs();
    let py = (y + 0.5 - cy).abs();
    let bx = cx - r;
    let by = cy - r;
    let qx = px - bx;
    let qy = py - by;
    let outside = (qx.max(0.0)).hypot(qy.max(0.0));
    let inside = qx.max(qy).min(0.0);
    let d = outside + inside - r;
    (0.5 - d).clamp(0.0, 1.0)
}

fn draw_image_content(
    layer: &ImageLayer,
    w: u32,
    h: u32,
    radius: f32,
    assets: &mut AssetCache,
) -> Option<RgbaImage> {
    let cached = assets.get_or_resize(&layer.path, layer.crop, w, h)?;
    let mut resized = (*cached).clone();

    if radius > 0.01 {
        for y in 0..h {
            for x in 0..w {
                let cov = rounded_rect_coverage(x as f32, y as f32, w as f32, h as f32, radius);
                if cov < 0.999 {
                    let p = resized.get_pixel_mut(x, y);
                    p[3] = (p[3] as f32 * cov).round() as u8;
                }
            }
        }
    }

    Some(resized)
}

/// Scales the paint layer's own raw pixel buffer to fit the layer's current
/// on-canvas size (only actually resamples if the layer's box has been
/// resized away from the buffer's native resolution).
fn draw_paint_content(layer: &PaintLayer, w: u32, h: u32) -> RgbaImage {
    let Some(buf) = RgbaImage::from_raw(layer.width, layer.height, layer.pixels.clone()) else {
        return RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
    };
    if layer.width == w && layer.height == h {
        buf
    } else {
        image::imageops::resize(&buf, w, h, image::imageops::FilterType::Triangle)
    }
}

fn draw_text_content(layer: &TextLayer, w: u32, h: u32, fonts: &FontManager) -> RgbaImage {
    let mut buf = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
    let Some(font) = fonts.font_for_weight(layer.weight) else {
        return buf;
    };
    let scale = ab_glyph::PxScale::from(layer.font_size);
    let sfont = font.as_scaled(scale);
    let line_height_px = layer.font_size * layer.line_height;

    let lines = wrap_text(&layer.content, w as f32, layer.font_size, layer.letter_spacing, font, scale);
    let total_height = line_height_px * lines.len() as f32;
    let mut cursor_y = (h as f32 - total_height) / 2.0;

    let [r, g, b, a] = layer.color;

    for line in &lines {
        let line_width = measure_line(line, layer.letter_spacing, font, scale);
        let mut cursor_x = match layer.align {
            TextAlign::Left => 0.0,
            TextAlign::Center => (w as f32 - line_width) / 2.0,
            TextAlign::Right => w as f32 - line_width,
        };
        let baseline_y = cursor_y + sfont.ascent();

        for ch in line.chars() {
            let glyph_id = font.glyph_id(ch);
            let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(cursor_x, baseline_y));
            let advance = sfont.h_advance(glyph_id);
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                outlined.draw(|gx, gy, coverage| {
                    if coverage <= 0.0 {
                        return;
                    }
                    let px = bounds.min.x as i32 + gx as i32;
                    let py = bounds.min.y as i32 + gy as i32;
                    if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                        return;
                    }
                    let out_a = (coverage * a as f32).round().clamp(0.0, 255.0) as u8;
                    let existing = *buf.get_pixel(px as u32, py as u32);
                    let blended = blend_pixel(existing, Rgba([r, g, b, out_a]));
                    buf.put_pixel(px as u32, py as u32, blended);
                });
            }
            cursor_x += advance + layer.letter_spacing;
        }
        cursor_y += line_height_px;
    }

    buf
}

fn measure_line(line: &str, letter_spacing: f32, font: &ab_glyph::FontVec, scale: ab_glyph::PxScale) -> f32 {
    let sfont = font.as_scaled(scale);
    let mut width = 0.0;
    for ch in line.chars() {
        width += sfont.h_advance(font.glyph_id(ch)) + letter_spacing;
    }
    if !line.is_empty() {
        width -= letter_spacing;
    }
    width.max(0.0)
}

fn wrap_text(
    text: &str,
    max_width: f32,
    _font_size: f32,
    letter_spacing: f32,
    font: &ab_glyph::FontVec,
    scale: ab_glyph::PxScale,
) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        for word in paragraph.split(' ') {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            let width = measure_line(&candidate, letter_spacing, font, scale);
            if width > max_width && !current.is_empty() {
                lines.push(current);
                current = word.to_string();
            } else {
                current = candidate;
            }
        }
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn draw_shape_content(shape: &ShapeLayer, w: u32, h: u32, radius: f32) -> RgbaImage {
    let mut buf = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
    let [r, g, b, a] = shape.fill_color;
    for y in 0..h {
        for x in 0..w {
            let cov = rounded_rect_coverage(x as f32, y as f32, w as f32, h as f32, radius);
            if cov > 0.0 {
                let out_a = (cov * a as f32).round().clamp(0.0, 255.0) as u8;
                buf.put_pixel(x, y, Rgba([r, g, b, out_a]));
            }
        }
    }
    buf
}

fn shade(c: [u8; 3], factor: f32) -> [u8; 3] {
    [
        (c[0] as f32 * factor).round().clamp(0.0, 255.0) as u8,
        (c[1] as f32 * factor).round().clamp(0.0, 255.0) as u8,
        (c[2] as f32 * factor).round().clamp(0.0, 255.0) as u8,
    ]
}

fn draw_device_frame_content(frame: &DeviceFrameLayer, w: u32, h: u32, assets: &mut AssetCache) -> RgbaImage {
    if let Some(path) = &frame.custom_image_path {
        if let Some(img) = assets.get_or_resize(path, None, w, h) {
            return (*img).clone();
        }
    }

    let mut buf = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
    let fw = w as f32;
    let fh = h as f32;
    let base = frame.style.rgb();
    let outer_radius = fw * 0.15;
    let bezel = (fw * 0.032).max(6.0);
    let inner_radius = (outer_radius - bezel).max(0.0);
    let rim_w = (fw * 0.006).max(1.5);
    let inner_edge_w = (fw * 0.006).max(1.5);

    for y in 0..h {
        for x in 0..w {
            let (xf, yf) = (x as f32, y as f32);
            let outer_cov = rounded_rect_coverage(xf, yf, fw, fh, outer_radius);
            if outer_cov <= 0.0 {
                continue;
            }
            let inner_cov =
                rounded_rect_coverage(xf - bezel, yf - bezel, fw - bezel * 2.0, fh - bezel * 2.0, inner_radius);
            let coverage = (outer_cov - inner_cov).clamp(0.0, 1.0);
            if coverage <= 0.0 {
                continue;
            }

            // Diagonal light so the bezel reads as a curved metal edge rather than a flat fill.
            let nx = (xf / fw - 0.5) * 2.0;
            let ny = (yf / fh - 0.5) * 2.0;
            let light = (-0.55 * nx - 0.8 * ny).clamp(-1.0, 1.0);
            let mut c = shade(base, 1.0 + light * 0.16);

            // Bright chamfer highlight right at the outer edge.
            let outer_shrunk = rounded_rect_coverage(
                xf - rim_w,
                yf - rim_w,
                fw - rim_w * 2.0,
                fh - rim_w * 2.0,
                (outer_radius - rim_w).max(0.0),
            );
            let rim_ring = (outer_cov - outer_shrunk).clamp(0.0, 1.0);
            if rim_ring > 0.0 {
                let hl_factor = if light > 0.0 { 1.55 } else { 1.25 };
                let hl = shade(base, hl_factor);
                let t = rim_ring * 0.8;
                c = [
                    (c[0] as f32 * (1.0 - t) + hl[0] as f32 * t) as u8,
                    (c[1] as f32 * (1.0 - t) + hl[1] as f32 * t) as u8,
                    (c[2] as f32 * (1.0 - t) + hl[2] as f32 * t) as u8,
                ];
            }

            // Dark seam right where the bezel meets the screen, for depth.
            let inner_expanded = rounded_rect_coverage(
                xf - (bezel - inner_edge_w),
                yf - (bezel - inner_edge_w),
                fw - (bezel - inner_edge_w) * 2.0,
                fh - (bezel - inner_edge_w) * 2.0,
                inner_radius + inner_edge_w,
            );
            let inner_ring = (inner_expanded - inner_cov).clamp(0.0, 1.0);
            if inner_ring > 0.0 {
                let t = inner_ring * 0.55;
                c = [
                    (c[0] as f32 * (1.0 - t)) as u8,
                    (c[1] as f32 * (1.0 - t)) as u8,
                    (c[2] as f32 * (1.0 - t)) as u8,
                ];
            }

            let a = (coverage * 255.0).round() as u8;
            buf.put_pixel(x, y, Rgba([c[0], c[1], c[2], a]));
        }
    }

    // Notch: a fully-rounded pill flush with the top edge, with a small camera dot.
    let notch_w = fw * 0.30;
    let notch_h = fh * 0.026;
    let notch_x = (fw - notch_w) / 2.0;
    let notch_radius = notch_h / 2.0;
    let notch_color = [12u8, 12, 15];
    for y in 0..(notch_h + bezel) as u32 {
        for x in 0..notch_w as u32 {
            let cov = rounded_rect_coverage(x as f32, y as f32 - bezel, notch_w, notch_h + bezel, notch_radius);
            if cov > 0.0 {
                let px = notch_x as u32 + x;
                let py = y;
                if px < w && py < h {
                    let a = (cov * 255.0).round() as u8;
                    buf.put_pixel(px, py, Rgba([notch_color[0], notch_color[1], notch_color[2], a]));
                }
            }
        }
    }
    // Camera dot inside the notch, offset toward one end, with a small glass highlight.
    let dot_d = notch_h * 0.62;
    let dot_cx = notch_x + notch_w * 0.78;
    let dot_cy = bezel + notch_h * 0.5;
    let dot_r2 = (dot_d * 0.5) * (dot_d * 0.5);
    let x0 = (dot_cx - dot_d).max(0.0) as u32;
    let x1 = (dot_cx + dot_d).min(fw) as u32;
    let y0 = (dot_cy - dot_d).max(0.0) as u32;
    let y1 = (dot_cy + dot_d).min(fh) as u32;
    for y in y0..y1 {
        for x in x0..x1 {
            let dx = x as f32 + 0.5 - dot_cx;
            let dy = y as f32 + 0.5 - dot_cy;
            let d2 = dx * dx + dy * dy;
            if d2 <= dot_r2 {
                let lens = shade([28, 34, 46], 1.0 + (-0.6 * dx - 0.6 * dy) / dot_d * 0.9);
                buf.put_pixel(x, y, Rgba([lens[0], lens[1], lens[2], 255]));
            }
        }
    }

    // Side buttons: glossy rounded pills with a top-lit metal gradient.
    let button_w = (bezel * 0.55).max(3.0);
    let inset = bezel * 0.28;
    let button_specs: [(f32, f32, bool); 4] = [
        (0.15, 0.05, false),
        (0.26, 0.09, false),
        (0.40, 0.09, false),
        (0.20, 0.11, true),
    ];
    for (rel_y, rel_h, side_right) in button_specs {
        let by = fh * rel_y;
        let bh = fh * rel_h;
        let bx = if side_right { fw - inset - button_w } else { inset };
        let radius = button_w * 0.5;
        for y in by.max(0.0) as u32..(by + bh).min(fh) as u32 {
            for x in bx.max(0.0) as u32..(bx + button_w).min(fw) as u32 {
                let lx = x as f32 - bx;
                let ly = y as f32 - by;
                let cov = rounded_rect_coverage(lx, ly, button_w, bh, radius);
                if cov > 0.0 {
                    let existing = *buf.get_pixel(x, y);
                    if existing[3] > 0 {
                        let t = (ly / bh).clamp(0.0, 1.0);
                        let button_color = shade([120, 122, 128], 1.35 - t * 0.5);
                        buf.put_pixel(x, y, Rgba([button_color[0], button_color[1], button_color[2], 255]));
                    }
                }
            }
        }
    }

    buf
}

fn blend_pixel(dst: Rgba<u8>, src: Rgba<u8>) -> Rgba<u8> {
    let sa = src[3] as f32 / 255.0;
    if sa <= 0.0 {
        return dst;
    }
    let da = dst[3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return Rgba([0, 0, 0, 0]);
    }
    let mut out = [0u8; 4];
    for i in 0..3 {
        let s = src[i] as f32;
        let d = dst[i] as f32;
        let v = (s * sa + d * da * (1.0 - sa)) / out_a;
        out[i] = v.round().clamp(0.0, 255.0) as u8;
    }
    out[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgba(out)
}

fn alpha_blit(dst: &mut RgbaImage, src: &RgbaImage, dst_x: i32, dst_y: i32) {
    let (dw, dh) = dst.dimensions();
    let (sw, sh) = src.dimensions();
    for y in 0..sh as i32 {
        let py = dst_y + y;
        if py < 0 || py >= dh as i32 {
            continue;
        }
        for x in 0..sw as i32 {
            let px = dst_x + x;
            if px < 0 || px >= dw as i32 {
                continue;
            }
            let s = *src.get_pixel(x as u32, y as u32);
            if s[3] == 0 {
                continue;
            }
            let d = *dst.get_pixel(px as u32, py as u32);
            dst.put_pixel(px as u32, py as u32, blend_pixel(d, s));
        }
    }
}

/// Rotates an RGBA image by `theta` radians about its center via inverse
/// mapping + bilinear sampling. Output has the same dimensions as input, so
/// callers must pre-pad enough that rotated corners aren't clipped.
fn rotate_image(src: &RgbaImage, theta: f32) -> RgbaImage {
    let (w, h) = src.dimensions();
    let mut out = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let cos_t = theta.cos();
    let sin_t = theta.sin();

    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let sx = dx * cos_t + dy * sin_t + cx;
            let sy = -dx * sin_t + dy * cos_t + cy;
            if let Some(px) = bilinear_sample(src, sx, sy) {
                out.put_pixel(x, y, px);
            }
        }
    }

    out
}

fn bilinear_sample(img: &RgbaImage, x: f32, y: f32) -> Option<Rgba<u8>> {
    let (w, h) = img.dimensions();
    if x < 0.0 || y < 0.0 || x >= w as f32 - 1.0 || y >= h as f32 - 1.0 {
        if x < -0.5 || y < -0.5 || x >= w as f32 + 0.5 || y >= h as f32 + 0.5 {
            return None;
        }
        let xi = x.round().clamp(0.0, (w - 1) as f32) as u32;
        let yi = y.round().clamp(0.0, (h - 1) as f32) as u32;
        return Some(*img.get_pixel(xi, yi));
    }
    let x0 = x.floor();
    let y0 = y.floor();
    let fx = x - x0;
    let fy = y - y0;
    let x0 = x0 as u32;
    let y0 = y0 as u32;
    let p00 = img.get_pixel(x0, y0);
    let p10 = img.get_pixel(x0 + 1, y0);
    let p01 = img.get_pixel(x0, y0 + 1);
    let p11 = img.get_pixel(x0 + 1, y0 + 1);
    let mut out = [0u8; 4];
    for i in 0..4 {
        let top = p00[i] as f32 * (1.0 - fx) + p10[i] as f32 * fx;
        let bot = p01[i] as f32 * (1.0 - fx) + p11[i] as f32 * fx;
        out[i] = (top * (1.0 - fy) + bot * fy).round().clamp(0.0, 255.0) as u8;
    }
    Some(Rgba(out))
}
