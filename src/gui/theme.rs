#![cfg(feature = "gui")]

use eframe::egui;

pub const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(10, 10, 10);
pub const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(20, 20, 20);
pub const PANEL_BG_LIGHT: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);

pub const ACCENT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(230, 57, 70);
pub const ACCENT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(255, 107, 107);
pub const ACCENT_DARK: egui::Color32 = egui::Color32::from_rgb(180, 45, 55);

pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(245, 245, 245);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(180, 180, 180);
pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(102, 102, 102);

pub const BORDER: egui::Color32 = egui::Color32::from_rgb(42, 42, 42);
pub const BORDER_ACCENT: egui::Color32 = egui::Color32::from_rgb(60, 60, 60);

pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(78, 205, 196);
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(255, 68, 68);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 193, 7);

pub fn theme_button(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(14.0))
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn theme_button_secondary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(14.0))
        .fill(PANEL_BG_LIGHT)
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn theme_button_primary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text)
            .color(egui::Color32::WHITE)
            .size(14.0)
            .strong(),
    )
    .fill(ACCENT_PRIMARY)
    .stroke(egui::Stroke::new(1.0, ACCENT_DARK))
    .min_size(egui::vec2(150.0, 36.0))
}

pub fn theme_button_danger(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text)
            .color(egui::Color32::WHITE)
            .size(14.0)
            .strong(),
    )
    .fill(ERROR)
    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 50, 50)))
    .min_size(egui::vec2(150.0, 36.0))
}

pub fn draw_horizontal_line(ui: &mut egui::Ui) {
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();

    painter.line_segment(
        [
            egui::pos2(rect.left(), rect.top() + 1.0),
            egui::pos2(rect.right(), rect.top() + 1.0),
        ],
        egui::Stroke::new(1.0, BORDER),
    );

    ui.add_space(8.0);
}
