use egui_extras::{TableBuilder, Column};

use crate::{records::{Page, GameTypeStorage, GameStorage, PaginatedStorage, NotedStorage}, modals::{GameSignModal, SignInModal}, app::DATE_TIME_FORMAT};

use super::{pagination, render_notes_entry};

#[derive(Debug, Default)]
pub struct GamePanel {
    page: Page,

    game_sign_modal: Option<GameSignModal>,
    game_sign_in_modal: Option<SignInModal<i64>>,
    
    current_notes: Option<(i64, String)>,
}

impl GamePanel {
    pub fn render(&mut self, ctx: &eframe::egui::Context, ui: &mut egui::Ui, game_types: &GameTypeStorage, game_records: &mut GameStorage) {
        ui.horizontal(|ui| {
            // Sign Out Modal Button
            if ui.button("Sign Out Game").clicked() {
                self.game_sign_modal = Some(GameSignModal::default());
            }
            
            // Pagination
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                pagination(ui, &mut self.page, game_records.count());
                game_records.set_page(self.page).expect(&format!("failed to refresh parcel records for page: {:?}", self.page));
            });
        });

        ui.add_space(8.0);

        // Sign Out Modal
        if let Some(modal) = &mut self.game_sign_modal {
            let close_modal = modal.render(ctx, game_types, game_records);

            if close_modal {
                self.game_sign_modal = None;
            }
        }

        // Sign In Modal
        if let Some(modal) = &mut self.game_sign_in_modal {
            let close_modal = modal.render(ctx, game_records);

            if close_modal {
                self.game_sign_in_modal = None;
                
            }
        }
        
        let mut update_notes = None;

        egui::ScrollArea::horizontal().show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .stick_to_bottom(true)
                .max_scroll_height(f32::INFINITY)
                .column(Column::auto().at_least(64.0).at_most(128.0).resizable(true))
                .column(Column::auto().at_least(64.0).at_most(128.0).resizable(true))
                .column(Column::initial(160.0).at_least(64.0).clip(true).resizable(true))
                .column(Column::initial(92.0).at_least(64.0).clip(true).resizable(true))
                .column(Column::auto().at_least(64.0).at_most(128.0).resizable(true))
                .column(Column::initial(92.0).at_least(64.0).clip(true).resizable(true))
                .column(Column::remainder().at_least(64.0).clip(true).resizable(true))
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Time Out").strong());
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Time In").strong());
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Game").strong());
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Student Name").strong());
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Student Number").strong());
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Receptionist").strong());
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Notes").strong());
                        });
                    });
                })
                .body(|mut body| {
                    for record in game_records.get_all() {
                        body.row(30.0, |mut row| {
                            // Time Out
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(record.time_out).format(DATE_TIME_FORMAT).to_string());
                                });
                            });
                            // Time In
                            row.col(|ui| {
                                if let Some(time_in) = record.time_in {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(time_in).format(DATE_TIME_FORMAT).to_string());
                                } else if ui.button("Sign In").clicked() {
                                    self.game_sign_in_modal = Some(SignInModal::new(record.id));
                                }
                            });
                            // Game & Quantity
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&format!("{} × {}", record.quantity, record.game));
                                });
                            });
                            // Student Name
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.student_name);
                                });
                            });
                            // Student Number
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.student_number);
                                });
                            });
                            // Receptionist
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(record.receptionist.as_ref().unwrap_or(&String::new()));
                                });
                            });
                            // Notes
                            row.col(|ui| {
                                let update = render_notes_entry(ui, record.id, &record.notes, &mut self.current_notes);
                                if update_notes.is_none() {
                                    update_notes = update;
                                }
                            });
                        })
                    }
                });
        });

        // Update notes down here to avoid mutating while immutably borrowed
        if let Some((id, notes)) = update_notes {
            game_records.update_notes(id, &notes).expect(&format!("failed to update notes for game record: {id}"));
            log::info!("updated notes for {id} to {notes:?}");
        }
    }
}
