use std::{path::PathBuf, sync::{Arc, Mutex}};

use crate::app::PAGE_SIZE;

use super::{Page, StorageError, PaginatedStorage, format_optional_time, InsertableStorage, ReceptionistSignableStorage, NotedStorage, ExportableStorage, GameRecord, NewGameRecord, Summary};

pub struct GameStorage {
    connection: Arc<Mutex<rusqlite::Connection>>,
    records: Vec<GameRecord>,
    page: Page,
    count: i64,
}

impl GameStorage {
    pub fn new(connection: Arc<Mutex<rusqlite::Connection>>) -> Result<GameStorage, StorageError> {
        let mut storage = GameStorage {
            connection,
            records: vec![],
            page: Page::LastPage,
            count: 0,
        };

        storage.refresh()?;

        Ok(storage)
    }
    
    pub fn get_signed_out(&mut self, item_type: &str) -> Result<i64, StorageError> {
        let num_signed_out = self.connection.lock().unwrap().prepare("SELECT IFNULL(SUM(quantity), 0) AS s FROM game_records WHERE game = ? AND time_in IS NULL")?
            .query_row((item_type,), |row| row.get::<_, i64>("s"))?;

        Ok(num_signed_out)
    }
}

impl PaginatedStorage<GameRecord, i64> for GameStorage {
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
            let count = self.connection.lock().unwrap().prepare("SELECT COUNT(*) AS c FROM game_records")?
                .query_row((), |row| row.get("c"))?;
            
            count
        };

        let page = self.page.as_i64(self.count);
        
        self.records = {
            let connection = self.connection.lock().unwrap();
            
            let mut stmt = connection.prepare("SELECT * FROM game_records LIMIT ? OFFSET ?")?;
            
            let records = stmt.query_map((PAGE_SIZE, page * PAGE_SIZE), |row| Self::parse_row(row))?
                .collect::<Result<_, _>>()?;

            records
        };
        
        log::debug!("refreshed game records");

        Ok(())
    }

    fn get_all(&self) -> &[GameRecord] {
        self.records.as_slice()
    }
    
    fn parse_row(row: &rusqlite::Row) -> Result<GameRecord, rusqlite::Error> {
        let time_out: String = row.get("time_out")?;
        let time_out = chrono::DateTime::parse_from_rfc3339(&time_out).expect("failed to parse game record time_out").with_timezone(&chrono::Utc);
        
        let time_in: Option<String> = row.get("time_in")?;
        let time_in = time_in.map(|time_in| chrono::DateTime::parse_from_rfc3339(&time_in).expect("failed to parse game record time_in").with_timezone(&chrono::Utc));
        
        Ok(GameRecord {
            id: row.get("id")?,
            game: row.get("game")?,
            quantity: row.get("quantity")?,
            student_name: row.get("student_name")?,
            student_number: row.get("student_number")?,
            receptionist: row.get("receptionist")?,
            time_out,
            time_in,
            notes: row.get("notes")?,
        })
    }
}

impl InsertableStorage<NewGameRecord<'_>, i64> for GameStorage {
    fn insert(&mut self, record: NewGameRecord) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "INSERT INTO game_records (id, game, quantity, student_name, student_number, receptionist, time_out, time_in, notes) VALUES (NULL, ?, ?, ?, ?, NULL, ?, NULL, ?)",
            (record.game, record.quantity, record.student_name, record.student_number.to_uppercase(), chrono::Utc::now().to_rfc3339(), record.notes)
        )?;

        self.refresh()?;
        
        Ok(())
    }
}

impl ReceptionistSignableStorage<GameRecord, i64> for GameStorage {
    fn signin(&mut self, id: i64, receptionist: &str) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "UPDATE game_records SET receptionist = ?, time_in = ? WHERE id = ?",
            (receptionist, chrono::Utc::now().to_rfc3339(), id)
        )?;

        self.refresh()?;
        
        Ok(())
    }
}

impl NotedStorage<GameRecord, i64> for GameStorage {
    fn update_notes(&mut self, id: i64, notes: &str) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "UPDATE game_records SET notes = ? WHERE id = ?",
            (notes, id)
        )?;

        self.refresh()?;
        
        Ok(())
    }
}

impl ExportableStorage<GameRecord> for GameStorage {
    fn fetch_all(&self) -> Result<Vec<GameRecord>, StorageError> {
        let records = {
            let connection = self.connection.lock().unwrap();
            
            let mut stmt = connection.prepare("SELECT * FROM game_records")?;
            
            let records = stmt.query_map((), |row| Self::parse_row(row))?
                .collect::<Result<_, _>>()?;

            records
        };
        
        log::debug!("fetched all game records");

        Ok(records)
    }

    fn csv_headers() -> &'static [&'static str] {
        &[
            "Time Out",
            "Time In",
            "Game",
            "Quantity",
            "Student Name",
            "Student Number",
            "Receptionist",
            "Notes",
        ]
    }

    fn write_record<W: std::io::Write>(writer: &mut csv::Writer<W>, record: &GameRecord) -> Result<(), csv::Error> {
        writer.write_record(&[
            record.time_out.to_rfc3339().as_str(),
            &format_optional_time(record.time_in),
            &record.game,
            &record.quantity.to_string(),
            &record.student_name,
            &record.student_number,
            record.receptionist.as_ref().map(|r| r.as_str()).unwrap_or_default(),
            &record.notes,
        ])
    }

    fn export_csv(&self, path: PathBuf) -> Result<(), StorageError> {
        super::export_csv(self, path)
    }
}

impl Summary for GameStorage {
    /// find all records with time_out in [start, end)
    fn summary(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Result<Vec<(String, i64)>, StorageError> {
        fn parse_row(row: &rusqlite::Row) -> Result<(String, i64), rusqlite::Error> {
            Ok((row.get("game")?, row.get("c")?))
        }

        let records = {
            let conn = self.connection.lock().unwrap();

            let mut stmt = conn.prepare("SELECT game, SUM(quantity) AS c FROM game_records WHERE ? <= time_out AND time_out < ? GROUP BY game ORDER BY game")?;

            let start = start.with_timezone(&chrono::Utc).to_rfc3339();
            let end = end.with_timezone(&chrono::Utc).to_rfc3339();

            let records = stmt.query_map((start, end), |row| parse_row(row))?;
            records.collect::<Result<Vec<_>, _>>()?
        };

        Ok(records)
    }
}
