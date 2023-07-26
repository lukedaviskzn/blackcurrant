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
pub struct NewKeyRecord<'a> {
    pub key: &'a str,
    pub student_name: &'a str,
    pub student_number: &'a str,
    pub notes: &'a str,
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
pub struct NewParcelRecord<'a> {
    pub parcel_desc: &'a str,
    pub student_name: &'a str,
    pub receptionist: &'a str,
    pub notes: &'a str,
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
pub struct NewGameRecord<'a> {
    pub game: &'a str,
    pub quantity: i64,
    pub student_name: &'a str,
    pub student_number: &'a str,
    pub notes: &'a str,
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
pub struct NewItemRecord<'a> {
    pub item: &'a str,
    pub quantity: i64,
    pub student_name: &'a str,
    pub student_number: &'a str,
    pub receptionist: &'a str,
    pub notes: &'a str,
}

#[derive(Debug, Clone)]
pub struct GameTypeRecord {
    pub game: String,
    pub quantity: i64,
}

#[derive(Debug, Clone)]
pub struct NewGameTypeRecord<'a> {
    pub game: &'a str,
    pub quantity: i64,
}