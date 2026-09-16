use crate::assets::{AssetCache, FontManager};
use crate::bundled::BundledPhotos;
use crate::history::History;
use crate::model::*;
use crate::render::render_project;
use crate::templates;
use eframe::egui;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

enum DragMode {
    Move { start_pointer: egui::Pos2, start_xy: (f32, f32) },
    /// Moves every layer in the current multi-selection together by the
    /// same screen-space delta (no snapping — snapping a whole group against
    /// its own members' edges gets ambiguous fast, so group moves are raw).
    MoveGroup { start_pointer: egui::Pos2, starts: Vec<(u64, f32, f32)> },
    ResizeCorner { corner: Corner, start_pointer: egui::Pos2, start_rect: (f32, f32, f32, f32) },
    Rotate { start_pointer_angle: f32, start_rotation: f32 },
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Screen {
    Gallery,
    Editor,
}

struct GalleryCard {
    name: &'static str,
    texture: egui::TextureHandle,
    build: fn(&BundledPhotos) -> Project,
}

pub struct App {
    project: Project,
    assets: AssetCache,
    fonts: FontManager,
    history: History,
    texture: Option<egui::TextureHandle>,
    dirty: bool,
    drag: Option<DragMode>,
    drag_snapshot_taken: bool,
    project_path: Option<PathBuf>,
    status: Option<String>,
    screen: Screen,
    gallery_cards: Option<Vec<GalleryCard>>,
    photos: BundledPhotos,
    editing_text: Option<u64>,
    editing_buffer: String,
    editing_before: Option<Project>,
    editing_focus_pending: bool,
    show_layers_panel: bool,
    show_properties_panel: bool,
    fullscreen_preview: bool,
    fullscreen_texture: Option<egui::TextureHandle>,
    dark_mode: bool,
    /// The full multi-selection (Shift/Cmd-click to add/remove a layer).
    /// Always kept in sync with `project.selected_layer`, which remains the
    /// "primary" entry used for the properties panel and resize/rotate
    /// handles — those stay single-layer only; multi-select only adds the
    /// ability to move everything selected together.
    selected_layers: Vec<u64>,
}

impl App {
    pub fn new() -> Self {
        App {
            project: Project::default(),
            assets: AssetCache::default(),
            fonts: FontManager::load(),
            history: History::new(),
            texture: None,
            dirty: true,
            drag: None,
            drag_snapshot_taken: false,
            project_path: None,
            status: None,
            screen: Screen::Gallery,
            gallery_cards: None,
            photos: BundledPhotos::extract_all(),
            editing_text: None,
            editing_buffer: String::new(),
            editing_before: None,
            editing_focus_pending: false,
            show_layers_panel: true,
            show_properties_panel: true,
            fullscreen_preview: false,
            fullscreen_texture: None,
            dark_mode: true,
            selected_layers: Vec::new(),
        }
    }

    fn select_only(&mut self, id: u64) {
        self.project.selected_layer = Some(id);
        self.selected_layers = vec![id];
    }

    fn clear_selection(&mut self) {
        self.project.selected_layer = None;
        self.selected_layers.clear();
    }

    fn toggle_selection(&mut self, id: u64) {
        if let Some(pos) = self.selected_layers.iter().position(|&x| x == id) {
            self.selected_layers.remove(pos);
            if self.project.selected_layer == Some(id) {
                self.project.selected_layer = self.selected_layers.last().copied();
            }
        } else {
            self.selected_layers.push(id);
            self.project.selected_layer = Some(id);
        }
    }

    fn ensure_gallery(&mut self, ctx: &egui::Context) {
        if self.gallery_cards.is_some() {
            return;
        }
        let thumb_w = 260u32;
        let mut cards = Vec::new();
        for entry in templates::template_gallery() {
            let project = (entry.build)(&self.photos);
            let scale = thumb_w as f32 / project.canvas_width as f32;
            let preview = scaled_project(&project, scale);
            let img = render_project(&preview, &mut self.assets, &self.fonts);
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [img.width() as usize, img.height() as usize],
                img.as_raw(),
            );
            let texture = ctx.load_texture(
                format!("tmpl-{}", entry.name),
                color_image,
                egui::TextureOptions::LINEAR,
            );
            cards.push(GalleryCard { name: entry.name, texture, build: entry.build });
        }
        self.gallery_cards = Some(cards);
    }

    fn load_template(&mut self, build: fn(&BundledPhotos) -> Project) {
        self.project = build(&self.photos);
        self.history = History::new();
        self.project_path = None;
        self.screen = Screen::Editor;
        self.mark_dirty();
    }

    /// Full-window, chrome-free view of the design at full render resolution
    /// (no toolbar, no panels) — lets a wide canvas like the MacBook template
    /// be reviewed at its largest possible size. Exit via Escape, the close
    /// button, or a click anywhere outside the image.
    fn ui_fullscreen_preview(&mut self, ctx: &egui::Context) {
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let mut exit = escape;

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_gray(24)))
            .show(ctx, |ui| {
                let available = ui.available_size();
                if let Some(tex) = &self.fullscreen_texture {
                    let tex_size = tex.size_vec2();
                    let scale = (available.x / tex_size.x).min(available.y / tex_size.y).min(1.0).max(0.01);
                    let draw_size = tex_size * scale;
                    let rect = ui.max_rect();
                    let image_rect = egui::Rect::from_center_size(rect.center(), draw_size);

                    let bg_response = ui.allocate_rect(rect, egui::Sense::click());
                    ui.painter().image(
                        tex.id(),
                        image_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                    if bg_response.clicked() && !image_rect.contains(bg_response.interact_pointer_pos().unwrap_or_default()) {
                        exit = true;
                    }

                    let close_rect = egui::Rect::from_min_size(rect.min + egui::vec2(16.0, 16.0), egui::vec2(36.0, 36.0));
                    if ui.put(close_rect, egui::Button::new("✕")).clicked() {
                        exit = true;
                    }
                }
            });

        if exit {
            self.exit_fullscreen_preview();
        }
    }

    fn ui_gallery(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Choose a template to start editing");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(self.theme_toggle_label()).clicked() {
                    self.dark_mode = !self.dark_mode;
                }
            });
        });
        ui.label("Pick a prepared mockup, then customize every layer just like in the editor.");
        ui.add_space(8.0);

        let mut chosen: Option<fn(&BundledPhotos) -> Project> = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if let Some(cards) = &self.gallery_cards {
                    for card in cards {
                        ui.allocate_ui(egui::vec2(180.0, 420.0), |ui| {
                            ui.vertical_centered(|ui| {
                                let tex_size = card.texture.size_vec2();
                                let display_w = 160.0;
                                let display_h = display_w * tex_size.y / tex_size.x;
                                let img_button = egui::ImageButton::new((
                                    card.texture.id(),
                                    egui::vec2(display_w, display_h),
                                ));
                                if ui.add(img_button).clicked() {
                                    chosen = Some(card.build);
                                }
                                ui.label(card.name);
                                if ui.button("Edit").clicked() {
                                    chosen = Some(card.build);
                                }
                            });
                        });
                    }
                } else {
                    ui.label("Loading templates...");
                }
            });
        });

        if let Some(build) = chosen {
            self.load_template(build);
        }
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn begin_edit(&mut self) -> Project {
        self.project.clone()
    }

    fn commit_edit(&mut self, before: Project) {
        self.history.push(before);
        self.mark_dirty();
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        if !self.dirty {
            return;
        }
        // Lower the preview resolution while actively dragging so each frame
        // renders fast enough to keep up with the mouse; snap back to full
        // preview quality the instant the drag ends.
        let preview_w = if self.drag.is_some() { 360u32 } else { 620u32 };
        let scale = preview_w as f32 / self.project.canvas_width as f32;
        let preview = scaled_project(&self.project, scale);
        let img = render_project(&preview, &mut self.assets, &self.fonts);
        let color_image =
            egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
        match &mut self.texture {
            Some(tex) => tex.set(color_image, egui::TextureOptions::LINEAR),
            None => {
                self.texture = Some(ctx.load_texture("preview", color_image, egui::TextureOptions::LINEAR))
            }
        }
        self.dirty = false;
    }

    /// Renders the design at full resolution and switches to a full-window
    /// preview with no toolbar or panels, so a landscape canvas (like the
    /// MacBook template) can be reviewed at its largest possible size.
    fn enter_fullscreen_preview(&mut self, ctx: &egui::Context) {
        let img = render_project(&self.project, &mut self.assets, &self.fonts);
        let color_image =
            egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
        self.fullscreen_texture =
            Some(ctx.load_texture("fullscreen_preview", color_image, egui::TextureOptions::LINEAR));
        self.fullscreen_preview = true;
    }

    fn exit_fullscreen_preview(&mut self) {
        self.fullscreen_preview = false;
        self.fullscreen_texture = None;
    }

    fn do_new(&mut self) {
        self.project = Project::default();
        for layer in &mut self.project.layers {
            if let LayerKind::DeviceFrame(frame) = &mut layer.kind {
                frame.custom_image_path = Some(self.photos.iphone_frame.clone());
            }
        }
        self.history = History::new();
        self.project_path = None;
        self.mark_dirty();
    }

    fn do_save(&mut self) {
        let path = self.project_path.clone().or_else(|| {
            rfd::FileDialog::new()
                .set_file_name("project.mockup.json")
                .add_filter("Mockup Project", &["json"])
                .save_file()
        });
        if let Some(path) = path {
            match serde_json::to_string_pretty(&self.project) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&path, json) {
                        self.status = Some(format!("Save failed: {e}"));
                    } else {
                        self.status = Some(format!("Saved to {}", path.display()));
                        self.project_path = Some(path);
                    }
                }
                Err(e) => self.status = Some(format!("Save failed: {e}")),
            }
        }
    }

    fn do_save_as(&mut self) {
        self.project_path = None;
        self.do_save();
    }

    fn do_open(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Mockup Project", &["json"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(text) => match serde_json::from_str::<Project>(&text) {
                    Ok(project) => {
                        self.project = project;
                        self.history = History::new();
                        self.project_path = Some(path);
                        self.mark_dirty();
                    }
                    Err(e) => self.status = Some(format!("Open failed: {e}")),
                },
                Err(e) => self.status = Some(format!("Open failed: {e}")),
            }
        }
    }

    fn do_export(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("export.png")
            .add_filter("PNG Image", &["png"])
            .save_file()
        {
            let img = render_project(&self.project, &mut self.assets, &self.fonts);
            match img.save(&path) {
                Ok(_) => self.status = Some(format!("Exported to {}", path.display())),
                Err(e) => self.status = Some(format!("Export failed: {e}")),
            }
        }
    }

    fn do_add_image(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg"])
            .pick_file()
        {
            let path_str = path.to_string_lossy().to_string();
            let before = self.begin_edit();
            let (w, h) = self
                .assets
                .get_or_load(&path_str)
                .map(|img| img.dimensions())
                .unwrap_or((800, 800));
            let target_w = 900.0f32.min(self.project.canvas_width as f32 * 0.8);
            let target_h = target_w * (h as f32 / w as f32);
            let id = self.project.alloc_id();
            let x = (self.project.canvas_width as f32 - target_w) / 2.0;
            let y = (self.project.canvas_height as f32 - target_h) / 2.0;
            self.project.layers.push(Layer {
                id,
                name: "Image".to_string(),
                visible: true,
                kind: LayerKind::Image(ImageLayer { path: path_str, crop: None, linked_frame_id: None }),
                transform: Transform {
                    x,
                    y,
                    width: target_w,
                    height: target_h,
                    rotation_deg: 0.0,
                    opacity: 100.0,
                    mirror_h: false,
                    mirror_v: false,
                    corner_radius: 0.0,
                    shadow: ShadowStyle::default(),
                },
            });
            self.select_only(id);
            self.commit_edit(before);
        }
    }

    fn do_add_text(&mut self) {
        let before = self.begin_edit();
        let id = self.project.alloc_id();
        self.project.layers.push(Layer {
            id,
            name: "Text".to_string(),
            visible: true,
            kind: LayerKind::Text(TextLayer {
                content: "New text".to_string(),
                font_size: 72.0,
                weight: 600,
                line_height: 1.2,
                letter_spacing: 0.0,
                color: [20, 20, 24, 255],
                align: TextAlign::Center,
            }),
            transform: Transform {
                x: 80.0,
                y: 500.0,
                width: self.project.canvas_width as f32 - 160.0,
                height: 200.0,
                rotation_deg: 0.0,
                opacity: 100.0,
                mirror_h: false,
                mirror_v: false,
                corner_radius: 0.0,
                shadow: ShadowStyle::default(),
            },
        });
        self.select_only(id);
        self.commit_edit(before);
    }

    fn do_add_frame(&mut self) {
        let before = self.begin_edit();
        let id = self.project.alloc_id();
        self.project.layers.push(Layer {
            id,
            name: "Device Frame".to_string(),
            visible: true,
            kind: LayerKind::DeviceFrame(DeviceFrameLayer {
                style: FrameColor::SpaceGray,
                custom_image_path: Some(self.photos.iphone_frame.clone()),
                kind: FrameKind::Phone,
            }),
            transform: Transform {
                x: self.project.canvas_width as f32 * 0.5 - 560.0,
                y: self.project.canvas_height as f32 * 0.5 - 1140.0,
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
        self.select_only(id);
        self.commit_edit(before);
    }

    fn do_add_laptop_frame(&mut self) {
        let before = self.begin_edit();
        let id = self.project.alloc_id();
        let w = (self.project.canvas_width as f32 * 0.8).min(2260.0);
        let h = w * (1509.0 / 2600.0);
        self.project.layers.push(Layer {
            id,
            name: "MacBook Frame".to_string(),
            visible: true,
            kind: LayerKind::DeviceFrame(DeviceFrameLayer {
                style: FrameColor::SpaceGray,
                custom_image_path: Some(self.photos.macbook_frame.clone()),
                kind: FrameKind::Laptop,
            }),
            transform: Transform {
                x: (self.project.canvas_width as f32 - w) / 2.0,
                y: (self.project.canvas_height as f32 - h) / 2.0,
                width: w,
                height: h,
                rotation_deg: 0.0,
                opacity: 100.0,
                mirror_h: false,
                mirror_v: false,
                corner_radius: 0.0,
                shadow: ShadowStyle::default(),
            },
        });
        self.select_only(id);
        self.commit_edit(before);
    }

    fn do_add_android_frame(&mut self) {
        let before = self.begin_edit();
        let id = self.project.alloc_id();
        let w = (self.project.canvas_width as f32 * 0.75).min(1000.0);
        let h = w * (6455.0 / 3091.0);
        self.project.layers.push(Layer {
            id,
            name: "Android Frame".to_string(),
            visible: true,
            kind: LayerKind::DeviceFrame(DeviceFrameLayer {
                style: FrameColor::SpaceGray,
                custom_image_path: Some(self.photos.android_frame.clone()),
                kind: FrameKind::Android,
            }),
            transform: Transform {
                x: (self.project.canvas_width as f32 - w) / 2.0,
                y: (self.project.canvas_height as f32 - h) / 2.0,
                width: w,
                height: h,
                rotation_deg: 0.0,
                opacity: 100.0,
                mirror_h: false,
                mirror_v: false,
                corner_radius: 0.0,
                shadow: ShadowStyle::default(),
            },
        });
        self.select_only(id);
        self.commit_edit(before);
    }

    fn do_add_android_waterdrop_frame(&mut self) {
        let before = self.begin_edit();
        let id = self.project.alloc_id();
        let w = (self.project.canvas_width as f32 * 0.75).min(1000.0);
        let h = w * (5121.0 / 2580.0);
        self.project.layers.push(Layer {
            id,
            name: "Android Frame".to_string(),
            visible: true,
            kind: LayerKind::DeviceFrame(DeviceFrameLayer {
                style: FrameColor::SpaceGray,
                custom_image_path: Some(self.photos.android_waterdrop_frame.clone()),
                kind: FrameKind::AndroidWaterdrop,
            }),
            transform: Transform {
                x: (self.project.canvas_width as f32 - w) / 2.0,
                y: (self.project.canvas_height as f32 - h) / 2.0,
                width: w,
                height: h,
                rotation_deg: 0.0,
                opacity: 100.0,
                mirror_h: false,
                mirror_v: false,
                corner_radius: 0.0,
                shadow: ShadowStyle::default(),
            },
        });
        self.select_only(id);
        self.commit_edit(before);
    }

    fn do_add_ipad_frame(&mut self) {
        let before = self.begin_edit();
        let id = self.project.alloc_id();
        let w = (self.project.canvas_width as f32 * 0.75).min(980.0);
        let h = w * (3722.0 / 2230.0);
        self.project.layers.push(Layer {
            id,
            name: "iPad Frame".to_string(),
            visible: true,
            kind: LayerKind::DeviceFrame(DeviceFrameLayer {
                style: FrameColor::SpaceGray,
                custom_image_path: Some(self.photos.ipad_frame.clone()),
                kind: FrameKind::Ipad,
            }),
            transform: Transform {
                x: (self.project.canvas_width as f32 - w) / 2.0,
                y: (self.project.canvas_height as f32 - h) / 2.0,
                width: w,
                height: h,
                rotation_deg: 0.0,
                opacity: 100.0,
                mirror_h: false,
                mirror_v: false,
                corner_radius: 0.0,
                shadow: ShadowStyle::default(),
            },
        });
        self.select_only(id);
        self.commit_edit(before);
    }

    fn do_add_shape(&mut self) {
        let before = self.begin_edit();
        let id = self.project.alloc_id();
        let w = 400.0;
        let h = 400.0;
        self.project.layers.push(Layer {
            id,
            name: "Shape".to_string(),
            visible: true,
            kind: LayerKind::Shape(ShapeLayer { fill_color: [40, 100, 240, 255] }),
            transform: Transform {
                x: (self.project.canvas_width as f32 - w) / 2.0,
                y: (self.project.canvas_height as f32 - h) / 2.0,
                width: w,
                height: h,
                rotation_deg: 0.0,
                opacity: 100.0,
                mirror_h: false,
                mirror_v: false,
                corner_radius: 40.0,
                shadow: ShadowStyle::default(),
            },
        });
        self.select_only(id);
        self.commit_edit(before);
    }

    fn do_duplicate(&mut self) {
        let targets = self.selection_or_primary();
        if targets.is_empty() {
            return;
        }
        let before = self.begin_edit();
        let mut new_ids = Vec::new();
        for sel in targets {
            if let Some(layer) = self.project.find_layer(sel).cloned() {
                let new_id = self.project.alloc_id();
                let mut new_layer = layer;
                new_layer.id = new_id;
                new_layer.name = format!("{} copy", new_layer.name);
                new_layer.transform.x += 24.0;
                new_layer.transform.y += 24.0;
                self.project.layers.push(new_layer);
                new_ids.push(new_id);
            }
        }
        if let Some(&last) = new_ids.last() {
            self.project.selected_layer = Some(last);
        }
        self.selected_layers = new_ids;
        self.commit_edit(before);
    }

    fn do_delete(&mut self) {
        let targets = self.selection_or_primary();
        if targets.is_empty() {
            return;
        }
        let before = self.begin_edit();
        self.project.layers.retain(|l| !targets.contains(&l.id));
        self.clear_selection();
        self.commit_edit(before);
    }

    /// The full multi-selection, falling back to just the primary selected
    /// layer when nothing is multi-selected (e.g. a plain single click).
    fn selection_or_primary(&self) -> Vec<u64> {
        if !self.selected_layers.is_empty() {
            self.selected_layers.clone()
        } else {
            self.project.selected_layer.into_iter().collect()
        }
    }

    fn do_undo(&mut self) {
        let current = self.project.clone();
        if let Some(prev) = self.history.undo(current) {
            self.project = prev;
            self.mark_dirty();
        }
    }

    fn do_redo(&mut self) {
        let current = self.project.clone();
        if let Some(next) = self.history.redo(current) {
            self.project = next;
            self.mark_dirty();
        }
    }

    fn ui_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Templates...").clicked() {
                self.screen = Screen::Gallery;
            }
            if ui.button("New").clicked() {
                self.do_new();
            }
            if ui.button("Open...").clicked() {
                self.do_open();
            }
            if ui.button("Save").clicked() {
                self.do_save();
            }
            if ui.button("Save As...").clicked() {
                self.do_save_as();
            }
            ui.separator();
            if ui
                .add_enabled(self.history.can_undo(), egui::Button::new("Undo"))
                .clicked()
            {
                self.do_undo();
            }
            if ui
                .add_enabled(self.history.can_redo(), egui::Button::new("Redo"))
                .clicked()
            {
                self.do_redo();
            }
            ui.separator();
            if ui
                .add_enabled(self.project.selected_layer.is_some(), egui::Button::new("Duplicate"))
                .clicked()
            {
                self.do_duplicate();
            }
            if ui
                .add_enabled(self.project.selected_layer.is_some(), egui::Button::new("Delete"))
                .clicked()
            {
                self.do_delete();
            }
            ui.separator();
            if ui.button("Export PNG...").clicked() {
                self.do_export();
            }
            ui.separator();
            if ui.button("Upscale Image File...").clicked() {
                self.do_upscale_file();
            }
            ui.separator();
            if ui.button("Preview Full Screen").clicked() {
                let ctx = ui.ctx().clone();
                self.enter_fullscreen_preview(&ctx);
            }
            ui.separator();
            ui.checkbox(&mut self.show_layers_panel, "Layers");
            ui.checkbox(&mut self.show_properties_panel, "Properties");
            if ui
                .button("Compact Panels")
                .on_hover_text("Shrink both side panels to their minimum width — handy for wide canvases like MacBook Showcase")
                .clicked()
            {
                let ctx = ui.ctx().clone();
                set_panel_width(&ctx, "layers", 110.0);
                set_panel_width(&ctx, "properties", 150.0);
            }
            ui.separator();
            if ui.button(self.theme_toggle_label()).clicked() {
                self.dark_mode = !self.dark_mode;
            }
        });
        if let Some(status) = &self.status {
            ui.label(egui::RichText::new(status).weak());
        }
    }

    fn theme_toggle_label(&self) -> String {
        if self.dark_mode {
            "☀ Light Mode".to_string()
        } else {
            "🌙 Dark Mode".to_string()
        }
    }

    fn do_upscale_file(&mut self) {
        let Some(input) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg"])
            .pick_file()
        else {
            return;
        };
        let Ok(img) = image::open(&input) else {
            self.status = Some("Upscale failed: could not open image".to_string());
            return;
        };
        let factor = 2u32;
        let (w, h) = (img.width() * factor, img.height() * factor);
        let resized = image::imageops::resize(&img, w, h, image::imageops::FilterType::Lanczos3);
        let default_name = format!(
            "{}_upscaled_2x.png",
            input.file_stem().and_then(|s| s.to_str()).unwrap_or("image")
        );
        if let Some(output) = rfd::FileDialog::new().set_file_name(&default_name).save_file() {
            match resized.save(&output) {
                Ok(_) => self.status = Some(format!("Upscaled 2x -> {}", output.display())),
                Err(e) => self.status = Some(format!("Upscale save failed: {e}")),
            }
        }
    }

    fn ui_layers_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Layers");
        ui.horizontal_wrapped(|ui| {
            if ui.button("+ Image").clicked() {
                self.do_add_image();
            }
            if ui.button("+ Text").clicked() {
                self.do_add_text();
            }
            if ui.button("+ Frame").clicked() {
                self.do_add_frame();
            }
            if ui.button("+ MacBook").clicked() {
                self.do_add_laptop_frame();
            }
            if ui.button("+ Android").clicked() {
                self.do_add_android_frame();
            }
            if ui.button("+ Android 2").clicked() {
                self.do_add_android_waterdrop_frame();
            }
            if ui.button("+ iPad").clicked() {
                self.do_add_ipad_frame();
            }
            if ui.button("+ Shape").clicked() {
                self.do_add_shape();
            }
        });
        ui.separator();

        let mut move_up: Option<usize> = None;
        let mut move_down: Option<usize> = None;
        let mut select: Option<(u64, bool)> = None;
        let mut toggle_visible: Option<u64> = None;
        let len = self.project.layers.len();

        ui.label(egui::RichText::new("Shift/Cmd-click to select multiple, then drag any of them to move the group.").weak().small());

        egui::ScrollArea::vertical().show(ui, |ui| {
            for idx in (0..len).rev() {
                let layer = &self.project.layers[idx];
                let is_selected = self.selected_layers.contains(&layer.id);
                ui.horizontal(|ui| {
                    let mut visible = layer.visible;
                    if ui.checkbox(&mut visible, "").changed() {
                        toggle_visible = Some(layer.id);
                    }
                    let label = format!("{} ({})", layer.name, layer.kind.type_name());
                    let resp = ui.selectable_label(is_selected, label);
                    if resp.clicked() {
                        let additive = ui.input(|i| i.modifiers.shift || i.modifiers.command || i.modifiers.ctrl);
                        select = Some((layer.id, additive));
                    }
                    if idx + 1 < len && ui.small_button("^").clicked() {
                        move_up = Some(idx);
                    }
                    if idx > 0 && ui.small_button("v").clicked() {
                        move_down = Some(idx);
                    }
                });
            }
        });

        if let Some((id, additive)) = select {
            if additive {
                self.toggle_selection(id);
            } else {
                self.select_only(id);
            }
        }
        if let Some(id) = toggle_visible {
            let before = self.begin_edit();
            if let Some(l) = self.project.find_layer_mut(id) {
                l.visible = !l.visible;
            }
            self.commit_edit(before);
        }
        if let Some(idx) = move_up {
            let before = self.begin_edit();
            self.project.layers.swap(idx, idx + 1);
            self.commit_edit(before);
        }
        if let Some(idx) = move_down {
            let before = self.begin_edit();
            self.project.layers.swap(idx, idx - 1);
            self.commit_edit(before);
        }
    }

    fn ui_background_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Canvas Background");
        let mut kind_idx = match &self.project.background {
            Background::Color(_) => 0,
            Background::Gradient { .. } => 1,
            Background::Image { .. } => 2,
        };
        let prev_idx = kind_idx;
        egui::ComboBox::from_label("Type")
            .selected_text(["Color", "Gradient", "Image"][kind_idx])
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut kind_idx, 0, "Color");
                ui.selectable_value(&mut kind_idx, 1, "Gradient");
                ui.selectable_value(&mut kind_idx, 2, "Image");
            });
        if kind_idx != prev_idx {
            let before = self.begin_edit();
            self.project.background = match kind_idx {
                0 => Background::Color([245, 246, 248, 255]),
                1 => Background::Gradient {
                    from: [255, 200, 220, 255],
                    to: [180, 200, 255, 255],
                    angle_deg: 45.0,
                },
                _ => Background::Image { path: String::new() },
            };
            self.commit_edit(before);
        }

        let mut bg_changed = false;
        match &mut self.project.background {
            Background::Color(c) => {
                let mut color = egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]);
                let resp = egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut color,
                    egui::color_picker::Alpha::Opaque,
                );
                if resp.changed() {
                    *c = color.to_array();
                    bg_changed = true;
                }
            }
            Background::Gradient { from, to, angle_deg } => {
                let mut c1 = egui::Color32::from_rgba_unmultiplied(from[0], from[1], from[2], from[3]);
                let mut c2 = egui::Color32::from_rgba_unmultiplied(to[0], to[1], to[2], to[3]);
                ui.horizontal(|ui| {
                    ui.label("From");
                    bg_changed |= egui::color_picker::color_edit_button_srgba(ui, &mut c1, egui::color_picker::Alpha::Opaque).changed();
                    ui.label("To");
                    bg_changed |= egui::color_picker::color_edit_button_srgba(ui, &mut c2, egui::color_picker::Alpha::Opaque).changed();
                });
                *from = c1.to_array();
                *to = c2.to_array();
                bg_changed |= ui.add(egui::Slider::new(angle_deg, 0.0..=360.0).text("Angle")).changed();
            }
            Background::Image { path } => {
                ui.horizontal(|ui| {
                    ui.label(if path.is_empty() { "(no image)" } else { path.as_str() });
                    if ui.button("Choose...").clicked() {
                        if let Some(p) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg"])
                            .pick_file()
                        {
                            *path = p.to_string_lossy().to_string();
                            bg_changed = true;
                        }
                    }
                });
            }
        }
        if bg_changed {
            self.mark_dirty();
        }
    }

    fn ui_properties_panel(&mut self, ui: &mut egui::Ui) {
        self.ui_background_panel(ui);
        ui.separator();

        let Some(id) = self.project.selected_layer else {
            ui.label("Select a layer to edit its properties.");
            return;
        };
        let Some(layer_idx) = self.project.layers.iter().position(|l| l.id == id) else {
            return;
        };

        ui.heading("Layer Properties");
        let mut name = self.project.layers[layer_idx].name.clone();
        if ui.text_edit_singleline(&mut name).changed() {
            self.project.layers[layer_idx].name = name;
        }

        ui.separator();
        ui.label("Transform");
        let mut changed = false;
        {
            let t = &mut self.project.layers[layer_idx].transform;
            ui.horizontal(|ui| {
                changed |= ui.add(egui::DragValue::new(&mut t.x).prefix("X: ")).changed();
                changed |= ui.add(egui::DragValue::new(&mut t.y).prefix("Y: ")).changed();
            });
            ui.horizontal(|ui| {
                changed |= ui
                    .add(egui::DragValue::new(&mut t.width).prefix("W: ").range(1.0..=4000.0))
                    .changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut t.height).prefix("H: ").range(1.0..=6000.0))
                    .changed();
            });
            changed |= ui
                .add(egui::Slider::new(&mut t.rotation_deg, -180.0..=180.0).text("Rotation"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut t.opacity, 0.0..=100.0).text("Opacity"))
                .changed();
            ui.horizontal(|ui| {
                changed |= ui.checkbox(&mut t.mirror_h, "Mirror H").changed();
                changed |= ui.checkbox(&mut t.mirror_v, "Mirror V").changed();
            });
        }

        if changed {
            self.sync_linked_photo_transform(id);
        }

        let supports_radius_shadow = matches!(
            self.project.layers[layer_idx].kind,
            LayerKind::Image(_) | LayerKind::Shape(_)
        );
        if supports_radius_shadow {
            let t = &mut self.project.layers[layer_idx].transform;
            changed |= ui
                .add(egui::Slider::new(&mut t.corner_radius, 0.0..=400.0).text("Corner Radius"))
                .changed();
            ui.separator();
            ui.label("Shadow");
            changed |= ui.checkbox(&mut t.shadow.enabled, "Enabled").changed();
            if t.shadow.enabled {
                changed |= ui.add(egui::Slider::new(&mut t.shadow.blur, 0.0..=120.0).text("Blur")).changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut t.shadow.offset_x).prefix("Offset X: "))
                    .changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut t.shadow.offset_y).prefix("Offset Y: "))
                    .changed();
                let mut color = egui::Color32::from_rgba_unmultiplied(
                    t.shadow.color[0],
                    t.shadow.color[1],
                    t.shadow.color[2],
                    t.shadow.color[3],
                );
                if egui::color_picker::color_edit_button_srgba(ui, &mut color, egui::color_picker::Alpha::OnlyBlend)
                    .changed()
                {
                    t.shadow.color = color.to_array();
                    changed = true;
                }
            }
        }

        ui.separator();
        let (layer_w, layer_h) = {
            let t = &self.project.layers[layer_idx].transform;
            (t.width, t.height)
        };
        match &mut self.project.layers[layer_idx].kind {
            LayerKind::Image(img) => {
                if ui.button("Replace Image...").clicked() {
                    if let Some(p) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg"])
                        .pick_file()
                    {
                        let path_str = p.to_string_lossy().to_string();
                        img.crop = templates::aspect_crop(&path_str, layer_w, layer_h);
                        img.path = path_str;
                        changed = true;
                    }
                }
                ui.label(&img.path);
                ui.separator();
                ui.label("Crop (normalized)");
                let mut crop = img.crop.unwrap_or_default();
                ui.horizontal(|ui| {
                    changed |= ui.add(egui::Slider::new(&mut crop.x, 0.0..=0.9).text("X")).changed();
                    changed |= ui.add(egui::Slider::new(&mut crop.y, 0.0..=0.9).text("Y")).changed();
                });
                ui.horizontal(|ui| {
                    changed |= ui.add(egui::Slider::new(&mut crop.w, 0.05..=1.0).text("W")).changed();
                    changed |= ui.add(egui::Slider::new(&mut crop.h, 0.05..=1.0).text("H")).changed();
                });
                img.crop = Some(crop);
                if ui.button("Reset Crop").clicked() {
                    img.crop = None;
                    changed = true;
                }
            }
            LayerKind::Text(text) => {
                changed |= ui.text_edit_multiline(&mut text.content).changed();
                changed |= ui.add(egui::Slider::new(&mut text.font_size, 8.0..=400.0).text("Font Size")).changed();
                let mut weight = text.weight as f32;
                if ui.add(egui::Slider::new(&mut weight, 100.0..=900.0).step_by(100.0).text("Weight")).changed() {
                    text.weight = weight.round() as u16;
                    changed = true;
                }
                changed |= ui.add(egui::Slider::new(&mut text.line_height, 0.8..=3.0).text("Line Height")).changed();
                changed |= ui
                    .add(egui::Slider::new(&mut text.letter_spacing, -10.0..=60.0).text("Letter Spacing"))
                    .changed();
                let mut color = egui::Color32::from_rgba_unmultiplied(
                    text.color[0],
                    text.color[1],
                    text.color[2],
                    text.color[3],
                );
                if egui::color_picker::color_edit_button_srgba(ui, &mut color, egui::color_picker::Alpha::OnlyBlend)
                    .changed()
                {
                    text.color = color.to_array();
                    changed = true;
                }
                ui.horizontal(|ui| {
                    if ui.selectable_label(text.align == TextAlign::Left, "Left").clicked() {
                        text.align = TextAlign::Left;
                        changed = true;
                    }
                    if ui.selectable_label(text.align == TextAlign::Center, "Center").clicked() {
                        text.align = TextAlign::Center;
                        changed = true;
                    }
                    if ui.selectable_label(text.align == TextAlign::Right, "Right").clicked() {
                        text.align = TextAlign::Right;
                        changed = true;
                    }
                });
            }
            LayerKind::Shape(shape) => {
                let mut color = egui::Color32::from_rgba_unmultiplied(
                    shape.fill_color[0],
                    shape.fill_color[1],
                    shape.fill_color[2],
                    shape.fill_color[3],
                );
                ui.horizontal(|ui| {
                    ui.label("Fill");
                    if egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut color,
                        egui::color_picker::Alpha::OnlyBlend,
                    )
                    .changed()
                    {
                        shape.fill_color = color.to_array();
                        changed = true;
                    }
                });
            }
            LayerKind::DeviceFrame(frame) => {
                let using_custom_image = frame.custom_image_path.is_some();
                ui.add_enabled_ui(!using_custom_image, |ui| {
                    egui::ComboBox::from_label("Frame Color")
                        .selected_text(frame.style.label())
                        .show_ui(ui, |ui| {
                            for style in FrameColor::ALL {
                                if ui.selectable_label(frame.style == style, style.label()).clicked() {
                                    frame.style = style;
                                    changed = true;
                                }
                            }
                        });
                });
                if using_custom_image {
                    ui.label(egui::RichText::new("(color ignored while a frame image is set)").weak().small());
                }

                ui.separator();
                ui.label("Frame image");
                ui.label(
                    egui::RichText::new(
                        "A real device mockup PNG replaces the built-in procedurally \
                         drawn frame, transparent screen area and all. Only load an \
                         image you have the rights to use.",
                    )
                    .weak()
                    .small(),
                );
                match &frame.custom_image_path {
                    Some(path) => {
                        ui.label(path.as_str());
                        if ui.button("Use built-in frame instead").clicked() {
                            frame.custom_image_path = None;
                            changed = true;
                        }
                    }
                    None => {}
                }
                if ui.button("Load custom frame image...").clicked() {
                    if let Some(p) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg"])
                        .pick_file()
                    {
                        frame.custom_image_path = Some(p.to_string_lossy().to_string());
                        changed = true;
                    }
                }
            }
        }

        if changed {
            self.mark_dirty();
        }
    }

    fn ui_canvas(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let aspect = self.project.canvas_width as f32 / self.project.canvas_height as f32;
        let mut w = available.x;
        let mut h = w / aspect;
        if h > available.y {
            h = available.y;
            w = h * aspect;
        }

        let (rect, response) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click_and_drag());

        if let Some(tex) = &self.texture {
            ui.painter().image(
                tex.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }

        let scale = rect.width() / self.project.canvas_width as f32;
        let to_project = |p: egui::Pos2| -> egui::Pos2 {
            egui::pos2((p.x - rect.min.x) / scale, (p.y - rect.min.y) / scale)
        };
        let to_screen = |v: f32| -> f32 { v * scale };

        // Inline text editing overlay: takes over the canvas response while active.
        if let Some(edit_id) = self.editing_text {
            let mut exit_edit = self.project.find_layer(edit_id).is_none();
            if let Some(layer) = self.project.find_layer(edit_id).cloned() {
                let t = &layer.transform;
                let min = rect.min + egui::vec2(t.x * scale, t.y * scale);
                let max = min + egui::vec2(t.width * scale, t.height * scale);
                let edit_rect = egui::Rect::from_min_max(min, max);

                ui.painter().rect_filled(edit_rect, 4.0, egui::Color32::from_white_alpha(245));
                ui.painter()
                    .rect_stroke(edit_rect, 4.0, egui::Stroke::new(2.0f32, egui::Color32::from_rgb(60, 140, 255)));

                let mut buffer = self.editing_buffer.clone();
                let edit_resp = ui.put(
                    edit_rect.shrink(6.0),
                    egui::TextEdit::multiline(&mut buffer)
                        .frame(false)
                        .font(egui::FontId::proportional(18.0)),
                );
                if self.editing_focus_pending {
                    edit_resp.request_focus();
                    self.editing_focus_pending = false;
                }

                if buffer != self.editing_buffer {
                    self.editing_buffer = buffer;
                    if let Some(LayerKind::Text(text)) = self.project.find_layer_mut(edit_id).map(|l| &mut l.kind) {
                        text.content = self.editing_buffer.clone();
                    }
                    self.mark_dirty();
                }

                let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
                let clicked_outside = response.clicked()
                    && response
                        .interact_pointer_pos()
                        .map_or(false, |p| !edit_rect.contains(p));
                if escape || clicked_outside {
                    exit_edit = true;
                }
            }

            if exit_edit {
                if let Some(before) = self.editing_before.take() {
                    self.history.push(before);
                }
                self.editing_text = None;
                self.mark_dirty();
            } else {
                return;
            }
        }

        // Selection outline + resize/rotate handles (for the currently selected layer).
        const HANDLE_R: f32 = 7.0;
        const ROTATE_STEM: f32 = 28.0;
        let accent = egui::Color32::from_rgb(60, 140, 255);
        let mut corner_handles: [Option<(Corner, egui::Pos2)>; 4] = [None; 4];
        let mut rotate_handle: Option<egui::Pos2> = None;
        // Resize/rotate handles only make sense for a single selected layer;
        // with a multi-selection, outline every selected layer instead.
        if self.selected_layers.len() > 1 {
            for &id in &self.selected_layers {
                if let Some(layer) = self.project.find_layer(id) {
                    let t = &layer.transform;
                    let min = rect.min + egui::vec2(t.x * scale, t.y * scale);
                    let max = min + egui::vec2(t.width * scale, t.height * scale);
                    ui.painter().rect_stroke(egui::Rect::from_min_max(min, max), 0.0, egui::Stroke::new(2.0f32, accent));
                }
            }
        } else if let Some(sel_id) = self.project.selected_layer {
            if let Some(layer) = self.project.find_layer(sel_id) {
                let t = &layer.transform;
                let min = rect.min + egui::vec2(t.x * scale, t.y * scale);
                let max = min + egui::vec2(t.width * scale, t.height * scale);
                let box_rect = egui::Rect::from_min_max(min, max);
                ui.painter().rect_stroke(box_rect, 0.0, egui::Stroke::new(2.0f32, accent));

                let top_center = egui::pos2((box_rect.min.x + box_rect.max.x) / 2.0, box_rect.min.y);
                let rotate_pos = top_center - egui::vec2(0.0, ROTATE_STEM);
                ui.painter().line_segment([top_center, rotate_pos], egui::Stroke::new(1.5f32, accent));
                ui.painter().circle_filled(rotate_pos, HANDLE_R, egui::Color32::WHITE);
                ui.painter().circle_stroke(rotate_pos, HANDLE_R, egui::Stroke::new(2.0f32, accent));
                rotate_handle = Some(rotate_pos);

                let corners = [
                    (Corner::TopLeft, box_rect.min),
                    (Corner::TopRight, egui::pos2(box_rect.max.x, box_rect.min.y)),
                    (Corner::BottomLeft, egui::pos2(box_rect.min.x, box_rect.max.y)),
                    (Corner::BottomRight, box_rect.max),
                ];
                for (i, (corner, pos)) in corners.into_iter().enumerate() {
                    ui.painter().circle_filled(pos, HANDLE_R, egui::Color32::WHITE);
                    ui.painter().circle_stroke(pos, HANDLE_R, egui::Stroke::new(2.0f32, accent));
                    corner_handles[i] = Some((corner, pos));
                }
            }
        }
        let handle_hit = |p: egui::Pos2, center: egui::Pos2| (p - center).length() <= HANDLE_R + 4.0;

        // Drag start: resize/rotate only if grabbing the selected layer's own
        // handle; otherwise always hit-test fresh so a click-and-drag on a
        // different layer grabs *that* layer instead of silently acting on a
        // stale selection (this was the cause of erratic/oversized jumps).
        if response.drag_started() {
            if let Some(pointer) = response.interact_pointer_pos() {
                if let Some(sel_id) = self.project.selected_layer {
                    if let Some(rp) = rotate_handle {
                        if handle_hit(pointer, rp) {
                            if let Some(layer) = self.project.find_layer(sel_id) {
                                let center = rect.min
                                    + egui::vec2(
                                        (layer.transform.x + layer.transform.width / 2.0) * scale,
                                        (layer.transform.y + layer.transform.height / 2.0) * scale,
                                    );
                                let d = pointer - center;
                                let angle = d.y.atan2(d.x).to_degrees() + 90.0;
                                self.drag = Some(DragMode::Rotate {
                                    start_pointer_angle: angle,
                                    start_rotation: layer.transform.rotation_deg,
                                });
                            }
                        }
                    }
                    if self.drag.is_none() {
                        for slot in corner_handles.iter().flatten() {
                            let (corner, pos) = *slot;
                            if handle_hit(pointer, pos) {
                                if let Some(layer) = self.project.find_layer(sel_id) {
                                    let t = &layer.transform;
                                    self.drag = Some(DragMode::ResizeCorner {
                                        corner,
                                        start_pointer: pointer,
                                        start_rect: (t.x, t.y, t.width, t.height),
                                    });
                                }
                                break;
                            }
                        }
                    }
                }
                if self.drag.is_none() {
                    let p = to_project(pointer);
                    if let Some(hit_id) = self.hit_test_layer(p) {
                        if self.selected_layers.len() > 1 && self.selected_layers.contains(&hit_id) {
                            // Grabbing a member of the current multi-selection
                            // moves every member together.
                            self.project.selected_layer = Some(hit_id);
                            let starts: Vec<(u64, f32, f32)> = self
                                .selected_layers
                                .iter()
                                .filter_map(|&id| self.project.find_layer(id).map(|l| (id, l.transform.x, l.transform.y)))
                                .collect();
                            self.drag = Some(DragMode::MoveGroup { start_pointer: pointer, starts });
                        } else {
                            self.select_only(hit_id);
                            if let Some(layer) = self.project.find_layer(hit_id) {
                                self.drag = Some(DragMode::Move {
                                    start_pointer: pointer,
                                    start_xy: (layer.transform.x, layer.transform.y),
                                });
                            }
                        }
                    }
                }
                self.drag_snapshot_taken = false;
            }
        }

        if response.dragged() {
            if !self.drag_snapshot_taken {
                self.history.push(self.project.clone());
                self.drag_snapshot_taken = true;
            }
            if let Some(pointer) = response.interact_pointer_pos() {
                let sel_id = self.project.selected_layer;
                if let (Some(id), Some(drag)) = (sel_id, &self.drag) {
                    match drag {
                        DragMode::Move { start_pointer, start_xy } => {
                            let start_proj = to_project(*start_pointer);
                            let cur_proj = to_project(pointer);
                            let delta = cur_proj - start_proj;
                            let mut new_x = start_xy.0 + delta.x;
                            let mut new_y = start_xy.1 + delta.y;

                            let (lw, lh) = self
                                .project
                                .find_layer(id)
                                .map(|l| (l.transform.width, l.transform.height))
                                .unwrap_or((0.0, 0.0));
                            let (xs, ys) = self.snap_candidates(id);
                            const THRESHOLD: f32 = 8.0;
                            let mut guide_x: Option<f32> = None;
                            let mut guide_y: Option<f32> = None;

                            for offset in [0.0, lw / 2.0, lw] {
                                let edge = new_x + offset;
                                if let Some(&closest) =
                                    xs.iter().min_by(|a, b| (**a - edge).abs().total_cmp(&(**b - edge).abs()))
                                {
                                    if (closest - edge).abs() < THRESHOLD {
                                        new_x = closest - offset;
                                        guide_x = Some(closest);
                                        break;
                                    }
                                }
                            }
                            for offset in [0.0, lh / 2.0, lh] {
                                let edge = new_y + offset;
                                if let Some(&closest) =
                                    ys.iter().min_by(|a, b| (**a - edge).abs().total_cmp(&(**b - edge).abs()))
                                {
                                    if (closest - edge).abs() < THRESHOLD {
                                        new_y = closest - offset;
                                        guide_y = Some(closest);
                                        break;
                                    }
                                }
                            }

                            if let Some(gx) = guide_x {
                                let sx = rect.min.x + to_screen(gx);
                                ui.painter().line_segment(
                                    [egui::pos2(sx, rect.min.y), egui::pos2(sx, rect.max.y)],
                                    egui::Stroke::new(1.5f32, egui::Color32::from_rgb(255, 60, 170)),
                                );
                            }
                            if let Some(gy) = guide_y {
                                let sy = rect.min.y + to_screen(gy);
                                ui.painter().line_segment(
                                    [egui::pos2(rect.min.x, sy), egui::pos2(rect.max.x, sy)],
                                    egui::Stroke::new(1.5f32, egui::Color32::from_rgb(255, 60, 170)),
                                );
                            }

                            let linked_photo = self.linked_screen_photo(id);

                            if let Some(layer) = self.project.find_layer_mut(id) {
                                layer.transform.x = new_x;
                                layer.transform.y = new_y;
                            }
                            if let Some(photo_id) = linked_photo {
                                // Recompute the inset screen rect from the frame's *new*
                                // position (width/height unchanged by a move) so the photo
                                // stays inside the frame's screen area, not stretched to
                                // its full outer bounds.
                                let kind = self.frame_kind(id);
                                let inset = self
                                    .project
                                    .find_layer(id)
                                    .map(|l| l.transform.clone())
                                    .map(|t| templates::inset_screen_rect(&t, kind));
                                if let (Some(photo), Some((ix, iy, iw, ih))) =
                                    (self.project.find_layer_mut(photo_id), inset)
                                {
                                    photo.transform.x = ix;
                                    photo.transform.y = iy;
                                    photo.transform.width = iw;
                                    photo.transform.height = ih;
                                }
                            }
                        }
                        DragMode::MoveGroup { start_pointer, starts } => {
                            let start_proj = to_project(*start_pointer);
                            let cur_proj = to_project(pointer);
                            let delta = cur_proj - start_proj;
                            for &(gid, sx, sy) in starts {
                                let new_x = sx + delta.x;
                                let new_y = sy + delta.y;
                                let linked_photo = self.linked_screen_photo(gid);
                                if let Some(layer) = self.project.find_layer_mut(gid) {
                                    layer.transform.x = new_x;
                                    layer.transform.y = new_y;
                                }
                                if let Some(photo_id) = linked_photo {
                                    let kind = self.frame_kind(gid);
                                    let inset = self
                                        .project
                                        .find_layer(gid)
                                        .map(|l| l.transform.clone())
                                        .map(|t| templates::inset_screen_rect(&t, kind));
                                    if let (Some(photo), Some((ix, iy, iw, ih))) =
                                        (self.project.find_layer_mut(photo_id), inset)
                                    {
                                        photo.transform.x = ix;
                                        photo.transform.y = iy;
                                        photo.transform.width = iw;
                                        photo.transform.height = ih;
                                    }
                                }
                            }
                        }
                        DragMode::ResizeCorner { corner, start_pointer, start_rect } => {
                            let start_proj = to_project(*start_pointer);
                            let cur_proj = to_project(pointer);
                            let delta = cur_proj - start_proj;
                            let (sx, sy, sw, sh) = *start_rect;

                            const MIN_SIZE: f32 = 10.0;
                            let (new_x, new_y, new_w, new_h) = match corner {
                                Corner::BottomRight => (sx, sy, (sw + delta.x).max(MIN_SIZE), (sh + delta.y).max(MIN_SIZE)),
                                Corner::BottomLeft => {
                                    let w = (sw - delta.x).max(MIN_SIZE);
                                    (sx + sw - w, sy, w, (sh + delta.y).max(MIN_SIZE))
                                }
                                Corner::TopRight => {
                                    let h = (sh - delta.y).max(MIN_SIZE);
                                    (sx, sy + sh - h, (sw + delta.x).max(MIN_SIZE), h)
                                }
                                Corner::TopLeft => {
                                    let w = (sw - delta.x).max(MIN_SIZE);
                                    let h = (sh - delta.y).max(MIN_SIZE);
                                    (sx + sw - w, sy + sh - h, w, h)
                                }
                            };

                            let linked_photo = self.linked_screen_photo(id);

                            if let Some(layer) = self.project.find_layer_mut(id) {
                                layer.transform.x = new_x;
                                layer.transform.y = new_y;
                                layer.transform.width = new_w;
                                layer.transform.height = new_h;
                            }
                            if let Some(photo_id) = linked_photo {
                                // The frame's transform was just updated above, so re-reading
                                // it here gives the new (x, y, w, h) to inset the photo from.
                                let kind = self.frame_kind(id);
                                let inset = self
                                    .project
                                    .find_layer(id)
                                    .map(|l| l.transform.clone())
                                    .map(|t| templates::inset_screen_rect(&t, kind));
                                if let (Some(photo), Some((ix, iy, iw, ih))) =
                                    (self.project.find_layer_mut(photo_id), inset)
                                {
                                    photo.transform.x = ix;
                                    photo.transform.y = iy;
                                    photo.transform.width = iw;
                                    photo.transform.height = ih;
                                }
                            }
                        }
                        DragMode::Rotate { start_pointer_angle, start_rotation } => {
                            let (cx, cy) = self
                                .project
                                .find_layer(id)
                                .map(|l| {
                                    (
                                        l.transform.x + l.transform.width / 2.0,
                                        l.transform.y + l.transform.height / 2.0,
                                    )
                                })
                                .unwrap_or((0.0, 0.0));
                            let center = rect.min + egui::vec2(cx * scale, cy * scale);
                            let d = pointer - center;
                            let angle = d.y.atan2(d.x).to_degrees() + 90.0;
                            let mut new_rotation = start_rotation + (angle - start_pointer_angle);
                            while new_rotation > 180.0 {
                                new_rotation -= 360.0;
                            }
                            while new_rotation < -180.0 {
                                new_rotation += 360.0;
                            }
                            // Hold Shift to snap to 15-degree increments.
                            if ui.input(|i| i.modifiers.shift) {
                                new_rotation = (new_rotation / 15.0).round() * 15.0;
                            }

                            let linked_photo = self.linked_screen_photo(id);
                            if let Some(layer) = self.project.find_layer_mut(id) {
                                layer.transform.rotation_deg = new_rotation;
                            }
                            if let Some(photo_id) = linked_photo {
                                if let Some(photo) = self.project.find_layer_mut(photo_id) {
                                    photo.transform.rotation_deg = new_rotation;
                                }
                            }
                        }
                    }
                    self.mark_dirty();
                }
            }
        }

        if response.drag_stopped() {
            self.drag = None;
            self.drag_snapshot_taken = false;
            self.mark_dirty(); // re-render at full preview quality now that dragging has stopped
        }

        if response.double_clicked() {
            if let Some(pointer) = response.interact_pointer_pos() {
                let p = to_project(pointer);
                if let Some(hit_id) = self.hit_test_layer(p) {
                    if let Some(layer) = self.project.find_layer(hit_id) {
                        if let LayerKind::Text(text) = &layer.kind {
                            self.editing_before = Some(self.project.clone());
                            self.editing_buffer = text.content.clone();
                            self.editing_text = Some(hit_id);
                            self.editing_focus_pending = true;
                            self.select_only(hit_id);
                        } else {
                            self.pick_image_for_layer(hit_id);
                        }
                    }
                }
            }
        } else if response.clicked() && !response.dragged() {
            if let Some(pointer) = response.interact_pointer_pos() {
                let p = to_project(pointer);
                let hit = self.hit_test_layer(p);
                let additive = ui.input(|i| i.modifiers.shift || i.modifiers.command || i.modifiers.ctrl);
                match hit {
                    Some(id) if additive => self.toggle_selection(id),
                    Some(id) => self.select_only(id),
                    None if !additive => self.clear_selection(),
                    None => {}
                }
            }
        }
    }

    /// Canvas-space x/y candidates (canvas edges/center plus every other
    /// visible layer's edges and center) to snap the dragged layer against.
    fn snap_candidates(&self, exclude_id: u64) -> (Vec<f32>, Vec<f32>) {
        let mut xs = vec![0.0, self.project.canvas_width as f32 / 2.0, self.project.canvas_width as f32];
        let mut ys = vec![0.0, self.project.canvas_height as f32 / 2.0, self.project.canvas_height as f32];
        for layer in &self.project.layers {
            if layer.id == exclude_id || !layer.visible {
                continue;
            }
            let t = &layer.transform;
            xs.push(t.x);
            xs.push(t.x + t.width / 2.0);
            xs.push(t.x + t.width);
            ys.push(t.y);
            ys.push(t.y + t.height / 2.0);
            ys.push(t.y + t.height);
        }
        (xs, ys)
    }

    fn hit_test_layer(&self, p: egui::Pos2) -> Option<u64> {
        for layer in self.project.layers.iter().rev() {
            if !layer.visible {
                continue;
            }
            let t = &layer.transform;
            if p.x >= t.x && p.x <= t.x + t.width && p.y >= t.y && p.y <= t.y + t.height {
                return Some(layer.id);
            }
        }
        None
    }

    /// Keeps a device frame's linked screen photo in lockstep with any
    /// property-panel transform edit (position, size, rotation, mirror) —
    /// the same sync that dragging already does on the canvas. The photo is
    /// kept inset inside the frame's screen area, not stretched to its full
    /// outer bounds, so the frame always reads as visibly bigger.
    fn sync_linked_photo_transform(&mut self, id: u64) {
        let Some(photo_id) = self.linked_screen_photo(id) else { return };
        let Some(frame_t) = self.project.find_layer(id).map(|l| l.transform.clone()) else { return };
        let (ix, iy, iw, ih) = templates::inset_screen_rect(&frame_t, self.frame_kind(id));
        if let Some(photo) = self.project.find_layer_mut(photo_id) {
            photo.transform.x = ix;
            photo.transform.y = iy;
            photo.transform.width = iw;
            photo.transform.height = ih;
            photo.transform.rotation_deg = frame_t.rotation_deg;
            photo.transform.mirror_h = frame_t.mirror_h;
            photo.transform.mirror_v = frame_t.mirror_v;
        }
    }

    /// Returns the screen-photo layer id for `id` if it's a device frame with
    /// one attached; used to move/resize the two together.
    fn linked_screen_photo(&self, id: u64) -> Option<u64> {
        let is_frame = matches!(self.project.find_layer(id).map(|l| &l.kind), Some(LayerKind::DeviceFrame(_)));
        if is_frame {
            self.find_screen_image_for_frame(id)
        } else {
            None
        }
    }

    /// A frame layer's screen-inset kind (phone vs. laptop proportions);
    /// defaults to Phone if `id` isn't a device frame.
    fn frame_kind(&self, id: u64) -> FrameKind {
        match self.project.find_layer(id).map(|l| &l.kind) {
            Some(LayerKind::DeviceFrame(f)) => f.kind,
            _ => FrameKind::Phone,
        }
    }

    /// Finds the Image layer linked to a device frame, i.e. the photo already
    /// sitting behind that frame's screen.
    fn find_screen_image_for_frame(&self, frame_id: u64) -> Option<u64> {
        self.project
            .layers
            .iter()
            .find(|l| matches!(&l.kind, LayerKind::Image(img) if img.linked_frame_id == Some(frame_id)))
            .map(|l| l.id)
    }

    /// Click-to-replace: double-clicking a device frame's screen (or an image
    /// layer directly) opens a native file picker and drops the chosen photo
    /// straight in, matching the frame's screen bounds if applicable.
    fn pick_image_for_layer(&mut self, hit_id: u64) {
        let Some(hit_layer) = self.project.find_layer(hit_id) else { return };

        match &hit_layer.kind {
            LayerKind::Image(_) => {
                let (tw, th) = (hit_layer.transform.width, hit_layer.transform.height);
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg"])
                    .pick_file()
                {
                    let path_str = path.to_string_lossy().to_string();
                    let crop = templates::aspect_crop(&path_str, tw, th);
                    let before = self.begin_edit();
                    if let Some(LayerKind::Image(img)) =
                        self.project.find_layer_mut(hit_id).map(|l| &mut l.kind)
                    {
                        img.path = path_str;
                        img.crop = crop;
                    }
                    self.select_only(hit_id);
                    self.commit_edit(before);
                }
            }
            LayerKind::DeviceFrame(frame) => {
                let frame_transform = hit_layer.transform.clone();
                let frame_kind = frame.kind;
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg"])
                    .pick_file()
                {
                    let path_str = path.to_string_lossy().to_string();
                    let (_, _, inset_w, inset_h) = templates::inset_screen_rect(&frame_transform, frame_kind);
                    let crop = templates::aspect_crop(&path_str, inset_w, inset_h);
                    let before = self.begin_edit();
                    if let Some(existing_id) = self.find_screen_image_for_frame(hit_id) {
                        if let Some(LayerKind::Image(img)) =
                            self.project.find_layer_mut(existing_id).map(|l| &mut l.kind)
                        {
                            img.path = path_str;
                            img.crop = crop;
                        }
                        self.select_only(existing_id);
                    } else {
                        let new_id = self.project.alloc_id();
                        let new_layer =
                            templates::screen_photo_layer(new_id, hit_id, &frame_transform, frame_kind, &path_str);
                        let frame_idx = self.project.layers.iter().position(|l| l.id == hit_id).unwrap_or(0);
                        self.project.layers.insert(frame_idx, new_layer);
                        self.select_only(new_id);
                    }
                    self.commit_edit(before);
                }
            }
            _ => {}
        }
    }
}

/// Forces a resizable `SidePanel`'s persisted width, so a "Compact Panels"
/// action can snap both side panels to their minimum width in one click
/// instead of requiring the user to drag each boundary by hand.
fn set_panel_width(ctx: &egui::Context, id_str: &str, width: f32) {
    let id = egui::Id::new(id_str);
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 100.0));
    ctx.data_mut(|d| d.insert_persisted(id, egui::containers::panel::PanelState { rect }));
}

fn scaled_project(project: &Project, scale: f32) -> Project {
    let mut p = project.clone();
    p.canvas_width = (project.canvas_width as f32 * scale).round() as u32;
    p.canvas_height = (project.canvas_height as f32 * scale).round() as u32;
    for layer in &mut p.layers {
        let t = &mut layer.transform;
        t.x *= scale;
        t.y *= scale;
        t.width *= scale;
        t.height *= scale;
        t.shadow.blur *= scale;
        t.shadow.offset_x *= scale;
        t.shadow.offset_y *= scale;
        t.corner_radius *= scale;
        if let LayerKind::Text(text) = &mut layer.kind {
            text.font_size *= scale;
            text.letter_spacing *= scale;
        }
    }
    p
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(if self.dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() });

        // Global Ctrl/Cmd+Z (undo) and Ctrl/Cmd+Shift+Z or Ctrl+Y (redo), but
        // only when no text field (inline text edit, layer name, etc.) has
        // keyboard focus — those get their own built-in undo instead.
        if self.screen == Screen::Editor && self.editing_text.is_none() {
            let no_field_focused = ctx.memory(|m| m.focused().is_none());
            if no_field_focused {
                let (undo, redo) = ctx.input(|i| {
                    let cmd = i.modifiers.command || i.modifiers.ctrl;
                    let z = i.key_pressed(egui::Key::Z);
                    let y = i.key_pressed(egui::Key::Y);
                    (cmd && z && !i.modifiers.shift, cmd && ((z && i.modifiers.shift) || y))
                });
                if undo {
                    self.do_undo();
                } else if redo {
                    self.do_redo();
                }
            }
        }

        if self.screen == Screen::Editor && self.fullscreen_preview {
            self.ui_fullscreen_preview(ctx);
            return;
        }

        match self.screen {
            Screen::Gallery => {
                self.ensure_gallery(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.ui_gallery(ui);
                });
            }
            Screen::Editor => {
                self.update_texture(ctx);

                egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
                    self.ui_toolbar(ui);
                });

                if self.show_layers_panel {
                    egui::SidePanel::left("layers")
                        .resizable(true)
                        .default_width(220.0)
                        .width_range(110.0..=400.0)
                        .show(ctx, |ui| {
                            self.ui_layers_panel(ui);
                        });
                }

                if self.show_properties_panel {
                    egui::SidePanel::right("properties")
                        .resizable(true)
                        .default_width(280.0)
                        .width_range(150.0..=480.0)
                        .show(ctx, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                self.ui_properties_panel(ui);
                            });
                        });
                }

                // A small inner margin keeps the canvas's own drag-sense area
                // (which otherwise starts flush at the panel boundary) from
                // overlapping the side panels' resize-handle strip, which
                // straddles that boundary and would otherwise lose the drag
                // to the canvas widget on top of it.
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(8.0))
                    .show(ctx, |ui| {
                        self.ui_canvas(ui);
                    });
            }
        }
    }
}
