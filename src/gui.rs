#![cfg(feature = "gui")]

use dioxus::prelude::*;
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::config::{Config, Preset, SilenceMode};

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

#[derive(Debug, Clone, Default)]
pub struct AppState {
    config: Config,
    selected_preset: String,
    watch_folder: PathBuf,
    output_folder: PathBuf,
    status: ProcessingStatus,
    activity_log: Vec<ActivityEntry>,
}

impl AppState {
    fn from_config(config: Config) -> Self {
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
        }
    }
}

pub fn App() -> Element {
    let mut state = use_signal(|| AppState::from_config(Config::default()));

    rsx! {
        style { {include_str!("gui_style.css")} }

        div { class: "container",
            header { class: "header",
                h1 { "AI Video Processor" }
            }

            WatchSection { state: state.clone() }

            SettingsSection { state: state.clone() }

            ActivitySection { state: state.clone() }
        }
    }
}

#[component]
fn WatchSection(state: Signal<AppState>) -> Element {
    let is_watching = state.read().status == ProcessingStatus::Watching;

    rsx! {
        section { class: "section watch-section",
            h2 { "Automation Mode" }

            div { class: "folder-row",
                label { "Watch Folder:" }
                FolderPicker {
                    path: state.read().watch_folder.clone(),
                    on_select: move |p| {
                        state.write().watch_folder = p;
                    }
                }
            }

            div { class: "folder-row",
                label { "Output Folder:" }
                FolderPicker {
                    path: state.read().output_folder.clone(),
                    on_select: move |p| {
                        state.write().output_folder = p;
                    }
                }
            }

            div { class: "button-row",
                button {
                    class: if is_watching { "btn btn-stop" } else { "btn btn-start" },
                    onclick: move |_| {
                        if is_watching {
                            state.write().status = ProcessingStatus::Idle;
                            state.write().activity_log.push(ActivityEntry::new("Stopped watching", true));
                        } else {
                            state.write().status = ProcessingStatus::Watching;
                            state.write().activity_log.push(ActivityEntry::new("Started watching for new videos", true));
                        }
                    },
                    if is_watching { "Stop Watching" } else { "Start Watching" }
                }
            }

            div { class: "status-row",
                match &state.read().status {
                    ProcessingStatus::Idle => rsx! { span { class: "status-idle", "Ready" } },
                    ProcessingStatus::Watching => rsx! {
                        span { class: "status-watching", "Watching for new videos..." }
                    },
                    ProcessingStatus::Processing(msg) => rsx! {
                        span { class: "status-processing", "Processing: {msg}" }
                    },
                    ProcessingStatus::Error(msg) => rsx! {
                        span { class: "status-error", "Error: {msg}" }
                    },
                }
            }
        }
    }
}

#[component]
fn SettingsSection(state: Signal<AppState>) -> Element {
    let presets = vec!["youtube", "shorts", "podcast", "minimal"];
    let current_preset = state.read().selected_preset.clone();

    rsx! {
        section { class: "section settings-section",
            h2 { "Settings" }

            div { class: "preset-row",
                label { "Preset:" }
                select {
                    value: current_preset,
                    onchange: move |evt| {
                        let preset_name = evt.value();
                        state.write().selected_preset = preset_name.clone();
                        if let Some(preset) = Preset::from_str(&preset_name) {
                            state.write().config = preset.to_config();
                        }
                        state.write().activity_log.push(ActivityEntry::new(format!("Switched to {} preset", preset_name), true));
                    },
                    for preset in presets {
                        option {
                            value: preset,
                            selected: preset == current_preset,
                            { preset.to_uppercase() }
                        }
                    }
                }
            }

            div { class: "settings-grid",
                div { class: "setting-item",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: state.read().config.silence.mode == SilenceMode::Cut,
                            onchange: move |_| {
                                let current = state.read().config.silence.mode;
                                state.write().config.silence.mode = match current {
                                    SilenceMode::Cut => SilenceMode::Speedup,
                                    SilenceMode::Speedup => SilenceMode::Cut,
                                };
                            }
                        }
                        " Remove Silence"
                    }
                }

                div { class: "setting-item",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: state.read().config.audio.enhance,
                            onchange: move |_| {
                                state.write().config.audio.enhance = !state.read().config.audio.enhance;
                            }
                        }
                        " Enhance Audio"
                    }
                }

                div { class: "setting-item",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: state.read().config.video.stabilize,
                            onchange: move |_| {
                                state.write().config.video.stabilize = !state.read().config.video.stabilize;
                            }
                        }
                        " Stabilize Video"
                    }
                }

                div { class: "setting-item",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: state.read().config.video.color_correct,
                            onchange: move |_| {
                                state.write().config.video.color_correct = !state.read().config.video.color_correct;
                            }
                        }
                        " Color Correct"
                    }
                }

                div { class: "setting-item",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: state.read().config.video.reframe,
                            onchange: move |_| {
                                state.write().config.video.reframe = !state.read().config.video.reframe;
                            }
                        }
                        " Auto-Reframe"
                    }
                }

                div { class: "setting-item",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: state.read().config.video.blur_background,
                            onchange: move |_| {
                                state.write().config.video.blur_background = !state.read().config.video.blur_background;
                            }
                        }
                        " Blur Background"
                    }
                }
            }

            div { class: "threshold-row",
                label { "Silence Threshold (dB):" }
                input {
                    r#type: "number",
                    value: state.read().config.silence.threshold_db,
                    onchange: move |evt| {
                        if let Ok(val) = evt.value().parse::<f32>() {
                            state.write().config.silence.threshold_db = val;
                        }
                    }
                }
            }

            div { class: "threshold-row",
                label { "Target LUFS:" }
                input {
                    r#type: "number",
                    value: state.read().config.audio.target_lufs,
                    onchange: move |evt| {
                        if let Ok(val) = evt.value().parse::<f32>() {
                            state.write().config.audio.target_lufs = val;
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ActivitySection(state: Signal<AppState>) -> Element {
    let log = state.read().activity_log.clone();

    rsx! {
        section { class: "section activity-section",
            h2 { "Activity Log" }

            div { class: "activity-log",
                if log.is_empty() {
                    p { class: "empty-log", "No activity yet. Start watching to see processed files." }
                } else {
                    for entry in log.iter().rev().take(20) {
                        div {
                            class: if entry.success { "log-entry log-success" } else { "log-entry log-error" },
                            span { class: "log-time", "{entry.timestamp}" }
                            span { class: "log-message", "{entry.message}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FolderPicker(path: PathBuf, on_select: EventHandler<PathBuf>) -> Element {
    let display_path = if path.as_os_str().is_empty() {
        "Select folder...".to_string()
    } else {
        path.display().to_string()
    };

    rsx! {
        div { class: "folder-picker",
            input {
                r#type: "text",
                readonly: true,
                value: display_path,
            }
            button {
                class: "btn btn-browse",
                onclick: move |_| {
                    if let Some(selected) = FileDialog::new().pick_folder() {
                        on_select.call(selected);
                    }
                },
                "Browse..."
            }
        }
    }
}
