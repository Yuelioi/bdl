#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
desktop_dir="$repo_root/apps/desktop"
skip_check=false
updater_artifacts=false
tauri_args=()
tauri_args_count=0

while (($# > 0)); do
  case "$1" in
    --skip-check) skip_check=true ;;
    --updater-artifacts) updater_artifacts=true ;;
    *)
      tauri_args+=("$1")
      tauri_args_count=$((tauri_args_count + 1))
      ;;
  esac
  shift
done

run_step() {
  local name="$1"
  shift
  printf '\n==> %s\n' "$name"
  "$@"
}

cd "$repo_root"

if [[ "$skip_check" == false ]]; then
  run_step "Workspace checks" "$script_dir/check.sh"
fi

build_args=(build)
if [[ "$updater_artifacts" == false ]]; then
  build_args+=(--config '{"bundle":{"createUpdaterArtifacts":false}}')
fi
if ((tauri_args_count > 0)); then
  build_args+=("${tauri_args[@]}")
fi
run_step "Tauri package" env CI=true pnpm --dir "$desktop_dir" tauri "${build_args[@]}"

artifact_list="$(mktemp -t bdl-package-artifacts.XXXXXX)"
trap 'rm -f "$artifact_list"' EXIT
find "$repo_root/target" \
  \( -type d -name '*.app' -o -type f \( -name '*.dmg' -o -name '*.app.tar.gz' -o -name '*.sig' -o -name '*.exe' -o -name '*.msi' -o -name '*.msix' -o -name '*.zip' -o -name '*.deb' -o -name '*.rpm' -o -name '*.AppImage' \) \) \
  -print | sort -u > "$artifact_list"

if [[ ! -s "$artifact_list" ]]; then
  echo "Tauri build completed but no package artifacts were found under $repo_root/target." >&2
  exit 1
fi

printf '\nArtifacts:\n'
while IFS= read -r artifact; do
  printf -- '- %s\n' "$artifact"
done < "$artifact_list"
