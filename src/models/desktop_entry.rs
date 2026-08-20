use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

/// A parsed .desktop entry. Transient fields (`source_file`, `directory`) are
/// kept for the UI but never written back to the file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopEntry {
    pub entry_type: String,
    pub name: String,
    #[serde(default)]
    pub generic_name: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub exec: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub terminal: bool,
    #[serde(default)]
    pub no_display: bool,
    #[serde(default)]
    pub startup_notify: bool,
    #[serde(default)]
    pub startup_wm_class: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub mime_types: Vec<String>,
    #[serde(default)]
    pub url: String,
    // Transient (not serialized to file)
    #[serde(skip_serializing, default)]
    pub source_file: Option<std::path::PathBuf>,
    #[serde(skip_serializing, default)]
    pub directory: Option<std::path::PathBuf>,
}

impl Default for DesktopEntry {
    fn default() -> Self {
        Self {
            entry_type: "Application".to_string(),
            name: String::new(),
            generic_name: String::new(),
            comment: String::new(),
            icon: String::new(),
            exec: String::new(),
            path: String::new(),
            terminal: false,
            no_display: false,
            startup_notify: false,
            startup_wm_class: String::new(),
            categories: Vec::new(),
            keywords: Vec::new(),
            mime_types: Vec::new(),
            url: String::new(),
            source_file: None,
            directory: None,
        }
    }
}

impl DesktopEntry {
    /// Generate a stable filename derived from the entry name, used when writing
    /// a brand-new entry to `~/.local/share/applications`.
    pub fn suggested_filename(&self) -> String {
        let base = self
            .name
            .trim()
            .to_ascii_lowercase()
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string();
        let file = if base.is_empty() {
            "new-entry".to_string()
        } else {
            base
        };
        // Avoid collisions with existing source file name if we already have one.
        if let Some(p) = &self.source_file {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name.ends_with(".desktop") {
                    return name.to_string();
                }
            }
        }
        format!("{file}.desktop")
    }

    /// Returns true if this entry resides in a system directory (e.g. /usr/share/applications)
    /// that requires elevated privileges (sudo/pkexec) to modify directly.
    pub fn is_system_entry(&self) -> bool {
        if let Some(path) = &self.source_file {
            let path_str = path.to_string_lossy();
            path_str.starts_with("/usr/")
                || path_str.starts_with("/var/")
                || path_str.starts_with("/etc/")
                || path_str.starts_with("/opt/")
        } else {
            false
        }
    }
}

/// Returns the list of locale suffix tags to check in priority order based on system locale.
/// E.g. for "pt_BR.UTF-8", returns vec!["pt_BR", "pt"].
pub fn get_locale_candidates() -> Vec<String> {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            let val = val.trim();
            if val.is_empty() || val.eq_ignore_ascii_case("c") || val.eq_ignore_ascii_case("posix") {
                continue;
            }
            let without_encoding = val.split('.').next().unwrap_or(val);
            let locale = without_encoding.split('@').next().unwrap_or(without_encoding);

            let mut candidates = Vec::new();
            if !locale.is_empty() {
                candidates.push(locale.to_string());
                if let Some((lang, _country)) = locale.split_once('_') {
                    if !lang.is_empty() && lang != locale {
                        candidates.push(lang.to_string());
                    }
                }
            }
            if !candidates.is_empty() {
                return candidates;
            }
        }
    }
    Vec::new()
}

/// Parse a .desktop file into a `DesktopEntry`. Falls back to empty strings on
/// missing keys. The directory and source_file metadata are set by the caller.
pub fn parse_desktop_file(content: &str, path: &Path) -> Result<DesktopEntry, std::io::Error> {
    let mut entry = DesktopEntry::default();
    let mut section = String::new();
    let mut all_keys: BTreeMap<String, String> = BTreeMap::new();

    for raw_line in content.lines() {
        let line = raw_line.trim_end_matches('\r');
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_string();
            continue;
        }
        if section != "Desktop Entry" {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().to_string();
        let value = value.trim().to_string();
        all_keys.insert(key.clone(), value.clone());
    }

    let locale_candidates = get_locale_candidates();

    let get_localized = |base_key: &str| -> String {
        for loc in &locale_candidates {
            let loc_key = format!("{base_key}[{loc}]");
            if let Some(val) = all_keys.get(&loc_key) {
                if !val.trim().is_empty() {
                    return val.clone();
                }
            }
        }
        all_keys.get(base_key).cloned().unwrap_or_default()
    };

    entry.entry_type = all_keys.get("Type").cloned().unwrap_or_default();
    if entry.entry_type.is_empty() {
        entry.entry_type = "Application".to_string();
    }
    entry.name = get_localized("Name");
    entry.generic_name = get_localized("GenericName");
    entry.comment = get_localized("Comment");
    entry.icon = all_keys.get("Icon").cloned().unwrap_or_default();
    entry.exec = all_keys.get("Exec").cloned().unwrap_or_default();
    entry.path = all_keys.get("Path").cloned().unwrap_or_default();
    entry.terminal = parse_bool(all_keys.get("Terminal").unwrap_or(&String::new()));
    entry.no_display = parse_bool(all_keys.get("NoDisplay").unwrap_or(&String::new()));
    entry.startup_notify = parse_bool(all_keys.get("StartupNotify").unwrap_or(&String::new()));
    entry.startup_wm_class = all_keys.get("StartupWMClass").cloned().unwrap_or_default();
    entry.categories = parse_list(all_keys.get("Categories").unwrap_or(&String::new()));
    entry.keywords = parse_list(&get_localized("Keywords"));
    entry.mime_types = parse_list(all_keys.get("MimeType").unwrap_or(&String::new()));
    entry.url = all_keys.get("URL").cloned().unwrap_or_default();

    entry.source_file = Some(path.to_path_buf());
    entry.directory = path.parent().map(|p| p.to_path_buf());

    Ok(entry)
}

/// `true`/`false` per spec; tolerate stray `1`/`0` from older entries.
fn parse_bool(s: &str) -> bool {
    matches!(s.trim(), "true" | "1")
}

/// List fields are separated by `;` and may have a trailing semicolon. Empty
/// segments are ignored.
fn parse_list(s: &str) -> Vec<String> {
    s.split(';')
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.to_string())
        .collect()
}

fn format_bool(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

fn format_list(list: &[String]) -> String {
    if list.is_empty() {
        String::new()
    } else {
        let joined = list.join(";");
        format!("{joined};")
    }
}

/// Serialize the entry back into the .desktop text format. Only the
/// `[Desktop Entry]` section is emitted; transient fields are skipped.
pub fn serialize_desktop_entry(entry: &DesktopEntry) -> String {
    let mut out = String::new();
    out.push_str("[Desktop Entry]\n");
    out.push_str("Type=");
    out.push_str(&entry.entry_type);
    out.push('\n');
    out.push_str("Name=");
    out.push_str(&entry.name);
    out.push('\n');
    if !entry.generic_name.is_empty() {
        out.push_str("GenericName=");
        out.push_str(&entry.generic_name);
        out.push('\n');
    }
    if !entry.comment.is_empty() {
        out.push_str("Comment=");
        out.push_str(&entry.comment);
        out.push('\n');
    }
    if !entry.icon.is_empty() {
        out.push_str("Icon=");
        out.push_str(&entry.icon);
        out.push('\n');
    }
    match entry.entry_type.as_str() {
        "Link" => {
            out.push_str("URL=");
            out.push_str(&entry.url);
            out.push('\n');
        }
        _ => {
            out.push_str("Exec=");
            out.push_str(&entry.exec);
            out.push('\n');
            if !entry.path.is_empty() {
                out.push_str("Path=");
                out.push_str(&entry.path);
                out.push('\n');
            }
            out.push_str("Terminal=");
            out.push_str(format_bool(entry.terminal));
            out.push('\n');
            out.push_str("StartupNotify=");
            out.push_str(format_bool(entry.startup_notify));
            out.push('\n');
        }
    }
    out.push_str("NoDisplay=");
    out.push_str(format_bool(entry.no_display));
    out.push('\n');
    if !entry.startup_wm_class.is_empty() {
        out.push_str("StartupWMClass=");
        out.push_str(&entry.startup_wm_class);
        out.push('\n');
    }
    if !entry.categories.is_empty() {
        out.push_str("Categories=");
        out.push_str(&format_list(&entry.categories));
        out.push('\n');
    }
    if !entry.keywords.is_empty() {
        out.push_str("Keywords=");
        out.push_str(&format_list(&entry.keywords));
        out.push('\n');
    }
    if !entry.mime_types.is_empty() {
        out.push_str("MimeType=");
        out.push_str(&format_list(&entry.mime_types));
        out.push('\n');
    }
    out
}

/// Write an entry to path, creating parent directories as needed.
pub fn write_desktop_file(entry: &DesktopEntry, path: &Path) -> Result<(), std::io::Error> {
    let text = serialize_desktop_entry(entry);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(text.as_bytes())?;
    file.flush()?;
    Ok(())
}
