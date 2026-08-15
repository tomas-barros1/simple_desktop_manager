use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::ui::components::{
    build_check, build_entry, build_labelled_row, build_path_row, build_section_header, BrowseMode,
};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Window};
use std::cell::RefCell;
use std::rc::Rc;

/// Build the Execution form section (Exec, Path, Terminal, URL).
pub fn build_exec_section(parent: &Window, state: &Rc<RefCell<DesktopEntry>>) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();

    container.append(&build_section_header(&t("section_execution")));

    let entry = state.borrow().clone();

    // Exec
    let exec_label = t("exec");
    let state_exec = state.clone();
    let exec_row = build_path_row(
        parent,
        &entry.exec,
        "document-open-symbolic",
        BrowseMode::File,
        move |e| {
            state_exec.borrow_mut().exec = e.text().to_string();
        },
    );
    container.append(&build_labelled_row(&exec_label, &exec_row));

    // Path
    let path_label = t("path");
    let state_path = state.clone();
    let path_row = build_path_row(
        parent,
        &entry.path,
        "folder-open-symbolic",
        BrowseMode::Folder,
        move |e| {
            state_path.borrow_mut().path = e.text().to_string();
        },
    );
    container.append(&build_labelled_row(&path_label, &path_row));

    // Terminal
    let term_label = t("run_in_terminal");
    let state_term = state.clone();
    let term_cb = build_check(entry.terminal, move |active| {
        state_term.borrow_mut().terminal = active;
    });
    container.append(&build_labelled_row(&term_label, &term_cb));

    // URL
    let url_label = t("url");
    let url_entry = build_entry(&entry.url);
    let state_url = state.clone();
    url_entry.connect_changed(move |e| {
        state_url.borrow_mut().url = e.text().to_string();
    });
    container.append(&build_labelled_row(&url_label, &url_entry));

    container
}
