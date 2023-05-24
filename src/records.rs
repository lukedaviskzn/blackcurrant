use std::rc::Rc;

use err_derive::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    KeyPanel,
    ParcelPanel,
    GamePanel,
    ItemPanel,
}

#[derive(Debug, Clone)]
pub struct KeyRecord {
    pub id: i64,
    pub key: String,
    pub student_name: String,
    pub student_number: String,
    pub receptionist: String,
    pub time_out: chrono::DateTime<chrono::Utc>,
    pub time_in: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct ParcelRecord {
    pub id: i64,
    pub parcel_desc: String,
    pub student_name: String,
    pub receptionist: String,
    pub time_in: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct GameRecord {
    pub id: i64,
    pub game: String,
    pub quantity: i64,
    pub student_name: String,
    pub student_number: String,
    pub receptionist: String,
    pub time_out: chrono::DateTime<chrono::Utc>,
    pub time_in: Option<chrono::DateTime<chrono::Utc>>,
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
}

#[derive(Debug, Clone)]
pub struct GameTypeRecord {
    pub game: String,
    pub quantity: i64,
}

#[derive(Debug, Error)]
pub enum RecordStorageError {
    // #[error(display = "Failed to connect to local database.")]
    // DatabaseConnectionError(sqlite::Error),
    #[error(display = "Failed to access local database.")]
    PreparedStatementError(sqlite::Error),
}

pub trait RecordStorage<T, I: Copy> {
    fn refresh(&mut self) -> Result<(), RecordStorageError>;
    fn get_all(&self) -> &[T];
    fn get(&self, id: I) -> Option<&T>;
    fn add(&mut self, record_info: T) -> Result<(), RecordStorageError>;
    fn update_time(&mut self, id: I) -> Result<(), RecordStorageError>;
    fn delete(&mut self, id: I) -> Result<(), RecordStorageError>;
}

pub struct KeyStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<KeyRecord>,
}

impl KeyStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<KeyStorage, RecordStorageError> {
        let mut storage = KeyStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl RecordStorage<KeyRecord, i64> for KeyStorage {
    fn refresh(&mut self) -> Result<(), RecordStorageError> {
        let records = self.connection.prepare("SELECT * FROM key_records")
            .map_err(|e| RecordStorageError::PreparedStatementError(e))?
            .into_iter().map(|row| row.unwrap());

        self.records = records.map(|record| {
            let time_out = record.read("time_out");
            let time_out = chrono::DateTime::parse_from_rfc3339(time_out).unwrap().with_timezone(&chrono::Utc);
            
            let time_in = record.read::<Option<&str>, _>("time_in");
            let time_in = time_in.map(|time_in| chrono::DateTime::parse_from_rfc3339(time_in).unwrap().with_timezone(&chrono::Utc));
            
            KeyRecord {
                id: record.read("id"),
                key: record.read::<&str, _>("key").into(),
                student_name: record.read::<&str, _>("student_name").into(),
                student_number: record.read::<&str, _>("student_number").into(),
                receptionist: record.read::<&str, _>("receptionist").into(),
                time_out,
                time_in,
            }
        }).collect();

        Ok(())
    }

    fn get_all(&self) -> &[KeyRecord] {
        self.records.as_slice()
    }

    fn get(&self, id: i64) -> Option<&KeyRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    fn add(&mut self, record: KeyRecord) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO key_records (id, key, student_name, student_number, receptionist, time_out, time_in) VALUES (NULL, ?, ?, ?, ?, ?, NULL)")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
        
            stmt.bind(&[record.key.as_str(), &record.student_name, &record.student_number, &record.receptionist, &record.time_out.to_rfc3339()][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }

    fn update_time(&mut self, id: i64) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE key_records SET time_in = ? WHERE id = ?")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[chrono::Utc::now().to_rfc3339().as_str(), &id.to_string()][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }

    fn delete(&mut self, _id: i64) -> Result<(), RecordStorageError> {
        unimplemented!();
    }
}

pub struct ParcelStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<ParcelRecord>,
}

impl ParcelStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<ParcelStorage, RecordStorageError> {
        let mut storage = ParcelStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl RecordStorage<ParcelRecord, i64> for ParcelStorage {
    fn refresh(&mut self) -> Result<(), RecordStorageError> {
        let records = self.connection.prepare("SELECT * FROM parcel_records")
            .map_err(|e| RecordStorageError::PreparedStatementError(e))?
            .into_iter().map(|row| row.unwrap());

        self.records = records.map(|record| {
            let time_in = record.read::<&str, _>("time_in");
            let time_in = chrono::DateTime::parse_from_rfc3339(time_in).unwrap().with_timezone(&chrono::Utc);
            
            ParcelRecord {
                id: record.read("id"),
                parcel_desc: record.read::<&str, _>("parcel_desc").into(),
                student_name: record.read::<&str, _>("student_name").into(),
                receptionist: record.read::<&str, _>("receptionist").into(),
                time_in,
            }
        }).collect();

        Ok(())
    }

    fn get_all(&self) -> &[ParcelRecord] {
        self.records.as_slice()
    }

    fn get(&self, id: i64) -> Option<&ParcelRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    fn add(&mut self, record: ParcelRecord) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO parcel_records (id, parcel_desc, student_name, receptionist, time_in) VALUES (NULL, ?, ?, ?, ?)")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[record.parcel_desc.as_str(), &record.student_name, &record.receptionist, &record.time_in.to_rfc3339()][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }

    fn update_time(&mut self, _id: i64) -> Result<(), RecordStorageError> {
        unimplemented!();
    }

    fn delete(&mut self, _id: i64) -> Result<(), RecordStorageError> {
        unimplemented!();
    }
}

pub struct GameStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<GameRecord>,
}

impl GameStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<GameStorage, RecordStorageError> {
        let mut storage = GameStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl RecordStorage<GameRecord, i64> for GameStorage {
    fn refresh(&mut self) -> Result<(), RecordStorageError> {
        let records = self.connection.prepare("SELECT * FROM game_records")
            .map_err(|e| RecordStorageError::PreparedStatementError(e))?
            .into_iter().map(|row| row.unwrap());

        self.records = records.map(|record| {
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
                receptionist: record.read::<&str, _>("receptionist").into(),
                time_out,
                time_in,
            }
        }).collect();

        Ok(())
    }

    fn get_all(&self) -> &[GameRecord] {
        self.records.as_slice()
    }

    fn get(&self, id: i64) -> Option<&GameRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    fn add(&mut self, record: GameRecord) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO game_records (id, game, quantity, student_name, student_number, receptionist, time_out, time_in) VALUES (NULL, ?, ?, ?, ?, ?, ?, NULL)")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[record.game.as_str(), &record.quantity.to_string(), &record.student_name, &record.student_number, &record.receptionist, &record.time_out.to_rfc3339()][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }

    fn update_time(&mut self, id: i64) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE game_records SET time_in = ? WHERE id = ?")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[chrono::Utc::now().to_rfc3339().as_str(), &id.to_string()][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }

    fn delete(&mut self, _id: i64) -> Result<(), RecordStorageError> {
        unimplemented!();
    }
}

pub struct ItemStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<ItemRecord>,
}

impl ItemStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<ItemStorage, RecordStorageError> {
        let mut storage = ItemStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl RecordStorage<ItemRecord, i64> for ItemStorage {
    fn refresh(&mut self) -> Result<(), RecordStorageError> {
        let records = self.connection.prepare("SELECT * FROM item_records")
            .map_err(|e| RecordStorageError::PreparedStatementError(e))?
            .into_iter().map(|row| row.unwrap());

        self.records = records.map(|record| {
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
            }
        }).collect();

        Ok(())
    }

    fn get_all(&self) -> &[ItemRecord] {
        self.records.as_slice()
    }

    fn get(&self, id: i64) -> Option<&ItemRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    fn add(&mut self, record: ItemRecord) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO item_records (id, item, quantity, student_name, student_number, receptionist, time_out) VALUES (NULL, ?, ?, ?, ?, ?, ?)")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
        
            stmt.bind(&[record.item.as_str(), &record.quantity.to_string(), &record.student_name, &record.student_number, &record.receptionist, &record.time_out.to_rfc3339()][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }

    fn update_time(&mut self, _id: i64) -> Result<(), RecordStorageError> {
        unimplemented!();
    }

    fn delete(&mut self, _id: i64) -> Result<(), RecordStorageError> {
        unimplemented!();
    }
}

pub struct KeyTypeStorage {
    connection: Rc<sqlite::Connection>,
    records: Vec<String>,
}

impl KeyTypeStorage {
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<KeyTypeStorage, RecordStorageError> {
        let mut storage = KeyTypeStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl RecordStorage<String, &str> for KeyTypeStorage {
    fn refresh(&mut self) -> Result<(), RecordStorageError> {
        let rows = self.connection.prepare("SELECT * FROM keys ORDER BY key")
            .map_err(|e| RecordStorageError::PreparedStatementError(e))?
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

    fn add(&mut self, key: String) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO keys (key) VALUES (?)")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[key.as_str()][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }

    fn update_time(&mut self, _id: &str) -> Result<(), RecordStorageError> {
        unimplemented!();
    }

    fn delete(&mut self, key: &str) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("DELETE FROM keys WHERE key = ?")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[key][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
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
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<GameTypeStorage, RecordStorageError> {
        let mut storage = GameTypeStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }

    pub fn update_quantity(&mut self, game: &str, quantity: i64) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("UPDATE games SET quantity = ? WHERE game = ?")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[quantity.to_string().as_str(), game][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}

impl RecordStorage<GameTypeRecord, &str> for GameTypeStorage {
    fn refresh(&mut self) -> Result<(), RecordStorageError> {
        let rows = self.connection.prepare("SELECT * FROM games ORDER BY game")
            .map_err(|e| RecordStorageError::PreparedStatementError(e))?
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

    fn add(&mut self, record: GameTypeRecord) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO games (game, quantity) VALUES (?, ?)")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[record.game.as_str(), &record.quantity.to_string()][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }

    fn update_time(&mut self, _id: &str) -> Result<(), RecordStorageError> {
        unimplemented!();
    }

    fn delete(&mut self, id: &str) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("DELETE FROM games WHERE game = ?")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[id][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
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
    pub fn new(connection: Rc<sqlite::Connection>) -> Result<ItemTypeStorage, RecordStorageError> {
        let mut storage = ItemTypeStorage {
            connection,
            records: vec![],
        };

        storage.refresh()?;

        Ok(storage)
    }
}

impl RecordStorage<String, &str> for ItemTypeStorage {
    fn refresh(&mut self) -> Result<(), RecordStorageError> {
        let rows = self.connection.prepare("SELECT * FROM items ORDER BY item")
            .map_err(|e| RecordStorageError::PreparedStatementError(e))?
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

    fn add(&mut self, item: String) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("INSERT INTO items (item) VALUES (?)")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[item.as_str()][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }

    fn update_time(&mut self, _id: &str) -> Result<(), RecordStorageError> {
        unimplemented!();
    }

    fn delete(&mut self, item: &str) -> Result<(), RecordStorageError> {
        {
            let mut stmt = self.connection.prepare("DELETE FROM items WHERE item = ?")
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            stmt.bind(&[item][..])
                .map_err(|e| RecordStorageError::PreparedStatementError(e))?;
            
            while let Ok(sqlite::State::Row) = stmt.next() {}
        }

        self.refresh()?;
        
        Ok(())
    }
}
