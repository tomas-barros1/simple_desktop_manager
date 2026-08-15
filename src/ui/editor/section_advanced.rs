use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::ui::components::{build_check, build_entry, build_labelled_row, build_section_header};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

/// Build the Advanced Options section (Startup Notify, WM Class, Hidden).
pub fn build_advanced_section(state: &Rc<RefCell<DesktopEntry>>) -> GtkBox {
    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();

    container.append(&build_section_header(&t("section_advanced")));

    let entry = state.borrow().clone();

    // Startup Notify
    let startup_label = t("startup_notify");
    let state_startup = state.clone();
    let startup_cb = build_check(entry.startup_notify, move |active| {
        state_startup.borrow_mut().startup_notify = active;
    });
    container.append(&build_labelled_row(&startup_label, &startup_cb));

    // Startup WM Class
    let wmclass_label = t("startup_wm_class");
    let wmclass_entry = build_entry(&entry.startup_wm_class);
    let state_wmclass = state.clone();
    wmclass_entry.connect_changed(move |e| {
        state_wmclass.borrow_mut().startup_wm_class = e.text().to_string();
    });
    container.append(&build_labelled_row(&wmclass_label, &wmclass_entry));

    // No Display (Hidden)
    let nodisp_label = t("no_display");
    let state_nodisp = state.clone();
    let nodisp_cb = build_check(entry.no_display, move |active| {
        state_nodisp.borrow_mut().no_display = active;
    });
    container.append(&build_labelled_row(&nodisp_label, &nodisp_cb));

    container
}
