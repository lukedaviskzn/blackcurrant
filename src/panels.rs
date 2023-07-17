use egui_extras::{TableBuilder, Column};

use crate::{modals::{KeySignModal, ParcelSignModal, GameSignModal, ItemSignModal, SignInModal}, records::{KeyTypeStorage, KeyStorage, ParcelStorage, GameStorage, ItemTypeStorage, ItemStorage, GameTypeStorage, TimeUpdateableStorage, PaginatedStorage, Page, NotedStorage}, app::{DATE_TIME_FORMAT, PAGE_SIZE}};

fn pagination(ui: &mut eframe::egui::Ui, page: &mut Page, count: i64) {
    let max_page = (count - 1) / PAGE_SIZE;

    let mut page_num = match page {
        Page::Page(page) => (*page).clamp(0, max_page),
        Page::LastPage => max_page,
    };
    
    if ui.add_enabled(page_num < max_page, egui::Button::new(">>")).clicked() {
        page_num = max_page;
    }
    
    if ui.add_enabled(page_num < max_page, egui::Button::new(">")).clicked() {
        page_num += 1;
    }
    
    ui.label(format!("{}", page_num+1));
    
    if ui.add_enabled(page_num > 0, egui::Button::new("<")).clicked() {
        page_num -= 1;
    }
    
    if ui.add_enabled(page_num > 0, egui::Button::new("<<")).clicked() {
        page_num = 0;
    }

    *page = if page_num >= max_page {
        Page::LastPage
    } else {
        Page::Page(page_num.clamp(0, max_page))
    };
}

#[derive(Debug, Default)]
pub struct KeyPanel {
    page: Page,

    key_sign_modal: Option<KeySignModal>,
    key_sign_in_modal: Option<SignInModal<i64>>,
    
    current_notes_id: Option<i64>,
    current_notes: String,
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
        
        let mut update_notes = false;
        
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
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(record.time_out).format(DATE_TIME_FORMAT).to_string());
                                });
                            });
                            row.col(|ui| {
                                if let Some(time_in) = record.time_in {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(time_in).format(DATE_TIME_FORMAT).to_string());
                                } else if ui.button("Sign In").clicked() {
                                    self.key_sign_in_modal = Some(SignInModal::new(record.id));
                                }
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
                                    ui.label(record.receptionist.as_ref().unwrap_or(&String::new()));
                                });
                            });
                            row.col(|ui| {
                                let response = ui.horizontal(|ui| {
                                    let editing = self.current_notes_id.filter(|id| *id == record.id).is_some();
                                    
                                    if !editing && ui.button(egui::RichText::new("\u{f303}").family(egui::FontFamily::Name("icons".into()))).clicked() {
                                        self.current_notes_id = Some(record.id);
                                        self.current_notes = record.notes.clone();
                                    }

                                    if editing {
                                        if ui.text_edit_singleline(&mut self.current_notes).lost_focus() {
                                            update_notes = true;
                                        }
                                    } else {
                                        ui.label(&record.notes);
                                    }
                                }).response;

                                if let Some(id) = self.current_notes_id {
                                    if id == record.id && response.clicked_elsewhere() {
                                        update_notes = true;
                                    }
                                }
                            });
                        })
                    }
                });
        });

        if update_notes {
            if let Some(id) = self.current_notes_id {
                key_records.update_notes(id, &self.current_notes).expect(&format!("failed to update notes for key record: {id}"));
            }
            self.current_notes_id = None;
        }
    }
}

#[derive(Debug, Default)]
pub struct ParcelPanel {
    page: Page,
    record_confirm: Option<i64>,
    
    parcel_sign_modal: Option<ParcelSignModal>,
    
    current_notes_id: Option<i64>,
    current_notes: String,
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
        let mut update_notes = false;
        
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
                                let response = ui.horizontal(|ui| {
                                    let editing = self.current_notes_id.filter(|id| *id == record.id).is_some();
                                    
                                    if !editing && ui.button(egui::RichText::new("\u{f303}").family(egui::FontFamily::Name("icons".into()))).clicked() {
                                        self.current_notes_id = Some(record.id);
                                        self.current_notes = record.notes.clone();
                                    }

                                    if editing {
                                        if ui.text_edit_singleline(&mut self.current_notes).lost_focus() {
                                            update_notes = true;
                                        }
                                    } else {
                                        ui.label(&record.notes).clicked();
                                    }
                                }).response;

                                if let Some(id) = self.current_notes_id {
                                    if id == record.id && response.clicked_elsewhere() {
                                        update_notes = true;
                                    }
                                }
                            });
                        })
                    }
                });
        });

        if let Some(record_id) = update_record {
            parcel_records.update_time(record_id).expect(&format!("failed to update time for parcel record: {record_id}"));
        }

        if update_notes {
            if let Some(id) = self.current_notes_id {
                parcel_records.update_notes(id, &self.current_notes).expect(&format!("failed to update notes for parcel record: {id}"));
            }
            self.current_notes_id = None;
        }
    }
}

#[derive(Debug, Default)]
pub struct GamePanel {
    page: Page,

    game_sign_modal: Option<GameSignModal>,
    game_sign_in_modal: Option<SignInModal<i64>>,
    
    current_notes_id: Option<i64>,
    current_notes: String,
}

impl GamePanel {
    pub fn render(&mut self, ctx: &eframe::egui::Context, ui: &mut egui::Ui, game_types: &GameTypeStorage, game_records: &mut GameStorage) {
        ui.horizontal(|ui| {
            if ui.button("Sign Out Game").clicked() {
                self.game_sign_modal = Some(GameSignModal::default());
            }
    
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                pagination(ui, &mut self.page, game_records.count());
                game_records.set_page(self.page).expect(&format!("failed to refresh parcel records for page: {:?}", self.page));
            });
        });

        ui.add_space(8.0);

        if let Some(modal) = &mut self.game_sign_modal {
            let close_modal = modal.render(ctx, game_types, game_records);

            if close_modal {
                self.game_sign_modal = None;
            }
        }

        if let Some(modal) = &mut self.game_sign_in_modal {
            let close_modal = modal.render(ctx, game_records);

            if close_modal {
                self.game_sign_in_modal = None;
                
            }
        }
        
        let mut update_notes = false;

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
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(record.time_out).format(DATE_TIME_FORMAT).to_string());
                                });
                            });
                            row.col(|ui| {
                                if let Some(time_in) = record.time_in {
                                    ui.label(&chrono::DateTime::<chrono::Local>::from(time_in).format(DATE_TIME_FORMAT).to_string());
                                } else if ui.button("Sign In").clicked() {
                                    self.game_sign_in_modal = Some(SignInModal::new(record.id));
                                }
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
                                    ui.label(record.receptionist.as_ref().unwrap_or(&String::new()));
                                });
                            });
                            row.col(|ui| {
                                let response = ui.horizontal(|ui| {
                                    let editing = self.current_notes_id.filter(|id| *id == record.id).is_some();
                                    
                                    if !editing && ui.button(egui::RichText::new("\u{f303}").family(egui::FontFamily::Name("icons".into()))).clicked() {
                                        self.current_notes_id = Some(record.id);
                                        self.current_notes = record.notes.clone();
                                    }

                                    if editing {
                                        if ui.text_edit_singleline(&mut self.current_notes).lost_focus() {
                                            update_notes = true;
                                        }
                                    } else {
                                        ui.label(&record.notes).clicked();
                                    }
                                }).response;

                                if let Some(id) = self.current_notes_id {
                                    if id == record.id && response.clicked_elsewhere() {
                                        update_notes = true;
                                    }
                                }
                            });
                        })
                    }
                });
        });

        if update_notes {
            if let Some(id) = self.current_notes_id {
                game_records.update_notes(id, &self.current_notes).expect(&format!("failed to update notes for game record: {id}"));
            }
            self.current_notes_id = None;
        }
    }
}

#[derive(Debug, Default)]
pub struct ItemPanel {
    page: Page,

    item_sign_modal: Option<ItemSignModal>,
    
    current_notes_id: Option<i64>,
    current_notes: String,
}

impl ItemPanel {
    pub fn render(&mut self, ctx: &eframe::egui::Context, ui: &mut egui::Ui, item_types: &ItemTypeStorage, item_records: &mut ItemStorage) {
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
            let close_modal = modal.render(ctx, item_types, item_records);

            if close_modal {
                self.item_sign_modal = None;
            }
        }
        
        let mut update_notes = false;
        
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
                .column(Column::remainder().at_least(64.0).clip(true).resizable(true))
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
                    header.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Notes").strong());
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
                            row.col(|ui| {
                                let response = ui.horizontal(|ui| {
                                    let editing = self.current_notes_id.filter(|id| *id == record.id).is_some();
                                    
                                    if !editing && ui.button(egui::RichText::new("\u{f303}").family(egui::FontFamily::Name("icons".into()))).clicked() {
                                        self.current_notes_id = Some(record.id);
                                        self.current_notes = record.notes.clone();
                                    }

                                    if editing {
                                        if ui.text_edit_singleline(&mut self.current_notes).lost_focus() {
                                            update_notes = true;
                                        }
                                    } else {
                                        ui.label(&record.notes).clicked();
                                    }
                                }).response;

                                if let Some(id) = self.current_notes_id {
                                    if id == record.id && response.clicked_elsewhere() {
                                        update_notes = true;
                                    }
                                }
                            });
                        })
                    }
                });
        });

        if update_notes {
            if let Some(id) = self.current_notes_id {
                item_records.update_notes(id, &self.current_notes).expect(&format!("failed to update notes for item record: {id}"));
            }
            self.current_notes_id = None;
        }
    }
}
