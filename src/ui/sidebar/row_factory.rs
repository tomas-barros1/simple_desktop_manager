use crate::services::icon_cache::IconCache;
use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Image, Label, ListItem, Orientation, SignalListItemFactory, StringObject};

pub const SEP: char = '\u{1f}';

/// Install factory signal handlers for the sidebar list view items.
pub fn install_row_factory(factory: &SignalListItemFactory) {
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
        let mut parts = payload.split(SEP);
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
        subtitle_label.set_visible(!subtitle_part.is_empty());

        if let Some(paintable) = icon_cache.lookup(icon_spec) {
            image.set_paintable(Some(&paintable));
        } else {
            image.set_icon_name(Some("application-x-executable"));
        }
    });

    factory.connect_unbind(move |_factory, _| {});
}
