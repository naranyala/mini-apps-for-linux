use eframe::egui;

const BG: egui::Color32 = egui::Color32::from_rgb(0x1E, 0x1E, 0x2E);
const PANEL: egui::Color32 = egui::Color32::from_rgb(0x2A, 0x2B, 0x3E);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x89, 0xB4, 0xFA);
const MUTED: egui::Color32 = egui::Color32::from_rgb(0xA6, 0xAD, 0xC8);
const GREEN: egui::Color32 = egui::Color32::from_rgb(0xA6, 0xE3, 0xA1);
const YELLOW: egui::Color32 = egui::Color32::from_rgb(0xF9, 0xE2, 0xAF);

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Mini Apps — Mockup Gallery")
            .with_inner_size([980.0, 720.0])
            .with_min_inner_size([680.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Mini Apps — Mockup Gallery",
        options,
        Box::new(|_cc| Ok(Box::new(MockupGallery::default()))),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mockup {
    Notes,
    Monitor,
    Mixer,
    Focus,
}

impl Mockup {
    const ALL: [Self; 4] = [Self::Notes, Self::Monitor, Self::Mixer, Self::Focus];

    fn title(self) -> &'static str {
        match self {
            Self::Notes => "Quick Notes",
            Self::Monitor => "System Monitor",
            Self::Mixer => "Audio Mixer",
            Self::Focus => "Focus Timer",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::Notes => "A calm space for quick thoughts and useful snippets.",
            Self::Monitor => "A glanceable overview of a fictional desktop session.",
            Self::Mixer => "A visual concept for per-app audio controls.",
            Self::Focus => "A simple focus session with a little room to breathe.",
        }
    }
}

struct Note {
    title: &'static str,
    tag: &'static str,
    body: &'static str,
}

const NOTES: [Note; 4] = [
    Note {
        title: "Weekend ideas",
        tag: "PERSONAL",
        body: "Take the long route to the bookshop.\n\nTry a new recipe and leave some time for absolutely nothing.",
    },
    Note {
        title: "Project sketch",
        tag: "WORK",
        body: "Keep the first version small. Make the next action obvious, then let the details grow only when they help.",
    },
    Note {
        title: "Useful commands",
        tag: "LINUX",
        body: "A little scratchpad for commands worth remembering.\n\nThis preview is sample text, not a terminal.",
    },
    Note {
        title: "Reading list",
        tag: "LATER",
        body: "• The Design of Everyday Things\n• A Psalm for the Wild-Built\n• Something picked up on a whim",
    },
];

struct MockupGallery {
    selected: Mockup,
    search: String,
    selected_note: usize,
    note_filter: String,
    master_volume: f32,
    app_volumes: [f32; 3],
    muted: [bool; 3],
    focus_running: bool,
    focus_minutes: u32,
}

impl Default for MockupGallery {
    fn default() -> Self {
        Self {
            selected: Mockup::Notes,
            search: String::new(),
            selected_note: 0,
            note_filter: String::new(),
            master_volume: 72.0,
            app_volumes: [82.0, 54.0, 36.0],
            muted: [false; 3],
            focus_running: false,
            focus_minutes: 25,
        }
    }
}

impl eframe::App for MockupGallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::central_panel(ui.style())
            .fill(BG)
            .inner_margin(egui::Margin::same(22))
            .show(ui, |ui| {
                self.header(ui);
                ui.add_space(16.0);
                self.navigation(ui);
                ui.add_space(16.0);
                egui::Frame::new()
                    .fill(PANEL)
                    .corner_radius(egui::CornerRadius::same(14))
                    .inner_margin(egui::Margin::same(20))
                    .show(ui, |ui| {
                        ui.heading(self.selected.title());
                        ui.label(egui::RichText::new(self.selected.subtitle()).color(MUTED));
                        ui.add_space(14.0);
                        match self.selected {
                            Mockup::Notes => self.notes_panel(ui),
                            Mockup::Monitor => Self::monitor_panel(ui),
                            Mockup::Mixer => self.mixer_panel(ui),
                            Mockup::Focus => self.focus_panel(ui),
                        }
                    });
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(
                            "Concept preview · sample data only · no system settings or files are changed",
                        )
                        .small()
                        .color(MUTED),
                    );
                });
            });
    }
}

impl MockupGallery {
    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Mini Apps").size(27.0).strong());
                ui.label(egui::RichText::new("Small ideas for a more helpful desktop").color(MUTED));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::Frame::new()
                    .fill(ACCENT.gamma_multiply(0.16))
                    .corner_radius(egui::CornerRadius::same(20))
                    .inner_margin(egui::Margin::symmetric(12, 7))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("UI MOCKUPS").strong().color(ACCENT));
                    });
            });
        });
        ui.add_space(12.0);
        ui.add(
            egui::TextEdit::singleline(&mut self.search)
                .hint_text("Search mockups…")
                .desired_width(f32::INFINITY),
        );
    }

    fn navigation(&mut self, ui: &mut egui::Ui) {
        let query = self.search.trim().to_lowercase();
        ui.horizontal_wrapped(|ui| {
            for mockup in Mockup::ALL {
                if !query.is_empty() && !mockup.title().to_lowercase().contains(&query) {
                    continue;
                }
                let selected = self.selected == mockup;
                if ui
                    .selectable_label(selected, egui::RichText::new(mockup.title()).strong())
                    .clicked()
                {
                    self.selected = mockup;
                }
            }
            if !query.is_empty()
                && !Mockup::ALL
                    .iter()
                    .any(|mockup| mockup.title().to_lowercase().contains(&query))
            {
                ui.label(egui::RichText::new("No mockups match that search").color(MUTED));
            }
        });
    }

    fn notes_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.note_filter)
                    .hint_text("Filter notes…")
                    .desired_width(220.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let _ = ui.button("＋ New note");
            });
        });
        ui.add_space(12.0);

        let filter = self.note_filter.trim().to_lowercase();
        ui.columns(2, |columns| {
            columns[0].set_min_width(210.0);
            egui::ScrollArea::vertical()
                .max_height(380.0)
                .show(&mut columns[0], |ui| {
                    for (index, note) in NOTES.iter().enumerate() {
                        if !filter.is_empty()
                            && !note.title.to_lowercase().contains(&filter)
                            && !note.tag.to_lowercase().contains(&filter)
                            && !note.body.to_lowercase().contains(&filter)
                        {
                            continue;
                        }
                        let selected = self.selected_note == index;
                        egui::Frame::new()
                            .fill(if selected {
                                ACCENT.gamma_multiply(0.13)
                            } else {
                                egui::Color32::TRANSPARENT
                            })
                            .corner_radius(egui::CornerRadius::same(9))
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                if ui
                                    .selectable_label(selected, egui::RichText::new(note.title).strong())
                                    .clicked()
                                {
                                    self.selected_note = index;
                                }
                                ui.label(egui::RichText::new(note.tag).small().color(ACCENT));
                                ui.label(
                                    egui::RichText::new(note.body.lines().next().unwrap_or(""))
                                        .small()
                                        .color(MUTED),
                                );
                            });
                        ui.add_space(5.0);
                    }
                });

            egui::Frame::new()
                .fill(BG.gamma_multiply(0.55))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(16))
                .show(&mut columns[1], |ui| {
                    let note = &NOTES[self.selected_note];
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(note.tag).small().strong().color(ACCENT));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let _ = ui.button("☆ Pin");
                        });
                    });
                    ui.add_space(8.0);
                    ui.heading(note.title);
                    ui.separator();
                    ui.label(egui::RichText::new(note.body).size(15.0));
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("Edited just now · demo note").small().color(MUTED));
                });
        });
    }

    fn monitor_panel(ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            Self::metric_card(ui, "PROCESSOR", "24%", 0.24, ACCENT);
            Self::metric_card(ui, "MEMORY", "6.2 / 16 GB", 0.39, GREEN);
            Self::metric_card(ui, "STORAGE", "418 / 1 TB", 0.42, YELLOW);
        });
        ui.add_space(12.0);
        ui.columns(2, |columns| {
            egui::Frame::new().fill(BG.gamma_multiply(0.55)).corner_radius(egui::CornerRadius::same(10)).inner_margin(egui::Margin::same(14)).show(&mut columns[0], |ui| {
                ui.label(egui::RichText::new("PERFORMANCE · SIMULATED").small().strong().color(MUTED));
                ui.add_space(8.0);
                let values = [0.30, 0.42, 0.38, 0.55, 0.47, 0.61, 0.50, 0.72, 0.58, 0.43, 0.67, 0.54, 0.76, 0.49, 0.37, 0.56, 0.45, 0.63, 0.51, 0.40];
                ui.horizontal(|ui| {
                    for value in values {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 88.0), egui::Sense::hover());
                        let bar = egui::Rect::from_min_max(
                            egui::pos2(rect.left(), rect.bottom() - rect.height() * value),
                            rect.right_bottom(),
                        );
                        ui.painter().rect_filled(rect, egui::CornerRadius::same(3), PANEL);
                        ui.painter().rect_filled(bar, egui::CornerRadius::same(3), ACCENT.gamma_multiply(0.8));
                    }
                });
                ui.label(egui::RichText::new("Last 20 sample intervals").small().color(MUTED));
            });
            egui::Frame::new().fill(BG.gamma_multiply(0.55)).corner_radius(egui::CornerRadius::same(10)).inner_margin(egui::Margin::same(14)).show(&mut columns[1], |ui| {
                ui.label(egui::RichText::new("SAMPLE PROCESSES").small().strong().color(MUTED));
                ui.add_space(8.0);
                for (name, cpu, memory) in [("desktop-shell", "4.8%", "412 MB"), ("music-player", "3.1%", "286 MB"), ("browser", "2.4%", "1.2 GB"), ("file-manager", "0.6%", "96 MB")] {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(name).monospace());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(memory).small().color(MUTED));
                            ui.add_space(12.0);
                            ui.label(egui::RichText::new(cpu).small().color(ACCENT));
                        });
                    });
                    ui.separator();
                }
            });
        });
    }

    fn metric_card(ui: &mut egui::Ui, title: &str, value: &str, amount: f32, color: egui::Color32) {
        egui::Frame::new()
            .fill(BG.gamma_multiply(0.55))
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::same(13))
            .show(ui, |ui| {
                ui.set_min_width(170.0);
                ui.label(egui::RichText::new(title).small().strong().color(MUTED));
                ui.add_space(5.0);
                ui.label(egui::RichText::new(value).size(20.0).strong());
                ui.add(egui::ProgressBar::new(amount).fill(color).desired_width(170.0));
            });
    }

    fn mixer_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("OUTPUT DEVICE").small().strong().color(MUTED));
            egui::ComboBox::from_id_salt("mock-output")
                .selected_text("Studio Speakers · Demo")
                .show_ui(ui, |ui| {
                    let _ = ui.selectable_label(true, "Studio Speakers · Demo");
                    let _ = ui.selectable_label(false, "Headphones · Demo");
                });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("SIMULATED DEVICE").small().color(YELLOW));
            });
        });
        ui.add_space(14.0);
        egui::Frame::new().fill(BG.gamma_multiply(0.55)).corner_radius(egui::CornerRadius::same(10)).inner_margin(egui::Margin::same(14)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Master volume").strong());
                ui.add(egui::Slider::new(&mut self.master_volume, 0.0..=100.0).suffix("%"));
            });
            ui.add_space(8.0);
            for (index, name) in ["Music Player", "Web Browser", "Video Call"].iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(*name).size(14.0));
                    ui.add_space(12.0);
                    ui.add_enabled(!self.muted[index], egui::Slider::new(&mut self.app_volumes[index], 0.0..=100.0).suffix("%"));
                    if ui.selectable_label(self.muted[index], if self.muted[index] { "Muted" } else { "Mute" }).clicked() {
                        self.muted[index] = !self.muted[index];
                    }
                });
            }
        });
        ui.add_space(10.0);
        ui.label(egui::RichText::new("Sliders update this preview only; no audio service is connected.").small().color(MUTED));
    }

    fn focus_panel(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            egui::Frame::new().fill(BG.gamma_multiply(0.55)).corner_radius(egui::CornerRadius::same(12)).inner_margin(egui::Margin::same(18)).show(&mut columns[0], |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("FOCUS SESSION").small().strong().color(ACCENT));
                    ui.add_space(14.0);
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(190.0, 190.0), egui::Sense::hover());
                    ui.painter().circle_stroke(rect.center(), 78.0, egui::Stroke::new(9.0, PANEL));
                    ui.painter().circle_stroke(rect.center(), 78.0, egui::Stroke::new(9.0, ACCENT));
                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, format!("{:02}:00", self.focus_minutes), egui::FontId::proportional(36.0), egui::Color32::WHITE);
                    ui.label(egui::RichText::new(if self.focus_running { "Session in progress · demo" } else { "Ready when you are" }).color(MUTED));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(if self.focus_running { "Ⅱ  Pause" } else { "▶  Start" }).clicked() {
                            self.focus_running = !self.focus_running;
                        }
                        if ui.button("↺ Reset").clicked() {
                            self.focus_running = false;
                            self.focus_minutes = 25;
                        }
                    });
                });
            });
            egui::Frame::new().fill(BG.gamma_multiply(0.55)).corner_radius(egui::CornerRadius::same(12)).inner_margin(egui::Margin::same(18)).show(&mut columns[1], |ui| {
                ui.label(egui::RichText::new("MAKE IT YOURS").small().strong().color(MUTED));
                ui.add_space(10.0);
                ui.label("Session length");
                for minutes in [15, 25, 45] {
                    if ui.selectable_label(self.focus_minutes == minutes, format!("{minutes} minutes")).clicked() {
                        self.focus_minutes = minutes;
                        self.focus_running = false;
                    }
                }
                ui.add_space(18.0);
                ui.label(egui::RichText::new("TODAY · SAMPLE").small().strong().color(MUTED));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    for (day, count) in [("M", 2.0), ("T", 3.0), ("W", 1.0), ("T", 2.0), ("F", 0.0), ("S", 0.0), ("S", 0.0)] {
                        ui.vertical(|ui| {
                            let (rect, _) = ui.allocate_exact_size(egui::vec2(24.0, 50.0), egui::Sense::hover());
                            let bar = egui::Rect::from_min_max(
                                egui::pos2(rect.left(), rect.bottom() - count * 10.0),
                                rect.right_bottom(),
                            );
                            ui.painter().rect_filled(rect, egui::CornerRadius::same(4), PANEL);
                            if count > 0.0 {
                                ui.painter().rect_filled(bar, egui::CornerRadius::same(4), GREEN);
                            }
                            ui.label(egui::RichText::new(day).small().color(MUTED));
                        });
                    }
                });
                ui.add_space(8.0);
                ui.label(egui::RichText::new("8 completed sessions · sample history").small().color(MUTED));
            });
        });
    }
}
