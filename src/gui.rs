#![cfg(feature = "gui")]

mod theme;

use eframe::egui;
use egui::RichText;
use rfd::FileDialog;
use std::path::PathBuf;

use ai_vid_editor::{Config, FolderSettings, JoinMode, WatchFolder};
use theme::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum Tab {
    #[default]
    All,
    Folders,
    Settings,
    Activity,
}

#[allow(dead_code)]
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
    settings: FolderSettings,
}

impl From<WatchFolder> for FolderState {
    fn from(folder: WatchFolder) -> Self {
        Self {
            input: folder.input,
            output: folder.output,
            preset: folder.preset,
            enabled: folder.enabled,
            settings: folder.settings,
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
            settings: state.settings,
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
            settings: FolderSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ModalState {
    show: bool,
    editing_idx: Option<usize>,
    input: PathBuf,
    output: PathBuf,
    preset: String,
    enabled: bool,
}

impl ModalState {
    fn reset_for_add(&mut self) {
        self.show = true;
        self.editing_idx = None;
        self.input = PathBuf::from("videos");
        self.output = PathBuf::from("videos/output");
        self.preset = "youtube".to_string();
        self.enabled = true;
    }

    fn set_for_edit(&mut self, idx: usize, folder: &FolderState) {
        self.show = true;
        self.editing_idx = Some(idx);
        self.input = folder.input.clone();
        self.output = folder.output.clone();
        self.preset = folder.preset.clone();
        self.enabled = folder.enabled;
    }

    fn close(&mut self) {
        self.show = false;
        self.editing_idx = None;
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
    modal: ModalState,
    selected_folder_idx: usize,
}

#[allow(dead_code)]
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
            current_tab: Tab::All,
            modal: ModalState::default(),
            selected_folder_idx: 0,
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

    #[allow(dead_code)]
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

    fn auto_save_config(&mut self) {
        self.config.paths.watch_folders = self.folders.iter().map(|f| f.clone().into()).collect();

        let path = if let Some(ref p) = self.config_path {
            Some(p.clone())
        } else {
            Config::default_config_path()
        };

        if let Some(path) = path {
            if let Err(e) = self.config.to_file(&path) {
                self.activity_log.push(ActivityEntry::simple(
                    format!("Failed to auto-save config: {}", e),
                    false,
                ));
            }
        }
    }

    fn add_folder_from_modal(&mut self) {
        let folder = FolderState {
            input: self.modal.input.clone(),
            output: self.modal.output.clone(),
            preset: self.modal.preset.clone(),
            enabled: self.modal.enabled,
            settings: FolderSettings::default(),
        };
        self.folders.push(folder);
        self.activity_log
            .push(ActivityEntry::simple("Added new watch folder", true));
        self.auto_save_config();
    }

    fn update_folder_from_modal(&mut self, idx: usize) {
        if let Some(folder) = self.folders.get_mut(idx) {
            folder.input = self.modal.input.clone();
            folder.output = self.modal.output.clone();
            folder.preset = self.modal.preset.clone();
            folder.enabled = self.modal.enabled;
            self.activity_log
                .push(ActivityEntry::simple("Updated watch folder", true));
            self.auto_save_config();
        }
    }

    fn remove_folder(&mut self, index: usize) {
        if self.folders.len() > 1 {
            self.folders.remove(index);
            self.activity_log
                .push(ActivityEntry::simple("Removed watch folder", true));
            self.auto_save_config();
        }
    }

    fn toggle_folder(&mut self, index: usize) {
        if let Some(folder) = self.folders.get_mut(index) {
            folder.enabled = !folder.enabled;
            let status = if folder.enabled {
                "enabled"
            } else {
                "disabled"
            };
            self.activity_log.push(ActivityEntry::simple(
                format!("Folder {} ({})", status, folder.input.display()),
                true,
            ));
            self.auto_save_config();
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
                    Tab::All => {
                        self.draw_folders_panel(ui);
                        ui.add_space(16.0);
                        self.draw_settings_panel(ui);
                        ui.add_space(16.0);
                        self.draw_activity_log(ui);
                    }
                    Tab::Folders => {
                        self.draw_folders_panel(ui);
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

        if self.state.modal.show {
            self.draw_modal(ctx);
        }
    }
}

impl App {
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        accent_bar().show(ui, |_ui| {});
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("AI Video Processor")
                    .size(18.0)
                    .color(ACCENT_PRIMARY)
                    .strong(),
            );
        });

        ui.add_space(10.0);

        egui::Frame::NONE
            .fill(PANEL_BG_LIGHT)
            .corner_radius(CORNER_RADIUS_SMALL)
            .inner_margin(egui::vec2(6.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let tabs = [
                        (Tab::All, "All"),
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
                    RichText::new("Folders")
                        .size(16.0)
                        .color(ACCENT_PRIMARY)
                        .strong(),
                );

                let (status_text, status_color, bg_color) = match &self.state.status {
                    ProcessingStatus::Idle => ("Paused", TEXT_SECONDARY, PANEL_BG_LIGHT),
                    ProcessingStatus::Watching => ("Watching", SUCCESS, SUCCESS_BG),
                    ProcessingStatus::Processing(_) => ("Processing", WARNING, PANEL_BG_LIGHT),
                    ProcessingStatus::Error(_) => ("Error", ERROR, ERROR_BG),
                };

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    status_badge_with_bg(ui, status_text, status_color, bg_color);
                    ui.add_space(8.0);
                    if ui.add(button_add("+ Add")).clicked() {
                        self.state.modal.reset_for_add();
                    }
                });
            });

            ui.add_space(16.0);

            let mut toggle_idx: Option<usize> = None;
            let mut edit_idx: Option<usize> = None;

            for (idx, folder) in self.state.folders.iter().enumerate() {
                let enabled = folder.enabled;
                let input = folder.input.clone();
                let output = folder.output.clone();
                let preset = folder.preset.clone();
                let muted_color = if enabled { TEXT_SECONDARY } else { TEXT_MUTED };
                let text_color = if enabled { TEXT_PRIMARY } else { TEXT_MUTED };

                let response = folder_card_compact(enabled).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .add(button_toggle(enabled, if enabled { "ON" } else { "OFF" }))
                            .clicked()
                        {
                            toggle_idx = Some(idx);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            preset_badge(&preset, ui);
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(RichText::new("Input:  ").color(muted_color).size(11.0));
                        ui.label(
                            RichText::new(truncate_path(&input.to_string_lossy(), 35))
                                .color(text_color)
                                .size(11.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(RichText::new("Output: ").color(muted_color).size(11.0));
                        ui.label(
                            RichText::new(truncate_path(&output.to_string_lossy(), 35))
                                .color(text_color)
                                .size(11.0),
                        );
                    });
                });

                if response.response.clicked() {
                    edit_idx = Some(idx);
                }

                ui.add_space(6.0);
            }

            if let Some(idx) = toggle_idx {
                self.state.toggle_folder(idx);
            }
            if let Some(idx) = edit_idx {
                let folder = &self.state.folders[idx];
                self.state.modal.set_for_edit(idx, folder);
            }
        });
    }

    fn draw_modal(&mut self, ctx: &egui::Context) {
        let title = if self.state.modal.editing_idx.is_some() {
            "Edit Folder"
        } else {
            "Add Folder"
        };

        let mut should_close = false;
        let mut should_save = false;
        let mut should_delete = false;

        let screen_rect = ctx.screen_rect();

        egui::Area::new(egui::Id::new("modal_overlay"))
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                modal_overlay().show(ui, |ui| {
                    ui.allocate_space(screen_rect.size());
                });
            });

        egui::Area::new(egui::Id::new("modal_dialog"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                modal_dialog().show(ui, |ui| {
                    ui.set_min_width(380.0);

                    ui.label(
                        RichText::new(title)
                            .size(16.0)
                            .color(ACCENT_PRIMARY)
                            .strong(),
                    );
                    ui.add_space(16.0);

                    ui.label(label_secondary("Input Folder"));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let mut input_str = self.state.modal.input.to_string_lossy().to_string();
                        ui.add_sized(
                            egui::vec2(ui.available_width() - 80.0, 28.0),
                            text_edit_style(&mut input_str),
                        );
                        self.state.modal.input = PathBuf::from(&input_str);
                        if ui.add(button_small("...")).clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                self.state.modal.input = path;
                            }
                        }
                    });

                    ui.add_space(10.0);

                    ui.label(label_secondary("Output Folder"));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let mut output_str = self.state.modal.output.to_string_lossy().to_string();
                        ui.add_sized(
                            egui::vec2(ui.available_width() - 80.0, 28.0),
                            text_edit_style(&mut output_str),
                        );
                        self.state.modal.output = PathBuf::from(&output_str);
                        if ui.add(button_small("...")).clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                self.state.modal.output = path;
                            }
                        }
                    });

                    ui.add_space(10.0);

                    ui.label(label_secondary("Preset"));
                    ui.add_space(6.0);
                    egui::ComboBox::from_id_salt("modal_preset")
                        .selected_text(
                            RichText::new(&self.state.modal.preset)
                                .color(TEXT_PRIMARY)
                                .size(12.0),
                        )
                        .width(ui.available_width())
                        .show_ui(ui, |ui| {
                            let presets = Config::available_presets();
                            for preset in presets {
                                ui.selectable_value(
                                    &mut self.state.modal.preset,
                                    preset.clone(),
                                    RichText::new(&preset).color(TEXT_PRIMARY).size(12.0),
                                );
                            }
                        });

                    ui.add_space(10.0);

                    ui.checkbox(
                        &mut self.state.modal.enabled,
                        RichText::new("Enabled").color(TEXT_PRIMARY).size(13.0),
                    );

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if let Some(_idx) = self.state.modal.editing_idx {
                            if self.state.folders.len() > 1 {
                                if ui.add(button_small("Delete")).clicked() {
                                    should_delete = true;
                                    should_close = true;
                                }
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_text = if self.state.modal.editing_idx.is_some() {
                                "Save"
                            } else {
                                "Add"
                            };
                            if ui.add(button_secondary(btn_text)).clicked() {
                                should_save = true;
                                should_close = true;
                            }
                            ui.add_space(8.0);
                            if ui.add(button_small("Cancel")).clicked() {
                                should_close = true;
                            }
                        });
                    });
                });
            });

        if should_close {
            if should_delete {
                if let Some(idx) = self.state.modal.editing_idx {
                    self.state.remove_folder(idx);
                }
            } else if should_save {
                if let Some(idx) = self.state.modal.editing_idx {
                    self.state.update_folder_from_modal(idx);
                } else {
                    self.state.add_folder_from_modal();
                }
            }
            self.state.modal.close();
        }
    }

    fn draw_settings_panel(&mut self, ui: &mut egui::Ui) {
        let folder_names: Vec<String> = self
            .state
            .folders
            .iter()
            .enumerate()
            .map(|(i, f)| {
                format!(
                    "{}. {}",
                    i + 1,
                    f.input.file_name().unwrap_or_default().to_string_lossy()
                )
            })
            .collect();

        let preset_name = self
            .state
            .folders
            .get(self.state.selected_folder_idx)
            .map(|f| f.preset.clone())
            .unwrap_or_default();

        let (
            enhance_val,
            remove_silence_val,
            stabilize_val,
            color_correct_val,
            reframe_val,
            blur_val,
            threshold_val,
            lufs_val,
        ) = {
            if let Some(folder) = self.state.folders.get(self.state.selected_folder_idx) {
                (
                    folder.settings.enhance_audio.unwrap_or(true),
                    folder.settings.remove_silence.unwrap_or(true),
                    folder.settings.stabilize.unwrap_or(false),
                    folder.settings.color_correct.unwrap_or(false),
                    folder.settings.reframe.unwrap_or(false),
                    folder.settings.blur_background.unwrap_or(false),
                    folder.settings.silence_threshold_db.unwrap_or(-30.0),
                    folder.settings.target_lufs.unwrap_or(-14.0),
                )
            } else {
                (true, true, false, false, false, false, -30.0, -14.0)
            }
        };

        settings_panel_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.horizontal_wrapped(|ui| {
                    for (idx, name) in folder_names.iter().enumerate() {
                        if ui
                            .add(button_pill(idx == self.state.selected_folder_idx, name))
                            .clicked()
                        {
                            self.state.selected_folder_idx = idx;
                        }
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    preset_badge(&preset_name, ui);
                });
            });

            ui.add_space(16.0);

            let mut needs_save = false;
            let folder_idx = self.state.selected_folder_idx;

            settings_section_frame(false).show(ui, |ui| {
                ui.label(
                    RichText::new("Processing")
                        .size(13.0)
                        .color(ACCENT_PRIMARY)
                        .strong(),
                );
                ui.add_space(10.0);

                ui.horizontal_wrapped(|ui| {
                    let mut enhance = enhance_val;
                    if Self::draw_settings_toggle_compact(ui, "Enhance Audio", &mut enhance) {
                        if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                            folder.settings.enhance_audio = Some(enhance);
                            needs_save = true;
                        }
                    }
                    ui.add_space(6.0);

                    let mut remove_silence = remove_silence_val;
                    if Self::draw_settings_toggle_compact(ui, "Remove Silence", &mut remove_silence) {
                        if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                            folder.settings.remove_silence = Some(remove_silence);
                            needs_save = true;
                        }
                    }
                    ui.add_space(6.0);

                    let mut stabilize = stabilize_val;
                    if Self::draw_settings_toggle_compact(ui, "Stabilize", &mut stabilize) {
                        if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                            folder.settings.stabilize = Some(stabilize);
                            needs_save = true;
                        }
                    }
                    ui.add_space(6.0);

                    let mut color_correct = color_correct_val;
                    if Self::draw_settings_toggle_compact(ui, "Color", &mut color_correct) {
                        if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                            folder.settings.color_correct = Some(color_correct);
                            needs_save = true;
                        }
                    }
                    ui.add_space(6.0);

                    let mut reframe = reframe_val;
                    if Self::draw_settings_toggle_compact(ui, "Reframe", &mut reframe) {
                        if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                            folder.settings.reframe = Some(reframe);
                            needs_save = true;
                        }
                    }
                    ui.add_space(6.0);

                    let mut blur = blur_val;
                    if Self::draw_settings_toggle_compact(ui, "Blur", &mut blur) {
                        if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                            folder.settings.blur_background = Some(blur);
                            needs_save = true;
                        }
                    }
                });
            });
                    ui,
                    "Blur Background",
                    "Fill side space when reframing to portrait.",
                    &mut blur,
                ) {
                    if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                        folder.settings.blur_background = Some(blur);
                        needs_save = true;
                    }
                }
            });

            ui.add_space(12.0);

            settings_section_frame(false).show(ui, |ui| {
                ui.label(
                    RichText::new("Advanced")
                        .size(13.0)
                        .color(ACCENT_PRIMARY)
                        .strong(),
                );
                ui.add_space(10.0);

                if ui.available_width() > 620.0 {
                    ui.columns(2, |cols| {
                        let mut threshold = threshold_val;
                        let threshold_label = format!("{threshold:.0} dB");
                        if Self::draw_advanced_slider(
                            &mut cols[0],
                            "Silence Threshold (dB)",
                            "Lower values keep more ambient speech.",
                            &mut threshold,
                            -60.0..=-10.0,
                            threshold_label,
                        ) {
                            if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                                folder.settings.silence_threshold_db = Some(threshold);
                                needs_save = true;
                            }
                        }

                        let mut lufs = lufs_val;
                        let lufs_label = format!("{lufs:.0} LUFS");
                        if Self::draw_advanced_slider(
                            &mut cols[1],
                            "Target LUFS",
                            "Final loudness target for exports.",
                            &mut lufs,
                            -24.0..=-6.0,
                            lufs_label,
                        ) {
                            if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                                folder.settings.target_lufs = Some(lufs);
                                needs_save = true;
                            }
                        }
                    });
                } else {
                    let mut threshold = threshold_val;
                    let threshold_label = format!("{threshold:.0} dB");
                    if Self::draw_advanced_slider(
                        ui,
                        "Silence Threshold (dB)",
                        "Lower values keep more ambient speech.",
                        &mut threshold,
                        -60.0..=-10.0,
                        threshold_label,
                    ) {
                        if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                            folder.settings.silence_threshold_db = Some(threshold);
                            needs_save = true;
                        }
                    }

                    ui.add_space(8.0);

                    let mut lufs = lufs_val;
                    let lufs_label = format!("{lufs:.0} LUFS");
                    if Self::draw_advanced_slider(
                        ui,
                        "Target LUFS",
                        "Final loudness target for exports.",
                        &mut lufs,
                        -24.0..=-6.0,
                        lufs_label,
                    ) {
                        if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                            folder.settings.target_lufs = Some(lufs);
                            needs_save = true;
                        }
                    }
                }
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.label(label_muted(
                    "Restore this folder's settings to default values.",
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(button_small("Reset to Defaults")).clicked() {
                        if let Some(folder) = self.state.folders.get_mut(folder_idx) {
                            folder.settings = FolderSettings::default();
                            needs_save = true;
                            self.state.activity_log.push(ActivityEntry::simple(
                                format!("Reset folder {} to defaults", folder_idx + 1),
                                true,
                            ));
                        }
                    }
                });
            });

            if needs_save {
                self.state.auto_save_config();
            }
        });
    }

    #[allow(dead_code)]
    fn draw_settings_metric(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
        let bg = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 24);
        egui::Frame::NONE
            .fill(bg)
            .corner_radius(6.0)
            .inner_margin(egui::vec2(10.0, 9.0))
            .stroke(egui::Stroke::new(1.0, color))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(label).size(10.0).color(TEXT_MUTED));
                    ui.label(RichText::new(value).size(13.0).color(TEXT_PRIMARY).strong());
                });
            });
    }

    fn draw_settings_toggle(
        ui: &mut egui::Ui,
        label: &str,
        help_text: &str,
        value: &mut bool,
    ) -> bool {
        let mut changed = false;
        settings_toggle_frame(*value).show(ui, |ui| {
            ui.horizontal(|ui| {
                let dot_color = if *value { ACCENT_PRIMARY } else { TEXT_MUTED };
                let (dot_rect, _) =
                    ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot_rect.center(), 3.5, dot_color);
                ui.add_space(6.0);
                ui.label(RichText::new(label).color(TEXT_PRIMARY).size(12.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let switch_text = if *value { "ON" } else { "OFF" };
                    if ui.add(button_toggle(*value, switch_text)).clicked() {
                        *value = !*value;
                        changed = true;
                    }
                });
            });
            ui.add_space(3.0);
            ui.label(label_muted(help_text));
        });
        changed
    }

    fn draw_settings_toggle_compact(ui: &mut egui::Ui, label: &str, value: &mut bool) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            let dot_color = if *value { ACCENT_PRIMARY } else { TEXT_MUTED };
            let (dot_rect, _) = ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
            ui.painter().circle_filled(dot_rect.center(), 3.0, dot_color);
            ui.add_space(4.0);
            ui.label(RichText::new(label).color(TEXT_PRIMARY).size(11.0));
            ui.add_space(4.0);
            let switch_text = if *value { "ON" } else { "OFF" };
            if ui.add(button_toggle(*value, switch_text)).clicked() {
                *value = !*value;
                changed = true;
            }
        });
        changed
    }

    fn draw_advanced_slider(
        ui: &mut egui::Ui,
        title: &str,
        help_text: &str,
        value: &mut f32,
        range: std::ops::RangeInclusive<f32>,
        value_label: String,
    ) -> bool {
        let mut changed = false;
        settings_toggle_frame(true).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(label_secondary(title));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    settings_value_badge(ui, &value_label);
                });
            });
            ui.add_space(8.0);
            if slider_glow(value, range, ui).changed() {
                changed = true;
            }
            ui.add_space(4.0);
            ui.label(label_muted(help_text));
        });
        changed
    }

    #[allow(dead_code)]
    fn get_preset_default(_preset: &str, _setting: &str) -> bool {
        match _setting {
            "enhance_audio" => true,
            _ => false,
        }
    }

    fn draw_activity_log(&mut self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Activity")
                        .size(16.0)
                        .color(ACCENT_PRIMARY)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(button_small("Clear")).clicked() {
                        self.state.activity_log.clear();
                    }
                });
            });

            ui.add_space(10.0);

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
