use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::ui::components::{build_dropdown, build_entry, build_labelled_row};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Build the General properties form section (Type, Name, Generic Name, Comment).
pub fn build_general_section(state: &Rc<RefCell<DesktopEntry>>) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();

    let entry = state.borrow().clone();

    // Type
    let type_label = t("type");
    let type_dd = build_dropdown(&["Application", "Link", "Directory"], &entry.entry_type);
    let state_type = state.clone();
    type_dd.connect_selected_notify(move |dd| {
        if let Some(item) = dd.selected_item() {
            if let Ok(obj) = item.downcast::<gtk4::StringObject>() {
                state_type.borrow_mut().entry_type = obj.string().to_string();
            }
        }
    });
    container.append(&build_labelled_row(&type_label, &type_dd));

    // Name
    let name_label = t("name");
    let name_entry = build_entry(&entry.name);
    let state_name = state.clone();
    name_entry.connect_changed(move |e| {
        state_name.borrow_mut().name = e.text().to_string();
    });
    container.append(&build_labelled_row(&name_label, &name_entry));

    // Generic Name
    let generic_label = t("generic_name");
    let generic_entry = build_entry(&entry.generic_name);
    let state_generic = state.clone();
    generic_entry.connect_changed(move |e| {
        state_generic.borrow_mut().generic_name = e.text().to_string();
    });
    container.append(&build_labelled_row(&generic_label, &generic_entry));

    // Comment
    let comment_label = t("comment");
    let comment_entry = build_entry(&entry.comment);
    let state_comment = state.clone();
    comment_entry.connect_changed(move |e| {
        state_comment.borrow_mut().comment = e.text().to_string();
    });
    container.append(&build_labelled_row(&comment_label, &comment_entry));

    container
}
