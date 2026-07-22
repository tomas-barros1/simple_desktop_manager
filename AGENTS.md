# AGENTS.md

## Overview

`simple_menu_manager` is a modern, lightweight Linux Desktop Entry (`.desktop` file) editor written in **Rust** using **GTK4** and **LibAdwaita**. It scans system and user XDG application directories, allowing users to browse, search, edit, create, run, and delete application shortcuts with full automatic internationalization (i18n) support.

---

## Project Structure

```
simple_menu_manager/
├── Cargo.toml                  # Project metadata & dependencies (GTK4, LibAdwaita, Serde, Tokio)
├── Cargo.lock                  # Dependency lockfile
├── i18n/                       # Translation JSON files
│   ├── en.json                 # English locale dictionary
│   └── pt_BR.json              # Brazilian Portuguese locale dictionary
├── src/
│   ├── main.rs                 # Application entry point & logging setup
│   ├── app/
│   │   ├── mod.rs              # App module exports
│   │   ├── application.rs      # adw::Application / gtk4::Application setup
│   │   └── window.rs           # Main ApplicationWindow layout & CSS provider
│   ├── models/
│   │   ├── mod.rs              # Models re-exports
│   │   └── desktop_entry.rs    # DesktopEntry struct, .desktop parser & serializer
│   ├── services/
│   │   ├── mod.rs              # Services module exports
│   │   ├── desktop_service.rs  # XDG path scanner, load/save/delete file operations
│   │   ├── icon_cache.rs       # GTK IconTheme lookup & image rendering cache
│   │   ├── i18n.rs             # Internationalization manager & system locale auto-detection
│   │   └── search_service.rs   # Case-insensitive entry search & filtering
│   └── ui/
│       ├── mod.rs              # UI components module exports
│       ├── components.rs       # Reusable GTK widgets
│       ├── editor.rs           # Form panel divided into styled sections (Execução, Aparência, Opções Avançadas)
│       └── sidebar.rs          # Searchable list sidebar with two-line item rendering and bottom action bar
└── AGENTS.md                   # Guidance document for AI agents and maintainers
```

---

## Internationalization (i18n)

- **Translation Files**: Located in `i18n/en.json` (English) and `i18n/pt_BR.json` (Brazilian Portuguese).
- **Service (`src/services/i18n.rs`)**:
  - Embedded via `include_str!` for zero-dependency runtime reliability.
  - Auto-detects system language at startup from environment variables (`LANG`, `LC_ALL`, `LC_MESSAGES`).
  - No language toggle in the UI; language automatically mirrors system settings.
  - Convenient macro/function `t("key")` for retrieving localized strings.

---

## User Interface & Features

- **Sidebar**:
  - Top search entry ("Pesquisar aplicativos...").
  - Two-line app item rendering (Title + Subtitle comment/generic name).
  - Bottom action bar with `+` (New entry) and pink/red Trash icon (Delete button).
- **Editor**:
  - Divided into 4 styled sections matching modern dark GTK desktop aesthetics:
    1. **General**: Type, Name, Generic Name, Comment
    2. **Execução**: Exec, Path, In Terminal (Switch), URL
    3. **Aparência e Classificação**: Icon, Categories, Keywords, Mime Types
    4. **Opções Avançadas**: Startup Notify (Switch), WM Class, Hidden (Switch)
  - Bottom action bar with **Run** (`▶`), **Cancel**, and **Save** buttons.
- **Delete Confirmation**:
  - Clicking Delete triggers an `adw::MessageDialog` popup asking for explicit confirmation before removing files.

---

## Development Workflows

- **Type Check**: `cargo check`
- **Build**: `cargo build`
- **Test**: `cargo test`
- **Run**: `cargo run`

---

## Guidelines for Contributors & AI Agents

1. **GTK4 / Rust Ownership**:
   - When downcasting GTK objects in signal closures, prefer owned `.downcast::<T>()` over `.downcast_ref::<T>()` on temporary objects to avoid lifetime borrowing issues.
2. **Localization Consistency**:
   - Keep keys in sync between `i18n/en.json` and `i18n/pt_BR.json`.
