use std::sync::{Arc, Mutex};

use crate::{app::{STUDENT_NUMBER_LENGTH, STAFF_NUMBER_LENGTH}, records::StudentInfo};

pub mod key_sign;
pub mod parcel_sign;
pub mod game_sign;
pub mod item_sign;

pub mod key_entry;
pub mod game_entry;
pub mod item_entry;

pub mod sign_in;
pub mod exit;
pub mod alert;
pub mod export;
pub mod about;
pub mod settings;
pub mod confirmation;
pub mod summary;

pub use key_sign::*;
pub use parcel_sign::*;
pub use game_sign::*;
pub use item_sign::*;

pub use key_entry::*;
pub use game_entry::*;
pub use item_entry::*;

pub use sign_in::*;
pub use exit::*;
pub use alert::*;
pub use export::*;
pub use about::*;
pub use settings::*;
pub use confirmation::*;
pub use summary::*;

fn render_modal_text_entry(ui: &mut egui::Ui, label: &str, error: &Option<String>, input: &mut String, max_length: usize) -> egui::Response {
    ui.label(label);
    
    let response = ui.add(egui::TextEdit::singleline(input).char_limit(max_length));
    
    if let Some(error) = error {
        ui.colored_label(egui::Rgba::from_rgb(0.25, 0.0, 0.0), error);
    }

    ui.add_space(4.0);

    response
}

fn render_student_number_popup(ui: &mut egui::Ui, student_info: Arc<Mutex<StudentInfo>>, popup_id: egui::Id, resp: &egui::Response, student_number: &mut String, student_name: &mut String) {
    let mut info = student_info.lock().unwrap();
    let students = info.get().unwrap().into_iter()
            .filter(|s| s.number.contains(&student_number.to_uppercase())).collect::<Vec<_>>();

    let show_popup = student_number.len() > 0 && resp.has_focus() && students.len() > 0;

    let mut set_student = None;

    egui::popup_below_widget(ui, popup_id, &resp, |ui| {
        for i in 0..(students.len().min(10)) {
            let student = students[i];
            if ui.selectable_label(false, format!("{} – {}", student.number, student.name)).clicked() {
                set_student = Some((student.number.clone(), student.name.clone()));
            }
        }
    });

    if let Some((number, name)) = set_student {
        *student_number = number;
        *student_name = name;
    }

    if show_popup {
        ui.memory_mut(|mem| mem.open_popup(popup_id))
    } else {
        ui.memory_mut(|mem| {
            if mem.is_popup_open(popup_id) {
                mem.close_popup()
            }
        });
    }
}

fn filter_student_number(student_number: &str, error_text: &mut Option<String>) -> bool {
    match student_number.parse::<i64>() {
        // Maybe Staff Number
        Ok(_) => {
            if student_number.len() != STAFF_NUMBER_LENGTH {
                *error_text = Some("Invalid staff number.".into());
            }
        },
        // Maybe Student Number
        Err(_) => {
            if student_number.len() != STUDENT_NUMBER_LENGTH ||
                    !student_number.chars().enumerate().all(|(i, c)| i < 6 && c.is_ascii_alphabetic() || i >= 6 && c.is_ascii_digit()) {
                *error_text = Some("Invalid student number.".into());
            }
        },
    }

    error_text.is_some()
}

fn filter_required(value: &str, error_text: &mut Option<String>) -> bool {
    if value.len() == 0 {
        *error_text = Some("Required".into());
        true
    } else {
        false
    }
}

fn filter_length(value: &str, max: usize, error_text: &mut Option<String>) -> bool {
    if value.len() > max {
        *error_text = Some(format!("Too long. (> {max} characters)"));
        true
    } else {
        false
    }
}
