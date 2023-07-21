pub mod key_panel;
pub mod parcel_panel;
pub mod game_panel;
pub mod item_panel;

pub use key_panel::*;
pub use parcel_panel::*;
pub use game_panel::*;
pub use item_panel::*;

use crate::{records::Page, app::{PAGE_SIZE, NOTES_MAX_LENGTH}};

const PENCIL_ICON: &str = "\u{f303}";

fn pagination(ui: &mut eframe::egui::Ui, page: &mut Page, count: i64) {
    let max_page = (count - 1) / PAGE_SIZE;

    let mut page_num = page.as_i64(count);
    
    // skip to end
    if ui.add_enabled(page_num < max_page, egui::Button::new(">>")).clicked() {
        page_num = max_page;
    }
    
    // next page
    if ui.add_enabled(page_num < max_page, egui::Button::new(">")).clicked() {
        page_num += 1;
    }
    
    // current page text
    ui.label(format!("{}", page_num+1));
    
    // previous page
    if ui.add_enabled(page_num > 0, egui::Button::new("<")).clicked() {
        page_num -= 1;
    }
    
    // skip to start
    if ui.add_enabled(page_num > 0, egui::Button::new("<<")).clicked() {
        page_num = 0;
    }

    *page = if page_num >= max_page {
        Page::LastPage
    } else {
        Page::Page(page_num.clamp(0, max_page))
    };
}

fn render_notes_entry(ui: &mut egui::Ui, record_id: i64, record_notes: &str, current_notes: &mut Option<(i64, String)>) -> Option<(i64, String)> {
    let mut update_notes = None;
    
    // Are we currently editing this record?
    let editing = if let Some((id, _)) = current_notes {
        *id == record_id
    } else {
        false
    };

    let response = ui.horizontal(|ui| {
        if editing {
            if let Some((_, notes)) = current_notes {
                // lost focus not guaranteed to be triggered, since text edit removed in same frame
                if ui.add(egui::TextEdit::singleline(notes).char_limit(NOTES_MAX_LENGTH)).lost_focus() {
                    update_notes = current_notes.take();
                }
            }
        } else {
            if ui.button(egui::RichText::new(PENCIL_ICON).family(egui::FontFamily::Name("icons".into()))).clicked() {
                // if currently editing something, update now before switching
                update_notes = current_notes.take();
                *current_notes = Some((record_id, record_notes.into()));
            }

            ui.label(record_notes);
        }
    }).response;

    // Hide when clicked elsewhere
    if editing && response.clicked_elsewhere() && update_notes.is_none() {
        update_notes = current_notes.take();
    }

    update_notes
}
