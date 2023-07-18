use crate::app::{AppConfig, NAME_MAX_LENGTH};

use super::{render_modal_text_entry, filter_length};

#[derive(Debug)]
pub struct SettingsModal {
    pub facility_name: String,
    facility_name_error: Option<String>,
    pub cancelled: bool,
}

impl SettingsModal {
    pub fn new(config: &AppConfig) -> SettingsModal {
        SettingsModal {
            facility_name: config.facility_name.clone(),
            facility_name_error: None,
            cancelled: false,
        }
    }

    pub fn render(&mut self, ctx: &eframe::egui::Context) -> bool {
        let mut close_modal = false;
        
        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                render_modal_text_entry(ui, "Facility Name", &self.facility_name_error, &mut self.facility_name, NAME_MAX_LENGTH);
                
                // Buttons
                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    if ui.button("OK").clicked() {
                        if !filter_length(&self.facility_name, NAME_MAX_LENGTH, &mut self.facility_name_error) {
                            // only close if no error
                            close_modal = true;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        close_modal = true;
                        self.cancelled = true;
                    }
                });
            });
        
        return close_modal;
    }
}
