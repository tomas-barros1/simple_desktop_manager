use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::ui::components::{build_adw_combo_row, build_adw_entry_row, build_preferences_group};
use libadwaita::prelude::*;
use libadwaita::PreferencesGroup;
use std::cell::RefCell;
use std::rc::Rc;

/// Build the General properties form group (Type, Name, Generic Name, Comment).
pub fn build_general_section(state: &Rc<RefCell<DesktopEntry>>) -> PreferencesGroup {
    let group = build_preferences_group(&t("section_general"));
    let entry = state.borrow().clone();

    // Type
    let state_type = state.clone();
    let type_row = build_adw_combo_row(
        &t("type"),
        &["Application", "Link", "Directory"],
        &entry.entry_type,
        move |val| {
            state_type.borrow_mut().entry_type = val;
        },
    );
    group.add(&type_row);

    // Name
    let state_name = state.clone();
    let name_row = build_adw_entry_row(&t("name"), &entry.name, move |val| {
        state_name.borrow_mut().name = val;
    });
    group.add(&name_row);

    // Generic Name
    let state_generic = state.clone();
    let generic_row = build_adw_entry_row(&t("generic_name"), &entry.generic_name, move |val| {
        state_generic.borrow_mut().generic_name = val;
    });
    group.add(&generic_row);

    // Comment
    let state_comment = state.clone();
    let comment_row = build_adw_entry_row(&t("comment"), &entry.comment, move |val| {
        state_comment.borrow_mut().comment = val;
    });
    group.add(&comment_row);

    group
}
