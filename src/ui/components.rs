use crate::services::i18n::t;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CheckButton, DropDown, Entry as GtkEntry, FileDialog, FileFilter,
    Label, Orientation, Separator, StringList, Widget, Window,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowseMode {
    File,
    Folder,
    Icon,
}

/// Create a styled section header label.
pub fn build_section_header(title: &str) -> Label {
    Label::builder()
        .label(title)
        .halign(Align::Start)
        .margin_top(16)
        .margin_bottom(6)
        .css_classes(["section-header", "accent"])
        .build()
}

/// Create a text entry field with initial content.
pub fn build_entry(text: &str) -> GtkEntry {
    GtkEntry::builder().text(text).hexpand(true).build()
}

/// Create a dropdown selector with given items and pre-selected item.
pub fn build_dropdown(items: &[&str], current: &str) -> DropDown {
    let list = StringList::new(items);
    let dd = DropDown::builder().model(&list).build();
    if let Some(pos) = items.iter().position(|i| *i == current) {
        dd.set_selected(pos as u32);
    }
    dd
}

/// Create a checkbox button with initial state and toggle callback.
pub fn build_check(active: bool, on_toggle: impl Fn(bool) + 'static) -> CheckButton {
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

/// Create a horizontal separator line.
pub fn build_separator() -> Separator {
    Separator::builder().orientation(Orientation::Horizontal).build()
}

/// Create a form row with a right-aligned label and an input widget.
pub fn build_labelled_row(text: &str, input: &impl IsA<Widget>) -> GtkBox {
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

/// Create a text entry field with a browse button that opens a native file/folder/icon dialog.
pub fn build_path_row(
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

/// Helper function to parse semicolon-delimited lists (categories, keywords, mime types).
pub fn parse_semicolon_list(s: &str) -> Vec<String> {
    s.split(';')
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.to_string())
        .collect()
}
