CREATE TABLE item_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item VARCHAR(512) NOT NULL,
    quantity INTEGER NOT NULL,
    student_name VARCHAR(512) NOT NULL,
    student_number VARCHAR(9) NOT NULL,
    receptionist VARCHAR(512) NOT NULL,
    time_out VARCHAR(64) NOT NULL,
    notes VARCHAR(512) NOT NULL
)
