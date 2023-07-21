use egui_extras::{TableBuilder, Column};

use crate::{records::{Page, ParcelStorage, PaginatedStorage, TimeUpdateableStorage, NotedStorage}, modals::ParcelSignModal, app::DATE_TIME_FORMAT};

use super::{pagination, render_notes_entry};

#[derive(Debug, Default)]
pub struct ParcelPanel {
    page: Page,
    record_confirm: Option<i64>,
    
    parcel_sign_modal: Option<ParcelSignModal>,
    
    current_notes: Option<(i64, String)>,
}

impl ParcelPanel {
    pub fn render(&mut self, ctx: &eframe::egui::Context, ui: &mut egui::Ui, parcel_records: &mut ParcelStorage) {
        ui.horizontal(|ui| {
            if ui.button("Sign In Parcel").clicked() {
                self.parcel_sign_modal = Some(ParcelSignModal::default());
            }
    
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                pagination(ui, &mut self.page, parcel_records.count());
                parcel_records.set_page(self.page).expect(&format!("failed to refresh parcel records for page: {:?}", self.page));
            });
        });

        ui.add_space(8.0);

        if let Some(modal) = &mut self.parcel_sign_modal {
            let close_modal = modal.render(ctx, parcel_records);

            if close_modal {
                self.parcel_sign_modal = None;
            }
        }

        let mut update_record = None;
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
                .column(Column::initial(92.0).at_least(64.0).clip(true).resizable(true))
                .column(Column::remainder().at_least(64.0).clip(true).resizable(true))
                .header(32.0, |mut header| {
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Time In").strong());
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Time Out").strong());
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Parcel Description").strong());
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Student Name").strong());
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
                    for record in parcel_records.get_all() {
                        body.row(32.0, |mut row| {
                            // time_in
                            // time_out
                            // parcel_desc
                            // student_name
                            // receptionist
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(record.time_in).format(DATE_TIME_FORMAT).to_string());
                                });
                            });
                            row.col(|ui| {
                                let require_confirmation = if let Some(record_id) = self.record_confirm {
                                    record_id == record.id
                                } else {
                                    false
                                };
                                
                                let response = ui.horizontal(|ui| {
                                    if let Some(time_out) = record.time_out {
                                        ui.label(&chrono::DateTime::<chrono::Local>::from(time_out).format(DATE_TIME_FORMAT).to_string());
                                    } else if require_confirmation {
                                        if ui.button("Confirm").clicked() {
                                            update_record = Some(record.id);
                                            self.record_confirm = None;
                                        }
                                        if ui.button("Cancel").clicked() {
                                            self.record_confirm = None;
                                        }
                                    } else {
                                        if ui.button("Sign Out").clicked() {
                                            self.record_confirm = Some(record.id);
                                        }
                                    }
                                }).response;

                                if require_confirmation && response.clicked_elsewhere() {
                                    self.record_confirm = None;
                                }
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.parcel_desc);
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.student_name);
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.receptionist);
                                });
                            });
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

        // Update time down here to avoid mutating while immutably borrowed
        if let Some(record_id) = update_record {
            parcel_records.update_time(record_id).expect(&format!("failed to update time for parcel record: {record_id}"));
        }

        // Update notes down here to avoid mutating while immutably borrowed
        if let Some((id, notes)) = update_notes {
            parcel_records.update_notes(id, &notes).expect(&format!("failed to update notes for parcel record: {id}"));
            log::info!("updated notes for {id} to {notes:?}");
        }
    }
}
