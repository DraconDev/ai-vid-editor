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
enum ProcessingStatus {
    Idle,
    Watching,
    Processing(String),
    Error(String),
}

#[derive(Debug, Clone)]
struct ActivityEntry {
    timestamp: String,
    filename: String,
    file_size: u64,
    duration: Option<u64>,
    progress: Option<f32>,
    status: EntryStatus,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EntryStatus {
    Success,
    Processing,
    Error,
}

impl ActivityEntry {
    fn success(filename: impl Into<String>, file_size: u64, duration: u64) -> Self {
        let now = chrono::Local::now();
        Self {
            timestamp: now.format("%H:%M:%S").to_string(),
            filename: filename.into(),
            file_size,
            duration: Some(duration),
            progress: None,
            status: EntryStatus::Success,
            message: String::new(),
        }
    }

    fn processing(filename: impl Into<String>, file_size: u64, progress: f32) -> Self {
        let now = chrono::Local::now();
        Self {
            timestamp: now.format("%H:%M:%S").to_string(),
            filename: filename.into(),
            file_size,
            duration: None,
            progress: Some(progress),
            status: EntryStatus::Processing,
            message: String::new(),
        }
    }

    fn error(filename: impl Into<String>, message: impl Into<String>) -> Self {
        let now = chrono::Local::now();
        Self {
            timestamp: now.format("%H:%M:%S").to_string(),
            filename: filename.into(),
            file_size: 0,
            duration: None,
            progress: None,
            status: EntryStatus::Error,
            message: message.into(),
        }
    }

    fn simple(message: impl Into<String>, success: bool) -> Self {
        let now = chrono::Local::now();
        Self {
            timestamp: now.format("%H:%M:%S").to_string(),
            filename: String::new(),
            file_size: 0,
            duration: None,
            progress: None,
            status: if success {
                EntryStatus::Success
            } else {
                EntryStatus::Error
            },
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct FileEntry {
    path: PathBuf,
    size: u64,
}

impl FileEntry {
    fn from_path(path: PathBuf) -> Self {
        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        Self { path, size }
    }

    fn filename(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
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
    manual_files: Vec<FileEntry>,
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
            activity_log: vec![ActivityEntry::simple(
                "Welcome! Configure settings and start processing.",
                true,
            )],
            current_tab: Tab::Watch,
            manual_files: Vec::new(),
            manual_output_folder: PathBuf::from("output"),
        }
    }

    fn load_config(&mut self, path: &PathBuf) {
        match Config::from_file(path) {
            Ok(config) => {
                self.config = config;
                self.watch_folder = self.config.paths.input_dir.clone().unwrap_or_default();
                self.output_folder = self.config.paths.output_dir.clone().unwrap_or_default();
                self.activity_log.push(ActivityEntry::simple(
                    format!("Loaded config from {}", path.display()),
                    true,
                ));
            }
            Err(e) => {
                self.activity_log.push(ActivityEntry::simple(
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
                self.activity_log.push(ActivityEntry::simple(
                    format!("Saved config to {}", path.display()),
                    true,
                ));
            }
            Err(e) => {
                self.activity_log.push(ActivityEntry::simple(
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
            self.activity_log.push(ActivityEntry::simple(
                format!("Loaded {} preset", preset_name),
                true,
            ));
        } else {
            match Config::from_preset_file(preset_name) {
                Ok(config) => {
                    self.config = config;
                    self.selected_preset = preset_name.to_string();
                    self.activity_log.push(ActivityEntry::simple(
                        format!("Loaded {} preset from file", preset_name),
                        true,
                    ));
                }
                Err(e) => {
                    self.activity_log.push(ActivityEntry::simple(
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

                ui.horizontal(|ui| {
                    ui.set_min_width(ui.available_width());

                    let left_width = (ui.available_width() * 0.55).min(500.0);
                    let right_width = ui.available_width() - left_width - 16.0;

                    ui.allocate_ui_with_layout(
                        egui::vec2(left_width, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            self.draw_main_panel(ui);
                        },
                    );

                    ui.add_space(16.0);

                    ui.allocate_ui_with_layout(
                        egui::vec2(right_width, ui.available_height()),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            self.draw_settings_panel(ui);
                        },
                    );
                });

                ui.add_space(12.0);
                self.draw_activity_log(ui);
            });
        });
    }
}

impl App {
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⚡ AI Video Processor")
                    .size(22.0)
                    .color(ACCENT_PRIMARY)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(button_secondary("Save")).clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("TOML", &["toml"])
                        .set_file_name("ai-vid-editor.toml")
                        .save_file()
                    {
                        self.state.save_config(&path);
                    }
                }
                if ui.add(button_secondary("Load")).clicked() {
                    if let Some(path) = FileDialog::new().add_filter("TOML", &["toml"]).pick_file()
                    {
                        self.state.load_config(&path);
                    }
                }
            });
        });
        ui.add_space(4.0);
        ui.label(label_secondary("Automated video processing"));
    }

    fn draw_main_panel(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                let watch_text = if self.state.current_tab == Tab::Watch {
                    RichText::new("👁 Watch").color(ACCENT_PRIMARY).strong()
                } else {
                    RichText::new("👁 Watch").color(TEXT_SECONDARY)
                };
                if ui
                    .selectable_label(self.state.current_tab == Tab::Watch, watch_text)
                    .clicked()
                {
                    self.state.current_tab = Tab::Watch;
                }

                let manual_text = if self.state.current_tab == Tab::Manual {
                    RichText::new("📁 Manual").color(ACCENT_PRIMARY).strong()
                } else {
                    RichText::new("📁 Manual").color(TEXT_SECONDARY)
                };
                if ui
                    .selectable_label(self.state.current_tab == Tab::Manual, manual_text)
                    .clicked()
                {
                    self.state.current_tab = Tab::Manual;
                }
            });

            ui.add_space(12.0);

            match self.state.current_tab {
                Tab::Watch => self.draw_watch_content(ui),
                Tab::Manual => self.draw_manual_content(ui),
            }
        });
    }

    fn draw_watch_content(&mut self, ui: &mut egui::Ui) {
        ui.label(section_header("📂 Watch Folder"));
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.add_sized(
                egui::vec2(ui.available_width() - 80.0, 24.0),
                text_edit_style(&mut self.state.watch_folder.to_string_lossy().to_string()),
            );
            if ui.add(button_secondary("Browse")).clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    self.state.watch_folder = path;
                }
            }
        });

        ui.add_space(12.0);

        ui.label(section_header("📤 Output Folder"));
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.add_sized(
                egui::vec2(ui.available_width() - 80.0, 24.0),
                text_edit_style(&mut self.state.output_folder.to_string_lossy().to_string()),
            );
            if ui.add(button_secondary("Browse")).clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    self.state.output_folder = path;
                }
            }
        });

        ui.add_space(16.0);

        let (status_text, status_color) = match &self.state.status {
            ProcessingStatus::Idle => ("Ready", TEXT_MUTED),
            ProcessingStatus::Watching => ("Active", SUCCESS),
            ProcessingStatus::Processing(_) => ("Processing", WARNING),
            ProcessingStatus::Error(_) => ("Error", ERROR),
        };

        inner_panel().show(ui, |ui| {
            ui.label(status_badge(status_text, status_color));
        });

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let is_watching = self.state.status == ProcessingStatus::Watching;
                    if is_watching {
                        if ui.add(button_danger("■ Stop Watching")).clicked() {
                            self.state.status = ProcessingStatus::Idle;
                            self.state
                                .activity_log
                                .push(ActivityEntry::simple("Stopped watching", true));
                        }
                    } else {
                        if ui.add(button_primary("▶ Start Watching")).clicked() {
                            self.state.status = ProcessingStatus::Watching;
                            self.state.activity_log.push(ActivityEntry::simple(
                                "Started watching for new videos",
                                true,
                            ));
                        }
                    }
                },
            );
        });
    }

    fn draw_manual_content(&mut self, ui: &mut egui::Ui) {
        let file_count = self.state.manual_files.len();

        ui.horizontal(|ui| {
            ui.label(section_header(&format!(
                "📹 Selected Files ({})",
                file_count
            )));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(button_secondary("Clear")).clicked() {
                    self.state.manual_files.clear();
                }
            });
        });

        ui.add_space(8.0);

        if self.state.manual_files.is_empty() {
            inner_panel().show(ui, |ui| {
                ui.label(label_secondary("No files selected"));
            });
        } else {
            let max_height = (file_count.min(5) * 28) as f32 + 16.0;
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .show(ui, |ui| {
                    for file in &self.state.manual_files {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📹").size(12.0));
                            ui.label(label_primary(file.filename()));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(label_muted(&format_file_size(file.size)));
                                },
                            );
                        });
                        ui.add_space(4.0);
                    }
                });
        }

        ui.add_space(12.0);

        if ui.add(button_secondary("+ Add Files")).clicked() {
            if let Some(paths) = FileDialog::new()
                .add_filter("Video", &["mp4", "mov", "avi", "mkv", "webm"])
                .pick_files()
            {
                for path in paths {
                    self.state.manual_files.push(FileEntry::from_path(path));
                }
                self.state.activity_log.push(ActivityEntry::simple(
                    format!("Added {} file(s)", self.state.manual_files.len()),
                    true,
                ));
            }
        }

        ui.add_space(12.0);

        ui.label(section_header("📤 Output Folder"));
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.add_sized(
                egui::vec2(ui.available_width() - 80.0, 24.0),
                text_edit_style(
                    &mut self
                        .state
                        .manual_output_folder
                        .to_string_lossy()
                        .to_string(),
                ),
            );
            if ui.add(button_secondary("Browse")).clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    self.state.manual_output_folder = path;
                }
            }
        });

        ui.add_space(16.0);

        let can_process = !self.state.manual_files.is_empty();
        ui.horizontal(|ui| {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let btn = button_primary(&format!(
                        "⚡ Process {} File{}",
                        file_count,
                        if file_count == 1 { "" } else { "s" }
                    ));
                    if ui.add_enabled(can_process, btn).clicked() {
                        self.state.activity_log.push(ActivityEntry::simple(
                            format!("Started processing {} file(s)", file_count),
                            true,
                        ));
                    }
                },
            );
        });
    }

    fn draw_settings_panel(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.label(
                RichText::new("⚙ Settings")
                    .size(16.0)
                    .color(ACCENT_PRIMARY)
                    .strong(),
            );

            ui.add_space(12.0);

            ui.label(label_secondary("📋 Preset"));
            ui.add_space(4.0);

            egui::ComboBox::from_id_salt("preset")
                .selected_text(RichText::new(&self.state.selected_preset).color(TEXT_PRIMARY))
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    let presets = Config::available_presets();
                    for preset in presets {
                        if ui
                            .selectable_value(
                                &mut self.state.selected_preset,
                                preset.clone(),
                                RichText::new(&preset).color(TEXT_PRIMARY),
                            )
                            .changed()
                        {
                            self.state.load_preset(&preset);
                        }
                    }
                });

            ui.add_space(16.0);
            ui.label(label_muted("--- Processing ---"));
            ui.add_space(8.0);

            let mut enhance = self.state.config.audio.enhance;
            if ui
                .checkbox(
                    &mut enhance,
                    RichText::new("🔊 Enhance Audio").color(TEXT_PRIMARY),
                )
                .changed()
            {
                self.state.config.audio.enhance = enhance;
            }

            let mut remove_silence = self.state.config.silence.mode == SilenceMode::Cut;
            if ui
                .checkbox(
                    &mut remove_silence,
                    RichText::new("🔇 Remove Silence").color(TEXT_PRIMARY),
                )
                .changed()
            {
                self.state.config.silence.mode = if remove_silence {
                    SilenceMode::Cut
                } else {
                    SilenceMode::Speedup
                };
            }

            let mut stabilize = self.state.config.video.stabilize;
            if ui
                .checkbox(
                    &mut stabilize,
                    RichText::new("🎥 Stabilize Video").color(TEXT_PRIMARY),
                )
                .changed()
            {
                self.state.config.video.stabilize = stabilize;
            }

            let mut color_correct = self.state.config.video.color_correct;
            if ui
                .checkbox(
                    &mut color_correct,
                    RichText::new("🎨 Color Correct").color(TEXT_PRIMARY),
                )
                .changed()
            {
                self.state.config.video.color_correct = color_correct;
            }

            let mut reframe = self.state.config.video.reframe;
            if ui
                .checkbox(
                    &mut reframe,
                    RichText::new("📐 Auto-Reframe (9:16)").color(TEXT_PRIMARY),
                )
                .changed()
            {
                self.state.config.video.reframe = reframe;
            }

            let mut blur = self.state.config.video.blur_background;
            if ui
                .checkbox(
                    &mut blur,
                    RichText::new("💫 Blur Background").color(TEXT_PRIMARY),
                )
                .changed()
            {
                self.state.config.video.blur_background = blur;
            }

            ui.add_space(16.0);
            ui.label(label_muted("--- Advanced ---"));
            ui.add_space(8.0);

            ui.label(label_secondary("Silence Threshold (dB)"));
            ui.add_space(4.0);
            ui.add(
                egui::Slider::new(&mut self.state.config.silence.threshold_db, -60.0..=-10.0)
                    .step_by(1.0),
            );

            ui.add_space(8.0);

            ui.label(label_secondary("Target LUFS"));
            ui.add_space(4.0);
            ui.add(
                egui::Slider::new(&mut self.state.config.audio.target_lufs, -24.0..=-6.0)
                    .step_by(1.0),
            );

            ui.add_space(8.0);

            ui.label(label_secondary("🔗 Join Mode"));
            ui.add_space(4.0);

            egui::ComboBox::from_id_salt("join")
                .selected_text(join_mode_display(&self.state.config.processing.join_mode))
                .width(ui.available_width())
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
                ui.add_space(8.0);
                ui.add(
                    egui::Slider::new(&mut self.state.config.processing.join_after_count, 1..=20)
                        .text("Files"),
                );
            }
        });
    }

    fn draw_activity_log(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("📋 Activity Log")
                        .size(16.0)
                        .color(ACCENT_PRIMARY)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(button_secondary("Clear")).clicked() {
                        self.state.activity_log.clear();
                    }
                });
            });

            ui.add_space(12.0);

            if self.state.activity_log.is_empty() {
                inner_panel().show(ui, |ui| {
                    ui.label(label_secondary("No activity yet"));
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .show(ui, |ui| {
                        for entry in self.state.activity_log.iter().rev().take(50) {
                            self.draw_log_entry(ui, entry);
                            ui.add_space(4.0);
                        }
                    });
            }
        });
    }

    fn draw_log_entry(&self, ui: &mut egui::Ui, entry: &ActivityEntry) {
        inner_panel().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(label_muted(&entry.timestamp));

                let (icon, color) = match entry.status {
                    EntryStatus::Success => ("✓", SUCCESS),
                    EntryStatus::Processing => ("⏳", PROCESSING),
                    EntryStatus::Error => ("✗", ERROR),
                };
                ui.label(RichText::new(icon).color(color).size(12.0));

                if !entry.filename.is_empty() {
                    ui.label(label_primary(&entry.filename));

                    if entry.file_size > 0 {
                        ui.label(label_muted(&format_file_size(entry.file_size)));
                    }

                    if let Some(dur) = entry.duration {
                        ui.label(label_muted(&format_duration(dur)));
                    }
                }

                if !entry.message.is_empty() {
                    ui.label(label_secondary(&entry.message));
                }
            });

            if let Some(progress) = entry.progress {
                ui.add_space(4.0);
                let progress_color = if progress < 1.0 { PROCESSING } else { SUCCESS };
                let progress_text = format!("{:.0}%", progress * 100.0);
                ui.add(
                    egui::ProgressBar::new(progress)
                        .text(progress_text)
                        .fill(progress_color),
                );
            }
        });
    }
}
