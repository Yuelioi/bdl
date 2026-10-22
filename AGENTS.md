# Repository instructions

Route durable, multi-session work through the Markdown workspace in `flightdeck/` and keep its focused Work handoff current.

At the start of a repository task, read `flightdeck/deck.md` and consult `flightdeck/knowledge/index.md` for relevant durable knowledge before acting. Keep reusable procedures and established decisions in `flightdeck/knowledge/`, indexed by topic and user trigger; keep task progress and validation history in `flightdeck/work/`. When finishing work, put verified reusable guidance in Knowledge topic files and link them from the relevant folder indexes; indexes contain navigation summaries, not topic bodies.

`flightdeck/` is local-only and must stay ignored by Git. If it is absent in a fresh checkout, do not assume local knowledge or support scripts are available.

## Flightdeck 工作台维护

开始或恢复仓库工作时，先读 `flightdeck/deck.md` 与 `flightdeck/knowledge/index.md`；
按相关目录摘要逐层进入所需知识，不预加载整个知识树。每个自有文档文件夹都保留 `index.md`，
说明范围、直属子文件用途与何时阅读，并链接子目录的索引；外部只读参考库保持原布局。
摘要不记录进度、当前结论、验证结果、日期或数量。Work 的 Goal、Status、Current、Next
及执行入口仍由 Work 页负责；Plan 管阶段顺序，Slice 管局部细节，Context 管稳定背景。
创建 Work 时在 References 引用适用 Knowledge 并说明用途；立即需要的资料放进 Next，
最多三个直接链接，不添加无关依赖。deck 保持 Knowledge 与 Work 索引入口。
新增、移动、重命名或删除文档时同步维护所在索引和受影响入链；移动前搜索全部工作台 Markdown，
包括 Work，而不只检查 Knowledge。正文、进度或验证结论更新不向父索引传播；只有用途范围改变才改摘要。
Focus 是 Open Work 的恢复优先级，可有多个；未聚焦的 Open Work 放 Other open work。
日常保存保留 Focus；明确搁置才移出。交付待确认仍为 Open，用户确认整个 Work 完成后才 Finished；
用户取消时为 Stopped。终态 Work 留在原位，历史摘要放 `flightdeck/work/index.md`，未知确认日期不推测。
新建工作台或完成升级验证后，在 deck 标题下写入实际加载的完整 Flightdeck 版本；普通保存与新建 Work 不刷新版本。
`flightdeck/` 仅作本地记录：`.gitignore` 包含 `/flightdeck/`；已有跟踪记录保留本地文件并从 Git 索引移除。
只维护工作台不代表执行 Work 下一步、提交、推送或发布。

## Windows 构建、缓存与大数据存放

- Rust 构建、检查和 Tauri 打包先经 `scripts/cargo-target.mjs` 解析全局根；本机输出为 `D:\coding\rust\target\bdl\`，Rust 二进制及 Tauri bundle 位于其 `release` 子目录。交付包保留在 `D:\coding\bdl\releases\<版本>\`，复制/上传入口使用实际解析路径。
- 现有 `dist/` 已完整保存在 `D:\coding\bdl\releases\local-dist-20261012\`，仓库原入口是 Junction；后续发布、复制和验收沿用已核对的入口或显式物理路径。
- Android/Gradle 大中间产物放在 `D:\coding\bdl\android\`；`gen/android/app/build/` 的物理目录为其 `app-build\` 子目录，用原路径 Junction 兼容构建和 APK 读取。生成工程源码入口保留普通目录，Tauri 的 `rust.rootDirRel = "../../../"` 依赖该父级布局；迁移前核对 Gradle 的实际路径解析。Android 构建不由 `CARGO_TARGET_DIR` 单独控制。
- 大下载与用户媒体放在 `D:\coding\bdl\downloads\`，制作素材放在 `D:\coding\bdl\promo\`；大缓存和一次性验收分别放在该项目目录的 `cache\`、`task-artifacts\<任务>\`。发布 APK、安装包与原始素材先区分保留用途，再清理编译中间物。
