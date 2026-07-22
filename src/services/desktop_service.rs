use crate::models::{parse_desktop_file, write_desktop_file, DesktopEntry};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// Standard XDG search paths for .desktop files. Higher priority first — the
/// user-local dir wins over the system ones on conflicts during editing.
pub fn search_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ]
    .into_iter()
    .chain(std::env::var("XDG_DATA_HOME").ok().map(|d| PathBuf::from(d).join("applications")))
    .chain(
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".local/share/applications")),
    )
    .collect()
}

/// Read all .desktop files from the standard XDG paths. Entries parsed from
/// later paths override earlier ones sharing the same filename, mirroring how
/// launchers resolve user overrides.
pub fn load_all() -> Vec<DesktopEntry> {
    let paths = search_paths();
    let mut by_file: std::collections::HashMap<String, DesktopEntry> =
        std::collections::HashMap::new();

    for dir in &paths {
        if !dir.is_dir() {
            continue;
        }
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(err) => {
                warn!(dir = %dir.display(), error = %err, "failed to read applications dir");
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }
            let name_key = match path.file_name().and_then(|s| s.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            match fs::read_to_string(&path) {
                Ok(content) => match parse_desktop_file(&content, &path) {
                    Ok(parsed) => {
                        debug!(file = %name_key, "parsed entry");
                        by_file.insert(name_key, parsed);
                    }
                    Err(err) => error!(file = %path.display(), error = %err, "parse failed"),
                },
                Err(err) => warn!(file = %path.display(), error = %err, "read failed"),
            }
        }
    }

    let mut result: Vec<DesktopEntry> = by_file.into_values().collect();
    // Sort by name, case-insensitive, with empty names last.
    result.sort_by(|a, b| {
        let an = a.name.trim().to_ascii_lowercase();
        let bn = b.name.trim().to_ascii_lowercase();
        let ane = an.is_empty();
        let bne = bn.is_empty();
        match (ane, bne) {
            (true, true) => std::cmp::Ordering::Equal,
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            (false, false) => an.cmp(&bn),
        }
    });
    info!(count = result.len(), "loaded desktop entries");
    result
}

/// Load a single entry from disk by path (used when refreshing after save).
#[allow(dead_code)]
pub fn load_one(path: &Path) -> Result<DesktopEntry, std::io::Error> {
    let content = fs::read_to_string(path)?;
    parse_desktop_file(&content, path)
}

/// Save `entry` to disk. If it already has a `source_file`, overwrite that;
/// otherwise place it in `~/.local/share/applications/<suggested>.desktop`.
pub fn save_entry(entry: &DesktopEntry) -> Result<PathBuf, std::io::Error> {
    let target = if let Some(existing) = &entry.source_file {
        existing.clone()
    } else {
        user_applications_dir().join(entry.suggested_filename())
    };
    write_desktop_file(entry, &target)?;
    info!(path = %target.display(), "saved entry");
    Ok(target)
}

/// Delete the underlying .desktop file for an entry. Access-control note: in
/// user-local dir writes are fine; system files are only removed when the
/// process has permission.
pub fn delete_entry(entry: &DesktopEntry) -> Result<(), std::io::Error> {
    if let Some(path) = &entry.source_file {
        fs::remove_file(path)?;
        info!(path = %path.display(), "deleted entry");
    }
    Ok(())
}

/// Default destination for brand-new entries. Honors `XDG_DATA_HOME`.
pub fn user_applications_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg).join("applications")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/share/applications")
    } else {
        PathBuf::from("./.local/share/applications")
    }
}
