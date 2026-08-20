use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::ui::components::{
    build_adw_entry_row, build_adw_icon_row, build_preferences_group, parse_semicolon_list,
};
use gtk4::prelude::*;
use gtk4::{Align, CheckButton, Window};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, ExpanderRow, PreferencesGroup};
use std::cell::RefCell;
use std::rc::Rc;

const STANDARD_CATEGORIES: &[(&str, &str)] = &[
    ("AudioVideo", "category_audiovideo"),
    ("Development", "category_development"),
    ("Education", "category_education"),
    ("Game", "category_game"),
    ("Graphics", "category_graphics"),
    ("Network", "category_network"),
    ("Office", "category_office"),
    ("Science", "category_science"),
    ("Settings", "category_settings"),
    ("System", "category_system"),
    ("Utility", "category_utility"),
];

/// Build the Appearance & Classification group (Icon, Categories Expander with Checkboxes, Keywords, Mime Types).
pub fn build_appearance_section(parent: &Window, state: &Rc<RefCell<DesktopEntry>>) -> PreferencesGroup {
    let group = build_preferences_group(&t("section_appearance"));
    let entry = state.borrow().clone();

    // Icon (with live preview prefix and browse dialog suffix)
    let state_icon = state.clone();
    let icon_row = build_adw_icon_row(
        parent,
        &t("icon"),
        &entry.icon,
        move |val| {
            state_icon.borrow_mut().icon = val;
        },
    );
    group.add(&icon_row);

    // Categories Expander with Checkboxes
    let cat_expander = build_categories_expander(state);
    group.add(&cat_expander);

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

fn format_categories_summary(categories: &[String]) -> String {
    if categories.is_empty() {
        t("categories_none")
    } else {
        categories
            .iter()
            .map(|c| {
                for (std_cat, i18n_key) in STANDARD_CATEGORIES {
                    if c.eq_ignore_ascii_case(std_cat) {
                        return t(i18n_key);
                    }
                }
                c.clone()
            })
            .collect::<Vec<String>>()
            .join(", ")
    }
}

fn build_categories_expander(state: &Rc<RefCell<DesktopEntry>>) -> ExpanderRow {
    let entry = state.borrow().clone();
    let expander = ExpanderRow::builder()
        .title(&t("categories"))
        .subtitle(&format_categories_summary(&entry.categories))
        .build();

    let mut custom_cats: Vec<String> = Vec::new();
    for c in &entry.categories {
        let is_std = STANDARD_CATEGORIES
            .iter()
            .any(|(std_cat, _)| c.eq_ignore_ascii_case(std_cat));
        if !is_std {
            custom_cats.push(c.clone());
        }
    }

    let custom_cats_cell = Rc::new(RefCell::new(custom_cats.clone()));

    for &(cat_name, i18n_key) in STANDARD_CATEGORIES {
        let is_active = entry
            .categories
            .iter()
            .any(|c| c.eq_ignore_ascii_case(cat_name));

        let check = CheckButton::builder()
            .active(is_active)
            .valign(Align::Center)
            .build();

        let row = ActionRow::builder()
            .title(&t(i18n_key))
            .subtitle(cat_name)
            .activatable_widget(&check)
            .build();

        row.add_suffix(&check);

        let state_cb = state.clone();
        let expander_cb = expander.clone();
        let cat_str = cat_name.to_string();

        check.connect_toggled(move |btn| {
            let active = btn.is_active();
            let mut current = state_cb.borrow().categories.clone();

            if active {
                if !current.iter().any(|c| c.eq_ignore_ascii_case(&cat_str)) {
                    current.push(cat_str.clone());
                }
            } else {
                current.retain(|c| !c.eq_ignore_ascii_case(&cat_str));
            }

            state_cb.borrow_mut().categories = current.clone();
            expander_cb.set_subtitle(&format_categories_summary(&current));
        });

        expander.add_row(&row);
    }

    // Custom categories entry row
    let state_custom = state.clone();
    let expander_custom = expander.clone();
    let custom_holder = custom_cats_cell;
    let custom_row = build_adw_entry_row(
        &t("category_other"),
        &custom_cats.join(";"),
        move |val| {
            let new_custom = parse_semicolon_list(&val);
            *custom_holder.borrow_mut() = new_custom.clone();

            // Rebuild categories = (all active standard categories) + (new custom)
            let current = state_custom.borrow().categories.clone();
            let mut updated: Vec<String> = Vec::new();

            for &(std_cat, _) in STANDARD_CATEGORIES {
                if current.iter().any(|c| c.eq_ignore_ascii_case(std_cat)) {
                    updated.push(std_cat.to_string());
                }
            }
            for c in new_custom {
                if !updated.iter().any(|u| u.eq_ignore_ascii_case(&c)) {
                    updated.push(c);
                }
            }

            state_custom.borrow_mut().categories = updated.clone();
            expander_custom.set_subtitle(&format_categories_summary(&updated));
        },
    );

    expander.add_row(&custom_row);
    expander
}
