use chrono::Datelike;
use egui_extras::{TableBuilder, Column};

use crate::{records::{KeyStorage, Summary, ParcelStorage, GameStorage, ItemStorage, CountWithin}, app::{ROW_HEIGHT, COL_LARGE_INITIAL_WIDTH, COL_MIN_WIDTH, COL_MAX_WIDTH}};

#[derive(Debug, Clone)]
pub struct SummaryModal {
    month_start: chrono::NaiveDate,
    key_summary: Vec<(String, i64)>,
    parcel_count: i64,
    game_summary: Vec<(String, i64)>,
    item_summary: Vec<(String, i64)>,
    refresh: bool,
}

impl Default for SummaryModal {
    fn default() -> Self {
        let now = chrono::Local::now().date_naive();
        let month_start = now - chrono::Days::new(now.day0() as u64) - chrono::Months::new(1);

        Self {
            month_start,
            key_summary: vec![],
            parcel_count: 0,
            game_summary: vec![],
            item_summary: vec![],
            refresh: true,
        }
    }
}

impl SummaryModal {
    fn get_summaries(&mut self, keys: &KeyStorage, parcels: &ParcelStorage, games: &GameStorage, items: &ItemStorage) {
        if !self.refresh {
            return;
        }

        self.refresh = false;

        let start = self.month_start.and_time(chrono::NaiveTime::MIN)
            // convert to 00:00 in local time zone
            .and_local_timezone(chrono::Local).single()
            // convert to UTC, since db in UTC
            .map(|date| date.with_timezone(&chrono::Utc))
            // if datetime does not exist (due to when countries change their timezone), default to UTC
            .unwrap_or(
                self.month_start.and_time(chrono::NaiveTime::MIN).and_local_timezone(chrono::Utc).unwrap()
            );
        let end = self.month_start + chrono::Months::new(1);
        let end = end.and_time(chrono::NaiveTime::MIN)
            // convert to 23:59:99.99 in local time zone
            .and_local_timezone(chrono::Local).single()
            // convert to UTC, since db in UTC
            .map(|date| date.with_timezone(&chrono::Utc))
            // if datetime does not exist (due to when countries change their timezone), default to UTC
            .unwrap_or(
                end.and_time(chrono::NaiveTime::MIN).and_local_timezone(chrono::Utc).unwrap()
            );
                
        self.key_summary = keys.summary(start, end).expect("failed to fetch key summary info from database");
        self.parcel_count = parcels.count_within(start, end).expect("failed to fetch parcel summary info from database");
        self.game_summary = games.summary(start, end).expect("failed to fetch game summary info from database");
        self.item_summary = items.summary(start, end).expect("failed to fetch item summary info from database");
    }
}

impl SummaryModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context, keys: &KeyStorage, parcels: &ParcelStorage, games: &GameStorage, items: &ItemStorage) -> bool {
        let mut close_modal = false;

        self.get_summaries(keys, parcels, games, items);

        egui::Window::new("Summary")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    egui_extras::StripBuilder::new(ui)
                        .size(egui_extras::Size::exact(14.0))
                        .size(egui_extras::Size::initial(96.0))
                        .size(egui_extras::Size::exact(14.0))
                        .horizontal(|mut s| {
                            s.cell(|ui| {
                                if ui.button("<").clicked() {
                                    self.month_start = self.month_start - chrono::Months::new(1);
                                    self.refresh = true;
                                }
                            });
                            s.cell(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(2.0);
                                    ui.label(self.month_start.format("%B %Y").to_string());
                                });
                            });
                            s.cell(|ui| {
                                let current_month = chrono::Utc::now().date_naive();
                                let current_month = current_month - chrono::Days::new(current_month.day0() as u64);
                                if ui.add_enabled(self.month_start < current_month, egui::Button::new(">")).clicked() {
                                    self.month_start = self.month_start + chrono::Months::new(1);
                                    self.refresh = true;
                                }
                            });
                        });
                });

                TableBuilder::new(ui)
                    .striped(true)
                    .max_scroll_height(384.0)
                    .column(Column::initial(COL_LARGE_INITIAL_WIDTH).at_least(COL_MIN_WIDTH).clip(true).resizable(true))
                    .column(Column::auto().at_least(COL_MIN_WIDTH).at_most(COL_MAX_WIDTH).resizable(false))
                    .body(|mut body| {
                        let mut total = 0;
                        self.key_summary.iter().for_each(|s| total += s.1);
                        
                        body.row(ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Keys").strong());
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(total.to_string()).strong());
                                });
                            });
                        });

                        for (key, count) in &self.key_summary {
                            body.row(ROW_HEIGHT, |mut row| {
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(key);
                                    });
                                });
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(&count.to_string());
                                    });
                                });
                            });
                        }
                        
                        let mut total = 0;
                        self.game_summary.iter().for_each(|s| total += s.1);

                        body.row(ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Games").strong());
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(total.to_string()).strong());
                                });
                            });
                        });
                        
                        for (game, count) in &self.game_summary {
                            body.row(ROW_HEIGHT, |mut row| {
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(game);
                                    });
                                });
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(&count.to_string());
                                    });
                                });
                            });
                        }

                        let mut total = 0;
                        self.item_summary.iter().for_each(|s| total += s.1);

                        body.row(ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Items").strong());
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(total.to_string()).strong());
                                });
                            });
                        });
                        
                        for (item, count) in &self.item_summary {
                            body.row(ROW_HEIGHT, |mut row| {
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(item);
                                    });
                                });
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(&count.to_string());
                                    });
                                });
                            });
                        }

                        body.row(ROW_HEIGHT, |mut row| {
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Parcels").strong());
                                });
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(self.parcel_count.to_string()).strong());
                                });
                            });
                        });
                    });

                ui.add_space(4.0);
                
                // Buttons
                if ui.button("Close").clicked() {
                    close_modal = true;
                }
            });
        
        return close_modal;
    }
}
