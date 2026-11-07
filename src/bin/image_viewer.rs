use std::fs;
use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke, StrokeKind};

const BG: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x2E);
const ACCENT: Color32 = Color32::from_rgb(0x89, 0xB4, 0xFA);
const BORDER: Color32 = Color32::from_rgb(0x45, 0x47, 0x5A);

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Mini Apps — Image Viewer & Categorizer")
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([680.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Image Viewer & Categorizer",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
            Ok(Box::<ImageViewer>::default())
        }),
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Category {
    #[default]
    Uncategorized,
    Photos,
    Screenshots,
    Artwork,
    Documents,
    Favorites,
}

impl Category {
    const ALL: [Self; 6] = [
        Self::Uncategorized,
        Self::Photos,
        Self::Screenshots,
        Self::Artwork,
        Self::Documents,
        Self::Favorites,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Uncategorized => "Uncategorized",
            Self::Photos => "Photos",
            Self::Screenshots => "Screenshots",
            Self::Artwork => "Artwork",
            Self::Documents => "Documents",
            Self::Favorites => "Favorites",
        }
    }
}

struct Entry {
    path: PathBuf,
    category: Category,
}

#[derive(Default)]
struct ImageViewer {
    folder: Option<PathBuf>,
    entries: Vec<Entry>,
    selected: Option<usize>,
    category_filter: Option<Category>,
    search: String,
    preview_path: Option<PathBuf>,
    preview: Option<egui::ColorImage>,
    texture: Option<egui::TextureHandle>,
    status: Option<(bool, String)>,
}

impl ImageViewer {
    fn open_folder(&mut self, ctx: &egui::Context) {
        if let Some(folder) = rfd::FileDialog::new()
            .set_title("Choose a folder containing images")
            .pick_folder()
        {
            self.load_folder(folder, ctx);
        }
    }

    fn load_folder(&mut self, folder: PathBuf, ctx: &egui::Context) {
        self.entries = scan_folder(&folder)
            .into_iter()
            .map(|path| Entry { path, category: Category::Uncategorized })
            .collect();
        self.folder = Some(folder);
        self.category_filter = None;
        self.search.clear();
        self.selected = (!self.entries.is_empty()).then_some(0);
        self.preview_path = None;
        self.preview = None;
        self.texture = None;
        self.load_preview(ctx);
        self.status = Some((false, format!("Found {} images", self.entries.len())));
    }

    fn load_preview(&mut self, _ctx: &egui::Context) {
        let Some(index) = self.selected else {
            self.preview_path = None;
            self.preview = None;
            self.texture = None;
            return;
        };
        let Some(path) = self.entries.get(index).map(|entry| entry.path.clone()) else {
            return;
        };
        if self.preview_path.as_ref() == Some(&path) {
            return;
        }
        match image::open(&path) {
            Ok(image) => {
                let rgba = image.to_rgba8();
                let (width, height) = rgba.dimensions();
                self.preview = Some(egui::ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize], rgba.as_raw(),
                ));
                self.preview_path = Some(path.clone());
                self.texture = None;
                self.status = Some((false, format!("{} × {}", width, height)));
            }
            Err(error) => {
                self.preview = None;
                self.preview_path = Some(path.clone());
                self.texture = None;
                self.status = Some((true, format!("Could not decode {}: {error}", file_name(&path))));
            }
        }
    }

    fn visible(&self, index: usize) -> bool {
        let entry = &self.entries[index];
        let name = file_name(&entry.path).to_lowercase();
        let query = self.search.trim().to_lowercase();
        self.category_filter.map_or(true, |category| category == entry.category)
            && (query.is_empty() || name.contains(&query))
    }

    fn set_category(&mut self, category: Category) {
        if let Some(entry) = self.selected.and_then(|index| self.entries.get_mut(index)) {
            entry.category = category;
            self.status = Some((false, format!("{} categorized as {}", file_name(&entry.path), category.label())));
        }
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        self.load_preview(ui.ctx());
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.heading("Image Viewer & Categorizer");
            ui.label(RichText::new("Browse and preview local images; organize them with session-only categories.").color(Color32::LIGHT_GRAY));
            ui.add_space(12.0);

            ui.horizontal_wrapped(|ui| {
                if ui.button("Open Folder…").clicked() { self.open_folder(ui.ctx()); }
                if ui.button("Refresh").clicked() {
                    if let Some(folder) = self.folder.clone() { self.load_folder(folder, ui.ctx()); }
                }
                if let Some(folder) = &self.folder {
                    ui.label(RichText::new(folder.display().to_string()).small().color(Color32::LIGHT_GRAY));
                }
            });
            ui.add_space(8.0);

            if self.folder.is_none() {
                ui.label("Choose a folder to get started. Categories are kept in memory and reset when the folder is reloaded.");
                if let Some((error, message)) = &self.status { ui.colored_label(if *error { Color32::LIGHT_RED } else { ACCENT }, message); }
                return;
            }

            ui.horizontal_wrapped(|ui| {
                if ui.selectable_label(self.category_filter.is_none(), "All").clicked() { self.category_filter = None; }
                for category in Category::ALL {
                    if ui.selectable_label(self.category_filter == Some(category), category.label()).clicked() {
                        self.category_filter = Some(category);
                    }
                }
            });
            ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search image names…").desired_width(f32::INFINITY));
            ui.add_space(8.0);

            ui.columns(2, |columns| {
                egui::ScrollArea::vertical().max_height(450.0).show(&mut columns[0], |ui| {
                    let mut selection = None;
                    for index in 0..self.entries.len() {
                        if !self.visible(index) { continue; }
                        let entry = &self.entries[index];
                        let label = format!("{}\n{}", file_name(&entry.path), entry.category.label());
                        if ui.selectable_label(self.selected == Some(index), label).clicked() { selection = Some(index); }
                    }
                    if let Some(index) = selection { self.selected = Some(index); self.load_preview(ui.ctx()); }
                });

                let selected = self.selected.filter(|index| *index < self.entries.len());
                if let Some(index) = selected {
                    let path = self.entries[index].path.clone();
                    let current_category = self.entries[index].category;
                    columns[1].vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong(file_name(&path));
                            if ui.button("Open externally").clicked() {
                                match std::process::Command::new("xdg-open").arg(&path).spawn() {
                                    Ok(_) => self.status = Some((false, "Opened with the system image viewer".into())),
                                    Err(error) => self.status = Some((true, format!("Could not launch xdg-open: {error}"))),
                                }
                            }
                        });
                        ui.label(RichText::new(path.display().to_string()).small().color(Color32::LIGHT_GRAY));
                        ui.add_space(8.0);
                        if let Some(image) = self.preview.clone() {
                            let recreate = self.texture.as_ref().map_or(true, |texture| texture.size() != image.size);
                            if recreate { self.texture = Some(ui.ctx().load_texture("image-viewer", image.clone(), egui::TextureOptions::LINEAR)); }
                            if let Some(texture) = &self.texture {
                                let available = ui.available_size();
                                let scale = (available.x / image.size[0] as f32).min((available.y / image.size[1] as f32).min(1.0)).max(0.01);
                                let size = egui::vec2(image.size[0] as f32 * scale, image.size[1] as f32 * scale);
                                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                                ui.painter().image(texture.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                                ui.painter().rect(rect, CornerRadius::same(5), Color32::TRANSPARENT, Stroke::new(1.0, BORDER), StrokeKind::Inside);
                            }
                        } else {
                            ui.label("Preview unavailable. Try another image or refresh the folder.");
                        }
                        ui.add_space(8.0);
                        ui.label(RichText::new("Categorize this image").strong());
                        ui.horizontal_wrapped(|ui| {
                            for category in Category::ALL {
                                if ui.selectable_label(current_category == category, category.label()).clicked() {
                                    self.set_category(category);
                                }
                            }
                        });
                    });
                } else {
                    columns[1].label("Select an image to preview it.");
                }
            });
            if let Some((error, message)) = &self.status {
                ui.add_space(6.0);
                ui.colored_label(if *error { Color32::LIGHT_RED } else { ACCENT }, message);
            }
        });
    }
}

impl eframe::App for ImageViewer {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::central_panel(ui.style()).fill(BG).inner_margin(egui::Margin::same(20)).show(ui, |ui| self.render(ui));
    }
}

fn scan_folder(folder: &Path) -> Vec<PathBuf> {
    let Ok(items) = fs::read_dir(folder) else { return Vec::new() };
    let mut paths: Vec<_> = items.flatten().map(|item| item.path())
        .filter(|path| path.is_file() && supported(path))
        .collect();
    paths.sort_by_key(|path| file_name(path).to_lowercase());
    paths
}

fn supported(path: &Path) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg")
}

fn file_name(path: &Path) -> &str {
    path.file_name().and_then(|name| name.to_str()).unwrap_or("(non-UTF-8 filename)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_supported_image_extensions_case_insensitively() {
        assert!(supported(Path::new("photo.PNG")));
        assert!(supported(Path::new("scan.jpeg")));
        assert!(!supported(Path::new("notes.txt")));
    }

    #[test]
    fn categories_have_clear_labels() {
        assert_eq!(Category::Uncategorized.label(), "Uncategorized");
        assert_eq!(Category::Favorites.label(), "Favorites");
    }
}
