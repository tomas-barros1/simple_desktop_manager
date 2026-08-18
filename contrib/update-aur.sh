#!/usr/bin/env bash
# ==============================================================================
# update-aur.sh - Update PKGBUILD version and checksum, then generate .SRCINFO
# Usage:
#   ./contrib/update-aur.sh <version>
# Example:
#   ./contrib/update-aur.sh 0.1.1
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKGBUILD_PATH="${SCRIPT_DIR}/PKGBUILD"
SRCINFO_PATH="${SCRIPT_DIR}/.SRCINFO"

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.1"
    exit 1
fi

NEW_VERSION="$1"
# Strip leading 'v' if present
NEW_VERSION="${NEW_VERSION#v}"

echo "==> Updating PKGBUILD to version ${NEW_VERSION}..."

# Update pkgver and reset pkgrel to 1
sed -i "s/^pkgver=.*/pkgver=${NEW_VERSION}/" "${PKGBUILD_PATH}"
sed -i "s/^pkgrel=.*/pkgrel=1/" "${PKGBUILD_PATH}"

# Fetch SHA256
URL="https://github.com/tomas-barros1/simple_desktop_manager/archive/refs/tags/v${NEW_VERSION}.tar.gz"
echo "==> Downloading archive and computing SHA256 from: ${URL}"

SHA256="$(curl -sL "${URL}" | sha256sum | awk '{print $1}')"

if [[ -z "${SHA256}" || "${#SHA256}" -ne 64 ]]; then
    echo "Error: Failed to compute valid SHA256 checksum (got '${SHA256}')" >&2
    exit 1
fi

echo "==> SHA256: ${SHA256}"

# Update sha256sums in PKGBUILD
sed -i "s/^sha256sums=.*/sha256sums=('${SHA256}')/" "${PKGBUILD_PATH}"

# Generate .SRCINFO if makepkg is available
if command -v makepkg &> /dev/null; then
    echo "==> Generating .SRCINFO using makepkg..."
    (cd "${SCRIPT_DIR}" && makepkg --printsrcinfo > "${SRCINFO_PATH}")
    echo "==> .SRCINFO successfully generated!"
else
    echo "Note: 'makepkg' not installed on this system. Generating basic .SRCINFO template..."
    cat <<EOF > "${SRCINFO_PATH}"
pkgbase = simple-menu-manager
	pkgdesc = Modern, lightweight Linux Desktop Entry (.desktop) editor written in Rust with GTK4 & LibAdwaita
	pkgver = ${NEW_VERSION}
	pkgrel = 1
	url = https://github.com/tomas-barros1/simple_desktop_manager
	arch = x86_64
	arch = aarch64
	license = MIT
	makedepends = cargo
	makedepends = rust
	makedepends = pkgconf
	depends = gtk4
	depends = libadwaita
	depends = glibc
	depends = gcc-libs
	source = simple-menu-manager-${NEW_VERSION}.tar.gz::https://github.com/tomas-barros1/simple_desktop_manager/archive/refs/tags/v${NEW_VERSION}.tar.gz
	sha256sums = ${SHA256}

pkgname = simple-menu-manager
EOF
fi

echo ""
echo "=========================================================================="
echo "✅ AUR files updated successfully for version ${NEW_VERSION}!"
echo "   - PKGBUILD: ${PKGBUILD_PATH}"
echo "   - .SRCINFO: ${SRCINFO_PATH}"
echo ""
echo "To publish to AUR:"
echo "  1. Clone your AUR repo: git clone ssh://aur@aur.archlinux.org/simple-menu-manager.git /tmp/aur-pkg"
echo "  2. Copy PKGBUILD and .SRCINFO to /tmp/aur-pkg/"
echo "  3. Commit and push: cd /tmp/aur-pkg && git add PKGBUILD .SRCINFO && git commit -m 'Release v${NEW_VERSION}' && git push"
echo "=========================================================================="
