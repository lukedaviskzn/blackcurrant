use crate::{records::{GameTypeStorage, GameStorage, Storage, QuantitySignableStorage, GameRecord, AddibleStorage}, app::NAME_MAX_LENGTH};

use super::{render_modal_text_entry, filter_student_number, filter_required, filter_length};

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
    // pub receptionist: String,
    // pub receptionist_error: Option<String>,
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
            // receptionist: Default::default(),
            // receptionist_error: Default::default(),
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
                            let already_out = game_records.get_signed_out(&self.game).expect("failed to get game signed status from database");

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
                render_modal_text_entry(ui, "Student Name", &self.student_name_error, &mut self.student_name);

                // Student Number
                render_modal_text_entry(ui, "Student Number", &self.student_number_error, &mut self.student_number);

                // Student Receptionist
                // render_modal_text_entry(ui, "Receptionist", self.receptionist_error, &mut self.receptionist);
                
                // Notes
                render_modal_text_entry(ui, "Notes", &self.notes_error, &mut self.notes);

                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Sign Out").clicked() {
                        let mut error = false;
                        
                        // Game
                        self.game_error = None;

                        let game = self.game.as_str();

                        error |= filter_required(game, &mut self.game_error);
                        error |= filter_length(game, NAME_MAX_LENGTH, &mut self.game_error);

                        // Quantity
                        self.quantity_error = None;
                        
                        if let Some(game) = game_types.get(game) {
                            // Count quantity of all games already out
                            let already_out = game_records.get_signed_out(&self.game).expect("failed to get game signed status from database");

                            if game.quantity - already_out <= 0{
                                self.quantity_error = Some(format!("Not enough of this game in stock."));
                                error = true;
                            }
                        }

                        // Student Name
                        self.student_name_error = None;

                        let student_name = self.student_name.trim();

                        error |= filter_required(student_name, &mut self.student_number_error);
                        error |= filter_length(student_name, NAME_MAX_LENGTH, &mut self.student_number_error);

                        // Student Number
                        self.student_number_error = None;

                        let student_number = self.student_number.trim();

                        error |= filter_required(student_number, &mut self.student_number_error);
                        error |= filter_student_number(student_number, &mut self.student_number_error);

                        // Notes
                        self.notes_error = None;

                        let notes = self.notes.trim();

                        error |= filter_length(notes, NAME_MAX_LENGTH, &mut self.notes_error);

                        // Entry valid, add record
                        if !error {
                            game_records.add(GameRecord {
                                id: 0,
                                game: self.game.clone(),
                                quantity: self.quantity,
                                student_name: self.student_name.clone(),
                                student_number: self.student_number.to_uppercase(),
                                // receptionist: self.receptionist.clone(),
                                receptionist: None,
                                time_out: chrono::Utc::now(),
                                time_in: None,
                                notes: self.notes.clone(),
                            }).expect("failed to add game record to database");
                            
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
