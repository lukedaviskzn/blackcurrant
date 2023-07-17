use std::{rc::Rc, path::PathBuf};

use tracing::debug;

use crate::app::PAGE_SIZE;

use super::{Page, StorageError, PaginatedStorage, format_optional_time, AddibleStorage, TimeUpdateableStorage, NotedStorage, ExportableStorage};

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

pub struct ParcelStorage {
    connection: Rc<rusqlite::Connection>,
    records: Vec<ParcelRecord>,
    page: Page,
    count: i64,
}

impl ParcelStorage {
    pub fn new(connection: Rc<rusqlite::Connection>) -> Result<ParcelStorage, StorageError> {
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
            let count = self.connection.prepare("SELECT COUNT(*) AS c FROM parcel_records")?
                .query_row([], |row| row.get("c"))?;
            
            count
        };

        let page = self.page.as_i64(self.count);
        
        self.records = {
            let mut stmt = self.connection.prepare("SELECT * FROM parcel_records LIMIT ? OFFSET ?")?;
            
            let records = stmt.query_map([PAGE_SIZE, page * PAGE_SIZE], |row| Self::parse_row(row))?
                .collect::<Result<_, _>>()?;

            records
        };
        
        debug!("refreshed parcel records");

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

impl AddibleStorage<ParcelRecord, i64> for ParcelStorage {
    fn add(&mut self, record: ParcelRecord) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO parcel_records (id, parcel_desc, student_name, receptionist, time_in, time_out, notes) VALUES (NULL, ?, ?, ?, ?, NULL, ?)",
            [record.parcel_desc.as_str(), &record.student_name, &record.receptionist, &record.time_in.to_rfc3339(), &record.notes])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl TimeUpdateableStorage<ParcelRecord, i64> for ParcelStorage {
    fn update_time(&mut self, id: i64) -> Result<(), StorageError> {
        self.connection.execute(
            "UPDATE parcel_records SET time_out = ? WHERE id = ?",
            [chrono::Utc::now().to_rfc3339().as_str(), &id.to_string()])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl NotedStorage<ParcelRecord, i64> for ParcelStorage {
    fn update_notes(&mut self, id: i64, notes: &str) -> Result<(), StorageError> {
        self.connection.execute(
            "UPDATE parcel_records SET notes = ? WHERE id = ?",
            [notes, &id.to_string()])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl ExportableStorage<ParcelRecord> for ParcelStorage {
    fn fetch_all(&self) -> Result<Vec<ParcelRecord>, StorageError> {
        todo!()
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
