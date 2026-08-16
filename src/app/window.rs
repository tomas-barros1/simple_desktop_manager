use crate::models::DesktopEntry;
use crate::services::desktop_service;
use crate::services::i18n::t;
use crate::services::monitor_service::watch_directories;
use crate::ui::editor::Editor;
use crate::ui::sidebar::Sidebar;
use gtk4::gio::FileMonitor;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, HeaderBar, Orientation, Paned, Stack};
use libadwaita::prelude::*;
use libadwaita::{ApplicationWindow, MessageDialog, ResponseAppearance};
use std::cell::RefCell;
use std::rc::Rc;

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

/// Main application window: split-view layout of sidebar + editor with live filesystem sync.
pub struct Window {
    pub widget: ApplicationWindow,
    _monitors: Vec<FileMonitor>,
}

impl Window {
    pub fn new(app: &libadwaita::Application) -> Self {
        init_css_provider();

        let win = ApplicationWindow::builder()
            .application(app)
            .title(&t("app_title"))
            .default_width(1020)
            .default_height(680)
            .build();

        let header_bar = HeaderBar::new();
        let entries = desktop_service::load_all();
        let sidebar = Rc::new(RefCell::new(Sidebar::new(entries)));
        let content_stack = build_content_stack();

        let paned = Paned::builder()
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
        win.set_content(Some(&main_box));

        let selected_entry: Rc<RefCell<Option<DesktopEntry>>> = Rc::new(RefCell::new(None));

        // Connect application event handlers and actions
        wire_window_events(
            &win,
            &sidebar,
            &content_stack,
            &selected_entry,
        );

        // Watch XDG application directories for real-time filesystem synchronization
        let monitors = setup_live_file_monitoring(
            &sidebar,
            &content_stack,
            &selected_entry,
        );

        Self {
            widget: win,
            _monitors: monitors,
        }
    }

    pub fn present(&self) {
        self.widget.present();
    }
}

fn init_css_provider() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(CSS_STYLE);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_content_stack() -> Stack {
    let placeholder = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .hexpand(true)
        .vexpand(true)
        .margin_top(48)
        .build();

    let hint = gtk4::Label::builder()
        .label(&t("select_entry_hint"))
        .css_classes(["dim-label", "title"])
        .halign(Align::Center)
        .build();
    placeholder.append(&hint);

    let stack = Stack::builder().build();
    stack.add_named(&placeholder, Some("empty"));
    stack.set_visible_child_name("empty");
    stack
}

fn wire_window_events(
    win: &ApplicationWindow,
    sidebar: &Rc<RefCell<Sidebar>>,
    content_stack: &Stack,
    selected_entry: &Rc<RefCell<Option<DesktopEntry>>>,
) {
    let win_dialog = win.clone();
    let stack_del = content_stack.clone();
    let sidebar_del = sidebar.clone();
    let selected_del = selected_entry.clone();

    // Delete handler
    let show_delete_confirmation = move |entry: DesktopEntry| {
        if entry.source_file.is_none() {
            *selected_del.borrow_mut() = None;
            sidebar_del.borrow().delete_btn.set_sensitive(false);
            if let Some(existing) = stack_del.child_by_name("editor") {
                stack_del.remove(&existing);
            }
            stack_del.set_visible_child_name("empty");
            return;
        }

        let entry_name = if entry.name.trim().is_empty() {
            entry.suggested_filename()
        } else {
            entry.name.clone()
        };

        let dialog = MessageDialog::builder()
            .transient_for(&win_dialog)
            .heading(&t("confirm_delete_title"))
            .body(&t("confirm_delete_body").replace("{name}", &entry_name))
            .build();

        dialog.add_response("cancel", &t("cancel"));
        dialog.add_response("delete", &t("delete_entry"));
        dialog.set_response_appearance("delete", ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));

        let win_err = win_dialog.clone();
        let stack_resp = stack_del.clone();
        let sidebar_resp = sidebar_del.clone();
        let selected_resp = selected_del.clone();
        let entry_to_delete = entry.clone();

        dialog.connect_response(None, move |_, response| {
            if response == "delete" {
                match desktop_service::delete_entry(&entry_to_delete) {
                    Ok(_) => {
                        *selected_resp.borrow_mut() = None;
                        sidebar_resp.borrow().delete_btn.set_sensitive(false);
                        let fresh = desktop_service::load_all();
                        sidebar_resp.borrow().refresh(fresh);
                        if let Some(existing) = stack_resp.child_by_name("editor") {
                            stack_resp.remove(&existing);
                        }
                        stack_resp.set_visible_child_name("empty");
                    }
                    Err(err) => {
                        show_error_dialog(&win_err, &err.to_string());
                    }
                }
            }
        });
        dialog.present();
    };

    // Save handler
    let sidebar_save = sidebar.clone();
    let selected_save = selected_entry.clone();
    let handle_save = move |saved_entry: DesktopEntry| {
        *selected_save.borrow_mut() = Some(saved_entry.clone());
        sidebar_save.borrow().delete_btn.set_sensitive(true);
        let fresh = desktop_service::load_all();
        sidebar_save.borrow().refresh(fresh);
        if let Some(src) = &saved_entry.source_file {
            sidebar_save.borrow().select_entry_by_path(src);
        }
    };

    // Open/Reload Editor closure
    let open_editor_holder: Rc<RefCell<Option<Box<dyn Fn(DesktopEntry)>>>> =
        Rc::new(RefCell::new(None));

    {
        let win_editor = win.clone();
        let stack_editor = content_stack.clone();
        let sidebar_editor = sidebar.clone();
        let selected_editor = selected_entry.clone();
        let save_fn = handle_save.clone();
        let open_holder_inner = open_editor_holder.clone();

        *open_editor_holder.borrow_mut() = Some(Box::new(move |entry: DesktopEntry| {
            let is_new = entry.source_file.is_none();
            *selected_editor.borrow_mut() = Some(entry.clone());
            sidebar_editor.borrow().delete_btn.set_sensitive(!is_new);

            let on_save = save_fn.clone();
            let cancel_entry = entry.clone();
            let open_again = open_holder_inner.clone();
            let stack_cancel = stack_editor.clone();
            let sel_cancel = selected_editor.clone();
            let side_cancel = sidebar_editor.clone();

            let editor = Editor::new(
                &win_editor,
                entry,
                move |e| on_save(e),
                move || {
                    if cancel_entry.source_file.is_none() {
                        *sel_cancel.borrow_mut() = None;
                        side_cancel.borrow().delete_btn.set_sensitive(false);
                        if let Some(existing) = stack_cancel.child_by_name("editor") {
                            stack_cancel.remove(&existing);
                        }
                        stack_cancel.set_visible_child_name("empty");
                    } else if let Some(open_fn) = open_again.borrow().as_ref() {
                        open_fn(cancel_entry.clone());
                    }
                },
            );

            if let Some(existing) = stack_editor.child_by_name("editor") {
                stack_editor.remove(&existing);
            }
            stack_editor.add_named(&editor.root, Some("editor"));
            stack_editor.set_visible_child_name("editor");
        }));
    }

    // Sidebar selection handler
    let sidebar_sel_cb = sidebar.clone();
    let selected_sel_cb = selected_entry.clone();
    let open_holder_sel = open_editor_holder.clone();
    sidebar.borrow().selection.connect_selected_notify(move |sel| {
        let Some(item) = sel.selected_item() else { return };
        let Some(string_obj) = item.downcast_ref::<gtk4::StringObject>() else { return };
        let payload = string_obj.string().to_string();
        let idx: usize = payload
            .rsplit_once(Sidebar::SEP)
            .map(|(_, i)| i.parse().unwrap_or(usize::MAX))
            .unwrap_or(usize::MAX);
        if let Some(entry) = sidebar_sel_cb.borrow().get_entry(idx) {
            let already_open = selected_sel_cb
                .borrow()
                .as_ref()
                .map(|cur| cur.source_file.is_some() && cur.source_file == entry.source_file)
                .unwrap_or(false);
            if !already_open {
                if let Some(open_fn) = open_holder_sel.borrow().as_ref() {
                    open_fn(entry);
                }
            }
        }
    });

    // Add button handler
    let sidebar_add = sidebar.clone();
    let open_holder_add = open_editor_holder.clone();
    sidebar.borrow().add_btn.connect_clicked(move |_| {
        sidebar_add.borrow().clear_selection();
        if let Some(open_fn) = open_holder_add.borrow().as_ref() {
            open_fn(DesktopEntry::default());
        }
    });

    // Delete button handler
    let selected_del_btn = selected_entry.clone();
    let confirm_del_sidebar = show_delete_confirmation;
    sidebar.borrow().delete_btn.connect_clicked(move |_| {
        let entry_opt = selected_del_btn.borrow().clone();
        if let Some(entry) = entry_opt {
            confirm_del_sidebar(entry);
        }
    });
}

fn show_error_dialog(parent: &ApplicationWindow, message: &str) {
    let err_dialog = MessageDialog::builder()
        .transient_for(parent)
        .heading("Error")
        .body(message)
        .build();
    err_dialog.add_response("ok", "OK");
    err_dialog.present();
}

fn setup_live_file_monitoring(
    sidebar: &Rc<RefCell<Sidebar>>,
    content_stack: &Stack,
    selected_entry: &Rc<RefCell<Option<DesktopEntry>>>,
) -> Vec<FileMonitor> {
    let sidebar_mon = sidebar.clone();
    let selected_mon = selected_entry.clone();
    let stack_mon = content_stack.clone();

    let search_dirs = desktop_service::search_paths();
    watch_directories(&search_dirs, 300, move || {
        let fresh = desktop_service::load_all();
        sidebar_mon.borrow().refresh(fresh);

        // If the currently selected desktop entry file was deleted externally, reset view
        if let Some(sel) = selected_mon.borrow().clone() {
            if let Some(src) = &sel.source_file {
                if !src.exists() {
                    *selected_mon.borrow_mut() = None;
                    sidebar_mon.borrow().delete_btn.set_sensitive(false);
                    if let Some(existing) = stack_mon.child_by_name("editor") {
                        stack_mon.remove(&existing);
                    }
                    stack_mon.set_visible_child_name("empty");
                } else {
                    sidebar_mon.borrow().select_entry_by_path(src);
                }
            }
        }
    })
}
