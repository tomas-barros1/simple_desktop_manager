use crate::app::window::Window;
use gtk4::gio::prelude::*;
use libadwaita::Application;

/// The [`libadwaita::Application`] singleton holding the active window.
pub fn application_new() -> Application {
    let app = Application::builder()
        .application_id("dev.simplemenu.DesktopManager")
        .build();

    app.connect_startup(|_| {
        let _ = libadwaita::init();
        libadwaita::StyleManager::default().set_color_scheme(libadwaita::ColorScheme::Default);
    });

    app.connect_activate(move |a| {
        let win = Window::new(a);
        win.present();
    });

    app
}
