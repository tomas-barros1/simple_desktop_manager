use gtk4::gdk::{Paintable, Texture};
use gtk4::glib::object::Cast;
use gtk4::{IconLookupFlags, IconTheme, TextDirection};
use std::path::PathBuf;
use tracing::warn;

/// Wrapper over a GTK `IconTheme` to look up named and file-path icons at a
/// 48px source size. The returned paintable can be displayed at 32px for crisp
/// hi-dpi rendering.
#[derive(Clone)]
pub struct IconCache {
    theme: IconTheme,
}

impl IconCache {
    pub fn new() -> Self {
        let theme = if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::IconTheme::for_display(&display)
        } else {
            gtk4::IconTheme::new()
        };

        // Add standard user icon directories
        if let Ok(home) = std::env::var("HOME") {
            let user_icons = PathBuf::from(home.clone()).join(".local/share/icons");
            if user_icons.is_dir() {
                theme.add_search_path(user_icons);
            }
            let dot_icons = PathBuf::from(home).join(".icons");
            if dot_icons.is_dir() {
                theme.add_search_path(dot_icons);
            }
        }

        // Ensure system XDG icon directories are registered
        if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
            for dir in xdg_data_dirs.split(':') {
                if !dir.trim().is_empty() {
                    let p = PathBuf::from(dir).join("icons");
                    if p.is_dir() {
                        theme.add_search_path(p);
                    }
                }
            }
        } else {
            let usr_share = PathBuf::from("/usr/share/icons");
            if usr_share.is_dir() {
                theme.add_search_path(usr_share);
            }
            let usr_local = PathBuf::from("/usr/local/share/icons");
            if usr_local.is_dir() {
                theme.add_search_path(usr_local);
            }
        }

        // Apply explicit user theme from GtkSettings if configured
        if let Some(settings) = gtk4::Settings::default() {
            if let Some(theme_name) = settings.gtk_icon_theme_name() {
                if !theme_name.is_empty() {
                    theme.set_theme_name(Some(&theme_name));
                }
            }
        }

        Self { theme }
    }

    #[allow(dead_code)]
    pub fn from_theme(theme: IconTheme) -> Self {
        Self { theme }
    }

    /// Resolve a 48px paintable for the icon spec. Falls back to
    /// `application-x-executable` when spec is empty or missing.
    pub fn lookup(&self, spec: &str) -> Option<Paintable> {
        let spec = spec.trim();
        if spec.is_empty() {
            return self.named("application-x-executable", 48);
        }
        let path = PathBuf::from(spec);
        if path.is_file() {
            return match Texture::from_filename(&path) {
                Ok(tex) => Some(tex.upcast::<Paintable>()),
                Err(err) => {
                    warn!(icon = %path.display(), error = %err, "texture load failed");
                    None
                }
            };
        }
        self.named(spec, 48)
    }

    fn named(&self, name: &str, size: i32) -> Option<Paintable> {
        let clean_name = name
            .strip_suffix(".png")
            .or_else(|| name.strip_suffix(".svg"))
            .or_else(|| name.strip_suffix(".xpm"))
            .or_else(|| name.strip_suffix(".ico"))
            .unwrap_or(name);

        if self.theme.has_icon(clean_name) {
            let paintable = self.theme.lookup_icon(
                clean_name,
                &[],
                size,
                1,
                TextDirection::None,
                IconLookupFlags::empty(),
            );
            return Some(paintable.upcast::<Paintable>());
        }

        if self.theme.has_icon(name) {
            let paintable = self.theme.lookup_icon(
                name,
                &[],
                size,
                1,
                TextDirection::None,
                IconLookupFlags::empty(),
            );
            return Some(paintable.upcast::<Paintable>());
        }

        None
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}
