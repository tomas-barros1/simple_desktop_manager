use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::ui::components::{
    build_adw_entry_row, build_adw_path_row, build_preferences_group, parse_semicolon_list,
    BrowseMode,
};
use gtk4::Window;
use libadwaita::prelude::*;
use libadwaita::PreferencesGroup;
use std::cell::RefCell;
use std::rc::Rc;

/// Build the Appearance & Classification group (Icon, Categories, Keywords, Mime Types).
pub fn build_appearance_section(parent: &Window, state: &Rc<RefCell<DesktopEntry>>) -> PreferencesGroup {
    let group = build_preferences_group(&t("section_appearance"));
    let entry = state.borrow().clone();

    // Icon
    let state_icon = state.clone();
    let icon_row = build_adw_path_row(
        parent,
        &t("icon"),
        &entry.icon,
        "image-x-generic-symbolic",
        BrowseMode::Icon,
        move |val| {
            state_icon.borrow_mut().icon = val;
        },
    );
    group.add(&icon_row);

    // Categories
    let state_cat = state.clone();
    let cat_row = build_adw_entry_row(&t("categories"), &entry.categories.join(";"), move |val| {
        state_cat.borrow_mut().categories = parse_semicolon_list(&val);
    });
    group.add(&cat_row);

    // Keywords
    let state_kw = state.clone();
    let kw_row = build_adw_entry_row(&t("keywords"), &entry.keywords.join(";"), move |val| {
        state_kw.borrow_mut().keywords = parse_semicolon_list(&val);
    });
    group.add(&kw_row);

    // MIME Types
    let state_mime = state.clone();
    let mime_row = build_adw_entry_row(&t("mime_types"), &entry.mime_types.join(";"), move |val| {
        state_mime.borrow_mut().mime_types = parse_semicolon_list(&val);
    });
    group.add(&mime_row);

    group
}
