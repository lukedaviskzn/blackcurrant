use std::rc::Rc;

use crate::{records::{Panel, KeyTypeStorage, KeyStorage, ParcelStorage, GameStorage, RecordStorage, GameTypeStorage, ItemTypeStorage, ItemStorage}, modals::{KeySignModal, ParcelSignModal, GameSignModal, AlertModal, KeyEntryModal, ExitModal, GameEntryModal, ItemEntryModal, ItemSignModal}, panels::{KeyPanel, ParcelPanel, GamePanel, ItemPanel}};

pub const NAME_MAX_LENGTH: usize = 256;
pub const STUDENT_NUMBER_LENGTH: usize = 9;
pub const STAFF_NUMBER_LENGTH: usize = 8;
pub const MAX_QUANTITY: i64 = 99;
pub const DATE_TIME_FORMAT: &str = "%d/%m/%y %H:%M";
pub const BACKUP_DATE_TIME_FORMAT: &str = "%Y-%m-%d_%H-%M-%S.%f";

pub struct App {
    current_panel: Panel,

    db_dir: std::path::PathBuf,

    file_save_handle: Option<std::thread::JoinHandle<Option<String>>>,
    // file_load_handle: Option<std::thread::JoinHandle<Option<String>>>,
    
    file_save_path: Option<String>,
    // file_load_path: Option<String>,

    key_types: KeyTypeStorage,
    game_types: GameTypeStorage,
    item_types: ItemTypeStorage,
    
    key_records: KeyStorage,
    parcel_records: ParcelStorage,
    game_records: GameStorage,
    item_records: ItemStorage,

    key_sign_modal: Option<KeySignModal>,
    parcel_sign_modal: Option<ParcelSignModal>,
    game_sign_modal: Option<GameSignModal>,
    item_sign_modal: Option<ItemSignModal>,
    
    key_entry_modal: Option<KeyEntryModal>,
    game_entry_modal: Option<GameEntryModal>,
    item_entry_modal: Option<ItemEntryModal>,

    alert_modal: Option<AlertModal>,
    exit_modal: Option<ExitModal>,
}

impl Default for App {
    fn default() -> Self {
        let db_dir = dirs::data_dir().unwrap().join("blackcurrant");

        std::fs::create_dir_all(&db_dir).unwrap();

        let db_dir = db_dir.join("db.sqlite");

        let connection = Rc::new(sqlite::open(&db_dir).unwrap());
        
        let key_table_exists = connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='keys';").unwrap().into_iter().count() > 0;
        let game_table_exists = connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='games';").unwrap().into_iter().count() > 0;
        let item_table_exists = connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='items';").unwrap().into_iter().count() > 0;
        let key_record_table_exists = connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='key_records';").unwrap().into_iter().count() > 0;
        let parcel_record_table_exists = connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='parcel_records';").unwrap().into_iter().count() > 0;
        let game_record_table_exists = connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='game_records';").unwrap().into_iter().count() > 0;
        let item_record_table_exists = connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='item_records';").unwrap().into_iter().count() > 0;

        if !key_table_exists {
            connection.execute("CREATE TABLE keys (
                key VARCHAR(512) PRIMARY KEY
            )").unwrap();
        }

        if !game_table_exists {
            connection.execute("CREATE TABLE games (
                game VARCHAR(512) PRIMARY KEY,
                quantity INTEGER NOT NULL
            )").unwrap();
        }

        if !item_table_exists {
            connection.execute("CREATE TABLE items (
                item VARCHAR(512) PRIMARY KEY
            )").unwrap();
        }

        if !key_record_table_exists {
            connection.execute("CREATE TABLE key_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key VARCHAR(512) NOT NULL,
                student_name VARCHAR(512) NOT NULL,
                student_number VARCHAR(9) NOT NULL,
                receptionist VARCHAR(512) NOT NULL,
                time_out VARCHAR(64) NOT NULL,
                time_in VARCHAR(64)
            )").unwrap();
        }

        if !parcel_record_table_exists {
            connection.execute("CREATE TABLE parcel_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                parcel_desc VARCHAR(512) NOT NULL,
                student_name VARCHAR(512) NOT NULL,
                receptionist VARCHAR(512) NOT NULL,
                time_in VARCHAR(64) NOT NULL
            )").unwrap();
        }

        if !game_record_table_exists {
            connection.execute("CREATE TABLE game_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game VARCHAR(512) NOT NULL,
                quantity INTEGER NOT NULL,
                student_name VARCHAR(512) NOT NULL,
                student_number VARCHAR(9) NOT NULL,
                receptionist VARCHAR(512) NOT NULL,
                time_out VARCHAR(64) NOT NULL,
                time_in VARCHAR(64)
            )").unwrap();
        }

        if !item_record_table_exists {
            connection.execute("CREATE TABLE item_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item VARCHAR(512) NOT NULL,
                quantity INTEGER NOT NULL,
                student_name VARCHAR(512) NOT NULL,
                student_number VARCHAR(9) NOT NULL,
                receptionist VARCHAR(512) NOT NULL,
                time_out VARCHAR(64) NOT NULL
            )").unwrap();
        }

        Self {
            // connection: Rc::clone(&connection),
            
            current_panel: Panel::KeyPanel,

            file_save_handle: None,
            // file_load_handle: None,
            
            file_save_path: None,
            // file_load_path: None,

            db_dir,

            key_types: KeyTypeStorage::new(Rc::clone(&connection)).unwrap(),
            game_types: GameTypeStorage::new(Rc::clone(&connection)).unwrap(),
            // item_types: ItemTypeStorage::new(Rc::clone(&connection)).unwrap(),
            item_types: ItemTypeStorage::new(Rc::clone(&connection)).unwrap(),

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
                self.file_save_path = handle.join().unwrap();
            }
        }

        if let Some(file_save_path) = &self.file_save_path {
            std::fs::copy(&self.db_dir, file_save_path).unwrap();
            
            println!("Saving database backup");
            
            self.file_save_path = None;

            self.alert_modal = Some(AlertModal { title: "Backup Successful".into(), description: None });
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
                                    .save_file().map(|path| path.display().to_string())
                            }));
                            ui.close_menu();
                        }
                        // if ui.button("Load Backup").clicked() {
                        //     self.file_load_handle = Some(std::thread::spawn(|| {
                        //         rfd::FileDialog::new().pick_file().map(|path| path.display().to_string())
                        //     }));
                        // }
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
                        if ui.button("Quit").clicked() {
                            self.exit_modal = Some(ExitModal::default());
                            ui.close_menu();
                        }
                    });
                });

                ui.separator();
    
                ui.heading("Blackcurrant");

                ui.separator();

                ui.vertical_centered_justified(|ui| {
                    if ui.button("Keys").clicked() {
                        self.current_panel = Panel::KeyPanel;
                        self.key_records.refresh().unwrap();
                    }
                    if ui.button("Parcels").clicked() {
                        self.current_panel = Panel::ParcelPanel;
                        self.parcel_records.refresh().unwrap();
                    }
                    if ui.button("Games").clicked() {
                        self.current_panel = Panel::GamePanel;
                        self.game_records.refresh().unwrap();
                    }
                    if ui.button("Items").clicked() {
                        self.current_panel = Panel::ItemPanel;
                        self.item_records.refresh().unwrap();
                    }
                });
            });
        
        match self.current_panel {
            Panel::KeyPanel => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        KeyPanel::render(ctx, ui, &mut self.key_sign_modal, &self.key_types, &mut self.key_records);
                    });
            },
            Panel::ParcelPanel => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        ParcelPanel::render(ctx, ui, &mut self.parcel_sign_modal, &mut self.parcel_records);
                    });
            },
            Panel::GamePanel => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        GamePanel::render(ctx, ui, &mut self.game_sign_modal, &self.game_types, &mut self.game_records);
                    });
            },
            Panel::ItemPanel => {
                egui::CentralPanel::default()
                    .show(ctx, |ui| {
                        ItemPanel::render(ctx, ui, &mut self.item_sign_modal, &self.item_types, &mut self.item_records);
                    });
            },
        };
    }
}
