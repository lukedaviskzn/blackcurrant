use egui_extras::{TableBuilder, Column};
use tracing::debug;

use crate::{records::{KeyTypeStorage, Storage, AddibleStorage, DeletableStorage}, app::NAME_MAX_LENGTH};

use super::render_modal_text_entry;

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
                render_modal_text_entry(ui, "Key Name", &self.key_error, &mut self.key);

                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Add Key").clicked() {
                        let mut error = false;
                        
                        // Key
                        self.key_error = None;

                        let key = self.key.trim();

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
                            key_types.add(self.key.clone()).expect("failed to add key type to database");
                            
                            self.key.clear();
                        }
                    }
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });

        if let Some(key) = delete_key {
            key_types.delete(&key).expect("failed to delete key type from database");
            debug!("deleted key type");
        }

        return close_modal;
    }
}
