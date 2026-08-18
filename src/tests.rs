//! Application-wide test module. Covers the .desktop parser/serializer, the
//! search filter, the XDG directory scanner, and launcher command sanitization.

use crate::models::{
    parse_desktop_file, serialize_desktop_entry, write_desktop_file, DesktopEntry,
};
use crate::services::desktop_service;
use crate::services::launcher_service::{clean_exec_command, launch_entry};
use crate::services::search_service;
use crate::ui::components::parse_semicolon_list;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "\
[Desktop Entry]
Type=Application
Name=Firefox
GenericName=Web Browser
Comment=Browse the web
Icon=firefox
Exec=firefox %u
Path=/usr/bin
Terminal=false
StartupNotify=true
NoDisplay=false
Categories=Network;WebBrowser;
Keywords=browser;internet;
MimeType=text/html;x-scheme-handler/http;
";

fn sample_entry(path: &Path) -> DesktopEntry {
    parse_desktop_file(SAMPLE, path).unwrap()
}

fn entry_named(
    name: &str,
    generic: &str,
    exec: &str,
    categories: &[&str],
    keywords: &[&str],
) -> DesktopEntry {
    let mut e = DesktopEntry::default();
    e.name = name.to_string();
    e.generic_name = generic.to_string();
    e.exec = exec.to_string();
    e.categories = categories.iter().map(|s| s.to_string()).collect();
    e.keywords = keywords.iter().map(|s| s.to_string()).collect();
    e
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("smm-test-{label}-{}", std::process::id()))
}

// --- models::desktop_entry --------------------------------------------------

#[test]
fn parse_sample_entries_all_fields() {
    let entry = sample_entry(Path::new("/tmp/firefox.desktop"));
    assert_eq!(entry.entry_type, "Application");
    assert_eq!(entry.name, "Firefox");
    assert_eq!(entry.generic_name, "Web Browser");
    assert_eq!(entry.comment, "Browse the web");
    assert_eq!(entry.icon, "firefox");
    assert_eq!(entry.exec, "firefox %u");
    assert_eq!(entry.path, "/usr/bin");
    assert!(!entry.terminal);
    assert!(entry.startup_notify);
    assert!(!entry.no_display);
    assert_eq!(entry.categories, vec!["Network", "WebBrowser"]);
    assert_eq!(entry.keywords, vec!["browser", "internet"]);
    assert_eq!(entry.mime_types, vec!["text/html", "x-scheme-handler/http"]);
    assert_eq!(
        entry.source_file,
        Some(Path::new("/tmp/firefox.desktop").to_path_buf())
    );
}

#[test]
fn parse_skips_comments_other_sections_and_crlf() {
    let content = "\r\n# a comment\r\n[Desktop Entry]\r\nName=Test\r\nType=Link\r\n[Other Section]\r\nIgnored=yes\r\n";
    let entry = parse_desktop_file(content, Path::new("/tmp/t.desktop")).unwrap();
    assert_eq!(entry.name, "Test");
    assert_eq!(entry.entry_type, "Link");
    assert_eq!(entry.generic_name, "");
}

#[test]
fn serialize_round_trip_preserves_fields() {
    let entry = sample_entry(Path::new("/tmp/firefox.desktop"));
    let reparsed = parse_desktop_file(
        &serialize_desktop_entry(&entry),
        Path::new("/tmp/firefox.desktop"),
    )
    .unwrap();
    assert_eq!(reparsed.name, entry.name);
    assert_eq!(reparsed.generic_name, entry.generic_name);
    assert_eq!(reparsed.comment, entry.comment);
    assert_eq!(reparsed.icon, entry.icon);
    assert_eq!(reparsed.exec, entry.exec);
    assert_eq!(reparsed.path, entry.path);
    assert_eq!(reparsed.categories, entry.categories);
    assert_eq!(reparsed.keywords, entry.keywords);
    assert_eq!(reparsed.mime_types, entry.mime_types);
    assert_eq!(reparsed.terminal, entry.terminal);
    assert_eq!(reparsed.startup_notify, entry.startup_notify);
    assert_eq!(reparsed.no_display, entry.no_display);
}

#[test]
fn serialize_omits_empty_optional_fields() {
    let mut entry = DesktopEntry::default();
    entry.name = "Empty".to_string();
    let text = serialize_desktop_entry(&entry);
    assert!(text.contains("Name=Empty"));
    assert!(text.contains("Exec="));
    assert!(!text.contains("GenericName="));
    assert!(!text.contains("Categories="));
    assert!(!text.contains("StartupWMClass="));
}

#[test]
fn link_type_serializes_url_not_exec() {
    let mut entry = DesktopEntry::default();
    entry.entry_type = "Link".to_string();
    entry.name = "Site".to_string();
    entry.url = "https://example.com".to_string();
    let text = serialize_desktop_entry(&entry);
    assert!(text.contains("Type=Link"));
    assert!(text.contains("URL=https://example.com"));
    assert!(!text.contains("Exec="));
}

#[test]
fn suggested_filename_sanitizes_names() {
    let mut entry = DesktopEntry::default();
    entry.name = "GTA San Andreas".to_string();
    assert_eq!(entry.suggested_filename(), "gta-san-andreas.desktop");

    entry.name = "  ...Firefox!!  ".to_string();
    assert_eq!(entry.suggested_filename(), "firefox.desktop");

    entry.name = String::new();
    assert_eq!(entry.suggested_filename(), "new-entry.desktop");
}

// --- services::search_service -----------------------------------------------

#[test]
fn search_matches_case_insensitively_across_fields() {
    let entries = vec![
        entry_named(
            "Firefox",
            "Web Browser",
            "firefox %u",
            &["Network", "WebBrowser"],
            &["internet", "mozilla"],
        ),
        entry_named("Discord", "Chat", "discord", &[], &["voip", "chat"]),
    ];
    assert!(search_service::matches(&entries[0], "FIREFOX"));
    assert!(search_service::matches(&entries[0], "browser"));
    assert!(search_service::matches(&entries[0], "mozilla"));
    assert!(search_service::matches(&entries[0], "web browser"));
    assert!(search_service::matches(&entries[0], "network"));
    assert!(search_service::matches(&entries[1], "voip"));
    assert!(!search_service::matches(&entries[0], "discord"));
    assert!(!search_service::matches(&entries[1], "firefox"));
    assert!(search_service::matches(&entries[0], "   "));
}

#[test]
fn search_filter_preserves_order() {
    let entries = vec![
        entry_named("Brave", "Browser", "brave", &[], &[]),
        entry_named("Firefox", "Browser", "firefox", &[], &[]),
        entry_named("VLC", "Player", "vlc", &[], &[]),
    ];
    let res = search_service::filter(&entries, "browser");
    let names: Vec<&str> = res.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["Brave", "Firefox"]);
}

// --- services::desktop_service ----------------------------------------------

#[test]
fn collect_desktop_files_walks_subdirectories() {
    let root = temp_dir("recursive");
    fs::create_dir_all(root.join("wine/Programs/GTA")).unwrap();
    fs::write(root.join("top.desktop"), "x").unwrap();
    fs::write(root.join("wine/program.desktop"), "x").unwrap();
    fs::write(root.join("wine/Programs/GTA/gta.desktop"), "x").unwrap();
    fs::write(root.join("ignored.txt"), "x").unwrap();
    fs::create_dir_all(root.join("empty-dir")).unwrap();

    let mut out = Vec::new();
    desktop_service::collect_desktop_files(&root, &mut out);
    let mut names: Vec<String> = out
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();
    names.sort();
    assert_eq!(names, vec!["gta.desktop", "program.desktop", "top.desktop"]);

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn write_and_read_desktop_file_round_trip() {
    let dir = temp_dir("write");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("app.desktop");
    let entry = sample_entry(&path);
    write_desktop_file(&entry, &path).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let reread = parse_desktop_file(&content, &path).unwrap();
    assert_eq!(reread.name, "Firefox");
    assert_eq!(reread.categories, entry.categories);

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn save_and_delete_entry_lifecycle() {
    let dir = temp_dir("lifecycle");
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("test-app.desktop");

    let mut entry = DesktopEntry::default();
    entry.name = "Lifecycle Test".to_string();
    entry.exec = "test-exec".to_string();
    entry.source_file = Some(file_path.clone());

    // Save
    let saved_path = desktop_service::save_entry(&entry).unwrap();
    assert_eq!(saved_path, file_path);
    assert!(file_path.exists());

    // Delete
    let delete_result = desktop_service::delete_entry(&entry);
    assert!(delete_result.is_ok());
    assert!(!file_path.exists());

    // Delete when already non-existent is idempotent
    assert!(desktop_service::delete_entry(&entry).is_ok());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn save_entry_with_refcell_state_does_not_panic() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let dir = temp_dir("refcell_save");
    fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("refcell-app.desktop");

    let mut entry = DesktopEntry::default();
    entry.name = "RefCell Save Test".to_string();
    entry.exec = "test-exec".to_string();
    entry.source_file = Some(file_path.clone());

    let state = Rc::new(RefCell::new(entry));

    let entry_to_save = state.borrow().clone();
    let target_path = desktop_service::save_entry(&entry_to_save).unwrap();
    state.borrow_mut().source_file = Some(target_path.clone());
    state.borrow_mut().directory = target_path.parent().map(|p| p.to_path_buf());

    assert_eq!(state.borrow().source_file, Some(file_path));

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn search_paths_deduplication() {
    let paths = desktop_service::search_paths();
    let mut seen = std::collections::HashSet::new();
    for p in &paths {
        assert!(seen.insert(p.clone()), "Duplicate path found: {:?}", p);
    }
}

// --- services::launcher_service ---------------------------------------------

#[test]
fn clean_exec_command_strips_percent_field_codes() {
    assert_eq!(clean_exec_command("firefox %u"), "firefox");
    assert_eq!(clean_exec_command("vlc --started-from-file %F"), "vlc --started-from-file");
    assert_eq!(clean_exec_command("gimp-2.10 %U %f"), "gimp-2.10");
    assert_eq!(clean_exec_command("myapp %i %c %k"), "myapp");
}

#[test]
fn launch_entry_fails_on_empty_command() {
    let entry = DesktopEntry::default();
    assert!(launch_entry(&entry).is_err());
}

// --- ui::components ---------------------------------------------------------

#[test]
fn parse_semicolon_list_splits_and_trims_empty() {
    assert_eq!(
        parse_semicolon_list("Network;WebBrowser;;Internet;"),
        vec!["Network", "WebBrowser", "Internet"]
    );
    assert_eq!(parse_semicolon_list(""), Vec::<String>::new());
    assert_eq!(parse_semicolon_list("   ; ; "), Vec::<String>::new());
}
