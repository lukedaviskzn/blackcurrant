#[derive(Debug, Clone, Default)]
pub struct AlertModal {
    pub title: String,
    pub description: Option<String>,
}

impl AlertModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context) -> bool {
        let mut close_modal = false;
        
        egui::Window::new(&self.title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    if let Some(desc) = &self.description {
                        ui.label(desc);
                    }
                    if ui.button("Ok").clicked() {
                        close_modal = true;
                    }
                });
            });
        
        return close_modal;
    }
}
