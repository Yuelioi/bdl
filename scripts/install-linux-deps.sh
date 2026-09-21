#!/usr/bin/env bash
set -euo pipefail

# Ubuntu/Debian development dependencies. Keep release builds on Ubuntu 22.04
# so newer builders do not silently raise the minimum glibc requirement.
if [[ "$(uname -s)" != Linux ]] || ! command -v apt-get >/dev/null; then
  echo 'This helper requires an Ubuntu/Debian Linux environment.' >&2
  exit 1
fi

elevate=()
if ((EUID != 0)); then elevate=(sudo); fi
"${elevate[@]}" apt-get update
"${elevate[@]}" apt-get install -y --no-install-recommends \
  build-essential pkg-config curl wget file patchelf \
  libwebkit2gtk-4.1-dev libssl-dev libxdo-dev \
  libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev \
  ffmpeg xdg-utils
