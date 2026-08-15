use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::ui::components::{
    build_entry, build_labelled_row, build_path_row, build_section_header, parse_semicolon_list,
    BrowseMode,
};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Window};
use std::cell::RefCell;
use std::rc::Rc;

/// Build the Appearance & Classification section (Icon, Categories, Keywords, Mime Types).
pub fn build_appearance_section(parent: &Window, state: &Rc<RefCell<DesktopEntry>>) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();

    container.append(&build_section_header(&t("section_appearance")));

    let entry = state.borrow().clone();

    // Icon
    let icon_label = t("icon");
    let state_icon = state.clone();
    let icon_row = build_path_row(
        parent,
        &entry.icon,
        "image-x-generic-symbolic",
        BrowseMode::Icon,
        move |e| {
            state_icon.borrow_mut().icon = e.text().to_string();
        },
    );
    container.append(&build_labelled_row(&icon_label, &icon_row));

    // Categories
    let cat_label = t("categories");
    let cat_entry = build_entry(&entry.categories.join(";"));
    let state_cat = state.clone();
    cat_entry.connect_changed(move |e| {
        state_cat.borrow_mut().categories = parse_semicolon_list(&e.text());
    });
    container.append(&build_labelled_row(&cat_label, &cat_entry));

    // Keywords
    let kw_label = t("keywords");
    let kw_entry = build_entry(&entry.keywords.join(";"));
    let state_kw = state.clone();
    kw_entry.connect_changed(move |e| {
        state_kw.borrow_mut().keywords = parse_semicolon_list(&e.text());
    });
    container.append(&build_labelled_row(&kw_label, &kw_entry));

    // MIME Types
    let mime_label = t("mime_types");
    let mime_entry = build_entry(&entry.mime_types.join(";"));
    let state_mime = state.clone();
    mime_entry.connect_changed(move |e| {
        state_mime.borrow_mut().mime_types = parse_semicolon_list(&e.text());
    });
    container.append(&build_labelled_row(&mime_label, &mime_entry));

    container
}
