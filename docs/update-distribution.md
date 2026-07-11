# Update distribution

BDL uses Tauri's signed updater protocol. The application UI and installation flow do not depend on GitHub-specific APIs; they consume a standard signed update manifest.

## Current source

- GitHub Releases: `https://github.com/Yuelioi/bdl/releases/latest/download/latest.json`

## Mirror contract

A future Gitee release, object-storage bucket, CDN, reverse proxy, or dynamic update service must expose the same Tauri update manifest and signed artifacts over HTTPS. Keep the signing key unchanged across mirrors so installed clients retain the same trust root.

Static mirrors can be added to `plugins.updater.endpoints` in `apps/desktop/src-tauri/tauri.conf.json`. For resilient mainland-China routing, prefer a small dynamic endpoint under a project-controlled domain that selects a healthy artifact mirror while returning the standard `version`, `url`, `signature`, `notes`, and `pub_date` fields.

The private signing key must never be committed. Store `TAURI_SIGNING_PRIVATE_KEY` and, when applicable, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in GitHub Actions secrets and keep an offline backup.

## Release procedure

1. Run `./scripts/version.ps1 patch` (or `minor`, `major`, or an exact version).
2. Run `./scripts/check.ps1` and commit the version change.
3. Create and push the matching tag, for example `git tag v0.1.1`.
4. The `Release` workflow creates a draft GitHub Release with signed Windows bundles and `latest.json`.
5. Review the draft release notes and artifacts, then publish it.
