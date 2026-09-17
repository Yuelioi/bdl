#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
cargo_target_dir="$(node "$script_dir/cargo-target.mjs" "$repo_root")"
export CARGO_TARGET_DIR="$cargo_target_dir"
skip_clippy=false
run_visual=false

while (($# > 0)); do
  case "$1" in
    --skip-clippy) skip_clippy=true ;;
    --visual) run_visual=true ;;
    *)
      echo "Unknown option: $1" >&2
      echo "Usage: ./scripts/check.sh [--skip-clippy] [--visual]" >&2
      exit 2
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

verify_windows_child_process_policy() {
  local helper="$repo_root/crates/bdl-core/src/process.rs"
  local raw_launches
  if command -v rg >/dev/null 2>&1; then
    raw_launches="$(cd "$repo_root/crates" && rg -n 'Command::new[[:space:]]*\(' -g '*.rs' -g '!bdl-core/src/process.rs' || true)"
  else
    raw_launches="$(find "$repo_root/crates" -type f -name '*.rs' ! -path "$helper" -exec grep -nHE 'Command::new[[:space:]]*\(' {} + || true)"
  fi
  if [[ -n "$raw_launches" ]]; then
    echo "Runtime child processes bypass the hidden-window command helper:" >&2
    echo "$raw_launches" >&2
    return 1
  fi
  if command -v rg >/dev/null 2>&1; then
    rg -q 'CREATE_NO_WINDOW' "$helper"
    rg -q '\.creation_flags[[:space:]]*\(' "$helper"
  else
    grep -q 'CREATE_NO_WINDOW' "$helper"
    grep -Eq '\.creation_flags[[:space:]]*\(' "$helper"
  fi
  echo "Windows child-process policy verified."
}

run_rust_tests() {
  local loopback_no_proxy="127.0.0.1,localhost"
  if [[ -n "${NO_PROXY:-}" ]]; then
    loopback_no_proxy="${NO_PROXY},${loopback_no_proxy}"
  fi
  env NO_PROXY="$loopback_no_proxy" no_proxy="$loopback_no_proxy" cargo test --workspace
}

cd "$repo_root"

printf 'Cargo target: %s\n' "$CARGO_TARGET_DIR"

run_step "Rust format" cargo fmt --all --check
if [[ "$skip_clippy" == false ]]; then
  run_step "Rust clippy" cargo clippy --workspace --all-targets -- -D warnings
fi
run_step "Rust tests" run_rust_tests
run_step "Windows child process policy" verify_windows_child_process_policy
run_step "Desktop frontend build" pnpm --dir apps/desktop build
run_step "Desktop code lint" pnpm --dir apps/desktop lint
run_step "Desktop style lint" pnpm --dir apps/desktop lint:styles
run_step "Desktop style format" pnpm --dir apps/desktop format:styles:check
run_step "Desktop frontend tests" pnpm --dir apps/desktop test

if [[ "$run_visual" == true ]]; then
  run_step "Desktop visual regression tests" pnpm --dir apps/desktop test:visual
else
  printf '\n==> Desktop visual regression tests\nSkipped; Windows remains the canonical snapshot platform. Pass --visual to run locally.\n'
fi

run_step "Git whitespace check" git diff --check
printf '\nAll checks passed.\n'
