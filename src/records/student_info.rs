use chrono::Datelike;
use itertools::Itertools;

use super::StorageError;

pub struct Student {
    pub name: String,
    pub number: String,
}

pub struct StudentInfo {
    year_start: chrono::DateTime<chrono::Utc>,
    students: Vec<Student>,
}

impl StudentInfo {
    pub fn new(conn: &mut rusqlite::Connection) -> Result<StudentInfo, StorageError> {
        let mut students = StudentInfo {
            year_start: chrono::Utc::now(),
            students: vec![],
        };

        students.refresh(conn)?;

        Ok(students)
    }

    pub fn refresh(&mut self, conn: &mut rusqlite::Connection) -> Result<(), StorageError> {
        self.year_start = {
            let now = chrono::Local::now();

            let year_start = chrono::NaiveDate::from_ymd_opt(now.year(), 1, 1).unwrap()
                .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());

            let year_start = match year_start.and_local_timezone(chrono::Utc) {
                chrono::LocalResult::None => year_start.and_utc(),
                chrono::LocalResult::Single(d) => d,
                chrono::LocalResult::Ambiguous(a, _) => a,
            };

            year_start
        };
        
        let mut stmt = conn.prepare("
            SELECT student_number, student_name FROM (
                SELECT student_number, student_name, ROW_NUMBER() OVER(PARTITION BY student_number ORDER BY time_out DESC) rn FROM (
                    SELECT student_number, student_name, time_out FROM key_records WHERE time_out >= ?
                    UNION
                    SELECT student_number, student_name, time_out FROM item_records WHERE time_out >= ?
                    UNION
                    SELECT student_number, student_name, time_out FROM game_records WHERE time_out >= ?
                )
            ) WHERE rn = 1 ORDER BY student_number
        ")?;

        let year_start = self.year_start.to_rfc3339();
        
        let records = stmt.query_map((&year_start, &year_start, &year_start), |row| Ok(Student {
            name: row.get("student_name")?,
            number: row.get("student_number")?,
        }))?.collect::<Result<Vec<_>, _>>()?;

        self.students = records;

        Ok(())
    }
    
    pub fn get(&mut self) -> Result<&[Student], StorageError> {
        Ok(&self.students)
    }
}

