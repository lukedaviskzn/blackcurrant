use std::{path::PathBuf, thread::JoinHandle, sync::{Arc, Mutex}};

use image::EncodableLayout;

use crate::{records::{RecordType, KeyTypeStorage, KeyStorage, ParcelStorage, GameStorage, GameTypeStorage, ItemTypeStorage, ItemStorage, PaginatedStorage, StorageError, ExportableStorage, Storage}, modals::{AlertModal, KeyEntryModal, ExitModal, GameEntryModal, ItemEntryModal, ExportModal, AboutModal, SettingsModal, ConfirmationModal, SummaryModal}, panels::{KeyPanel, ParcelPanel, GamePanel, ItemPanel}};

pub const APP_NAME: &str = "Blackcurrant";

pub const NAME_MAX_LENGTH: usize = 512;
pub const NOTES_MAX_LENGTH: usize = 512;
pub const STUDENT_NUMBER_LENGTH: usize = 9;
pub const STAFF_NUMBER_LENGTH: usize = 8;
pub const MAX_QUANTITY: i64 = 99;
pub const DATE_TIME_FORMAT: &str = "%d/%m/%Y %H:%M";
pub const BACKUP_DATE_TIME_FORMAT: &str = "%Y-%m-%d_%H-%M-%S.%f";
pub const PAGE_SIZE: i64 = 100;
pub const ROW_HEIGHT: f32 = 20.0;
pub const COL_MIN_WIDTH: f32 = 64.0;
pub const COL_MAX_WIDTH: f32 = 128.0;
pub const COL_SMALL_INITIAL_WIDTH: f32 = 92.0;
pub const COL_LARGE_INITIAL_WIDTH: f32 = 160.0;

pub const CONFIRMATION_TITLE: &str = "Are you sure?";
pub const RESTORE_CONFIRM_TEXT: &str = "Restoring from a backup will delete all records which are not present in the backup.";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub facility_name: String,
    pub turso_db: String,
    pub client_id: String,
    pub auth_token: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            facility_name: "".into(),
            turso_db: "".into(),
            client_id: uuid::Uuid::new_v4().as_hyphenated().to_string(),
            auth_token: None,
        }
    }
}

pub struct App {
    current_panel: RecordType,

    connection: Arc<Mutex<rusqlite::Connection>>,

    backup_path_handle: Option<JoinHandle<Option<PathBuf>>>,
    restore_path_handle: Option<JoinHandle<Option<PathBuf>>>,
    export_path_handle: Option<JoinHandle<(RecordType, Option<PathBuf>)>>,
    
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
    settings_modal: Option<SettingsModal>,
    local_restore_confirm_modal: Option<ConfirmationModal>,
    summary_modal: Option<SummaryModal>,

    icon: egui::TextureHandle,
    config: AppConfig,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, icon: image::DynamicImage) -> Self {
        let db_dir = dirs::data_dir().expect("no application data directory").join(APP_NAME);

        std::fs::create_dir_all(&db_dir).expect("failed to create application data directory");

        let db_dir = db_dir.join("db.sqlite");

        let mut connection = rusqlite::Connection::open(&db_dir).expect(&format!("failed to open database file: {db_dir:?}"));

        log::info!("connected to sqlite database");

        crate::embedded::migrations::runner().run(&mut connection).expect("failed to run migrations");

        log::info!("migrations complete");

        let connection = Arc::new(Mutex::new(connection));

        let app = App {
            current_panel: RecordType::Key,

            backup_path_handle: None,
            restore_path_handle: None,
            export_path_handle: None,

            connection: Arc::clone(&connection),

            key_types: KeyTypeStorage::new(Arc::clone(&connection)).expect("failed to initialise key type storage"),
            game_types: GameTypeStorage::new(Arc::clone(&connection)).expect("failed to initialise game type storage"),
            item_types: ItemTypeStorage::new(Arc::clone(&connection)).expect("failed to initialise item type storage"),

            key_panel: KeyPanel::default(),
            parcel_panel: ParcelPanel::default(),
            game_panel: GamePanel::default(),
            item_panel: ItemPanel::default(),

            key_records: KeyStorage::new(Arc::clone(&connection)).expect("failed to initialise key record storage"),
            parcel_records: ParcelStorage::new(Arc::clone(&connection)).expect("failed to initialise parcel record storage"),
            game_records: GameStorage::new(Arc::clone(&connection)).expect("failed to initialise game record storage"),
            item_records: ItemStorage::new(Arc::clone(&connection)).expect("failed to initialise item record storage"),
            
            key_entry_modal: None,
            game_entry_modal: None,
            item_entry_modal: None,

            alert_modal: None,
            exit_modal: None,
            export_modal: None,
            about_modal: None,
            settings_modal: None,
            local_restore_confirm_modal: None,
            summary_modal: None,

            icon: cc.egui_ctx.load_texture(
                "logo",
                egui::ColorImage::from_rgba_unmultiplied(
                    [icon.width() as usize, icon.height() as usize],
                    icon.as_rgba8().expect("invalid icon format").as_bytes(),
                ),
                Default::default(),
            ),
            config: confy::load(APP_NAME, None).unwrap_or_default(),
        };
        
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

        log::info!("fonts loaded");
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
        if let Some(handle) = &self.backup_path_handle {
            if handle.is_finished() {
                let handle = self.backup_path_handle.take().unwrap();

                if let Some(backup_path) = handle.join().expect("backup path thread panicked") {
                    log::info!("saving database backup");
                    
                    match self.connection.lock().unwrap().backup(rusqlite::DatabaseName::Main, backup_path, None) {
                        Ok(_) => {
                            self.alert_modal = Some(AlertModal { title: "Backup Successful".into(), description: None });
                            log::info!("backup successful");
                        },
                        Err(err) => {
                            self.alert_modal = Some(AlertModal {
                                title: "Backup Failed".into(),
                                description: Some(format!("Failed to backup database: {err}")),
                            });
                            log::error!("failed to backup database: {err}");
                        },
                    }
                }
            }
        }

        // Restore file location thread running separately.
        if let Some(handle) = &self.restore_path_handle {
            if handle.is_finished() {
                let handle = self.restore_path_handle.take().unwrap();

                if let Some(restore_path) = handle.join().expect("restore path thread panicked") {
                    log::info!("restoring from database backup");
                    
                    match self.connection.lock().unwrap().restore(rusqlite::DatabaseName::Main, restore_path, None::<Box<dyn Fn(rusqlite::backup::Progress) -> ()>>) {
                        Ok(_) => {

                            self.alert_modal = Some(AlertModal { title: "Restore Successful".into(), description: None });
                            log::info!("restore successful");
                        },
                        Err(err) => {
                            self.alert_modal = Some(AlertModal {
                                title: "Restore Failed".into(),
                                description: Some(format!("Failed to restore database: {err}")),
                            });
                            log::error!("failed to restore database: {err}");
                        },
                    }
                    // after restore, run migrations
                    crate::embedded::migrations::runner().run(&mut *self.connection.lock().unwrap()).expect("failed to run migrations after restore");
                    // refresh everything after restore
                    self.key_types.refresh().expect("failed to refresh key types");
                    self.game_types.refresh().expect("failed to refresh game types");
                    self.item_types.refresh().expect("failed to refresh item types");
                    self.key_records.refresh().expect("failed to refresh key records");
                    self.parcel_records.refresh().expect("failed to refresh parcel records");
                    self.game_records.refresh().expect("failed to refresh game records");
                    self.item_records.refresh().expect("failed to refresh item records");
                }
            }
        }

        // Export file location thread running separately.
        if let Some(handle) = &self.export_path_handle {
            if handle.is_finished() {
                let handle = self.export_path_handle.take().unwrap();

                if let (record_type, Some(export_path)) = handle.join().expect("file export thread panicked") {
                    let result = match record_type {
                        RecordType::Key => self.key_records.export_csv(export_path),
                        RecordType::Parcel => self.parcel_records.export_csv(export_path),
                        RecordType::Game => self.game_records.export_csv(export_path),
                        RecordType::Item => self.item_records.export_csv(export_path),
                    };

                    log::info!("exporting records");

                    match result {
                        Ok(_) => {
                            self.alert_modal = Some(AlertModal { title: "Export Successful".into(), description: None });
                            log::info!("export successful");
                        },
                        Err(err) => {
                            self.alert_modal = Some(AlertModal {
                                title: "Export Failed".into(),
                                description: match &err {
                                    StorageError::DbError(_) => Some("An expected error occurred while accessing the database.".into()),
                                    StorageError::ExportCsvError(err) => Some(format!("Failed to export data: {err}")),
                                    StorageError::ExportIoError(err) => Some(format!("Failed to export data: {err}")),
                                }
                            });
                            log::error!("failed to export: {err}");
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
                self.export_path_handle = modal.path_handle.take();
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

        // Settings Modal
        if let Some(modal) = &mut self.settings_modal {
            let close_modal = modal.render(ctx);

            if close_modal {
                if !modal.cancelled {
                    self.config.facility_name = modal.facility_name.trim().into();
                    
                    match confy::store(APP_NAME, None, &self.config) {
                        Ok(_) => log::info!("updated configuration file"),
                        Err(err) => log::error!("failed to write to configuration file: {err}"),
                    }
                }
                self.settings_modal = None;
            }
        }

        // Restore Confirmation Modal
        if let Some(modal) = &mut self.local_restore_confirm_modal {
            let close_modal = modal.render(ctx);

            if modal.confirmed {
                self.restore_path_handle = Some(std::thread::spawn(|| {
                    rfd::FileDialog::new()
                        .add_filter("Sqlite DB Backup", &["sqlite"])
                        .pick_file()
                }));
            }

            if close_modal {
                self.local_restore_confirm_modal = None;
            }
        }

        // Summary Modal
        if let Some(modal) = &mut self.summary_modal {
            let close_modal = modal.render(ctx, &self.key_records, &self.parcel_records, &self.game_records, &self.item_records);

            if close_modal {
                self.summary_modal = None;
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
                        if ui.button("Export Records").clicked() {
                            self.export_modal = Some(ExportModal::default());
                            ui.close_menu();
                        }
                        if ui.button("Summary").clicked() {
                            self.summary_modal = Some(SummaryModal::default());
                            ui.close_menu();
                        }
                        if ui.button("Settings").clicked() {
                            self.settings_modal = Some(SettingsModal::new(&mut self.config));
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
                    ui.menu_button("Edit", |ui| {
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
                    });
                    ui.menu_button("Backup", |ui| {
                        if ui.button("Save Local Backup").clicked() {
                            self.backup_path_handle = Some(std::thread::spawn(|| {
                                rfd::FileDialog::new()
                                    .add_filter("Sqlite DB Backup", &["sqlite"])
                                    .set_file_name(&format!("backup_{}.sqlite", chrono::Local::now().format(BACKUP_DATE_TIME_FORMAT).to_string()))
                                    .save_file()
                            }));
                            ui.close_menu();
                        }
                        if ui.button("Restore Local Backup").clicked() {
                            self.local_restore_confirm_modal = Some(
                                ConfirmationModal::new(
                                    CONFIRMATION_TITLE,
                                    Some(RESTORE_CONFIRM_TEXT)
                                )
                            );
                            ui.close_menu();
                        }
                    });
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.image(&self.icon, (22.0, 22.0));
                    ui.heading(egui::RichText::new(APP_NAME).color(egui::Color32::WHITE));
                });

                if self.config.facility_name.len() > 0 {
                    ui.label(&self.config.facility_name);
                }

                ui.add_space(4.0);

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
