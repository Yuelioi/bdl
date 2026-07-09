# Index - bdl-downloader-app

## State

Task 7 CLI vertical slice is complete on branch `bdl-downloader-app`. The workspace has a Rust/Tauri/Vue scaffold, frontend-safe normalized DTOs, a pure Bilibili input classifier, a `bpi-rs` backed video resolver, a planner that turns selected normalized parts into backend-owned download tasks/resources, a resumable `reqwest` fetcher, an ffmpeg muxer, and a CLI parse/download path.

## Next

Execute Task 8 in `plan.md`: add the Tauri command and event bridge.

## Read now

- flightdeck/knowledge/bdl-downloader/product-flow.md
- flightdeck/knowledge/bdl-downloader/architecture.md
- flightdeck/work/bdl-downloader-app/plan.md

## Read if

- E:/projects/tools/bpi-rs/docs/api-index.md - when mapping a supported source type to existing `bpi-rs` APIs.


## Progress

Current:
- Created the initial flightdeck deck.
- Captured BDL product flow and architecture decisions.
- Moved durable design material into `flightdeck/knowledge/bdl-downloader/`.
- Wrote the phased implementation plan in `flightdeck/work/bdl-downloader-app/plan.md`.
- Completed Task 1 scaffold in commits `c3cac74` and `b482e22`.
- Task 1 verification passed: `cargo check --workspace`, `pnpm --dir apps/desktop install`, and `pnpm --dir apps/desktop build`.
- Code review required removing the absolute `bpi-rs` dependency from Task 1 and replacing the placeholder icon with a generated Tauri icon set; both are done.
- Completed Task 2 DTOs in commits `b5d10e5` and `347a1ba`.
- Task 2 verification passed: `cargo test -p bdl-core --test normalization`, `cargo fmt --all --check`, and `cargo check --workspace`.
- DTO quality review required tagged `StreamQuality`, transparent ID wrappers used in DTO fields, object-shaped `HeaderPair`, and full JSON shape tests; all are done.
- Non-blocking hardening noted for later: add deserialization/round-trip tests for transparent IDs and tagged `StreamQuality`.
- Completed Task 3 input classifier in commits `b1a076c` and `40ac013`.
- Task 3 verification passed: `cargo test -p bdl-core --test input_classifier`, `cargo fmt --all --check`, and `cargo check --workspace`.
- Classifier quality review required short URLs to remain `Unknown`, exact BVID parsing, AV URL parsing, host/path-aware URL handling, plain uploader space support, and broader edge tests; all are done.
- Non-blocking hardening noted for later: add more host spoofing regression tests such as `bilibili.com.evil.test` and `b23.tv.evil.test`.
- Completed Task 4 single-video resolver in commit `f2ab9cc`.
- Task 4 verification passed: `cargo fmt --all --check`, `cargo test -p bdl-core`, and `cargo check --workspace`.
- The resolver uses portable `bpi-rs = 0.2.3` with only the `video` feature enabled, keeps `bpi-rs` usage inside `crates/bdl-core/src/resolver/video.rs`, and uses a fake `VideoApi` adapter for non-live tests.
- Execution note from the user: continue locally without subagents unless explicitly requested again.
- Fixed the Task 4 source/test mismatch in commit `8907a25`.
- Completed Task 5 planner and task model in commit `61bbb0b`.
- Task 5 verification passed: `cargo test -p bdl-core --test planner --test queue_state`, `cargo test -p bdl-core`, `cargo fmt --all --check`, and `cargo check --workspace`.
- Planner creates stable task/resource IDs, plans only selected loaded parts, chooses video/audio media streams in fast mode, adds cover/subtitle/danmaku/NFO intents for complete archive mode, and returns actionable errors when required streams are missing.
- Completed Task 6 resumable fetcher in commit `cb85078`.
- Task 6 verification passed: `cargo test -p bdl-core --test fetcher_resume`, `cargo test -p bdl-core`, `cargo fmt --all --check`, and `cargo check --workspace`.
- Fetcher writes to temp `.bdlpart` files first, stores progress in `<temp>.state`, resumes with `Range`, restarts when content length changes, forwards resource headers, retries failed GETs up to the configured limit, and emits progress through a channel.
- Completed Task 7 ffmpeg muxer and CLI vertical slice in commit `f6656cf`.
- Task 7 verification passed: `cargo test -p bdl-core --test muxer`, `cargo test --workspace`, `cargo check --workspace`, `cargo fmt --all --check`, `cargo run -p bdl-cli -- ffmpeg-check`, and `cargo run -p bdl-cli -- parse BV1xx411c7mD --json`.
- `ffmpeg-check` found `D:\scoop\shims\ffmpeg.exe` on this machine; CLI live parse returned normalized JSON for `BV1xx411c7mD`.
- CLI commands now include `parse`, `download`, `verify-cookie`, and `ffmpeg-check`; the initial `download` command supports the single-video pipeline using resolver, planner, fetcher, and muxer.

Decisions:
- Main navigation: `解析`, `传输`, `历史`, `设置`; account lives in the top-right account button.
- Core pipeline: `Input -> Resolver -> NormalizedDownloadPlan -> TaskQueue -> Downloader -> PostProcess -> History`.
- Frontend consumes normalized DTOs only; Rust backend owns business state and transfer state.
- Download engine is custom around `reqwest`/`tokio`, with a future `Fetcher` trait for optional alternate backends.
- Resume is resource-level, with URL refresh through `bpi-rs` when CDN URLs expire.
