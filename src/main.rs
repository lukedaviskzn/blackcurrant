#![windows_subsystem = "windows"]

mod app;
mod records;
mod modals;
mod panels;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Blackcurrant",
        native_options,
        Box::new(|cc| Box::new(app::App::new(cc))),
    )
}
