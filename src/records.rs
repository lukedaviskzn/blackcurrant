use std::{rc::Rc, fmt::Display, path::PathBuf};

use err_derive::Error;
use strum::EnumIter;

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

use crate::app::PAGE_SIZE;

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
pub struct KeyRecord {
    pub id: i64,
    pub key: String,
    pub student_name: String,
    pub student_number: String,
    pub receptionist: Option<String>,
    pub time_out: chrono::DateTime<chrono::Utc>,
    pub time_in: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct ParcelRecord {
    pub id: i64,
    pub parcel_desc: String,
    pub student_name: String,
    pub receptionist: String,
    pub time_in: chrono::DateTime<chrono::Utc>,
    pub time_out: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct GameRecord {
    pub id: i64,
    pub game: String,
    pub quantity: i64,
    pub student_name: String,
    pub student_number: String,
    pub receptionist: Option<String>,
    pub time_out: chrono::DateTime<chrono::Utc>,
    pub time_in: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct ItemRecord {
    pub id: i64,
    pub item: String,
    pub quantity: i64,
    pub student_name: String,
    pub student_number: String,
    pub receptionist: String,
    pub time_out: chrono::DateTime<chrono::Utc>,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct GameTypeRecord {
    pub game: String,
    pub quantity: i64,
}

#[derive(Debug, Error)]
pub enum StorageError {
    // #[error(display = "Failed to connect to local database.")]
    // DatabaseConnectionError(sqlite::Error),
    #[error(display = "Failed to access local database.")]
    PreparedStatementError(sqlite::Error),
    #[error(display = "Failed to export database.")]
    ExportError(Box<dyn std::error::Error>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    Page(i64),
    #[default]
    LastPage,
}

pub trait Storage<T, I: Copy> {
    fn refresh(&mut self) -> Result<(), StorageError>;
    fn get_all(&self) -> &[T];
    fn get(&self, id: I) -> Option<&T>;
}

pub trait PaginatedStorage<T, I: Copy> {
    fn page(&self) -> Page;
    fn set_page(&mut self, page: Page) -> Result<(), StorageError>;
    fn count(&self) -> i64;
    fn refresh(&mut self) -> Result<(), StorageError>;
    fn get_all(&self) -> &[T];
    // fn get(&self, id: I) -> Option<&T>;
    fn export_csv(&self, path: PathBuf) -> Result<(), StorageError>;
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

pub struct KeyStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<KeyRecord>,
    page: Page,
    count: i64,
}

impl KeyStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<KeyStorage, StorageError> {
        let mut storage = KeyStorage {
            connection,
            records: vec![],
            page: Page::LastPage,
            count: 0,
        };

        storage.refresh()?;

        Ok(storage)
    }

    fn parse_record(record: &sqlite::Row) -> KeyRecord {
        let time_out = record.read("time_out");
        let time_out = chrono::DateTime::parse_from_rfc3339(time_out).unwrap().with_timezone(&chrono::Utc);
        
        let time_in = record.read::<Option<&str>, _>("time_in");
        let time_in = time_in.map(|time_in| chrono::DateTime::parse_from_rfc3339(time_in).unwrap().with_timezone(&chrono::Utc));
        
        KeyRecord {
            id: record.read("id"),
            key: record.read::<&str, _>("key").into(),
            student_name: record.read::<&str, _>("student_name").into(),
            student_number: record.read::<&str, _>("student_number").into(),
            receptionist: record.read::<Option<&str>, _>("receptionist").map(|s| s.into()),
            time_out,
            time_in,
            notes: record.read::<&str, _>("notes").into(),
        }
    }
}

impl PaginatedStorage<KeyRecord, i64> for KeyStorage {
    fn page(&self) -> Page {
        self.page
    }
    
    fn set_page(&mut self, page: Page) -> Result<(), StorageError> {
        if page != self.page {
            self.page = match page {
                Page::Page(page) => Page::Page(page.clamp(0, self.count / PAGE_SIZE)),
                Page::LastPage => Page::LastPage,
            };
            self.refresh()
        } else {
            Ok(())
        }
    }

    fn count(&self) -> i64 {
        self.count
    }
    
    fn refresh(&mut self) -> Result<(), StorageError> {
        self.count = {
            let count = self.connection.prepare("SELECT COUNT(*) AS c FROM key_records")
                .map_err(|e| StorageError::PreparedStatementError(e))?
                .into_iter().map(|row| row.unwrap()).next().unwrap();
            
            count.read("c")
        };

        let max_page = (self.count - 1) / PAGE_SIZE;

        let page = match self.page {
            Page::Page(page) => {
                let page = page.clamp(0, max_page);
                self.page = Page::Page(page);
                page
            },
            Page::LastPage => max_page,
        };
        
        let records = {
            let mut stmt = self.connection.prepare("SELECT * FROM key_records LIMIT ? OFFSET ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[PAGE_SIZE, page * PAGE_SIZE][..]).map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.into_iter().map(|row| row.unwrap())
        };

        self.records = records.map(|record| Self::parse_record(&record)).collect();

        Ok(())
    }

    fn get_all(&self) -> &[KeyRecord] {
        self.records.as_slice()
    }

    // fn get(&self, id: i64) -> Option<&KeyRecord> {
    //     self.records.iter().find(|r| r.id == id)
    // }

    fn export_csv(&self, mut path: PathBuf) -> Result<(), StorageError> {
        path = set_export_path_extention(path);

        let mut writer = csv::Writer::from_path(path).map_err(|e| StorageError::ExportError(Box::new(e)))?;

        writer.write_record(&[
            "Time Out",
            "Time In",
            "Key",
            "Student Name",
            "Student Number",
            "Receptionist",
            "Notes",
        ]).map_err(|e| StorageError::ExportError(Box::new(e)))?;
        
        let records = {
            let stmt = self.connection.prepare("SELECT * FROM key_records")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.into_iter().map(|row| row.unwrap())
        };

        let records = records.map(|record| Self::parse_record(&record));
        
        for record in records {
            writer.write_record(&[
                record.time_out.to_rfc3339().as_str(),
                &format_optional_time(record.time_in),
                &record.key,
                &record.student_name,
                &record.student_number,
                &record.receptionist.unwrap_or_default(),
                &record.notes,
            ]).map_err(|e| StorageError::ExportError(Box::new(e)))?;
        }
        
        writer.flush().map_err(|e| StorageError::ExportError(Box::new(e)))?;

        Ok(())
    }
}

impl AddibleStorage<KeyRecord, i64> for KeyStorage {
    fn add(&mut self, record: KeyRecord) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO key_records (id, key, student_name, student_number, receptionist, time_out, time_in, notes) VALUES (NULL, ?, ?, ?, NULL, ?, NULL, ?)")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
        
            stmt.bind(&[record.key.as_str(), &record.student_name, &record.student_number, &record.time_out.to_rfc3339(), &record.notes][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl TimeReceptionistUpdateableStorage<KeyRecord, i64> for KeyStorage {
    fn update_receptionist_and_time(&mut self, id: i64, receptionist: &str) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE key_records SET receptionist = ?, time_in = ? WHERE id = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[receptionist, &chrono::Utc::now().to_rfc3339(), &id.to_string()][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl NotedStorage<KeyRecord, i64> for KeyStorage {
    fn update_notes(&mut self, id: i64, notes: &str) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE key_records SET notes = ? WHERE id = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[notes, &id.to_string()][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl SignableStorage<KeyRecord, &str> for KeyStorage {
    fn get_signed_out(&mut self, item_type: &str) -> Result<Option<KeyRecord>, StorageError> {
        let mut stmt = self.connection.prepare("SELECT * FROM key_records WHERE key = ? AND time_in IS NULL")
            .map_err(|e| StorageError::PreparedStatementError(e))?;
        
        stmt.bind(&[item_type][..]).map_err(|e| StorageError::PreparedStatementError(e))?;
        
        let record = stmt.into_iter().map(|row| row.unwrap()).next();

        if let Some(record) = record {
            let time_out = record.read("time_out");
            let time_out = chrono::DateTime::parse_from_rfc3339(time_out).unwrap().with_timezone(&chrono::Utc);
            
            let time_in = record.read::<Option<&str>, _>("time_in");
            let time_in = time_in.map(|time_in| chrono::DateTime::parse_from_rfc3339(time_in).unwrap().with_timezone(&chrono::Utc));
            
            Ok(Some(KeyRecord {
                id: record.read("id"),
                key: record.read::<&str, _>("key").into(),
                student_name: record.read::<&str, _>("student_name").into(),
                student_number: record.read::<&str, _>("student_number").into(),
                receptionist: record.read::<Option<&str>, _>("receptionist").map(|s| s.into()),
                time_out,
                time_in,
                notes: record.read::<&str, _>("notes").into(),
            }))
        } else {
            Ok(None)
        }
    }
}

pub struct ParcelStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<ParcelRecord>,
    page: Page,
    count: i64,
}

impl ParcelStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<ParcelStorage, StorageError> {
        let mut storage = ParcelStorage {
            connection,
            records: vec![],
            page: Page::LastPage,
            count: 0,
        };

        storage.refresh()?;

        Ok(storage)
    }

    fn parse_record(record: &sqlite::Row) -> ParcelRecord {
        let time_in = record.read::<&str, _>("time_in");
        let time_in = chrono::DateTime::parse_from_rfc3339(time_in).unwrap().with_timezone(&chrono::Utc);
        
        let time_out = record.read::<Option<&str>, _>("time_out");
        let time_out = time_out.map(|time_out| chrono::DateTime::parse_from_rfc3339(time_out).unwrap().with_timezone(&chrono::Utc));
        
        ParcelRecord {
            id: record.read("id"),
            parcel_desc: record.read::<&str, _>("parcel_desc").into(),
            student_name: record.read::<&str, _>("student_name").into(),
            receptionist: record.read::<&str, _>("receptionist").into(),
            time_in,
            time_out,
            notes: record.read::<&str, _>("notes").into(),
        }
    }
}

impl PaginatedStorage<ParcelRecord, i64> for ParcelStorage {
    fn page(&self) -> Page {
        self.page
    }
    
    fn set_page(&mut self, page: Page) -> Result<(), StorageError> {
        if page != self.page {
            self.page = match page {
                Page::Page(page) => Page::Page(page.clamp(0, self.count / PAGE_SIZE)),
                Page::LastPage => Page::LastPage,
            };
            self.refresh()
        } else {
            Ok(())
        }
    }

    fn count(&self) -> i64 {
        self.count
    }
    
    fn refresh(&mut self) -> Result<(), StorageError> {
        self.count = {
            let count = self.connection.prepare("SELECT COUNT(*) AS c FROM parcel_records")
                .map_err(|e| StorageError::PreparedStatementError(e))?
                .into_iter().map(|row| row.unwrap()).next().unwrap();
            
            count.read("c")
        };

        let max_page = (self.count - 1) / PAGE_SIZE;

        let page = match self.page {
            Page::Page(page) => {
                let page = page.clamp(0, max_page);
                self.page = Page::Page(page);
                page
            },
            Page::LastPage => max_page,
        };
        
        let records = {
            let mut stmt = self.connection.prepare("SELECT * FROM parcel_records LIMIT ? OFFSET ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[PAGE_SIZE, page * PAGE_SIZE][..]).map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.into_iter().map(|row| row.unwrap())
        };

        self.records = records.map(|record| Self::parse_record(&record)).collect();

        Ok(())
    }

    fn get_all(&self) -> &[ParcelRecord] {
        self.records.as_slice()
    }

    // fn get(&self, id: i64) -> Option<&ParcelRecord> {
    //     self.records.iter().find(|r| r.id == id)
    // }

    fn export_csv(&self, mut path: PathBuf) -> Result<(), StorageError> {
        path = set_export_path_extention(path);

        let mut writer = csv::Writer::from_path(path).map_err(|e| StorageError::ExportError(Box::new(e)))?;

        writer.write_record(&[
            "Time In",
            "Time Out",
            "Parcel Description",
            "Student Name",
            "Receptionist",
            "Notes",
        ]).map_err(|e| StorageError::ExportError(Box::new(e)))?;
        
        let records = {
            let stmt = self.connection.prepare("SELECT * FROM parcel_records")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.into_iter().map(|row| row.unwrap())
        };

        let records = records.map(|record| Self::parse_record(&record));
        
        for record in records {
            writer.write_record(&[
                record.time_in.to_rfc3339().as_str(),
                &format_optional_time(record.time_out),
                &record.parcel_desc,
                &record.student_name,
                &record.receptionist,
                &record.notes,
            ]).map_err(|e| StorageError::ExportError(Box::new(e)))?;
        }
        // RecordType::Game => {
        //     writer.write_record(&[
        //         "Time Out",
        //         "Time In",
        //         "Game",
        //         "Quantity",
        //         "Student Name",
        //         "Student Number",
        //         "Receptionist",
        //         "Notes",
        //     ])?;
            
        //     for record in game_records.get_all() {
        //         writer.write_record(&[
        //             record.time_out.to_rfc3339().as_str(),
        //             &format_time_in(record.time_in),
        //             &record.game,
        //             &record.quantity.to_string(),
        //             &record.student_name,
        //             &record.student_number,
        //             &record.receptionist,
        //             &record.notes,
        //         ])?;
        //     }
        // },
        // RecordType::Item => {
        //     writer.write_record(&[
        //         "Time Out",
        //         "Item",
        //         "Quantity",
        //         "Student Name",
        //         "Student Number",
        //         "Receptionist",
        //         "Notes",
        //     ])?;
            
        //     for record in item_records.get_all() {
        //         writer.write_record(&[
        //             record.time_out.to_rfc3339().as_str(),
        //             &record.item,
        //             &record.quantity.to_string(),
        //             &record.student_name,
        //             &record.student_number,
        //             &record.receptionist,
        //             &record.notes,
        //         ])?;
        //     }
        // },
        
        writer.flush().map_err(|e| StorageError::ExportError(Box::new(e)))?;

        Ok(())
    }
}

impl AddibleStorage<ParcelRecord, i64> for ParcelStorage {
    fn add(&mut self, record: ParcelRecord) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO parcel_records (id, parcel_desc, student_name, receptionist, time_in, time_out, notes) VALUES (NULL, ?, ?, ?, ?, NULL, ?)")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[record.parcel_desc.as_str(), &record.student_name, &record.receptionist, &record.time_in.to_rfc3339(), &record.notes][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl TimeUpdateableStorage<ParcelRecord, i64> for ParcelStorage {
    fn update_time(&mut self, id: i64) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE parcel_records SET time_out = ? WHERE id = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[chrono::Utc::now().to_rfc3339().as_str(), &id.to_string()][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl NotedStorage<ParcelRecord, i64> for ParcelStorage {
    fn update_notes(&mut self, id: i64, notes: &str) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE parcel_records SET notes = ? WHERE id = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[notes, &id.to_string()][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

pub struct GameStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<GameRecord>,
    page: Page,
    count: i64,
}

impl GameStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<GameStorage, StorageError> {
        let mut storage = GameStorage {
            connection,
            records: vec![],
            page: Page::LastPage,
            count: 0,
        };

        storage.refresh()?;

        Ok(storage)
    }

    fn parse_record(record: &sqlite::Row) -> GameRecord {
        let time_out = record.read("time_out");
        let time_out = chrono::DateTime::parse_from_rfc3339(time_out).unwrap().with_timezone(&chrono::Utc);
        
        let time_in = record.read::<Option<&str>, _>("time_in");
        let time_in = time_in.map(|time_in| chrono::DateTime::parse_from_rfc3339(time_in).unwrap().with_timezone(&chrono::Utc));
        
        GameRecord {
            id: record.read("id"),
            game: record.read::<&str, _>("game").into(),
            quantity: record.read("quantity"),
            student_name: record.read::<&str, _>("student_name").into(),
            student_number: record.read::<&str, _>("student_number").into(),
            receptionist: record.read::<Option<&str>, _>("receptionist").map(|s| s.into()),
            time_out,
            time_in,
            notes: record.read::<&str, _>("notes").into(),
        }
    }
}

impl PaginatedStorage<GameRecord, i64> for GameStorage {
    fn page(&self) -> Page {
        self.page
    }
    
    fn set_page(&mut self, page: Page) -> Result<(), StorageError> {
        if page != self.page {
            self.page = match page {
                Page::Page(page) => Page::Page(page.clamp(0, self.count / PAGE_SIZE)),
                Page::LastPage => Page::LastPage,
            };
            self.refresh()
        } else {
            Ok(())
        }
    }

    fn count(&self) -> i64 {
        self.count
    }
    
    fn refresh(&mut self) -> Result<(), StorageError> {
        self.count = {
            let count = self.connection.prepare("SELECT COUNT(*) AS c FROM game_records")
                .map_err(|e| StorageError::PreparedStatementError(e))?
                .into_iter().map(|row| row.unwrap()).next().unwrap();
            
            count.read("c")
        };

        let max_page = (self.count - 1) / PAGE_SIZE;

        let page = match self.page {
            Page::Page(page) => {
                let page = page.clamp(0, max_page);
                self.page = Page::Page(page);
                page
            },
            Page::LastPage => max_page,
        };
        
        let records = {
            let mut stmt = self.connection.prepare("SELECT * FROM game_records LIMIT ? OFFSET ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[PAGE_SIZE, page * PAGE_SIZE][..]).map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.into_iter().map(|row| row.unwrap())
        };

        self.records = records.map(|record| Self::parse_record(&record)).collect();

        Ok(())
    }

    fn get_all(&self) -> &[GameRecord] {
        self.records.as_slice()
    }

    // fn get(&self, id: i64) -> Option<&GameRecord> {
    //     self.records.iter().find(|r| r.id == id)
    // }

    fn export_csv(&self, mut path: PathBuf) -> Result<(), StorageError> {
        path = set_export_path_extention(path);

        let mut writer = csv::Writer::from_path(path).map_err(|e| StorageError::ExportError(Box::new(e)))?;

        writer.write_record(&[
            "Time Out",
            "Time In",
            "Game",
            "Quantity",
            "Student Name",
            "Student Number",
            "Receptionist",
            "Notes",
        ]).map_err(|e| StorageError::ExportError(Box::new(e)))?;
        
        let records = {
            let stmt = self.connection.prepare("SELECT * FROM game_records")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.into_iter().map(|row| row.unwrap())
        };

        let records = records.map(|record| Self::parse_record(&record));
        
        for record in records {
            writer.write_record(&[
                record.time_out.to_rfc3339().as_str(),
                &format_optional_time(record.time_in),
                &record.game,
                &record.quantity.to_string(),
                &record.student_name,
                &record.student_number,
                &record.receptionist.unwrap_or_default(),
                &record.notes,
            ]).map_err(|e| StorageError::ExportError(Box::new(e)))?;
        }
        
        writer.flush().map_err(|e| StorageError::ExportError(Box::new(e)))?;

        Ok(())
    }
}

impl AddibleStorage<GameRecord, i64> for GameStorage {
    fn add(&mut self, record: GameRecord) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO game_records (id, game, quantity, student_name, student_number, receptionist, time_out, time_in, notes) VALUES (NULL, ?, ?, ?, ?, NULL, ?, NULL, ?)")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[record.game.as_str(), &record.quantity.to_string(), &record.student_name, &record.student_number, &record.time_out.to_rfc3339(), &record.notes][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl TimeReceptionistUpdateableStorage<GameRecord, i64> for GameStorage {
    fn update_receptionist_and_time(&mut self, id: i64, receptionist: &str) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE game_records SET receptionist = ?, time_in = ? WHERE id = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[receptionist, &chrono::Utc::now().to_rfc3339(), &id.to_string()][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl NotedStorage<GameRecord, i64> for GameStorage {
    fn update_notes(&mut self, id: i64, notes: &str) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE game_records SET notes = ? WHERE id = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[notes, &id.to_string()][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl QuantitySignableStorage<&str> for GameStorage {
    fn get_signed_out(&mut self, item_type: &str) -> Result<i64, StorageError> {
        let mut stmt = self.connection.prepare("SELECT IFNULL(SUM(quantity), 0) AS s FROM game_records WHERE game = ? AND time_in IS NULL")
            .map_err(|e| StorageError::PreparedStatementError(e))?;
        
        stmt.bind(&[item_type][..]).map_err(|e| StorageError::PreparedStatementError(e))?;
        
        let record = stmt.into_iter().map(|row| row.unwrap()).next().unwrap();

        Ok(record.read("s"))
    }
}

pub struct ItemStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<ItemRecord>,
    page: Page,
    count: i64,
}

impl ItemStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<ItemStorage, StorageError> {
        let mut storage = ItemStorage {
            connection,
            records: vec![],
            page: Page::LastPage,
            count: 0,
        };

        storage.refresh()?;

        Ok(storage)
    }

    fn parse_record(record: &sqlite::Row) -> ItemRecord {
        let time_out = record.read("time_out");
        let time_out = chrono::DateTime::parse_from_rfc3339(time_out).unwrap().with_timezone(&chrono::Utc);
        
        ItemRecord {
            id: record.read("id"),
            item: record.read::<&str, _>("item").into(),
            quantity: record.read("quantity"),
            student_name: record.read::<&str, _>("student_name").into(),
            student_number: record.read::<&str, _>("student_number").into(),
            receptionist: record.read::<&str, _>("receptionist").into(),
            time_out,
            notes: record.read::<&str, _>("notes").into(),
        }
    }
}

impl PaginatedStorage<ItemRecord, i64> for ItemStorage {
    fn page(&self) -> Page {
        self.page
    }
    
    fn set_page(&mut self, page: Page) -> Result<(), StorageError> {
        if page != self.page {
            self.page = match page {
                Page::Page(page) => Page::Page(page.clamp(0, self.count / PAGE_SIZE)),
                Page::LastPage => Page::LastPage,
            };
            self.refresh()
        } else {
            Ok(())
        }
    }

    fn count(&self) -> i64 {
        self.count
    }
    
    fn refresh(&mut self) -> Result<(), StorageError> {
        self.count = {
            let count = self.connection.prepare("SELECT COUNT(*) AS c FROM item_records")
                .map_err(|e| StorageError::PreparedStatementError(e))?
                .into_iter().map(|row| row.unwrap()).next().unwrap();
            
            count.read("c")
        };

        let max_page = (self.count - 1) / PAGE_SIZE;

        let page = match self.page {
            Page::Page(page) => {
                let page = page.clamp(0, max_page);
                self.page = Page::Page(page);
                page
            },
            Page::LastPage => max_page,
        };
        
        let records = {
            let mut stmt = self.connection.prepare("SELECT * FROM item_records LIMIT ? OFFSET ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[PAGE_SIZE, page * PAGE_SIZE][..]).map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.into_iter().map(|row| row.unwrap())
        };

        self.records = records.map(|record| Self::parse_record(&record)).collect();

        Ok(())
    }

    fn get_all(&self) -> &[ItemRecord] {
        self.records.as_slice()
    }

    // fn get(&self, id: i64) -> Option<&ItemRecord> {
    //     self.records.iter().find(|r| r.id == id)
    // }

    fn export_csv(&self, mut path: PathBuf) -> Result<(), StorageError> {
        path = set_export_path_extention(path);

        let mut writer = csv::Writer::from_path(path).map_err(|e| StorageError::ExportError(Box::new(e)))?;

        writer.write_record(&[
            "Time Out",
            "Item",
            "Quantity",
            "Student Name",
            "Student Number",
            "Receptionist",
            "Notes",
        ]).map_err(|e| StorageError::ExportError(Box::new(e)))?;
        
        let records = {
            let stmt = self.connection.prepare("SELECT * FROM item_records")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.into_iter().map(|row| row.unwrap())
        };

        let records = records.map(|record| Self::parse_record(&record));
        
        for record in records {
            writer.write_record(&[
                record.time_out.to_rfc3339().as_str(),
                &record.item,
                &record.quantity.to_string(),
                &record.student_name,
                &record.student_number,
                &record.receptionist,
                &record.notes,
            ]).map_err(|e| StorageError::ExportError(Box::new(e)))?;
        }
        
        writer.flush().map_err(|e| StorageError::ExportError(Box::new(e)))?;

        Ok(())
    }
}

impl AddibleStorage<ItemRecord, i64> for ItemStorage {
    fn add(&mut self, record: ItemRecord) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO item_records (id, item, quantity, student_name, student_number, receptionist, time_out, notes) VALUES (NULL, ?, ?, ?, ?, ?, ?, ?)")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
        
            stmt.bind(&[record.item.as_str(), &record.quantity.to_string(), &record.student_name, &record.student_number, &record.receptionist, &record.time_out.to_rfc3339(), &record.notes][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl NotedStorage<ItemRecord, i64> for ItemStorage {
    fn update_notes(&mut self, id: i64, notes: &str) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE item_records SET notes = ? WHERE id = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[notes, &id.to_string()][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

pub struct KeyTypeStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<String>,
}

impl KeyTypeStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<KeyTypeStorage, StorageError> {
        let mut storage = KeyTypeStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl Storage<String, &str> for KeyTypeStorage {
    fn refresh(&mut self) -> Result<(), StorageError> {
        let rows = self.connection.prepare("SELECT * FROM keys ORDER BY key")
            .map_err(|e| StorageError::PreparedStatementError(e))?
            .into_iter().map(|row| row.unwrap());

        self.records = rows.map(|r| r.read::<&str, _>("key").into()).collect();

        Ok(())
    }

    fn get_all(&self) -> &[String] {
        self.records.as_slice()
    }

    fn get(&self, id: &str) -> Option<&String> {
        self.records.iter().find(|r| *r == id)
    }
}

impl AddibleStorage<String, &str> for KeyTypeStorage {
    fn add(&mut self, key: String) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO keys (key) VALUES (?)")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[key.as_str()][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl DeletableStorage<String, &str> for KeyTypeStorage {
    fn delete(&mut self, key: &str) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("DELETE FROM keys WHERE key = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[key][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

pub struct GameTypeStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<GameTypeRecord>,
}

impl GameTypeStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<GameTypeStorage, StorageError> {
        let mut storage = GameTypeStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }

    pub fn update_quantity(&mut self, game: &str, quantity: i64) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE games SET quantity = ? WHERE game = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[quantity.to_string().as_str(), game][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl Storage<GameTypeRecord, &str> for GameTypeStorage {
    fn refresh(&mut self) -> Result<(), StorageError> {
        let rows = self.connection.prepare("SELECT * FROM games ORDER BY game")
            .map_err(|e| StorageError::PreparedStatementError(e))?
            .into_iter().map(|row| row.unwrap());

        self.records = rows.map(|r| GameTypeRecord {
            game: r.read::<&str, _>("game").into(),
            quantity: r.read::<i64, _>("quantity").into(),
        }).collect();

        Ok(())
    }

    fn get_all(&self) -> &[GameTypeRecord] {
        self.records.as_slice()
    }

    fn get(&self, id: &str) -> Option<&GameTypeRecord> {
        self.records.iter().find(|r| r.game == id)
    }
}

impl AddibleStorage<GameTypeRecord, &str> for GameTypeStorage {
    fn add(&mut self, record: GameTypeRecord) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO games (game, quantity) VALUES (?, ?)")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[record.game.as_str(), &record.quantity.to_string()][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl DeletableStorage<GameTypeRecord, &str> for GameTypeStorage {
    fn delete(&mut self, id: &str) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("DELETE FROM games WHERE game = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[id][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

pub struct ItemTypeStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<String>,
}

impl ItemTypeStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<ItemTypeStorage, StorageError> {
        let mut storage = ItemTypeStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl Storage<String, &str> for ItemTypeStorage {
    fn refresh(&mut self) -> Result<(), StorageError> {
        let rows = self.connection.prepare("SELECT * FROM items ORDER BY item")
            .map_err(|e| StorageError::PreparedStatementError(e))?
            .into_iter().map(|row| row.unwrap());

        self.records = rows.map(|r| r.read::<&str, _>("item").into()).collect();

        Ok(())
    }

    fn get_all(&self) -> &[String] {
        self.records.as_slice()
    }

    fn get(&self, id: &str) -> Option<&String> {
        self.records.iter().find(|r| *r == id)
    }
}

impl AddibleStorage<String, &str> for ItemTypeStorage {
    fn add(&mut self, item: String) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO items (item) VALUES (?)")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[item.as_str()][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl DeletableStorage<String, &str> for ItemTypeStorage {
    fn delete(&mut self, item: &str) -> Result<(), StorageError> {
        {
            let mut stmt = self.connection.prepare("DELETE FROM items WHERE item = ?")
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[item][..])
                .map_err(|e| StorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}
