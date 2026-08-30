# macOS platform audit

> Decision update: the maintainer selected a no-fee public distribution policy after this audit. Current releases use ad-hoc signing without Apple notarization and document the per-app Gatekeeper exception; the Work context and release knowledge own the active policy.

## Baseline

- Host used for the initial audit: macOS 15.5 on Apple Silicon.
- The Rust workspace already selects `keyring`'s `apple-native` backend on macOS.
- External commands use a Windows-only `CREATE_NO_WINDOW` flag behind `cfg(windows)` and otherwise use normal Tokio processes.
- The Tauri bundle already contains an `.icns` icon and enables updater artifacts.
- The initial macOS workspace build needed a first-time dependency download; its final result belongs in the Work handoff rather than this audit.

## Concrete gaps

| Area | Current behavior | Required change |
| --- | --- | --- |
| Application data | Defaults to `.bdl` under the process working directory. Finder-launched apps cannot rely on that directory being stable or writable. | Initialize state with Tauri's platform application-data directory and retain a bounded legacy fallback. |
| Downloads | Defaults to a relative `downloads` directory. | Use the user's platform Downloads directory, namespaced as `BDL`, when no preference exists. |
| FFmpeg | Searches only the inherited `PATH`. Finder launches commonly omit Homebrew and MacPorts prefixes. | Search `PATH` first, then established macOS install locations; keep explicit selection authoritative. |
| Secure credentials | macOS dependency exists but has no CI compile coverage or manual Keychain check. | Compile/test on macOS and include Keychain save/load/delete in release QA. |
| File actions | Uses the Tauri opener abstraction and appears portable. | Preserve the adapter and smoke-test Finder open/reveal behavior. |
| Window shell | Disables native decorations and always renders Windows-style controls on the right. | Use an overlay macOS title bar with native traffic lights; hide web window controls and reserve left-side traffic-light space. |
| Developer gate | `check.ps1`, `package.ps1`, and `version.ps1` are the only maintained entry points. | Add POSIX check/package/version entry points or a shared cross-platform implementation; retain PowerShell compatibility. |
| Visual tests | Snapshot names and canonical rendering are Windows-specific. | Keep Windows as the canonical visual baseline; run deterministic non-visual frontend checks on macOS unless macOS baselines are deliberately added. |
| CI | Runs only on `windows-latest`. | Add a macOS job covering Rust format/Clippy/tests, frontend build/lint/tests, and a Tauri bundle smoke build. |
| Bundle configuration | One Windows-shaped window config; bundle target is `all`. | Add the Tauri macOS platform overlay and build universal `.app`/DMG output. |
| Release | Windows job owns release creation and publication. | Stage Windows and macOS artifacts into one draft, require both jobs, then publish once. |
| Signing/notarization | Only Tauri updater signing is documented. | Document and wire Apple certificate, Developer ID, notarization, and team credentials without committing secrets. |
| User docs | Installation and FFmpeg guidance are Windows-only; macOS remains community preview. | Add Homebrew/manual FFmpeg guidance, Gatekeeper expectations, build commands, artifact names, and the promoted support status after validation. |

## Acceptance matrix

| Capability | Automated evidence | Manual release evidence |
| --- | --- | --- |
| Rust/Tauri portability | macOS format, strict Clippy, workspace tests, and release bundle build pass. | App launches on a clean supported macOS account. |
| Frontend portability | Type/build, ESLint, Stylelint, formatting, and Vitest pass on macOS. | Native traffic lights, drag region, resizing, keyboard use, light/dark, and narrow layout work. |
| Storage and recovery | Tests cover injected platform defaults and legacy fallback selection. | Settings/tasks survive restart and an upgrade from an old `.bdl` directory. |
| FFmpeg | Unit coverage includes macOS fallback candidate ordering; environment probe finds an installed binary. | Explicit picker and Homebrew-discovered FFmpeg both mux MP4 and MKV. |
| Credentials | `apple-native` compiles and secure-store unit tests pass. | QR/cookie login survives restart, appears in Keychain, and logout removes it. |
| File integration | Existing opener command tests/build remain green. | Open file and open directory invoke Finder at the correct output. |
| Distribution | CI produces universal updater archive and DMG. | Developer ID signature, notarization, stapling, Gatekeeper assessment, install, update, and rollback checks pass. |

## Release credentials

Repository work can prepare and validate the pipeline, but a notarized public macOS release remains gated on maintainer-owned Apple Developer credentials and the existing Tauri updater private key. No secret material belongs in source, logs, Flightdeck, or build artifacts.
