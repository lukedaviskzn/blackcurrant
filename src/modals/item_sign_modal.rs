use crate::{records::{ItemTypeStorage, ItemStorage, Storage, InsertableStorage, NewItemRecord}, app::{MAX_QUANTITY, NAME_MAX_LENGTH, STUDENT_NUMBER_LENGTH, NOTES_MAX_LENGTH}};

use super::{render_modal_text_entry, filter_student_number, filter_required, filter_length};

#[derive(Debug, Clone)]
pub struct ItemSignModal {
    pub item: String,
    pub item_error: Option<String>,
    pub quantity: i64,
    pub quantity_str: String,
    pub quantity_error: Option<String>,
    pub student_name: String,
    pub student_name_error: Option<String>,
    pub student_number: String,
    pub student_number_error: Option<String>,
    pub receptionist: String,
    pub receptionist_error: Option<String>,
    pub notes: String,
    pub notes_error: Option<String>,
}

impl Default for ItemSignModal {
    fn default() -> Self {
        Self {
            item: Default::default(),
            item_error: Default::default(),
            quantity: 1,
            quantity_str: "1".into(),
            quantity_error: Default::default(),
            student_name: Default::default(),
            student_name_error: Default::default(),
            student_number: Default::default(),
            student_number_error: Default::default(),
            receptionist: Default::default(),
            receptionist_error: Default::default(),
            notes: Default::default(),
            notes_error: Default::default(),
        }
    }
}

impl ItemSignModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context, item_types: &ItemTypeStorage, item_records: &mut ItemStorage) -> bool {
        let mut add_record = None;
        let mut close_modal = false;

        if self.item.len() == 0 && item_types.get_all().len() == 1 {
            self.item = item_types.get_all()[0].clone();
        }

        egui::Window::new("Sign Out Item")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // Item

                ui.label("Item");
                
                egui::ComboBox::from_label("")
                    .width(192.0)
                    .selected_text(&self.item)
                    .show_ui(ui, |ui| {
                        for item in item_types.get_all() {
                            ui.selectable_value(&mut self.item, item.clone(), item);
                        }
                    });
                
                if let Some(error) = &self.item_error {
                    ui.colored_label(egui::Rgba::from_rgb(0.25, 0.0, 0.0), error);
                }

                ui.add_space(4.0);

                // Quantity
                ui.label("Quantity");
                
                ui.horizontal(|ui| {
                    let mut updated = false;

                    if ui.button("+").clicked() {
                        self.quantity += 1;
                        updated = true;
                    }

                    let response = ui.add(egui::TextEdit::singleline(&mut self.quantity_str).desired_width(64.0));
                    
                    if !updated && (response.lost_focus() || response.clicked_elsewhere()) {
                        self.quantity = self.quantity_str.chars().filter(|c| '0' <= *c && *c <= '9').collect::<String>().parse().unwrap_or(1);
                        updated = true;
                    }
                    
                    if ui.button("−").clicked() {
                        self.quantity -= 1;
                        updated = true;
                    }

                    if updated {
                        self.quantity = self.quantity.clamp(1, MAX_QUANTITY);
                        self.quantity_str = self.quantity.to_string();
                    }
                });

                ui.add_space(4.0);

                // Student Name
                render_modal_text_entry(ui, "Student Name", &self.student_name_error, &mut self.student_name, NAME_MAX_LENGTH);

                // Student Number
                render_modal_text_entry(ui, "Student Number", &self.student_number_error, &mut self.student_number, STUDENT_NUMBER_LENGTH);
                
                // Student Receptionist
                render_modal_text_entry(ui, "Receptionist", &self.receptionist_error, &mut self.receptionist, NAME_MAX_LENGTH);
                
                // Notes
                render_modal_text_entry(ui, "Notes", &self.notes_error, &mut self.notes, NOTES_MAX_LENGTH);
                
                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Sign Out").clicked() {
                        let mut error = false;
                        
                        // Item
                        self.item_error = None;

                        let item = self.item.as_str();

                        error |= filter_required(item, &mut self.item_error);

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

                        // Receptionist
                        self.receptionist_error = None;

                        let receptionist = self.receptionist.trim();

                        if receptionist.len() == 0 {
                            self.receptionist_error = Some("Required".into());
                            error = true;
                        }

                        if receptionist.len() > NAME_MAX_LENGTH {
                            self.receptionist_error = Some(format!("Name too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

                        // Notes
                        
                        self.notes_error = None;

                        let notes = self.notes.trim();

                        if notes.len() > NAME_MAX_LENGTH {
                            self.notes_error = Some(format!("Notes too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

                        // Entry valid, add record
                        if !error {
                            add_record = Some(NewItemRecord {
                                item: &self.item,
                                quantity: self.quantity,
                                student_name: &self.student_name,
                                student_number: &self.student_number,
                                receptionist: &self.receptionist,
                                notes: &self.notes,
                            });
                            close_modal = true;

                            log::info!("added item record");
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        close_modal = true;
                    }
                });
            });
        
        if let Some(record) = add_record {
            item_records.insert(record).expect("failed to add item record to database");
        }
        
        return close_modal;
    }
}
