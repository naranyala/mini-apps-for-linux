use std::fs;
use std::path::{Path, PathBuf};

use egui::{
    Align2, Button, Color32, CornerRadius, FontId, Frame, Label, Margin, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, TextEdit,
};

use crate::{
    card_frame, card_title, header_impl, status_pill, ACCENT, BG, BORDER, GREEN, RED, SUBTEXT,
    TEXT,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ImageCategory {
    Uncategorized,
    Photos,
    Screenshots,
    Artwork,
    Documents,
    Favorites,
}

impl Default for ImageCategory {
    fn default() -> Self {
        Self::Uncategorized
    }
}

impl ImageCategory {
    pub fn all() -> [Self; 6] {
        [
            Self::Uncategorized,
            Self::Photos,
            Self::Screenshots,
            Self::Artwork,
            Self::Documents,
            Self::Favorites,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Uncategorized => "Uncategorized",
            Self::Photos => "Photos",
            Self::Screenshots => "Screenshots",
            Self::Artwork => "Artwork",
            Self::Documents => "Documents",
            Self::Favorites => "Favorites",
        }
    }

    pub fn color(self) -> Color32 {
        match self {
            Self::Uncategorized => BORDER,
            Self::Photos => ACCENT,
            Self::Screenshots => GREEN,
            Self::Artwork => Color32::from_rgb(0xFF, 0xA5, 0xC7),
            Self::Documents => Color32::from_rgb(0x8B, 0xC7, 0xFF),
            Self::Favorites => RED,
        }
    }
}

#[derive(Clone)]
pub struct ImageEntry {
    pub path: String,
    pub file_name: String,
    pub category: ImageCategory,
}

#[derive(Default)]
pub struct ImageViewerApp {
    folder: Option<String>,
    entries: Vec<ImageEntry>,
    selected: Option<usize>,
    filter: String,
    active_category: Option<ImageCategory>,
    preview_image: Option<egui::ColorImage>,
    preview_texture: Option<egui::TextureHandle>,
    status: crate::Status,
}

impl ImageViewerApp {
    pub fn panel(&mut self, ui: &mut egui::Ui) {
        self.ensure_preview(ui.ctx());

        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            header_impl(
                ui,
                "Image Viewer & Categorizer",
                "Browse a folder of images, preview them, and tag them by category",
            );

            ui.horizontal(|ui| {
                if ui.button("Open Folder").clicked() {
                    self.load_folder(ui.ctx());
                }
                if ui.button("Refresh").clicked() {
                    if let Some(folder) = self.folder.clone() {
                        self.refresh_folder(ui.ctx(), &folder);
                    }
                }
                if ui.button("Clear").clicked() {
                    self.entries.clear();
                    self.selected = None;
                    self.preview_image = None;
                    self.preview_texture = None;
                    self.folder = None;
                    self.status = Some((false, "Image library cleared".into()));
                }
            });
            ui.add_space(8.0);

            if self.folder.is_none() {
                ui.add(
                    Label::new(
                        RichText::new(
                            "No image folder selected yet — press \u{201C}Open Folder\u{201D} and choose a directory.",
                        )
                        .weak()
                        .monospace(),
                    )
                    .wrap(),
                );
                ui.add_space(12.0);
                status_pill(ui, self.status.clone());
                return;
            }

            ui.horizontal_wrapped(|ui| {
                if ui.button("All").clicked() {
                    self.active_category = None;
                }
                for category in ImageCategory::all() {
                    let selected = self.active_category == Some(category);
                    let button = ui.add_sized(
                        [110.0, 28.0],
                        egui::Button::new(category.label()).fill(if selected {
                            category.color().gamma_multiply(0.18)
                        } else {
                            BG
                        }),
                    );
                    if button.clicked() {
                        self.active_category = Some(category);
                    }
                }
            });
            ui.add_space(8.0);

            ui.add(
                TextEdit::singleline(&mut self.filter)
                    .hint_text("Search by name…")
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(12.0);

            let visible = self.visible_indices();
            if visible.is_empty() {
                ui.add(
                    Label::new(
                        RichText::new("No images match the current filter or category.")
                            .weak()
                            .monospace(),
                    )
                    .wrap(),
                );
            } else {
                ui.columns(2, |columns| {
                    let left = &mut columns[0];
                    left.set_min_width(220.0);
                    egui::ScrollArea::vertical().max_height(420.0).show(left, |ui| {
                        for idx in &visible {
                            let entry = &self.entries[*idx];
                            let selected = self.selected == Some(*idx);
                            let row = egui::Frame::new()
                                .fill(if selected {
                                    ACCENT.gamma_multiply(0.14)
                                } else {
                                    BG.gamma_multiply(0.55)
                                })
                                .stroke(Stroke::new(1.0, if selected { ACCENT } else { BORDER }))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(8));
                            row.show(ui, |ui| {
                                if ui
                                    .selectable_label(selected, entry.file_name.as_str())
                                    .clicked()
                                {
                                    self.selected = Some(*idx);
                                    self.ensure_preview(ui.ctx());
                                }
                                ui.add_space(3.0);
                                ui.horizontal(|ui| {
                                    ui.add(
                                        Label::new(
                                            RichText::new(entry.category.label())
                                                .small()
                                                .strong()
                                                .color(entry.category.color()),
                                        )
                                        .sense(Sense::hover()),
                                    );
                                });
                            });
                            ui.add_space(6.0);
                        }
                    });

                    let right = &mut columns[1];
                    if let Some(selected_idx) = self.selected {
                        if let Some(entry) = self.entries.get(selected_idx) {
                            let preview_label = format!(
                                "{} · {}",
                                entry.category.label(),
                                pretty_path(&entry.path)
                            );
                            right.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        Label::new(
                                            RichText::new(preview_label)
                                                .strong()
                                                .color(TEXT)
                                                .small(),
                                        )
                                        .sense(Sense::hover()),
                                    );
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("Open").clicked() {
                                            self.open_selected();
                                        }
                                    });
                                });
                                ui.add_space(10.0);

                                if let Some(image) = self.preview_image.clone() {
                                    if self
                                        .preview_texture
                                        .as_ref()
                                        .map_or(true, |tex| tex.size() != image.size)
                                    {
                                        self.preview_texture = Some(
                                            ui.ctx().load_texture(
                                                "image-viewer-preview",
                                                image.clone(),
                                                egui::TextureOptions::LINEAR,
                                            ),
                                        );
                                    }
                                    if let Some(tex) = &self.preview_texture {
                                        let max_size = ui.available_size();
                                        let scale = fit_image_size(&image.size, max_size);
                                        let draw_size = egui::vec2(
                                            image.size[0] as f32 * scale,
                                            image.size[1] as f32 * scale,
                                        );
                                        let (rect, _) = ui.allocate_exact_size(draw_size, Sense::hover());
                                        ui.painter().image(
                                            tex.id(),
                                            rect,
                                            egui::Rect::from_min_max(
                                                egui::pos2(0.0, 0.0),
                                                egui::pos2(1.0, 1.0),
                                            ),
                                            Color32::WHITE,
                                        );
                                        ui.painter().rect(
                                            rect,
                                            CornerRadius::same(6),
                                            Color32::TRANSPARENT,
                                            Stroke::new(1.0, BORDER),
                                            StrokeKind::Inside,
                                        );
                                    }
                                } else {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new("Preview unavailable — image could not be decoded.")
                                                .weak()
                                                .monospace(),
                                        )
                                        .wrap(),
                                    );
                                }

                                ui.add_space(12.0);
                                ui.label(
                                    RichText::new(format!(
                                        "Path: {}",
                                        entry.path
                                    ))
                                        .small()
                                        .monospace()
                                        .color(SUBTEXT),
                                );
                                ui.add_space(6.0);
                                ui.horizontal_wrapped(|ui| {
                                    for category in ImageCategory::all() {
                                        let active = entry.category == category;
                                        if ui
                                            .add_sized(
                                                [86.0, 28.0],
                                                egui::Button::new(category.label()).fill(if active {
                                                    category.color().gamma_multiply(0.20)
                                                } else {
                                                    BG
                                                }),
                                            )
                                            .clicked()
                                        {
                                            self.set_selected_category(category);
                                        }
                                    }
                                });
                            });
                        }
                    } else {
                        right.label(
                            RichText::new("Select an image to preview it.").weak().monospace(),
                        );
                    }
                });
            }

            status_pill(ui, self.status.clone());
        });
    }

    fn visible_indices(&self) -> Vec<usize> {
        let query = self.filter.trim().to_lowercase();
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                let matches_filter = query.is_empty()
                    || entry.file_name.to_lowercase().contains(&query)
                    || entry.path.to_lowercase().contains(&query);
                let matches_category = self
                    .active_category
                    .map_or(true, |category| entry.category == category);
                matches_filter && matches_category then Some(idx) else None
            })
            .collect()
    }

    fn load_folder(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select image folder")
            .pick_folder()
        {
            self.refresh_folder(ctx, &path.display().to_string());
        }
    }

    fn refresh_folder(&mut self, ctx: &egui::Context, folder: &str) {
        let path = PathBuf::from(folder);
        let mut entries = scan_image_folder(&path);
        entries.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
        self.folder = Some(folder.to_owned());
        self.entries = entries;
        self.filter.clear();
        self.active_category = None;
        self.selected = if self.entries.is_empty() { None } else { Some(0) };
        self.preview_image = None;
        self.preview_texture = None;
        self.status = Some((false, format!("Loaded {} images", self.entries.len())));
        self.ensure_preview(ctx);
    }

    fn ensure_preview(&mut self, ctx: &egui::Context) {
        let Some(selected) = self.selected else {
            self.preview_image = None;
            self.preview_texture = None;
            return;
        };
        if let Some(entry) = self.entries.get(selected) {
            if self.preview_image.is_some() {
                return;
            }
            match image::open(&entry.path) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let image = egui::ColorImage::from_rgba_unmultiplied(
                        [w as usize, h as usize],
                        rgba.as_raw(),
                    );
                    self.preview_image = Some(image);
                    self.preview_texture = None;
                    self.status = Some((false, format!("Preview ready: {}", entry.file_name)));
                }
                Err(err) => {
                    self.preview_image = None;
                    self.preview_texture = None;
                    self.status = Some((true, format!("Could not open {}: {err}", entry.file_name)));
                }
            }
        }
        let _ = ctx;
    }

    fn set_selected_category(&mut self, category: ImageCategory) {
        if let Some(idx) = self.selected {
            if let Some(entry) = self.entries.get_mut(idx) {
                entry.category = category;
                self.status = Some((false, format!("{} categorized as {}", entry.file_name, category.label())));
            }
        }
    }

    fn open_selected(&mut self) {
        if let Some(entry) = self.selected.and_then(|idx| self.entries.get(idx)) {
            let path = PathBuf::from(&entry.path);
            match std::process::Command::new("xdg-open").arg(&path).spawn() {
                Ok(_) => self.status = Some((false, format!("Opening {}", entry.file_name))),
                Err(err) => {
                    self.status = Some((true, format!("Failed to open {}: {err}", entry.file_name)))
                }
            }
        }
    }
}

fn scan_image_folder(path: &Path) -> Vec<ImageEntry> {
    let mut entries = Vec::new();
    if !path.exists() || !path.is_dir() {
        return entries;
    }
    if let Ok(iter) = fs::read_dir(path) {
        for item in iter.flatten() {
            let p = item.path();
            if !p.is_file() {
                continue;
            }
            let Some(ext) = p.extension().and_then(|s| s.to_str()) else {
                continue;
            };
            if !is_supported_image_extension(ext) {
                continue;
            }
            let file_name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_owned();
            entries.push(ImageEntry {
                path: p.to_string_lossy().to_string(),
                file_name,
                category: ImageCategory::Uncategorized,
            });
        }
    }
    entries.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    entries
}

fn is_supported_image_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif"
    )
}

fn pretty_path(path: &str) -> String {
    path.rsplit('/').next().filter(|p| !p.is_empty()).unwrap_or(path).to_owned()
}

fn fit_image_size(image_size: &[usize; 2], max_size: egui::Vec2) -> f32 {
    let w = image_size[0] as f32;
    let h = image_size[1] as f32;
    let scale_x = max_size.x / w.max(1.0);
    let scale_y = max_size.y / h.max(1.0);
    scale_x.min(scale_y).clamp(0.1, 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_extensions_are_recognized() {
        assert!(is_supported_image_extension("png"));
        assert!(is_supported_image_extension("JPG"));
        assert!(!is_supported_image_extension("txt"));
    }

    #[test]
    fn image_categories_have_expected_labels() {
        assert_eq!(ImageCategory::Photos.label(), "Photos");
        assert_eq!(ImageCategory::Favorites.label(), "Favorites");
    }
}

