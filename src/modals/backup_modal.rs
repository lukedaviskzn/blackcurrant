use std::{sync::{Arc, Mutex}, str::FromStr, path::PathBuf, thread::JoinHandle};

use libsql_client::{Config, Client, Statement, args, Value, Row};
use tokio::{task::JoinHandle as TokioHandle, sync::mpsc};

use crate::{app::{AppConfig, ROW_HEIGHT, CONFIRMATION_TITLE, RESTORE_CONFIRM_TEXT, BACKUP_DATE_TIME_FORMAT}, records::{KeyStorage, ParcelStorage, GameStorage, KeyTypeStorage, GameTypeStorage, ItemTypeStorage, PaginatedStorage, Storage, ItemStorage}};

use super::ConfirmationModal;

enum ContentReason {
    Restore,
    Save(PathBuf),
}

enum Message {
    ConnectionEstablished,
    ConnectionFailed,
    Status(String),
    Backups(Vec<OnlineBackup>),
    BackupCreated,
    BackupContents(Vec<u8>, ContentReason),
    BackupDeleted,
    ActionFailed,
}

enum Instruction {
    GetBackups,
    CreateBackup(Vec<u8>),
    GetBackup(i64, ContentReason),
    DeleteBackup(i64),
}

struct OnlineBackup {
    id: i64,
    time: chrono::DateTime<chrono::Utc>,
}

// Backup Table Schema
// CREATE TABLE backups (
//     id INTEGER PRIMARY KEY AUTOINCREMENT,
//     client_id TEXT NOT NULL,
//     backup_time TEXT NOT NULL,
//     backup_time TEXT NOT NULL,
//     backup_contents BLOB NOT NULL
// );

impl OnlineBackup {
    fn parse_row(row: Row) -> Option<OnlineBackup> {
        let id = row.values.get(0)?;
        let id = if let Value::Integer { value } = id {
            *value
        } else {
            return None;
        };
        
        let time = row.values.get(1)?;
        let time = if let Value::Text { value } = time {
            chrono::DateTime::<chrono::Utc>::from_str(value).ok()?
        } else {
            return None;
        };
    
        Some(OnlineBackup {
            id,
            time,
        })
    }
}

pub struct BackupModal {
    #[allow(unused)]
    runtime: tokio::runtime::Runtime,
    local_connection: Arc<Mutex<rusqlite::Connection>>,
    connection: Option<TokioHandle<()>>,
    rx: mpsc::UnboundedReceiver<Message>,
    tx: mpsc::UnboundedSender<Instruction>,
    backups: Option<Vec<OnlineBackup>>,
    status: Option<String>,
    
    has_connection: bool,
    refresh: bool,
    loading: bool,

    restore_confirm_modal: Option<(ConfirmationModal, i64)>,
    delete_confirm_modal: Option<(ConfirmationModal, i64)>,
    save_backup: Option<(JoinHandle<Option<PathBuf>>, i64)>,
}

impl BackupModal {
    pub fn new(local_connection: Arc<Mutex<rusqlite::Connection>>, config: &AppConfig) -> Self {
        let url = format!("libsql://{}.turso.io", config.turso_db).as_str().try_into();
    
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

        let mut backup_status = None;

        let (tx, rx) = mpsc::unbounded_channel();
        let (reply_tx, reply_rx) = mpsc::unbounded_channel();

        let handle = if let Ok(url) = url {
            let auth_token = config.auth_token.clone();

            if let Some(auth_token) = auth_token {
                let tx = tx.clone();
                let mut rx = reply_rx;
                let client_id = config.client_id.clone();
    
                Some(runtime.spawn(async move {
                    log::info!("connecting to backup database");
                    tx.send(Message::Status("Connecting to backup server...".into())).unwrap();
    
                    let client = Client::from_config(Config {
                        url,
                        auth_token: Some(auth_token),
                    }).await.ok();
                    
                    let client = if let Some(client) = client {
                        log::info!("connected to backup database");
                        tx.send(Message::ConnectionEstablished).unwrap();
                        client
                    } else {
                        log::error!("failed to connect to backup database");
                        tx.send(Message::ConnectionFailed).unwrap();
                        return;
                    };
                    
                    while let Some(instruction) = rx.recv().await {
                        match instruction {
                            Instruction::GetBackups => {
                                log::debug!("fetching backups");
    
                                let results = client.execute(Statement::with_args(
                                    "SELECT id, backup_time FROM backups WHERE client_id = ? ORDER BY backup_time",
                                    args!(&client_id)
                                )).await;
                                
                                let results = results.map(|rs| {
                                        rs.rows.into_iter().map(|row| OnlineBackup::parse_row(row).expect("invalid row format"))
                                            .collect::<Vec<_>>()
                                    });
                                
                                if let Ok(backups) = results {
                                    log::trace!("fetched backups");

                                    // sleep because otherwise people will not notice that something happened
                                    std::thread::sleep(std::time::Duration::from_millis(500));
    
                                    tx.send(Message::Backups(backups)).unwrap();
                                } else {
                                    log::error!("failed to fetch backups");
                                    
                                    tx.send(Message::ActionFailed).unwrap();
                                    tx.send(Message::Status("Failed to get list of backups.".into())).unwrap();
                                }
                            },
                            Instruction::CreateBackup(bytes) => {
                                log::debug!("creating backup");
                                
                                let result = client.execute(Statement::with_args(
                                    "INSERT INTO backups (id, client_id, backup_time, backup_contents) VALUES (NULL, ?, ?, ?)",
                                    args!(&client_id, chrono::Utc::now().to_rfc3339(), bytes)
                                )).await;
                                
                                if result.is_ok() {
                                    log::trace!("created backup");

                                    // sleep because otherwise people will not notice that something happened
                                    std::thread::sleep(std::time::Duration::from_millis(500));
    
                                    tx.send(Message::BackupCreated).unwrap();
                                } else {
                                    log::error!("failed to create backup");
    
                                    tx.send(Message::ActionFailed).unwrap();
                                    tx.send(Message::Status("Failed to upload backup.".into())).unwrap();
                                }
    
                                // clearing old backups
                                let result = client.execute(Statement::with_args(
                                    "DELETE FROM backups WHERE client_id = ? AND id NOT IN (SELECT id FROM backups WHERE client_id = ? ORDER BY backup_time DESC LIMIT 10)",
                                    args!(&client_id, &client_id)
                                )).await;
    
                                if result.is_err() {
                                    log::error!("failed to delete old backups");
                                }
                            },
                            Instruction::GetBackup(id, reason) => {
                                log::debug!("fetching backup contents");
    
                                let result = client.execute(Statement::with_args(
                                    "SELECT backup_contents FROM backups WHERE id = ?",
                                    args!(id)
                                )).await;
                                
                                let result = if let Ok(result) = result {
                                    Ok(result.rows.into_iter().next().map(|row| if let Value::Blob { value } = row.values.into_iter().next().unwrap() {
                                        value
                                    } else {
                                        unreachable!();
                                    }))
                                } else {
                                    Err(())
                                };
                                
                                if let Ok(contents) = result {
                                    if let Some(contents) = contents {
                                        log::trace!("fetched backup contents");

                                        // sleep because otherwise people will not notice that something happened
                                        std::thread::sleep(std::time::Duration::from_millis(500));

                                        tx.send(Message::BackupContents(contents, reason)).unwrap();
                                    } else {
                                        tx.send(Message::ActionFailed).unwrap();
                                        tx.send(Message::Status("Backup cannot be found.".into())).unwrap();
                                    }
                                } else {
                                    log::error!("failed to fetch backups");
                                    
                                    tx.send(Message::ActionFailed).unwrap();
                                    tx.send(Message::Status("Failed to get backup contents.".into())).unwrap();
                                }
                            },
                            Instruction::DeleteBackup(id) => {
                                log::debug!("deleting backup {id}");
    
                                let results = client.execute(Statement::with_args(
                                    "DELETE FROM backups WHERE id = ?",
                                    args!(id)
                                )).await;
                                
                                if results.is_ok() {
                                    log::trace!("deleted backup {id}");
                                    tx.send(Message::BackupDeleted).unwrap();
                                } else {
                                    log::error!("failed to delete backup {id}");
                                    tx.send(Message::ActionFailed).unwrap();
                                    tx.send(Message::Status("Failed to delete backup.".into())).unwrap();
                                }
                            },
                        }
                    }
    
                    log::info!("backup thread concluded");
                }))
            } else {
                backup_status = Some("No authentication token, cannot use online backup.".into());
                None
            }
        } else {
            log::error!("invalid url, cannot connect to backup database");
            backup_status = Some("No valid backup server specified in config.".into());
            
            None
        };

        Self {
            runtime,
            local_connection,
            connection: handle,
            rx,
            tx: reply_tx,
            status: backup_status,
            backups: None,
            
            has_connection: false,
            refresh: true,
            loading: false,
            
            restore_confirm_modal: None,
            delete_confirm_modal: None,
            save_backup: None,
        }
    }
    
    pub fn render(&mut self, ctx: &eframe::egui::Context, keys: &mut KeyStorage, parcels: &mut ParcelStorage, games: &mut GameStorage, items: &mut ItemStorage, key_types: &mut KeyTypeStorage, game_types: &mut GameTypeStorage, item_types: &mut ItemTypeStorage) -> bool {
        let mut close_modal = false;

        if self.has_connection && self.refresh {
            self.tx.send(Instruction::GetBackups).unwrap();

            self.loading = true;
            self.refresh = false;
        }

        if let Some((handle, _)) = &self.save_backup {
            if handle.is_finished() {
                let (handle, id) = self.save_backup.take().unwrap();
                if let Some(path) = handle.join().unwrap() {
                    self.tx.send(Instruction::GetBackup(id, ContentReason::Save(path))).unwrap();
                    self.loading = true;
                }
            }
        }

        // Restore Confirmation Modal
        if let Some((modal, id)) = &mut self.restore_confirm_modal {
            let close_modal = modal.render(ctx);

            if modal.confirmed {
                self.tx.send(Instruction::GetBackup(*id, ContentReason::Restore)).unwrap();
                self.loading = true;
            }

            if close_modal {
                self.restore_confirm_modal = None;
            }
        }

        // Delete Confirmation Modal
        if let Some((modal, id)) = &mut self.delete_confirm_modal {
            let close_modal = modal.render(ctx);

            if modal.confirmed {
                self.tx.send(Instruction::DeleteBackup(*id)).unwrap();
                self.loading = true;
            }

            if close_modal {
                self.delete_confirm_modal = None;
            }
        }

        while let Ok(message) = self.rx.try_recv() {
            match message {
                Message::ConnectionEstablished => {
                    self.has_connection = true;

                    self.refresh = true;
                    self.loading = true;
                    self.status = None;
                },
                Message::ConnectionFailed => {
                    self.connection = None;
                    self.has_connection = false;
                    
                    self.loading = false;
                    self.status = Some("Failed to connect to backup server.".into())
                },
                Message::Status(result) => self.status = Some(result),
                Message::Backups(backups) => {
                    self.backups = Some(backups);

                    self.loading = false;
                    self.status = None;
                },
                Message::BackupCreated => {
                    self.refresh = true;
                    self.loading = true;
                    
                    self.status = None;
                },
                Message::BackupContents(contents, ContentReason::Restore) => {
                    let tmp = tempfile::NamedTempFile::new().expect("failed to create backup restore temp file");

                    std::fs::write(tmp.path(), contents).expect("failed to write backup contents to temp file");
                    
                    let result = self.local_connection.lock().unwrap()
                        .restore(rusqlite::DatabaseName::Main, tmp.path(), None::<Box<dyn Fn(rusqlite::backup::Progress) -> ()>>);
                    
                    if let Ok(_) = result {
                        // after restore, run migrations
                        crate::embedded::migrations::runner().run(&mut *self.local_connection.lock().unwrap()).expect("failed to run migrations after restore");
                        // refresh everything after restore
                        key_types.refresh().expect("failed to refresh key types");
                        game_types.refresh().expect("failed to refresh game types");
                        item_types.refresh().expect("failed to refresh item types");
                        keys.refresh().expect("failed to refresh key records");
                        parcels.refresh().expect("failed to refresh parcel records");
                        games.refresh().expect("failed to refresh game records");
                        items.refresh().expect("failed to refresh item records");
                        
                        log::info!("backup restored");
                        self.status = Some("Backup restored.".into());
                        self.loading = false;
                    } else {
                        log::error!("backup restore failed");
                        self.status = Some("Failed to restore backup.".into());
                        self.loading = false;
                    }
                },
                Message::BackupContents(contents, ContentReason::Save(path)) => {
                    if let Err(_) = std::fs::write(path, contents) {
                        log::error!("backup save failed");
                        self.status = Some("Failed to save backup.".into());
                        self.loading = false;
                    }
                    self.loading = false;
                },
                Message::BackupDeleted => {
                    self.refresh = true;
                    self.loading = true;

                    self.status = None;
                },
                Message::ActionFailed => self.loading = false,
            }
        }
        
        egui::Window::new("Online Backups")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.add_enabled(self.has_connection && !self.loading, egui::Button::new("Create Backup")).clicked() {
                        let local_conn = self.local_connection.lock().unwrap();
    
                        let tmp = tempfile::NamedTempFile::new().expect("failed to create backup temp file");
                        
                        if let Ok(_) = local_conn.backup(rusqlite::DatabaseName::Main, tmp.path(), None) {
                            if let Ok(bytes) = std::fs::read(tmp.path()) {
                                log::trace!("sending backup to db thread");
                                self.loading = true;
                                self.tx.send(Instruction::CreateBackup(bytes)).unwrap();
                            } else {
                                log::error!("failed to fetch backup for upload");
                                self.status = Some("Failed to fetch backup for upload.".into());
                            }
                        } else {
                            log::error!("failed to create backup for upload");
                            self.status = Some("Failed to create backup.".into());
                        }
                    }
                    
                    // loading
                    egui_extras::StripBuilder::new(ui)
                        .size(egui_extras::Size::exact(230.0))
                        .horizontal(|mut s| {
                            s.cell(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                    if self.loading {
                                        ui.add_space(16.0);
                                        ui.spinner();
                                    }
                                });
                            });
                        });
                });
    
                ui.add_space(4.0);

                if let Some(backups) = &self.backups {
                    egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .max_scroll_height(384.0)
                        .column(egui_extras::Column::remainder().resizable(true))
                        .header(ROW_HEIGHT, |mut header| {
                            header.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Backups").strong());
                                });
                            });
                        })
                        .body(|mut body| {
                            for backup in backups {
                                body.row(ROW_HEIGHT, |mut row| {
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(&chrono::DateTime::<chrono::Local>::from(backup.time).format("%H:%M:%S, %A, %e %B %Y").to_string());
                                            
                                            ui.add_space(8.0);
                                            
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                                // delete
                                                if ui.add_enabled(!self.loading, egui::Button::new(egui::RichText::new("\u{f1f8}").family(egui::FontFamily::Name("icons".into())))).clicked() {
                                                    self.delete_confirm_modal = Some((
                                                        ConfirmationModal::new(CONFIRMATION_TITLE, "This backup will be permanently deleted.".into()),
                                                        backup.id,
                                                    ));
                                                }
                                                // download
                                                if ui.add_enabled(!self.loading, egui::Button::new(egui::RichText::new("\u{f019}").family(egui::FontFamily::Name("icons".into())))).clicked() {
                                                    let time = chrono::DateTime::<chrono::Local>::from(backup.time);
                                                    self.save_backup = Some((std::thread::spawn(move || {
                                                        rfd::FileDialog::new()
                                                            .add_filter("Sqlite DB Backup", &["sqlite"])
                                                            .set_file_name(&format!("backup_{}.sqlite", time.format(BACKUP_DATE_TIME_FORMAT).to_string()))
                                                            .save_file()
                                                    }), backup.id));
                                                }
                                                // restore
                                                if ui.add_enabled(!self.loading, egui::Button::new(egui::RichText::new("\u{f2f9}").family(egui::FontFamily::Name("icons".into())))).clicked() {
                                                    self.restore_confirm_modal = Some((
                                                        ConfirmationModal::new(CONFIRMATION_TITLE, Some(RESTORE_CONFIRM_TEXT)),
                                                        backup.id
                                                    ));
                                                }
                                            });
                                        });
                                    });
                                });
                            }

                            if backups.len() == 0 {
                                body.row(ROW_HEIGHT, |mut row| {
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("None");
                                        });
                                    });
                                });
                            }
                        });
                }

                ui.add_space(4.0);
                
                if let Some(status) = &self.status {
                    ui.label(status);
                    ui.add_space(4.0);
                }
                
                // Buttons
                if ui.button("Close").clicked() {
                    close_modal = true;
                }
            });
        
        return close_modal;
    }
}
