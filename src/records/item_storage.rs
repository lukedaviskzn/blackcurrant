use std::{rc::Rc, path::PathBuf};

use tracing::debug;

use crate::app::PAGE_SIZE;

use super::{Page, StorageError, PaginatedStorage, AddibleStorage, NotedStorage, ExportableStorage};

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

pub struct ItemStorage {
    connection: Rc<rusqlite::Connection>,
    records: Vec<ItemRecord>,
    page: Page,
    count: i64,
}

impl ItemStorage {
    pub fn new(connection: Rc<rusqlite::Connection>) -> Result<ItemStorage, StorageError> {
        let mut storage = ItemStorage {
            connection,
            records: vec![],
            page: Page::LastPage,
            count: 0,
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl PaginatedStorage<ItemRecord, i64> for ItemStorage {
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
            let count = self.connection.prepare("SELECT COUNT(*) AS c FROM item_records")?
                .query_row([], |row| row.get("c"))?;
            
            count
        };

        let page = self.page.as_i64(self.count);
        
        self.records = {
            let mut stmt = self.connection.prepare("SELECT * FROM item_records LIMIT ? OFFSET ?")?;
            
            let records = stmt.query_map([PAGE_SIZE, page * PAGE_SIZE], |row| Self::parse_row(row))?
                .collect::<Result<_, _>>()?;

            records
        };
        
        debug!("refreshed item records");

        Ok(())
    }

    fn get_all(&self) -> &[ItemRecord] {
        self.records.as_slice()
    }

    fn parse_row(row: &rusqlite::Row) -> Result<ItemRecord, rusqlite::Error> {
        let time_out: String = row.get("time_out")?;
        let time_out = chrono::DateTime::parse_from_rfc3339(&time_out).expect("failed to parse item record time_out").with_timezone(&chrono::Utc);
        
        Ok(ItemRecord {
            id: row.get("id")?,
            item: row.get("item")?,
            quantity: row.get("quantity")?,
            student_name: row.get("student_name")?,
            student_number: row.get("student_number")?,
            receptionist: row.get("receptionist")?,
            time_out,
            notes: row.get("notes")?,
        })
    }
}

impl AddibleStorage<ItemRecord, i64> for ItemStorage {
    fn add(&mut self, record: ItemRecord) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO item_records (id, item, quantity, student_name, student_number, receptionist, time_out, notes) VALUES (NULL, ?, ?, ?, ?, ?, ?, ?)",
            [record.item.as_str(), &record.quantity.to_string(), &record.student_name, &record.student_number, &record.receptionist, &record.time_out.to_rfc3339(), &record.notes])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl NotedStorage<ItemRecord, i64> for ItemStorage {
    fn update_notes(&mut self, id: i64, notes: &str) -> Result<(), StorageError> {
        self.connection.execute(
            "UPDATE item_records SET notes = ? WHERE id = ?",
            [notes, &id.to_string()])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl ExportableStorage<ItemRecord> for ItemStorage {
    fn fetch_all(&self) -> Result<Vec<ItemRecord>, StorageError> {
        let records = {
            let mut stmt = self.connection.prepare("SELECT * FROM item_records")?;
            
            let records = stmt.query_map([], |row| Self::parse_row(row))?
                .collect::<Result<_, _>>()?;

            records
        };
        
        debug!("fetched all item records");

        Ok(records)
    }

    fn csv_headers() -> &'static [&'static str] {
        &[
            "Time Out",
            "Item",
            "Quantity",
            "Student Name",
            "Student Number",
            "Receptionist",
            "Notes",
        ]
    }

    fn write_record<W: std::io::Write>(writer: &mut csv::Writer<W>, record: &ItemRecord) -> Result<(), csv::Error> {
        writer.write_record(&[
            record.time_out.to_rfc3339().as_str(),
            &record.item,
            &record.quantity.to_string(),
            &record.student_name,
            &record.student_number,
            &record.receptionist,
            &record.notes,
        ])
    }

    fn export_csv(&self, path: PathBuf) -> Result<(), StorageError> {
        super::export_csv(self, path)
    }
}
