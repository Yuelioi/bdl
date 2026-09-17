# macOS support plan

- [x] Audit platform assumptions and define a concrete macOS acceptance matrix.
- [x] Make development checks and helper scripts work on macOS while preserving Windows entry points.
- [x] Fix Rust/Tauri runtime integrations for macOS paths, credential persistence, FFmpeg, file actions, child processes, and updater behavior.
- [ ] Validate application-owned encrypted credentials on a clean Mac, including restart persistence, logout deletion, and no Keychain prompt.
- [x] Adapt and verify the desktop shell, title bar, native window controls, permissions, icons, and bundle configuration on macOS.
- [x] Add macOS CI for Rust and frontend checks plus package smoke builds.
- [x] Add macOS release bundles and document signing, notarization, architecture choices, and updater metadata.
- [x] Run the full feasible macOS validation matrix, update support documentation, and record release checks for the selected no-fee policy.

## Validation record

- `./scripts/check.sh` passes on macOS 15.5 Apple Silicon, including clippy, the complete Rust workspace, frontend build/lints, and 127 Vitest tests across 38 files.
- `./scripts/package.sh --skip-check` produces an ad-hoc signed version `0.3.0` `BDL.app` and `BDL_0.3.0_aarch64.dmg`; `codesign --verify --deep --strict`, the explicit `Signature=adhoc` check, and `hdiutil verify` pass.
- A GUI smoke run confirms native macOS traffic lights, no duplicate Windows controls, the default `~/Downloads/BDL` destination, application data under `~/Library/Application Support/com.yueli.bdl`, and Homebrew FFmpeg discovery at `/opt/homebrew/bin/ffmpeg`.
- The Intel release leg is configured in GitHub Actions and locally cross-verified: the full workspace checks for `x86_64-apple-darwin`, and the packaged application contains an ad-hoc signed Mach-O x86_64 binary with version `0.3.0` and minimum macOS `13.0`.
- All release version sources are synchronized at `0.3.0`; `node scripts/verify-release-tag.mjs v0.3.0` and the complete `./scripts/check.sh` gate pass.
- The current release policy deliberately uses ad-hoc signed, unnotarized macOS packages. The workflow needs the existing Tauri updater private key but no paid Apple membership or `APPLE_*` secrets; release text and README disclose the Gatekeeper behavior.
- [BDL v0.3.0](https://github.com/Yuelioi/bdl/releases/tag/v0.3.0) is public with eleven staged assets, including Apple Silicon and Intel DMGs, their signed updater archives, Windows installers, and `latest.json`; all three Release build jobs succeeded.
- [BDL v0.3.1](https://github.com/Yuelioi/bdl/releases/tag/v0.3.1) is public with eleven assets, including Apple Silicon and Intel DMGs, signed updater archives, Windows installers, and `latest.json`; all three platform builds and the publish job succeeded.
- Hosted CI exposed and verified two release-operations fixes: `check.sh` now falls back to `find`/`grep` when ripgrep is unavailable, and draft publication locates authenticated draft releases through `listReleases`. The latest Windows and macOS CI jobs pass.
