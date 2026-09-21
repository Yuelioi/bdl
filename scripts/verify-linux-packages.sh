#!/usr/bin/env bash
set -euo pipefail
bundle_dir="${1:?Usage: verify-linux-packages.sh path/to/bundle}"
shopt -s nullglob
deb_files=("$bundle_dir"/deb/*.deb)
appimages=("$bundle_dir"/appimage/*.AppImage)
if ((${#deb_files[@]} != 1 || ${#appimages[@]} != 1)); then
  echo 'Expected exactly one deb and one AppImage in a clean Linux build.' >&2
  exit 1
fi
[[ "$(dpkg-deb -f "${deb_files[0]}" Architecture)" == amd64 ]]
depends="$(dpkg-deb -f "${deb_files[0]}" Depends)"
[[ "$depends" == *ffmpeg* && "$depends" == *xdg-utils* && "$depends" == *libwebkit2gtk-4.1* ]]
contents="$(dpkg-deb --contents "${deb_files[0]}")"
[[ "$contents" == *usr/bin/bdl-desktop* && "$contents" == *share/applications/* ]]
[[ -s "${appimages[0]}" ]]
file "${appimages[0]}" | grep -q 'ELF 64-bit.*x86-64'
echo 'Linux package architecture, runtime dependencies and desktop entry verified.'
