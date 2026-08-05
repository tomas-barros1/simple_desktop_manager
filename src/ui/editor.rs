use crate::models::DesktopEntry;
use crate::services::desktop_service;
use crate::services::i18n::t;
use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CheckButton, DropDown, Entry as GtkEntry, FileDialog, FileFilter,
    Label, Orientation, ScrolledWindow, Separator, StringList, Widget, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy)]
enum BrowseMode {
    File,
    Folder,
    Icon,
}

/// Reusable form panel using native LibAdwaita and GTK4 theme styling.
#[allow(dead_code)]
pub struct Editor {
    pub root: GtkBox,
    state: Rc<RefCell<DesktopEntry>>,
}

impl Editor {
    pub fn new(parent: &impl IsA<Window>, entry: DesktopEntry) -> Self {
        let parent: Window = parent.upcast_ref().clone();
        let state = Rc::new(RefCell::new(entry.clone()));
        let root = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();

        let viewport = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let form = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(6)
            .margin_start(24)
            .margin_end(24)
            .margin_top(18)
            .margin_bottom(18)
            .build();
        viewport.set_child(Some(&form));
        root.append(&viewport);

        // --- 1. General Fields ---
        let type_label = t("type");
        let type_dd = build_dropdown(&["Application", "Link", "Directory"], &entry.entry_type);
        let state_type = state.clone();
        type_dd.connect_selected_notify(move |dd| {
            if let Some(item) = dd.selected_item() {
                if let Ok(obj) = item.downcast::<gtk4::StringObject>() {
                    state_type.borrow_mut().entry_type = obj.string().to_string();
                }
            }
        });
        form.append(&labelled(&type_label, &type_dd));

        let name_label = t("name");
        let name_entry = build_entry(&entry.name);
        let state_name = state.clone();
        name_entry.connect_changed(move |e| {
            state_name.borrow_mut().name = e.text().to_string();
        });
        form.append(&labelled(&name_label, &name_entry));

        let generic_label = t("generic_name");
        let generic_entry = build_entry(&entry.generic_name);
        let state_generic = state.clone();
        generic_entry.connect_changed(move |e| {
            state_generic.borrow_mut().generic_name = e.text().to_string();
        });
        form.append(&labelled(&generic_label, &generic_entry));

        let comment_label = t("comment");
        let comment_entry = build_entry(&entry.comment);
        let state_comment = state.clone();
        comment_entry.connect_changed(move |e| {
            state_comment.borrow_mut().comment = e.text().to_string();
        });
        form.append(&labelled(&comment_label, &comment_entry));

        // --- 2. Section: Execução ---
        form.append(&build_section_header(&t("section_execution")));

        let exec_label = t("exec");
        let state_exec = state.clone();
        let exec_row = build_path_row(
            &parent,
            &entry.exec,
            "document-open-symbolic",
            BrowseMode::File,
            move |e| {
                state_exec.borrow_mut().exec = e.text().to_string();
            },
        );
        form.append(&labelled(&exec_label, &exec_row));

        let path_label = t("path");
        let state_path = state.clone();
        let path_row = build_path_row(
            &parent,
            &entry.path,
            "folder-open-symbolic",
            BrowseMode::Folder,
            move |e| {
                state_path.borrow_mut().path = e.text().to_string();
            },
        );
        form.append(&labelled(&path_label, &path_row));

        let term_label = t("run_in_terminal");
        let state_term = state.clone();
        let term_cb = build_check(entry.terminal, move |active| {
            state_term.borrow_mut().terminal = active;
        });
        form.append(&labelled(&term_label, &term_cb));

        let url_label = t("url");
        let url_entry = build_entry(&entry.url);
        let state_url = state.clone();
        url_entry.connect_changed(move |e| {
            state_url.borrow_mut().url = e.text().to_string();
        });
        form.append(&labelled(&url_label, &url_entry));

        // --- 3. Section: Aparência e Classificação ---
        form.append(&build_section_header(&t("section_appearance")));

        let icon_label = t("icon");
        let state_icon = state.clone();
        let icon_row = build_path_row(
            &parent,
            &entry.icon,
            "image-x-generic-symbolic",
            BrowseMode::Icon,
            move |e| {
                state_icon.borrow_mut().icon = e.text().to_string();
            },
        );
        form.append(&labelled(&icon_label, &icon_row));

        let cat_label = t("categories");
        let cat_entry = build_entry(&entry.categories.join(";"));
        let state_cat = state.clone();
        cat_entry.connect_changed(move |e| {
            state_cat.borrow_mut().categories = parse_semi(&e.text());
        });
        form.append(&labelled(&cat_label, &cat_entry));

        let kw_label = t("keywords");
        let kw_entry = build_entry(&entry.keywords.join(";"));
        let state_kw = state.clone();
        kw_entry.connect_changed(move |e| {
            state_kw.borrow_mut().keywords = parse_semi(&e.text());
        });
        form.append(&labelled(&kw_label, &kw_entry));

        let mime_label = t("mime_types");
        let mime_entry = build_entry(&entry.mime_types.join(";"));
        let state_mime = state.clone();
        mime_entry.connect_changed(move |e| {
            state_mime.borrow_mut().mime_types = parse_semi(&e.text());
        });
        form.append(&labelled(&mime_label, &mime_entry));

        // --- 4. Section: Opções Avançadas ---
        form.append(&build_section_header(&t("section_advanced")));

        let startup_label = t("startup_notify");
        let state_startup = state.clone();
        let startup_cb = build_check(entry.startup_notify, move |active| {
            state_startup.borrow_mut().startup_notify = active;
        });
        form.append(&labelled(&startup_label, &startup_cb));

        let wmclass_label = t("startup_wm_class");
        let wmclass_entry = build_entry(&entry.startup_wm_class);
        let state_wmclass = state.clone();
        wmclass_entry.connect_changed(move |e| {
            state_wmclass.borrow_mut().startup_wm_class = e.text().to_string();
        });
        form.append(&labelled(&wmclass_label, &wmclass_entry));

        let nodisp_label = t("no_display");
        let state_nodisp = state.clone();
        let nodisp_cb = build_check(entry.no_display, move |active| {
            state_nodisp.borrow_mut().no_display = active;
        });
        form.append(&labelled(&nodisp_label, &nodisp_cb));

        // --- Footer: Action Buttons (Run, Cancel, Save) bottom-right ---
        let footer = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .halign(Align::End)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(16)
            .margin_end(16)
            .build();

        let status = Label::builder()
            .ellipsize(EllipsizeMode::End)
            .hexpand(true)
            .halign(Align::Start)
            .css_classes(["dim-label"])
            .build();

        let cancel_label = t("cancel");
        let save_label = t("save");

        let run_btn = Button::builder()
            .icon_name("media-playback-start-symbolic")
            .tooltip_text(&t("run"))
            .build();

        let cancel_btn = Button::builder().label(&cancel_label).build();

        let save_btn = Button::builder()
            .label(&save_label)
            .css_classes(["suggested-action"])
            .build();

        let state_run = state.clone();
        let status_run = status.clone();
        run_btn.connect_clicked(move |_| {
            let entry_curr = state_run.borrow();
            let exec_cmd = entry_curr.exec.trim();
            let path_dir = entry_curr.path.trim();
            if exec_cmd.is_empty() && !entry_curr.url.is_empty() {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&entry_curr.url)
                    .spawn();
                status_run.set_text(&t("status_opening_url"));
            } else if !exec_cmd.is_empty() {
                let tokens: Vec<&str> = exec_cmd
                    .split_whitespace()
                    .filter(|s| !s.starts_with('%'))
                    .collect();
                let needs_auth = tokens.first() == Some(&"sudo");
                let cmd_clean = if needs_auth {
                    tokens[1..].join(" ")
                } else {
                    tokens.join(" ")
                };
                let mut command = std::process::Command::new("sh");
                if needs_auth {
                    command.arg("-c").arg(format!("pkexec {cmd_clean}"));
                } else {
                    command.arg("-c").arg(&cmd_clean);
                }
                if !path_dir.is_empty() {
                    command.current_dir(path_dir);
                }
                match command.spawn() {
                    Ok(_) => status_run.set_text(&t("status_launched")),
                    Err(err) => {
                        status_run.set_text(&t("status_error").replace("{error}", &err.to_string()));
                    }
                }
            }
        });

        let status_cancel = status.clone();
        cancel_btn.connect_clicked(move |_| {
            status_cancel.set_text(&t("status_cancelled"));
        });

        let state_save = state.clone();
        let status_save = status.clone();
        save_btn.connect_clicked(move |_| match desktop_service::save_entry(&state_save.borrow()) {
            Ok(_p) => status_save.set_text(&t("status_saved")),
            Err(err) => {
                let err_msg = t("status_error").replace("{error}", &err.to_string());
                status_save.set_text(&err_msg);
            }
        });

        footer.append(&status);
        footer.append(&run_btn);
        footer.append(&cancel_btn);
        footer.append(&save_btn);

        root.append(&separator());
        root.append(&footer);

        Self { root, state }
    }

    #[allow(dead_code)]
    pub fn current(&self) -> DesktopEntry {
        self.state.borrow().clone()
    }
}

fn build_section_header(title: &str) -> Label {
    Label::builder()
        .label(title)
        .halign(Align::Start)
        .margin_top(16)
        .margin_bottom(6)
        .css_classes(["section-header", "accent"])
        .build()
}

fn parse_semi(s: &str) -> Vec<String> {
    s.split(';')
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.to_string())
        .collect()
}

fn build_entry(text: &str) -> GtkEntry {
    GtkEntry::builder().text(text).hexpand(true).build()
}

fn build_dropdown(items: &[&str], current: &str) -> DropDown {
    let list = StringList::new(items);
    let dd = DropDown::builder().model(&list).build();
    if let Some(pos) = items.iter().position(|i| *i == current) {
        dd.set_selected(pos as u32);
    }
    dd
}

fn build_check(active: bool, on_toggle: impl Fn(bool) + 'static) -> CheckButton {
    let cb = CheckButton::builder()
        .active(active)
        .halign(Align::Start)
        .valign(Align::Center)
        .build();
    cb.connect_toggled(move |b| {
        on_toggle(b.is_active());
    });
    cb
}

fn build_path_row(
    parent: &Window,
    value: &str,
    icon_name: &str,
    mode: BrowseMode,
    on_change: impl Fn(&GtkEntry) + 'static,
) -> GtkBox {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    let entry = build_entry(value);
    entry.connect_changed(move |e| on_change(e));

    let browse = Button::builder()
        .icon_name(icon_name)
        .tooltip_text(&t("browse"))
        .build();

    let entry_dlg = entry.clone();
    let parent_dlg = parent.clone();
    browse.connect_clicked(move |_| {
        let dialog = FileDialog::builder()
            .title(&dialog_title(mode))
            .modal(true)
            .build();
        dialog.set_filters(Some(&file_filters(mode)));

        let entry_pick = entry_dlg.clone();
        let parent_pick = parent_dlg.clone();
        let on_pick = move |result: Result<gtk4::gio::File, gtk4::glib::Error>| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    entry_pick.set_text(&path.to_string_lossy());
                }
            }
        };

        match mode {
            BrowseMode::Folder => dialog.select_folder(
                Some(&parent_pick),
                None::<&gtk4::gio::Cancellable>,
                on_pick,
            ),
            BrowseMode::File | BrowseMode::Icon => dialog.open(
                Some(&parent_pick),
                None::<&gtk4::gio::Cancellable>,
                on_pick,
            ),
        }
    });

    row.append(&entry);
    row.append(&browse);
    row
}

fn dialog_title(mode: BrowseMode) -> String {
    match mode {
        BrowseMode::Icon => t("dialog_icon_title"),
        BrowseMode::File => t("dialog_exec_title"),
        BrowseMode::Folder => t("dialog_folder_title"),
    }
}

fn file_filters(mode: BrowseMode) -> gtk4::gio::ListStore {
    let filters = gtk4::gio::ListStore::new::<FileFilter>();
    if matches!(mode, BrowseMode::Icon) {
        let images = FileFilter::new();
        images.set_name(Some(&t("filter_images")));
        for mime in [
            "image/png",
            "image/svg+xml",
            "image/x-icon",
            "image/jpeg",
            "image/webp",
            "image/x-xpixmap",
        ] {
            images.add_mime_type(mime);
        }
        filters.append(&images);
    }
    let all = FileFilter::new();
    all.set_name(Some(&t("filter_all_files")));
    all.add_pattern("*");
    filters.append(&all);
    filters
}

fn labelled(text: &str, input: &impl IsA<Widget>) -> GtkBox {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(16)
        .margin_top(4)
        .margin_bottom(4)
        .build();
    let label = Label::builder()
        .label(text)
        .width_request(160)
        .halign(Align::End)
        .valign(Align::Center)
        .xalign(1.0)
        .css_classes(["field-label", "dim-label"])
        .build();
    if input.is::<CheckButton>() {
        input.set_halign(Align::Start);
    } else {
        input.set_hexpand(true);
        input.set_halign(Align::Fill);
    }
    row.append(&label);
    row.append(input);
    row
}

fn separator() -> Separator {
    Separator::builder().orientation(Orientation::Horizontal).build()
}
