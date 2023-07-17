use std::{rc::Rc, path::PathBuf, thread::JoinHandle};

use crate::{records::{RecordType, KeyTypeStorage, KeyStorage, ParcelStorage, GameStorage, GameTypeStorage, ItemTypeStorage, ItemStorage, PaginatedStorage, StorageError, ExportableStorage}, modals::{AlertModal, KeyEntryModal, ExitModal, GameEntryModal, ItemEntryModal, ExportModal, AboutModal}, panels::{KeyPanel, ParcelPanel, GamePanel, ItemPanel}};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub const NAME_MAX_LENGTH: usize = 256;
pub const STUDENT_NUMBER_LENGTH: usize = 9;
pub const STAFF_NUMBER_LENGTH: usize = 8;
pub const MAX_QUANTITY: i64 = 99;
pub const DATE_TIME_FORMAT: &str = "%d/%m/%y %H:%M";
pub const BACKUP_DATE_TIME_FORMAT: &str = "%Y-%m-%d_%H-%M-%S.%f";
pub const PAGE_SIZE: i64 = 100;

pub struct App {
    current_panel: RecordType,

    db_dir: std::path::PathBuf,

    file_save_handle: Option<JoinHandle<Option<PathBuf>>>,
    export_handle: Option<JoinHandle<(RecordType, Option<PathBuf>)>>,
    // file_load_handle: Option<std::thread::JoinHandle<Option<String>>>,
    
    key_types: KeyTypeStorage,
    game_types: GameTypeStorage,
    item_types: ItemTypeStorage,
    
    key_records: KeyStorage,
    parcel_records: ParcelStorage,
    game_records: GameStorage,
    item_records: ItemStorage,
    
    key_panel: KeyPanel,
    parcel_panel: ParcelPanel,
    game_panel: GamePanel,
    item_panel: ItemPanel,
    
    key_entry_modal: Option<KeyEntryModal>,
    game_entry_modal: Option<GameEntryModal>,
    item_entry_modal: Option<ItemEntryModal>,

    alert_modal: Option<AlertModal>,
    exit_modal: Option<ExitModal>,
    export_modal: Option<ExportModal>,
    about_modal: Option<AboutModal>,
}

impl Default for App {
    fn default() -> Self {
        let db_dir = dirs::data_dir().expect("no application data directory").join("Blackcurrant");

        std::fs::create_dir_all(&db_dir).expect("failed to create application data directory");

        let db_dir = db_dir.join("db.sqlite");

        let mut connection = rusqlite::Connection::open(&db_dir).expect(&format!("failed to open database file: {db_dir:?}"));
        embedded::migrations::runner().run(&mut connection).expect("failed to run migrations");

        let connection = Rc::new(connection);

        Self {
            current_panel: RecordType::Key,

            file_save_handle: None,
            export_handle: None,
            // file_load_handle: None,

            db_dir,

            key_types: KeyTypeStorage::new(Rc::clone(&connection)).expect("failed to initialise key type storage"),
            game_types: GameTypeStorage::new(Rc::clone(&connection)).expect("failed to initialise game type storage"),
            item_types: ItemTypeStorage::new(Rc::clone(&connection)).expect("failed to initialise item type storage"),

            key_panel: KeyPanel::default(),
            parcel_panel: ParcelPanel::default(),
            game_panel: GamePanel::default(),
            item_panel: ItemPanel::default(),

            key_records: KeyStorage::new(Rc::clone(&connection)).expect("failed to initialise key record storage"),
            parcel_records: ParcelStorage::new(Rc::clone(&connection)).expect("failed to initialise parcel record storage"),
            game_records: GameStorage::new(Rc::clone(&connection)).expect("failed to initialise game record storage"),
            item_records: ItemStorage::new(Rc::clone(&connection)).expect("failed to initialise item record storage"),
            
            key_entry_modal: None,
            game_entry_modal: None,
            item_entry_modal: None,

            alert_modal: None,
            exit_modal: None,
            export_modal: None,
            about_modal: None,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = Self::default();
        
        App::setup_custom_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(egui::Visuals {
            dark_mode: true,
            ..Default::default()
        });

        app
    }

    fn setup_custom_fonts(ctx: &egui::Context) {
        // Start with the default fonts
        let mut fonts = egui::FontDefinitions::default();
        
        fonts.font_data.insert(
            "fa-solid-900".to_owned(),
            egui::FontData::from_static(include_bytes!("../fonts/fa-solid-900.ttf")),
        );
    
        fonts
            .families
            .entry(egui::FontFamily::Name("icons".into()))
            .or_default()
            .insert(0, "fa-solid-900".to_owned());
    
        ctx.set_fonts(fonts);
    }
    
}

impl eframe::App for App {
    fn on_close_event(&mut self) -> bool {
        if let Some(modal) = &mut self.exit_modal {
            modal.close_process
        } else {
            self.exit_modal = Some(ExitModal::default());
            false
        }
    }

    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        // Backup file location thread running separately.
        if let Some(handle) = &self.file_save_handle {
            if handle.is_finished() {
                let handle = self.file_save_handle.take().expect("unreachable");

                if let Some(file_save_path) = handle.join().expect("file save thread panicked") {
                    println!("Saving database backup");
                    
                    match std::fs::copy(&self.db_dir, file_save_path) {
                        Ok(_) => {
                            self.alert_modal = Some(AlertModal { title: "Backup Successful".into(), description: None });
                        },
                        Err(err) => {
                            self.alert_modal = Some(AlertModal {
                                title: "Backup Failed".into(),
                                description: Some(format!("Failed to backup database: {err}")),
                            });
                        },
                    }
                }
            }
        }
        
        // Export file location thread running separately.
        if let Some(handle) = &self.export_handle {
            if handle.is_finished() {
                let handle = self.export_handle.take().expect("unreachable");

                if let (record_type, Some(export_path)) = handle.join().expect("file export thread panicked") {
                    let result = match record_type {
                        RecordType::Key => self.key_records.export_csv(export_path),
                        RecordType::Parcel => self.parcel_records.export_csv(export_path),
                        RecordType::Game => self.game_records.export_csv(export_path),
                        RecordType::Item => self.item_records.export_csv(export_path),
                    };

                    println!("Exporting records");

                    match result {
                        Ok(_) => {
                            self.alert_modal = Some(AlertModal { title: "Export Successful".into(), description: None });
                        },
                        Err(err) => {
                            self.alert_modal = Some(AlertModal {
                                title: "Export Failed".into(),
                                description: match err {
                                    StorageError::DbError(_) => Some("An expected error occurred while accessing the database.".into()),
                                    StorageError::ExportCsvError(err) => Some(format!("Failed to export data: {err}")),
                                    StorageError::ExportIoError(err) => Some(format!("Failed to export data: {err}")),
                                }
                            });
                        },
                    }
                }
            }
        }

        // Exit Modal
        if let Some(modal) = &mut self.exit_modal {
            let close_modal = modal.render(ctx, frame);

            if close_modal {
                self.exit_modal = None;
            }
        }

        // Alert Modal
        if let Some(modal) = &mut self.alert_modal {
            let close_modal = modal.render(ctx);

            if close_modal {
                self.alert_modal = None;
            }
        }

        // Export Modal
        if let Some(modal) = &mut self.export_modal {
            let close_modal = modal.render(ctx);

            if close_modal {
                self.export_handle = modal.export_handle.take();
                self.export_modal = None;
            }
        }

        // About Modal
        if let Some(modal) = &mut self.about_modal {
            let close_modal = modal.render(ctx);

            if close_modal {
                self.about_modal = None;
            }
        }

        // Key Type Entry Modal
        if let Some(modal) = &mut self.key_entry_modal {
            let close_modal = modal.render(ctx, &mut self.key_types);

            if close_modal {
                self.key_entry_modal = None;
            }
        }

        // Game Type Entry Modal
        if let Some(modal) = &mut self.game_entry_modal {
            let close_modal = modal.render(ctx, &mut self.game_types);

            if close_modal {
                self.game_entry_modal = None;
            }
        }

        // Item Type Entry Modal
        if let Some(modal) = &mut self.item_entry_modal {
            let close_modal = modal.render(ctx, &mut self.item_types);

            if close_modal {
                self.item_entry_modal = None;
            }
        }

        egui::SidePanel::new(egui::panel::Side::Left, egui::Id::new("left_panel"))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Save Backup").clicked() {
                            self.file_save_handle = Some(std::thread::spawn(|| {
                                rfd::FileDialog::new()
                                    .add_filter("Sqlite DB Backup", &["sqlite"])
                                    .set_file_name(&format!("backup_{}.sqlite", chrono::Local::now().format(BACKUP_DATE_TIME_FORMAT).to_string()))
                                    .save_file()
                            }));
                            ui.close_menu();
                        }
                        // if ui.button("Load Backup").clicked() {
                        //     self.file_load_handle = Some(std::thread::spawn(|| {
                        //         rfd::FileDialog::new().pick_file().map(|path| path.display().to_string())
                        //     }));
                        // }
                        if ui.button("Export Records").clicked() {
                            self.export_modal = Some(ExportModal::default());
                            ui.close_menu();
                        }
                        if ui.button("Edit Keys").clicked() {
                            self.key_entry_modal = Some(KeyEntryModal::default());
                            ui.close_menu();
                        }
                        if ui.button("Edit Games").clicked() {
                            self.game_entry_modal = Some(GameEntryModal::default());
                            ui.close_menu();
                        }
                        if ui.button("Edit Items").clicked() {
                            self.item_entry_modal = Some(ItemEntryModal::default());
                            ui.close_menu();
                        }
                        if ui.button("About").clicked() {
                            self.about_modal = Some(AboutModal::default());
                            ui.close_menu();
                        }
                        if ui.button("Quit").clicked() {
                            self.exit_modal = Some(ExitModal::default());
                            ui.close_menu();
                        }
                    });
                });

                ui.separator();
    
                ui.heading("Blackcurrant RMS");

                ui.separator();

                ui.vertical_centered_justified(|ui| {
                    if ui.button("Keys").clicked() {
                        self.current_panel = RecordType::Key;
                        self.key_records.refresh().expect("failed to refresh key records");
                    }
                    if ui.button("Parcels").clicked() {
                        self.current_panel = RecordType::Parcel;
                        self.parcel_records.refresh().expect("failed to refresh key records");
                    }
                    if ui.button("Games").clicked() {
                        self.current_panel = RecordType::Game;
                        self.game_records.refresh().expect("failed to refresh key records");
                    }
                    if ui.button("Items").clicked() {
                        self.current_panel = RecordType::Item;
                        self.item_records.refresh().expect("failed to refresh key records");
                    }
                });
            });
        
        match self.current_panel {
            RecordType::Key => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        self.key_panel.render(ctx, ui, &self.key_types, &mut self.key_records);
                    });
            },
            RecordType::Parcel => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        self.parcel_panel.render(ctx, ui, &mut self.parcel_records);
                    });
            },
            RecordType::Game => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        self.game_panel.render(ctx, ui, &self.game_types, &mut self.game_records);
                    });
            },
            RecordType::Item => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        self.item_panel.render(ctx, ui, &self.item_types, &mut self.item_records);
                    });
            },
        };
    }
}
