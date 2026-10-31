#!/usr/bin/env bash
# Copies what `tauri build --target $TARGET` produced into OUT_DIR under stable names: the
# installers, plus the bare binary (for Homebrew and portable use).
# Usage: TARGET=<rust target> collect-artifacts.sh VERSION OUT_DIR
set -euo pipefail

version=$1
out=$2
release="target/$TARGET/release"
bundle="$release/bundle"
mkdir -p "$out"

case "$TARGET" in
  *-apple-darwin)
    cp "$bundle"/dmg/*.dmg "$out/"
    tar -czf "$out/gicio-$version-$TARGET.tar.gz" -C "$release" gicio
    ;;
  *-linux-*)
    cp "$bundle"/deb/*.deb "$bundle"/rpm/*.rpm "$bundle"/appimage/*.AppImage "$out/"
    ;;
  *-windows-*)
    cp "$bundle"/msi/*.msi "$bundle"/nsis/*.exe "$out/"
    powershell -NoProfile -Command "Compress-Archive -Path '$release/gicio.exe' -DestinationPath '$out/gicio-$version-$TARGET.zip'"
    ;;
  *)
    echo "unknown target $TARGET" >&2
    exit 1
    ;;
esac

ls -la "$out"
