mod app;
mod models;
mod services;
mod ui;

use app::application::application_new;
use gtk4::gio::prelude::*;
use services::i18n::I18n;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    I18n::init();

    let app = application_new();
    app.run();
}
