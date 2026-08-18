use crate::models::DesktopEntry;
use crate::services::desktop_service;
use crate::services::i18n::t;
use crate::services::launcher_service::{self, LaunchOutcome};
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Orientation};
use libadwaita::{Toast, ToastOverlay};
use std::cell::RefCell;
use std::rc::Rc;

/// Build the bottom action bar for the editor with Run, Cancel, and Save buttons.
pub fn build_editor_footer(
    state: &Rc<RefCell<DesktopEntry>>,
    toast_overlay: &ToastOverlay,
    on_save: impl Fn(DesktopEntry) + 'static,
    on_cancel: impl Fn() + 'static,
) -> GtkBox {
    let footer = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(Align::End)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    let run_btn = Button::builder()
        .icon_name("media-playback-start-symbolic")
        .tooltip_text(&t("run"))
        .build();

    let cancel_btn = Button::builder().label(&t("cancel")).build();

    let save_btn = Button::builder()
        .label(&t("save"))
        .css_classes(["suggested-action"])
        .build();

    // Run action
    let state_run = state.clone();
    let toast_run = toast_overlay.clone();
    run_btn.connect_clicked(move |_| {
        let entry = state_run.borrow().clone();
        match launcher_service::launch_entry(&entry) {
            Ok(LaunchOutcome::UrlOpened(_)) => {
                toast_run.add_toast(Toast::new(&t("status_opening_url")));
            }
            Ok(LaunchOutcome::CommandLaunched(_)) => {
                toast_run.add_toast(Toast::new(&t("status_launched")));
            }
            Err(err) => {
                let err_msg = t("status_error").replace("{error}", &err.to_string());
                toast_run.add_toast(Toast::new(&err_msg));
            }
        }
    });

    // Cancel action
    let on_cancel_rc = Rc::new(on_cancel);
    cancel_btn.connect_clicked(move |_| {
        on_cancel_rc();
    });

    // Save action
    let state_save = state.clone();
    let toast_save = toast_overlay.clone();
    let on_save_rc = Rc::new(on_save);
    save_btn.connect_clicked(move |_| {
        let entry_to_save = state_save.borrow().clone();
        match desktop_service::save_entry(&entry_to_save) {
            Ok(target_path) => {
                state_save.borrow_mut().source_file = Some(target_path.clone());
                state_save.borrow_mut().directory = target_path.parent().map(|p| p.to_path_buf());
                toast_save.add_toast(Toast::new(&t("status_saved")));
                let saved = state_save.borrow().clone();
                on_save_rc(saved);
            }
            Err(err) => {
                let err_msg = t("status_error").replace("{error}", &err.to_string());
                toast_save.add_toast(Toast::new(&err_msg));
            }
        }
    });

    footer.append(&run_btn);
    footer.append(&cancel_btn);
    footer.append(&save_btn);

    footer
}
