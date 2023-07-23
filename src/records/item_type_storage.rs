use std::sync::{Arc, Mutex};

use super::{StorageError, Storage, DeletableStorage, AddibleStorage};

pub struct ItemTypeStorage {
    connection: Arc<Mutex<rusqlite::Connection>>,
    records: Vec<String>,
}

impl ItemTypeStorage {
    pub fn new(connection: Arc<Mutex<rusqlite::Connection>>) -> Result<ItemTypeStorage, StorageError> {
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
        let connection = self.connection.lock().unwrap();
        
        let mut stmt = connection.prepare("SELECT * FROM items ORDER BY item")?;
        
        let records = stmt
            .query_map([], |row| Self::parse_row(row))?
            .collect::<Result<_, _>>()?;

        self.records = records;

        log::debug!("refreshed item types");

        Ok(())
    }

    fn get_all(&self) -> &[String] {
        self.records.as_slice()
    }

    fn get(&self, id: &str) -> Option<&String> {
        self.records.iter().find(|r| *r == id)
    }

    fn parse_row(row: &rusqlite::Row) -> Result<String, rusqlite::Error> {
        row.get("item")
    }
}

impl AddibleStorage<String, &str> for ItemTypeStorage {
    fn add(&mut self, item: String) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "INSERT INTO items (item) VALUES (?)",
            [&item])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl DeletableStorage<String, &str> for ItemTypeStorage {
    fn delete(&mut self, item: &str) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "DELETE FROM items WHERE item = ?",
        [&item])?;

        self.refresh()?;
        
        Ok(())
    }
}
