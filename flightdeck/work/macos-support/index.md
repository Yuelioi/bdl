# macOS support

## Goal

Make BDL a supported macOS desktop application with a reproducible development path, platform-correct runtime integrations, automated checks, distributable application bundles, and accurate user documentation, without weakening the existing Windows experience.

## Status

Open

## Current

The macOS implementation and no-fee distribution policy are complete and locally verified on macOS 15.5 Apple Silicon. The application uses native app-data and Downloads locations, discovers common Homebrew/MacPorts FFmpeg installs, renders native macOS window controls, and passes the cross-platform check script. Version `0.3.0` produces a valid ad-hoc signed Apple Silicon `.app` and DMG plus an ad-hoc signed Intel `x86_64-apple-darwin` `.app`; CI and release workflows cover both architectures without Apple credentials. GitHub Releases explicitly identify Mac packages as unnotarized and direct users to a safe per-app Gatekeeper exception flow.

## Next

Push the branch to run GitHub-hosted macOS CI (including the Intel target), then cut tag `v0.3.0` and confirm both architecture downloads, the documented Gatekeeper flow, updater metadata, and a real Keychain-backed login on a clean Mac.

## Execution

Implementation is complete. Remaining execution needs the existing Tauri updater key, a pushed GitHub ref, and clean-Mac verification of the public unnotarized package flow; Apple credentials are intentionally not required.

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

## References

- [Context](context.md)
- [Plan](plan.md)
- [Architecture knowledge](../../knowledge/bdl-downloader/architecture.md)
- [Release knowledge](../../knowledge/bdl-downloader/release-and-updates.md)
- [Supported platforms](../../../docs/SUPPORTED_PLATFORMS.md)
- [Platform audit](references/platform-audit.md)
