use std::sync::{Arc, Mutex};

use egui_extras::{TableBuilder, Column};

use crate::{records::{Page, ItemTypeStorage, ItemStorage, NotedStorage, PaginatedStorage, StudentInfo}, modal::ItemSignModal, app::{DATE_TIME_FORMAT, ROW_HEIGHT, COL_MAX_WIDTH, COL_LARGE_INITIAL_WIDTH, COL_SMALL_INITIAL_WIDTH, COL_MIN_WIDTH}};

use super::{pagination, render_notes_entry};

#[derive(Debug, Default)]
pub struct ItemPanel {
    page: Page,

    item_sign_modal: Option<ItemSignModal>,
    
    current_notes: Option<(i64, String)>,
}

impl ItemPanel {
    pub fn render(&mut self, ctx: &eframe::egui::Context, ui: &mut egui::Ui, item_types: &ItemTypeStorage, item_records: &mut ItemStorage, student_info: Arc<Mutex<StudentInfo>>) {
        ui.horizontal(|ui| {
            if ui.button("Sign Out Item").clicked() {
                self.item_sign_modal = Some(ItemSignModal::default());
            }
    
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                pagination(ui, &mut self.page, item_records.count());
                item_records.set_page(self.page).expect(&format!("failed to refresh parcel records for page: {:?}", self.page));
            });
        });

        ui.add_space(8.0);

        if let Some(modal) = &mut self.item_sign_modal {
            let close_modal = modal.render(ctx, item_types, item_records, student_info);

            if close_modal {
                self.item_sign_modal = None;
            }
        }
        
        let mut update_notes = None;
        
        egui::ScrollArea::horizontal().show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .stick_to_bottom(true)
                .max_scroll_height(f32::INFINITY)
                .column(Column::auto().at_most(COL_MAX_WIDTH).resizable(true))
                .column(Column::initial(COL_LARGE_INITIAL_WIDTH).at_least(COL_MAX_WIDTH).clip(true).resizable(true))
                .column(Column::initial(COL_SMALL_INITIAL_WIDTH).at_least(COL_MAX_WIDTH).clip(true).resizable(true))
                .column(Column::auto().at_least(COL_MIN_WIDTH).at_most(COL_MAX_WIDTH).resizable(true))
                .column(Column::initial(COL_SMALL_INITIAL_WIDTH).at_least(COL_MIN_WIDTH).clip(true).resizable(true))
                .column(Column::remainder().at_least(COL_MIN_WIDTH).clip(true).resizable(true))
                .header(ROW_HEIGHT, |mut header| {
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Time Out").strong());
                        });
                    });
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Item").strong());
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
                    for record in item_records.get_all() {
                        body.row(ROW_HEIGHT, |mut row| {
                            // Time Out
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(record.time_out).format(DATE_TIME_FORMAT).to_string());
                                });
                            });
                            // Item & Quantity
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&format!("{} × {}", record.quantity, record.item));
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
                                    ui.label(&record.receptionist);
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
            item_records.update_notes(id, &notes).expect(&format!("failed to update notes for item record: {id}"));
            log::info!("updated notes for {id} to {notes:?}");
        }
    }
}
