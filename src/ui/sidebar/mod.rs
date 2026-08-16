pub mod row_factory;

use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::services::search_service;
use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CustomFilter, Entry as GtkEntry, FilterChange, FilterListModel, ListView,
    Orientation, ScrolledWindow, SignalListItemFactory, SingleSelection, StringList, StringObject,
};
use row_factory::{install_row_factory, SEP};
use std::cell::RefCell;
use std::rc::Rc;

/// Sidebar: searchable application list with an Add/Delete action bar at the bottom.
#[allow(dead_code)]
pub struct Sidebar {
    pub root: GtkBox,
    pub search: GtkEntry,
    pub list_view: ListView,
    pub selection: SingleSelection,
    pub store: StringList,
    pub add_btn: Button,
    pub delete_btn: Button,
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
            .margin_bottom(8)
            .build();
        root.append(&search);

        // String store
        let payloads = build_payloads(&entries_rc.borrow());
        let str_refs: Vec<&str> = payloads.iter().map(|s| s.as_str()).collect();
        let store = StringList::new(&str_refs);

        // Custom search filter using search_service::matches
        let query_cell = Rc::new(RefCell::new(String::new()));
        let query_filter = query_cell.clone();
        let filter_entries = entries_rc.clone();

        let filter = CustomFilter::new(move |item: &gtk4::glib::Object| {
            let q = query_filter.borrow().to_ascii_lowercase();
            if q.trim().is_empty() {
                return true;
            }
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
            search_service::matches(e, &q)
        });

        let filter_model = FilterListModel::new(Some(store.clone()), Some(filter.clone()));
        let selection = SingleSelection::new(Some(filter_model));
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
        let filter_changed = filter.clone();
        search.connect_changed(move |s| {
            *query_search.borrow_mut() = s.text().to_string();
            filter_changed.changed(FilterChange::Different);
        });

        // Bottom Action Bar (Add and Delete buttons)
        let bottom_bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .css_classes(["sidebar-bottom-bar"])
            .build();

        let add_btn = Button::builder()
            .icon_name("list-add-symbolic")
            .hexpand(true)
            .css_classes(["btn-add-sidebar"])
            .tooltip_text(&t("new_entry"))
            .build();

        let delete_btn = Button::builder()
            .icon_name("user-trash-symbolic")
            .hexpand(true)
            .sensitive(false)
            .css_classes(["destructive-action", "btn-delete-sidebar"])
            .tooltip_text(&t("delete_entry"))
            .build();

        bottom_bar.append(&add_btn);
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
            format!(
                "{}{}{}{}{}{}{}",
                display_name,
                Sidebar::SEP,
                e.icon,
                Sidebar::SEP,
                subtitle,
                Sidebar::SEP,
                i
            )
        })
        .collect()
}
