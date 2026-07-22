use gtk4::gdk::{Paintable, Texture};
use gtk4::gio::prelude::FileExt;
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
        Self {
            theme: IconTheme::default(),
        }
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
        let paintable = self.theme.lookup_icon(
            name,
            &[],
            size,
            1,
            TextDirection::None,
            IconLookupFlags::FORCE_REGULAR,
        );
        match paintable.file() {
            Some(gio_file) => match gio_file.path() {
                Some(p) => Texture::from_filename(&p)
                    .map(|t| t.upcast::<Paintable>())
                    .map_err(|err| warn!(icon = %p.display(), error = %err, "load texture failed"))
                    .ok(),
                None => {
                    warn!(icon = name, "icon has no file path");
                    None
                }
            },
            None => {
                warn!(icon = name, "icon not found");
                None
            }
        }
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}
