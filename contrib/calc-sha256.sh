#!/usr/bin/env bash
# ==============================================================================
# calc-sha256.sh - Calculate SHA256 checksums for local files, tags, or release tarballs.
# Usage:
#   ./contrib/calc-sha256.sh <file_or_url>
#   ./contrib/calc-sha256.sh --tag <tag_name>
# ==============================================================================

set -euo pipefail

REPO_URL="https://github.com/tomas-barros1/simple_desktop_manager"

if [[ $# -eq 0 ]]; then
    echo "Usage:"
    echo "  $0 <path_to_file>"
    echo "  $0 <url>"
    echo "  $0 --tag <version_tag> (e.g. v0.1.1)"
    echo "  $0 --release <version> (e.g. 0.1.1)"
    exit 1
fi

TARGET="$1"

if [[ "$TARGET" == "--tag" ]]; then
    if [[ $# -lt 2 ]]; then
        echo "Error: missing tag name. Example: $0 --tag v0.1.1" >&2
        exit 1
    fi
    TAG="$2"
    URL="${REPO_URL}/archive/refs/tags/${TAG}.tar.gz"
    echo "==> Fetching and computing SHA256 for tag source: ${URL}"
    curl -sL "${URL}" | sha256sum | awk '{print $1}'
elif [[ "$TARGET" == "--release" ]]; then
    if [[ $# -lt 2 ]]; then
        echo "Error: missing version. Example: $0 --release 0.1.1" >&2
        exit 1
    fi
    VER="$2"
    URL="${REPO_URL}/releases/download/v${VER}/simple-menu-manager-v${VER}-x86_64-linux.tar.gz"
    echo "==> Fetching and computing SHA256 for release binary: ${URL}"
    curl -sL "${URL}" | sha256sum | awk '{print $1}'
elif [[ "$TARGET" =~ ^https?:// ]]; then
    echo "==> Fetching and computing SHA256 for URL: ${TARGET}"
    curl -sL "${TARGET}" | sha256sum | awk '{print $1}'
elif [[ -f "$TARGET" ]]; then
    echo "==> Computing SHA256 for local file: ${TARGET}"
    sha256sum "$TARGET" | awk '{print $1}'
else
    echo "Error: target not found: ${TARGET}" >&2
    exit 1
fi
