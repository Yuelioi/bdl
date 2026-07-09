# Index - bdl-downloader-app

## State

Task 2 core DTOs are complete on branch `bdl-downloader-app`. The workspace has a Rust/Tauri/Vue scaffold plus frontend-safe normalized DTOs; input classification and resolver logic have not started.

## Next

Execute Task 3 in `plan.md`: implement the input classifier and run `cargo test -p bdl-core --test input_classifier`.

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

Decisions:
- Main navigation: `解析`, `传输`, `历史`, `设置`; account lives in the top-right account button.
- Core pipeline: `Input -> Resolver -> NormalizedDownloadPlan -> TaskQueue -> Downloader -> PostProcess -> History`.
- Frontend consumes normalized DTOs only; Rust backend owns business state and transfer state.
- Download engine is custom around `reqwest`/`tokio`, with a future `Fetcher` trait for optional alternate backends.
- Resume is resource-level, with URL refresh through `bpi-rs` when CDN URLs expire.
