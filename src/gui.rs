#![cfg(feature = "gui")]

mod theme;

use eframe::egui;
use egui::RichText;
use rfd::FileDialog;
use std::path::PathBuf;

use ai_vid_editor::{Config, JoinMode, SilenceMode, WatchFolder};
use theme::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Tab {
    #[default]
    Folders,
    Settings,
    Activity,
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
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
struct FolderState {
    input: PathBuf,
    output: PathBuf,
    preset: String,
    enabled: bool,
    editing: bool,
}

impl From<WatchFolder> for FolderState {
    fn from(folder: WatchFolder) -> Self {
        Self {
            input: folder.input,
            output: folder.output,
            preset: folder.preset,
            enabled: folder.enabled,
            editing: false,
        }
    }
}

impl From<FolderState> for WatchFolder {
    fn from(state: FolderState) -> Self {
        Self {
            input: state.input,
            output: state.output,
            preset: state.preset,
            enabled: state.enabled,
        }
    }
}

impl Default for FolderState {
    fn default() -> Self {
        Self {
            input: PathBuf::from("videos"),
            output: PathBuf::from("videos/output"),
            preset: "youtube".to_string(),
            enabled: true,
            editing: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    config: Config,
    folders: Vec<FolderState>,
    status: ProcessingStatus,
    activity_log: Vec<ActivityEntry>,
    config_path: Option<PathBuf>,
    current_tab: Tab,
}

fn join_mode_display(mode: &JoinMode) -> String {
    match mode {
        JoinMode::Off => "Off".to_string(),
        JoinMode::ByDate => "By Date".to_string(),
        JoinMode::ByName => "By Name".to_string(),
        JoinMode::AfterCount => "After N Files".to_string(),
    }
}

#[allow(dead_code)]
fn notify_complete(filename: &str) {
    let _ = notify_rust::Notification::new()
        .summary("Processing Complete")
        .body(&format!("{} has been processed", filename))
        .show();
}

#[allow(dead_code)]
fn notify_error(filename: &str, error: &str) {
    let _ = notify_rust::Notification::new()
        .summary("Processing Error")
        .body(&format!("Failed to process {}: {}", filename, error))
        .show();
}

impl AppState {
    fn new() -> Self {
        let config = Config::default();
        let folders: Vec<FolderState> = if config.paths.watch_folders.is_empty() {
            vec![FolderState::default()]
        } else {
            config
                .paths
                .watch_folders
                .iter()
                .map(|f| f.clone().into())
                .collect()
        };

        let mut state = Self {
            config,
            folders,
            status: ProcessingStatus::Watching,
            activity_log: vec![ActivityEntry::simple("Started watching for videos", true)],
            config_path: None,
            current_tab: Tab::Folders,
        };

        if let Some(path) = Config::default_config_path() {
            if path.exists() {
                state.load_config(&path);
            }
        }

        state
    }

    fn load_config(&mut self, path: &PathBuf) {
        match Config::from_file(path) {
            Ok(config) => {
                self.config = config.clone();
                self.folders = if self.config.paths.watch_folders.is_empty() {
                    vec![FolderState::default()]
                } else {
                    self.config
                        .paths
                        .watch_folders
                        .iter()
                        .map(|f| f.clone().into())
                        .collect()
                };
                self.config_path = Some(path.clone());
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
        self.config.paths.watch_folders = self.folders.iter().map(|f| f.clone().into()).collect();

        match self.config.to_file(path) {
            Ok(()) => {
                self.config_path = Some(path.clone());
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

    fn add_folder(&mut self) {
        self.folders.push(FolderState::default());
        self.activity_log
            .push(ActivityEntry::simple("Added new watch folder", true));
    }

    fn remove_folder(&mut self, index: usize) {
        if self.folders.len() > 1 {
            self.folders.remove(index);
            self.activity_log
                .push(ActivityEntry::simple("Removed watch folder", true));
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
                ui.add_space(8.0);

                egui::ScrollArea::vertical().show(ui, |ui| match self.state.current_tab {
                    Tab::Folders => {
                        self.draw_folders_panel(ui);
                        ui.add_space(16.0);
                        self.draw_settings_panel(ui);
                        ui.add_space(16.0);
                        self.draw_activity_log(ui);
                    }
                    Tab::Settings => {
                        self.draw_settings_panel(ui);
                    }
                    Tab::Activity => {
                        self.draw_activity_log(ui);
                    }
                });
            });
        });
    }
}

impl App {
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        accent_bar().show(ui, |_ui| {});
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("AI Video Processor")
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
                ui.add_space(8.0);
                if ui.add(button_secondary("Load")).clicked() {
                    if let Some(path) = FileDialog::new().add_filter("TOML", &["toml"]).pick_file()
                    {
                        self.state.load_config(&path);
                    }
                }
            });
        });

        ui.add_space(12.0);

        egui::Frame::NONE
            .fill(PANEL_BG_LIGHT)
            .corner_radius(CORNER_RADIUS_SMALL)
            .inner_margin(egui::vec2(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let tabs = [
                        (Tab::Folders, "Folders"),
                        (Tab::Settings, "Settings"),
                        (Tab::Activity, "Activity"),
                    ];
                    for (tab, name) in tabs {
                        if ui
                            .add(button_tab(self.state.current_tab == tab, name))
                            .clicked()
                        {
                            self.state.current_tab = tab;
                        }
                    }
                });
            });
    }

    fn draw_folders_panel(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Watch Folders")
                        .size(18.0)
                        .color(ACCENT_PRIMARY)
                        .strong(),
                );

                let (status_text, status_color, bg_color, icon) = match &self.state.status {
                    ProcessingStatus::Idle => ("Paused", TEXT_SECONDARY, PANEL_BG_LIGHT, "○"),
                    ProcessingStatus::Watching => ("Watching", SUCCESS, SUCCESS_BG, "●"),
                    ProcessingStatus::Processing(_) => ("Processing", WARNING, PANEL_BG_LIGHT, "◐"),
                    ProcessingStatus::Error(_) => ("Error", ERROR, ERROR_BG, "✗"),
                };

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    status_badge_with_bg(ui, status_text, icon, status_color, bg_color);
                    ui.add_space(8.0);
                    if ui.add(button_secondary("+ Add Folder")).clicked() {
                        self.state.add_folder();
                    }
                });
            });

            ui.add_space(16.0);

            let folder_count = self.state.folders.len();
            let mut to_remove: Option<usize> = None;
            let mut activity_entries: Vec<ActivityEntry> = Vec::new();
            let mut toggle_idx: Option<usize> = None;
            let mut edit_toggle_idx: Option<usize> = None;
            let mut preset_changes: Vec<(usize, String)> = Vec::new();

            for (idx, folder) in self.state.folders.iter_mut().enumerate() {
                folder_card(folder.enabled).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .add(button_toggle(
                                folder.enabled,
                                if folder.enabled { "ON" } else { "OFF" },
                            ))
                            .clicked()
                        {
                            folder.enabled = !folder.enabled;
                            toggle_idx = Some(idx);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if folder_count > 1 {
                                if ui.add(button_small("Remove")).clicked() {
                                    to_remove = Some(idx);
                                }
                            }
                            ui.add_space(4.0);
                            if ui.add(button_small("Edit")).clicked() {
                                edit_toggle_idx = Some(idx);
                            }
                        });
                    });

                    ui.add_space(10.0);

                    let text_color = if folder.enabled {
                        TEXT_PRIMARY
                    } else {
                        TEXT_MUTED
                    };
                    let muted_color = if folder.enabled {
                        TEXT_SECONDARY
                    } else {
                        TEXT_MUTED
                    };

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Input: ").color(muted_color).size(12.0));
                        ui.label(
                            RichText::new(folder.input.to_string_lossy().to_string())
                                .color(text_color)
                                .size(12.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Output:").color(muted_color).size(12.0));
                        ui.label(
                            RichText::new(folder.output.to_string_lossy().to_string())
                                .color(text_color)
                                .size(12.0),
                        );
                    });

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Preset:").color(muted_color).size(12.0));
                        ui.label(RichText::new(&folder.preset).color(text_color).size(12.0));
                    });

                    if folder.editing {
                        ui.add_space(12.0);
                        ui.label(label_muted("--- Edit ---"));
                        ui.add_space(8.0);

                        ui.label(label_secondary("Input Folder"));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let mut input_str = folder.input.to_string_lossy().to_string();
                            ui.add_sized(
                                egui::vec2(ui.available_width() - 80.0, 28.0),
                                text_edit_style(&mut input_str),
                            );
                            if ui.add(button_small("Browse")).clicked() {
                                if let Some(path) = FileDialog::new().pick_folder() {
                                    folder.input = path;
                                }
                            }
                        });

                        ui.add_space(10.0);

                        ui.label(label_secondary("Output Folder"));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let mut output_str = folder.output.to_string_lossy().to_string();
                            ui.add_sized(
                                egui::vec2(ui.available_width() - 80.0, 28.0),
                                text_edit_style(&mut output_str),
                            );
                            if ui.add(button_small("Browse")).clicked() {
                                if let Some(path) = FileDialog::new().pick_folder() {
                                    folder.output = path;
                                }
                            }
                        });

                        ui.add_space(10.0);

                        ui.label(label_secondary("Preset"));
                        ui.add_space(4.0);
                        egui::ComboBox::from_id_salt(format!("preset_{}", idx))
                            .selected_text(
                                RichText::new(&folder.preset).color(TEXT_PRIMARY).size(13.0),
                            )
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                let presets = Config::available_presets();
                                for preset in presets {
                                    if ui
                                        .selectable_value(
                                            &mut folder.preset,
                                            preset.clone(),
                                            RichText::new(&preset).color(TEXT_PRIMARY).size(13.0),
                                        )
                                        .changed()
                                    {
                                        preset_changes.push((idx, preset.clone()));
                                    }
                                }
                            });
                    }
                });
                ui.add_space(10.0);
            }

            if let Some(idx) = to_remove {
                self.state.remove_folder(idx);
            }
            if let Some(idx) = toggle_idx {
                let folder = &self.state.folders[idx];
                activity_entries.push(ActivityEntry::simple(
                    format!(
                        "Folder {} {}",
                        if folder.enabled {
                            "enabled"
                        } else {
                            "disabled"
                        },
                        folder.input.display()
                    ),
                    true,
                ));
            }
            if let Some(idx) = edit_toggle_idx {
                self.state.folders[idx].editing = !self.state.folders[idx].editing;
            }
            for (idx, preset) in preset_changes {
                activity_entries.push(ActivityEntry::simple(
                    format!("Changed preset to {} for folder {}", preset, idx),
                    true,
                ));
            }
            for entry in activity_entries {
                self.state.activity_log.push(entry);
            }
        });
    }

    fn draw_settings_panel(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.label(
                RichText::new("Settings")
                    .size(18.0)
                    .color(ACCENT_PRIMARY)
                    .strong(),
            );

            ui.add_space(16.0);
            ui.label(label_muted("--- Processing ---"));
            ui.add_space(12.0);

            let mut enhance = self.state.config.audio.enhance;
            if ui
                .checkbox(
                    &mut enhance,
                    RichText::new("Enhance Audio")
                        .color(TEXT_PRIMARY)
                        .size(14.0),
                )
                .changed()
            {
                self.state.config.audio.enhance = enhance;
            }

            let mut remove_silence = self.state.config.silence.mode == SilenceMode::Cut;
            if ui
                .checkbox(
                    &mut remove_silence,
                    RichText::new("Remove Silence")
                        .color(TEXT_PRIMARY)
                        .size(14.0),
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
                    RichText::new("Stabilize Video")
                        .color(TEXT_PRIMARY)
                        .size(14.0),
                )
                .changed()
            {
                self.state.config.video.stabilize = stabilize;
            }

            let mut color_correct = self.state.config.video.color_correct;
            if ui
                .checkbox(
                    &mut color_correct,
                    RichText::new("Color Correct")
                        .color(TEXT_PRIMARY)
                        .size(14.0),
                )
                .changed()
            {
                self.state.config.video.color_correct = color_correct;
            }

            let mut reframe = self.state.config.video.reframe;
            if ui
                .checkbox(
                    &mut reframe,
                    RichText::new("Auto-Reframe (9:16)")
                        .color(TEXT_PRIMARY)
                        .size(14.0),
                )
                .changed()
            {
                self.state.config.video.reframe = reframe;
            }

            let mut blur = self.state.config.video.blur_background;
            if ui
                .checkbox(
                    &mut blur,
                    RichText::new("Blur Background")
                        .color(TEXT_PRIMARY)
                        .size(14.0),
                )
                .changed()
            {
                self.state.config.video.blur_background = blur;
            }

            ui.add_space(20.0);
            ui.label(label_muted("--- Advanced ---"));
            ui.add_space(12.0);

            ui.label(label_secondary("Silence Threshold (dB)"));
            ui.add_space(4.0);
            ui.add(
                egui::Slider::new(&mut self.state.config.silence.threshold_db, -60.0..=-10.0)
                    .step_by(1.0),
            );

            ui.add_space(10.0);

            ui.label(label_secondary("Target LUFS"));
            ui.add_space(4.0);
            ui.add(
                egui::Slider::new(&mut self.state.config.audio.target_lufs, -24.0..=-6.0)
                    .step_by(1.0),
            );

            ui.add_space(10.0);

            ui.label(label_secondary("Join Mode"));
            ui.add_space(4.0);

            egui::ComboBox::from_id_salt("join")
                .selected_text(
                    RichText::new(join_mode_display(&self.state.config.processing.join_mode))
                        .color(TEXT_PRIMARY)
                        .size(13.0),
                )
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.state.config.processing.join_mode,
                        JoinMode::Off,
                        RichText::new("Off").color(TEXT_PRIMARY).size(13.0),
                    );
                    ui.selectable_value(
                        &mut self.state.config.processing.join_mode,
                        JoinMode::ByDate,
                        RichText::new("By Date").color(TEXT_PRIMARY).size(13.0),
                    );
                    ui.selectable_value(
                        &mut self.state.config.processing.join_mode,
                        JoinMode::ByName,
                        RichText::new("By Name").color(TEXT_PRIMARY).size(13.0),
                    );
                    ui.selectable_value(
                        &mut self.state.config.processing.join_mode,
                        JoinMode::AfterCount,
                        RichText::new("After N Files")
                            .color(TEXT_PRIMARY)
                            .size(13.0),
                    );
                });

            if self.state.config.processing.join_mode == JoinMode::AfterCount {
                ui.add_space(10.0);
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
                    RichText::new("Activity Log")
                        .size(18.0)
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
                for entry in self.state.activity_log.iter().rev().take(20) {
                    match entry.status {
                        EntryStatus::Success => {
                            if entry.filename.is_empty() {
                                log_entry_simple(ui, &entry.timestamp, &entry.message, true);
                            } else {
                                log_entry_success(
                                    ui,
                                    &entry.timestamp,
                                    &entry.filename,
                                    &format_file_size(entry.file_size),
                                    &entry
                                        .duration
                                        .map(|d| format_duration(d))
                                        .unwrap_or_default(),
                                );
                            }
                        }
                        EntryStatus::Processing => {
                            log_entry_processing(
                                ui,
                                &entry.timestamp,
                                &entry.filename,
                                entry.progress.unwrap_or(0.0),
                            );
                        }
                        EntryStatus::Error => {
                            log_entry_error(ui, &entry.timestamp, &entry.filename, &entry.message);
                        }
                    }
                    ui.add_space(6.0);
                }
            }
        });
    }
}
