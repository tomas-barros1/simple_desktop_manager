use crate::models::DesktopEntry;
use crate::services::desktop_service;
use crate::services::i18n::t;
use crate::ui::editor::Editor;
use crate::ui::sidebar::Sidebar;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, HeaderBar, Orientation};
use libadwaita::prelude::*;
use libadwaita::{ApplicationWindow, MessageDialog, ResponseAppearance};
use std::cell::RefCell;
use std::rc::Rc;

/// Structural layout CSS, allowing GTK / LibAdwaita system themes to define colors.
const CSS_STYLE: &str = r#"
.sidebar-row-title {
    font-weight: bold;
    font-size: 13px;
}

.sidebar-row-subtitle {
    font-size: 11px;
}

.section-header {
    font-size: 15px;
    font-weight: bold;
}

.field-label {
    font-size: 13px;
    font-weight: 500;
}
"#;

/// Main application window: split-view layout of sidebar + editor.
pub struct Window {
    pub widget: ApplicationWindow,
}

impl Window {
    pub fn new(app: &libadwaita::Application) -> Self {
        let provider = gtk4::CssProvider::new();
        provider.load_from_data(CSS_STYLE);
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let win = ApplicationWindow::builder()
            .application(app)
            .title(&t("app_title"))
            .default_width(1020)
            .default_height(680)
            .build();

        let header_bar = HeaderBar::new();

        let entries = desktop_service::load_all();
        let sidebar = Rc::new(RefCell::new(Sidebar::new(entries.clone())));
        let placeholder = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .margin_top(48)
            .build();
        let hint_label = t("select_entry_hint");
        let hint = gtk4::Label::builder()
            .label(&hint_label)
            .css_classes(["dim-label", "title"])
            .halign(Align::Center)
            .build();
        placeholder.append(&hint);

        let content_stack = gtk4::Stack::builder().build();
        content_stack.add_named(&placeholder, Some("empty"));
        content_stack.set_visible_child_name("empty");

        let paned = gtk4::Paned::builder()
            .wide_handle(true)
            .start_child(&sidebar.borrow().root)
            .end_child(&content_stack)
            .shrink_start_child(false)
            .shrink_end_child(false)
            .position(320)
            .build();

        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .build();
        main_box.append(&header_bar);
        main_box.append(&paned);

        let store_entries: Rc<RefCell<Vec<DesktopEntry>>> = Rc::new(RefCell::new(entries.clone()));
        let selected_entry: Rc<RefCell<Option<DesktopEntry>>> = Rc::new(RefCell::new(None));

        // Shared function to present the Delete Confirmation Popup
        let win_dialog = win.clone();
        let stack_del = content_stack.clone();
        let show_delete_confirmation = Rc::new(move |entry: DesktopEntry| {
            let dialog = MessageDialog::builder()
                .transient_for(&win_dialog)
                .heading(&t("confirm_delete_title"))
                .body(&t("confirm_delete_body").replace("{name}", &entry.name))
                .build();

            dialog.add_response("cancel", &t("cancel"));
            dialog.add_response("delete", &t("delete_entry"));
            dialog.set_response_appearance("delete", ResponseAppearance::Destructive);
            dialog.set_default_response(Some("cancel"));

            let stack_resp = stack_del.clone();
            let entry_to_delete = entry.clone();
            dialog.connect_response(None, move |_, response| {
                if response == "delete" {
                    let _ = desktop_service::delete_entry(&entry_to_delete);
                    if let Some(existing) = stack_resp.child_by_name("editor") {
                        stack_resp.remove(&existing);
                    }
                    stack_resp.set_visible_child_name("empty");
                }
            });
            dialog.present();
        });

        // Single-click selection handler
        let stack_clone2 = content_stack.clone();
        let store_clone = store_entries.clone();
        let selected_clone = selected_entry.clone();

        sidebar.borrow().selection.connect_selected_notify(move |sel| {
            let Some(item) = sel.selected_item() else { return };
            let Some(string_obj) = item.downcast_ref::<gtk4::StringObject>() else { return };
            let payload = string_obj.string().to_string();
            let idx: usize = payload
                .rsplit_once(Sidebar::SEP)
                .map(|(_, i)| i.parse().unwrap_or(0))
                .unwrap_or(0);
            if let Some(entry) = store_clone.borrow().get(idx).cloned() {
                *selected_clone.borrow_mut() = Some(entry.clone());
                let editor = Editor::new(entry);
                if let Some(existing) = stack_clone2.child_by_name("editor") {
                    stack_clone2.remove(&existing);
                }
                stack_clone2.add_named(&editor.root, Some("editor"));
                stack_clone2.set_visible_child_name("editor");
            }
        });

        // Add button action ('+' in sidebar bottom bar)
        let stack_clone_new = content_stack.clone();
        let selected_new = selected_entry.clone();
        sidebar.borrow().add_btn.connect_clicked(move |_| {
            let new_entry = DesktopEntry::default();
            *selected_new.borrow_mut() = Some(new_entry.clone());
            let editor = Editor::new(new_entry);
            if let Some(existing) = stack_clone_new.child_by_name("editor") {
                stack_clone_new.remove(&existing);
            }
            stack_clone_new.add_named(&editor.root, Some("editor"));
            stack_clone_new.set_visible_child_name("editor");
        });

        // Sidebar Delete button action
        let selected_del = selected_entry.clone();
        let confirm_delete_sidebar = show_delete_confirmation.clone();
        sidebar.borrow().delete_btn.connect_clicked(move |_| {
            let entry_opt = selected_del.borrow().clone();
            if let Some(entry) = entry_opt {
                confirm_delete_sidebar(entry);
            }
        });

        win.set_content(Some(&main_box));
        Self { widget: win }
    }

    pub fn present(&self) {
        self.widget.present();
    }
}
