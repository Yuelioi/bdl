# Repository instructions

Use the installed Flightdeck skill for durable repository work; recover from `flightdeck/deck.md`. Keep `/flightdeck/` in `.gitignore`; these records are local-only. If the workspace is absent in a fresh checkout, do not assume local knowledge or support scripts are available.

For interface redesign and shared component changes, follow [UI design guidelines](docs/UI_DESIGN.md) for typography, control sizes, touch targets, density, and visual verification.

## Windows 构建、缓存与大数据存放

- Rust 构建、检查和 Tauri 打包先经 `scripts/cargo-target.mjs` 解析全局根；本机输出为 `D:\coding\rust\target\bdl\`，Rust 二进制及 Tauri bundle 位于其 `release` 子目录。交付包保留在 `D:\coding\bdl\releases\<版本>\`，复制/上传入口使用实际解析路径。
- 现有 `dist/` 已完整保存在 `D:\coding\bdl\releases\local-dist-20261012\`，仓库原入口是 Junction；后续发布、复制和验收沿用已核对的入口或显式物理路径。
- Android/Gradle 大中间产物放在 `D:\coding\bdl\android\`；`gen/android/app/build/` 的物理目录为其 `app-build\` 子目录，用原路径 Junction 兼容构建和 APK 读取。生成工程源码入口保留普通目录，Tauri 的 `rust.rootDirRel = "../../../"` 依赖该父级布局；迁移前核对 Gradle 的实际路径解析。Android 构建不由 `CARGO_TARGET_DIR` 单独控制。
- 大下载与用户媒体放在 `D:\coding\bdl\downloads\`，制作素材放在 `D:\coding\bdl\promo\`；大缓存和一次性验收分别放在该项目目录的 `cache\`、`task-artifacts\<任务>\`。发布 APK、安装包与原始素材先区分保留用途，再清理编译中间物。
