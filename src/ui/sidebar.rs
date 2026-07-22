use crate::models::DesktopEntry;
use crate::services::i18n::t;
use crate::services::icon_cache::IconCache;
use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CustomFilter, Entry as GtkEntry, FilterChange, FilterListModel,
    Image, Label, ListItem, ListView, Orientation, SignalListItemFactory, SingleSelection,
    StringList, StringObject,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Sidebar: search bar + scrollable filtered list + bottom action bar (Add / Delete).
#[allow(dead_code)]
pub struct Sidebar {
    pub root: GtkBox,
    pub search: GtkEntry,
    pub list_view: ListView,
    pub selection: SingleSelection,
    pub store: StringList,
    pub add_btn: Button,
    pub delete_btn: Button,
}

impl Sidebar {
    /// Row payload separator. `<name>\u{1f}<icon>\u{1f}<subtitle>\u{1f}<index>`.
    pub const SEP: char = '\u{1f}';

    pub fn new(entries: Vec<DesktopEntry>) -> Self {
        let root = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .css_classes(["sidebar"])
            .build();

        let search_placeholder = t("search_placeholder");
        let search = GtkEntry::builder()
            .placeholder_text(&search_placeholder)
            .primary_icon_name("system-search-symbolic")
            .hexpand(true)
            .margin_start(8)
            .margin_end(8)
            .margin_top(8)
            .margin_bottom(8)
            .build();
        root.append(&search);

        let store = StringList::new(&[]);
        for (i, e) in entries.iter().enumerate() {
            let subtitle = if !e.comment.is_empty() {
                &e.comment
            } else {
                &e.generic_name
            };
            let payload = format!(
                "{}{}{}{}{}{}{}",
                e.name,
                Self::SEP,
                e.icon,
                Self::SEP,
                subtitle,
                Self::SEP,
                i
            );
            store.append(&payload);
        }

        let query_cell = Rc::new(RefCell::new(String::new()));
        let query_filter = query_cell.clone();
        let filter_entries = entries.clone();
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
            let Some(e) = filter_entries.get(idx) else {
                return false;
            };
            let fields = [
                &e.name,
                &e.generic_name,
                &e.comment,
                &e.exec,
                &e.categories.join(" "),
            ];
            fields
                .iter()
                .any(|f| f.to_ascii_lowercase().contains(&q))
        });

        let filter_model = FilterListModel::new(Some(store.clone()), Some(filter.clone()));
        let selection = SingleSelection::new(Some(filter_model.clone()));
        selection.set_autoselect(false);

        let factory = SignalListItemFactory::new();
        install_factory_bindings(&factory, entries.clone());

        let list_view = ListView::new(Some(selection.clone()), Some(factory.clone()));
        let scroller = gtk4::ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .child(&list_view)
            .build();
        root.append(&scroller);

        let query_search = query_cell;
        search.connect_changed(move |s| {
            *query_search.borrow_mut() = s.text().to_string();
            filter.changed(FilterChange::Different);
        });

        // Bottom Action Bar inside Sidebar (Add '+' and Trash 'Delete' buttons)
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
        }
    }
}

fn install_factory_bindings(factory: &SignalListItemFactory, entries: Vec<DesktopEntry>) {
    factory.connect_setup(move |_factory, item| {
        let row = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(12)
            .margin_end(12)
            .build();

        let icon = Image::builder()
            .pixel_size(32)
            .width_request(32)
            .height_request(32)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        let text_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .valign(Align::Center)
            .hexpand(true)
            .build();

        let name = Label::builder()
            .ellipsize(EllipsizeMode::End)
            .halign(Align::Start)
            .css_classes(["sidebar-row-title"])
            .build();

        let subtitle = Label::builder()
            .ellipsize(EllipsizeMode::End)
            .halign(Align::Start)
            .css_classes(["sidebar-row-subtitle", "dim-label"])
            .build();

        text_box.append(&name);
        text_box.append(&subtitle);

        row.append(&icon);
        row.append(&text_box);

        if let Some(list_item) = item.downcast_ref::<ListItem>() {
            list_item.set_child(Some(&row));
        }
    });

    let icon_cache = IconCache::new();
    factory.connect_bind(move |_factory, item| {
        let Some(list_item) = item.downcast_ref::<ListItem>() else {
            return;
        };
        let Some(item_obj) = list_item.item() else {
            return;
        };
        let Some(string_obj) = item_obj.downcast::<StringObject>().ok() else {
            return;
        };

        let payload = string_obj.string().to_string();
        let mut parts = payload.split(Sidebar::SEP);
        let name_part = parts.next().unwrap_or("");
        let icon_spec = parts.next().unwrap_or("");
        let subtitle_part = parts.next().unwrap_or("");

        let Some(row_widget) = list_item.child() else {
            return;
        };
        let Some(row) = row_widget.downcast::<GtkBox>().ok() else {
            return;
        };
        let Some(first_child) = row.first_child() else {
            return;
        };
        let Some(image) = first_child.clone().downcast::<Image>().ok() else {
            return;
        };
        let Some(next_child) = first_child.next_sibling() else {
            return;
        };
        let Some(text_box) = next_child.downcast::<GtkBox>().ok() else {
            return;
        };
        let Some(name_widget) = text_box.first_child() else {
            return;
        };
        let Some(name_label) = name_widget.clone().downcast::<Label>().ok() else {
            return;
        };
        let Some(subtitle_widget) = name_widget.next_sibling() else {
            return;
        };
        let Some(subtitle_label) = subtitle_widget.downcast::<Label>().ok() else {
            return;
        };

        name_label.set_text(name_part);
        subtitle_label.set_text(subtitle_part);
        if subtitle_part.is_empty() {
            subtitle_label.set_visible(false);
        } else {
            subtitle_label.set_visible(true);
        }

        if let Some(paintable) = icon_cache.lookup(icon_spec) {
            image.set_paintable(Some(&paintable));
        } else {
            let _ = &entries;
            image.set_icon_name(Some("application-x-executable"));
        }
    });

    factory.connect_unbind(move |_factory, _| {});
}
