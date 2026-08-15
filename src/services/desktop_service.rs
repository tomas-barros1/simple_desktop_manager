use crate::models::{parse_desktop_file, serialize_desktop_entry, write_desktop_file, DesktopEntry};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// Standard XDG search paths for .desktop files. Higher priority first — the
/// user-local dir wins over the system ones on conflicts during editing.
pub fn search_paths() -> Vec<PathBuf> {
    let mut raw: Vec<PathBuf> = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
    ];
    if let Ok(d) = std::env::var("XDG_DATA_HOME") {
        raw.push(PathBuf::from(d).join("applications"));
    }
    if let Ok(h) = std::env::var("HOME") {
        raw.push(PathBuf::from(h).join(".local/share/applications"));
    }
    let mut seen = std::collections::HashSet::new();
    let mut paths = Vec::new();
    for p in raw {
        if seen.insert(p.clone()) {
            paths.push(p);
        }
    }
    paths
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
        let mut files = Vec::new();
        collect_desktop_files(dir, &mut files);
        for path in files {
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
/// When the target is not writable (e.g. system directories), re-runs the
/// write through `pkexec`, which prompts the user for their password.
pub fn save_entry(entry: &DesktopEntry) -> Result<PathBuf, std::io::Error> {
    let target = if let Some(existing) = &entry.source_file {
        existing.clone()
    } else {
        let user_dir = user_applications_dir();
        fs::create_dir_all(&user_dir)?;
        user_dir.join(entry.suggested_filename())
    };
    match write_desktop_file(entry, &target) {
        Ok(_) => {}
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            save_with_pkexec(entry, &target)?;
        }
        Err(err) => return Err(err),
    }
    info!(path = %target.display(), "saved entry");
    Ok(target)
}

fn save_with_pkexec(entry: &DesktopEntry, target: &Path) -> Result<(), std::io::Error> {
    let content = serialize_desktop_entry(entry);
    let tmp = std::env::temp_dir().join(format!("smm-{}.desktop", std::process::id()));
    fs::write(&tmp, content)?;
    let status = std::process::Command::new("pkexec")
        .arg("cp")
        .arg(&tmp)
        .arg(target)
        .status();
    let _ = fs::remove_file(&tmp);
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(std::io::Error::other(format!(
            "pkexec exited with status {s}"
        ))),
        Err(err) => Err(err),
    }
}

/// Delete the underlying .desktop file for an entry. User-local files are
/// removed directly; system files that need elevation re-run the removal
/// through `pkexec`, which prompts the user for their password.
pub fn delete_entry(entry: &DesktopEntry) -> Result<(), std::io::Error> {
    if let Some(path) = &entry.source_file {
        if !path.exists() {
            return Ok(());
        }
        match fs::remove_file(path) {
            Ok(_) => info!(path = %path.display(), "deleted entry"),
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                let status = std::process::Command::new("pkexec")
                    .arg("rm")
                    .arg("-f")
                    .arg(path)
                    .status();
                match status {
                    Ok(s) if s.success() => {
                        info!(path = %path.display(), "deleted entry via pkexec");
                    }
                    Ok(s) => {
                        return Err(std::io::Error::other(format!(
                            "pkexec exited with status {s}"
                        )));
                    }
                    Err(err) => return Err(err),
                }
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

/// Recursively collect every `.desktop` file under `dir`, including
/// subdirectories (e.g. Wine's `~/.local/share/applications/wine/Programs/...`).
pub(crate) fn collect_desktop_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        warn!(dir = %dir.display(), "failed to read applications dir");
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_desktop_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
            out.push(path);
        }
    }
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
