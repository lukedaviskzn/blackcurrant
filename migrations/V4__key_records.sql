CREATE TABLE key_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    `key` VARCHAR(512) NOT NULL,
    student_name VARCHAR(512) NOT NULL,
    student_number VARCHAR(9) NOT NULL,
    receptionist VARCHAR(512),
    time_out VARCHAR(64) NOT NULL,
    time_in VARCHAR(64),
    notes VARCHAR(512) NOT NULL
)
