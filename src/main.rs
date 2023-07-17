// disable console on windows in release mode
#![cfg_attr(
    all(
        target_os = "windows",
        not(debug_assertions),
    ),
    windows_subsystem = "windows"
)]

mod app;
mod records;
mod modals;
mod panels;

fn main() -> eframe::Result<()> {
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::Vec2 { x: 1024.0, y: 600.0 });
    eframe::run_native(
        "Blackcurrant Reception Management System",
        native_options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    )
}
