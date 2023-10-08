use std::{thread::JoinHandle, path::PathBuf};

use strum::IntoEnumIterator;

use crate::{records::RecordType, app::BACKUP_DATE_TIME_FORMAT};

#[derive(Debug, Default)]
pub struct ExportModal {
    pub record_type: RecordType,
    pub path_handle: Option<JoinHandle<(RecordType, Option<PathBuf>)>>,
}

impl ExportModal {
    pub fn render(&mut self, ctx: &eframe::egui::Context) -> bool {
        let mut close_modal = false;
        
        egui::Window::new("Export Records")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                for record_type in RecordType::iter() {
                    ui.radio_value(&mut self.record_type, record_type, format!("{} Records", record_type));
                }

                // Buttons
                
                ui.horizontal(|ui| {
                    if ui.button("Export").clicked() {
                        let record_type = self.record_type;
                        
                        self.path_handle = Some(std::thread::spawn(move || {
                            let record_type_str = record_type.to_string().to_lowercase();

                            log::info!("opening export save dialogue");
                            
                            let path = rfd::FileDialog::new()
                                .add_filter("CSV File", &["csv"])
                                .set_file_name(&format!("{record_type_str}_records_{}.csv", chrono::Local::now().format(BACKUP_DATE_TIME_FORMAT).to_string()))
                                .save_file();

                                log::info!("finished export save dialogue");
                            
                            (record_type, path)
                        }));
                        
                        close_modal = true;
                    }
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });
        
        return close_modal;
    }
}
