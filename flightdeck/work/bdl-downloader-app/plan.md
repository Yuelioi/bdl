# BDL Downloader App Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri desktop Bilibili downloader that parses Bilibili inputs through `bpi-rs`, normalizes them into one source tree model, creates reliable download tasks, downloads video/audio resources, and post-processes them with ffmpeg.

**Architecture:** Rust owns parsing, normalization, task state, downloading, persistence, account state, and Tauri command/event boundaries. Vue owns the UI shell, source selection, task display, settings forms, and account dialogs. The first deliverable is a single-video vertical slice; later phases expand source types, persistence, account login, archive assets, and packaging.

**Tech Stack:** Rust 2024, Tauri 2, Vue 3, TypeScript, Pinia, Vite, `bpi-rs`, `reqwest`, `tokio`, `rusqlite`, system secure storage, ffmpeg.

---

## Current Plan Override

The original Transfer page scope in Task 11 was sufficient for a functional prototype, but it is superseded for product-quality work by `flightdeck/work/bdl-downloader-app/transfer-product-refactor-plan.md`.

Apply that focused plan for Transfer work before continuing unresolved resolver expansion. Its key product decisions are:

- Main navigation is `解析`, `传输`, `设置`.
- Completed records remain in Transfer under `已完成`; do not reintroduce a separate History page.
- Transfer list should be a quasi-table for batch task management.
- The inspector should prioritize diagnostics and recommended recovery actions.
- Queue ordering controls are out of scope.

## Phase Gates

- Phase 0 is done when the workspace builds and the desktop shell opens.
- Phase 1 is done when `bdl-core` can classify input and emit stable normalized DTOs from tests.
- Phase 2 is done when a BV/AV/video URL resolves through `bpi-rs` into a normalized source tree.
- Phase 3 is done when selected parts become queued download tasks.
- Phase 4 is done when the CLI can download one video/audio pair and ffmpeg merges it.
- Phase 5 is done when the Tauri UI can parse a single video and add it to transfer state.
- Phase 6 is done when the queue supports pause, resume, retry, cancellation, and resource-level resume.
- Phase 7 is done when favorites, uploader videos, collections, series, bangumi, and courses parse into the same tree model.
- Phase 8 is done when account persistence, cookie import, QR login, and login verification work.
- Phase 9 is done when history, settings, naming templates, derived assets, logs, and cleanup are usable.
- Phase 10 is done when packaging and cross-platform checks pass.

## File Structure

Create this shape unless a generated Tauri scaffold requires a small adjustment:

```text
Cargo.toml
crates/
  bdl-core/
    Cargo.toml
    src/
      lib.rs
      error.rs
      ids.rs
      input.rs
      model.rs
      resolver/
        mod.rs
        video.rs
        paged.rs
        bangumi.rs
        cheese.rs
      planner.rs
      queue.rs
      fetcher.rs
      muxer.rs
      storage.rs
      settings.rs
      account.rs
      logging.rs
    tests/
      input_classifier.rs
      normalization.rs
      planner.rs
      queue_state.rs
      fetcher_resume.rs
      muxer.rs
  bdl-tauri/
    Cargo.toml
    src/
      lib.rs
      commands.rs
      events.rs
      state.rs
      secure_store.rs
      file_actions.rs
  bdl-cli/
    Cargo.toml
    src/
      main.rs
apps/
  desktop/
    package.json
    index.html
    vite.config.ts
    tsconfig.json
    src/
      main.ts
      App.vue
      styles/
        tokens.css
        base.css
      ui/
        Button.vue
        IconButton.vue
        TextField.vue
        Textarea.vue
        Select.vue
        Checkbox.vue
        Tabs.vue
        Dialog.vue
        Drawer.vue
        ToastHost.vue
        ProgressBar.vue
        StatusBadge.vue
        Tree.vue
        TaskRow.vue
      stores/
        parse.ts
        queue.ts
        settings.ts
        account.ts
        ui.ts
      pages/
        ParsePage.vue
        TransferPage.vue
        HistoryPage.vue
        SettingsPage.vue
      api/
        tauri.ts
        dto.ts
    src-tauri/
      Cargo.toml
      tauri.conf.json
      src/
        main.rs
```

The root `docs/` directory should stay absent unless we later decide to publish user-facing docs. Durable implementation knowledge lives under `flightdeck/knowledge/`; active handoff stays under `flightdeck/work/bdl-downloader-app/`.

---

### Task 1: Scaffold Workspace

**Files:**

- Create: `Cargo.toml`
- Create: `crates/bdl-core/Cargo.toml`
- Create: `crates/bdl-core/src/lib.rs`
- Create: `crates/bdl-core/src/error.rs`
- Create: `crates/bdl-tauri/Cargo.toml`
- Create: `crates/bdl-tauri/src/lib.rs`
- Create: `crates/bdl-cli/Cargo.toml`
- Create: `crates/bdl-cli/src/main.rs`
- Create: `apps/desktop/package.json`
- Create: `apps/desktop/src-tauri/Cargo.toml`
- Create: `apps/desktop/src-tauri/src/main.rs`

- [ ] **Step 1: Write the root Cargo workspace**

`Cargo.toml`:

```toml
[workspace]
resolver = "3"
members = [
  "crates/bdl-core",
  "crates/bdl-tauri",
  "crates/bdl-cli",
  "apps/desktop/src-tauri",
]

[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "MIT"
authors = ["YUELI <yuelioi1210@gmail.com>"]

[workspace.dependencies]
anyhow = "1"
async-trait = "0.1"
bytes = "1"
chrono = { version = "0.4", features = ["clock", "serde"] }
futures = "0.3"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "stream", "json", "cookies", "gzip", "deflate", "brotli"] }
rusqlite = { version = "0.32", features = ["bundled", "chrono"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "fs", "process", "sync", "time", "io-util"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4", "serde"] }
```

- [ ] **Step 2: Add minimal Rust crates**

`crates/bdl-core/Cargo.toml`:

```toml
[package]
name = "bdl-core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
async-trait.workspace = true
bytes.workspace = true
chrono.workspace = true
futures.workspace = true
reqwest.workspace = true
rusqlite.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio.workspace = true
tracing.workspace = true
uuid.workspace = true
```

Do not add `bpi-rs` in Task 1. Add it when Task 4 introduces the resolver, using a portable dependency strategy rather than a committed absolute local path.

`crates/bdl-core/src/lib.rs`:

```rust
pub mod error;

pub use error::{BdlError, BdlResult};
```

`crates/bdl-core/src/error.rs`:

```rust
pub type BdlResult<T> = Result<T, BdlError>;

#[derive(Debug, thiserror::Error)]
pub enum BdlError {
    #[error("invalid input: {message}")]
    InvalidInput { message: String },

    #[error("unsupported source: {kind}")]
    UnsupportedSource { kind: String },

    #[error("bpi error: {0}")]
    Bpi(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
```

`crates/bdl-tauri/Cargo.toml`:

```toml
[package]
name = "bdl-tauri"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
bdl-core = { path = "../bdl-core" }
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tokio.workspace = true
tracing.workspace = true
```

`crates/bdl-tauri/src/lib.rs`:

```rust
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

`crates/bdl-cli/Cargo.toml`:

```toml
[package]
name = "bdl-cli"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
anyhow.workspace = true
bdl-core = { path = "../bdl-core" }
tokio.workspace = true
```

`crates/bdl-cli/src/main.rs`:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("bdl-cli");
    Ok(())
}
```

- [ ] **Step 3: Add the desktop scaffold**

Use Tauri 2 compatible scaffolding. If generated files differ, keep the package under `apps/desktop/` and the Rust app under `apps/desktop/src-tauri/`.

`apps/desktop/package.json`:

```json
{
  "name": "bdl-desktop",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite --host 127.0.0.1",
    "build": "vue-tsc --noEmit && vite build",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "pinia": "^2.3.0",
    "vue": "^3.5.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@vitejs/plugin-vue": "^5.2.0",
    "typescript": "^5.7.0",
    "vite": "^6.0.0",
    "vue-tsc": "^2.2.0"
  }
}
```

`apps/desktop/src-tauri/Cargo.toml`:

```toml
[package]
name = "bdl-desktop"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
bdl-tauri = { path = "../../../crates/bdl-tauri" }
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

`apps/desktop/src-tauri/src/main.rs`:

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("failed to run BDL desktop app");
}
```

- [ ] **Step 4: Verify the scaffold**

Run:

```powershell
cargo check --workspace
```

Expected: `Finished dev profile` with no compile errors.

Run:

```powershell
pnpm --dir apps/desktop install
pnpm --dir apps/desktop build
```

Expected: frontend build succeeds after minimal Vue files are present.

- [ ] **Step 5: Commit**

```powershell
git add Cargo.toml crates apps .gitignore flightdeck
git commit -m "chore: scaffold bdl workspace"
```

---

### Task 2: Define Core DTOs and IDs

**Files:**

- Create: `crates/bdl-core/src/ids.rs`
- Create: `crates/bdl-core/src/model.rs`
- Modify: `crates/bdl-core/src/lib.rs`
- Test: `crates/bdl-core/tests/normalization.rs`

- [ ] **Step 1: Write DTO serialization tests**

`crates/bdl-core/tests/normalization.rs`:

```rust
use bdl_core::model::{
    AssetKind, FetchPolicy, MediaKind, NormalizedGroup, NormalizedItem, NormalizedPart,
    NormalizedSourceTree, SourceKind, SourceSummary, StreamCodec, StreamQuality,
};

#[test]
fn normalized_source_tree_serializes_stable_shape() {
    let tree = NormalizedSourceTree {
        source: SourceSummary {
            id: "src_1".into(),
            kind: SourceKind::Video,
            input: "BV1xx411c7mD".into(),
            title: "Example".into(),
            loaded_count: 1,
            total_count: Some(1),
            has_more: false,
        },
        groups: vec![NormalizedGroup {
            id: "grp_1".into(),
            kind: "video".into(),
            title: "Example".into(),
            items: vec![NormalizedItem {
                id: "item_1".into(),
                title: "Example".into(),
                owner_name: Some("owner".into()),
                cover_url: None,
                duration_seconds: Some(120),
                parts: vec![NormalizedPart {
                    id: "part_1".into(),
                    title: "P1".into(),
                    aid: Some(1),
                    bvid: Some("BV1xx411c7mD".into()),
                    cid: Some(2),
                    streams: vec![],
                    assets: vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)],
                }],
            }],
            page: None,
        }],
    };

    let json = serde_json::to_value(&tree).expect("tree serializes");
    assert_eq!(json["source"]["kind"], "video");
    assert_eq!(json["groups"][0]["items"][0]["parts"][0]["assets"][0]["kind"], "cover");
}

#[test]
fn stream_quality_and_codec_are_frontend_safe_strings() {
    assert_eq!(serde_json::to_string(&StreamQuality::Best).unwrap(), "\"best\"");
    assert_eq!(serde_json::to_string(&StreamCodec::Hevc).unwrap(), "\"hevc\"");
    assert_eq!(serde_json::to_string(&MediaKind::Audio).unwrap(), "\"audio\"");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cargo test -p bdl-core --test normalization
```

Expected: FAIL because `model` and DTO types do not exist.

- [ ] **Step 3: Add DTO modules**

`crates/bdl-core/src/lib.rs`:

```rust
pub mod error;
pub mod ids;
pub mod model;

pub use error::{BdlError, BdlResult};
```

`crates/bdl-core/src/ids.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);
```

`crates/bdl-core/src/model.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Video,
    Bangumi,
    Cheese,
    Favorite,
    Collection,
    Series,
    Uploader,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSummary {
    pub id: String,
    pub kind: SourceKind,
    pub input: String,
    pub title: String,
    pub loaded_count: usize,
    pub total_count: Option<usize>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedSourceTree {
    pub source: SourceSummary,
    pub groups: Vec<NormalizedGroup>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedGroup {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub items: Vec<NormalizedItem>,
    pub page: Option<PageState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageState {
    pub page_number: u32,
    pub page_size: u32,
    pub loaded_count: usize,
    pub total_count: Option<usize>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedItem {
    pub id: String,
    pub title: String,
    pub owner_name: Option<String>,
    pub cover_url: Option<String>,
    pub duration_seconds: Option<u64>,
    pub parts: Vec<NormalizedPart>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedPart {
    pub id: String,
    pub title: String,
    pub aid: Option<u64>,
    pub bvid: Option<String>,
    pub cid: Option<u64>,
    pub streams: Vec<MediaStream>,
    pub assets: Vec<DerivedAsset>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaStream {
    pub id: String,
    pub kind: MediaKind,
    pub quality: StreamQuality,
    pub codec: StreamCodec,
    pub bandwidth: Option<u64>,
    pub urls: Vec<String>,
    pub headers: Vec<(String, String)>,
    pub acquired_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Video,
    Audio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamQuality {
    Best,
    Quality(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamCodec {
    Auto,
    Avc,
    Hevc,
    Av1,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Cover,
    Subtitle,
    Danmaku,
    Nfo,
}

impl AssetKind {
    pub fn with_policy(self, fetch_policy: FetchPolicy) -> DerivedAsset {
        DerivedAsset {
            kind: self,
            format: None,
            fetch_policy,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedAsset {
    pub kind: AssetKind,
    pub format: Option<String>,
    pub fetch_policy: FetchPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchPolicy {
    Never,
    OnDemand,
    Always,
}
```

- [ ] **Step 4: Run tests**

Run:

```powershell
cargo test -p bdl-core --test normalization
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add crates/bdl-core
git commit -m "feat: define normalized downloader DTOs"
```

---

### Task 3: Input Classifier

**Files:**

- Create: `crates/bdl-core/src/input.rs`
- Modify: `crates/bdl-core/src/lib.rs`
- Test: `crates/bdl-core/tests/input_classifier.rs`

- [ ] **Step 1: Write classifier tests**

`crates/bdl-core/tests/input_classifier.rs`:

```rust
use bdl_core::input::{classify_input, ClassifiedInput};
use bdl_core::model::SourceKind;

#[test]
fn classifies_bv_and_av_ids() {
    assert_eq!(
        classify_input("BV1xx411c7mD").unwrap(),
        ClassifiedInput::VideoBvid("BV1xx411c7mD".into())
    );

    assert_eq!(
        classify_input("av170001").unwrap(),
        ClassifiedInput::VideoAid(170001)
    );
}

#[test]
fn classifies_video_url() {
    let input = "https://www.bilibili.com/video/BV1xx411c7mD/?spm_id_from=333.1007";
    assert_eq!(
        classify_input(input).unwrap(),
        ClassifiedInput::VideoBvid("BV1xx411c7mD".into())
    );
}

#[test]
fn classifies_supported_url_kinds_without_extracting_final_ids() {
    assert_eq!(
        classify_input("https://space.bilibili.com/12345/video").unwrap().source_kind(),
        SourceKind::Uploader
    );
    assert_eq!(
        classify_input("https://space.bilibili.com/12345/favlist?fid=678").unwrap().source_kind(),
        SourceKind::Favorite
    );
    assert_eq!(
        classify_input("https://www.bilibili.com/bangumi/play/ss123").unwrap().source_kind(),
        SourceKind::Bangumi
    );
    assert_eq!(
        classify_input("https://www.bilibili.com/cheese/play/ss456").unwrap().source_kind(),
        SourceKind::Cheese
    );
}

#[test]
fn rejects_unknown_input_with_actionable_message() {
    let err = classify_input("not a bilibili thing").unwrap_err();
    assert!(err.to_string().contains("无法识别这个输入"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cargo test -p bdl-core --test input_classifier
```

Expected: FAIL because `input` module does not exist.

- [ ] **Step 3: Implement classifier**

Use `url = "2"` and `regex = "1"` in `crates/bdl-core/Cargo.toml`, then implement:

```rust
pub mod input;
```

`crates/bdl-core/src/input.rs`:

```rust
use crate::error::{BdlError, BdlResult};
use crate::model::SourceKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassifiedInput {
    VideoBvid(String),
    VideoAid(u64),
    ShortUrl(String),
    Uploader { mid: u64 },
    Favorite { raw_url: String },
    Collection { raw_url: String },
    Series { raw_url: String },
    Bangumi { raw_url: String },
    Cheese { raw_url: String },
}

impl ClassifiedInput {
    pub fn source_kind(&self) -> SourceKind {
        match self {
            Self::VideoBvid(_) | Self::VideoAid(_) | Self::ShortUrl(_) => SourceKind::Video,
            Self::Uploader { .. } => SourceKind::Uploader,
            Self::Favorite { .. } => SourceKind::Favorite,
            Self::Collection { .. } => SourceKind::Collection,
            Self::Series { .. } => SourceKind::Series,
            Self::Bangumi { .. } => SourceKind::Bangumi,
            Self::Cheese { .. } => SourceKind::Cheese,
        }
    }
}

pub fn classify_input(input: &str) -> BdlResult<ClassifiedInput> {
    let trimmed = input.trim();

    if let Some(bvid) = extract_bvid(trimmed) {
        return Ok(ClassifiedInput::VideoBvid(bvid));
    }

    if let Some(aid) = extract_aid(trimmed) {
        return Ok(ClassifiedInput::VideoAid(aid));
    }

    if trimmed.starts_with("https://b23.tv/") || trimmed.starts_with("http://b23.tv/") {
        return Ok(ClassifiedInput::ShortUrl(trimmed.to_string()));
    }

    if trimmed.contains("space.bilibili.com/") && trimmed.contains("favlist") {
        return Ok(ClassifiedInput::Favorite { raw_url: trimmed.to_string() });
    }

    if trimmed.contains("space.bilibili.com/") && trimmed.contains("/video") {
        let mid = trimmed
            .split("space.bilibili.com/")
            .nth(1)
            .and_then(|tail| tail.split(['/', '?']).next())
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(unrecognized)?;
        return Ok(ClassifiedInput::Uploader { mid });
    }

    if trimmed.contains("/bangumi/play/") {
        return Ok(ClassifiedInput::Bangumi { raw_url: trimmed.to_string() });
    }

    if trimmed.contains("/cheese/play/") {
        return Ok(ClassifiedInput::Cheese { raw_url: trimmed.to_string() });
    }

    if trimmed.contains("/medialist/play/") || trimmed.contains("season_id=") {
        return Ok(ClassifiedInput::Collection { raw_url: trimmed.to_string() });
    }

    if trimmed.contains("/series/") || trimmed.contains("series_id=") {
        return Ok(ClassifiedInput::Series { raw_url: trimmed.to_string() });
    }

    Err(unrecognized())
}

fn extract_bvid(input: &str) -> Option<String> {
    input
        .split(|ch: char| !(ch.is_ascii_alphanumeric()))
        .find(|part| part.starts_with("BV") && part.len() >= 10)
        .map(ToOwned::to_owned)
}

fn extract_aid(input: &str) -> Option<u64> {
    let lower = input.to_ascii_lowercase();
    lower
        .strip_prefix("av")
        .and_then(|digits| digits.parse::<u64>().ok())
}

fn unrecognized() -> BdlError {
    BdlError::InvalidInput {
        message: "无法识别这个输入。请粘贴 BV/AV、视频、番剧、课程、收藏夹、合集或 UP 主空间链接。".into(),
    }
}
```

- [ ] **Step 4: Run tests**

Run:

```powershell
cargo test -p bdl-core --test input_classifier
```

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add crates/bdl-core
git commit -m "feat: classify bilibili inputs"
```

---

### Task 4: Single Video Resolver Through bpi-rs

**Files:**

- Create: `crates/bdl-core/src/resolver/mod.rs`
- Create: `crates/bdl-core/src/resolver/video.rs`
- Modify: `crates/bdl-core/src/lib.rs`
- Test: `crates/bdl-core/tests/video_resolver.rs`

- [x] **Step 1: Define resolver contract with a fixture test**

The test should not hit the network. Use a fake adapter that returns one view and one playurl shape, then assert normalization. Keep live network tests behind `BDL_LIVE_TEST=1`.

Expected resolver API:

```rust
pub struct ResolveOptions {
    pub fetch_streams: bool,
}

#[async_trait::async_trait]
pub trait Resolver {
    async fn resolve(&self, input: ClassifiedInput, options: ResolveOptions) -> BdlResult<NormalizedSourceTree>;
}
```

- [x] **Step 2: Implement `VideoResolver`**

Use `bpi-rs` only in `resolver/video.rs`. Convert raw API results into `NormalizedSourceTree`. For a normal video:

- Source kind: `video`.
- Group: one group named after the video title.
- Item: one item per video.
- Part: one part per page/cid.
- Streams: video/audio streams from `video.play_url` if `fetch_streams` is true.
- Assets: cover on demand; subtitles/danmaku not fetched in fast mode.

- [x] **Step 3: Add an opt-in live test**

`crates/bdl-core/tests/video_resolver.rs` should include:

```rust
#[tokio::test]
async fn live_video_resolver_resolves_bv_when_enabled() {
    if std::env::var("BDL_LIVE_TEST").ok().as_deref() != Some("1") {
        return;
    }

    // Use a stable public BV sample from the existing bpi-rs contract fixtures.
    // Assert that at least one group, item, and part is returned.
}
```

- [x] **Step 4: Run non-live tests**

Run:

```powershell
cargo test -p bdl-core video_resolver
```

Expected: PASS without network.

- [x] **Step 5: Commit**

```powershell
git add crates/bdl-core
git commit -m "feat: resolve single videos through bpi"
```

---

### Task 5: Planner and Task Model

**Files:**

- Create: `crates/bdl-core/src/planner.rs`
- Create: `crates/bdl-core/src/queue.rs`
- Modify: `crates/bdl-core/src/lib.rs`
- Test: `crates/bdl-core/tests/planner.rs`
- Test: `crates/bdl-core/tests/queue_state.rs`

- [x] **Step 1: Write planner tests**

Cover these exact cases:

- A selected single part creates one task.
- Fast download creates video and audio resources only.
- Complete archive adds cover, subtitle, danmaku, and NFO resource intents.
- Missing audio stream returns an actionable planner error.
- Task IDs and resource IDs are stable enough to resume by persisted task record.

- [x] **Step 2: Implement task models**

Core types:

```rust
pub enum TaskStatus {
    Waiting,
    Parsing,
    Downloading,
    Muxing,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

pub enum ResourceStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

pub struct DownloadTask {
    pub id: String,
    pub title: String,
    pub source_id: String,
    pub status: TaskStatus,
    pub resources: Vec<DownloadResource>,
    pub output_path: std::path::PathBuf,
}

pub struct DownloadResource {
    pub id: String,
    pub kind: DownloadResourceKind,
    pub intent: DownloadResourceIntent,
    pub current_urls: Vec<String>,
    pub headers: Vec<(String, String)>,
    pub target_path: std::path::PathBuf,
    pub temp_path: std::path::PathBuf,
    pub status: ResourceStatus,
}
```

- [x] **Step 3: Implement selection planner**

The planner accepts:

- `NormalizedSourceTree`.
- Selected `part_id` list.
- `DownloadOptions`.

It returns `Vec<DownloadTask>`.

Do not let the frontend create resource paths or infer stream choices. Use default options from settings plus per-selection overrides.

- [x] **Step 4: Run tests**

Run:

```powershell
cargo test -p bdl-core --test planner --test queue_state
```

Expected: PASS.

- [x] **Step 5: Commit**

```powershell
git add crates/bdl-core
git commit -m "feat: plan selected parts into download tasks"
```

---

### Task 6: Reqwest Fetcher With Segment State

**Files:**

- Create: `crates/bdl-core/src/fetcher.rs`
- Test: `crates/bdl-core/tests/fetcher_resume.rs`

- [x] **Step 1: Write fetcher tests with a local HTTP server**

Use a test server that supports:

- `HEAD` with content length.
- `GET` with `Range`.
- A forced 500 response for retry tests.

Tests:

- Full download writes the exact expected bytes.
- Segment download resumes from existing `.bdlpart`.
- Retry stops after configured retry count.
- If content length changes between resume attempts, the fetcher restarts that resource instead of mixing segments.

- [x] **Step 2: Implement `Fetcher` trait and `ReqwestFetcher`**

Required behavior:

- Adds provided headers such as `Referer`, `User-Agent`, and `Cookie` when present.
- Uses `Range` for segment downloads when content length is known.
- Writes to temp files first.
- Writes `.bdlpart` resource state beside the temp file.
- Emits progress events through a callback or channel, not directly through Tauri.

- [x] **Step 3: Run fetcher tests**

Run:

```powershell
cargo test -p bdl-core --test fetcher_resume
```

Expected: PASS.

- [x] **Step 4: Commit**

```powershell
git add crates/bdl-core
git commit -m "feat: add resumable reqwest fetcher"
```

---

### Task 7: ffmpeg Muxer and CLI Vertical Slice

**Files:**

- Create: `crates/bdl-core/src/muxer.rs`
- Modify: `crates/bdl-cli/src/main.rs`
- Test: `crates/bdl-core/tests/muxer.rs`

- [x] **Step 1: Write muxer tests**

Tests:

- `ffmpeg` missing returns `MuxError::FfmpegNotFound`.
- A configured ffmpeg path is preferred over system lookup.
- A non-zero ffmpeg exit code returns the exit code and stderr summary.

- [x] **Step 2: Implement `MediaMuxer`**

Required command shape:

```text
ffmpeg -y -i video.m4s -i audio.m4s -c copy output.mp4
```

For mkv:

```text
ffmpeg -y -i video.m4s -i audio.m4s -c copy output.mkv
```

- [x] **Step 3: Implement CLI commands**

`bdl-cli` supports:

```text
bdl-cli parse <input> --json
bdl-cli download <input> --output <dir> --quality best
bdl-cli verify-cookie
bdl-cli ffmpeg-check
```

The initial `download` command may support only single normal video. It must use the same resolver, planner, fetcher, and muxer as the future Tauri app.

- [x] **Step 4: Run CLI checks**

Run:

```powershell
cargo run -p bdl-cli -- ffmpeg-check
cargo run -p bdl-cli -- parse BV1xx411c7mD --json
```

Expected: `ffmpeg-check` reports found or missing without panicking; parse command returns normalized JSON or a clean API error.

- [x] **Step 5: Commit**

```powershell
git add crates/bdl-core crates/bdl-cli
git commit -m "feat: add ffmpeg muxer and cli vertical slice"
```

---

### Task 8: Tauri Command and Event Bridge

**Files:**

- Create: `crates/bdl-tauri/src/commands.rs`
- Create: `crates/bdl-tauri/src/events.rs`
- Create: `crates/bdl-tauri/src/state.rs`
- Modify: `crates/bdl-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/main.rs`

- [x] **Step 1: Implement app state**

`AppState` owns:

- `BpiClient` wrapper or resolver service.
- In-memory parse source registry.
- Task queue handle.
- Settings snapshot.
- Event emitter abstraction.

- [x] **Step 2: Register commands**

Command names must match the architecture knowledge:

```text
parse_create_source
parse_load_more
parse_load_all
parse_close_source
parse_refresh_source
selection_create_tasks
queue_list
queue_pause
queue_resume
queue_cancel
queue_retry
queue_remove
queue_open_file
queue_open_dir
settings_get
settings_update
account_get
account_login_qr_start
account_login_qr_poll
account_import_cookie
account_logout
account_verify
```

Commands that are not implemented yet must return a typed `unsupported` error. Do not silently stub success.

- [x] **Step 3: Emit canonical events**

Events:

```text
parse://source-updated
parse://items-appended
queue://task-updated
queue://resource-updated
queue://log-appended
settings://updated
account://updated
```

- [x] **Step 4: Verify command registration**

Run:

```powershell
cargo check -p bdl-desktop
```

Expected: PASS.

- [x] **Step 5: Commit**

```powershell
git add crates/bdl-tauri apps/desktop/src-tauri
git commit -m "feat: expose downloader commands to tauri"
```

---

### Task 9: UI Kit and App Shell

**Files:**

- Create: `apps/desktop/src/main.ts`
- Create: `apps/desktop/src/App.vue`
- Create: `apps/desktop/src/styles/tokens.css`
- Create: `apps/desktop/src/styles/base.css`
- Create: `apps/desktop/src/ui/*.vue`
- Create: `apps/desktop/src/stores/ui.ts`

- [x] **Step 1: Add tokens before business UI**

Use CSS custom properties for:

```text
background, surface, panel, border, text, muted, accent, danger, warning, success
spacing 4/8/12/16/24/32
radius 4/6/8
font sizes 12/13/14/16/18/22
fixed heights button 32, input 34, toolbar 40, task row 64
```

- [x] **Step 2: Build reusable controls**

Create:

```text
Button
IconButton
TextField
Textarea
Select
Checkbox
Tabs
Dialog
Drawer
ToastHost
ProgressBar
StatusBadge
Tree
TaskRow
```

Rules:

- No nested cards.
- No hero page.
- No one-off button styles in pages.
- Cards use radius 8 px or less.
- Text must fit within controls at 1100 px minimum window width.

- [x] **Step 3: Build app shell**

Navigation labels:

```text
解析
传输
历史
设置
```

Top-right account button:

```text
登录
```

Logged-in state is added in account phase.

- [x] **Step 4: Run frontend build**

Run:

```powershell
pnpm --dir apps/desktop build
```

Expected: PASS.

- [x] **Step 5: Commit**

```powershell
git add apps/desktop/src
git commit -m "feat: add desktop app shell and ui kit"
```

---

### Task 10: Parse Page

**Files:**

- Create: `apps/desktop/src/api/tauri.ts`
- Create: `apps/desktop/src/api/dto.ts`
- Create: `apps/desktop/src/stores/parse.ts`
- Create: `apps/desktop/src/pages/ParsePage.vue`

- [x] **Step 1: Define frontend DTOs matching Rust**

Types:

```ts
export type SourceKind =
  | "video"
  | "bangumi"
  | "cheese"
  | "favorite"
  | "collection"
  | "series"
  | "uploader"
  | "unknown";

export interface NormalizedSourceTree {
  source: SourceSummary;
  groups: NormalizedGroup[];
}
```

Keep TypeScript field names identical to Rust serde output.

- [x] **Step 2: Implement parse store**

Store state:

- `sources`.
- `activeSourceId`.
- `selectionBySource`.
- `loadingBySource`.
- `errorsBySource`.

Actions:

- `createSource(input: string)`.
- `loadMore(sourceId: string)`.
- `parseAll(sourceId: string, limit?: number)`.
- `closeSource(sourceId: string)`.
- `refreshSource(sourceId: string)`.
- `toggleNode(sourceId: string, nodeId: string)`.
- `createTasksForSelection(sourceId: string)`.

- [x] **Step 3: Implement parse page layout**

Layout:

- Top input area.
- Left source list.
- Center result tree.
- Right details/download settings panel or drawer.

Primary action: `下载已选择`.

Do not auto-navigate after task creation. Show toast with `查看传输`.

- [x] **Step 4: Run frontend build**

Run:

```powershell
pnpm --dir apps/desktop build
```

Expected: PASS.

- [x] **Step 5: Commit**

```powershell
git add apps/desktop/src
git commit -m "feat: add parse page"
```

---

### Task 11: Transfer Page and Queue Store

**Files:**

- Create: `apps/desktop/src/stores/queue.ts`
- Create: `apps/desktop/src/pages/TransferPage.vue`
- Modify: `apps/desktop/src/ui/TaskRow.vue`

- [x] **Step 1: Implement queue store**

State:

- `tasks`.
- `activeFilter`.
- `selectedTaskId`.
- `logsByTask`.

Actions:

- `list()`.
- `pause(taskId)`.
- `resume(taskId)`.
- `cancel(taskId)`.
- `retry(taskId)`.
- `remove(taskId)`.
- `openFile(taskId)`.
- `openDir(taskId)`.

Event handling:

- `queue://task-updated`.
- `queue://resource-updated`.
- `queue://log-appended`.

- [x] **Step 2: Implement transfer filters**

Filters:

```text
正在下载
队列中
已暂停
失败
已完成
全部
```

- [x] **Step 3: Implement task row**

Task row height target: 64 px.

First line:

- title.
- status badge.
- total progress.
- action buttons.

Second line:

- quality.
- audio.
- codec.
- speed.
- ETA.
- save path or current stage.

- [x] **Step 4: Run frontend build**

Run:

```powershell
pnpm --dir apps/desktop build
```

Expected: PASS.

- [x] **Step 5: Commit**

```powershell
git add apps/desktop/src
git commit -m "feat: add transfer queue ui"
```

---

### Task 12: Persistent Storage and Recovery

**Files:**

- Create: `crates/bdl-core/src/storage.rs`
- Create: `crates/bdl-core/src/settings.rs`
- Test: `crates/bdl-core/tests/storage.rs`
- Modify: `crates/bdl-tauri/src/state.rs`

- [x] **Step 1: Write SQLite migration tests**

Test:

- Creates `tasks.sqlite`.
- Creates tables `tasks`, `resources`, `segments`, `history`, `task_logs`.
- Inserts a task with two resources.
- Reloads it after reopening the connection.

- [x] **Step 2: Implement storage**

Use `rusqlite`. Set:

```sql
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
```

Use one storage actor or one serialized storage handle to avoid concurrent write races.

- [x] **Step 3: Implement startup recovery**

Startup classifies persisted tasks:

- Incomplete.
- Completed.
- Failed.

Default behavior: do not auto-resume. Expose resumable tasks in transfer UI.

- [x] **Step 4: Run storage tests**

Run:

```powershell
cargo test -p bdl-core --test storage
```

Expected: PASS.

- [x] **Step 5: Commit**

```powershell
git add crates/bdl-core crates/bdl-tauri
git commit -m "feat: persist tasks and recover queue state"
```

---

### Task 13: Account Persistence and Login

**Files:**

- Create: `crates/bdl-core/src/account.rs`
- Create: `crates/bdl-tauri/src/secure_store.rs`
- Create: `apps/desktop/src/stores/account.ts`
- Add: account dialog component under `apps/desktop/src/pages` or `apps/desktop/src/ui`

- [x] **Step 1: Implement cookie import first**

Behavior:

- User pastes cookie.
- Backend validates it has login cookie shape.
- Backend injects it into `bpi-rs`.
- Backend verifies account through available account APIs.
- Cookie is persisted in system secure storage.
- SQLite stores account summary only.

Current implementation persists the cookie through `crates/bdl-tauri/src/secure_store.rs`, backed by `.bdl/account.cookie`. The wrapper keeps call sites isolated so a later OS keychain backend can replace the file-backed store without changing account commands.

- [x] **Step 2: Implement QR login**

Use `bpi-rs` QR APIs:

- Generate QR.
- Poll session.
- Persist resulting account cookie/session when confirmed.
- Emit `account://updated`.

- [x] **Step 3: Implement UI**

Top-right account button:

- Logged out: `登录`.
- Logged in: avatar, name, VIP/status.

Dialog tabs:

- `扫码登录`.
- `Cookie 导入`.

Dropdown:

- Re-check login.
- Import cookie.
- Logout.

- [x] **Step 4: Redaction test**

Add tests that logs redact:

```text
SESSDATA
bili_jct
DedeUserID
Cookie
```

- [x] **Step 5: Commit**

```powershell
git add crates/bdl-core crates/bdl-tauri apps/desktop/src
git commit -m "feat: add account login and secure cookie storage"
```

Landed as smaller commits:

- `1b3f3a4 feat: add account cookie model`
- `01c6e50 feat: persist imported account cookie`
- `31427dc feat: wire account cookie login ui`
- `93884dc feat: add qr account login flow`

---

### Task 14: Paged and Multi-Type Resolvers

**Files:**

- Create/modify: `crates/bdl-core/src/resolver/paged.rs`
- Create/modify: `crates/bdl-core/src/resolver/bangumi.rs`
- Create/modify: `crates/bdl-core/src/resolver/cheese.rs`
- Modify: `crates/bdl-core/src/resolver/mod.rs`
- Test: `crates/bdl-core/tests/paged_resolvers.rs`

- [x] **Step 1: Implement page-size policy**

Policy:

```text
page_size = min(api_known_max_or_default, 100)
```

Known defaults:

- Favorite resources: 20.
- Uploader videos: 30.
- Collection/series: 10 or 20 depending on endpoint.

- [x] **Step 2: Add favorite and uploader parsing**

Favorite and uploader parsing are complete for video resources:

- `crates/bdl-core/src/resolver/uploader.rs` resolves uploader video pages through `bpi-rs user.uploaded_videos`.
- `crates/bdl-core/src/resolver/favorite.rs` resolves favorite video resource pages through `bpi-rs fav.list_detail`.
- `parse_load_more` appends one uploader/favorite page.
- `parse_load_all` appends uploader/favorite pages up to an explicit/default limit of 100.
- `下载已选择` hydrates selected list placeholders before planning downloads.

Requirements:

- Initial parse loads only the first page.
- `parse_load_more` appends one page.
- `parse_load_all` requires an explicit limit or confirmation from frontend.
- `下载已选择` only creates tasks for loaded selected items.

- [ ] **Step 3: Add collection and series parsing**

Normalize collection/series into the same `Source -> Group -> Item -> Part` tree.

- [ ] **Step 4: Add bangumi and course parsing**

Normalize seasons/courses into the same tree. Do not assume VIP capabilities; use actual streams returned by `bpi-rs`.

- [ ] **Step 5: Commit**

```powershell
git add crates/bdl-core crates/bdl-tauri apps/desktop/src
git commit -m "feat: parse paged and episodic sources"
```

---

### Task 15: Derived Assets, Naming, History, and Settings

**Files:**

- Modify: `crates/bdl-core/src/planner.rs`
- Modify: `crates/bdl-core/src/muxer.rs`
- Modify: `crates/bdl-core/src/settings.rs`
- Create: `crates/bdl-core/src/naming.rs`
- Create: `crates/bdl-core/src/history.rs`
- Create: `apps/desktop/src/pages/HistoryPage.vue`
- Create: `apps/desktop/src/pages/SettingsPage.vue`
- Create: `apps/desktop/src/stores/settings.ts`

- [ ] **Step 1: Implement naming templates**

Defaults:

```text
{title}/{title} - P{part_index} - {part_title}.{ext}
{series_title}/S{season_index}E{episode_index} - {episode_title}.{ext}
{collection_title}/{index} - {title}.{ext}
```

Variables:

```text
title, part_title, part_index, bvid, aid, cid, owner_name, owner_mid,
series_title, season_index, episode_index, collection_title, index,
quality, codec, date, ext
```

Rules:

- Sanitize invalid path characters.
- Trim trailing dots and spaces on Windows.
- Append `(1)`, `(2)`, etc. on conflict.
- Provide preview in settings.

- [ ] **Step 2: Implement archive assets**

Presets:

- `快速下载`: final merged video only.
- `完整归档`: video, cover, subtitles, danmaku, NFO.
- `自定义`: user-selected assets.

Do not fetch subtitles/danmaku during fast parse.

- [ ] **Step 3: Implement history page**

Search fields:

- title.
- source URL/input.
- save path.

Actions:

- open file.
- open directory.
- re-download.
- copy source link.
- delete history record.

- [ ] **Step 4: Implement settings page**

Sections:

- `下载`.
- `媒体`.
- `归档`.
- `高级`.

Do not place account settings here.

- [ ] **Step 5: Commit**

```powershell
git add crates/bdl-core crates/bdl-tauri apps/desktop/src
git commit -m "feat: add archive options history and settings"
```

---

### Task 16: Packaging, QA, and Release Checks

**Files:**

- Modify: `apps/desktop/src-tauri/tauri.conf.json`
- Create: `scripts/check.ps1`
- Create: `scripts/package.ps1`
- Update: `.gitignore`

- [ ] **Step 1: Add a single check script**

`scripts/check.ps1` runs:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir apps/desktop build
```

- [ ] **Step 2: Add package script**

`scripts/package.ps1` runs:

```powershell
pnpm --dir apps/desktop tauri build
```

It must print where artifacts were written.

- [ ] **Step 3: Manual QA checklist**

Verify:

- App starts at minimum 1100x720.
- Parse page accepts BV and URL.
- `下载已选择` stays on parse page and shows toast.
- Transfer badge increments.
- Transfer page shows running/completed task.
- Pause/resume/cancel/retry actions do not corrupt state.
- Restart shows incomplete tasks but does not auto-resume by default.
- Cookie import persists across restart.
- Logs redact cookies and signed URL details.
- ffmpeg missing message is actionable.

- [ ] **Step 4: Commit**

```powershell
git add scripts apps/desktop/src-tauri .gitignore flightdeck
git commit -m "chore: add release checks and packaging scripts"
```

---

## Execution Rules

- Build one phase at a time; do not start multi-source parsing before the single-video vertical slice works.
- Keep `bdl-core` independent of Tauri APIs.
- Keep Tauri commands as glue; they call core services and emit events.
- Frontend transfer state is backend-authoritative.
- Commit after each task.
- Update `flightdeck/work/bdl-downloader-app/index.md` after each completed phase with current state and next step.

## Verification Matrix

Run these before claiming a phase complete:

```powershell
cargo fmt --check
cargo test --workspace
cargo check --workspace
pnpm --dir apps/desktop build
```

For UI phases, also run the Tauri app and inspect:

```powershell
pnpm --dir apps/desktop tauri dev
```

For downloader phases, run:

```powershell
cargo run -p bdl-cli -- ffmpeg-check
cargo run -p bdl-cli -- parse <known-public-bv> --json
```

For live download checks, use a small public video and a throwaway output directory ignored by git.

## Plan Self-Review

Spec coverage:

- Product pages are covered by Tasks 9, 10, 11, and 15.
- Core Rust architecture is covered by Tasks 1 through 8 and 12 through 14.
- Resource-level resume, URL refresh, and downloader behavior are covered by Tasks 5 and 6.
- Account persistence is covered by Task 13.
- History, settings, naming, derived resources, logs, and packaging are covered by Tasks 15 and 16.

Red-flag scan:

- No task is allowed to land an empty stub that returns success for unsupported behavior.
- Future unsupported commands must return typed unsupported errors until their phase implements them.

Type consistency:

- `NormalizedSourceTree`, `NormalizedGroup`, `NormalizedItem`, and `NormalizedPart` are the shared model across resolver, planner, Tauri, and frontend DTOs.
- Task and resource state are backend-owned and mirrored to frontend via queue events.
