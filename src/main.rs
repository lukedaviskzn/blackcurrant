// disable console on windows in release mode
#![cfg_attr(
    all(
        target_os = "windows",
        not(debug_assertions),
    ),
    windows_subsystem = "windows"
)]

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

mod app;
mod records;
mod modal;
mod panel;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let icon = image::load_from_memory(include_bytes!("../icon/icon.png")).expect("failed to read icon");

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2 { x: 1024.0, y: 600.0 }),
        icon_data: Some(eframe::IconData {
            rgba: icon.to_rgba8().into_vec(),
            width: icon.width(),
            height: icon.height(),
        }),
        ..Default::default()
    };

    eframe::run_native(
        "Blackcurrant",
        native_options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    )
}
