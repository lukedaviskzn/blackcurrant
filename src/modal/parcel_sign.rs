use crate::{app::{NAME_MAX_LENGTH, NOTES_MAX_LENGTH}, records::{ParcelStorage, InsertableStorage, NewParcelRecord}};

use super::{render_modal_text_entry, filter_required, filter_length};

#[derive(Debug, Clone, Default)]
pub struct ParcelSignModal {
    pub parcel_desc: String,
    pub parcel_desc_error: Option<String>,
    pub student_name: String,
    pub student_name_error: Option<String>,
    pub receptionist: String,
    pub receptionist_error: Option<String>,
    pub notes: String,
    pub notes_error: Option<String>,
}

impl ParcelSignModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context, parcel_records: &mut ParcelStorage) -> bool {
        let mut close_modal = false;

        egui::Window::new("Sign In Parcel")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // Key
                render_modal_text_entry(ui, "Parcel Description", &self.parcel_desc_error, &mut self.parcel_desc, NAME_MAX_LENGTH);

                // Student Name
                render_modal_text_entry(ui, "Recipient Name", &self.student_name_error, &mut self.student_name, NAME_MAX_LENGTH);

                // Receptionist
                render_modal_text_entry(ui, "Receptionist", &self.receptionist_error, &mut self.receptionist, NAME_MAX_LENGTH);
                
                // Notes
                render_modal_text_entry(ui, "Notes", &self.notes_error, &mut self.notes, NOTES_MAX_LENGTH);

                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Sign In").clicked() {
                        let mut error = false;
                        
                        // Key
                        self.parcel_desc_error = None;

                        let parcel_desc = self.parcel_desc.trim();

                        error |= filter_required(parcel_desc, &mut self.parcel_desc_error);
                        error |= filter_length(parcel_desc, NAME_MAX_LENGTH, &mut self.parcel_desc_error);

                        // Student Name
                        self.student_name_error = None;

                        let student_name = self.student_name.trim();

                        error |= filter_required(student_name, &mut self.student_name_error);
                        error |= filter_length(student_name, NAME_MAX_LENGTH, &mut self.student_name_error);

                        // Receptionist
                        self.receptionist_error = None;

                        let receptionist = self.receptionist.trim();

                        error |= filter_required(receptionist, &mut self.receptionist_error);
                        error |= filter_length(receptionist, NAME_MAX_LENGTH, &mut self.receptionist_error);

                        // Notes
                        self.notes_error = None;

                        let notes = self.notes.trim();

                        error |= filter_length(notes, NAME_MAX_LENGTH, &mut self.notes_error);

                        // Entry valid, add record
                        if !error {
                            parcel_records.insert(NewParcelRecord {
                                parcel_desc: &self.parcel_desc,
                                student_name: &self.student_name,
                                receptionist: &self.receptionist,
                                notes: &self.notes,
                            }).expect("failed to add parcel record to database");
                            
                            close_modal = true;

                            log::info!("added parcel record");
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        close_modal = true;
                    }
                });
            });

        return close_modal;
    }
}
