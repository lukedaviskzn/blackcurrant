#[derive(Debug, Default)]
pub struct ExitModal {
    pub close_process: bool,
}

impl ExitModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) -> bool {
        let mut close_modal = false;
        
        egui::Window::new("Are you sure you want to exit?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
                .show(ctx, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Yes").clicked() {
                            frame.close();
                            self.close_process = true;
                        }
                        // ui.separator();
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });
        
        return close_modal;
    }
}
