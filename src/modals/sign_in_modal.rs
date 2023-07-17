use crate::{records::TimeReceptionistUpdateableStorage, app::NAME_MAX_LENGTH};

use super::{render_modal_text_entry, filter_required, filter_length};

#[derive(Debug, Clone)]
pub struct SignInModal<I: Copy> {
    pub receptionist: String,
    pub receptionist_error: Option<String>,
    pub record_id: I,
}

impl<I: Copy> SignInModal<I> {
    pub fn new(id: I) -> SignInModal<I> {
        SignInModal {
            receptionist: Default::default(),
            receptionist_error: Default::default(),
            record_id: id,
        }
    }

    pub fn render<T>(&mut self, ctx: &eframe::egui::Context, records: &mut impl TimeReceptionistUpdateableStorage<T, I>) -> bool {
        let mut update_record = false;
        let mut close_modal = false;

        egui::Window::new("Sign In")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                
                // Receptionist
                render_modal_text_entry(ui, "Receptionist", &self.receptionist_error, &mut self.receptionist);
                
                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Sign In").clicked() {
                        let mut error = false;
                        
                        // Receptionist
                        
                        self.receptionist_error = None;

                        let receptionist = self.receptionist.trim();

                        error |= filter_required(receptionist, &mut self.receptionist_error);
                        error |= filter_length(receptionist, NAME_MAX_LENGTH, &mut self.receptionist_error);

                        // Entry valid, add record
                        if !error {
                            update_record = true;
                            close_modal = true;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        close_modal = true;
                    }
                });
            });
        
        if update_record {
            records.update_receptionist_and_time(self.record_id, &self.receptionist).expect("failed to update receptionist on record");
        }
        
        return close_modal;
    }
}
