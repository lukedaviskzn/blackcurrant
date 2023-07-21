use egui_extras::{TableBuilder, Column};

use crate::{records::{ItemTypeStorage, Storage, AddibleStorage, DeletableStorage}, app::NAME_MAX_LENGTH};

use super::{render_modal_text_entry, filter_required, filter_length};

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
                render_modal_text_entry(ui, "Item Name", &self.item_error, &mut self.item, NAME_MAX_LENGTH);

                ui.add_space(4.0);

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Add Item").clicked() {
                        let mut error = false;
                        
                        // Item
                        self.item_error = None;

                        let item = self.item.trim();

                        error |= filter_required(item, &mut self.item_error);
                        error |= filter_length(item, NAME_MAX_LENGTH, &mut self.item_error);

                        // not particularly efficient, but unlikely to be an issue
                        if let Some(_) = item_types.get_all().iter().find(|k| k.as_str() == item) {
                            self.item_error = Some("An item with this name already exists.".into());
                            error = true;
                        }

                        // Entry valid, add record
                        if !error {
                            item_types.add(self.item.clone()).expect("failed to add item type to database");
                            
                            self.item.clear();
                        }
                    }
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });

        if let Some(item) = delete_item {
            item_types.delete(&item).expect("failed to delete item type from database");
            log::debug!("deleted item type");
        }

        return close_modal;
    }
}
