use gtk4::gdk::Paintable;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Image, Label, Orientation, Widget};

/// A 32px icon image. Sources a 48px paintable so hi-dpi displays render crisp.
#[allow(dead_code)]
pub fn build_icon_overlay(icon: Option<Paintable>) -> Image {
    let image = Image::builder()
        .width_request(32)
        .height_request(32)
        .pixel_size(32)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();
    if let Some(p) = icon {
        image.set_paintable(Some(&p));
    } else {
        image.set_icon_name(Some("application-x-executable"));
    }
    image
}

/// Labeled field pair: 100px-wide header followed by an `hexpand` input.
#[allow(dead_code)]
pub fn build_field_label(text: &str) -> Label {
    Label::builder()
        .label(text)
        .width_request(100)
        .halign(Align::Start)
        .valign(Align::Center)
        .xalign(0.0)
        .build()
}

#[allow(dead_code)]
pub fn build_row(label_text: &str, input: &impl IsA<Widget>) -> GtkBox {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .margin_start(12)
        .margin_end(12)
        .margin_top(6)
        .margin_bottom(6)
        .build();
    let label = build_field_label(label_text);
    label.add_css_class(&"dim-label");
    input.set_hexpand(true);
    input.set_halign(Align::Fill);
    row.append(&label);
    row.append(input);
    row
}
