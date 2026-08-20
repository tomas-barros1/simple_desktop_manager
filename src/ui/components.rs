use crate::services::i18n::t;
use gtk4::prelude::*;
use gtk4::{
    Align, Button, FileDialog, FileFilter, Orientation, Separator, StringList, Window,
};
use libadwaita::prelude::*;
use libadwaita::{ComboRow, EntryRow, PreferencesGroup, SwitchRow};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowseMode {
    File,
    Folder,
    Icon,
}

/// Create a styled Adwaita preferences group.
pub fn build_preferences_group(title: &str) -> PreferencesGroup {
    PreferencesGroup::builder()
        .title(title)
        .margin_top(6)
        .margin_bottom(6)
        .build()
}

/// Create an AdwEntryRow with title, initial value, and change callback.
pub fn build_adw_entry_row(
    title: &str,
    text: &str,
    on_change: impl Fn(String) + 'static,
) -> EntryRow {
    let row = EntryRow::builder()
        .title(title)
        .text(text)
        .build();
    let on_change = Rc::new(on_change);
    row.connect_changed(move |r| {
        on_change(r.text().to_string());
    });
    row
}

/// Create an AdwComboRow selector with given items and pre-selected item.
pub fn build_adw_combo_row(
    title: &str,
    items: &[&str],
    current: &str,
    on_select: impl Fn(String) + 'static,
) -> ComboRow {
    let list = StringList::new(items);
    let row = ComboRow::builder()
        .title(title)
        .model(&list)
        .build();
    if let Some(pos) = items.iter().position(|i| *i == current) {
        row.set_selected(pos as u32);
    }
    let on_select = Rc::new(on_select);
    row.connect_selected_notify(move |r| {
        if let Some(item) = r.selected_item() {
            if let Ok(obj) = item.downcast::<gtk4::StringObject>() {
                on_select(obj.string().to_string());
            }
        }
    });
    row
}

/// Create an AdwSwitchRow button with initial state and toggle callback.
pub fn build_adw_switch_row(
    title: &str,
    active: bool,
    on_toggle: impl Fn(bool) + 'static,
) -> SwitchRow {
    let row = SwitchRow::builder()
        .title(title)
        .active(active)
        .build();
    let on_toggle = Rc::new(on_toggle);
    row.connect_active_notify(move |r| {
        on_toggle(r.is_active());
    });
    row
}

/// Create an AdwEntryRow with a browse button suffix that opens a native file/folder/icon dialog.
pub fn build_adw_path_row(
    parent: &Window,
    title: &str,
    value: &str,
    icon_name: &str,
    mode: BrowseMode,
    on_change: impl Fn(String) + 'static,
) -> EntryRow {
    let row = EntryRow::builder()
        .title(title)
        .text(value)
        .build();

    let browse = Button::builder()
        .icon_name(icon_name)
        .valign(Align::Center)
        .css_classes(["flat"])
        .tooltip_text(&t("browse"))
        .build();

    let row_dlg = row.clone();
    let parent_dlg = parent.clone();
    browse.connect_clicked(move |_| {
        let dialog = FileDialog::builder()
            .title(&dialog_title(mode))
            .modal(true)
            .build();
        dialog.set_filters(Some(&file_filters(mode)));

        let row_pick = row_dlg.clone();
        let parent_pick = parent_dlg.clone();
        let on_pick = move |result: Result<gtk4::gio::File, gtk4::glib::Error>| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let path_str = path.to_string_lossy();
                    if mode == BrowseMode::File && path_str.contains(' ') && !path_str.starts_with('"') {
                        row_pick.set_text(&format!("\"{path_str}\""));
                    } else {
                        row_pick.set_text(&path_str);
                    }
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

    let on_change = Rc::new(on_change);
    row.connect_changed(move |r| {
        on_change(r.text().to_string());
    });

    row.add_suffix(&browse);
    row
}

/// Create an AdwEntryRow for icons with a live icon preview prefix and browse button suffix.
pub fn build_adw_icon_row(
    parent: &Window,
    title: &str,
    value: &str,
    on_change: impl Fn(String) + 'static,
) -> EntryRow {
    let row = EntryRow::builder()
        .title(title)
        .text(value)
        .build();

    let icon_cache = crate::services::icon_cache::IconCache::new();
    let preview = gtk4::Image::builder()
        .pixel_size(28)
        .width_request(28)
        .height_request(28)
        .valign(Align::Center)
        .margin_start(4)
        .build();

    if let Some(paintable) = icon_cache.lookup(value) {
        preview.set_paintable(Some(&paintable));
    } else {
        preview.set_icon_name(Some("application-x-executable"));
    }

    let browse = Button::builder()
        .icon_name("image-x-generic-symbolic")
        .valign(Align::Center)
        .css_classes(["flat"])
        .tooltip_text(&t("browse"))
        .build();

    let row_dlg = row.clone();
    let parent_dlg = parent.clone();
    browse.connect_clicked(move |_| {
        let dialog = FileDialog::builder()
            .title(&t("dialog_icon_title"))
            .modal(true)
            .build();
        dialog.set_filters(Some(&file_filters(BrowseMode::Icon)));

        let row_pick = row_dlg.clone();
        let parent_pick = parent_dlg.clone();
        let on_pick = move |result: Result<gtk4::gio::File, gtk4::glib::Error>| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    row_pick.set_text(&path.to_string_lossy());
                }
            }
        };

        dialog.open(
            Some(&parent_pick),
            None::<&gtk4::gio::Cancellable>,
            on_pick,
        );
    });

    let on_change = Rc::new(on_change);
    let preview_update = preview.clone();
    let cache_update = icon_cache.clone();
    row.connect_changed(move |r| {
        let text = r.text().to_string();
        if let Some(paintable) = cache_update.lookup(&text) {
            preview_update.set_paintable(Some(&paintable));
        } else {
            preview_update.set_icon_name(Some("application-x-executable"));
        }
        on_change(text);
    });

    row.add_prefix(&preview);
    row.add_suffix(&browse);
    row
}

/// Create a horizontal separator line.
pub fn build_separator() -> Separator {
    Separator::builder().orientation(Orientation::Horizontal).build()
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
