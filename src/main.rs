use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use eframe::egui;
use egui::{
    pos2, Align, Align2, Button, Color32, CornerRadius, FontId, Frame, Label, Margin, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, TextEdit, TextStyle, TextureOptions, ViewportBuilder,
    Vec2,
};

// ─── Catppuccin-mocha-inspired palette ────────────────────────────────────────
const BG: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x2E); // base: window background
const SURFACE: Color32 = Color32::from_rgb(0x2A, 0x2B, 0x3E); // cards
const WIDGET: Color32 = Color32::from_rgb(0x36, 0x37, 0x4B); // buttons, checkbox fill
const WIDGET_HOVER: Color32 = Color32::from_rgb(0x41, 0x42, 0x57);
const FIELD_BG: Color32 = Color32::from_rgb(0x18, 0x18, 0x25); // text input background
const CODE_BG: Color32 = Color32::from_rgb(0x11, 0x11, 0x1B); // preview block
const BORDER: Color32 = Color32::from_rgb(0x45, 0x47, 0x5A);
const TEXT: Color32 = Color32::from_rgb(0xCD, 0xD6, 0xF4);
const SUBTEXT: Color32 = Color32::from_rgb(0xA6, 0xAD, 0xC8);
const ACCENT: Color32 = Color32::from_rgb(0x89, 0xB4, 0xFA); // mocha blue
const GREEN: Color32 = Color32::from_rgb(0xA6, 0xE3, 0xA1);
const RED: Color32 = Color32::from_rgb(0xF3, 0x8B, 0xA8);
const YELLOW: Color32 = Color32::from_rgb(0xF9, 0xE2, 0xAF);

const FALLBACK_ICON: &str = "utilities-terminal";
const XDG_APPS_HINT: &str =
    "Entries are written to ~/.local/share/applications/ (chmod 0755) — FreeDesktop.org specification";
const CARD_SIZE: Vec2 = Vec2::new(172.0, 148.0);
const CLIP_HISTORY_MAX: usize = 100;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Mini Apps")
            .with_inner_size([820.0, 640.0])
            .with_min_inner_size([600.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Mini Apps",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(MiniAppsApp::default()))
        }),
    )
}

/// Force a dark, polished theme with a coherent palette.
fn apply_theme(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);

    ctx.style_mut_of(egui::Theme::Dark, |style| {
        // Type scale
        style.text_styles = [
            (TextStyle::Heading, FontId::proportional(22.0)),
            (TextStyle::Body, FontId::proportional(14.0)),
            (TextStyle::Button, FontId::proportional(14.0)),
            (TextStyle::Small, FontId::proportional(10.5)),
            (TextStyle::Monospace, FontId::monospace(12.5)),
        ]
        .into();

        // Spacing & metrics
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 6.0);
        style.spacing.icon_spacing = 6.0;
        style.spacing.interact_size = egui::vec2(40.0, 28.0);

        let visuals = &mut style.visuals;
        visuals.panel_fill = BG;
        visuals.window_fill = BG;
        visuals.extreme_bg_color = FIELD_BG; // text fields
        visuals.code_bg_color = CODE_BG; // preview block
        visuals.faint_bg_color = SURFACE;
        visuals.selection.bg_fill = ACCENT.gamma_multiply(0.30);
        visuals.selection.stroke = Stroke::new(0.5, ACCENT);
        visuals.window_stroke = Stroke::new(1.0, BORDER);
        visuals.window_corner_radius = CornerRadius::same(10);
        visuals.popup_shadow = egui::epaint::Shadow::NONE;
        visuals.window_shadow = egui::epaint::Shadow::NONE;

        let set_widget = |w: &mut egui::style::WidgetVisuals,
                          bg: Color32,
                          stroke: Stroke,
                          fg: Color32,
                          radius: u8| {
            w.weak_bg_fill = bg;
            w.bg_fill = bg;
            w.bg_stroke = stroke;
            w.fg_stroke = Stroke::new(1.0, fg);
            w.corner_radius = CornerRadius::same(radius);
        };

        set_widget(&mut visuals.widgets.noninteractive, BG, Stroke::NONE, SUBTEXT, 6);
        set_widget(
            &mut visuals.widgets.inactive,
            WIDGET,
            Stroke::new(1.0, BORDER),
            TEXT,
            6,
        );
        set_widget(
            &mut visuals.widgets.hovered,
            WIDGET_HOVER,
            Stroke::new(1.0, BORDER),
            TEXT,
            6,
        );
        set_widget(
            &mut visuals.widgets.active,
            WIDGET_HOVER,
            Stroke::new(1.0, ACCENT),
            TEXT,
            6,
        );
        set_widget(
            &mut visuals.widgets.open,
            WIDGET_HOVER,
            Stroke::new(1.0, BORDER),
            TEXT,
            6,
        );
    });
}

// ═════════════════════════════════ Fuzzy search ═══════════════════════════════

/// Simple subsequence fuzzy matcher returning a relevance score.
///
/// Scoring: +1 per matched char, +4 for consecutive runs, +3 when the matched
/// char starts a word. `None` when the query is not a subsequence.
fn fuzzy_score(query: &str, target: &str) -> Option<i64> {
    let query: Vec<char> = query.to_lowercase().chars().collect();
    if query.is_empty() {
        return Some(0);
    }
    let hay = target.to_lowercase();

    let mut score = 0i64;
    let mut qi = 0usize;
    let mut prev_match: Option<usize> = None;
    let mut word_start = true;

    for (i, c) in hay.chars().enumerate() {
        let separator = c.is_whitespace() || c == '-' || c == '_' || c == '.';
        if qi < query.len() && c == query[qi] {
            score += 1;
            if i > 0 && prev_match == Some(i - 1) {
                score += 4; // consecutive characters
            }
            if word_start {
                score += 3; // beginning of a word
            }
            prev_match = Some(i);
            qi += 1;
        }
        word_start = separator;
    }

    (qi == query.len()).then_some(score)
}

// ═════════════════════════════════ Tools catalog ══════════════════════════════

#[derive(Clone, Copy, PartialEq)]
enum ToolId {
    ShortcutMaker,
    ClipboardInspector,
    ScreenshotTool,
    ColorPicker,
}

struct Tool {
    title: &'static str,
    description: &'static str,
    keywords: &'static str,
    monogram: &'static str,
    id: ToolId,
}

const TOOLS: &[Tool] = &[
    Tool {
        title: "Desktop Shortcut Maker",
        description: "Create .desktop launcher entries",
        keywords: "shortcut launcher desktop entry application menu xdg freedesktop",
        monogram: "DS",
        id: ToolId::ShortcutMaker,
    },
    Tool {
        title: "Clipboard Inspector",
        description: "Searchable history of what you copy",
        keywords: "clipboard history copy paste xclip wayland",
        monogram: "CB",
        id: ToolId::ClipboardInspector,
    },
    Tool {
        title: "Screenshot Tool",
        description: "Capture the screen or a region",
        keywords: "screenshot capture region full screen portal grim",
        monogram: "SS",
        id: ToolId::ScreenshotTool,
    },
    Tool {
        title: "Color Picker",
        description: "Sample colors off your screen as hex",
        keywords: "color picker eyedropper hex sample pixel",
        monogram: "CP",
        id: ToolId::ColorPicker,
    },
];

#[derive(Default)]
enum View {
    #[default]
    Launcher,
    Tool {
        tool_index: usize,
    },
}

#[derive(Default)]
struct MiniAppsApp {
    view: View,
    search: String,
    search_focus_done: bool,
    shortcut_maker: ShortcutMakerApp,
    clipboard: ClipboardApp,
    screenshot: ScreenshotApp,
    color_picker: ColorPickerApp,
}

impl eframe::App for MiniAppsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The bare `App::ui` frame has no margin/background, so wrap in a central panel.
        Frame::central_panel(ui.style())
            .inner_margin(Margin::same(24))
            .show(ui, |ui| {
                let tool_index = match &self.view {
                    View::Launcher => None,
                    View::Tool { tool_index } => Some(*tool_index),
                };
                match tool_index {
                    None => {
                        ScrollArea::vertical()
                            .auto_shrink(false)
                            .show(ui, |ui| self.launcher(ui));
                    }
                    Some(i) => self.tool_panel(ui, i),
                }
            });
    }
}

impl MiniAppsApp {
    // ── Launcher (grid + fuzzy search) ───────────────────────────────────────

    fn launcher(&mut self, ui: &mut egui::Ui) {
        header_impl(
            ui,
            "Mini Apps",
            "A collection of niche Linux desktop utilities",
        );
        ui.add_space(18.0);

        // ── Search field ─────────────────────────────────────────────────
        let search_resp = ui.add(
            TextEdit::singleline(&mut self.search)
                .hint_text("Search tools… (Enter opens the best match)")
                .desired_width(f32::INFINITY)
                .min_size(egui::vec2(0.0, 36.0)),
        );
        if !self.search_focus_done {
            search_resp.request_focus();
            self.search_focus_done = true;
        }
        ui.add_space(16.0);

        // ── Fuzzy filter over titles + keywords ──────────────────────────
        let query = self.search.trim();
        let mut matches: Vec<(usize, i64)> = TOOLS
            .iter()
            .enumerate()
            .filter_map(|(i, tool)| {
                let score =
                    fuzzy_score(query, tool.title).or_else(|| fuzzy_score(query, tool.keywords))?;
                Some((i, score))
            })
            .collect();
        matches.sort_by(|a, b| b.1.cmp(&a.1)); // best score first (0 = unfiltered)

        // ── Grid of tool cards ───────────────────────────────────────────
        if matches.is_empty() {
            ui.add(
                Label::new(
                    RichText::new(format!("No tools match \u{201C}{query}\u{201D}"))
                        .weak()
                        .monospace(),
                )
                .wrap(),
            );
            ui.add_space(8.0);
        } else {
            let columns = ((ui.available_width() + 18.0) / (CARD_SIZE.x + 18.0))
                .floor()
                .clamp(1.0, 4.0) as usize;
            egui::Grid::new("tool-grid")
                .num_columns(columns)
                .spacing([18.0, 18.0])
                .show(ui, |ui| {
                    for chunk in matches.chunks(columns) {
                        for &(index, _) in chunk {
                            if self.tool_card(ui, index) {
                                self.open_tool(index);
                            }
                        }
                        ui.end_row();
                    }
                });
        }

        // ── Enter opens the best match ───────────────────────────────────
        if search_resp.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
            && let Some(&(index, _)) = matches.first()
        {
            self.open_tool(index);
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add(
                Label::new(
                    RichText::new(format!("{} tools · niche Linux desktop utilities", TOOLS.len()))
                        .small()
                        .color(SUBTEXT),
                )
                .sense(Sense::hover()),
            );
        });
    }

    /// Paints one clickable tool card. Returns `true` when clicked.
    fn tool_card(&mut self, ui: &mut egui::Ui, index: usize) -> bool {
        let tool = &TOOLS[index];
        let (rect, _) = ui.allocate_exact_size(CARD_SIZE, Sense::hover());
        let resp = ui
            .interact(rect, egui::Id::unique(("tool-card", index)), Sense::click())
            .on_hover_text("Click to open");

        let hovered = resp.hovered();
        let (fill, stroke) = if hovered {
            (WIDGET_HOVER, Stroke::new(1.2, ACCENT))
        } else {
            (SURFACE, Stroke::new(1.0, BORDER))
        };
        ui.painter().rect(
            rect,
            CornerRadius::same(12),
            fill,
            stroke,
            StrokeKind::Inside,
        );

        let painter = ui.painter();
        let mut pos = rect.left_top() + egui::vec2(14.0, 14.0);

        // Monogram tile
        let tile = egui::Rect::from_min_size(pos, egui::vec2(44.0, 44.0));
        painter.rect_filled(tile, CornerRadius::same(8), ACCENT.gamma_multiply(0.22));
        painter.text(
            tile.center(),
            Align2::CENTER_CENTER,
            tool.monogram,
            FontId::proportional(17.0),
            ACCENT,
        );

        // Title
        pos.y += 44.0 + 12.0;
        painter.text(pos, Align2::LEFT_TOP, tool.title, FontId::proportional(13.5), TEXT);

        // Description
        pos.y += 20.0;
        painter.text(pos, Align2::LEFT_TOP, tool.description, FontId::proportional(10.5), SUBTEXT);

        // Breadcrumb hint
        painter.text(
            rect.right_top() - egui::vec2(10.0, -8.0),
            Align2::RIGHT_TOP,
            if hovered { "\u{2192}" } else { "" },
            FontId::proportional(16.0),
            ACCENT,
        );

        resp.clicked()
    }

    fn open_tool(&mut self, index: usize) {
        self.view = View::Tool { tool_index: index };
        self.search.clear();
        self.search_focus_done = false;
    }

    // ── Tool workspace (Back bar + dispatch) ─────────────────────────────────

    fn tool_panel(&mut self, ui: &mut egui::Ui, tool_index: usize) {
        let tool = &TOOLS[tool_index];
        let tool_id = tool.id;

        // Top bar: back navigation
        ui.horizontal(|ui| {
            if ui.button("Back").clicked()
                || ui.input(|i| i.key_pressed(egui::Key::Escape))
            {
                self.view = View::Launcher;
                self.search_focus_done = false;
            }
            ui.separator();
            ui.add(
                Label::new(
                    RichText::new(format!("{} · {}", tool.monogram, tool.title))
                        .strong()
                        .size(16.0),
                )
                .sense(Sense::hover()),
            );
        });
        ui.add_space(8.0);

        match tool_id {
            ToolId::ShortcutMaker => self.shortcut_maker_panel(ui),
            ToolId::ClipboardInspector => self.clipboard_panel(ui),
            ToolId::ScreenshotTool => self.screenshot_panel(ui),
            ToolId::ColorPicker => self.color_picker_panel(ui),
        }
    }
}

// ══════════════════════════ Tool 1: shortcut maker ════════════════════════════

impl MiniAppsApp {
    fn shortcut_maker_panel(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            header_impl(
                ui,
                "Desktop Shortcut Maker",
                "Create FreeDesktop.org\u{2013}compliant launcher entries for any binary on your system",
            );
            ui.add_space(4.0);

            let maker = &mut self.shortcut_maker;

            // ── Application Details ──────────────────────────────────────
            let width = ui.available_width();
            let app = card_frame();
            app.show(ui, |ui| {
                ui.set_min_width(width - 28.0);
                ui.vertical(|ui| {
                    card_title(ui, "Application Details");
                    ui.add_space(10.0);

                    egui::Grid::new("details")
                        .num_columns(2)
                        .spacing([10.0, 12.0])
                        .min_col_width(150.0)
                        .show(ui, |ui| {
                            form_label(ui, "Name");
                            if text_field(ui, "e.g. My Great App", &mut maker.name) {
                                maker.invalidate_preview();
                            }
                            ui.end_row();

                            form_label(ui, "Binary Location (Exec)");
                            ui.horizontal(|ui| {
                                if row_text_field(
                                    ui,
                                    "/path/to/binary",
                                    &mut maker.exec_path,
                                    88.0,
                                ) {
                                    maker.invalidate_preview();
                                }
                                browse_button(ui, || {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_title("Select Binary Executable")
                                        .pick_file()
                                    {
                                        maker.exec_path = path.display().to_string();
                                        maker.invalidate_preview();
                                    }
                                });
                            });
                            ui.end_row();

                            form_label(ui, "Icon");
                            ui.horizontal(|ui| {
                                if row_text_field(
                                    ui,
                                    "/path/to/icon.png (optional)",
                                    &mut maker.icon_path,
                                    88.0,
                                ) {
                                    maker.invalidate_preview();
                                }
                                browse_button(ui, || {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .set_title("Select Icon Image")
                                        .add_filter(
                                            "Image files",
                                            &["png", "jpg", "jpeg", "svg", "xpm"],
                                        )
                                        .pick_file()
                                    {
                                        maker.icon_path = path.display().to_string();
                                        maker.invalidate_preview();
                                    }
                                });
                            });
                            ui.end_row();

                            form_label(ui, "Description");
                            if text_field(
                                ui,
                                "A short description of the app (optional)",
                                &mut maker.description,
                            ) {
                                maker.invalidate_preview();
                            }
                            ui.end_row();
                        });

                    if maker.icon_path.trim().is_empty() {
                        ui.add_space(4.0);
                        ui.add(
                            Label::new(
                                RichText::new(format!(
                                    "No icon set — falls back to \u{201C}{FALLBACK_ICON}\u{201D}"
                                ))
                                .small()
                                .color(SUBTEXT),
                            )
                            .sense(Sense::hover()),
                        );
                    }
                });
            });
            ui.add_space(12.0);

            // ── Behavior ─────────────────────────────────────────────────
            let width = ui.available_width();
            let app = card_frame();
            app.show(ui, |ui| {
                ui.set_min_width(width - 28.0);
                ui.vertical(|ui| {
                    card_title(ui, "Behavior");
                    ui.add_space(10.0);

                    egui::Grid::new("behavior")
                        .num_columns(2)
                        .spacing([10.0, 12.0])
                        .min_col_width(150.0)
                        .show(ui, |ui| {
                            form_label(ui, "Categories");
                            if text_field(ui, "Utility;", &mut maker.categories) {
                                maker.invalidate_preview();
                            }
                            ui.end_row();

                            form_label(ui, "Startup");
                            if ui
                                .checkbox(
                                    &mut maker.launch_in_terminal,
                                    "Launch in a terminal window",
                                )
                                .changed()
                            {
                                maker.invalidate_preview();
                            }
                            ui.end_row();
                        });
                });
            });
            ui.add_space(12.0);

            // ── Actions ----------------------------------------------------
            let ready = maker.inputs_valid();
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                let create = ui
                    .add_enabled(ready, Button::new("Create Shortcut").fill(ACCENT))
                    .on_disabled_hover_text("Application Name and Binary Location are required");
                if create.clicked() {
                    maker.create_desktop();
                }

                let preview = ui
                    .add_enabled(ready, Button::new("Preview .desktop"))
                    .on_disabled_hover_text("Application Name and Binary Location are required");
                if preview.clicked() {
                    match maker.generate_desktop_content() {
                        Ok(content) => {
                            maker.preview = Some(content);
                            maker.status = None;
                        }
                        Err(e) => maker.status = Some((true, e)),
                    }
                }

                if ui.button("Reset").clicked() {
                    maker.reset_fields();
                }
            });
            ui.add_space(6.0);

            // ── Preview ----------------------------------------------------
            let width = ui.available_width();
            let preview_text = maker.preview.clone();
            let card = card_frame();
            card.show(ui, |ui| {
                ui.set_min_width(width - 28.0);
                ui.vertical(|ui| {
                    card_title(ui, "Preview");
                    ui.add_space(10.0);
                    match &preview_text {
                        Some(preview) => {
                            ui.code(preview);
                        }
                        None => {
                            ui.add(
                                Label::new(
                                    RichText::new(
                                        "No preview yet \u{2014} fill the fields above and press \u{201C}Preview .desktop\u{201D}",
                                    )
                                    .weak()
                                    .monospace(),
                                )
                                .wrap(),
                            );
                        }
                    }
                });
            });
            if preview_text.is_some() {
                ui.add_space(12.0);
            }

            // ── Status pill ────────────────────────────────────────────────
            status_pill(ui, maker.status.clone());

            ui.add_space(4.0);
            ui.separator();
            ui.add(Label::new(RichText::new(XDG_APPS_HINT).small().color(SUBTEXT)).wrap());
        });
    }
}

/// (is_error, message) tuple used by all tools for status pills.
type Status = Option<(bool, String)>;

#[derive(Default)]
struct ShortcutMakerApp {
    name: String,
    exec_path: String,
    icon_path: String,
    description: String,
    categories: String,
    launch_in_terminal: bool,
    preview: Option<String>,
    status: Status,
}

impl ShortcutMakerApp {
    fn trimmed(&self) -> (&str, &str, &str, &str) {
        (
            self.name.trim(),
            self.exec_path.trim(),
            self.icon_path.trim(),
            self.description.trim(),
        )
    }

    fn inputs_valid(&self) -> bool {
        let (name, exec_path, _, _) = self.trimmed();
        !name.is_empty() && !exec_path.is_empty()
    }

    /// Build the `.desktop` content from the current fields.
    fn generate_desktop_content(&self) -> Result<String, String> {
        let (name, exec_path, icon_path, description) = self.trimmed();

        if name.is_empty() || exec_path.is_empty() {
            return Err("Application Name and Binary Location are required!".into());
        }

        let exec = exec_path.replace(' ', "\\ "); // Desktop Entry Spec: escape spaces
        let icon = if icon_path.is_empty() {
            FALLBACK_ICON
        } else {
            icon_path
        };
        let comment = if description.is_empty() { name } else { description };
        let categories = normalize_categories(&self.categories);

        Ok(format!(
            "[Desktop Entry]\nVersion=1.0\nType=Application\nName={name}\nExec={exec}\nIcon={icon}\nComment={comment}\nTerminal={}\nCategories={categories}\n",
            if self.launch_in_terminal { "true" } else { "false" }
        ))
    }

    /// Write the file to ~/.local/share/applications/ and chmod +x.
    fn create_desktop(&mut self) {
        let result = self
            .generate_desktop_content()
            .and_then(|content| write_desktop_file(self.name.trim(), &content));

        self.status = Some(match result {
            Ok(path) => (false, format!("Shortcut created at {}", path.display())),
            Err(e) => (true, e),
        });
    }

    fn reset_fields(&mut self) {
        self.name.clear();
        self.exec_path.clear();
        self.icon_path.clear();
        self.description.clear();
        self.categories = "Utility;".into();
        self.launch_in_terminal = false;
        self.preview = None;
        self.status = None;
    }

    fn invalidate_preview(&mut self) {
        self.preview = None;
    }
}

/// Ensure the Categories value ends with `;` per the spec.
fn normalize_categories(raw: &str) -> String {
    let cats = raw.trim();
    if cats.is_empty() {
        "Utility;".to_owned()
    } else if cats.ends_with(';') {
        cats.to_owned()
    } else {
        format!("{cats};")
    }
}

/// Sanitize the app name into a filesystem-friendly `.desktop` file name.
fn desktop_file_name(name: &str) -> String {
    let safe: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect();
    let safe = safe.trim().replace(' ', "_");
    format!("{safe}.desktop")
}

fn write_desktop_file(name: &str, content: &str) -> Result<PathBuf, String> {
    let target_dir = dirs_home()?
        .join(".local")
        .join("share")
        .join("applications");
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create directory: {e}"))?;

    let target_path = target_dir.join(desktop_file_file_name(name));
    std::fs::write(&target_path, content).map_err(|e| format!("Failed to write file: {e}"))?;

    // Make it executable (0755), mirroring the original behavior.
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&target_path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Failed to set permissions: {e}"))?;

    Ok(target_path)
}

fn desktop_file_file_name(name: &str) -> PathBuf {
    PathBuf::from(desktop_file_name(name))
}

fn dirs_home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| "Could not determine home directory ($HOME is not set)".into())
}

// ══════════════════════════ Tool 2: clipboard inspector ═══════════════════════

#[derive(Clone)]
struct ClipEntry {
    id: u64,
    content: String,
}

/// Bounded clipboard history store shared with the watcher thread.
type ClipStore = (Vec<ClipEntry>, u64);

#[derive(Default)]
struct ClipboardApp {
    store: Option<Arc<Mutex<ClipStore>>>,
    watch: Option<Arc<AtomicBool>>,
    watcher_status: Option<mpsc::Receiver<Result<(), String>>>,
    watch_error: Option<String>,
    paused: bool,
    selected: Option<u64>,
    filter: String,
    status: Status,
    watch_started: bool,
}

impl ClipboardApp {
    /// Starts the background watcher once (idempotent).
    fn ensure_watch(&mut self, ctx: &egui::Context) {
        if self.watch_started {
            return;
        }
        self.watch_started = true;

        let store = Arc::new(Mutex::new((Vec::new(), 0u64)));
        let running = Arc::new(AtomicBool::new(true));
        let (status_tx, status_rx) = mpsc::channel();
        self.store = Some(store.clone());
        self.watch = Some(running.clone());
        self.watcher_status = Some(status_rx);

        let ctx_copy = ctx.clone();
        std::thread::spawn(move || {
            let mut clip = match arboard::Clipboard::new() {
                Ok(clip) => clip,
                Err(error) => {
                    let _ = status_tx.send(Err(error.to_string()));
                    ctx_copy.request_repaint(); // surfaces an empty history view
                    return;
                }
            };
            let _ = status_tx.send(Ok(()));
            let mut last = clip.get_text().ok();
            loop {
                if running.load(Ordering::Relaxed) {
                    if let Ok(text) = clip.get_text() {
                        if last.as_deref() != Some(text.as_str()) {
                            last = Some(text.clone());
                            if !text.trim().is_empty() {
                                if let Ok(mut guard) = store.lock() {
                                    let (entries, next) = &mut *guard;
                                    // dedupe: drop an older identical entry, then insert on top
                                    entries.retain(|e| e.content != text);
                                    entries.insert(
                                        0,
                                        ClipEntry {
                                            id: *next,
                                            content: text,
                                        },
                                    );
                                    *next += 1;
                                    entries.truncate(CLIP_HISTORY_MAX);
                                    ctx_copy.request_repaint();
                                }
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
    }

    fn pause_resume(&mut self) {
        if let Some(watch) = &self.watch {
            self.paused = !self.paused;
            watch.store(!self.paused, Ordering::Relaxed);
        }
    }

    fn retry_watch(&mut self, ctx: &egui::Context) {
        self.watch_started = false;
        self.watcher_status = None;
        self.watch_error = None;
        self.status = None;
        self.ensure_watch(ctx);
    }
}

impl MiniAppsApp {
    fn clipboard_panel(&mut self, ui: &mut egui::Ui) {
        // Start the watcher with this session's egui context (idempotent).
        let ctx = ui.ctx().clone();
        self.clipboard.ensure_watch(&ctx);

        let clipboard = &mut self.clipboard;
        let watcher_status = clipboard
            .watcher_status
            .as_ref()
            .and_then(|rx| rx.try_recv().ok());
        if let Some(result) = watcher_status {
            clipboard.watcher_status = None;
            if let Err(error) = result {
                clipboard.watch_error = Some(error.clone());
                clipboard.status = Some((true, format!("Clipboard unavailable: {error}")));
            }
        }

        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            header_impl(
                ui,
                "Clipboard Inspector",
                "A searchable history of everything copied while the app is running",
            );

            // ── Toolbar ──────────────────────────────────────────────────
            ui.horizontal(|ui| {
                let (button_label, state_label, state_color) = if clipboard.watch_error.is_some() {
                    ("Retry", "Unavailable", RED)
                } else if clipboard.paused {
                    ("Resume", "Paused", YELLOW)
                } else {
                    ("Pause", "Monitoring", GREEN)
                };
                ui.add(
                    Label::new(
                        RichText::new("\u{25CF}").size(10.0).color(state_color),
                    )
                    .sense(Sense::hover()),
                );
                if ui.button(button_label).clicked() {
                    if clipboard.watch_error.is_some() {
                        clipboard.retry_watch(&ctx);
                    } else {
                        clipboard.pause_resume();
                    }
                }
                ui.add(
                    Label::new(RichText::new(state_label).small().color(SUBTEXT))
                        .sense(Sense::hover()),
                );
                if ui.button("Clear All").clicked() {
                    if let Some(store) = &clipboard.store {
                        if let Ok(mut guard) = store.lock() {
                            guard.0.clear();
                        }
                    }
                    clipboard.selected = None;
                    clipboard.status = Some((false, "History cleared".into()));
                }
            });

            ui.horizontal(|ui| {
                ui.add_space(2.0);
                text_field(ui, "Filter…", &mut clipboard.filter);
            });
            ui.add_space(4.0);

            // ── History list ─────────────────────────────────────────────
            let filter = clipboard.filter.trim().to_lowercase();
            let snapshot = clipboard
                .store
                .as_ref()
                .and_then(|s| s.lock().ok())
                .map(|g| g.0.clone())
                .unwrap_or_default();
            if snapshot.is_empty() {
                let empty_msg = if clipboard.paused {
                    "Paused. Resume to record new clipboard contents."
                } else {
                    "History is empty; copy something."
                };
                ui.add(
                    Label::new(RichText::new(empty_msg).weak())
                        .wrap(),
                );
            } else {
                let width = ui.available_width();
                let card = card_frame();
                card.show(ui, |ui| {
                    ui.set_min_width(width - 28.0);
                    ui.vertical(|ui| {
                        card_title(ui, "History");
                        ui.add_space(10.0);
                        ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                            let mut shown = 0usize;
                            for entry in &snapshot {
                                if !filter.is_empty()
                                    && !entry.content.to_lowercase().contains(&filter)
                                {
                                    continue;
                                }
                                let preview = preview_of(&entry.content, 96);
                                let sel = clipboard.selected == Some(entry.id);
                                let resp = ui.selectable_label(sel, preview);
                                if resp.clicked() {
                                    clipboard.selected = Some(entry.id);
                                }
                                if resp.secondary_clicked() {
                                    match copy_to_clipboard(&entry.content) {
                                        Ok(()) => clipboard.status =
                                            Some((false, String::from("Entry copied to clipboard"))),
                                        Err(e) => clipboard.status = Some((true, e)),
                                    }
                                }
                                shown += 1;
                            }
                            if shown == 0 {
                                ui.add(
                                    Label::new(
                                        RichText::new("No entries match this filter")
                                            .weak()
                                            .monospace(),
                                    )
                                    .wrap(),
                                );
                            }
                        });
                    });
                });
            }
            ui.add_space(12.0);

            // ── Selected entry detail ────────────────────────────────────
            if let Some(selected_id) = clipboard.selected {
                if let Some(entry) = snapshot.iter().find(|e| e.id == selected_id) {
                    let width = ui.available_width();
                    let card = card_frame();
                    card.show(ui, |ui| {
                        ui.set_min_width(width - 28.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                card_title(ui, "Selected Entry");
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("Remove").clicked() {
                                        if let Some(store) = &clipboard.store {
                                            if let Ok(mut guard) = store.lock() {
                                                guard.0.retain(|e| e.id != selected_id);
                                            }
                                        }
                                        clipboard.selected = None;
                                    }
                                    if ui.button("Copy").clicked() {
                                        match copy_to_clipboard(&entry.content) {
                                            Ok(()) => clipboard.status = Some((false, String::from("Entry copied to clipboard"))),
                                            Err(e) => clipboard.status = Some((true, e)),
                                        }
                                    }
                                });
                            });
                            ui.add_space(8.0);
                            let mut scratch = entry.content.clone();
                            ui.add(
                                TextEdit::multiline(&mut scratch)
                                    .interactive(false)
                                    .code_editor()
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(6),
                            );
                            let (lines, chars) = summary_of(&entry.content);
                            ui.add(
                                Label::new(
                                    RichText::new(format!("{chars} chars · {lines} lines"))
                                        .small()
                                        .color(SUBTEXT),
                                )
                                .sense(Sense::hover()),
                            );
                        });
                    });
                }
            }

            ui.add_space(8.0);
            status_pill(ui, clipboard.status.clone());

            ui.separator();
            ui.add(
                Label::new(
                    RichText::new("Right-click an entry to copy it quickly; the watcher polls the clipboard twice per second.")
                        .small()
                        .color(SUBTEXT),
                )
                .wrap(),
            );
        });
    }
}

fn preview_of(content: &str, maxlen: usize) -> String {
    let first_line = content.lines().next().unwrap_or("");
    let mut out: String = first_line.chars().take(maxlen).collect();
    if first_line.chars().count() > maxlen {
        out.push('\u{2026}');
    } else if content.lines().count() > 1 {
        out.push_str(" \u{2026}");
    }
    out.replace('\n', " ").replace('\t', " ")
}

fn summary_of(content: &str) -> (usize, usize) {
    (content.lines().count(), content.chars().count())
}

/// Copy text into the system clipboard (best effort across X11/Wayland).
fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clip = arboard::Clipboard::new().map_err(|e| format!("Clipboard unavailable: {e}"))?;
    clip.set_text(text.to_owned())
        .map_err(|e| format!("Failed to write clipboard: {e}"))
}

// ══════════════════════════ Screenshot portal plumbing ════════════════════════

enum CaptureResult {
    Saved(String), // file:// URI
    Cancelled,
    Failed(String),
}

/// Takes a screenshot via the freedesktop portal in a background thread.
/// `interactive` shows the regional selection offered by the desktop.
fn spawn_portal_capture(ctx: egui::Context, interactive: bool) -> mpsc::Receiver<CaptureResult> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        use ashpd::desktop::screenshot::ScreenshotRequest;

        let result = (|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let request = rt
                .block_on(ScreenshotRequest::default().interactive(interactive).send())
                .map_err(map_portal_error)?;
            let shot = request.response().map_err(map_portal_error)?;
            Ok(shot.uri().as_str().to_owned())
        })();

        let _ = tx.send(match result {
            Err(e) if e == "cancelled" => CaptureResult::Cancelled,
            Err(e) => CaptureResult::Failed(e),
            Ok(uri) => CaptureResult::Saved(uri),
        });
        ctx.request_repaint();
    });
    fn map_portal_error(e: ashpd::Error) -> String {
        // ashpd keeps ResponseError private, so detect user cancellation by message.
        let msg = e.to_string();
        if msg.contains("cancelled") {
            "cancelled".into()
        } else {
            msg
        }
    }
    rx
}

#[derive(Default)]
struct ScreenshotApp {
    rx: Option<mpsc::Receiver<CaptureResult>>,
    busy: bool,
    captures: Vec<String>, // file:// URIs, newest first
    status: Status,
    last_path: Option<String>,
}

impl ScreenshotApp {
    fn capture(&mut self, ctx: egui::Context, interactive: bool) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.status = Some((false, if interactive {
            "Region capture started — select an area in the portal overlay".into()
        } else {
            "Full screen capture started".into()
        }));
        self.rx = Some(spawn_portal_capture(ctx, interactive));
    }

    /// Drains the background channel; called each frame.
    fn poll(&mut self) {
        if self.rx.is_none() {
            return;
        }
        let msg = self
            .rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok());
        if let Some(result) = msg {
            self.rx = None;
            self.busy = false;
            match result {
                CaptureResult::Saved(uri) => {
                    let path = uri.trim_start_matches("file://").to_owned();
                    self.captures.insert(0, path.clone());
                    self.captures.truncate(10);
                    self.last_path = Some(path.clone());
                    self.status = Some((false, format!("Capture saved to {path}")));
                }
                CaptureResult::Cancelled => {
                    self.status = Some((false, "Capture cancelled".into()));
                }
                CaptureResult::Failed(e) => {
                    self.status = Some((true, format!("Capture failed: {e}")));
                }
            }
        }
    }

    fn copy_last_path(&mut self) {
        if let Some(path) = self.last_path.clone() {
            match copy_to_clipboard(&path) {
                Ok(()) => self.status = Some((false, "Path copied to clipboard".into())),
                Err(e) => self.status = Some((true, e)),
            }
        }
    }

    fn open_last(&mut self) {
        if let Some(path) = self.last_path.clone() {
            match std::process::Command::new("xdg-open").arg(&path).spawn() {
                Ok(_) => self.status = Some((false, "Opening image…".into())),
                Err(e) => self.status = Some((true, format!("Failed to open: {e}"))),
            }
        }
    }
}

impl MiniAppsApp {
    fn screenshot_panel(&mut self, ui: &mut egui::Ui) {
        let screenshot = &mut self.screenshot;
        screenshot.poll();

        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            header_impl(
                ui,
                "Screenshot Tool",
                "Captures through your desktop portal — region or full screen, no extra permissions",
            );

            ui.horizontal(|ui| {
                let region = ui
                    .add_enabled(!screenshot.busy, Button::new("Capture Region").fill(ACCENT));
                if region.clicked() {
                    screenshot.capture(ui.ctx().clone(), true);
                }
                let full = ui.add_enabled(!screenshot.busy, Button::new("Capture Full Screen"));
                if full.clicked() {
                    screenshot.capture(ui.ctx().clone(), false);
                }
                if screenshot.busy {
                    ui.add(
                        Label::new(RichText::new("waiting for portal…").color(SUBTEXT).small())
                            .sense(Sense::hover()),
                    );
                }
            });
            ui.add_space(16.0);

            if let Some(path) = screenshot.last_path.clone() {
                let width = ui.available_width();
                let card = card_frame();
                card.show(ui, |ui| {
                    ui.set_min_width(width - 28.0);
                    ui.vertical(|ui| {
                        card_title(ui, "Latest Capture");
                        ui.add_space(10.0);
                        ui.code(&path);
                        ui.vertical(|ui| {
                            if ui.button("Copy Path").clicked() {
                                screenshot.copy_last_path();
                            }
                            if ui.button("Open in Image Viewer").clicked() {
                                screenshot.open_last();
                            }
                        });
                    });
                });
                ui.add_space(12.0);
            }

            if !screenshot.captures.is_empty() {
                let width = ui.available_width();
                let card = card_frame();
                card.show(ui, |ui| {
                    ui.set_min_width(width - 28.0);
                    ui.vertical(|ui| {
                        card_title(ui, "Recent Captures");
                        ui.add_space(10.0);
                        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            for path in screenshot.captures.clone() {
                                let resp = ui.selectable_label(
                                    screenshot.last_path.as_deref() == Some(path.as_str()),
                                    pretty_path(&path),
                                );
                                if resp.clicked() {
                                    screenshot.last_path = Some(path.clone());
                                    screenshot.status = None;
                                }
                            }
                        });
                    });
                });
                ui.add_space(12.0);
            }

            status_pill(ui, screenshot.status.clone());

            ui.separator();
            ui.add(
                Label::new(
                    RichText::new(
                        "Region capture shows your desktop's selection overlay; results are saved by the portal (usually ~/Pictures).",
                    )
                    .small()
                    .color(SUBTEXT),
                )
                .wrap(),
            );
        });
    }
}

fn pretty_path(path: &str) -> String {
    path.rsplit('/')
        .next()
        .filter(|base| !base.is_empty())
        .unwrap_or(path)
        .to_owned()
}

// ══════════════════════════ Tool 4: color picker ══════════════════════════════

#[derive(Default)]
struct ColorPickerApp {
    rx: Option<mpsc::Receiver<CaptureResult>>,
    busy: bool,
    image: Option<egui::ColorImage>,
    texture: Option<egui::TextureHandle>,
    path: Option<String>,
    hover: Option<[u8; 4]>,
    current: Option<[u8; 4]>,
    picked: Vec<[u8; 4]>, // newest first, deduped, capped
    status: Status,
}

impl ColorPickerApp {
    fn start_pick(&mut self, ctx: egui::Context) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.status =
            Some((false, "Select a region in the portal overlay to sample colors".into()));
        self.rx = Some(spawn_portal_capture(ctx, true));
    }

    fn poll(&mut self) {
        if self.rx.is_none() {
            return;
        }
        let msg = self.rx.as_ref().and_then(|r| r.try_recv().ok());
        if let Some(result) = msg {
            self.rx = None;
            self.busy = false;
            match result {
                CaptureResult::Saved(uri) => {
                    let path = uri.trim_start_matches("file://").to_owned();
                    match image::open(&path) {
                        Ok(img) => {
                            let rgba = img.to_rgba8();
                            let (w, h) = rgba.dimensions();
                            let color = egui::ColorImage::from_rgba_unmultiplied(
                                [w as usize, h as usize],
                                rgba.as_raw(),
                            );
                            self.status = Some((
                                false,
                                format!("Loaded {}×{} sample — click a pixel", w, h),
                            ));
                            self.current = None;
                            self.hover = None;
                            self.path = Some(path);
                            self.image = Some(color);
                            self.texture = None;
                        }
                        Err(e) => {
                            self.status =
                                Some((true, format!("Failed to decode capture: {e}")));
                        }
                    }
                }
                CaptureResult::Cancelled => {
                    self.status = Some((false, "Capture cancelled".into()));
                }
                CaptureResult::Failed(e) => {
                    self.status = Some((true, format!("Capture failed: {e}")));
                }
            }
        }
    }

    fn store_pixel(&mut self, ctx: &egui::Context, px: [u8; 4]) {
        self.current = Some(px);
        if let Some(pos) = self.picked.iter().position(|c| *c == px) {
            if pos != 0 {
                self.picked.remove(pos);
                self.picked.insert(0, px);
            }
        } else {
            self.picked.insert(0, px);
            self.picked.truncate(12);
        }
        let hex = hex_str(px);
        match copy_to_clipboard(&hex) {
            Ok(()) => self.status = Some((false, format!("{hex} copied to clipboard"))),
            Err(e) => self.status = Some((true, e)),
        }
        let _ = ctx; // texture updates happen through normal repaint
    }
}

fn hex_str(px: [u8; 4]) -> String {
    format!("#{:02X}{:02X}{:02X}", px[0], px[1], px[2])
}

impl MiniAppsApp {
    fn color_picker_panel(&mut self, ui: &mut egui::Ui) {
        let picker = &mut self.color_picker;
        picker.poll();

        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            header_impl(
                ui,
                "Color Picker",
                "Capture a screen region, hover a pixel, click to copy its hex value",
            );

            // ── Toolbar ──────────────────────────────────────────────────
            ui.horizontal(|ui| {
                let pick = ui.add_enabled(!picker.busy, Button::new("Sample Region").fill(ACCENT));
                if pick.clicked() {
                    picker.start_pick(ui.ctx().clone());
                }
                if picker.busy {
                    ui.add(
                        Label::new(RichText::new("waiting for portal…").color(SUBTEXT).small())
                            .sense(Sense::hover()),
                    );
                }
            });
            ui.add_space(12.0);

            // ── Currently selected color ─────────────────────────────────
            if let Some(px) = picker.current {
                let width = ui.available_width();
                let card = card_frame();
                card.show(ui, |ui| {
                    ui.set_min_width(width - 28.0);
                    ui.vertical(|ui| {
                        card_title(ui, "Current Color");
                        ui.add_space(10.0);
                        current_color_row(ui, px);
                        if ui.button("Copy Again").clicked() {
                            let hex = hex_str(px);
                            match copy_to_clipboard(&hex) {
                                Ok(()) => picker.status = Some((false, format!("{hex} copied"))),
                                Err(e) => picker.status = Some((true, e)),
                            }
                        }
                    });
                });
                ui.add_space(12.0);
            }

            // ── Image sampling canvas ────────────────────────────────────
            let image = picker.image.clone();
            if let Some(img) = &image {
                // (Re)create the GPU texture when a new sample arrives or none exists yet.
                if picker
                    .texture
                    .as_ref()
                    .map_or(true, |t| t.size() != img.size)
                {
                    picker.texture = Some(
                        ui.ctx()
                            .load_texture("screen-sample", img.clone(), TextureOptions::LINEAR),
                    );
                }
            }
            let tex = picker.texture.clone();
            if let (Some(tex), Some(img)) = (tex, image) {
                let width = ui.available_width();
                let card = card_frame();
                card.show(ui, |ui| {
                    ui.set_min_width(width - 28.0);
                    ui.vertical(|ui| {
                        card_title(ui, "Sampling Canvas");
                        ui.add_space(10.0);
                        color_canvas(ui, &tex, &img, picker);
                    });
                });
                ui.add_space(12.0);
            } else {
                ui.add(
                    Label::new(
                        RichText::new(
                            "No sample loaded yet — press \u{201C}Sample Region\u{201D} and select a screen area.",
                        )
                        .weak()
                        .monospace(),
                    )
                    .wrap(),
                );
                ui.add_space(12.0);
            }

            // ── Picked colors ────────────────────────────────────────────
            if !picker.picked.is_empty() {
                let width = ui.available_width();
                let card = card_frame();
                card.show(ui, |ui| {
                    ui.set_min_width(width - 28.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            card_title(ui, "Picked Colors");
                            if ui.button("Clear").clicked() {
                                picker.picked.clear();
                            }
                        });
                        ui.add_space(10.0);
                        pick_history(ui, picker);
                    });
                });
                ui.add_space(12.0);
            }

            status_pill(ui, picker.status.clone());

            ui.separator();
            ui.add(
                Label::new(
                    RichText::new(
                        "Tip: hover the canvas for a zoomed loupe; click a pixel to select & copy its hex value.",
                    )
                    .small()
                    .color(SUBTEXT),
                )
                .wrap(),
            );
        });
    }
}

fn current_color_row(ui: &mut egui::Ui, px: [u8; 4]) {
    px_color_chip(ui, px, 56.0, ACCENT);
    ui.add(
        Label::new(
            RichText::new(format!("  {}  R {} G {} B {}", hex_str(px), px[0], px[1], px[2]))
                .monospace(),
        )
        .sense(Sense::hover()),
    );
}

fn px_color_chip(ui: &mut egui::Ui, px: [u8; 4], size: f32, border: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(8), color_of(&px));
    ui.painter().rect(
        rect,
        CornerRadius::same(8),
        Color32::TRANSPARENT,
        Stroke::new(1.5, border),
        StrokeKind::Inside,
    );
}

fn color_of(px: &[u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(px[0], px[1], px[2], px[3])
}

fn pick_history(ui: &mut egui::Ui, picker: &mut ColorPickerApp) {
    ui.horizontal_wrapped(|ui| {
        let picked = picker.picked.clone();
        let mut reopen: Option<[u8; 4]> = None;
        for px in picked {
            let hex = hex_str(px);
            let (rect, resp) = ui.allocate_exact_size(
                egui::vec2(86.0, 30.0),
                Sense::click(),
            );
            let painter = ui.painter();
            painter.rect_filled(rect, CornerRadius::same(6), color_of(&px));
            painter.rect(
                rect,
                CornerRadius::same(6),
                Color32::TRANSPARENT,
                Stroke::new(1.0, BORDER),
                StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                hex,
                FontId::monospace(10.0),
                readable_on(&px),
            );
            if resp.clicked() {
                reopen = Some(px);
            }
        }
        if let Some(px) = reopen {
            let ctx = ui.ctx().clone();
            picker.store_pixel(&ctx, px);
        }
    });
}

/// Dark or light text depending on background luminance.
fn readable_on(px: &[u8; 4]) -> Color32 {
    let lum = 0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32;
    if lum > 130.0 {
        Color32::from_rgb(0x1E, 0x1E, 0x2E) // dark text on light swatch
    } else {
        TEXT
    }
}

fn color_canvas(
    ui: &mut egui::Ui,
    tex: &egui::TextureHandle,
    img: &egui::ColorImage,
    picker: &mut ColorPickerApp,
) {
    let w = img.size[0] as f32;
    let h = img.size[1] as f32;
    let available = ui.available_size() * 0.95;
    let scale = (available.x / w).min((available.y - 24.0) / h).clamp(0.05, 2.5);
    let draw_size = egui::vec2(w * scale, h * scale);

    let (rect, resp) = ui.allocate_exact_size(draw_size, Sense::click_and_drag());
    ui.painter().image(
        tex.id(),
        rect,
        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    ui.painter().rect(
        rect,
        CornerRadius::same(6),
        Color32::TRANSPARENT,
        Stroke::new(1.0, BORDER),
        StrokeKind::Inside,
    );

    let mut hovered_px = None;
    if let Some(cursor) = resp.hover_pos() {
        if rect.contains(cursor) {
            let mx = ((cursor.x - rect.left()) / scale) as usize;
            let my = ((cursor.y - rect.top()) / scale) as usize;
            let idx = my * img.size[0] + mx.min(img.size[0] - 1);
            if idx < img.pixels.len() {
                let c = img.pixels[idx];
                let px = [c.r(), c.g(), c.b(), c.a()];
                hovered_px = Some(px);
                picker.hover = Some(px);
                draw_loupe(ui, img, rect, scale, cursor, px);
            }
        }
    }
    if resp.clicked() {
        if let Some(px) = hovered_px.or(picker.hover) {
            let ctx = ui.ctx().clone();
            picker.store_pixel(&ctx, px);
        }
    }
}

/// Draws a magnified 7×7 pixel loupe next to the cursor.
fn draw_loupe(
    ui: &mut egui::Ui,
    img: &egui::ColorImage,
    canvas: egui::Rect,
    scale: f32,
    cursor: egui::Pos2,
    center_px: [u8; 4],
) {
    const CELL: f32 = 16.0;
    const RADIUS: i32 = 3; // 7×7 window
    let (w, h) = (img.size[0] as i32, img.size[1] as i32);

    let ccol = ((cursor.x - canvas.left()) / scale) as i32;
    let crow = ((cursor.y - canvas.top()) / scale) as i32;

    let zoom_size = egui::vec2(CELL * 7.0, CELL * 7.0);
    let mut origin = cursor + egui::vec2(18.0, 18.0);
    // clamp inside the canvas
    origin.x = origin.x.min(canvas.right() - zoom_size.x).max(canvas.left());
    origin.y = origin.y.min(canvas.bottom() - zoom_size.y).max(canvas.top());

    let painter = ui.painter();
    painter.rect_filled(
        egui::Rect::from_min_size(origin, zoom_size),
        CornerRadius::same(4),
        Color32::from_black_alpha(200),
    );

    for dy in -RADIUS..=RADIUS {
        for dx in -RADIUS..=RADIUS {
            let sx = (ccol + dx).clamp(0, w - 1);
            let sy = (crow + dy).clamp(0, h - 1);
            let px = &img.pixels[(sy * w + sx) as usize];
            let cell = egui::Rect::from_min_size(
                origin + egui::vec2(dx as f32 * CELL, dy as f32 * CELL),
                egui::vec2(CELL - 1.0, CELL - 1.0),
            );
            painter.rect_filled(cell, CornerRadius::same(2), *px);
        }
    }

    // Accent frame + hex caption
    let center_cell = egui::Rect::from_min_size(
        origin + egui::vec2(CELL * 3.0, CELL * 3.0),
        egui::vec2(CELL, CELL),
    );
    painter.rect(
        center_cell,
        CornerRadius::same(2),
        Color32::TRANSPARENT,
        Stroke::new(2.0, ACCENT),
        StrokeKind::Inside,
    );
    painter.text(
        origin + egui::vec2(0.0, zoom_size.y + 6.0),
        Align2::LEFT_TOP,
        hex_str(center_px),
        FontId::monospace(12.0),
        ACCENT,
    );
}

// ═══════════════════════════ Shared UI helpers ════════════════════════════════

fn header_impl(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.add_space(8.0);
    ui.add(Label::new(RichText::new(title).heading()).sense(Sense::hover()));
    ui.add(Label::new(RichText::new(subtitle).color(SUBTEXT)).sense(Sense::hover()));
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 3.0), Sense::hover());
    ui.painter().rect_filled(
        rect,
        egui::epaint::CornerRadiusF32::same(1.5),
        ACCENT.gamma_multiply(0.85),
    );
    ui.add_space(10.0);
}

/// Rounded status pill (green success / red error).
fn status_pill(ui: &mut egui::Ui, status: Status) {
    let Some((is_error, msg)) = status else {
        return;
    };
    let color = if is_error { RED } else { GREEN };

    Frame::new()
        .fill(color.gamma_multiply(0.12))
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.5)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add(Label::new(RichText::new("\u{25CF}").size(10.0).color(color)).sense(Sense::hover()));
                ui.add_space(4.0);
                ui.add(Label::new(RichText::new(msg).color(TEXT)).wrap().sense(Sense::hover()));
            });
        });
    ui.add_space(4.0);
}

/// Standard card frame used by every section.
fn card_frame() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(14))
}

/// Small uppercase card section title.
fn card_title(ui: &mut egui::Ui, title: &str) {
    ui.add(
        Label::new(
            RichText::new(title.to_uppercase())
                .small()
                .strong()
                .color(SUBTEXT),
        )
        .sense(Sense::hover()),
    );
}

fn form_label(ui: &mut egui::Ui, label: &str) {
    ui.add(Label::new(RichText::new(label).color(SUBTEXT).strong()));
}

/// Fills-remaining-width single-line text field. Returns `changed`.
fn text_field(ui: &mut egui::Ui, hint: &str, value: &mut String) -> bool {
    ui.add(
        TextEdit::singleline(value)
            .hint_text(hint)
            .desired_width(f32::INFINITY),
    )
    .changed()
}

/// Text field inside a row with a trailing button; leaves `reserve` px for it.
fn row_text_field(ui: &mut egui::Ui, hint: &str, value: &mut String, reserve: f32) -> bool {
    let w = (ui.available_width() - reserve).max(80.0);
    ui.add(
        TextEdit::singleline(value)
            .hint_text(hint)
            .desired_width(w),
    )
    .changed()
}

fn browse_button(ui: &mut egui::Ui, action: impl FnOnce()) {
    if ui.add_sized([80.0, 26.0], Button::new("Browse")).clicked() {
        action();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_desktop_entry() {
        let app = ShortcutMakerApp {
            name: "My Great App".into(),
            exec_path: "/opt/apps/My Great App/run".into(),
            icon_path: "/home/user/icon.png".into(),
            description: "Does things".into(),
            categories: "Utility;".into(),
            ..Default::default()
        };
        let content = app.generate_desktop_content().unwrap();
        assert!(content.starts_with("[Desktop Entry]\n"));
        assert!(content.contains("Name=My Great App"));
        assert!(content.contains("Exec=/opt/apps/My\\ Great\\ App/run"));
        assert!(content.contains("Icon=/home/user/icon.png"));
        assert!(content.contains("Terminal=false"));
        assert!(content.ends_with("Categories=Utility;\n"));
    }

    #[test]
    fn name_and_exec_are_required() {
        let app = ShortcutMakerApp::default();
        assert!(app.generate_desktop_content().is_err());

        let app = ShortcutMakerApp {
            name: "Foo".into(),
            ..Default::default()
        };
        assert!(app.generate_desktop_content().is_err());
    }

    #[test]
    fn fallback_icon_and_comment() {
        let app = ShortcutMakerApp {
            name: "Foo".into(),
            exec_path: "/bin/foo".into(),
            ..Default::default()
        };
        let content = app.generate_desktop_content().unwrap();
        assert!(content.contains(&format!("Icon={FALLBACK_ICON}")));
        assert!(content.contains("Comment=Foo"));
    }

    #[test]
    fn sanitizes_file_names() {
        assert_eq!(desktop_file_name("My Great App!"), "My_Great_App.desktop");
        assert_eq!(
            desktop_file_name("  foo-bar_baz.exe "),
            "foo-bar_bazexe.desktop"
        );
    }

    #[test]
    fn terminal_flag_is_serialized() {
        let app = ShortcutMakerApp {
            name: "Foo".into(),
            exec_path: "/bin/foo".into(),
            launch_in_terminal: true,
            ..Default::default()
        };
        assert!(app
            .generate_desktop_content()
            .unwrap()
            .contains("Terminal=true"));
    }

    #[test]
    fn categories_are_normalized() {
        assert_eq!(normalize_categories(""), "Utility;");
        assert_eq!(normalize_categories("Development"), "Development;");
        assert_eq!(normalize_categories("Development;IDE"), "Development;IDE;");
    }

    #[test]
    fn fuzzy_matches_subsequences() {
        assert!(fuzzy_score("ds", "Desktop Shortcut Maker").is_some());
        assert!(fuzzy_score("dsk", "Desktop Shortcut Maker").is_some());
        assert!(fuzzy_score("shortcut maker", "Desktop Shortcut Maker").is_some());
        assert!(fuzzy_score("xyz", "Desktop Shortcut Maker").is_none());
        assert_eq!(fuzzy_score("", "anything"), Some(0));
    }

    #[test]
    fn fuzzy_prefers_word_starts_and_consecutive() {
        let consecutive = fuzzy_score("sh", "shortcut tool").unwrap();
        let scattered = fuzzy_score("st", "shortcut tool").unwrap();
        assert!(consecutive > scattered);
        // Keyword search finds tools by function.
        assert!(fuzzy_score(
            "launcher",
            "shortcut launcher desktop entry application menu xdg freedesktop"
        )
        .is_some());
    }

    #[test]
    fn previews_are_truncated() {
        assert_eq!(preview_of("hello", 96), "hello");
        assert_eq!(preview_of("", 96), "");
        assert!(preview_of(&"aaaa".repeat(40), 10).ends_with('\u{2026}'));
    }

    #[test]
    fn hex_formatting() {
        assert_eq!(hex_str([0, 0, 0, 255]), "#000000");
        assert_eq!(hex_str([255, 255, 255, 255]), "#FFFFFF");
        assert_eq!(hex_str([0x89, 0xB4, 0xFA, 255]), "#89B4FA");
    }

    #[test]
    fn readable_text_chooses_contrast() {
        assert_eq!(readable_on(&[0, 0, 0, 255]), TEXT);
        assert_eq!(readable_on(&[255, 255, 255, 255]), BG);
    }
}
