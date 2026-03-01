#![windows_subsystem = "windows"]

use eframe::{egui::{self, Color32, RichText, Vec2}};
use egui::ScrollArea;
use gtfo_codecracker::{load_words_from_str, match_pattern, blend_color};

// const LETTER_SIZE: f32 = 20.0;
const SCROLL_HEIGHT: f32 = 720.0;

const UI_SIZE_EXPANDED: Vec2 = Vec2::new(250.0, 800.0);
const UI_SIZE_COLLAPSED: Vec2 = Vec2::new(64.0, 64.0);

const CSV_DATA: &str = include_str!("../../../gtfo-codecracker/data/gtfo-possible-codes.csv");

pub struct CodeCrackerApp {
    words: Vec<String>,
    pattern: String,
    results: Vec<String>,
    icon: Option<egui::TextureHandle>,
    // --- NEW STATE TRACKING ---
    is_expanded: bool,
    always_on_top: bool,
}

impl Default for CodeCrackerApp {
    fn default() -> Self {
        let words = load_words_from_str(CSV_DATA).unwrap_or_default();
        Self {
            words,
            pattern: String::new(),
            results: Vec::new(),
            icon: None,
            is_expanded: true, // Start expanded
            always_on_top: true,
        }
    }
}

impl eframe::App for CodeCrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Determine if we SHOULD be expanded
        let focused = ctx.input(|i| i.viewport().focused).unwrap_or_default();
        let should_be_expanded = focused || !self.results.is_empty();

        // 2. Only send resize commands when the state CHANGES
        if should_be_expanded != self.is_expanded {
            let new_size = if should_be_expanded { UI_SIZE_EXPANDED } else { UI_SIZE_COLLAPSED };
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(new_size));
            self.is_expanded = should_be_expanded;
        }

        // 3. Handle Always-on-Top logic (can be toggled in UI)
        // Use WindowLevel instead of AlwaysOnTop
        let level = if self.always_on_top {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));

        if self.is_expanded {
            egui::CentralPanel::default().show(ctx, |ui| {
                let color = if self.pattern.is_empty() {
                    Color32::LIGHT_BLUE
                } else if !self.results.is_empty() {
                    blend_color(self.results.len())
                } else {
                    Color32::RED
                };

                ui.horizontal(|ui| {
                    ui.heading(RichText::new("GTFO Cracker").size(22.0).color(color));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Small toggle for Always on Top
                        ui.checkbox(&mut self.always_on_top, "📌");
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Pattern:").size(18.0));
                    
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.pattern)
                            .hint_text("----")
                            .char_limit(4)
                            .font(egui::TextStyle::Heading)
                    );

                    // Auto-focus the input when the window becomes active
                    if focused && self.pattern.is_empty() {
                        response.request_focus();
                    }

                    if response.changed() {
                        self.results = match_pattern(&self.pattern, &self.words);
                    }
                });

                ui.separator();

                if self.pattern.is_empty() {
                    ui.label("Enter 4 letters (use - for unknown)");
                } else if self.results.is_empty() {
                    ui.label(RichText::new("No matches found").color(Color32::RED));
                } else {
                    ui.label(RichText::new(format!("Matches: {}", self.results.len())).color(Color32::GOLD));
                    
                    ScrollArea::vertical()
                        .max_height(SCROLL_HEIGHT)
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            for word in &self.results {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(word).monospace());
                                    if ui.button("📋").on_hover_text("Copy to clipboard").clicked() {
                                        ui.ctx().copy_text(word.clone());
                                    }
                                });
                            }
                        });
                }
            });
        } else {
            // COLLAPSED VIEW: Just show the icon
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(texture) = &self.icon {
                    ui.image(texture);
                } else {
                    ui.label("...");
                }
            });
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // Standard icon loading logic remains the same...
    let icon_bytes = include_bytes!("../../../gtfo-codecracker/data/exec-brute-force.png");
    let image = image::load_from_memory(icon_bytes).expect("Failed to load icon").into_rgba8();
    let (width, height) = image.dimensions();
    let pixels = image.into_raw();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(UI_SIZE_EXPANDED)
            .with_always_on_top()
            .with_decorations(true) // Keeps it looking like a standard app
            .with_transparent(false),
        ..Default::default()
    };

    eframe::run_native(
        "GTFO Code Cracker",
        options,
        Box::new(move |cc| {
            let texture = cc.egui_ctx.load_texture(
                "app_icon",
                egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &pixels),
                egui::TextureOptions::LINEAR,
            );

            Ok(Box::new(CodeCrackerApp {
                words: load_words_from_str(CSV_DATA).unwrap_or_default(),
                pattern: String::new(),
                results: Vec::new(),
                icon: Some(texture),
                is_expanded: true,
                always_on_top: true,
            }))
        }),
    )
}
