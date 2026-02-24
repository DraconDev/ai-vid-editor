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
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(gui::App::new()))
        }),
    )
}
