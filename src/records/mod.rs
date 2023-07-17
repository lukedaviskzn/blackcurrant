use std::{fmt::Display, path::PathBuf};

use err_derive::Error;
use strum::EnumIter;

pub mod key_storage;
pub mod parcel_storage;
pub mod game_storage;
pub mod item_storage;
pub mod key_type_storage;
pub mod game_type_storage;
pub mod item_type_storage;

pub use key_storage::*;
pub use parcel_storage::*;
pub use game_storage::*;
pub use item_storage::*;
pub use key_type_storage::*;
pub use game_type_storage::*;
pub use item_type_storage::*;

use crate::app::PAGE_SIZE;

fn set_export_path_extention(mut path: PathBuf) -> PathBuf {
    if let Some(extension) = path.extension() {
        if extension.to_string_lossy().parse::<u64>().is_ok() {
            // If the extention is a number, then the user has not 
            // specified an extension and it's detecting the 
            // fractional part of the seconds as the extension. 
            // In which case, we cannot use the regular set extension 
            // function.
            let mut p = path.into_os_string();
            p.push(".csv");
            path = p.into();
        }
    }
    
    path.set_extension("csv");

    path
}

fn format_optional_time(time: Option<chrono::DateTime<chrono::Utc>>) -> String {
    match time {
        Some(time) => time.to_rfc3339(),
        None => "".into(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum RecordType {
    #[default]
    Key,
    Parcel,
    Game,
    Item,
}

impl Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordType::Key => f.write_str("Key"),
            RecordType::Parcel => f.write_str("Parcel"),
            RecordType::Game => f.write_str("Game"),
            RecordType::Item => f.write_str("Item"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameTypeRecord {
    pub game: String,
    pub quantity: i64,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error(display = "Database operation failed. {}", _0)]
    DbError(#[source] rusqlite::Error),
    #[error(display = "Failed to export database. {}", _0)]
    ExportCsvError(#[source] csv::Error),
    #[error(display = "Failed to export database. {}", _0)]
    ExportIoError(#[source] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    Page(i64),
    #[default]
    LastPage,
}

impl Page {
    fn clamp(self, count: i64) -> Page {
        match self {
            Page::Page(page) => Page::Page(page.clamp(0, (count - 1) / PAGE_SIZE)),
            Page::LastPage => Page::LastPage,
        }
    }

    fn as_i64(&self, count: i64) -> i64 {
        let max_page = (count - 1) / PAGE_SIZE;

        match &self {
            Page::Page(page) => (*page).clamp(0, max_page),
            Page::LastPage => max_page,
        }
    }
}

pub trait Storage<T, I: Copy> {
    fn refresh(&mut self) -> Result<(), StorageError>;
    fn get_all(&self) -> &[T];
    fn get(&self, id: I) -> Option<&T>;
    fn parse_row(row: &rusqlite::Row) -> Result<T, rusqlite::Error>;
}

pub trait PaginatedStorage<T, I: Copy> {
    fn page(&self) -> Page;
    fn set_page(&mut self, page: Page) -> Result<(), StorageError>;
    fn count(&self) -> i64;
    fn refresh(&mut self) -> Result<(), StorageError>;
    fn get_all(&self) -> &[T];
    fn parse_row(row: &rusqlite::Row) -> Result<T, rusqlite::Error>;
}

pub trait AddibleStorage<T, I: Copy> {
    fn add(&mut self, record_info: T) -> Result<(), StorageError>;
}

pub trait DeletableStorage<T, I: Copy> {
    fn delete(&mut self, id: I) -> Result<(), StorageError>;
}

pub trait TimeUpdateableStorage<T, I: Copy> {
    fn update_time(&mut self, id: I) -> Result<(), StorageError>;
}

pub trait TimeReceptionistUpdateableStorage<T, I: Copy> {
    fn update_receptionist_and_time(&mut self, id: I, receptionist: &str) -> Result<(), StorageError>;
}

pub trait NotedStorage<T, I: Copy> {
    fn update_notes(&mut self, id: I, note: &str) -> Result<(), StorageError>;
}

pub trait SignableStorage<T, IT: Copy> {
    fn get_signed_out(&mut self, item_type: IT) -> Result<Option<T>, StorageError>;
}

pub trait QuantitySignableStorage<T: Copy> {
    fn get_signed_out(&mut self, item_type: T) -> Result<i64, StorageError>;
}

pub trait ExportableStorage<T> {
    fn fetch_all(&self) -> Result<Vec<T>, StorageError>;
    fn csv_headers() -> &'static [&'static str];
    fn write_record<W: std::io::Write>(writer: &mut csv::Writer<W>, record: &T) -> Result<(), csv::Error>;
    fn export_csv(&self, path: PathBuf) -> Result<(), StorageError>;
}

fn export_csv<T, S: ExportableStorage<T>>(storage: &S, mut path: PathBuf) -> Result<(), StorageError> {
    path = set_export_path_extention(path);

    let mut writer = csv::Writer::from_path(path)?;

    writer.write_record(S::csv_headers())?;
    
    let records = storage.fetch_all()?;
    
    for record in records {
        S::write_record(&mut writer, &record)?;
    }
    
    writer.flush()?;

    Ok(())
}
