use egui_extras::{TableBuilder, Column};

use crate::{records::{Page, KeyStorage, KeyTypeStorage, PaginatedStorage, NotedStorage}, modals::{KeySignModal, SignInModal}, app::DATE_TIME_FORMAT};

use super::{pagination, render_notes_entry};

#[derive(Debug, Default)]
pub struct KeyPanel {
    page: Page,

    key_sign_modal: Option<KeySignModal>,
    key_sign_in_modal: Option<SignInModal<i64>>,
    
    current_notes: Option<(i64, String)>,
}

impl KeyPanel {
    pub fn render(&mut self, ctx: &eframe::egui::Context, ui: &mut egui::Ui, key_types: &KeyTypeStorage, key_records: &mut KeyStorage) {
        ui.horizontal(|ui| {
            if ui.button("Sign Out Key").clicked() {
                self.key_sign_modal = Some(KeySignModal::default());
            }
    
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                pagination(ui, &mut self.page, key_records.count());
                key_records.set_page(self.page).expect(&format!("failed to refresh key records for page: {:?}", self.page));
            });
        });

        ui.add_space(8.0);

        if let Some(modal) = &mut self.key_sign_modal {
            let close_modal = modal.render(ctx, key_types, key_records);

            if close_modal {
                self.key_sign_modal = None;
            }
        }

        if let Some(modal) = &mut self.key_sign_in_modal {
            let close_modal = modal.render(ctx, key_records);

            if close_modal {
                self.key_sign_in_modal = None;
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
                            ui.label(egui::RichText::new("Key").strong());
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
                    for record in key_records.get_all() {
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
                                    self.key_sign_in_modal = Some(SignInModal::new(record.id));
                                }
                            });
                            // Key
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.key);
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
            key_records.update_notes(id, &notes).expect(&format!("failed to update notes for key record: {id}"));
            log::info!("updated notes for {id} to {notes:?}");
        }
    }
}
