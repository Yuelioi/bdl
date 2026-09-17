# macOS support

## Goal

Make BDL a supported macOS desktop application with a reproducible development path, platform-correct runtime integrations, automated checks, distributable application bundles, and accurate user documentation, without weakening the existing Windows experience.

## Status

Open

## Current

`main` is synchronized with `origin/main` at `2189ca6`, including the released v0.3.0 macOS implementation and its deliberate no-fee ad-hoc signing policy. The working tree removes macOS Keychain access entirely: macOS persists the account Cookie with ChaCha20-Poly1305 under the application-data directory, using a separate random key file and private filesystem permissions. Windows and Linux keep their existing OS credential backends. No pre-release credential format is migrated because the application has no deployed user base. Startup restores the non-secret account summary first and loads the persistent Cookie lazily when authenticated state is needed. Strict Clippy passes and the `bdl-tauri` test suite passes on Windows.

## Next

Validate the encrypted macOS credential backend on a clean Mac: log in, restart BDL, confirm the account survives, log out and confirm the credential files are removed, and confirm normal use never triggers a Keychain password dialog.

## Execution

The v0.3.0 release workflow is complete and still verifies `Signature=adhoc`. The encrypted macOS credential change is present only in the working tree and has not been committed or published. Other pre-existing local edits remain untouched.

## Progress

- Created a dedicated durable Work for macOS support.
- Confirmed the current public support matrix lists macOS as community preview with no CI or release bundle.
- Audited the runtime, UI shell, scripts, workflows, bundle configuration, tests, and release documentation; captured the acceptance matrix and concrete gaps.
- Moved persistent state to Tauri's platform application-data directory with a legacy `.bdl` fallback and defaulted downloads to the operating system Downloads directory.
- Added macOS FFmpeg fallback discovery, native title-bar behavior, a macOS bundle overlay, portable check/package/version helpers, macOS CI, and dual-architecture release jobs.
- Passed the complete local check gate and 127 frontend tests; built and verified an arm64 `.app` and DMG; completed a GUI smoke run with native controls and Homebrew FFmpeg discovery.
- Hardened local HTTP tests for macOS connection closure and made loopback tests bypass host proxy settings deterministically.
- Installed the Intel Rust target through the local proxy and a faster mirror, passed `cargo check --workspace --all-targets --target x86_64-apple-darwin --locked`, and produced a signed x86_64 `.app` with the expected version and deployment target.
- Kept the pre-bump v0.2.1 arm64 app running for a real UI review; the window uses native traffic lights and successfully reloads persisted application data.
- Bumped all release version sources from `0.2.1` to `0.3.0`; tag verification and the complete macOS check gate pass at the new version.
- Replaced the paid Apple credential gate with an explicit ad-hoc release policy, removed all `APPLE_*` workflow inputs, and expanded cross-platform installation documentation with safe macOS Gatekeeper instructions.
- Re-ran the complete check gate, built `BDL_0.3.0_aarch64.dmg`, verified its checksum and `Signature=adhoc`, and rebuilt the Intel application as a signed Mach-O x86_64 bundle at version `0.3.0`.
- Pushed `9e3189c` and annotated tag `v0.3.0`; both macOS Release matrix jobs, the Windows Release job, and all updater artifact uploads succeeded.
- Fixed macOS CI for runners without ripgrep in `d7d4536`; the latest hosted Windows and macOS CI jobs pass.
- Fixed draft publication lookup in `fc5beee`, recovered the already-staged release once, verified eleven public assets, and removed the one-time recovery workflow.
- Synchronized the local `main` branch from `26866f9` to `2189ca6`, preserving and restoring the pre-existing working-tree edits around the upstream macOS changes.
- Reworked startup account restoration locally so opening BDL does not read credential secrets immediately; Cookie access is deferred until authenticated state is actually needed.
- Replaced the macOS `apple-native` Keychain backend with an application-owned ChaCha20-Poly1305 encrypted file backend. The random 32-byte key and encrypted Cookie are separate private files and logout removes both. Pre-release credential formats are intentionally unsupported. `cargo clippy -p bdl-tauri --all-targets -- -D warnings` and the `bdl-tauri` library tests pass on Windows.

## References

- [Context](context.md)
- [Plan](plan.md)
- [Architecture knowledge](../../knowledge/bdl-downloader/architecture.md)
- [Release knowledge](../../knowledge/bdl-downloader/release-and-updates.md)
- [Supported platforms](../../../docs/SUPPORTED_PLATFORMS.md)
- [Platform audit](references/platform-audit.md)
