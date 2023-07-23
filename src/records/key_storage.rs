use std::{path::PathBuf, sync::{Arc, Mutex}};

use rusqlite::OptionalExtension;

use crate::app::PAGE_SIZE;

use super::{Page, StorageError, PaginatedStorage, format_optional_time, AddibleStorage, TimeReceptionistUpdateableStorage, NotedStorage, SignableStorage, ExportableStorage};

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

pub struct KeyStorage {
    connection: Arc<Mutex<rusqlite::Connection>>,
    records: Vec<KeyRecord>,
    page: Page,
    count: i64,
}

impl KeyStorage {
    pub fn new(connection: Arc<Mutex<rusqlite::Connection>>) -> Result<KeyStorage, StorageError> {
        let mut storage = KeyStorage {
            connection,
            records: vec![],
            page: Page::LastPage,
            count: 0,
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl PaginatedStorage<KeyRecord, i64> for KeyStorage {
    fn page(&self) -> Page {
        self.page
    }
    
    fn set_page(&mut self, page: Page) -> Result<(), StorageError> {
        if page != self.page {
            self.page = page.clamp(self.count);
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
            let count = self.connection.lock().unwrap().prepare("SELECT COUNT(*) AS c FROM key_records")?
                .query_row([], |row| row.get("c"))?;
            
            count
        };

        let page = self.page.as_i64(self.count);
        
        self.records = {
            let connection = self.connection.lock().unwrap();
            
            let mut stmt = connection.prepare("SELECT * FROM key_records LIMIT ? OFFSET ?")?;
            
            let records = stmt.query_map([PAGE_SIZE, page * PAGE_SIZE], |row| Self::parse_row(row))?
                .collect::<Result<_, _>>()?;

            records
        };
        
        log::debug!("refreshed key records");

        Ok(())
    }

    fn get_all(&self) -> &[KeyRecord] {
        self.records.as_slice()
    }

    fn parse_row(row: &rusqlite::Row) -> Result<KeyRecord, rusqlite::Error> {
        let time_out: String = row.get("time_out")?;
        let time_out = chrono::DateTime::parse_from_rfc3339(&time_out).expect(&format!("db contains invalid key time_out string: {time_out}")).with_timezone(&chrono::Utc);
        
        let time_in: Option<String> = row.get("time_in")?;
        let time_in = time_in.map(|time_in| chrono::DateTime::parse_from_rfc3339(&time_in).expect(&format!("db contains invalid key time_in string: {time_in}")).with_timezone(&chrono::Utc));
        
        Ok(KeyRecord {
            id: row.get("id")?,
            key: row.get("key")?,
            student_name: row.get("student_name")?,
            student_number: row.get("student_number")?,
            receptionist: row.get("receptionist")?,
            time_out,
            time_in,
            notes: row.get("notes")?,
        })
    }
}

impl AddibleStorage<KeyRecord, i64> for KeyStorage {
    fn add(&mut self, record: KeyRecord) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "INSERT INTO key_records (id, key, student_name, student_number, receptionist, time_out, time_in, notes) VALUES (NULL, ?, ?, ?, NULL, ?, NULL, ?)",
            [record.key.as_str(), &record.student_name, &record.student_number, &record.time_out.to_rfc3339(), &record.notes])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl TimeReceptionistUpdateableStorage<KeyRecord, i64> for KeyStorage {
    fn update_receptionist_and_time(&mut self, id: i64, receptionist: &str) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "UPDATE key_records SET receptionist = ?, time_in = ? WHERE id = ?",
            [receptionist, &chrono::Utc::now().to_rfc3339(), &id.to_string()])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl NotedStorage<KeyRecord, i64> for KeyStorage {
    fn update_notes(&mut self, id: i64, notes: &str) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "UPDATE key_records SET notes = ? WHERE id = ?",
            [notes, &id.to_string()])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl SignableStorage<KeyRecord, &str> for KeyStorage {
    fn get_signed_out(&mut self, item_type: &str) -> Result<Option<KeyRecord>, StorageError> {
        let connection = self.connection.lock().unwrap();
        
        let mut stmt = connection.prepare("SELECT * FROM key_records WHERE key = ? AND time_in IS NULL LIMIT 1")?;
        
        let record = stmt.query_row(&[item_type], |row| Self::parse_row(row))
            .optional()?;

        Ok(record)
    }
}

impl ExportableStorage<KeyRecord> for KeyStorage {
    fn fetch_all(&self) -> Result<Vec<KeyRecord>, StorageError> {
        let records = {
            let connection = self.connection.lock().unwrap();
            
            let mut stmt = connection.prepare("SELECT * FROM key_records")?;
            
            let records = stmt.query_map([], |row| Self::parse_row(row))?
                .collect::<Result<_, _>>()?;

            records
        };
        
        log::debug!("fetched all key records");

        Ok(records)
    }

    fn csv_headers() -> &'static [&'static str] {
        &[
            "Time Out",
            "Time In",
            "Key",
            "Student Name",
            "Student Number",
            "Receptionist",
            "Notes",
        ]
    }

    fn write_record<W: std::io::Write>(writer: &mut csv::Writer<W>, record: &KeyRecord) -> Result<(), csv::Error> {
        writer.write_record(&[
            record.time_out.to_rfc3339().as_str(),
            &format_optional_time(record.time_in),
            &record.key,
            &record.student_name,
            &record.student_number,
            record.receptionist.as_ref().unwrap_or(&String::new()),
            &record.notes,
        ])
    }
    
    fn export_csv(&self, path: PathBuf) -> Result<(), StorageError> {
        super::export_csv(self, path)
    }
}
