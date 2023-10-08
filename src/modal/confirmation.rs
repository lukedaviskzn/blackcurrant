#[derive(Debug, Clone, Default)]
pub struct ConfirmationModal {
    pub title: String,
    pub description: Option<String>,
    pub confirmed: bool,
}

impl ConfirmationModal {
    pub fn new(title: impl Into<String>, description: Option<impl Into<String>>) -> ConfirmationModal {
        ConfirmationModal {
            title: title.into(),
            description: description.map(|d| d.into()),
            confirmed: false,
        }
    }

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
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.button("Cancel").clicked() {
                        self.confirmed = false;
                        close_modal = true;
                    }
                    if ui.button("OK").clicked() {
                        self.confirmed = true;
                        close_modal = true;
                    }
                });
            });
        
        return close_modal;
    }
}
