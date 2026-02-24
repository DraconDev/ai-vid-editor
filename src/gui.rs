#![cfg(feature = "gui")]

use eframe::egui;
use egui::RichText;
use rfd::FileDialog;
use std::path::PathBuf;

use crate::config::{Config, Preset, SilenceMode};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Tab {
    #[default]
    Watch,
    Manual,
}

#[derive(Debug, Clone, PartialEq)]
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

    join_mode: JoinMode,
    join_after_count: u32,

    manual_input_files: Vec<PathBuf>,
    manual_output_folder: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum JoinMode {
    Off,
    ByDate,
    ByName,
    AfterCount,
}

impl Default for JoinMode {
    fn default() -> Self {
        Self::Off
    }
}

impl std::fmt::Display for JoinMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinMode::Off => write!(f, "Off"),
            JoinMode::ByDate => write!(f, "By Date"),
            JoinMode::ByName => write!(f, "By Name"),
            JoinMode::AfterCount => write!(f, "After N Files"),
        }
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
            join_mode: JoinMode::Off,
            join_after_count: 5,
            manual_input_files: Vec::new(),
            manual_output_folder: PathBuf::from("output"),
        }
    }

    fn load_preset(&mut self, preset_name: &str) {
        if let Some(preset) = Preset::from_str(preset_name) {
            self.config = preset.to_config();
            self.selected_preset = preset_name.to_string();
            self.activity_log.push(ActivityEntry::new(
                format!("Loaded {} preset", preset_name),
                true,
            ));
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
                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.draw_tabs(ui);
                    ui.add_space(10.0);
                    self.draw_settings(ui);
                    ui.add_space(10.0);
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
                    .size(24.0)
                    .color(egui::Color32::from_rgb(78, 204, 163)),
            );
        });
        ui.add_space(5.0);
        ui.label(
            RichText::new("Automated video processing - Configure and let it run")
                .color(egui::Color32::GRAY),
        );
        ui.separator();
    }

    fn draw_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.state.current_tab, Tab::Watch, "Watch Mode");
            ui.selectable_value(&mut self.state.current_tab, Tab::Manual, "Manual Process");
        });
        ui.separator();

        match self.state.current_tab {
            Tab::Watch => self.draw_watch_tab(ui),
            Tab::Manual => self.draw_manual_tab(ui),
        }
    }

    fn draw_watch_tab(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 33, 62))
            .rounding(8.0)
            .inner_margin(15.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Automation Mode")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(78, 204, 163)),
                );
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Watch Folder:");
                    ui.text_edit_singleline(
                        &mut self.state.watch_folder.to_string_lossy().to_string(),
                    );
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.state.watch_folder = path;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Output Folder:");
                    ui.text_edit_singleline(
                        &mut self.state.output_folder.to_string_lossy().to_string(),
                    );
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.state.output_folder = path;
                        }
                    }
                });

                ui.add_space(10.0);

                let is_watching = self.state.status == ProcessingStatus::Watching;
                let button_text = if is_watching {
                    "Stop Watching"
                } else {
                    "Start Watching"
                };
                let button_color = if is_watching {
                    egui::Color32::from_rgb(233, 69, 96)
                } else {
                    egui::Color32::from_rgb(78, 204, 163)
                };

                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(button_text).color(egui::Color32::BLACK),
                            )
                            .fill(button_color)
                            .min_size(egui::vec2(150.0, 35.0)),
                        )
                        .clicked()
                    {
                        if is_watching {
                            self.state.status = ProcessingStatus::Idle;
                            self.state
                                .activity_log
                                .push(ActivityEntry::new("Stopped watching", true));
                        } else {
                            self.state.status = ProcessingStatus::Watching;
                            self.state
                                .activity_log
                                .push(ActivityEntry::new("Started watching for new videos", true));
                        }
                    }
                });

                ui.add_space(10.0);

                let status_text = match &self.state.status {
                    ProcessingStatus::Idle => "Ready".to_string(),
                    ProcessingStatus::Watching => "Watching for new videos...".to_string(),
                    ProcessingStatus::Processing(msg) => format!("Processing: {}", msg),
                    ProcessingStatus::Error(msg) => format!("Error: {}", msg),
                };

                let status_color = match &self.state.status {
                    ProcessingStatus::Idle => egui::Color32::GRAY,
                    ProcessingStatus::Watching => egui::Color32::from_rgb(78, 204, 163),
                    ProcessingStatus::Processing(_) => egui::Color32::from_rgb(255, 217, 61),
                    ProcessingStatus::Error(_) => egui::Color32::from_rgb(233, 69, 96),
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
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 33, 62))
            .rounding(8.0)
            .inner_margin(15.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Manual Processing")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(78, 204, 163)),
                );
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Select Files...").clicked() {
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
                    if ui.button("Clear").clicked() {
                        self.state.manual_input_files.clear();
                    }
                });

                ui.add_space(5.0);

                if self.state.manual_input_files.is_empty() {
                    ui.label(RichText::new("No files selected").color(egui::Color32::GRAY));
                } else {
                    ui.label(format!(
                        "{} file(s) selected:",
                        self.state.manual_input_files.len()
                    ));
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for path in &self.state.manual_input_files {
                                ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                            }
                        });
                }

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Output Folder:");
                    ui.text_edit_singleline(
                        &mut self
                            .state
                            .manual_output_folder
                            .to_string_lossy()
                            .to_string(),
                    );
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.state.manual_output_folder = path;
                        }
                    }
                });

                ui.add_space(10.0);

                let can_process = !self.state.manual_input_files.is_empty()
                    && !self.state.manual_output_folder.as_os_str().is_empty();

                if ui
                    .add_enabled(
                        can_process,
                        egui::Button::new(
                            RichText::new("Process Files").color(egui::Color32::BLACK),
                        )
                        .fill(egui::Color32::from_rgb(78, 204, 163))
                        .min_size(egui::vec2(150.0, 35.0)),
                    )
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
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 33, 62))
            .rounding(8.0)
            .inner_margin(15.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Settings")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(78, 204, 163)),
                );
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Preset:");
                    egui::ComboBox::from_label("")
                        .selected_text(self.state.selected_preset.clone())
                        .show_ui(ui, |ui| {
                            for preset in ["youtube", "shorts", "podcast", "minimal"] {
                                if ui
                                    .selectable_value(
                                        &mut self.state.selected_preset,
                                        preset.to_string(),
                                        preset,
                                    )
                                    .changed()
                                {
                                    self.state.load_preset(preset);
                                }
                            }
                        });
                });

                ui.add_space(10.0);

                ui.columns(2, |columns| {
                    columns[0].vertical(|ui| {
                        ui.label(RichText::new("Processing Options").strong());
                        ui.checkbox(&mut self.state.config.audio.enhance, "Enhance Audio");
                        ui.checkbox(&mut self.state.config.video.stabilize, "Stabilize Video");
                        ui.checkbox(&mut self.state.config.video.color_correct, "Color Correct");
                        ui.checkbox(&mut self.state.config.video.reframe, "Auto-Reframe (9:16)");
                        ui.checkbox(
                            &mut self.state.config.video.blur_background,
                            "Blur Background",
                        );
                    });

                    columns[1].vertical(|ui| {
                        ui.label(RichText::new("Silence Mode").strong());
                        let mut is_cut = self.state.config.silence.mode == SilenceMode::Cut;
                        if ui.checkbox(&mut is_cut, "Remove Silence (Cut)").changed() {
                            self.state.config.silence.mode = if is_cut {
                                SilenceMode::Cut
                            } else {
                                SilenceMode::Speedup
                            };
                        }

                        let mut is_speedup = self.state.config.silence.mode == SilenceMode::Speedup;
                        if ui.checkbox(&mut is_speedup, "Speedup Silences").changed() {
                            self.state.config.silence.mode = if is_speedup {
                                SilenceMode::Speedup
                            } else {
                                SilenceMode::Cut
                            };
                        }

                        ui.add_space(5.0);
                        ui.label(RichText::new("Join Mode").strong());
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{}", self.state.join_mode))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.state.join_mode,
                                    JoinMode::Off,
                                    "Off",
                                );
                                ui.selectable_value(
                                    &mut self.state.join_mode,
                                    JoinMode::ByDate,
                                    "By Date",
                                );
                                ui.selectable_value(
                                    &mut self.state.join_mode,
                                    JoinMode::ByName,
                                    "By Name",
                                );
                                ui.selectable_value(
                                    &mut self.state.join_mode,
                                    JoinMode::AfterCount,
                                    "After N Files",
                                );
                            });

                        if self.state.join_mode == JoinMode::AfterCount {
                            ui.add(
                                egui::Slider::new(&mut self.state.join_after_count, 1..=20)
                                    .text("Files"),
                            );
                        }
                    });
                });

                ui.add_space(10.0);

                ui.columns(2, |columns| {
                    columns[0].horizontal(|ui| {
                        ui.label("Silence Threshold (dB):");
                        ui.add(
                            egui::Slider::new(
                                &mut self.state.config.silence.threshold_db,
                                -60.0..=-10.0,
                            )
                            .step_by(1.0),
                        );
                    });

                    columns[1].horizontal(|ui| {
                        ui.label("Target LUFS:");
                        ui.add(
                            egui::Slider::new(
                                &mut self.state.config.audio.target_lufs,
                                -24.0..=-6.0,
                            )
                            .step_by(1.0),
                        );
                    });
                });
            });
    }

    fn draw_activity_log(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 33, 62))
            .rounding(8.0)
            .inner_margin(15.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Activity Log").size(16.0).color(egui::Color32::from_rgb(78, 204, 163)));
                    if ui.button("Clear").clicked() {
                        self.state.activity_log.clear();
                    }
                });
                ui.add_space(10.0);
                
                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    if self.state.activity_log.is_empty() {
                        ui.label(RichText::new("No activity yet. Start watching or process files to see activity.").color(egui::Color32::GRAY));
                    } else {
                        for entry in self.state.activity_log.iter().rev().take(50) {
                            let color = if entry.success {
                                egui::Color32::from_rgb(78, 204, 163)
                            } else {
                                egui::Color32::from_rgb(233, 69, 96)
                            };
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&entry.timestamp).color(egui::Color32::GRAY).monospace());
                                ui.label(RichText::new(&entry.message).color(color));
                            });
                        }
                    }
                });
            });
    }
}
