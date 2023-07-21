use egui_extras::{TableBuilder, Column};

use crate::{records::{GameTypeStorage, GameTypeRecord, Storage, AddibleStorage, DeletableStorage}, app::{MAX_QUANTITY, NAME_MAX_LENGTH}};

use super::{render_modal_text_entry, filter_required, filter_length};

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
                            game_types.update_quantity(&game, quantity).expect("failed to update game type quantity");
                        }
                    });

                if game_types.get_all().len() > 0 {
                    ui.separator();
                }

                // Game Name
                render_modal_text_entry(ui, "Game Name", &self.game_error, &mut self.game, NAME_MAX_LENGTH);

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
                        
                        let game = self.game.trim();
                        
                        error |= filter_required(game, &mut self.game_error);
                        error |= filter_length(game, NAME_MAX_LENGTH, &mut self.game_error);

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
                            }).expect("failed to add game type to database");
                            
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
            game_types.delete(&game).expect("failed to delete game type from database");
            log::debug!("deleted game type");
        }

        return close_modal;
    }
}
