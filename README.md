# Desktop Manager (`simple_menu_manager`)

> A modern, lightweight Linux Desktop Entry (`.desktop` file) editor written in **Rust** using **GTK4** and **LibAdwaita**.

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)
![GTK4](https://img.shields.io/badge/GTK-4.10%2B-blue.svg)
![LibAdwaita](https://img.shields.io/badge/LibAdwaita-1.4%2B-purple.svg)
![License](https://img.shields.io/badge/license-MIT-green.svg)

---

## Features

- 🔍 **XDG Application Scanner**: Automatically loads and merges `.desktop` files from system (`/usr/share/applications`) and user paths (`~/.local/share/applications`).
- 🖱️ **Single-Click Editing**: Click any application in the sidebar to open its editor panel instantly.
- 🌍 **Automatic System i18n**: Seamlessly detects system language (`LANG` / `LC_ALL`) and provides native dictionaries for **English** and **Brazilian Portuguese (`pt_BR`)**.
- ⚙️ **Categorized Form Editor**: Organized into 4 structured sections:
  1. **General**: Type, Name, Generic Name, Comment
  2. **Execution**: Exec, Path, In Terminal (Checkbox), URL
  3. **Appearance & Classification**: Icon, Categories, Keywords, MIME Types
  4. **Advanced Options**: Startup Notify (Checkbox), WM Class (`StartupWMClass`), Hidden (Checkbox)
- 🚀 **Direct App Launcher**: Test applications instantly using the Play button (`▶`) in the editor footer.
- 🗑️ **Delete Confirmation Popup**: Safety popup (`adw::MessageDialog`) preventing accidental deletion.
- 🎨 **Dark Modern Aesthetics**: Polished GTK4 dark theme matching modern Linux desktop environments.

---

## Prerequisites

Before building, make sure system dependencies for GTK4 and LibAdwaita are installed:

### Ubuntu / Debian / Pop!_OS
```bash
sudo apt update
sudo apt install -y libgtk-4-dev libadwaita-1-dev pkg-config build-essential
```

### Fedora / RHEL
```bash
sudo dnf install -y gtk4-devel libadwaita-devel pkgconf-pkg-config
```

### Arch Linux / Manjaro
```bash
sudo pacman -S gtk4 libadwaita pkgconf
```

---

## Building & Installation

### Using Makefile

```bash
# Build debug binary
make

# Build release binary
make release

# Install to system (/usr/local/bin & /usr/local/share/applications)
sudo make install

# Uninstall from system
sudo make uninstall
```

### Using Cargo

```bash
# Run locally
cargo run

# Build optimized release binary
cargo build --release
```

---

## GitHub Releases & CI/CD

Automated releases and continuous integration workflows are configured:
- `.github/workflows/ci.yml`: Runs `cargo check`, `cargo test`, and `cargo build` on every push.
- `.github/workflows/release.yml`: Builds optimized release tarballs and publishes GitHub Releases automatically whenever a new version tag (e.g. `v0.1.0`) is pushed.

---

## Contributing & Development

Refer to [AGENTS.md](./AGENTS.md) for module architecture, guidelines, and code conventions.
