use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::ui::components::{
    build_adw_entry_row, build_adw_path_row, build_adw_switch_row, build_preferences_group,
    BrowseMode,
};
use gtk4::Window;
use libadwaita::prelude::*;
use libadwaita::PreferencesGroup;
use std::cell::RefCell;
use std::rc::Rc;

/// Build the Execution form group (Exec, Path, Terminal, URL).
pub fn build_exec_section(parent: &Window, state: &Rc<RefCell<DesktopEntry>>) -> PreferencesGroup {
    let group = build_preferences_group(&t("section_execution"));
    let entry = state.borrow().clone();

    // Path
    let state_path = state.clone();
    let path_row = build_adw_path_row(
        parent,
        &t("path"),
        &entry.path,
        "folder-open-symbolic",
        BrowseMode::Folder,
        move |val| {
            state_path.borrow_mut().path = val;
        },
    );

    // Exec
    let state_exec = state.clone();
    let path_row_clone = path_row.clone();
    let exec_row = build_adw_path_row(
        parent,
        &t("exec"),
        &entry.exec,
        "document-open-symbolic",
        BrowseMode::File,
        move |val| {
            state_exec.borrow_mut().exec = val.clone();
            // Auto-populate Path with binary's directory if Path is currently empty
            if state_exec.borrow().path.trim().is_empty() {
                let clean_val = val.trim_matches('"').trim_matches('\'');
                if let Some(parent) = std::path::Path::new(clean_val).parent() {
                    let parent_str = parent.to_string_lossy().to_string();
                    if !parent_str.is_empty() {
                        state_exec.borrow_mut().path = parent_str.clone();
                        path_row_clone.set_text(&parent_str);
                    }
                }
            }
        },
    );
    group.add(&exec_row);
    group.add(&path_row);

    // Terminal
    let state_term = state.clone();
    let term_row = build_adw_switch_row(&t("run_in_terminal"), entry.terminal, move |active| {
        state_term.borrow_mut().terminal = active;
    });
    group.add(&term_row);

    // URL
    let state_url = state.clone();
    let url_row = build_adw_entry_row(&t("url"), &entry.url, move |val| {
        state_url.borrow_mut().url = val;
    });
    group.add(&url_row);

    group
}
