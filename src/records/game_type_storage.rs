use std::rc::Rc;

use super::{GameTypeRecord, StorageError, Storage, AddibleStorage, DeletableStorage};

pub struct GameTypeStorage {
    connection: Rc<rusqlite::Connection>,
    records: Vec<GameTypeRecord>,
}

impl GameTypeStorage {
    pub fn new(connection: Rc<rusqlite::Connection>) -> Result<GameTypeStorage, StorageError> {
        let mut storage = GameTypeStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }

    pub fn update_quantity(&mut self, game: &str, quantity: i64) -> Result<(), StorageError> {
        self.connection.execute(
            "UPDATE games SET quantity = ? WHERE game = ?",
            [quantity.to_string().as_str(), game])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl Storage<GameTypeRecord, &str> for GameTypeStorage {
    fn refresh(&mut self) -> Result<(), StorageError> {
        let mut stmt = self.connection.prepare("SELECT * FROM games ORDER BY game")?;
        
        let records = stmt
            .query_map([], |row| Self::parse_row(row))?
            .collect::<Result<_, _>>()?;

        self.records = records;

        Ok(())
    }

    fn get_all(&self) -> &[GameTypeRecord] {
        self.records.as_slice()
    }

    fn get(&self, id: &str) -> Option<&GameTypeRecord> {
        self.records.iter().find(|r| r.game == id)
    }

    fn parse_row(row: &rusqlite::Row) -> Result<GameTypeRecord, rusqlite::Error> {
        Ok(GameTypeRecord {
            game: row.get("game")?,
            quantity: row.get("quantity")?,
        })
    }
}

impl AddibleStorage<GameTypeRecord, &str> for GameTypeStorage {
    fn add(&mut self, record: GameTypeRecord) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO games (game, quantity) VALUES (?, ?)",
            [record.game.as_str(), &record.quantity.to_string()])?;

        self.refresh()?;
        
        Ok(())
    }
}

impl DeletableStorage<GameTypeRecord, &str> for GameTypeStorage {
    fn delete(&mut self, id: &str) -> Result<(), StorageError> {
        self.connection.execute(
            "DELETE FROM games WHERE game = ?",
            [id])?;

        self.refresh()?;
        
        Ok(())
    }
}
