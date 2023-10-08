use std::sync::{Arc, Mutex};

use crate::{records::{KeyTypeStorage, KeyStorage, Storage, InsertableStorage, NewKeyRecord, StudentInfo}, app::{DATE_TIME_FORMAT, NAME_MAX_LENGTH, STUDENT_NUMBER_LENGTH, NOTES_MAX_LENGTH}};

use super::{render_modal_text_entry, filter_student_number, filter_required, filter_length, render_student_number_popup};

#[derive(Debug, Clone, Default)]
pub struct KeySignModal {
    pub key: String,
    pub key_error: Option<String>,
    pub student_name: String,
    pub student_name_error: Option<String>,
    pub student_number: String,
    pub student_number_error: Option<String>,
    // pub receptionist: String,
    // pub receptionist_error: Option<String>,
    pub notes: String,
    pub notes_error: Option<String>,
}

impl KeySignModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context, key_types: &KeyTypeStorage, key_records: &mut KeyStorage, student_info: Arc<Mutex<StudentInfo>>) -> bool {
        let mut add_record = None;
        let mut close_modal = false;

        egui::Window::new("Sign Out Key")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // Key

                ui.label("Key");
                
                egui::ComboBox::from_label("")
                    .width(192.0)
                    .selected_text(&self.key)
                    .show_ui(ui, |ui| {
                        for key in key_types.get_all() {
                            ui.selectable_value(&mut self.key, key.clone(), key);
                        }
                    });
                
                if let Some(error) = &self.key_error {
                    ui.colored_label(egui::Rgba::from_rgb(0.25, 0.0, 0.0), error);
                }

                ui.add_space(4.0);

                // Student Number                
                let resp = render_modal_text_entry(ui, "Student Number", &self.student_number_error, &mut self.student_number, STUDENT_NUMBER_LENGTH);
                render_student_number_popup(ui, student_info, "key_sign_student_number_popup".into(), &resp, &mut self.student_number, &mut self.student_name);

                // Student Name
                render_modal_text_entry(ui, "Student Name", &self.student_name_error, &mut self.student_name, NAME_MAX_LENGTH);
                
                // Notes
                render_modal_text_entry(ui, "Notes", &self.notes_error, &mut self.notes, NOTES_MAX_LENGTH);
                
                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Sign Out").clicked() {
                        let mut error = false;
                        
                        // Key
                        self.key_error = None;

                        let key = self.key.as_str();

                        error |= filter_required(key, &mut self.key_error);

                        if let Some(record) = key_records.get_signed_out(key).expect("failed to get key signed status from database") {
                            self.key_error = Some(format!("Key already signed out on {}.", chrono::DateTime::<chrono::Local>::from(record.time_out).format(DATE_TIME_FORMAT)));
                            error = true;
                        }

                        // Student Name
                        self.student_name_error = None;

                        let student_name = self.student_name.trim();

                        error |= filter_required(student_name, &mut self.student_name_error);
                        error |= filter_length(student_name, NAME_MAX_LENGTH, &mut self.student_name_error);

                        // Student Number
                        self.student_number_error = None;

                        let student_number = self.student_number.trim();

                        // filter student number first so it overwrites with "required" text if blank
                        error |= filter_student_number(student_number, &mut self.student_number_error);
                        error |= filter_required(student_number, &mut self.student_number_error);

                        // Notes
                        
                        self.notes_error = None;

                        let notes = self.notes.trim();

                        error |= filter_length(notes, NAME_MAX_LENGTH, &mut self.notes_error);

                        // Entry valid, add record
                        if !error {
                            add_record = Some(NewKeyRecord {
                                key: &self.key,
                                student_name: &self.student_name,
                                student_number: &self.student_number,
                                notes: &self.notes,
                            });
                            close_modal = true;

                            log::info!("added key record");
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        close_modal = true;
                    }
                });
            });
        
        if let Some(record) = add_record {
            key_records.insert(record).expect("failed to add key record to database");
        }
        
        return close_modal;
    }
}
