# BDL release and update operations

## Version source of truth

Run the repository helper from the root:

```powershell
./scripts/version.ps1 patch
./scripts/version.ps1 minor
./scripts/version.ps1 major
./scripts/version.ps1 0.2.0
```

On macOS or Linux, use the equivalent Bash entry point:

```bash
./scripts/version.sh patch
./scripts/version.sh minor
./scripts/version.sh major
./scripts/version.sh 0.2.0
```

The helper refuses inconsistent starting versions and synchronizes all Rust package manifests, `apps/desktop/package.json`, `apps/desktop/src-tauri/tauri.conf.json`, `Cargo.lock`, and the README version. Do not edit only one manifest.

## Release sequence

```powershell
./scripts/version.ps1 patch
./scripts/check.ps1
git add .
git commit -m "release v0.1.1"
git tag v0.1.1
git push origin main --tags
```

The `.github/workflows/release.yml` workflow builds Windows x64 plus separate macOS Apple Silicon and Intel bundles with `tauri-apps/tauri-action`. Each matrix leg stages signed updater artifacts in one draft GitHub Release; a dependent job publishes it only after every build and signature check succeeds.

GitHub's `getReleaseByTag` endpoint does not resolve the staged draft reliably. The publish job must use the authenticated `listReleases` endpoint, select the matching `tag_name`, and then call `updateRelease` with that draft's numeric ID. Keep publication dependent on every matrix build so a partial release never becomes public.

On macOS, the same sequence uses `./scripts/version.sh` and `./scripts/check.sh`. `./scripts/package.sh` is the local package entry point; it disables updater artifacts unless `--updater-artifacts` is passed so a developer can build without the private updater key.

## Signing trust root

- Tauri updater signatures are mandatory and separate from an optional paid Windows Authenticode certificate.
- The public key is embedded in `apps/desktop/src-tauri/tauri.conf.json` and may be committed.
- The private key currently lives at `C:\Users\yl\.tauri\bdl.key`; never copy its contents into the repository, Flightdeck, logs, issues, or chat.
- GitHub Actions needs the private key as the `TAURI_SIGNING_PRIVATE_KEY` repository secret. The current key has no password, so `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` may be absent.
- Keep at least one offline backup of the private key. Losing it prevents future releases from updating already-installed clients. Rotating it requires an explicitly designed trust migration.

The project's current no-fee macOS distribution policy uses the ad-hoc identity `-` for local and GitHub-hosted release builds. Release CI must not require `APPLE_*` secrets and must verify both `codesign --verify --deep --strict` and `Signature=adhoc`. Every release and the README must disclose that the DMG is not Apple-notarized and document the per-app Control-click/Open or Privacy & Security/Open Anyway flow. Never advise users to disable Gatekeeper globally. If the maintainer later adopts paid Developer ID distribution, treat that as an explicit policy migration: add Apple credentials, notarization, stapling, Gatekeeper assessment, and clean-Mac verification together.

## Client update policy

- Automatic update checks default to off and are stored as an application preference.
- Enabling automatic checks performs a silent startup check only; it never downloads or installs without user action.
- Manual checks and failures stay in the Settings > Application Update surface.
- The title bar always shows the installed version and only adds an update affordance when a newer signed release exists.
- Downloads show progress; installation is user-initiated. Windows exits the app while the installer applies the update.
- Restore Defaults turns automatic checks off again.

The frontend update state lives in `apps/desktop/src/stores/update.ts`; its settings UI lives in `apps/desktop/src/pages/settings/SettingsUpdateSection.vue`. Tauri updater/process plugins and permissions live under `apps/desktop/src-tauri/`.

The updater plugin returns an `Update` class instance with JavaScript private fields. Pinia/Vue must never deep-proxy that instance: store it with `markRaw`, while copying display fields such as version and notes into ordinary reactive state. A proxied updater instance throws `Cannot read private member from an object whose class did not declare it` when installation starts. Keep the private-field regression test in `src/stores/update.spec.ts`.

## Distribution and mainland-China fallback

The client consumes the standard Tauri signed manifest protocol, not GitHub-specific APIs. GitHub Releases is the initial source:

```text
https://github.com/Yuelioi/bdl/releases/latest/download/latest.json
```

Future sources may be Gitee Releases, Aliyun OSS, Tencent COS, another CDN/object store, a project-owned static host, or a reverse proxy. Every mirror must serve the same signed artifacts. A mirror does not become trusted merely by hosting a file; the embedded public key remains the trust root.

For reliable mainland-China delivery, prefer a project-owned HTTPS dynamic endpoint such as `update.yuelili.com`. It can select a healthy GitHub/Gitee/object-storage artifact URL while returning the normal Tauri fields (`version`, `url`, `signature`, `notes`, and `pub_date`). This keeps source routing server-side and avoids shipping a new client whenever mirror priority changes.

Static fallback endpoints can be appended to `plugins.updater.endpoints` in `tauri.conf.json`, but Tauri only advances to the next configured endpoint for non-2xx responses. A dynamic project-owned endpoint is therefore the stronger long-term resilience seam.

## Application icon

The editable pink icon source is `apps/desktop/src-tauri/icons/app-icon.svg`. Regenerate platform resources with:

```powershell
pnpm --dir apps/desktop tauri icon src-tauri/icons/app-icon.svg
```

Commit the SVG master and generated icon set together. The mark is deliberately code-native vector artwork so future brand changes remain deterministic and scalable.

## Windows production shell

- The desktop entry point must keep `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`; without it, a production EXE opens a console window beside the Tauri window.
- The application root suppresses the WebView's native browser context menu. Product-owned context menus may still handle the same event and render their own actions.
- After a Windows release build, verify the PE optional header subsystem is `2` (`IMAGE_SUBSYSTEM_WINDOWS_GUI`), not `3` (`IMAGE_SUBSYSTEM_WINDOWS_CUI`).
- When uploading the updater private key through PowerShell, remove a leading UTF-8 BOM before writing `TAURI_SIGNING_PRIVATE_KEY`; a BOM is parsed as invalid Base64 input.

## Verification

Before tagging, `./scripts/check.ps1` must pass on Windows and `./scripts/check.sh` must pass on macOS. Portable shell checks may prefer ripgrep when installed but must provide a system-tool fallback for clean GitHub-hosted macOS runners. macOS package smoke checks must include `codesign --verify --deep --strict BDL.app`, confirmation that `codesign -dv --verbose=4` reports `Signature=adhoc`, and `hdiutil verify BDL_<version>_<arch>.dmg`. Update-specific coverage includes the Pinia preference/check tests and deterministic Playwright workspace screenshots. A real end-to-end install still requires a published release newer than the installed build and valid GitHub and Tauri updater signing credentials; Apple credentials are not part of the current release policy.
