#!/usr/bin/env bash
# System libraries Tauri needs to build gicio on Debian/Ubuntu.
set -euo pipefail

sudo=$(command -v sudo || true)
$sudo apt-get update
$sudo apt-get install -y --no-install-recommends \
  build-essential \
  file \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  patchelf
