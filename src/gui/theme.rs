#![cfg(feature = "gui")]

use eframe::egui;

pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(16, 16, 16);
pub const PANEL_BG_LIGHT: egui::Color32 = egui::Color32::from_rgb(24, 24, 24);
pub const PANEL_BG_LIGHTER: egui::Color32 = egui::Color32::from_rgb(34, 34, 34);

pub const ACCENT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(230, 57, 70);
pub const ACCENT_DARK: egui::Color32 = egui::Color32::from_rgb(180, 45, 55);

pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(250, 250, 250);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(170, 170, 170);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(100, 100, 100);

pub const BORDER: egui::Color32 = egui::Color32::from_rgb(36, 36, 36);
pub const BORDER_LIGHT: egui::Color32 = egui::Color32::from_rgb(50, 50, 50);

pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(78, 205, 196);
pub const SUCCESS_BG: egui::Color32 = egui::Color32::from_rgb(20, 55, 50);
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(255, 68, 68);
pub const ERROR_BG: egui::Color32 = egui::Color32::from_rgb(55, 20, 20);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 193, 7);
pub const PROCESSING: egui::Color32 = egui::Color32::from_rgb(100, 149, 237);
pub const PROCESSING_BG: egui::Color32 = egui::Color32::from_rgb(25, 40, 60);

pub const CORNER_RADIUS: f32 = 12.0;
pub const CORNER_RADIUS_SMALL: f32 = 8.0;
#[allow(dead_code)]
pub const CORNER_RADIUS_PILL: f32 = 24.0;

pub fn accent_bar() -> egui::Frame {
    egui::Frame::NONE
        .fill(ACCENT_PRIMARY)
        .inner_margin(egui::vec2(0.0, 3.0))
}

pub fn panel_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_BG)
        .corner_radius(CORNER_RADIUS)
        .inner_margin(20.0)
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn inner_panel() -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_BG_LIGHT)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(14.0)
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn card_frame(bg: egui::Color32) -> egui::Frame {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(14.0)
        .stroke(egui::Stroke::new(1.0, BORDER))
}

#[allow(dead_code)]
pub fn section_header(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(15.0)
        .color(ACCENT_PRIMARY)
        .strong()
}

pub fn label_primary(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_PRIMARY).size(14.0)
}

pub fn label_secondary(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_SECONDARY).size(13.0)
}

pub fn label_muted(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_MUTED).size(12.0)
}

pub fn text_edit_style(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(text)
        .text_color(TEXT_PRIMARY)
        .background_color(PANEL_BG_LIGHTER)
        .desired_width(f32::INFINITY)
        .cursor_at_end(true)
}

pub fn button_secondary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_PRIMARY).size(13.0))
        .fill(PANEL_BG_LIGHTER)
        .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(80.0, 34.0))
}

pub fn button_small(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(12.0))
        .fill(PANEL_BG_LIGHT)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(50.0, 28.0))
}

#[allow(dead_code)]
pub fn button_icon(icon: &str, _tooltip: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(icon).size(14.0))
        .fill(PANEL_BG_LIGHTER)
        .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(32.0, 32.0))
        .sense(egui::Sense::click())
}

#[allow(dead_code)]
pub fn button_primary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text)
            .color(egui::Color32::WHITE)
            .size(15.0)
            .strong(),
    )
    .fill(ACCENT_PRIMARY)
    .stroke(egui::Stroke::new(2.0, ACCENT_DARK))
    .corner_radius(CORNER_RADIUS_PILL)
    .min_size(egui::vec2(180.0, 48.0))
}

#[allow(dead_code)]
pub fn button_danger(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(ERROR).size(15.0).strong())
        .fill(ERROR_BG)
        .stroke(egui::Stroke::new(2.0, ERROR))
        .corner_radius(CORNER_RADIUS_PILL)
        .min_size(egui::vec2(180.0, 48.0))
}

pub fn button_toggle(is_active: bool, text: impl Into<String>) -> egui::Button<'static> {
    if is_active {
        egui::Button::new(
            egui::RichText::new(text)
                .color(TEXT_PRIMARY)
                .size(12.0)
                .strong(),
        )
        .fill(ACCENT_PRIMARY)
        .stroke(egui::Stroke::new(1.0, ACCENT_DARK))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(60.0, 28.0))
    } else {
        egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(12.0))
            .fill(PANEL_BG_LIGHT)
            .stroke(egui::Stroke::new(1.0, BORDER))
            .corner_radius(CORNER_RADIUS_SMALL)
            .min_size(egui::vec2(60.0, 28.0))
    }
}

pub fn folder_card(enabled: bool) -> egui::Frame {
    let bg = if enabled {
        PANEL_BG_LIGHTER
    } else {
        PANEL_BG_LIGHT
    };
    let border = if enabled { BORDER_LIGHT } else { BORDER };
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(16.0)
        .stroke(egui::Stroke::new(1.0, border))
}

pub fn status_badge_with_bg(
    ui: &mut egui::Ui,
    status: &str,
    icon: &str,
    color: egui::Color32,
    bg: egui::Color32,
) {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(egui::vec2(16.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).size(16.0));
                ui.label(egui::RichText::new(status).color(color).size(14.0).strong());
            });
        });
}

pub fn log_entry_success(
    ui: &mut egui::Ui,
    timestamp: &str,
    filename: &str,
    size: &str,
    duration: &str,
) {
    card_frame(SUCCESS_BG).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("✓").color(SUCCESS).size(18.0));
            ui.label(label_muted(timestamp));
            ui.label(label_primary(filename));
        });
        ui.horizontal(|ui| {
            ui.add_space(26.0);
            ui.label(label_muted(&format!("{} • {}", size, duration)));
        });
    });
}

pub fn log_entry_processing(ui: &mut egui::Ui, timestamp: &str, filename: &str, progress: f32) {
    card_frame(PROCESSING_BG).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("⏳").color(PROCESSING).size(16.0));
            ui.label(label_muted(timestamp));
            ui.label(label_primary(filename));
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(26.0);
            ui.add(
                egui::ProgressBar::new(progress)
                    .text(format!("{:.0}%", progress * 100.0))
                    .fill(PROCESSING)
                    .corner_radius(4.0)
                    .desired_width(200.0),
            );
        });
    });
}

pub fn log_entry_error(ui: &mut egui::Ui, timestamp: &str, filename: &str, message: &str) {
    card_frame(ERROR_BG).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("✗").color(ERROR).size(18.0));
            ui.label(label_muted(timestamp));
            ui.label(label_primary(filename));
        });
        ui.horizontal(|ui| {
            ui.add_space(26.0);
            ui.label(egui::RichText::new(message).color(ERROR).size(12.0));
        });
    });
}

pub fn log_entry_simple(ui: &mut egui::Ui, timestamp: &str, message: &str, success: bool) {
    let (icon, bg) = if success {
        ("✓", SUCCESS_BG)
    } else {
        ("✗", ERROR_BG)
    };
    let color = if success { SUCCESS } else { ERROR };

    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(egui::vec2(14.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(icon).color(color).size(16.0));
                ui.label(label_muted(timestamp));
                ui.label(label_secondary(message));
            });
        });
}

pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_duration(seconds: u64) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}
