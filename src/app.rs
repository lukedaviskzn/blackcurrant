use std::{rc::Rc, path::PathBuf, thread::JoinHandle};

use crate::{records::{RecordType, KeyTypeStorage, KeyStorage, ParcelStorage, GameStorage, GameTypeStorage, ItemTypeStorage, ItemStorage, PaginatedStorage, StorageError}, modals::{KeySignModal, ParcelSignModal, GameSignModal, AlertModal, KeyEntryModal, ExitModal, GameEntryModal, ItemEntryModal, ItemSignModal, ExportModal, AboutModal}, panels::{KeyPanel, ParcelPanel, GamePanel, ItemPanel}};

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

    key_sign_modal: Option<KeySignModal>,
    parcel_sign_modal: Option<ParcelSignModal>,
    game_sign_modal: Option<GameSignModal>,
    item_sign_modal: Option<ItemSignModal>,
    
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
        let db_dir = dirs::data_dir().unwrap().join("blackcurrant");

        std::fs::create_dir_all(&db_dir).unwrap();

        let db_dir = db_dir.join("db.sqlite");

        let connection = Rc::new(sqlite::open(&db_dir).unwrap());

        let migrations_table_exists = connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='migrations';").unwrap().into_iter().count() > 0;
        
        // Create migrations table
        if !migrations_table_exists {
            connection.execute("CREATE TABLE migrations (
                migration INTEGER PRIMARY KEY
            )").unwrap();
        }

        // NEVER remove an item from this array, unless you are 100% sure you are using an empty DB, rather create another query later which undoes it
        let migrations = vec![
            // Create key type table
            "CREATE TABLE keys (
                key VARCHAR(512) PRIMARY KEY
            )",
            // Create game type table
            "CREATE TABLE games (
                game VARCHAR(512) PRIMARY KEY,
                quantity INTEGER NOT NULL
            )",
            // Create item type table
            "CREATE TABLE items (
                item VARCHAR(512) PRIMARY KEY
            )",
            // Create key record table
            "CREATE TABLE key_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key VARCHAR(512) NOT NULL,
                student_name VARCHAR(512) NOT NULL,
                student_number VARCHAR(9) NOT NULL,
                receptionist VARCHAR(512) NOT NULL,
                time_out VARCHAR(64) NOT NULL,
                time_in VARCHAR(64),
                notes VARCHAT(512) NOT NULL
            )",
            // Create parcel record table
            "CREATE TABLE parcel_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                parcel_desc VARCHAR(512) NOT NULL,
                student_name VARCHAR(512) NOT NULL,
                receptionist VARCHAR(512) NOT NULL,
                time_in VARCHAR(64) NOT NULL,
                time_out VARCHAR(64),
                notes VARCHAT(512) NOT NULL
            )",
            // Create game record table
            "CREATE TABLE game_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game VARCHAR(512) NOT NULL,
                quantity INTEGER NOT NULL,
                student_name VARCHAR(512) NOT NULL,
                student_number VARCHAR(9) NOT NULL,
                receptionist VARCHAR(512) NOT NULL,
                time_out VARCHAR(64) NOT NULL,
                time_in VARCHAR(64),
                notes VARCHAT(512) NOT NULL
            )",
            // Create item record table
            "CREATE TABLE item_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item VARCHAR(512) NOT NULL,
                quantity INTEGER NOT NULL,
                student_name VARCHAR(512) NOT NULL,
                student_number VARCHAR(9) NOT NULL,
                receptionist VARCHAR(512) NOT NULL,
                time_out VARCHAR(64) NOT NULL,
                notes VARCHAT(512) NOT NULL
            )",
        ];

        for (i, migration) in migrations.iter().enumerate() {
            let already_done = {
                let mut stmt = connection.prepare("SELECT migration FROM migrations WHERE migration = ?;").unwrap();
                stmt.bind(&[i as i64][..]).unwrap();
                
                stmt.iter().map(|row| row.unwrap()).count() > 0
            };

            if !already_done {
                // Perform migration
                connection.execute(migration).unwrap();
                
                // Record migration
                let mut stmt = connection.prepare("INSERT INTO migrations (migration) VALUES (?)").unwrap();
                stmt.bind(&[i as i64][..]).unwrap();
                
                while let Ok(sqlite::State::Row) = stmt.next() {}
            }
        }

        Self {
            // connection: Rc::clone(&connection),
            
            current_panel: RecordType::Key,

            file_save_handle: None,
            export_handle: None,
            // file_load_handle: None,

            db_dir,

            key_types: KeyTypeStorage::new(Rc::clone(&connection)).unwrap(),
            game_types: GameTypeStorage::new(Rc::clone(&connection)).unwrap(),
            // item_types: ItemTypeStorage::new(Rc::clone(&connection)).unwrap(),
            item_types: ItemTypeStorage::new(Rc::clone(&connection)).unwrap(),

            key_panel: KeyPanel::default(),
            parcel_panel: ParcelPanel::default(),
            game_panel: GamePanel::default(),
            item_panel: ItemPanel::default(),

            key_records: KeyStorage::new(Rc::clone(&connection)).unwrap(),
            parcel_records: ParcelStorage::new(Rc::clone(&connection)).unwrap(),
            game_records: GameStorage::new(Rc::clone(&connection)).unwrap(),
            item_records: ItemStorage::new(Rc::clone(&connection)).unwrap(),

            key_sign_modal: None,
            parcel_sign_modal: None,
            game_sign_modal: None,
            item_sign_modal: None,
            
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
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let app = Self::default();

        app
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
        if let Some(file_save_handle) = &self.file_save_handle {
            if file_save_handle.is_finished() {
                let handle = self.file_save_handle.take().unwrap();

                if let Some(file_save_path) = handle.join().unwrap() {
                    std::fs::copy(&self.db_dir, file_save_path).unwrap();
                    
                    println!("Saving database backup");
                    
                    self.alert_modal = Some(AlertModal { title: "Backup Successful".into(), description: None });
                }
            }
        }
        
        // Export file location thread running separately.
        if let Some(export_handle) = &self.export_handle {
            if export_handle.is_finished() {
                let handle = self.export_handle.take().unwrap();

                if let (record_type, Some(export_path)) = handle.join().unwrap() {
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
                                    StorageError::PreparedStatementError(_) => Some("An expected error occurred while accessing the local database.".into()),
                                    StorageError::ExportError(_) => Some("Unable to export data to specified file path.".into()),
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
                        self.key_records.refresh().unwrap();
                    }
                    if ui.button("Parcels").clicked() {
                        self.current_panel = RecordType::Parcel;
                        self.parcel_records.refresh().unwrap();
                    }
                    if ui.button("Games").clicked() {
                        self.current_panel = RecordType::Game;
                        self.game_records.refresh().unwrap();
                    }
                    if ui.button("Items").clicked() {
                        self.current_panel = RecordType::Item;
                        self.item_records.refresh().unwrap();
                    }
                });
            });
        
        match self.current_panel {
            RecordType::Key => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        self.key_panel.render(ctx, ui, &mut self.key_sign_modal, &self.key_types, &mut self.key_records);
                    });
            },
            RecordType::Parcel => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        self.parcel_panel.render(ctx, ui, &mut self.parcel_sign_modal, &mut self.parcel_records);
                    });
            },
            RecordType::Game => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        self.game_panel.render(ctx, ui, &mut self.game_sign_modal, &self.game_types, &mut self.game_records);
                    });
            },
            RecordType::Item => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        self.item_panel.render(ctx, ui, &mut self.item_sign_modal, &self.item_types, &mut self.item_records);
                    });
            },
        };
    }
}
