#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

BINARY_PATH="${1:-${ROOT_DIR}/target/release/mysql-gui}"
OUTPUT_PATH="${2:-${ROOT_DIR}/mysql-gui-x86_64.AppImage}"

if [ ! -f "${BINARY_PATH}" ]; then
    echo "Error: Binary not found at ${BINARY_PATH}"
    echo "Run 'cargo build --release' first or pass the path to mysql-gui as arg 1."
    exit 1
fi

APPDIR="$(mktemp -d /tmp/mysql-gui-appdir.XXXXXX)"
cleanup() {
    rm -rf "${APPDIR}"
}
trap cleanup EXIT

echo "--> Preparing AppDir at ${APPDIR}..."
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/lib"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"

# Copy binary
cp "${BINARY_PATH}" "${APPDIR}/usr/bin/mysql-gui"
chmod +x "${APPDIR}/usr/bin/mysql-gui"

# Copy AppRun
cp "${SCRIPT_DIR}/AppRun" "${APPDIR}/AppRun"
chmod +x "${APPDIR}/AppRun"

# Copy desktop file
cp "${ROOT_DIR}/assets/mysql-gui.desktop" "${APPDIR}/mysql-gui.desktop"
cp "${ROOT_DIR}/assets/mysql-gui.desktop" "${APPDIR}/usr/share/applications/mysql-gui.desktop"

# Copy icons
cp "${ROOT_DIR}/assets/icons/mysql-gui.png" "${APPDIR}/mysql-gui.png"
cp "${ROOT_DIR}/assets/icons/mysql-gui.png" "${APPDIR}/.DirIcon"
cp "${ROOT_DIR}/assets/icons/mysql-gui.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/mysql-gui.png"

# Obtain appimagetool if not in PATH
APPIMAGETOOL=""
if command -v appimagetool >/dev/null 2>&1; then
    APPIMAGETOOL="appimagetool"
else
    CACHE_TOOL="/tmp/appimagetool-x86_64.AppImage"
    if [ ! -f "${CACHE_TOOL}" ]; then
        echo "--> Downloading appimagetool..."
        curl -sLo "${CACHE_TOOL}" "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
        chmod +x "${CACHE_TOOL}"
    fi
    APPIMAGETOOL="${CACHE_TOOL}"
fi

echo "--> Building AppImage to ${OUTPUT_PATH}..."
# In Docker/CI or environments without FUSE, pass --appimage-extract-and-run
if [ -n "${APPIMAGE_EXTRACT_AND_RUN:-}" ] || ! [ -c /dev/fuse ]; then
    ARCH=x86_64 "${APPIMAGETOOL}" --appimage-extract-and-run --no-appstream "${APPDIR}" "${OUTPUT_PATH}"
else
    ARCH=x86_64 "${APPIMAGETOOL}" --no-appstream "${APPDIR}" "${OUTPUT_PATH}"
fi

echo "--> AppImage created successfully: ${OUTPUT_PATH}"
ls -lh "${OUTPUT_PATH}"
