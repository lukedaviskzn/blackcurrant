use crate::app::{STUDENT_NUMBER_LENGTH, STAFF_NUMBER_LENGTH};

pub mod key_sign_modal;
pub mod parcel_sign_modal;
pub mod game_sign_modal;
pub mod item_sign_modal;

pub mod key_entry_modal;
pub mod game_entry_modal;
pub mod item_entry_modal;

pub mod sign_in_modal;
pub mod exit_modal;
pub mod alert_modal;
pub mod export_modal;
pub mod about_modal;
pub mod settings_modal;
pub mod confirmation_modal;

pub use key_sign_modal::*;
pub use parcel_sign_modal::*;
pub use game_sign_modal::*;
pub use item_sign_modal::*;

pub use key_entry_modal::*;
pub use game_entry_modal::*;
pub use item_entry_modal::*;

pub use sign_in_modal::*;
pub use exit_modal::*;
pub use alert_modal::*;
pub use export_modal::*;
pub use about_modal::*;
pub use settings_modal::*;
pub use confirmation_modal::*;

fn render_modal_text_entry(ui: &mut egui::Ui, label: &str, error: &Option<String>, input: &mut String, max_length: usize) {
    ui.label(label);
    ui.add(egui::TextEdit::singleline(input).char_limit(max_length));
    
    if let Some(error) = error {
        ui.colored_label(egui::Rgba::from_rgb(0.25, 0.0, 0.0), error);
    }

    ui.add_space(4.0);
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
