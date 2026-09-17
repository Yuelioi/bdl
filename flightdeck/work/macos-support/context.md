# macOS support context

## Product intent

macOS users should be able to install and run the same local-first Bilibili download workflow as Windows users, including parsing, durable transfers, FFmpeg finalization, secure account storage, file reveal/open actions, and signed application updates where release credentials are available.

## Stable constraints

- Preserve the Rust-owned downloader pipeline and stable Tauri command/event contracts.
- Keep Windows behavior and packaging working while introducing platform-specific behavior behind narrow configuration or compile-time seams.
- Cookies must never be stored as plaintext, in SQLite, logs, diagnostics, or frontend state. Under the current no-fee ad-hoc release policy, macOS uses application-owned ChaCha20-Poly1305 encrypted credential files with private filesystem permissions instead of Keychain, avoiding authorization prompts across unsigned/ad-hoc updates.
- FFmpeg may be selected explicitly or discovered from the effective application environment; macOS GUI launches cannot assume an interactive shell `PATH`.
- Repository checks must be runnable from macOS without PowerShell as the only entry point.
- The current public macOS release policy must not require a paid Apple Developer membership: packages use an explicit ad-hoc identity and user-facing Gatekeeper instructions. Tauri updater publication still requires the maintainer-owned updater key, which must never be embedded in source.

## Initial target

- Development and automated CI on current macOS runners.
- Tauri `.app` and DMG bundles for Apple Silicon and Intel where dependencies support both targets.
- Platform-correct encrypted credential persistence, window controls, paths, FFmpeg execution, file reveal/open behavior, and updater artifact metadata.
- Documentation that clearly identifies downloadable macOS packages as ad-hoc signed and not Apple-notarized, including the safe per-app Gatekeeper exception flow.

## Out of scope

- Bundling FFmpeg binaries or codecs into the application unless the audit proves the existing external-FFmpeg model cannot provide a usable macOS experience.
- Purchasing an Apple Developer membership or creating signing/notarization credentials on behalf of the maintainer.
