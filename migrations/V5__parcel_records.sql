CREATE TABLE parcel_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parcel_desc VARCHAR(512) NOT NULL,
    student_name VARCHAR(512) NOT NULL,
    receptionist VARCHAR(512) NOT NULL,
    time_in VARCHAR(64) NOT NULL,
    time_out VARCHAR(64),
    notes VARCHAR(512) NOT NULL
)
