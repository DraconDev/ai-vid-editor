#![cfg(feature = "gui")]

mod theme;

use eframe::egui;
use egui::RichText;
use rfd::FileDialog;
use std::path::PathBuf;

use ai_vid_editor::{Config, JoinMode, Preset, SilenceMode};
use theme::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Tab {
    #[default]
    Watch,
    Manual,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum ProcessingStatus {
    Idle,
    Watching,
    Processing(String),
    Error(String),
}

#[derive(Debug, Clone)]
struct ActivityEntry {
    timestamp: String,
    message: String,
    success: bool,
}

impl ActivityEntry {
    fn new(message: impl Into<String>, success: bool) -> Self {
        let now = chrono::Local::now();
        Self {
            timestamp: now.format("%H:%M:%S").to_string(),
            message: message.into(),
            success,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    config: Config,
    selected_preset: String,
    watch_folder: PathBuf,
    output_folder: PathBuf,
    status: ProcessingStatus,
    activity_log: Vec<ActivityEntry>,
    current_tab: Tab,

    manual_input_files: Vec<PathBuf>,
    manual_output_folder: PathBuf,
}

fn join_mode_display(mode: &JoinMode) -> String {
    match mode {
        JoinMode::Off => "Off".to_string(),
        JoinMode::ByDate => "By Date".to_string(),
        JoinMode::ByName => "By Name".to_string(),
        JoinMode::AfterCount => "After N Files".to_string(),
    }
}

impl AppState {
    fn new() -> Self {
        let config = Config::default();
        let watch_folder = config
            .paths
            .input_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("watch"));
        let output_folder = config
            .paths
            .output_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("output"));

        Self {
            config,
            selected_preset: "youtube".to_string(),
            watch_folder,
            output_folder,
            status: ProcessingStatus::Idle,
            activity_log: Vec::new(),
            current_tab: Tab::Watch,
            manual_input_files: Vec::new(),
            manual_output_folder: PathBuf::from("output"),
        }
    }

    fn load_config(&mut self, path: &PathBuf) {
        match Config::from_file(path) {
            Ok(config) => {
                self.config = config;
                self.watch_folder = self.config.paths.input_dir.clone().unwrap_or_default();
                self.output_folder = self.config.paths.output_dir.clone().unwrap_or_default();
                self.activity_log.push(ActivityEntry::new(
                    format!("Loaded config from {}", path.display()),
                    true,
                ));
            }
            Err(e) => {
                self.activity_log.push(ActivityEntry::new(
                    format!("Failed to load config: {}", e),
                    false,
                ));
            }
        }
    }

    fn save_config(&mut self, path: &PathBuf) {
        self.config.paths.input_dir = Some(self.watch_folder.clone());
        self.config.paths.output_dir = Some(self.output_folder.clone());

        match self.config.to_file(path) {
            Ok(()) => {
                self.activity_log.push(ActivityEntry::new(
                    format!("Saved config to {}", path.display()),
                    true,
                ));
            }
            Err(e) => {
                self.activity_log.push(ActivityEntry::new(
                    format!("Failed to save config: {}", e),
                    false,
                ));
            }
        }
    }

    fn load_preset(&mut self, preset_name: &str) {
        if let Some(preset) = Preset::from_str(preset_name) {
            self.config = Config::from(preset.to_config());
            self.selected_preset = preset_name.to_string();
            self.activity_log.push(ActivityEntry::new(
                format!("Loaded {} preset", preset_name),
                true,
            ));
        } else {
            match Config::from_preset_file(preset_name) {
                Ok(config) => {
                    self.config = config;
                    self.selected_preset = preset_name.to_string();
                    self.activity_log.push(ActivityEntry::new(
                        format!("Loaded {} preset from file", preset_name),
                        true,
                    ));
                }
                Err(e) => {
                    self.activity_log.push(ActivityEntry::new(
                        format!("Failed to load preset: {}", e),
                        false,
                    ));
                }
            }
        }
    }
}

pub struct App {
    state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.draw_header(ui);
                ui.add_space(12.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_tabs(ui);
                    ui.add_space(12.0);
                    self.draw_settings(ui);
                    ui.add_space(12.0);
                    self.draw_activity_log(ui);
                });
            });
        });
    }
}

impl App {
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("AI Video Processor")
                    .size(26.0)
                    .color(ACCENT_PRIMARY),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(theme_button("Save Config")).clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("TOML", &["toml"])
                        .set_file_name("ai-vid-editor.toml")
                        .save_file()
                    {
                        self.state.save_config(&path);
                    }
                }
                if ui.add(theme_button("Load Config")).clicked() {
                    if let Some(path) = FileDialog::new().add_filter("TOML", &["toml"]).pick_file()
                    {
                        self.state.load_config(&path);
                    }
                }
            });
        });
        ui.add_space(8.0);
        ui.label(
            RichText::new("Automated video processing — Configure and let it run")
                .color(TEXT_SECONDARY),
        );
        ui.add_space(4.0);
        ui.add(egui::Separator::default().stroke(egui::Stroke::new(1.0, BORDER)));
    }

    fn draw_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let watch_tab =
                egui::RichText::new("Watch Mode").color(if self.state.current_tab == Tab::Watch {
                    ACCENT_PRIMARY
                } else {
                    TEXT_PRIMARY
                });
            if ui
                .selectable_label(self.state.current_tab == Tab::Watch, watch_tab)
                .clicked()
            {
                self.state.current_tab = Tab::Watch;
            }

            let manual_tab = egui::RichText::new("Manual Process").color(
                if self.state.current_tab == Tab::Manual {
                    ACCENT_PRIMARY
                } else {
                    TEXT_PRIMARY
                },
            );
            if ui
                .selectable_label(self.state.current_tab == Tab::Manual, manual_tab)
                .clicked()
            {
                self.state.current_tab = Tab::Manual;
            }
        });
        ui.add(egui::Separator::default().stroke(egui::Stroke::new(1.0, BORDER)));

        match self.state.current_tab {
            Tab::Watch => self.draw_watch_tab(ui),
            Tab::Manual => self.draw_manual_tab(ui),
        }
    }

    fn draw_watch_tab(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(PANEL_BG)
            .corner_radius(8.0)
            .inner_margin(16.0)
            .stroke(egui::Stroke::new(1.0, BORDER))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Automation Mode")
                        .size(16.0)
                        .color(ACCENT_PRIMARY),
                );
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.add_sized(
                        [100.0, 20.0],
                        egui::Label::new(RichText::new("Watch Folder:").color(TEXT_PRIMARY)),
                    );
                    ui.add_sized(
                        [300.0, 20.0],
                        egui::TextEdit::singleline(
                            &mut self.state.watch_folder.to_string_lossy().to_string(),
                        )
                        .text_color(TEXT_PRIMARY),
                    );
                    if ui.add(theme_button_secondary("Browse...")).clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.state.watch_folder = path;
                        }
                    }
                });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.add_sized(
                        [100.0, 20.0],
                        egui::Label::new(RichText::new("Output Folder:").color(TEXT_PRIMARY)),
                    );
                    ui.add_sized(
                        [300.0, 20.0],
                        egui::TextEdit::singleline(
                            &mut self.state.output_folder.to_string_lossy().to_string(),
                        )
                        .text_color(TEXT_PRIMARY),
                    );
                    if ui.add(theme_button_secondary("Browse...")).clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.state.output_folder = path;
                        }
                    }
                });

                ui.add_space(16.0);

                let is_watching = self.state.status == ProcessingStatus::Watching;
                let button_text = if is_watching {
                    "Stop Watching"
                } else {
                    "Start Watching"
                };

                ui.horizontal(|ui| {
                    if is_watching {
                        if ui.add(theme_button_danger(button_text)).clicked() {
                            self.state.status = ProcessingStatus::Idle;
                            self.state
                                .activity_log
                                .push(ActivityEntry::new("Stopped watching", true));
                        }
                    } else {
                        if ui.add(theme_button_primary(button_text)).clicked() {
                            self.state.status = ProcessingStatus::Watching;
                            self.state
                                .activity_log
                                .push(ActivityEntry::new("Started watching for new videos", true));
                        }
                    }
                });

                ui.add_space(12.0);

                let status_text = match &self.state.status {
                    ProcessingStatus::Idle => "Ready".to_string(),
                    ProcessingStatus::Watching => "Watching for new videos...".to_string(),
                    ProcessingStatus::Processing(msg) => format!("Processing: {}", msg),
                    ProcessingStatus::Error(msg) => format!("Error: {}", msg),
                };

                let status_color = match &self.state.status {
                    ProcessingStatus::Idle => TEXT_SECONDARY,
                    ProcessingStatus::Watching => ACCENT_PRIMARY,
                    ProcessingStatus::Processing(_) => ACCENT_SECONDARY,
                    ProcessingStatus::Error(_) => ERROR,
                };

                ui.horizontal(|ui| {
                    if self.state.status == ProcessingStatus::Watching {
                        ui.spinner();
                    }
                    ui.label(RichText::new(status_text).color(status_color));
                });
            });
    }

    fn draw_manual_tab(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(PANEL_BG)
            .corner_radius(8.0)
            .inner_margin(16.0)
            .stroke(egui::Stroke::new(1.0, BORDER))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Manual Processing")
                        .size(16.0)
                        .color(ACCENT_PRIMARY),
                );
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.add(theme_button_secondary("Select Files...")).clicked() {
                        if let Some(paths) = FileDialog::new()
                            .add_filter("Video", &["mp4", "mov", "avi", "mkv", "webm"])
                            .pick_files()
                        {
                            self.state.manual_input_files = paths;
                            self.state.activity_log.push(ActivityEntry::new(
                                format!("Selected {} file(s)", self.state.manual_input_files.len()),
                                true,
                            ));
                        }
                    }
                    if ui.add(theme_button_secondary("Clear")).clicked() {
                        self.state.manual_input_files.clear();
                    }
                });

                ui.add_space(8.0);

                if self.state.manual_input_files.is_empty() {
                    ui.label(RichText::new("No files selected").color(TEXT_SECONDARY));
                } else {
                    ui.label(
                        RichText::new(format!(
                            "{} file(s) selected:",
                            self.state.manual_input_files.len()
                        ))
                        .color(TEXT_PRIMARY),
                    );
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for path in &self.state.manual_input_files {
                                ui.label(
                                    RichText::new(
                                        path.file_name().unwrap_or_default().to_string_lossy(),
                                    )
                                    .color(TEXT_SECONDARY)
                                    .monospace(),
                                );
                            }
                        });
                }

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.add_sized(
                        [100.0, 20.0],
                        egui::Label::new(RichText::new("Output Folder:").color(TEXT_PRIMARY)),
                    );
                    ui.add_sized(
                        [300.0, 20.0],
                        egui::TextEdit::singleline(
                            &mut self
                                .state
                                .manual_output_folder
                                .to_string_lossy()
                                .to_string(),
                        )
                        .text_color(TEXT_PRIMARY),
                    );
                    if ui.add(theme_button_secondary("Browse...")).clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.state.manual_output_folder = path;
                        }
                    }
                });

                ui.add_space(16.0);

                let can_process = !self.state.manual_input_files.is_empty()
                    && !self.state.manual_output_folder.as_os_str().is_empty();

                if ui
                    .add_enabled(can_process, theme_button_primary("Process Files"))
                    .clicked()
                {
                    self.state.activity_log.push(ActivityEntry::new(
                        format!(
                            "Processing {} file(s)...",
                            self.state.manual_input_files.len()
                        ),
                        true,
                    ));
                }
            });
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(PANEL_BG)
            .corner_radius(8.0)
            .inner_margin(16.0)
            .stroke(egui::Stroke::new(1.0, BORDER))
            .show(ui, |ui| {
                ui.label(RichText::new("Settings").size(16.0).color(ACCENT_PRIMARY));
                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    ui.add_sized(
                        [60.0, 20.0],
                        egui::Label::new(RichText::new("Preset:").color(TEXT_PRIMARY)),
                    );
                    egui::ComboBox::from_id_salt("preset_combo")
                        .selected_text(
                            RichText::new(&self.state.selected_preset).color(TEXT_PRIMARY),
                        )
                        .width(150.0)
                        .show_ui(ui, |ui| {
                            let presets = Config::available_presets();
                            for preset in presets {
                                let is_selected = self.state.selected_preset == preset;
                                if ui
                                    .selectable_label(
                                        is_selected,
                                        RichText::new(&preset).color(TEXT_PRIMARY),
                                    )
                                    .clicked()
                                {
                                    self.state.load_preset(&preset);
                                }
                            }
                        });
                });

                ui.add_space(12.0);

                ui.columns(2, |columns| {
                    columns[0].vertical(|ui| {
                        ui.label(
                            RichText::new("Processing Options")
                                .strong()
                                .color(TEXT_PRIMARY),
                        );
                        ui.add_space(4.0);

                        ui.checkbox(
                            &mut self.state.config.audio.enhance,
                            RichText::new("Enhance Audio").color(TEXT_PRIMARY),
                        );
                        ui.checkbox(
                            &mut self.state.config.video.stabilize,
                            RichText::new("Stabilize Video").color(TEXT_PRIMARY),
                        );
                        ui.checkbox(
                            &mut self.state.config.video.color_correct,
                            RichText::new("Color Correct").color(TEXT_PRIMARY),
                        );
                        ui.checkbox(
                            &mut self.state.config.video.reframe,
                            RichText::new("Auto-Reframe (9:16)").color(TEXT_PRIMARY),
                        );
                        ui.checkbox(
                            &mut self.state.config.video.blur_background,
                            RichText::new("Blur Background").color(TEXT_PRIMARY),
                        );
                    });

                    columns[1].vertical(|ui| {
                        ui.label(RichText::new("Silence Mode").strong().color(TEXT_PRIMARY));
                        ui.add_space(4.0);

                        let mut is_cut = self.state.config.silence.mode == SilenceMode::Cut;
                        if ui
                            .checkbox(
                                &mut is_cut,
                                RichText::new("Remove Silence (Cut)").color(TEXT_PRIMARY),
                            )
                            .changed()
                        {
                            self.state.config.silence.mode = if is_cut {
                                SilenceMode::Cut
                            } else {
                                SilenceMode::Speedup
                            };
                        }

                        let mut is_speedup = self.state.config.silence.mode == SilenceMode::Speedup;
                        if ui
                            .checkbox(
                                &mut is_speedup,
                                RichText::new("Speedup Silences").color(TEXT_PRIMARY),
                            )
                            .changed()
                        {
                            self.state.config.silence.mode = if is_speedup {
                                SilenceMode::Speedup
                            } else {
                                SilenceMode::Cut
                            };
                        }

                        ui.add_space(8.0);
                        ui.label(RichText::new("Join Mode").strong().color(TEXT_PRIMARY));
                        ui.add_space(4.0);

                        egui::ComboBox::from_id_salt("join_combo")
                            .selected_text(
                                RichText::new(join_mode_display(
                                    &self.state.config.processing.join_mode,
                                ))
                                .color(TEXT_PRIMARY),
                            )
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.state.config.processing.join_mode,
                                    JoinMode::Off,
                                    RichText::new("Off").color(TEXT_PRIMARY),
                                );
                                ui.selectable_value(
                                    &mut self.state.config.processing.join_mode,
                                    JoinMode::ByDate,
                                    RichText::new("By Date").color(TEXT_PRIMARY),
                                );
                                ui.selectable_value(
                                    &mut self.state.config.processing.join_mode,
                                    JoinMode::ByName,
                                    RichText::new("By Name").color(TEXT_PRIMARY),
                                );
                                ui.selectable_value(
                                    &mut self.state.config.processing.join_mode,
                                    JoinMode::AfterCount,
                                    RichText::new("After N Files").color(TEXT_PRIMARY),
                                );
                            });

                        if self.state.config.processing.join_mode == JoinMode::AfterCount {
                            ui.add_space(4.0);
                            ui.add(
                                egui::Slider::new(
                                    &mut self.state.config.processing.join_after_count,
                                    1..=20,
                                )
                                .text("Files")
                                .text_color(TEXT_PRIMARY),
                            );
                        }
                    });
                });

                ui.add_space(12.0);

                ui.columns(2, |columns| {
                    columns[0].horizontal(|ui| {
                        ui.label(RichText::new("Silence Threshold (dB):").color(TEXT_SECONDARY));
                        ui.add(
                            egui::Slider::new(
                                &mut self.state.config.silence.threshold_db,
                                -60.0..=-10.0,
                            )
                            .step_by(1.0)
                            .text_color(TEXT_PRIMARY),
                        );
                    });

                    columns[1].horizontal(|ui| {
                        ui.label(RichText::new("Target LUFS:").color(TEXT_SECONDARY));
                        ui.add(
                            egui::Slider::new(
                                &mut self.state.config.audio.target_lufs,
                                -24.0..=-6.0,
                            )
                            .step_by(1.0)
                            .text_color(TEXT_PRIMARY),
                        );
                    });
                });
            });
    }

    fn draw_activity_log(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .fill(PANEL_BG)
            .corner_radius(8.0)
            .inner_margin(16.0)
            .stroke(egui::Stroke::new(1.0, BORDER))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Activity Log").size(16.0).color(ACCENT_PRIMARY));
                    if ui.add(theme_button_secondary("Clear")).clicked() {
                        self.state.activity_log.clear();
                    }
                });
                ui.add_space(12.0);
                
                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    if self.state.activity_log.is_empty() {
                        ui.label(RichText::new("No activity yet. Start watching or process files to see activity.").color(TEXT_SECONDARY));
                    } else {
                        for entry in self.state.activity_log.iter().rev().take(50) {
                            let color = if entry.success { SUCCESS } else { ERROR };
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&entry.timestamp).color(TEXT_MUTED).monospace());
                                ui.label(RichText::new(&entry.message).color(color));
                            });
                        }
                    }
                });
            });
    }
}
