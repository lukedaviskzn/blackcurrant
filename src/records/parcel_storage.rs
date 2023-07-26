use std::{path::PathBuf, sync::{Arc, Mutex}};

use crate::app::PAGE_SIZE;

use super::{Page, StorageError, PaginatedStorage, format_optional_time, InsertableStorage, SignableStorage, NotedStorage, ExportableStorage, ParcelRecord, NewParcelRecord};

pub struct ParcelStorage {
    connection: Arc<Mutex<rusqlite::Connection>>,
    records: Vec<ParcelRecord>,
    page: Page,
    count: i64,
}

impl ParcelStorage {
    pub fn new(connection: Arc<Mutex<rusqlite::Connection>>) -> Result<ParcelStorage, StorageError> {
        let mut storage = ParcelStorage {
            connection,
            records: vec![],
            page: Page::LastPage,
            count: 0,
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl PaginatedStorage<ParcelRecord, i64> for ParcelStorage {
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
            let count = self.connection.lock().unwrap().prepare("SELECT COUNT(*) AS c FROM parcel_records")?
                .query_row((), |row| row.get("c"))?;
            
            count
        };

        let page = self.page.as_i64(self.count);
        
        self.records = {
            let connection = self.connection.lock().unwrap();
            
            let mut stmt = connection.prepare("SELECT * FROM parcel_records LIMIT ? OFFSET ?")?;
            
            let records = stmt.query_map((PAGE_SIZE, page * PAGE_SIZE), |row| Self::parse_row(row))?
                .collect::<Result<_, _>>()?;

            records
        };
        
        log::debug!("refreshed parcel records");

        Ok(())
    }

    fn get_all(&self) -> &[ParcelRecord] {
        self.records.as_slice()
    }

    fn parse_row(row: &rusqlite::Row) -> Result<ParcelRecord, rusqlite::Error> {
        let time_in: String = row.get("time_in")?;
        let time_in = chrono::DateTime::parse_from_rfc3339(&time_in).expect(&format!("db contains invalid parcel time_in string: {time_in}")).with_timezone(&chrono::Utc);
        
        let time_out: Option<String> = row.get("time_out")?;
        let time_out = time_out.map(|time_out| chrono::DateTime::parse_from_rfc3339(&time_out).expect(&format!("db contains invalid parcel time_out string: {time_out}")).with_timezone(&chrono::Utc));
        
        Ok(ParcelRecord {
            id: row.get("id")?,
            parcel_desc: row.get("parcel_desc")?,
            student_name: row.get("student_name")?,
            receptionist: row.get("receptionist")?,
            time_in,
            time_out,
            notes: row.get("notes")?,
        })
    }
}

impl InsertableStorage<NewParcelRecord<'_>, i64> for ParcelStorage {
    fn insert(&mut self, record: NewParcelRecord) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "INSERT INTO parcel_records (id, parcel_desc, student_name, receptionist, time_in, time_out, notes) VALUES (NULL, ?, ?, ?, ?, NULL, ?)",
            (record.parcel_desc, record.student_name, record.receptionist, chrono::Utc::now().to_rfc3339(), record.notes)
        )?;

        self.refresh()?;
        
        Ok(())
    }
}

impl SignableStorage<ParcelRecord, i64> for ParcelStorage {
    fn signin(&mut self, id: i64) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "UPDATE parcel_records SET time_out = ? WHERE id = ?",
            (chrono::Utc::now().to_rfc3339(), id)
        )?;

        self.refresh()?;
        
        Ok(())
    }
}

impl NotedStorage<ParcelRecord, i64> for ParcelStorage {
    fn update_notes(&mut self, id: i64, notes: &str) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "UPDATE parcel_records SET notes = ? WHERE id = ?",
            (notes, id)
        )?;

        self.refresh()?;
        
        Ok(())
    }
}

impl ExportableStorage<ParcelRecord> for ParcelStorage {
    fn fetch_all(&self) -> Result<Vec<ParcelRecord>, StorageError> {
        let records = {
            let connection = self.connection.lock().unwrap();
            
            let mut stmt = connection.prepare("SELECT * FROM parcel_records")?;
            
            let records = stmt.query_map((), |row| Self::parse_row(row))?
                .collect::<Result<_, _>>()?;

            records
        };
        
        log::debug!("fetched all parcel records");

        Ok(records)
    }

    fn csv_headers() -> &'static [&'static str] {
        &[
            "Time In",
            "Time Out",
            "Parcel Description",
            "Student Name",
            "Receptionist",
            "Notes",
        ]
    }

    fn write_record<W: std::io::Write>(writer: &mut csv::Writer<W>, record: &ParcelRecord) -> Result<(), csv::Error> {
        writer.write_record(&[
            record.time_in.to_rfc3339().as_str(),
            &format_optional_time(record.time_out),
            &record.parcel_desc,
            &record.student_name,
            &record.receptionist,
            &record.notes,
        ])
    }

    fn export_csv(&self, path: PathBuf) -> Result<(), StorageError> {
        super::export_csv(self, path)
    }
}
