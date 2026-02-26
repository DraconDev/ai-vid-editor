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

pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(180, 90, 95);
pub const SUCCESS_BG: egui::Color32 = egui::Color32::from_rgb(50, 25, 27);
pub const ERROR: egui::Color32 = egui::Color32::from_rgb(255, 68, 68);
pub const ERROR_BG: egui::Color32 = egui::Color32::from_rgb(55, 20, 20);
pub const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 193, 7);
pub const PROCESSING: egui::Color32 = egui::Color32::from_rgb(100, 149, 237);
pub const PROCESSING_BG: egui::Color32 = egui::Color32::from_rgb(25, 40, 60);
pub const SETTINGS_PANEL_BG: egui::Color32 = egui::Color32::from_rgb(14, 17, 23);
pub const SETTINGS_SECTION_BG: egui::Color32 = egui::Color32::from_rgb(20, 24, 33);
pub const SETTINGS_SECTION_BG_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgb(35, 24, 31);
pub const SETTINGS_SECTION_BORDER_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgb(112, 54, 66);

pub const CORNER_RADIUS: f32 = 12.0;
pub const CORNER_RADIUS_SMALL: f32 = 8.0;
#[allow(dead_code)]
pub const CORNER_RADIUS_PILL: f32 = 24.0;

#[allow(dead_code)]
pub fn glow_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(230, 57, 70, 80)
}

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

pub fn settings_panel_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(SETTINGS_PANEL_BG)
        .corner_radius(CORNER_RADIUS)
        .inner_margin(22.0)
        .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
        .shadow(egui::epaint::Shadow {
            offset: [0, 6],
            blur: 20,
            spread: 0,
            color: egui::Color32::from_black_alpha(100),
        })
}

pub fn settings_section_frame(highlight: bool) -> egui::Frame {
    let (bg, border) = if highlight {
        (
            SETTINGS_SECTION_BG_HIGHLIGHT,
            SETTINGS_SECTION_BORDER_HIGHLIGHT,
        )
    } else {
        (SETTINGS_SECTION_BG, BORDER_LIGHT)
    };
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(14.0)
        .stroke(egui::Stroke::new(1.0, border))
}

pub fn settings_toggle_frame(enabled: bool) -> egui::Frame {
    let border = if enabled { ACCENT_DARK } else { BORDER_LIGHT };
    let bg = if enabled {
        egui::Color32::from_rgb(43, 29, 36)
    } else {
        egui::Color32::from_rgb(23, 28, 38)
    };
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(egui::vec2(12.0, 10.0))
        .stroke(egui::Stroke::new(1.0, border))
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
        .inner_margin(egui::vec2(10.0, 6.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
}

pub fn folder_card_compact(enabled: bool) -> egui::Frame {
    let bg = if enabled {
        PANEL_BG_LIGHTER
    } else {
        PANEL_BG_LIGHT
    };
    let border = if enabled { BORDER_LIGHT } else { BORDER };
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(egui::vec2(14.0, 10.0))
        .stroke(egui::Stroke::new(1.0, border))
}

#[allow(dead_code)]
pub fn section_header(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(17.0)
        .color(ACCENT_PRIMARY)
        .strong()
}

pub fn label_primary(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_PRIMARY).size(16.0)
}

pub fn label_secondary(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_SECONDARY).size(15.0)
}

pub fn label_muted(text: &str) -> egui::RichText {
    egui::RichText::new(text).color(TEXT_MUTED).size(14.0)
}

pub fn text_edit_style(text: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(text)
        .text_color(TEXT_PRIMARY)
        .background_color(PANEL_BG_LIGHTER)
        .desired_width(f32::INFINITY)
        .cursor_at_end(true)
        .min_size(egui::vec2(0.0, 40.0))
        .vertical_align(egui::Align::Center)
}

pub fn button_secondary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_PRIMARY).size(15.0))
        .fill(PANEL_BG_LIGHTER)
        .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(80.0, 38.0))
}

pub fn button_small(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(14.0))
        .fill(PANEL_BG_LIGHT)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(60.0, 34.0))
}

#[allow(dead_code)]
pub fn button_icon(icon: &str, _tooltip: &str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(icon).size(16.0))
        .fill(PANEL_BG_LIGHTER)
        .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(36.0, 36.0))
        .sense(egui::Sense::click())
}

#[allow(dead_code)]
pub fn button_primary(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(
        egui::RichText::new(text)
            .color(egui::Color32::WHITE)
            .size(17.0)
            .strong(),
    )
    .fill(ACCENT_PRIMARY)
    .stroke(egui::Stroke::new(2.0, ACCENT_DARK))
    .corner_radius(CORNER_RADIUS_PILL)
    .min_size(egui::vec2(180.0, 52.0))
}

#[allow(dead_code)]
pub fn button_danger(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(ERROR).size(17.0).strong())
        .fill(ERROR_BG)
        .stroke(egui::Stroke::new(2.0, ERROR))
        .corner_radius(CORNER_RADIUS_PILL)
        .min_size(egui::vec2(180.0, 52.0))
}

pub fn button_toggle(is_active: bool, text: impl Into<String>) -> egui::Button<'static> {
    let btn = if is_active {
        egui::Button::new(
            egui::RichText::new(text)
                .color(TEXT_PRIMARY)
                .size(13.0)
                .strong(),
        )
        .fill(ACCENT_PRIMARY)
        .stroke(egui::Stroke::new(1.0, ACCENT_PRIMARY))
    } else {
        egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(13.0))
            .fill(PANEL_BG_LIGHT)
            .stroke(egui::Stroke::new(1.0, BORDER))
    };
    btn.corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(55.0, 30.0))
}

pub fn button_tab(is_active: bool, text: impl Into<String>) -> egui::Button<'static> {
    let btn = if is_active {
        egui::Button::new(
            egui::RichText::new(text)
                .color(TEXT_PRIMARY)
                .size(15.0)
                .strong(),
        )
        .fill(PANEL_BG_LIGHTER)
        .stroke(egui::Stroke::new(0.0, egui::Color32::TRANSPARENT))
    } else {
        egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(15.0))
            .fill(PANEL_BG)
            .stroke(egui::Stroke::new(0.0, egui::Color32::TRANSPARENT))
    };
    btn.corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(85.0, 36.0))
}

pub fn button_pill(is_active: bool, text: impl Into<String>) -> egui::Button<'static> {
    let btn = if is_active {
        egui::Button::new(
            egui::RichText::new(text)
                .color(TEXT_PRIMARY)
                .size(13.0)
                .strong(),
        )
        .fill(ACCENT_PRIMARY)
        .stroke(egui::Stroke::new(1.0, ACCENT_PRIMARY))
    } else {
        egui::Button::new(egui::RichText::new(text).color(TEXT_SECONDARY).size(13.0))
            .fill(PANEL_BG_LIGHTER)
            .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
    };
    btn.corner_radius(CORNER_RADIUS_PILL)
        .min_size(egui::vec2(55.0, 30.0))
}

pub fn button_add(text: impl Into<String>) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(text).color(ACCENT_PRIMARY).size(14.0))
        .fill(PANEL_BG)
        .stroke(egui::Stroke::new(1.0, ACCENT_PRIMARY))
        .corner_radius(CORNER_RADIUS_SMALL)
        .min_size(egui::vec2(65.0, 32.0))
}

pub fn slider_glow(
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    ui: &mut egui::Ui,
) -> egui::Response {
    let spacing = ui.spacing();
    let slider_width = spacing.slider_width;
    let rail_height = 4.0;
    let handle_radius = 7.0;

    let available_width = ui.available_width();
    let width = slider_width.min(available_width).max(100.0);

    let (rect, mut response) = ui.allocate_exact_size(
        egui::vec2(width, handle_radius * 2.0 + 4.0),
        egui::Sense::click_and_drag(),
    );

    let range_size = *range.end() - *range.start();
    let fraction = (*value - *range.start()) / range_size;

    let handle_x = egui::lerp(rect.left()..=rect.right(), fraction);
    let handle_center = egui::pos2(handle_x, rect.center().y);

    let track_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.center().y - rail_height / 2.0),
        egui::vec2(rect.width(), rail_height),
    );

    let painter = ui.painter();

    painter.rect_filled(track_rect, 2.0, egui::Color32::from_rgb(40, 40, 40));

    if fraction > 0.0 {
        let filled_width = (handle_x - rect.left()).max(0.0);
        let filled_rect =
            egui::Rect::from_min_size(track_rect.left_top(), egui::vec2(filled_width, rail_height));
        painter.rect_filled(filled_rect, 2.0, ACCENT_PRIMARY);
    }

    painter.circle_filled(handle_center, handle_radius, ACCENT_PRIMARY);
    painter.circle_filled(
        handle_center,
        handle_radius - 2.5,
        egui::Color32::from_rgb(255, 120, 130),
    );

    if response.clicked() || response.dragged() {
        let pointer_pos = ui.input(|i| i.pointer.interact_pos());
        if let Some(pos) = pointer_pos {
            let new_fraction = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            let new_value = *range.start() + new_fraction * range_size;
            let stepped = (new_value / 1.0).round() * 1.0;
            *value = stepped.clamp(*range.start(), *range.end());
            response.mark_changed();
        }
    }

    let value_text = format!("{}", *value as i32);
    let text_color = TEXT_SECONDARY;
    let font_id = egui::FontId::proportional(13.0);
    let text_galley = painter.layout_no_wrap(value_text, font_id, text_color);
    let text_pos = egui::pos2(
        rect.right() + 12.0,
        rect.center().y - text_galley.size().y / 2.0,
    );
    painter.galley(text_pos, text_galley, text_color);

    response
}

pub fn modal_overlay() -> egui::Frame {
    egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200))
}

pub fn modal_dialog() -> egui::Frame {
    egui::Frame::NONE
        .fill(PANEL_BG)
        .corner_radius(CORNER_RADIUS)
        .inner_margin(24.0)
        .stroke(egui::Stroke::new(1.0, BORDER_LIGHT))
        .shadow(egui::epaint::Shadow {
            offset: [0, 8],
            blur: 32,
            spread: 0,
            color: egui::Color32::from_black_alpha(150),
        })
}

pub fn preset_badge(preset: &str, ui: &mut egui::Ui) {
    let color = match preset {
        "youtube" => egui::Color32::from_rgb(230, 57, 70),
        "shorts" => egui::Color32::from_rgb(255, 140, 0),
        "podcast" => egui::Color32::from_rgb(100, 149, 237),
        _ => ACCENT_PRIMARY,
    };
    egui::Frame::NONE
        .fill(color)
        .corner_radius(4.0)
        .inner_margin(egui::vec2(12.0, 6.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(preset)
                    .color(TEXT_PRIMARY)
                    .size(12.0)
                    .strong(),
            );
        });
}

pub fn settings_value_badge(ui: &mut egui::Ui, value: &str) {
    egui::Frame::NONE
        .fill(egui::Color32::from_rgb(46, 24, 28))
        .corner_radius(4.0)
        .inner_margin(egui::vec2(10.0, 5.0))
        .stroke(egui::Stroke::new(1.0, ACCENT_DARK))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(value)
                    .color(TEXT_PRIMARY)
                    .size(12.0)
                    .strong(),
            );
        });
}

pub fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        let start = &path[..max_len / 2 - 2];
        let end = &path[path.len() - max_len / 2 + 2..];
        format!("{}...{}", start, end)
    }
}

pub fn status_badge_with_bg(
    ui: &mut egui::Ui,
    status: &str,
    dot_color: egui::Color32,
    bg: egui::Color32,
) {
    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(egui::vec2(14.0, 10.0))
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, dot_color);
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(status)
                        .color(dot_color)
                        .size(15.0)
                        .strong(),
                );
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
            let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 4.0, SUCCESS);
            ui.add_space(6.0);
            ui.label(label_muted(timestamp));
            ui.add_space(6.0);
            ui.label(label_primary(filename));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(label_muted(&format!("{} - {}", size, duration)));
            });
        });
    });
}

pub fn log_entry_processing(ui: &mut egui::Ui, timestamp: &str, filename: &str, progress: f32) {
    card_frame(PROCESSING_BG).show(ui, |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 5.0, PROCESSING);
            ui.add_space(6.0);
            ui.label(label_muted(timestamp));
            ui.add_space(8.0);
            ui.label(label_primary(filename));
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(20.0);
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
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 5.0, ERROR);
            ui.add_space(6.0);
            ui.label(label_muted(timestamp));
            ui.add_space(8.0);
            ui.label(label_primary(filename));
        });
        ui.add_space(4.0);
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new(message).color(ERROR).size(14.0));
        });
    });
}

pub fn log_entry_simple(ui: &mut egui::Ui, timestamp: &str, message: &str, success: bool) {
    let (color, bg) = if success {
        (SUCCESS, SUCCESS_BG)
    } else {
        (ERROR, ERROR_BG)
    };

    egui::Frame::NONE
        .fill(bg)
        .corner_radius(CORNER_RADIUS_SMALL)
        .inner_margin(egui::vec2(12.0, 8.0))
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, color);
                ui.add_space(6.0);
                ui.label(label_muted(timestamp));
                ui.add_space(8.0);
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
