#![cfg(feature = "gui")]

use eframe::egui;

pub const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(10, 10, 10);
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(18, 18, 18);
pub const PANEL_BG_LIGHT: egui::Color32 = egui::Color32::from_rgb(28, 28, 28);
pub const PANEL_BG_LIGHTER: egui::Color32 = egui::Color32::from_rgb(38, 38, 38);

pub const ACCENT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(230, 57, 70);
pub const ACCENT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(255, 107, 107);
pub const ACCENT_DARK: egui::Color32 = egui::Color32::from_rgb(180, 45, 55);

pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(245, 245, 245);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(170, 170, 170);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(100, 100, 100);

pub const BORDER: egui::Color32 = egui::Color32::from_rgb(40, 40, 40);

pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(78, 205, 196);
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(255, 68, 68);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 193, 7);
pub const PROCESSING: egui::Color32 = egui::Color32::from_rgb(100, 149, 237);

pub fn panel_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_BG)
        .corner_radius(8.0)
        .inner_margin(16.0)
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn inner_panel() -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_BG_LIGHT)
        .corner_radius(6.0)
        .inner_margin(10.0)
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn section_header(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(14.0)
        .color(ACCENT_PRIMARY)
        .strong()
}

pub fn label_primary(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_PRIMARY).size(13.0)
}

pub fn label_secondary(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_SECONDARY).size(12.0)
}

pub fn label_muted(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_MUTED).size(11.0)
}

pub fn text_edit_style(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(text)
        .text_color(TEXT_PRIMARY)
        .background_color(PANEL_BG_LIGHTER)
        .desired_width(f32::INFINITY)
}

pub fn button_secondary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(12.0))
        .fill(PANEL_BG_LIGHT)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(4.0)
}

pub fn button_primary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text)
            .color(egui::Color32::WHITE)
            .size(13.0)
            .strong(),
    )
    .fill(ACCENT_PRIMARY)
    .stroke(egui::Stroke::new(1.0, ACCENT_DARK))
    .corner_radius(6.0)
    .min_size(egui::vec2(160.0, 38.0))
}

pub fn button_danger(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text)
            .color(egui::Color32::WHITE)
            .size(13.0)
            .strong(),
    )
    .fill(ERROR)
    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(180, 50, 50)))
    .corner_radius(6.0)
    .min_size(egui::vec2(160.0, 38.0))
}

pub fn status_badge(status: &str, color: egui::Color32) -> egui::RichText {
    egui::RichText::new(format!("● {}", status))
        .color(color)
        .size(13.0)
        .strong()
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
