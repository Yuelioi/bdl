# BDL downloader app context

## Stable domain decisions

- A normalized source tree separates source discovery from task planning and media acquisition.
- Queue and resource state are Rust-owned and persisted; frontend stores mirror snapshots and events.
- Resume operates at resource/segment level, while expired signed URLs are refreshed through resolver intent stored with the task.
- Successful muxing is the completion commit point; raw track cleanup must never turn a valid final output back into failure.
- Account state belongs in the global account surface, with cookies stored only through the secure-store adapter.
- Queue ordering controls are intentionally outside the product scope.

## Natural verification locations

- Automated repository gate: `scripts/check.ps1`
- Release checklist: `docs/RELEASE_CHECKLIST.md`
- Platform support: `docs/SUPPORTED_PLATFORMS.md`
- Architecture overview: `docs/ARCHITECTURE.md`
