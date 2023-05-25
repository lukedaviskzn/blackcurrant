use std::{thread::JoinHandle, path::PathBuf};

use egui_extras::{TableBuilder, Column};
use strum::IntoEnumIterator;

use crate::{records::{KeyTypeStorage, Storage, KeyStorage, KeyRecord, ParcelRecord, ParcelStorage, GameStorage, GameRecord, GameTypeStorage, GameTypeRecord, ItemTypeStorage, ItemRecord, ItemStorage, RecordType, AddibleStorage, DeletableStorage, SignableStorage, QuantitySignableStorage}, app::{DATE_TIME_FORMAT, NAME_MAX_LENGTH, STUDENT_NUMBER_LENGTH, MAX_QUANTITY, STAFF_NUMBER_LENGTH, BACKUP_DATE_TIME_FORMAT}};

fn render_modal_text_entry(ui: &mut egui::Ui, label: &str, error: Option<&str>, input: &mut String) {
    ui.label(label);
    ui.text_edit_singleline(input);
    
    if let Some(error) = error {
        ui.colored_label(egui::Rgba::from_rgb(0.25, 0.0, 0.0), error);
    }

    ui.add_space(4.0);
}

fn filter_student_number(student_number: &str) -> Option<String> {
    let mut error = None;

    match student_number.parse::<i64>() {
        // Maybe Stuff Number
        Ok(_) => {
            if student_number.len() != STAFF_NUMBER_LENGTH {
                error = Some("Invalid staff number.".into());
            }
        },
        // Maybe Student Number
        Err(_) => {
            if student_number.len() != STUDENT_NUMBER_LENGTH ||
                    (student_number.chars().enumerate().filter(|(i, c)| *i < 6 && c.is_alphabetic() || *i >= 6 && c.is_numeric()).count() != STUDENT_NUMBER_LENGTH) {
                error = Some("Invalid student number.".into());
            }
        },
    }

    if student_number.len() == 0 {
        error = Some("Required".into());
    }

    return error;
}

#[derive(Debug, Clone, Default)]
pub struct KeySignModal {
    pub key: String,
    pub key_error: Option<String>,
    pub student_name: String,
    pub student_name_error: Option<String>,
    pub student_number: String,
    pub student_number_error: Option<String>,
    pub receptionist: String,
    pub receptionist_error: Option<String>,
    pub notes: String,
    pub notes_error: Option<String>,
}

impl KeySignModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context, key_types: &KeyTypeStorage, key_records: &mut KeyStorage) -> bool {
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

                // Student Name
                render_modal_text_entry(ui, "Student Name", self.student_name_error.as_ref().map(|s| s.as_str()), &mut self.student_name);

                // Student Number
                render_modal_text_entry(ui, "Student Number", self.student_number_error.as_ref().map(|s| s.as_str()), &mut self.student_number);
                
                // Receptionist
                render_modal_text_entry(ui, "Receptionist", self.receptionist_error.as_ref().map(|s| s.as_str()), &mut self.receptionist);
                
                // Notes
                render_modal_text_entry(ui, "Notes", self.notes_error.as_ref().map(|s| s.as_str()), &mut self.notes);
                
                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Sign Out").clicked() {
                        let mut error = false;
                        
                        // Key
                        
                        self.key_error = None;

                        let key = self.key.as_str();

                        if key.len() == 0 {
                            self.key_error = Some("Required".into());
                            error = true;
                        }

                        if let Some(record) = key_records.get_signed_out(key).unwrap() {
                            self.key_error = Some(format!("Key already signed out on {}.", chrono::DateTime::<chrono::Local>::from(record.time_out).format(DATE_TIME_FORMAT)));
                            error = true;
                        }

                        // Student Name
                        
                        self.student_name_error = None;

                        let student_name = self.student_name.trim();

                        if student_name.len() == 0 {
                            self.student_name_error = Some("Required".into());
                            error = true;
                        }
                        
                        if student_name.len() > NAME_MAX_LENGTH {
                            self.student_name_error = Some(format!("Name too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

                        // Student Number
                        
                        self.student_number_error = None;

                        let student_number = self.student_number.trim();

                        self.student_number_error = filter_student_number(student_number);

                        if self.student_number_error.is_some() {
                            error = true;
                        }

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
                            add_record = Some(KeyRecord {
                                id: 0,
                                key: self.key.clone(),
                                student_name: self.student_name.clone(),
                                student_number: self.student_number.to_uppercase(),
                                receptionist: self.receptionist.clone(),
                                time_out: chrono::Utc::now(),
                                time_in: None,
                                notes: self.notes.clone(),
                            });
                            close_modal = true;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        close_modal = true;
                    }
                });
            });
        
        if let Some(record) = add_record {
            key_records.add(record).unwrap();
        }
        
        return close_modal;
    }
}

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
                render_modal_text_entry(ui, "Parcel Description", self.parcel_desc_error.as_ref().map(|s| s.as_str()), &mut self.parcel_desc);

                // Student Name
                render_modal_text_entry(ui, "Student Name", self.student_name_error.as_ref().map(|s| s.as_str()), &mut self.student_name);

                // Receptionist
                render_modal_text_entry(ui, "Receptionist", self.receptionist_error.as_ref().map(|s| s.as_str()), &mut self.receptionist);
                
                // Notes
                render_modal_text_entry(ui, "Notes", self.notes_error.as_ref().map(|s| s.as_str()), &mut self.notes);

                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Sign In").clicked() {
                        let mut error = false;
                        
                        // Key
                        
                        self.parcel_desc_error = None;

                        let parcel_desc = self.parcel_desc.trim();

                        if parcel_desc.len() == 0 {
                            self.parcel_desc_error = Some("Required".into());
                            error = true;
                        }
                        
                        if parcel_desc.len() > NAME_MAX_LENGTH {
                            self.parcel_desc_error = Some(format!("Description too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

                        // Student Name
                        
                        self.student_name_error = None;

                        let student_name = self.student_name.trim();

                        if student_name.len() == 0 {
                            self.student_name_error = Some("Required".into());
                            error = true;
                        }
                        
                        if student_name.len() > NAME_MAX_LENGTH {
                            self.student_name_error = Some(format!("Name too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

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
                            parcel_records.add(ParcelRecord {
                                id: 0,
                                parcel_desc: self.parcel_desc.clone(),
                                student_name: self.student_name.clone(),
                                receptionist: self.receptionist.clone(),
                                time_in: chrono::Utc::now(),
                                time_out: None,
                                notes: self.notes.clone(),
                            }).unwrap();
                            
                            close_modal = true;
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

#[derive(Debug, Clone)]
pub struct GameSignModal {
    pub game: String,
    pub game_error: Option<String>,
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

impl Default for GameSignModal {
    fn default() -> Self {
        Self {
            game: Default::default(),
            game_error: Default::default(),
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

impl GameSignModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context, game_types: &GameTypeStorage, game_records: &mut GameStorage) -> bool {
        let mut close_modal = false;

        egui::Window::new("Sign Out Game")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // Game

                ui.label("Game");
                
                egui::ComboBox::from_label("")
                    .width(192.0)
                    .selected_text(&self.game)
                    .show_ui(ui, |ui| {
                        for game in game_types.get_all() {
                            ui.selectable_value(&mut self.game, game.game.clone(), game.game.clone());
                        }
                    });
                
                if let Some(error) = &self.game_error {
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
                        let max_quantity = if let Some(game) = game_types.get(&self.game) {
                            // Count quantity of all games already out
                            let already_out = game_records.get_signed_out(&self.game).unwrap();

                            (game.quantity - already_out).max(1)
                        } else {
                            1
                        };
                        self.quantity = self.quantity.clamp(1, max_quantity);
                        self.quantity_str = self.quantity.to_string();
                    }
                });
                
                if let Some(error) = &self.quantity_error {
                    ui.colored_label(egui::Rgba::from_rgb(0.25, 0.0, 0.0), error);
                }

                ui.add_space(4.0);

                // Student Name
                render_modal_text_entry(ui, "Student Name", self.student_name_error.as_ref().map(|s| s.as_str()), &mut self.student_name);

                // Student Number
                render_modal_text_entry(ui, "Student Number", self.student_number_error.as_ref().map(|s| s.as_str()), &mut self.student_number);

                // Student Receptionist
                render_modal_text_entry(ui, "Receptionist", self.receptionist_error.as_ref().map(|s| s.as_str()), &mut self.receptionist);
                
                // Notes
                render_modal_text_entry(ui, "Notes", self.notes_error.as_ref().map(|s| s.as_str()), &mut self.notes);

                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Sign Out").clicked() {
                        let mut error = false;
                        
                        // Game
                        
                        self.game_error = None;

                        let game = self.game.as_str();

                        if game.len() == 0 {
                            self.game_error = Some("Required".into());
                            error = true;
                        }
                        
                        if game.len() > NAME_MAX_LENGTH {
                            self.game_error = Some(format!("Name too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

                        // Quantity
                        
                        self.quantity_error = None;
                        
                        if let Some(game) = game_types.get(game) {
                            // Count quantity of all games already out
                            let already_out = game_records.get_signed_out(&self.game).unwrap();

                            if game.quantity - already_out <= 0{
                                self.quantity_error = Some(format!("Not enough of this game in reception."));
                                error = true;
                            }
                        }

                        // Student Name
                        
                        self.student_name_error = None;

                        let student_name = self.student_name.trim();

                        if student_name.len() == 0 {
                            self.student_name_error = Some("Required".into());
                            error = true;
                        }
                        
                        if student_name.len() > NAME_MAX_LENGTH {
                            self.student_name_error = Some(format!("Name too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

                        // Student Number
                        
                        self.student_number_error = None;

                        let student_number = self.student_number.trim();

                        self.student_number_error = filter_student_number(student_number);

                        if self.student_number_error.is_some() {
                            error = true;
                        }

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
                            game_records.add(GameRecord {
                                id: 0,
                                game: self.game.clone(),
                                quantity: self.quantity,
                                student_name: self.student_name.clone(),
                                student_number: self.student_number.to_uppercase(),
                                receptionist: self.receptionist.clone(),
                                time_out: chrono::Utc::now(),
                                time_in: None,
                                notes: self.notes.clone(),
                            }).unwrap();
                            
                            close_modal = true;
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
                render_modal_text_entry(ui, "Student Name", self.student_name_error.as_ref().map(|s| s.as_str()), &mut self.student_name);

                // Student Number
                render_modal_text_entry(ui, "Student Number", self.student_number_error.as_ref().map(|s| s.as_str()), &mut self.student_number);
                
                // Student Receptionist
                render_modal_text_entry(ui, "Receptionist", self.receptionist_error.as_ref().map(|s| s.as_str()), &mut self.receptionist);
                
                // Notes
                render_modal_text_entry(ui, "Notes", self.notes_error.as_ref().map(|s| s.as_str()), &mut self.notes);
                
                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Sign Out").clicked() {
                        let mut error = false;
                        
                        // Item
                        
                        self.item_error = None;

                        let item = self.item.as_str();

                        if item.len() == 0 {
                            self.item_error = Some("Required".into());
                            error = true;
                        }

                        // Student Name
                        
                        self.student_name_error = None;

                        let student_name = self.student_name.trim();

                        if student_name.len() == 0 {
                            self.student_name_error = Some("Required".into());
                            error = true;
                        }
                        
                        if student_name.len() > NAME_MAX_LENGTH {
                            self.student_name_error = Some(format!("Name too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

                        // Student Number
                        
                        self.student_number_error = None;

                        let student_number = self.student_number.trim();

                        self.student_number_error = filter_student_number(student_number);

                        if self.student_number_error.is_some() {
                            error = true;
                        }

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
                            add_record = Some(ItemRecord {
                                id: 0,
                                item: self.item.clone(),
                                quantity: self.quantity,
                                student_name: self.student_name.clone(),
                                student_number: self.student_number.to_uppercase(),
                                receptionist: self.receptionist.clone(),
                                time_out: chrono::Utc::now(),
                                notes: self.notes.clone(),
                            });
                            close_modal = true;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        close_modal = true;
                    }
                });
            });
        
        if let Some(record) = add_record {
            item_records.add(record).unwrap();
        }
        
        return close_modal;
    }
}

#[derive(Debug, Clone, Default)]
pub struct KeyEntryModal {
    pub key: String,
    pub key_error: Option<String>,
}

impl KeyEntryModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context, key_types: &mut KeyTypeStorage) -> bool {
        let mut close_modal = false;
        let mut delete_key = None;

        egui::Window::new("Keys")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // Key Name

                TableBuilder::new(ui)
                    .max_scroll_height(f32::INFINITY)
                    .column(Column::remainder())
                    .body(|mut body| {
                        for key in key_types.get_all() {
                            body.row(24.0, |mut row| {
                                row.col(|ui| {
                                    ui.horizontal(|ui| {

                                        if ui.add(egui::Button::new("−").small().fill(egui::Rgba::from_rgb(0.25, 0.0, 0.0))).clicked() {
                                            delete_key = Some(key.clone());
                                        }
                                        
                                        ui.add_space(8.0);
                                        ui.label(key);
                                    });
                                });
                            })
                        }
                    });

                if key_types.get_all().len() > 0 {
                    ui.separator();
                }

                // Key Name
                render_modal_text_entry(ui, "Key Name", self.key_error.as_ref().map(|s| s.as_str()), &mut self.key);

                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Add Key").clicked() {
                        let mut error = false;
                        
                        // Key
                        
                        self.key_error = None;

                        let key = self.key.as_str();

                        if key.len() == 0 {
                            self.key_error = Some("Required".into());
                            error = true;
                        }

                        if key.len() > NAME_MAX_LENGTH {
                            self.key_error = Some(format!("Name too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

                        // not particularly efficient, but unlikely to be an issue
                        if let Some(_) = key_types.get_all().iter().find(|k| k.as_str() == key) {
                            self.key_error = Some("A key with this name already exists.".into());
                            error = true;
                        }

                        // Entry valid, add record
                        if !error {
                            key_types.add(self.key.clone()).unwrap();
                            
                            self.key.clear();
                        }
                    }
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });

        if let Some(key) = delete_key {
            key_types.delete(&key).unwrap();
        }

        return close_modal;
    }
}

#[derive(Debug, Clone)]
pub struct GameEntryModal {
    pub game: String,
    pub game_error: Option<String>,
    pub quantity: i64,
    pub quantity_str: String,
    pub quantity_error: Option<String>,
}

impl Default for GameEntryModal {
    fn default() -> Self {
        Self {
            game: Default::default(),
            game_error: Default::default(),
            quantity: 1,
            quantity_str: "1".into(),
            quantity_error: Default::default()
        }
    }
}

impl GameEntryModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context, game_types: &mut GameTypeStorage) -> bool {
        let mut close_modal = false;
        let mut delete_game = None;

        egui::Window::new("Games")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // Key Name

                TableBuilder::new(ui)
                    .max_scroll_height(f32::INFINITY)
                    .column(Column::remainder())
                    .body(|mut body| {
                        let mut update_quantity = None;

                        for game in game_types.get_all() {
                            body.row(24.0, |mut row| {
                                row.col(|ui| {
                                    ui.horizontal(|ui| {

                                        if ui.add(egui::Button::new("−").small().fill(egui::Rgba::from_rgb(0.25, 0.0, 0.0))).clicked() {
                                            delete_game = Some(game.game.clone());
                                        }
                                        
                                        ui.add_space(8.0);
                                        ui.label(&format!("{} × {}", game.quantity, game.game));

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                            if ui.button("+").clicked() {
                                                update_quantity = Some((game.game.clone(), game.quantity + 1));
                                            }
                                            if ui.button("−").clicked() {
                                                update_quantity = Some((game.game.clone(), game.quantity - 1));
                                            }
                                            ui.add_space(8.0);
                                        });
                                    });
                                });
                            })
                        }

                        if let Some((game, quantity)) = update_quantity {
                            let quantity = quantity.clamp(1, MAX_QUANTITY);
                            game_types.update_quantity(&game, quantity).unwrap();
                        }
                    });

                if game_types.get_all().len() > 0 {
                    ui.separator();
                }

                // Game Name
                render_modal_text_entry(ui, "Game Name", self.game_error.as_ref().map(|s| s.as_str()), &mut self.game);

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
                
                if let Some(error) = &self.quantity_error {
                    ui.colored_label(egui::Rgba::from_rgb(0.25, 0.0, 0.0), error);
                }

                ui.add_space(8.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Add Game").clicked() {
                        let mut error = false;
                        
                        // Game Name
                        
                        self.game_error = None;

                        let game = self.game.as_str();

                        if game.len() == 0 {
                            self.game_error = Some("Required".into());
                            error = true;
                        }

                        if game.len() > NAME_MAX_LENGTH {
                            self.game_error = Some(format!("Name too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

                        // not particularly efficient, but unlikely to be an issue
                        if let Some(_) = game_types.get_all().iter().find(|g| g.game == game) {
                            self.game_error = Some("A game with this name already exists.".into());
                            error = true;
                        }

                        // Entry valid, add record
                        if !error {
                            game_types.add(GameTypeRecord {
                                game: self.game.clone(),
                                quantity: self.quantity,
                            }).unwrap();
                            
                            self.game.clear();
                            self.quantity = 1;
                            self.quantity_str = self.quantity.to_string();
                        }
                    }
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });

        if let Some(game) = delete_game {
            println!("Deleting game");
            game_types.delete(&game).unwrap();
        }

        return close_modal;
    }
}

#[derive(Debug, Clone)]
pub struct ItemEntryModal {
    pub item: String,
    pub item_error: Option<String>,
}

impl Default for ItemEntryModal {
    fn default() -> Self {
        Self {
            item: Default::default(),
            item_error: Default::default(),
        }
    }
}

impl ItemEntryModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context, item_types: &mut ItemTypeStorage) -> bool {
        let mut close_modal = false;
        let mut delete_item = None;

        egui::Window::new("Items")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // Item Name

                TableBuilder::new(ui)
                    .max_scroll_height(f32::INFINITY)
                    .column(Column::remainder())
                    .body(|mut body| {
                        for item in item_types.get_all() {
                            body.row(24.0, |mut row| {
                                row.col(|ui| {
                                    ui.horizontal(|ui| {

                                        if ui.add(egui::Button::new("−").small().fill(egui::Rgba::from_rgb(0.25, 0.0, 0.0))).clicked() {
                                            delete_item = Some(item.clone());
                                        }
                                        
                                        ui.add_space(8.0);
                                        ui.label(item);
                                    });
                                });
                            })
                        }
                    });

                if item_types.get_all().len() > 0 {
                    ui.separator();
                }

                // Item Name
                render_modal_text_entry(ui, "Item Name", self.item_error.as_ref().map(|s| s.as_str()), &mut self.item);

                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Add Item").clicked() {
                        let mut error = false;
                        
                        // Item
                        
                        self.item_error = None;

                        let item = self.item.as_str();

                        if item.len() == 0 {
                            self.item_error = Some("Required".into());
                            error = true;
                        }

                        if item.len() > NAME_MAX_LENGTH {
                            self.item_error = Some(format!("Name too long. (> {NAME_MAX_LENGTH} characters)"));
                            error = true;
                        }

                        // not particularly efficient, but unlikely to be an issue
                        if let Some(_) = item_types.get_all().iter().find(|k| k.as_str() == item) {
                            self.item_error = Some("An item with this name already exists.".into());
                            error = true;
                        }

                        // Entry valid, add record
                        if !error {
                            item_types.add(self.item.clone()).unwrap();
                            
                            self.item.clear();
                        }
                    }
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });

        if let Some(item) = delete_item {
            item_types.delete(&item).unwrap();
        }

        return close_modal;
    }
}

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

#[derive(Debug, Default)]
pub struct ExportModal {
    pub record_type: RecordType,
    pub export_handle: Option<JoinHandle<(RecordType, Option<PathBuf>)>>,
}

impl ExportModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context) -> bool {
        let mut close_modal = false;
        
        egui::Window::new("Export Records")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                for record_type in RecordType::iter() {
                    ui.radio_value(&mut self.record_type, record_type, format!("{} Records", record_type));
                }

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Export").clicked() {
                        let record_type = self.record_type;
                        
                        self.export_handle = Some(std::thread::spawn(move || {
                            let record_type_str = record_type.to_string().to_lowercase();
                            
                            let path = rfd::FileDialog::new()
                                .add_filter("CSV File", &["csv"])
                                .set_file_name(&format!("{record_type_str}_records_{}.csv", chrono::Local::now().format(BACKUP_DATE_TIME_FORMAT).to_string()))
                                .save_file();
                            
                            (record_type, path)
                        }));
                        
                        close_modal = true;
                    }
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });
        
        return close_modal;
    }
}

#[derive(Debug, Clone, Default)]
pub struct AboutModal;

impl AboutModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context) -> bool {
        let mut close_modal = false;
        
        egui::Window::new("About Blackcurrant")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("The Blackcurrant Reception Managment System was created for the purpose of managing records of common UCT residence tasks.");
                
                // Note non-breaking space inbetween phone number sections.
                ui.label("It was developed by Luke Davis (2023 UCR/Smuts Hall House Committee Member, Academics, IT & Media Portfolios, 071 302 5271).");

                ui.add_space(4.0);
                
                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });
        
        return close_modal;
    }
}
