use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::ui::components::{build_adw_entry_row, build_adw_switch_row, build_preferences_group};
use libadwaita::prelude::*;
use libadwaita::PreferencesGroup;
use std::cell::RefCell;
use std::rc::Rc;

/// Build the Advanced Options group (Startup Notify, WM Class, Hidden).
pub fn build_advanced_section(state: &Rc<RefCell<DesktopEntry>>) -> PreferencesGroup {
    let group = build_preferences_group(&t("section_advanced"));
    let entry = state.borrow().clone();

    // Startup Notify
    let state_startup = state.clone();
    let startup_row = build_adw_switch_row(&t("startup_notify"), entry.startup_notify, move |active| {
        state_startup.borrow_mut().startup_notify = active;
    });
    group.add(&startup_row);

    // Startup WM Class
    let state_wmclass = state.clone();
    let wmclass_row = build_adw_entry_row(&t("startup_wm_class"), &entry.startup_wm_class, move |val| {
        state_wmclass.borrow_mut().startup_wm_class = val;
    });
    group.add(&wmclass_row);

    // No Display (Hidden)
    let state_nodisp = state.clone();
    let nodisp_row = build_adw_switch_row(&t("no_display"), entry.no_display, move |active| {
        state_nodisp.borrow_mut().no_display = active;
    });
    group.add(&nodisp_row);

    group
}
