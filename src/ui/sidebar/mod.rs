pub mod row_factory;

use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::services::search_service;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CustomFilter, Entry as GtkEntry, FilterChange, FilterListModel,
    Label, ListView, Orientation, ScrolledWindow, SignalListItemFactory, SingleSelection, StringList,
    StringObject, ToggleButton,
};
use row_factory::{install_row_factory, SEP};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginFilter {
    All,
    User,
    System,
}

/// Sidebar: searchable application list with origin filter and Add/Delete action bar.
#[allow(dead_code)]
pub struct Sidebar {
    pub root: GtkBox,
    pub search: GtkEntry,
    pub list_view: ListView,
    pub selection: SingleSelection,
    pub store: StringList,
    pub add_btn: Button,
    pub delete_btn: Button,
    pub count_label: Label,
    entries: Rc<RefCell<Vec<DesktopEntry>>>,
    filter: CustomFilter,
}

impl Sidebar {
    pub const SEP: char = SEP;

    pub fn new(entries: Vec<DesktopEntry>) -> Self {
        let entries_rc = Rc::new(RefCell::new(entries));
        let root = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .css_classes(["sidebar"])
            .build();

        // Search entry
        let search = GtkEntry::builder()
            .placeholder_text(&t("search_placeholder"))
            .primary_icon_name("system-search-symbolic")
            .hexpand(true)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(6)
            .build();
        root.append(&search);

        // Segmented filter buttons [Todos | Usuário | Sistema]
        let filter_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .homogeneous(true)
            .margin_start(8)
            .margin_end(8)
            .margin_bottom(8)
            .css_classes(["linked"])
            .build();

        let btn_all = ToggleButton::builder()
            .label(&t("filter_all"))
            .active(true)
            .build();

        let btn_user = ToggleButton::builder()
            .label(&t("filter_user"))
            .group(&btn_all)
            .build();

        let btn_system = ToggleButton::builder()
            .label(&t("filter_system"))
            .group(&btn_all)
            .build();

        filter_box.append(&btn_all);
        filter_box.append(&btn_user);
        filter_box.append(&btn_system);
        root.append(&filter_box);

        // String store
        let payloads = build_payloads(&entries_rc.borrow());
        let str_refs: Vec<&str> = payloads.iter().map(|s| s.as_str()).collect();
        let store = StringList::new(&str_refs);

        // Custom search & origin filter
        let origin_cell = Rc::new(RefCell::new(OriginFilter::All));
        let query_cell = Rc::new(RefCell::new(String::new()));
        let query_filter = query_cell.clone();
        let origin_filter = origin_cell.clone();
        let filter_entries = entries_rc.clone();

        let filter = CustomFilter::new(move |item: &gtk4::glib::Object| {
            let Some(string_obj) = item.downcast_ref::<StringObject>() else {
                return false;
            };
            let payload = string_obj.string().to_string();
            let mut parts = payload.rsplitn(2, Self::SEP);
            let idx_str = parts.next().unwrap_or("0");
            let idx: usize = idx_str.parse().unwrap_or(0);
            let guard = filter_entries.borrow();
            let Some(e) = guard.get(idx) else {
                return false;
            };

            // Check origin filter
            match *origin_filter.borrow() {
                OriginFilter::All => {}
                OriginFilter::User => {
                    if e.is_system_entry() {
                        return false;
                    }
                }
                OriginFilter::System => {
                    if !e.is_system_entry() {
                        return false;
                    }
                }
            }

            // Check search query
            let q = query_filter.borrow().to_ascii_lowercase();
            if q.trim().is_empty() {
                return true;
            }
            search_service::matches(e, &q)
        });

        let filter_model = FilterListModel::new(Some(store.clone()), Some(filter.clone()));
        let selection = SingleSelection::new(Some(filter_model.clone()));
        selection.set_autoselect(false);

        let factory = SignalListItemFactory::new();
        install_row_factory(&factory);

        let list_view = ListView::new(Some(selection.clone()), Some(factory));
        let scroller = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&list_view)
            .build();
        root.append(&scroller);

        // Search input change handler
        let query_search = query_cell;
        let filter_changed_search = filter.clone();
        search.connect_changed(move |s| {
            *query_search.borrow_mut() = s.text().to_string();
            filter_changed_search.changed(FilterChange::Different);
        });

        // Origin filter toggle handlers
        let origin_all = origin_cell.clone();
        let filter_changed_all = filter.clone();
        btn_all.connect_toggled(move |b| {
            if b.is_active() {
                *origin_all.borrow_mut() = OriginFilter::All;
                filter_changed_all.changed(FilterChange::Different);
            }
        });

        let origin_usr = origin_cell.clone();
        let filter_changed_usr = filter.clone();
        btn_user.connect_toggled(move |b| {
            if b.is_active() {
                *origin_usr.borrow_mut() = OriginFilter::User;
                filter_changed_usr.changed(FilterChange::Different);
            }
        });

        let origin_sys = origin_cell;
        let filter_changed_sys = filter.clone();
        btn_system.connect_toggled(move |b| {
            if b.is_active() {
                *origin_sys.borrow_mut() = OriginFilter::System;
                filter_changed_sys.changed(FilterChange::Different);
            }
        });

        // Bottom Action Bar with App Counter, Add and Delete buttons
        let bottom_bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(8)
            .margin_end(8)
            .margin_top(6)
            .margin_bottom(8)
            .css_classes(["sidebar-bottom-bar"])
            .build();

        let add_btn = Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(["btn-add-sidebar"])
            .tooltip_text(&t("new_entry"))
            .build();

        let count_label = Label::builder()
            .halign(Align::Center)
            .valign(Align::Center)
            .hexpand(true)
            .css_classes(["dim-label", "caption"])
            .build();

        let delete_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .sensitive(false)
            .css_classes(["destructive-action", "btn-delete-sidebar"])
            .tooltip_text(&t("delete_entry"))
            .build();

        // Connect count updates
        let count_lbl_cb = count_label.clone();
        let fmodel_cb = filter_model.clone();
        update_count_display(&count_label, filter_model.n_items());
        filter_model.connect_items_changed(move |_, _, _, _| {
            update_count_display(&count_lbl_cb, fmodel_cb.n_items());
        });

        bottom_bar.append(&add_btn);
        bottom_bar.append(&count_label);
        bottom_bar.append(&delete_btn);
        root.append(&bottom_bar);

        Self {
            root,
            search,
            list_view,
            selection,
            store,
            add_btn,
            delete_btn,
            count_label,
            entries: entries_rc,
            filter,
        }
    }

    /// Retrieve an entry by its original index.
    pub fn get_entry(&self, idx: usize) -> Option<DesktopEntry> {
        self.entries.borrow().get(idx).cloned()
    }

    /// Get all entries currently held by the sidebar.
    #[allow(dead_code)]
    pub fn entries(&self) -> Vec<DesktopEntry> {
        self.entries.borrow().clone()
    }

    /// Clear the active selection in the list.
    pub fn clear_selection(&self) {
        self.selection.set_selected(gtk4::INVALID_LIST_POSITION);
    }

    /// Select an entry by its filesystem path.
    pub fn select_entry_by_path(&self, path: &std::path::Path) {
        let guard = self.entries.borrow();
        let target_idx = guard
            .iter()
            .position(|e| e.source_file.as_deref() == Some(path));
        drop(guard);

        if let Some(target_idx) = target_idx {
            if let Some(model) = self.selection.model() {
                for i in 0..model.n_items() {
                    if let Some(item) = model.item(i) {
                        if let Ok(str_obj) = item.downcast::<StringObject>() {
                            let payload = str_obj.string().to_string();
                            let idx: usize = payload
                                .rsplit_once(Self::SEP)
                                .map(|(_, idx_s)| idx_s.parse().unwrap_or(usize::MAX))
                                .unwrap_or(usize::MAX);
                            if idx == target_idx {
                                self.selection.set_selected(i);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Replace the entry list and rebuild rows (used after deleting/saving/adding).
    pub fn refresh(&self, new_entries: Vec<DesktopEntry>) {
        *self.entries.borrow_mut() = new_entries;
        let old_count = self.store.n_items();
        let payloads = build_payloads(&self.entries.borrow());
        let str_refs: Vec<&str> = payloads.iter().map(|s| s.as_str()).collect();
        self.store.splice(0, old_count, &str_refs);
        self.filter.changed(FilterChange::Different);
        self.selection.set_selected(gtk4::INVALID_LIST_POSITION);
    }
}

fn build_payloads(entries: &[DesktopEntry]) -> Vec<String> {
    entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let display_name = if e.name.trim().is_empty() {
                e.suggested_filename()
            } else {
                e.name.clone()
            };
            let subtitle = if !e.comment.is_empty() {
                &e.comment
            } else {
                &e.generic_name
            };
            let origin = if e.is_system_entry() { "sys" } else { "usr" };
            format!(
                "{}{}{}{}{}{}{}{}{}",
                display_name,
                Sidebar::SEP,
                e.icon,
                Sidebar::SEP,
                subtitle,
                Sidebar::SEP,
                origin,
                Sidebar::SEP,
                i
            )
        })
        .collect()
}

fn update_count_display(label: &Label, count: u32) {
    if count == 1 {
        label.set_text(&t("apps_count_single"));
    } else {
        label.set_text(&t("apps_count").replace("{count}", &count.to_string()));
    }
}
