use egui_extras::{TableBuilder, Column};

use crate::{modals::{KeySignModal, ParcelSignModal, GameSignModal, ItemSignModal}, records::{KeyTypeStorage, KeyStorage, RecordStorage, ParcelStorage, GameStorage, ItemTypeStorage, ItemStorage, GameTypeStorage}, app::DATE_TIME_FORMAT};

pub struct KeyPanel;

impl KeyPanel {
    pub fn render(ctx: &eframe::egui::Context, ui: &mut egui::Ui, key_sign_modal: &mut Option<KeySignModal>, key_types: &KeyTypeStorage, key_records: &mut KeyStorage) {
        if ui.button("Sign Out Key").clicked() {
            *key_sign_modal = Some(KeySignModal::default());
        }

        ui.add_space(8.0);

        if let Some(modal) = key_sign_modal {
            let close_modal = modal.render(ctx, key_types, key_records);

            if close_modal {
                *key_sign_modal = None;
            }
        }

        let mut update_record = None;
        
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
                })
                .body(|mut body| {
                    for record in key_records.get_all() {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(record.time_out).format(DATE_TIME_FORMAT).to_string());
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    if let Some(time_in) = record.time_in {
                                        ui.label(&chrono::DateTime::<chrono::Local>::from(time_in).format(DATE_TIME_FORMAT).to_string());
                                    } else if ui.button("Sign In").clicked() {
                                        // record.time_in = Some(chrono::Utc::now());
                                        update_record = Some(record.id);
                                    }
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.key);
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.student_name);
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.student_number);
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.receptionist);
                                });
                            });
                        })
                    }
                });
        });

        if let Some(record_id) = update_record {
            key_records.update_time(record_id).unwrap();
        }
    }
}

pub struct ParcelPanel;

impl ParcelPanel {
    pub fn render(ctx: &eframe::egui::Context, ui: &mut egui::Ui, parcel_sign_modal: &mut Option<ParcelSignModal>, parcel_records: &mut ParcelStorage) {
        if ui.button("Sign In Parcel").clicked() {
            *parcel_sign_modal = Some(ParcelSignModal::default());
        }

        ui.add_space(8.0);

        if let Some(modal) = parcel_sign_modal {
            let close_modal = modal.render(ctx, parcel_records);

            if close_modal {
                *parcel_sign_modal = None;
            }
        }
        
        egui::ScrollArea::horizontal().show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .stick_to_bottom(true)
                .max_scroll_height(f32::INFINITY)
                .column(Column::auto().at_least(64.0).at_most(128.0).resizable(true))
                .column(Column::initial(160.0).at_least(64.0).clip(true).resizable(true))
                .column(Column::initial(92.0).at_least(64.0).clip(true).resizable(true))
                .column(Column::initial(92.0).at_least(64.0).clip(true).resizable(true))
                .header(32.0, |mut header| {
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Time In").strong());
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
                })
                .body(|mut body| {
                    for record in parcel_records.get_all() {
                        body.row(32.0, |mut row| {
                            // time_in
                            // parcel_desc
                            // student_name
                            // receptionist
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(record.time_in).format(DATE_TIME_FORMAT).to_string());
                                });
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
                        })
                    }
                });
        });
    }
}

pub struct GamePanel;

impl GamePanel {
    pub fn render(ctx: &eframe::egui::Context, ui: &mut egui::Ui, game_sign_modal: &mut Option<GameSignModal>, game_types: &GameTypeStorage, game_records: &mut GameStorage) {
        if ui.button("Sign Out Game").clicked() {
            *game_sign_modal = Some(GameSignModal::default());
        }

        ui.add_space(8.0);

        if let Some(modal) = game_sign_modal {
            let close_modal = modal.render(ctx, game_types, game_records);

            if close_modal {
                *game_sign_modal = None;
            }
        }

        let mut update_record = None;
        
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
                })
                .body(|mut body| {
                    for record in game_records.get_all() {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(record.time_out).format(DATE_TIME_FORMAT).to_string());
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    if let Some(time_in) = record.time_in {
                                        ui.label(&chrono::DateTime::<chrono::Local>::from(time_in).format(DATE_TIME_FORMAT).to_string());
                                    } else if ui.button("Sign In").clicked() {
                                        // record.time_in = Some(chrono::Utc::now());
                                        update_record = Some(record.id);
                                    }
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&format!("{} × {}", record.quantity, record.game));
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.student_name);
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.student_number);
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.receptionist);
                                });
                            });
                        })
                    }
                });
        });

        if let Some(record_id) = update_record {
            game_records.update_time(record_id).unwrap();
        }
    }
}

pub struct ItemPanel;

impl ItemPanel {
    pub fn render(ctx: &eframe::egui::Context, ui: &mut egui::Ui, item_sign_modal: &mut Option<ItemSignModal>, item_types: &ItemTypeStorage, item_records: &mut ItemStorage) {
        if ui.button("Sign Out Item").clicked() {
            *item_sign_modal = Some(ItemSignModal::default());
        }

        ui.add_space(8.0);

        if let Some(modal) = item_sign_modal {
            let close_modal = modal.render(ctx, item_types, item_records);

            if close_modal {
                *item_sign_modal = None;
            }
        }
        
        egui::ScrollArea::horizontal().show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .stick_to_bottom(true)
                .max_scroll_height(f32::INFINITY)
                .column(Column::auto().at_least(64.0).at_most(128.0).resizable(true))
                .column(Column::initial(160.0).at_least(64.0).clip(true).resizable(true))
                .column(Column::initial(92.0).at_least(64.0).clip(true).resizable(true))
                .column(Column::auto().at_least(64.0).at_most(128.0).resizable(true))
                .column(Column::initial(92.0).at_least(64.0).clip(true).resizable(true))
                .header(20.0, |mut header| {
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
                })
                .body(|mut body| {
                    for record in item_records.get_all() {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(record.time_out).format(DATE_TIME_FORMAT).to_string());
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&format!("{} × {}", record.quantity, record.item));
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.student_name);
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.student_number);
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&record.receptionist);
                                });
                            });
                        })
                    }
                });
        });
    }
}
