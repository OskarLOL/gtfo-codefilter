#![windows_subsystem = "windows"]

use eframe::egui::{self, Color32, RichText, Vec2};
use egui::ScrollArea;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, hotkey::HotKey};
use gtfo_codefilter::{load_words_from_str, match_pattern};

const SCROLL_HEIGHT: f32 = 720.0;

const UI_SIZE_EXPANDED: Vec2 = Vec2::new(250.0, 800.0);
const UI_SIZE_COLLAPSED: Vec2 = Vec2::new(64.0, 64.0);

const CSV_DATA: &str = include_str!("../../data/gtfo-possible-codes.csv");

// ---------------------------------------------------------------------------
// Color helpers (GUI-only, not needed by the library)
// ---------------------------------------------------------------------------

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) + (b as f32 - a as f32) * t) as u8
}

fn blend_color(num: usize) -> Color32 {
    // 1 match = Green, 10+ matches = Yellow
    let t = ((num.saturating_sub(1)) as f32 / 9.0).clamp(0.0, 1.0);

    Color32::from_rgb(
        lerp(0, 255, t), // R: 0 -> 255
        255,             // G: Always 255
        0,               // B: Always 0
    )
}

// ---------------------------------------------------------------------------
// Hotkey configuration
// ---------------------------------------------------------------------------

/// The set of hotkeys the user can pick from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyOption {
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

impl HotkeyOption {
    const ALL: [HotkeyOption; 12] = [
        Self::F1,
        Self::F2,
        Self::F3,
        Self::F4,
        Self::F5,
        Self::F6,
        Self::F7,
        Self::F8,
        Self::F9,
        Self::F10,
        Self::F11,
        Self::F12,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
        }
    }

    fn hotkey_code(self) -> global_hotkey::hotkey::Code {
        use global_hotkey::hotkey::Code;
        match self {
            Self::F1 => Code::F1,
            Self::F2 => Code::F2,
            Self::F3 => Code::F3,
            Self::F4 => Code::F4,
            Self::F5 => Code::F5,
            Self::F6 => Code::F6,
            Self::F7 => Code::F7,
            Self::F8 => Code::F8,
            Self::F9 => Code::F9,
            Self::F10 => Code::F10,
            Self::F11 => Code::F11,
            Self::F12 => Code::F12,
        }
    }

    fn to_global_hotkey(self) -> HotKey {
        HotKey::new(None, self.hotkey_code())
    }
}

pub struct CodeFilterApp {
    words: Vec<String>,
    pattern: String,
    results: Vec<String>,
    icon: Option<egui::TextureHandle>,
    // --- state tracking ---
    is_expanded: bool,
    always_on_top: bool,
    auto_clear: bool,
    focus_input_next_frame: bool,
    /// Previous frame's focus state — used to detect focus transitions.
    was_focused: bool,
    // --- hotkey state ---
    hotkey_manager: GlobalHotKeyManager,
    selected_hotkey: HotkeyOption,
    /// The currently registered hotkey (so we can unregister before re-registering).
    registered_hotkey: Option<HotKey>,
    listening: bool,
}

impl CodeFilterApp {
    fn new(icon: Option<egui::TextureHandle>, hotkey_manager: GlobalHotKeyManager) -> Self {
        let words = load_words_from_str(CSV_DATA).unwrap_or_default();

        let default_hotkey = HotkeyOption::F2;
        let hotkey = default_hotkey.to_global_hotkey();
        let registered = hotkey_manager.register(hotkey).ok().map(|()| hotkey);

        Self {
            words,
            pattern: String::new(),
            results: Vec::new(),
            icon,
            is_expanded: true,
            always_on_top: true,
            auto_clear: true,
            focus_input_next_frame: false,
            was_focused: true, // assume focused on startup since we start expanded
            hotkey_manager,
            selected_hotkey: default_hotkey,
            registered_hotkey: registered,
            listening: true,
        }
    }

    /// Unregister the current hotkey (if any) and register a new one.
    fn update_registered_hotkey(&mut self, new_option: HotkeyOption) {
        // Unregister old
        if let Some(old) = self.registered_hotkey.take() {
            let _ = self.hotkey_manager.unregister(old);
        }

        // Register new (only if listening is enabled)
        if self.listening {
            let hotkey = new_option.to_global_hotkey();
            if self.hotkey_manager.register(hotkey).is_ok() {
                self.registered_hotkey = Some(hotkey);
            }
        }
    }

    /// Enable or disable the hotkey by registering/unregistering it.
    fn set_listening(&mut self, enabled: bool) {
        if enabled && self.registered_hotkey.is_none() {
            let hotkey = self.selected_hotkey.to_global_hotkey();
            if self.hotkey_manager.register(hotkey).is_ok() {
                self.registered_hotkey = Some(hotkey);
            }
        } else if !enabled && let Some(old) = self.registered_hotkey.take() {
            let _ = self.hotkey_manager.unregister(old);
        }
    }
}

impl eframe::App for CodeFilterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let focused = ctx.input(|i| i.viewport().focused).unwrap_or_default();

        // --- Check for global hotkey events (replaces rdev background thread) ---
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv()
            && let Some(registered) = &self.registered_hotkey
            && event.id() == registered.id()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // --- Focus-driven state transitions ---
        let gained_focus = focused && !self.was_focused;
        let lost_focus = !focused && self.was_focused;

        if gained_focus {
            self.is_expanded = true;
            self.focus_input_next_frame = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(UI_SIZE_EXPANDED));
        }

        if lost_focus {
            self.is_expanded = false;
            if self.auto_clear {
                self.pattern.clear();
                self.results.clear();
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(UI_SIZE_COLLAPSED));
        }

        self.was_focused = focused;

        // Esc-to-collapse: minimize the window to trigger the collapse via focus-out
        let esc_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if self.is_expanded && esc_pressed {
            self.is_expanded = false;
            if self.auto_clear {
                self.pattern.clear();
                self.results.clear();
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(UI_SIZE_COLLAPSED));
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }

        // Request a repaint every 100ms so we pick up hotkey events promptly
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // Handle Always-on-Top logic
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
                    ui.heading(RichText::new("GTFO Filter").size(22.0).color(color));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut self.always_on_top, "\u{1f4cc}");
                    });
                });

                ui.horizontal(|ui| {
                    // Visual indicator: green dot when focused and ready for input
                    let indicator_color = if focused {
                        Color32::from_rgb(0, 200, 0)
                    } else {
                        Color32::from_rgb(80, 80, 80)
                    };
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(rect.center(), 5.0, indicator_color);

                    ui.label(RichText::new("Pattern:").size(18.0));

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.pattern)
                            .hint_text("----")
                            .char_limit(4)
                            .font(egui::TextStyle::Heading),
                    );

                    // Focus input on hotkey expand or when empty and window is focused
                    if self.focus_input_next_frame || (focused && self.pattern.is_empty()) {
                        response.request_focus();
                        self.focus_input_next_frame = false;
                    }

                    if response.changed() {
                        self.results = match_pattern(&self.pattern, &self.words);
                    }
                });

                ui.separator();

                // Hotkey selector + listening toggle
                ui.horizontal(|ui| {
                    ui.label("Summon hotkey:");
                    let prev = self.selected_hotkey;
                    egui::ComboBox::from_id_salt("hotkey_selector")
                        .selected_text(self.selected_hotkey.label())
                        .show_ui(ui, |ui| {
                            for option in HotkeyOption::ALL {
                                ui.selectable_value(
                                    &mut self.selected_hotkey,
                                    option,
                                    option.label(),
                                );
                            }
                        });
                    if self.selected_hotkey != prev {
                        self.update_registered_hotkey(self.selected_hotkey);
                    }
                });

                ui.horizontal(|ui| {
                    // Listening toggle — registers/unregisters the global hotkey
                    let listen_label = if self.listening {
                        RichText::new("\u{1f513} Listening").color(Color32::from_rgb(0, 200, 0))
                    } else {
                        RichText::new("\u{1f512} Sleeping").color(Color32::GRAY)
                    };
                    if ui.checkbox(&mut self.listening, listen_label).changed() {
                        self.set_listening(self.listening);
                    }

                    ui.separator();

                    // Auto-clear toggle
                    ui.checkbox(&mut self.auto_clear, "Auto-clear");
                });

                ui.separator();

                if self.pattern.is_empty() {
                    ui.label("Enter 4 letters (use - for unknown)");
                } else if self.results.is_empty() {
                    ui.label(RichText::new("No matches found").color(Color32::RED));
                } else {
                    ui.label(
                        RichText::new(format!("Matches: {}", self.results.len()))
                            .color(Color32::GOLD),
                    );

                    ScrollArea::vertical()
                        .max_height(SCROLL_HEIGHT)
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            for word in &self.results {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(word).monospace());
                                    if ui
                                        .button("\u{1f4cb}")
                                        .on_hover_text("Copy to clipboard")
                                        .clicked()
                                    {
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
    let icon_bytes = include_bytes!("../../data/exec-brute-force.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let pixels = image.into_raw();

    // Set the window/taskbar icon from the same embedded PNG
    let window_icon = egui::IconData {
        rgba: pixels.clone(),
        width,
        height,
    };

    // GlobalHotKeyManager must be created on the main thread (it needs a
    // message loop on Windows). We create it here before entering the eframe
    // event loop so it participates in the same thread's message pump.
    let hotkey_manager =
        GlobalHotKeyManager::new().expect("Failed to initialize global hotkey manager");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(UI_SIZE_EXPANDED)
            .with_always_on_top()
            .with_decorations(true)
            .with_transparent(false)
            .with_icon(std::sync::Arc::new(window_icon)),
        ..Default::default()
    };

    eframe::run_native(
        "GTFO Code Filter",
        options,
        Box::new(move |cc| {
            let texture = cc.egui_ctx.load_texture(
                "app_icon",
                egui::ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    &pixels,
                ),
                egui::TextureOptions::LINEAR,
            );

            Ok(Box::new(CodeFilterApp::new(Some(texture), hotkey_manager)))
        }),
    )
}
