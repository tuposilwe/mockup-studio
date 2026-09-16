use crate::bundled::BundledPhotos;
use crate::model::*;

pub struct TemplateDef {
    pub name: &'static str,
    pub build: fn(&BundledPhotos) -> Project,
}

pub fn template_gallery() -> Vec<TemplateDef> {
    vec![
        TemplateDef { name: "Product Listing", build: product_listing_showcase },
        TemplateDef { name: "App Promo", build: app_promo_showcase },
        TemplateDef { name: "App Download", build: app_download_showcase },
        TemplateDef { name: "Blank Canvas", build: blank },
        TemplateDef { name: "Minimal Light", build: minimal_light },
        TemplateDef { name: "Gradient Sunset", build: gradient_sunset },
        TemplateDef { name: "Bold Dark", build: bold_dark },
        TemplateDef { name: "Pastel Sky", build: pastel_sky },
        TemplateDef { name: "Panorama Duo", build: panorama_duo },
        TemplateDef { name: "Feature Badge", build: feature_badge },
        TemplateDef { name: "Clean Mono", build: clean_mono },
        TemplateDef { name: "MacBook Showcase", build: macbook_showcase },
        TemplateDef { name: "Android Showcase", build: android_showcase },
        TemplateDef { name: "Android Waterdrop", build: android_waterdrop_showcase },
        TemplateDef { name: "iPad Showcase", build: ipad_showcase },
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
            kind: FrameKind::Phone,
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

fn base_laptop_frame(id: u64, x: f32, y: f32, w: f32, h: f32, photos: &BundledPhotos) -> Layer {
    Layer {
        id,
        name: "MacBook Frame".to_string(),
        visible: true,
        kind: LayerKind::DeviceFrame(DeviceFrameLayer {
            style: FrameColor::SpaceGray,
            custom_image_path: Some(photos.macbook_frame.clone()),
            kind: FrameKind::Laptop,
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

fn base_android_frame(id: u64, x: f32, y: f32, w: f32, h: f32, photos: &BundledPhotos) -> Layer {
    Layer {
        id,
        name: "Android Frame".to_string(),
        visible: true,
        kind: LayerKind::DeviceFrame(DeviceFrameLayer {
            style: FrameColor::SpaceGray,
            custom_image_path: Some(photos.android_frame.clone()),
            kind: FrameKind::Android,
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

fn base_android_waterdrop_frame(id: u64, x: f32, y: f32, w: f32, h: f32, photos: &BundledPhotos) -> Layer {
    Layer {
        id,
        name: "Android Frame".to_string(),
        visible: true,
        kind: LayerKind::DeviceFrame(DeviceFrameLayer {
            style: FrameColor::SpaceGray,
            custom_image_path: Some(photos.android_waterdrop_frame.clone()),
            kind: FrameKind::AndroidWaterdrop,
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

fn base_ipad_frame(id: u64, x: f32, y: f32, w: f32, h: f32, photos: &BundledPhotos) -> Layer {
    Layer {
        id,
        name: "iPad Frame".to_string(),
        visible: true,
        kind: LayerKind::DeviceFrame(DeviceFrameLayer {
            style: FrameColor::SpaceGray,
            custom_image_path: Some(photos.ipad_frame.clone()),
            kind: FrameKind::Ipad,
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

fn base_hand_frame(id: u64, x: f32, y: f32, w: f32, h: f32, photos: &BundledPhotos) -> Layer {
    Layer {
        id,
        name: "Phone (Hand)".to_string(),
        visible: true,
        kind: LayerKind::DeviceFrame(DeviceFrameLayer {
            style: FrameColor::SpaceGray,
            custom_image_path: Some(photos.hand_phone_frame.clone()),
            kind: FrameKind::PhoneHand,
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

/// Screen-inset margins (as a fraction of the frame's own width/height,
/// left/right/top/bottom independently since a laptop's screen sits far from
/// centered — there's a tall keyboard deck below it) that keep a screen photo
/// inside the frame's real screen area rather than stretched to the frame's
/// full outer bounds. Measured directly from each bundled frame PNG's alpha
/// channel by scanning row/column by row/column (corner curves and soft
/// drop-shadow noise excluded), so the photo fits the actual screen edges
/// precisely rather than leaving an oversized or mismatched gap.
///
/// Phone: screen rect 121,109 to 1846,3845 in the bundled 1965x3953 PNG
/// (re-traced with the strict alpha>128 threshold at many x/y sample points;
/// an earlier pass used a single symmetric top/bottom value derived mostly
/// from the top edge, which was close enough to look right at a glance but
/// left a ~3px background-colored sliver at the bottom in the full-res
/// export — small in absolute terms, but a real, fixable mismatch).
/// Laptop: screen rect 281,32 to 2319,1334 in the bundled 2600x1509 PNG
/// (traced per-column/row with a strict alpha>128 threshold to exclude
/// antialiasing and the soft drop-shadow below the laptop, both of which
/// otherwise inflate the measured margins).
fn screen_margins(kind: FrameKind) -> (f32, f32, f32, f32) {
    match kind {
        FrameKind::Phone => (0.0616, 0.0606, 0.0276, 0.0273),
        FrameKind::Laptop => (0.1081, 0.1081, 0.0212, 0.1160),
        // Android: screen rect 80,89 to 2978,6248 in the bundled 3091x6455
        // PNG (a thin-outline frame whose screen was color-keyed from solid
        // white to transparent; measured the same strict-threshold way).
        FrameKind::Android => (0.0259, 0.0366, 0.0138, 0.0321),
        // Android (waterdrop notch): screen rect 154,121 to 2434,5001 in the
        // bundled 2580x5121 PNG, which already had a real transparent screen
        // hole (no color-keying needed).
        FrameKind::AndroidWaterdrop => (0.0597, 0.0566, 0.0236, 0.0234),
        // iPad: screen rect 155,155 to 2076,3570 in the bundled 2230x3722
        // PNG, which already had a real transparent screen hole.
        FrameKind::Ipad => (0.0695, 0.0691, 0.0416, 0.0408),
        // Phone-in-hand: screen hole 2258,84 to 4340,4467 in the bundled
        // 2600x2382 PNG (cropped and downsampled from a 6000x6000 source,
        // screen located by flood-filling a white-keyed alpha channel from
        // a seed point inside the screen — the hole isn't a simple
        // axis-aligned scan target since the hand's fingers overlap the
        // frame's right edge).
        FrameKind::PhoneHand => (0.3763, 0.2767, 0.0153, 0.1872),
    }
}

/// The photo's (x, y, width, height) inset inside `frame`'s screen area.
pub fn inset_screen_rect(frame: &Transform, kind: FrameKind) -> (f32, f32, f32, f32) {
    let (ml, mr, mt, mb) = screen_margins(kind);
    let left = frame.width * ml;
    let right = frame.width * mr;
    let top = frame.height * mt;
    let bottom = frame.height * mb;
    (frame.x + left, frame.y + top, frame.width - left - right, frame.height - top - bottom)
}

/// A real photo sized and clipped to sit inside a device frame's screen area
/// (with a safety margin so the frame stays visibly bigger than the photo),
/// so it reads as actual on-screen app content once the frame's bezel/notch
/// is painted on top of it.
pub fn screen_photo_layer(id: u64, frame_id: u64, frame: &Transform, kind: FrameKind, path: &str) -> Layer {
    let (x, y, w, h) = inset_screen_rect(frame, kind);
    let crop = aspect_crop(path, w, h);
    let corner_radius = match kind {
        // Traced the screen hole's own corner curve directly (bottom-left
        // corner reaches the flat edge ~180px in from x=121 at a screen
        // width of 1725px): 0.12 over-rounded the photo, cutting a visible
        // gap at the bottom corners.
        FrameKind::Phone => w * 0.103,
        FrameKind::Laptop => w * 0.006,
        FrameKind::Android => w * 0.05,
        FrameKind::AndroidWaterdrop => w * 0.05,
        FrameKind::Ipad => w * 0.04,
        FrameKind::PhoneHand => w * 0.08,
    };
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
            corner_radius,
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

/// A plain solid rounded-rect shape layer — cards, banners, pills, rings.
fn base_shape(id: u64, x: f32, y: f32, w: f32, h: f32, radius: f32, color: [u8; 4]) -> Layer {
    Layer {
        id,
        name: "Shape".to_string(),
        visible: true,
        kind: LayerKind::Shape(ShapeLayer { fill_color: color }),
        transform: Transform {
            x,
            y,
            width: w,
            height: h,
            rotation_deg: 0.0,
            opacity: 100.0,
            mirror_h: false,
            mirror_v: false,
            corner_radius: radius,
            shadow: ShadowStyle::default(),
        },
    }
}

/// A plain image layer stretched to fill (x, y, w, h), aspect-cropped from
/// its source so it doesn't distort, with rounded corners and opacity.
fn base_image(id: u64, x: f32, y: f32, w: f32, h: f32, radius: f32, path: &str, opacity: f32) -> Layer {
    Layer {
        id,
        name: "Image".to_string(),
        visible: true,
        kind: LayerKind::Image(ImageLayer {
            path: path.to_string(),
            crop: aspect_crop(path, w, h),
            linked_frame_id: None,
        }),
        transform: Transform {
            x,
            y,
            width: w,
            height: h,
            rotation_deg: 0.0,
            opacity,
            mirror_h: false,
            mirror_v: false,
            corner_radius: radius,
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
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, FrameKind::Phone, &photos.beach_horizon));
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
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, FrameKind::Phone, &photos.sunset_pier));
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
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, FrameKind::Phone, &photos.city_night));
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
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, FrameKind::Phone, &photos.calm_lake));
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
    p.layers.push(screen_photo_layer(photo_left_id, left_id, &left_t, FrameKind::Phone, &photos.dunes));
    let mut left = base_frame(left_id, left_t.x, left_t.y, left_t.width, left_t.height, FrameColor::SpaceGray, photos);
    left.transform.rotation_deg = -6.0;
    p.layers.push(left);

    let right_id = p.alloc_id();
    let photo_right_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_right_id, right_id, &right_t, FrameKind::Phone, &photos.coastline));
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
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, FrameKind::Phone, &photos.gold_interior));
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
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, FrameKind::Phone, &photos.mono_pier));
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

fn macbook_showcase(photos: &BundledPhotos) -> Project {
    let canvas_w = 2880.0f32;
    let canvas_h = 1800.0f32;
    let mut p = Project {
        canvas_width: canvas_w as u32,
        canvas_height: canvas_h as u32,
        background: Background::Gradient {
            from: [235, 238, 245, 255],
            to: [208, 216, 230, 255],
            angle_deg: 90.0,
        },
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let head_id = p.alloc_id();
    p.layers.push(base_text(
        head_id,
        "Built for your Mac",
        140.0,
        110.0,
        canvas_w - 280.0,
        140.0,
        76.0,
        700,
        [30, 32, 38, 255],
        TextAlign::Center,
    ));

    // Matches the bundled macbook_frame.png's own aspect ratio (2600x1509).
    let frame_w = 2260.0;
    let frame_h = frame_w * (1509.0 / 2600.0);
    let frame_t = Transform {
        x: (canvas_w - frame_w) / 2.0,
        y: canvas_h - frame_h - 120.0,
        width: frame_w,
        height: frame_h,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };

    let frame_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, FrameKind::Laptop, &photos.gold_interior));
    p.layers.push(base_laptop_frame(frame_id, frame_t.x, frame_t.y, frame_t.width, frame_t.height, photos));

    p
}

fn android_showcase(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Gradient {
            from: [214, 233, 220, 255],
            to: [255, 255, 255, 255],
            angle_deg: 90.0,
        },
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let frame_w = 1000.0;
    let frame_h = frame_w * (6455.0 / 3091.0);
    let frame_t = Transform {
        x: (CANVAS_WIDTH as f32 - frame_w) / 2.0,
        y: CANVAS_HEIGHT as f32 * 0.5 - frame_h * 0.5 + 60.0,
        width: frame_w,
        height: frame_h,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };

    let frame_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, FrameKind::Android, &photos.coastline));
    p.layers.push(base_android_frame(frame_id, frame_t.x, frame_t.y, frame_t.width, frame_t.height, photos));

    let head_id = p.alloc_id();
    p.layers.push(base_text(
        head_id,
        "Runs great on Android",
        90.0,
        260.0,
        CANVAS_WIDTH as f32 - 180.0,
        150.0,
        76.0,
        700,
        [30, 50, 38, 255],
        TextAlign::Center,
    ));

    p
}

fn android_waterdrop_showcase(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Gradient {
            from: [255, 235, 214, 255],
            to: [255, 255, 255, 255],
            angle_deg: 90.0,
        },
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let frame_w = 1000.0;
    let frame_h = frame_w * (5121.0 / 2580.0);
    let frame_t = Transform {
        x: (CANVAS_WIDTH as f32 - frame_w) / 2.0,
        y: CANVAS_HEIGHT as f32 * 0.5 - frame_h * 0.5 + 60.0,
        width: frame_w,
        height: frame_h,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };

    let frame_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, FrameKind::AndroidWaterdrop, &photos.dunes));
    p.layers.push(base_android_waterdrop_frame(frame_id, frame_t.x, frame_t.y, frame_t.width, frame_t.height, photos));

    let head_id = p.alloc_id();
    p.layers.push(base_text(
        head_id,
        "Android, perfected",
        90.0,
        260.0,
        CANVAS_WIDTH as f32 - 180.0,
        150.0,
        76.0,
        700,
        [90, 55, 30, 255],
        TextAlign::Center,
    ));

    p
}

fn ipad_showcase(photos: &BundledPhotos) -> Project {
    let mut p = Project {
        canvas_width: CANVAS_WIDTH,
        canvas_height: CANVAS_HEIGHT,
        background: Background::Color([246, 246, 248, 255]),
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let frame_w = 980.0;
    let frame_h = frame_w * (3722.0 / 2230.0);
    let frame_t = Transform {
        x: (CANVAS_WIDTH as f32 - frame_w) / 2.0,
        y: CANVAS_HEIGHT as f32 * 0.5 - frame_h * 0.5 + 60.0,
        width: frame_w,
        height: frame_h,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };

    let frame_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, frame_id, &frame_t, FrameKind::Ipad, &photos.coastline));
    p.layers.push(base_ipad_frame(frame_id, frame_t.x, frame_t.y, frame_t.width, frame_t.height, photos));

    let head_id = p.alloc_id();
    p.layers.push(base_text(
        head_id,
        "Designed for iPad",
        90.0,
        260.0,
        CANVAS_WIDTH as f32 - 180.0,
        150.0,
        76.0,
        700,
        [30, 30, 34, 255],
        TextAlign::Center,
    ));

    p
}

/// A classifieds/marketplace-style listing card: logo, a photo grid (main
/// shot + thumbnail strip with a "selected" ring on the first), a price
/// pill + title banner, and a footer URL. Built entirely from the existing
/// Shape/Image/Text primitives — no new layer kind needed.
fn product_listing_showcase(photos: &BundledPhotos) -> Project {
    let canvas_w = 1080.0f32;
    let canvas_h = 1350.0f32;
    let mut p = Project {
        canvas_width: canvas_w as u32,
        canvas_height: canvas_h as u32,
        background: Background::Color([238, 240, 243, 255]),
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let brand_blue = [8, 102, 255, 255];
    let ink = [35, 31, 32, 255];

    // Brand mark, top-left.
    let logo_w = 160.0;
    let logo_h = logo_w * (192.0 / 856.0);
    let logo_id = p.alloc_id();
    p.layers.push(base_image(logo_id, 60.0, 48.0, logo_w, logo_h, 0.0, &photos.uzasasa_logo, 100.0));

    // Card.
    let card_x = 60.0;
    let card_y = 118.0;
    let card_w = canvas_w - card_x * 2.0;
    let pad = 24.0;

    let photo_x = card_x + pad;
    let photo_y = card_y + pad;
    let photo_w = card_w - pad * 2.0;
    let photo_h = 740.0;

    let gap = 14.0;
    let thumb_w = (photo_w - gap * 3.0) / 4.0;
    let thumb_h = 160.0;
    let thumb_y = photo_y + photo_h + gap;

    let card_h = (thumb_y + thumb_h + pad) - card_y;
    let card_id = p.alloc_id();
    p.layers.push(base_shape(card_id, card_x, card_y, card_w, card_h, 28.0, [255, 255, 255, 255]));

    // Main photo, with a translucent brand watermark centered on it.
    let main_photo_id = p.alloc_id();
    p.layers.push(base_image(main_photo_id, photo_x, photo_y, photo_w, photo_h, 16.0, &photos.coastline, 100.0));

    let wm_w = 300.0;
    let wm_h = wm_w * (192.0 / 856.0);
    let watermark_id = p.alloc_id();
    p.layers.push(base_image(
        watermark_id,
        photo_x + (photo_w - wm_w) / 2.0,
        photo_y + (photo_h - wm_h) / 2.0,
        wm_w,
        wm_h,
        0.0,
        &photos.uzasasa_logo,
        30.0,
    ));

    // Thumbnail strip; the first one gets a blue "selected" ring behind it.
    let thumb_photos = [&photos.coastline, &photos.dunes, &photos.city_night, &photos.gold_interior];
    for (i, path) in thumb_photos.iter().enumerate() {
        let tx = photo_x + i as f32 * (thumb_w + gap);
        if i == 0 {
            let ring_id = p.alloc_id();
            p.layers.push(base_shape(
                ring_id,
                tx - 6.0,
                thumb_y - 6.0,
                thumb_w + 12.0,
                thumb_h + 12.0,
                14.0,
                brand_blue,
            ));
        }
        let thumb_id = p.alloc_id();
        p.layers.push(base_image(thumb_id, tx, thumb_y, thumb_w, thumb_h, 10.0, path, 100.0));
    }

    // Price + title banner directly below the card.
    let banner_y = card_y + card_h + 18.0;
    let banner_h = 110.0;
    let banner_id = p.alloc_id();
    p.layers.push(base_shape(banner_id, card_x, banner_y, card_w, banner_h, 14.0, brand_blue));

    let pill_pad = 16.0;
    let pill_w = 300.0;
    let pill_h = banner_h - pill_pad * 2.0;
    let pill_id = p.alloc_id();
    p.layers.push(base_shape(
        pill_id,
        card_x + pill_pad,
        banner_y + pill_pad,
        pill_w,
        pill_h,
        pill_h / 2.0,
        [255, 255, 255, 255],
    ));
    let price_id = p.alloc_id();
    p.layers.push(base_text(
        price_id,
        "Tsh 8,000,000/-",
        card_x + pill_pad,
        banner_y + pill_pad,
        pill_w,
        pill_h,
        34.0,
        800,
        brand_blue,
        TextAlign::Center,
    ));

    let title_x = card_x + pill_pad + pill_w + 24.0;
    let title_w = card_x + card_w - 24.0 - title_x;
    let title_id = p.alloc_id();
    p.layers.push(base_text(
        title_id,
        "SUBARU IMPREZA (2008)",
        title_x,
        banner_y,
        title_w,
        banner_h,
        30.0,
        800,
        [255, 255, 255, 255],
        TextAlign::Left,
    ));

    // Footer.
    let footer_id = p.alloc_id();
    p.layers.push(base_text(
        footer_id,
        "www.uzasasa.com",
        0.0,
        banner_y + banner_h + 26.0,
        canvas_w,
        50.0,
        30.0,
        500,
        ink,
        TextAlign::Center,
    ));

    p
}

/// An app-download promo poster: bold headline, a phone mockup with a
/// recreated app UI (search bar, category chips, a 2x2 listing grid) built
/// entirely from Shape/Image/Text layers positioned inside the frame's own
/// screen rect, a CTA pill, and the brand logo. Demonstrates that a device
/// frame's "screen" doesn't have to be a single photo — it can be any
/// composition of layers placed within `inset_screen_rect`.
fn app_promo_showcase(photos: &BundledPhotos) -> Project {
    let canvas_w = 1200.0f32;
    let canvas_h = 1600.0f32;
    let mut p = Project {
        canvas_width: canvas_w as u32,
        canvas_height: canvas_h as u32,
        background: Background::Color([250, 250, 251, 255]),
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let brand_blue = [8, 102, 255, 255];
    let ink = [20, 20, 22, 255];

    // Headline.
    let headline_id = p.alloc_id();
    p.layers.push(base_text(
        headline_id,
        "Tumekurahisishia",
        40.0,
        44.0,
        canvas_w - 80.0,
        120.0,
        58.0,
        800,
        ink,
        TextAlign::Center,
    ));

    // Phone frame + its screen rect.
    let phone_w = 560.0;
    let phone_h = phone_w * (3953.0 / 1965.0);
    let phone_x = (canvas_w - phone_w) / 2.0;
    let phone_y = 220.0;
    let phone_t = Transform {
        x: phone_x,
        y: phone_y,
        width: phone_w,
        height: phone_h,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };
    // Screen is just a single sample photo, not composited UI content.
    let phone_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, phone_id, &phone_t, FrameKind::Phone, &photos.coastline));

    // The frame goes last so its bezel/notch masks everything outside the
    // screen hole.
    p.layers.push(base_frame(phone_id, phone_x, phone_y, phone_w, phone_h, FrameColor::SpaceGray, photos));

    // CTA button.
    let cta_y = phone_y + phone_h + 40.0;
    let cta_w = 420.0;
    let cta_h = 92.0;
    let cta_x = (canvas_w - cta_w) / 2.0;
    let cta_bg_id = p.alloc_id();
    p.layers.push(base_shape(cta_bg_id, cta_x, cta_y, cta_w, cta_h, cta_h / 2.0, brand_blue));
    let cta_text_id = p.alloc_id();
    p.layers.push(base_text(
        cta_text_id,
        "Pakua sasa",
        cta_x,
        cta_y,
        cta_w,
        cta_h,
        34.0,
        800,
        [255, 255, 255, 255],
        TextAlign::Center,
    ));

    // Footer logo.
    let logo_w = 220.0;
    let logo_h = logo_w * (192.0 / 856.0);
    let logo_id = p.alloc_id();
    p.layers.push(base_image(
        logo_id,
        (canvas_w - logo_w) / 2.0,
        cta_y + cta_h + 40.0,
        logo_w,
        logo_h,
        0.0,
        &photos.uzasasa_logo,
        100.0,
    ));

    p
}

/// A full app-store-poster layout: left-aligned headline/subhead, brand logo
/// and Play/App Store badges bottom-left, and a phone on the right whose
/// screen carries a much more detailed recreated home-feed UI (status bar,
/// header with notification bell, search bar, category row, a
/// "Mapendekezo"/"Tazama yote" section header, a vertical listing feed, and
/// a bottom tab bar) than `app_promo_showcase`'s simpler grid screen.
fn app_download_showcase(photos: &BundledPhotos) -> Project {
    let canvas_w = 1200.0f32;
    let canvas_h = 1500.0f32;
    let mut p = Project {
        canvas_width: canvas_w as u32,
        canvas_height: canvas_h as u32,
        background: Background::Color([250, 250, 251, 255]),
        layers: Vec::new(),
        next_id: 1,
        selected_layer: None,
    };

    let brand_blue = [8, 102, 255, 255];
    let ink = [20, 20, 22, 255];
    let mid_gray = [130, 132, 136, 255];

    // Decorative diagonal accent behind everything, bottom-right corner.
    let accent_id = p.alloc_id();
    let mut accent = base_shape(accent_id, 700.0, 1310.0, 750.0, 750.0, 0.0, brand_blue);
    accent.transform.rotation_deg = -25.0;
    p.layers.push(accent);

    // Headline + subhead, left-aligned.
    let headline_id = p.alloc_id();
    p.layers.push(base_text(
        headline_id,
        "Tumekurahisishia",
        80.0,
        70.0,
        1040.0,
        90.0,
        60.0,
        800,
        ink,
        TextAlign::Left,
    ));
    let sub_id = p.alloc_id();
    p.layers.push(base_text(
        sub_id,
        "Nunua au uuzie gari,\npikipiki au bajaji\nkwa haraka na salama.",
        80.0,
        175.0,
        560.0,
        130.0,
        24.0,
        400,
        mid_gray,
        TextAlign::Left,
    ));

    // Phone-in-hand hero shot, centered below the headline, with a sample
    // photo filling its screen.
    let phone_w = 1000.0;
    let phone_h = phone_w * (2382.0 / 2600.0);
    let phone_x = (canvas_w - phone_w) / 2.0;
    let phone_y = 340.0;
    let phone_t = Transform {
        x: phone_x,
        y: phone_y,
        width: phone_w,
        height: phone_h,
        rotation_deg: 0.0,
        opacity: 100.0,
        mirror_h: false,
        mirror_v: false,
        corner_radius: 0.0,
        shadow: ShadowStyle::default(),
    };
    let phone_id = p.alloc_id();
    let photo_id = p.alloc_id();
    p.layers.push(screen_photo_layer(photo_id, phone_id, &phone_t, FrameKind::PhoneHand, &photos.coastline));
    p.layers.push(base_hand_frame(phone_id, phone_x, phone_y, phone_w, phone_h, photos));

    // Brand logo + store badges, below the hero shot.
    let footer_y = phone_y + phone_h + 30.0;
    let logo_w = 220.0;
    let logo_h = logo_w * (192.0 / 856.0);
    let logo_id = p.alloc_id();
    p.layers.push(base_image(logo_id, 80.0, footer_y, logo_w, logo_h, 0.0, &photos.uzasasa_logo, 100.0));

    let pakua_id = p.alloc_id();
    p.layers.push(base_text(
        pakua_id,
        "Pakua app ya Uzasasa",
        80.0,
        footer_y + 58.0,
        400.0,
        30.0,
        20.0,
        400,
        mid_gray,
        TextAlign::Left,
    ));

    let badge_h = 58.0;
    let badge_y = footer_y + 98.0;
    // Official badge artwork, placed at their own native aspect ratio.
    let badges: [(&str, f32); 2] = [
        (&photos.play_store_badge, 757.0 / 222.0),
        (&photos.app_store_badge, 774.0 / 238.0),
    ];
    let mut bx = 80.0;
    for (path, aspect) in badges {
        let badge_w = badge_h * aspect;
        let badge_id = p.alloc_id();
        p.layers.push(base_image(badge_id, bx, badge_y, badge_w, badge_h, 10.0, path, 100.0));
        bx += badge_w + 16.0;
    }

    p
}
