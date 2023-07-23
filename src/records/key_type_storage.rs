use std::sync::{Arc, Mutex};

use super::{StorageError, Storage, AddibleStorage, DeletableStorage};

pub struct KeyTypeStorage {
    connection: Arc<Mutex<rusqlite::Connection>>,
    records: Vec<String>,
}

impl KeyTypeStorage {
    pub fn new(connection: Arc<Mutex<rusqlite::Connection>>) -> Result<KeyTypeStorage, StorageError> {
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
        let connection = self.connection.lock().unwrap();
        
        let mut stmt = connection.prepare("SELECT * FROM keys ORDER BY key")?;
        
        let records = stmt
            .query_map([], |row| Self::parse_row(row))?
            .collect::<Result<_, _>>()?;
        
        self.records = records;

        log::debug!("refreshed key types");
        
        Ok(())
    }

    fn get_all(&self) -> &[String] {
        self.records.as_slice()
    }

    fn get(&self, id: &str) -> Option<&String> {
        self.records.iter().find(|r| *r == id)
    }

    fn parse_row(row: &rusqlite::Row) -> Result<String, rusqlite::Error> {
        row.get("key")
    }
}

impl AddibleStorage<String, &str> for KeyTypeStorage {
    fn add(&mut self, key: String) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "INSERT INTO keys (key) VALUES (?)",
            [&key])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl DeletableStorage<String, &str> for KeyTypeStorage {
    fn delete(&mut self, key: &str) -> Result<(), StorageError> {
        self.connection.lock().unwrap().execute(
            "DELETE FROM keys WHERE key = ?",
            [&key])?;

        self.refresh()?;
        
        Ok(())
    }
}
