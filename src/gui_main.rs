#![cfg(feature = "gui")]

use eframe::egui;

mod gui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 500.0])
            .with_title("AI Video Processor"),
        ..Default::default()
    };

    eframe::run_native(
        "AI Video Processor",
        options,
        Box::new(|cc| {
            configure_dark_theme(&cc.egui_ctx);
            Ok(Box::new(gui::App::new()))
        }),
    )
}

fn configure_dark_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals = egui::Visuals::dark();

    style.visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10);
    style.visuals.window_fill = egui::Color32::from_rgb(20, 20, 20);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(5, 5, 5);

    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 35, 35);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 45, 45);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 50, 50);

    style.visuals.selection.bg_fill = egui::Color32::from_rgb(230, 57, 70);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 107, 107));

    style.visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 150, 150));
    style.visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200));
    style.visuals.widgets.hovered.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(245, 245, 245));
    style.visuals.widgets.active.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));

    style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(42, 42, 42));

    ctx.set_style(style);
}

fn configure_dark_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals = egui::Visuals::dark();

    style.visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10);
    style.visuals.window_fill = egui::Color32::from_rgb(20, 20, 20);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(5, 5, 5);

    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 35, 35);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 45, 45);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(50, 50, 50);

    style.visuals.selection.bg_fill = egui::Color32::from_rgb(230, 57, 70);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 107, 107));

    style.visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(150, 150, 150));
    style.visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200));
    style.visuals.widgets.hovered.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(245, 245, 245));
    style.visuals.widgets.active.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));

    style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(42, 42, 42));
    style.visuals.panel_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(42, 42, 42));
    style.visuals.separator.color = egui::Color32::from_rgb(42, 42, 42);

    ctx.set_style(style);
}
