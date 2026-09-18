use serde::{Deserialize, Serialize};

pub const CANVAS_WIDTH: u32 = 1242;
pub const CANVAS_HEIGHT: u32 = 2688;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub background: Background,
    pub layers: Vec<Layer>,
    pub next_id: u64,
    pub selected_layer: Option<u64>,
}

impl Default for Project {
    fn default() -> Self {
        let mut project = Project {
            canvas_width: CANVAS_WIDTH,
            canvas_height: CANVAS_HEIGHT,
            background: Background::Color([245, 246, 248, 255]),
            layers: Vec::new(),
            next_id: 1,
            selected_layer: None,
        };

        let frame_id = project.alloc_id();
        project.layers.push(Layer {
            id: frame_id,
            name: "iPhone 12 Pro Max".to_string(),
            visible: true,
            kind: LayerKind::DeviceFrame(DeviceFrameLayer {
                style: FrameColor::SpaceGray,
                custom_image_path: None,
                kind: FrameKind::Phone,
            }),
            transform: Transform {
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
            },
        });

        let text_id = project.alloc_id();
        project.layers.push(Layer {
            id: text_id,
            name: "Headline".to_string(),
            visible: true,
            kind: LayerKind::Text(TextLayer {
                content: "Your headline here".to_string(),
                font_size: 90.0,
                weight: 700,
                line_height: 1.2,
                letter_spacing: 0.0,
                color: [30, 30, 34, 255],
                align: TextAlign::Center,
            }),
            transform: Transform {
                x: 80.0,
                y: 300.0,
                width: CANVAS_WIDTH as f32 - 160.0,
                height: 200.0,
                rotation_deg: 0.0,
                opacity: 100.0,
                mirror_h: false,
                mirror_v: false,
                corner_radius: 0.0,
                shadow: ShadowStyle::default(),
            },
        });

        project
    }
}

impl Project {
    pub fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn find_layer(&self, id: u64) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn find_layer_mut(&mut self, id: u64) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Background {
    Color([u8; 4]),
    Gradient {
        from: [u8; 4],
        to: [u8; 4],
        angle_deg: f32,
    },
    Image {
        path: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Layer {
    pub id: u64,
    pub name: String,
    pub visible: bool,
    pub kind: LayerKind,
    pub transform: Transform,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation_deg: f32,
    pub opacity: f32,
    pub mirror_h: bool,
    pub mirror_v: bool,
    pub corner_radius: f32,
    pub shadow: ShadowStyle,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShadowStyle {
    pub enabled: bool,
    pub color: [u8; 4],
    pub blur: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for ShadowStyle {
    fn default() -> Self {
        ShadowStyle {
            enabled: false,
            color: [0, 0, 0, 160],
            blur: 20.0,
            offset_x: 0.0,
            offset_y: 10.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum LayerKind {
    Image(ImageLayer),
    Text(TextLayer),
    DeviceFrame(DeviceFrameLayer),
    Shape(ShapeLayer),
    Paint(PaintLayer),
}

impl LayerKind {
    pub fn type_name(&self) -> &'static str {
        match self {
            LayerKind::Image(_) => "Image",
            LayerKind::Text(_) => "Text",
            LayerKind::DeviceFrame(_) => "Device Frame",
            LayerKind::Shape(_) => "Shape",
            LayerKind::Paint(_) => "Paint",
        }
    }
}

/// A freehand-paintable raster layer: a plain RGBA pixel buffer the brush
/// tool draws into directly, composited like any other layer. Used for
/// touch-ups — e.g. picking a color from the design with the eyedropper and
/// painting over an unwanted bit of a photo.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaintLayer {
    pub width: u32,
    pub height: u32,
    /// RGBA8, `width * height * 4` bytes, row-major top-to-bottom.
    pub pixels: Vec<u8>,
}

impl PaintLayer {
    pub fn new_transparent(width: u32, height: u32) -> Self {
        PaintLayer { width, height, pixels: vec![0u8; (width * height * 4) as usize] }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShapeLayer {
    pub fill_color: [u8; 4],
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageLayer {
    pub path: String,
    pub crop: Option<CropRect>,
    /// Set when this image is a device frame's screen content, so it can be
    /// moved/resized together with that frame. Points at the frame layer's id.
    #[serde(default)]
    pub linked_frame_id: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct CropRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Default for CropRect {
    fn default() -> Self {
        CropRect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }
    }
}

pub type FontWeight = u16;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextLayer {
    pub content: String,
    pub font_size: f32,
    pub weight: FontWeight,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub color: [u8; 4],
    pub align: TextAlign,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum FrameColor {
    SpaceGray,
    Silver,
    Gold,
    PacificBlue,
}

impl FrameColor {
    pub fn rgb(&self) -> [u8; 3] {
        match self {
            FrameColor::SpaceGray => [63, 63, 68],
            FrameColor::Silver => [226, 226, 230],
            FrameColor::Gold => [246, 220, 185],
            FrameColor::PacificBlue => [69, 100, 120],
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FrameColor::SpaceGray => "Space Gray",
            FrameColor::Silver => "Silver",
            FrameColor::Gold => "Gold",
            FrameColor::PacificBlue => "Pacific Blue",
        }
    }

    pub const ALL: [FrameColor; 4] = [
        FrameColor::SpaceGray,
        FrameColor::Silver,
        FrameColor::Gold,
        FrameColor::PacificBlue,
    ];
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeviceFrameLayer {
    pub style: FrameColor,
    /// When set, this real device-frame image (supplied by the user) is
    /// drawn instead of the built-in procedural silhouette.
    #[serde(default)]
    pub custom_image_path: Option<String>,
    /// Controls the screen-inset proportions used for a linked screen photo
    /// (a laptop has a much taller "chin" below the screen than a phone).
    #[serde(default)]
    pub kind: FrameKind,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FrameKind {
    #[default]
    Phone,
    Laptop,
    Android,
    AndroidWaterdrop,
    Ipad,
    /// A phone photographed/rendered in a hand, screen replaced with a real
    /// transparent hole — used for a more lifelike "hero shot" than the flat
    /// procedural phone frame.
    PhoneHand,
}
