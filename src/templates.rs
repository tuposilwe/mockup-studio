use crate::bundled::BundledPhotos;
use crate::model::*;

pub struct TemplateDef {
    pub name: &'static str,
    pub build: fn(&BundledPhotos) -> Project,
}

pub fn template_gallery() -> Vec<TemplateDef> {
    vec![
        TemplateDef { name: "Blank Canvas", build: blank },
        TemplateDef { name: "Minimal Light", build: minimal_light },
        TemplateDef { name: "Gradient Sunset", build: gradient_sunset },
        TemplateDef { name: "Bold Dark", build: bold_dark },
        TemplateDef { name: "Pastel Sky", build: pastel_sky },
        TemplateDef { name: "Panorama Duo", build: panorama_duo },
        TemplateDef { name: "Feature Badge", build: feature_badge },
        TemplateDef { name: "Clean Mono", build: clean_mono },
    ]
}

fn base_frame(id: u64, x: f32, y: f32, w: f32, h: f32, style: FrameColor, photos: &BundledPhotos) -> Layer {
    Layer {
        id,
        name: "Device Frame".to_string(),
        visible: true,
        kind: LayerKind::DeviceFrame(DeviceFrameLayer {
            style,
            custom_image_path: Some(photos.iphone_frame.clone()),
        }),
        transform: Transform {
            x,
            y,
            width: w,
            height: h,
            rotation_deg: 0.0,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: 0.0,
            shadow: ShadowStyle::default(),
        },
    }
}

fn base_text(
    id: u64,
    content: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    size: f32,
    weight: u16,
    color: [u8; 4],
    align: TextAlign,
) -> Layer {
    Layer {
        id,
        name: "Text".to_string(),
        visible: true,
        kind: LayerKind::Text(TextLayer {
            content: content.to_string(),
            font_size: size,
            weight,
            line_height: 1.2,
            letter_spacing: 0.0,
            color,
            align,
        }),
        transform: Transform {
            x,
            y,
            width: w,
            height: h,
            rotation_deg: 0.0,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: 0.0,
            shadow: ShadowStyle::default(),
        },
    }
}

/// Computes a normalized center-crop rect so a source photo fills a target
/// box without stretching distortion.
pub fn aspect_crop(path: &str, target_w: f32, target_h: f32) -> Option<CropRect> {
    let (sw, sh) = image::image_dimensions(path).ok()?;
    let (sw, sh) = (sw as f32, sh as f32);
    let src_ratio = sw / sh;
    let target_ratio = target_w / target_h;
    if src_ratio > target_ratio {
        let new_w = sh * target_ratio;
        let x = (sw - new_w) / 2.0 / sw;
        Some(CropRect { x, y: 0.0, w: new_w / sw, h: 1.0 })
    } else {
        let new_h = sw / target_ratio;
        let y = (sh - new_h) / 2.0 / sh;
        Some(CropRect { x: 0.0, y, w: 1.0, h: new_h / sh })
    }
}

/// Margins (as a fraction of the frame's own width/height) that keep a screen
/// photo inside the frame's real screen area rather than stretched to the
/// frame's full outer bounds. Measured directly from the bundled real iPhone
/// frame PNG's alpha channel (screen rect at 121,118 to 1845,3843 in a
/// 1965x3953 source image, scanned row/column by row/column, corner curves
/// excluded), so the photo fits the actual screen edges precisely rather
/// than leaving an oversized gap.
pub const SCREEN_MARGIN_X_FRAC: f32 = 0.061;
pub const SCREEN_MARGIN_Y_FRAC: f32 = 0.029;

/// The photo's (x, y, width, height) inset inside `frame`'s screen area.
pub fn inset_screen_rect(frame: &Transform) -> (f32, f32, f32, f32) {
    let margin_x = frame.width * SCREEN_MARGIN_X_FRAC;
    let margin_y = frame.height * SCREEN_MARGIN_Y_FRAC;
    (
        frame.x + margin_x,
        frame.y + margin_y,
        frame.width - margin_x * 2.0,
        frame.height - margin_y * 2.0,
    )
}

/// A real photo sized and clipped to sit inside a device frame's screen area
/// (with a safety margin so the frame stays visibly bigger than the photo),
/// so it reads as actual on-screen app content once the frame's bezel/notch
/// is painted on top of it.
pub fn screen_photo_layer(id: u64, frame_id: u64, frame: &Transform, path: &str) -> Layer {
    let (x, y, w, h) = inset_screen_rect(frame);
    let crop = aspect_crop(path, w, h);
    Layer {
        id,
        name: "Screen Photo".to_string(),
        visible: true,
        kind: LayerKind::Image(ImageLayer {
            path: path.to_string(),
            crop,
            linked_frame_id: Some(frame_id),
        }),
        transform: Transform {
            x,
            y,
            width: w,
            height: h,
            rotation_deg: frame.rotation_deg,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: w * 0.12,
            shadow: ShadowStyle::default(),
        },
    }
}

/// A soft decorative accent circle, useful as a background graphic behind a frame.
fn accent_circle(id: u64, cx: f32, cy: f32, diameter: f32, color: [u8; 4]) -> Layer {
    Layer {
        id,
        name: "Accent".to_string(),
        visible: true,
        kind: LayerKind::Shape(ShapeLayer { fill_color: color }),
        transform: Transform {
            x: cx - diameter / 2.0,
            y: cy - diameter / 2.0,
            width: diameter,
            height: diameter,
            rotation_deg: 0.0,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: diameter / 2.0,
            shadow: ShadowStyle::default(),
        },
    }
}

fn blank(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Color([245, 246, 248, 255]),
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };
    let id = p.alloc_id();
    p.layers.push(base_frame(
        id,
        CANVAS_WIDTH as f32 * 0.5 - 560.0,
        CANVAS_HEIGHT as f32 * 0.5 - 1140.0,
        1120.0,
        2280.0,
        FrameColor::SpaceGray,
        photos,
    ));
    p
}

fn minimal_light(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Color([245, 246, 248, 255]),
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let accent_id = p.alloc_id();
    p.layers.push(accent_circle(
        accent_id,
        CANVAS_WIDTH as f32 * 0.5,
        1500.0,
        1500.0,
        [210, 220, 235, 140],
    ));

    let frame_x = CANVAS_WIDTH as f32 * 0.5 - 560.0;
    let frame_y = CANVAS_HEIGHT as f32 * 0.5 - 1140.0;
    let frame_t = Transform {
        x: frame_x,
        y: frame_y,
        width: 1120.0,
        height: 2280.0,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };

    let frame_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, &photos.beach_horizon));
    p.layers.push(base_frame(frame_id, frame_x, frame_y, 1120.0, 2280.0, FrameColor::SpaceGray, photos));

    let text_id = p.alloc_id();
    p.layers.push(base_text(
        text_id,
        "Your headline here",
        80.0,
        300.0,
        CANVAS_WIDTH as f32 - 160.0,
        200.0,
        90.0,
        700,
        [30, 30, 34, 255],
        TextAlign::Center,
    ));
    p
}

fn gradient_sunset(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Gradient {
            from: [255, 154, 158, 255],
            to: [250, 208, 196, 255],
            angle_deg: 90.0,
        },
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let frame_t = Transform {
        x: CANVAS_WIDTH as f32 * 0.5 - 560.0,
        y: CANVAS_HEIGHT as f32 * 0.5 - 900.0,
        width: 1120.0,
        height: 2280.0,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };

    let frame_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, &photos.sunset_pier));
    p.layers.push(base_frame(
        frame_id,
        frame_t.x,
        frame_t.y,
        frame_t.width,
        frame_t.height,
        FrameColor::Silver,
        photos,
    ));

    let text_id = p.alloc_id();
    p.layers.push(base_text(
        text_id,
        "Plan your perfect day",
        100.0,
        2350.0,
        CANVAS_WIDTH as f32 - 200.0,
        260.0,
        84.0,
        800,
        [60, 30, 30, 255],
        TextAlign::Center,
    ));
    p
}

fn bold_dark(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Color([18, 20, 28, 255]),
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let glow_id = p.alloc_id();
    p.layers.push(accent_circle(
        glow_id,
        CANVAS_WIDTH as f32 * 0.5,
        1450.0,
        1700.0,
        [246, 220, 185, 35],
    ));

    let frame_t = Transform {
        x: CANVAS_WIDTH as f32 * 0.5 - 560.0,
        y: CANVAS_HEIGHT as f32 * 0.5 - 1140.0,
        width: 1120.0,
        height: 2280.0,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };

    let frame_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, &photos.city_night));
    p.layers.push(base_frame(
        frame_id,
        frame_t.x,
        frame_t.y,
        frame_t.width,
        frame_t.height,
        FrameColor::Gold,
        photos,
    ));

    let head_id = p.alloc_id();
    p.layers.push(base_text(
        head_id,
        "Track everything",
        90.0,
        260.0,
        CANVAS_WIDTH as f32 - 180.0,
        160.0,
        96.0,
        900,
        [255, 255, 255, 255],
        TextAlign::Center,
    ));
    let sub_id = p.alloc_id();
    p.layers.push(base_text(
        sub_id,
        "in one simple dashboard",
        90.0,
        430.0,
        CANVAS_WIDTH as f32 - 180.0,
        120.0,
        52.0,
        400,
        [190, 190, 200, 255],
        TextAlign::Center,
    ));
    p
}

fn pastel_sky(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Gradient {
            from: [206, 227, 255, 255],
            to: [255, 255, 255, 255],
            angle_deg: 90.0,
        },
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let frame_t = Transform {
        x: CANVAS_WIDTH as f32 * 0.5 - 560.0,
        y: CANVAS_HEIGHT as f32 * 0.5 - 750.0,
        width: 1120.0,
        height: 2280.0,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };

    let frame_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, &photos.calm_lake));
    p.layers.push(base_frame(
        frame_id,
        frame_t.x,
        frame_t.y,
        frame_t.width,
        frame_t.height,
        FrameColor::PacificBlue,
        photos,
    ));

    let head_id = p.alloc_id();
    p.layers.push(base_text(
        head_id,
        "Breathe easy",
        90.0,
        260.0,
        CANVAS_WIDTH as f32 - 180.0,
        150.0,
        88.0,
        600,
        [40, 60, 90, 255],
        TextAlign::Center,
    ));
    let sub_id = p.alloc_id();
    p.layers.push(base_text(
        sub_id,
        "Guided meditation, every day",
        90.0,
        420.0,
        CANVAS_WIDTH as f32 - 180.0,
        110.0,
        46.0,
        400,
        [70, 90, 120, 255],
        TextAlign::Center,
    ));
    p
}

fn panorama_duo(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Gradient {
            from: [255, 236, 210, 255],
            to: [252, 182, 159, 255],
            angle_deg: 45.0,
        },
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let left_t = Transform {
        x: 60.0,
        y: 900.0,
        width: 780.0,
        height: 1590.0,
        rotation_deg: -6.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };
    let right_t = Transform {
        x: 470.0,
        y: 940.0,
        width: 780.0,
        height: 1590.0,
        rotation_deg: 6.0,
        ..left_t.clone()
    };

    let left_id = p.alloc_id();
    let photo_left_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_left_id, left_id, &left_t, &photos.dunes));
    let mut left = base_frame(left_id, left_t.x, left_t.y, left_t.width, left_t.height, FrameColor::SpaceGray, photos);
    left.transform.rotation_deg = -6.0;
    p.layers.push(left);

    let right_id = p.alloc_id();
    let photo_right_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_right_id, right_id, &right_t, &photos.coastline));
    let mut right = base_frame(right_id, right_t.x, right_t.y, right_t.width, right_t.height, FrameColor::SpaceGray, photos);
    right.transform.rotation_deg = 6.0;
    p.layers.push(right);

    let head_id = p.alloc_id();
    p.layers.push(base_text(
        head_id,
        "Two views, one app",
        90.0,
        260.0,
        CANVAS_WIDTH as f32 - 180.0,
        150.0,
        84.0,
        800,
        [60, 40, 30, 255],
        TextAlign::Center,
    ));
    p
}

fn feature_badge(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Color([243, 244, 250, 255]),
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let frame_t = Transform {
        x: CANVAS_WIDTH as f32 * 0.5 - 560.0,
        y: CANVAS_HEIGHT as f32 * 0.5 - 850.0,
        width: 1120.0,
        height: 2280.0,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };

    let frame_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, &photos.gold_interior));
    p.layers.push(base_frame(
        frame_id,
        frame_t.x,
        frame_t.y,
        frame_t.width,
        frame_t.height,
        FrameColor::SpaceGray,
        photos,
    ));

    let pill_id = p.alloc_id();
    p.layers.push(Layer {
        id: pill_id,
        name: "Badge Pill".to_string(),
        visible: true,
        kind: LayerKind::Shape(ShapeLayer { fill_color: [255, 90, 95, 255] }),
        transform: Transform {
            x: CANVAS_WIDTH as f32 * 0.5 - 110.0,
            y: 180.0,
            width: 220.0,
            height: 80.0,
            rotation_deg: 0.0,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: 40.0,
            shadow: ShadowStyle::default(),
        },
    });

    let badge_id = p.alloc_id();
    p.layers.push(Layer {
        id: badge_id,
        name: "Badge".to_string(),
        visible: true,
        kind: LayerKind::Text(TextLayer {
            content: "NEW".to_string(),
            font_size: 40.0,
            weight: 800,
            line_height: 1.0,
            letter_spacing: 4.0,
            color: [255, 255, 255, 255],
            align: TextAlign::Center,
        }),
        transform: Transform {
            x: CANVAS_WIDTH as f32 * 0.5 - 110.0,
            y: 180.0,
            width: 220.0,
            height: 80.0,
            rotation_deg: 0.0,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: 0.0,
            shadow: ShadowStyle::default(),
        },
    });

    let head_id = p.alloc_id();
    p.layers.push(base_text(
        head_id,
        "Now with AI insights",
        90.0,
        320.0,
        CANVAS_WIDTH as f32 - 180.0,
        160.0,
        76.0,
        700,
        [30, 30, 34, 255],
        TextAlign::Center,
    ));
    p
}

fn clean_mono(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Color([255, 255, 255, 255]),
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let frame_t = Transform {
        x: CANVAS_WIDTH as f32 * 0.5 - 560.0,
        y: CANVAS_HEIGHT as f32 * 0.5 - 1140.0,
        width: 1120.0,
        height: 2280.0,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };

    let frame_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, &photos.mono_pier));
    p.layers.push(base_frame(
        frame_id,
        frame_t.x,
        frame_t.y,
        frame_t.width,
        frame_t.height,
        FrameColor::Silver,
        photos,
    ));

    let divider_id = p.alloc_id();
    p.layers.push(Layer {
        id: divider_id,
        name: "Divider".to_string(),
        visible: true,
        kind: LayerKind::Shape(ShapeLayer { fill_color: [20, 20, 20, 255] }),
        transform: Transform {
            x: CANVAS_WIDTH as f32 * 0.5 - 40.0,
            y: 2330.0,
            width: 80.0,
            height: 6.0,
            rotation_deg: 0.0,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: 3.0,
            shadow: ShadowStyle::default(),
        },
    });

    let head_id = p.alloc_id();
    p.layers.push(base_text(
        head_id,
        "Simplicity, delivered.",
        90.0,
        2380.0,
        CANVAS_WIDTH as f32 - 180.0,
        160.0,
        72.0,
        500,
        [20, 20, 20, 255],
        TextAlign::Center,
    ));
    p
}
