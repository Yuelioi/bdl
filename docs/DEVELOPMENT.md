# BDL 开发指南

本文档面向需要在本地运行、调试或打包 BDL 的开发者。产品下载、FFmpeg 安装和基本使用方法请查看 [README](../README.md)。

界面重设计与组件调整遵循 [界面设计规范](UI_DESIGN.md)，其中定义桌面与 Android 的字号、控件、触控热区和信息密度；概念图不能代替具体尺寸规范。

## 环境要求

- Rust `1.88+`
- Node.js `22+`
- pnpm `11+`
- FFmpeg
- 对应平台的 [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/)

BDL 的桌面目标包括 Windows、macOS，以及正在接入的 Linux x86_64；具体范围见 [支持平台](SUPPORTED_PLATFORMS.md)。

### Linux 构建环境

正式 Linux 包在 Ubuntu 22.04 x86_64 上构建，避免无意提高 glibc 最低版本。Ubuntu/Debian 开发机安装 Linux 版 Rust、Node 和 pnpm 后，在仓库根目录运行：

```bash
bash scripts/install-linux-deps.sh
pnpm --dir apps/desktop install --frozen-lockfile
pnpm --dir apps/desktop tauri dev
```

不要共用 Windows 的 `node_modules`、Rust target 目录或 Windows 版 Node/pnpm。WSL 开发建议在 Linux 文件系统中建立独立工作副本，图形界面依赖 WSLg，凭据持久化仍需要已解锁的 Secret Service 桌面服务。

未配置 XDG 下载目录时，默认路径回退为 `~/Downloads/BDL`；启动只计算路径，实际下载时才创建目录。已有的保存目录设置优先。

Linux 平台配置 `tauri.linux.conf.json` 自动合并，默认生成 AppImage 和 deb。已有相关检查通过时，可只执行打包和产物检查：

```bash
bash scripts/package.sh --skip-check
bash scripts/verify-linux-packages.sh ./target/release/bundle
```

自定义 `CARGO_TARGET_DIR` 时，验证命令应传入实际的 `release/bundle` 目录。Linux CI 编译原生后端、运行 FFmpeg/凭据边界测试并构建两个安装包；产物检查不替代实际桌面的登录、文件对话框和下载验收。

## 启动桌面应用

在仓库根目录中运行：

```bash
pnpm --dir apps/desktop install --frozen-lockfile
pnpm --dir apps/desktop tauri dev
```

如果设置了全局 `CARGO_TARGET_DIR`，本仓库的 PowerShell、Bash 和 Tauri 本地入口会把它当作 target 根目录，并自动追加仓库目录名；例如根目录为 `D:\coding\rust\target` 时，本项目实际使用 `D:\coding\rust\target\bdl`。没有设置时仍使用仓库内的 `target`。

本地打包默认不生成 Tauri updater 签名产物，因此不需要 updater 私钥。只有需要生成更新包时才在 Windows 使用 `./scripts/package.ps1 -UpdaterArtifacts`，或在 macOS/Linux 使用 `./scripts/package.sh --updater-artifacts`。

只运行前端：

```bash
pnpm --dir apps/desktop dev
```

## 检查与打包

在 Windows 上运行自动化项目检查（包含浏览器功能测试）：

```powershell
./scripts/check.ps1
```

在 macOS 上运行对应检查：

```bash
./scripts/check.sh
```

两者都包含 Rust 格式化、Clippy、工作区测试、Vue 类型检查、ESLint、Stylelint、Vitest 和生产构建。Windows 默认运行浏览器功能测试；macOS 可使用 `./scripts/check.sh --browser`。单独运行：`pnpm --dir apps/desktop test:browser`。需要检查构建后的功能时，先构建，再设置 `BDL_BROWSER_PRODUCTION=1` 运行该命令。

### 视觉验收与功能测试的边界

浏览器测试使用模拟的原生 API，只检查交互与业务状态；组件测试检查行为和可访问语义；Windows WebView smoke 仅检查启动、DOM 就绪和脚本错误。这些检查通过均不表示界面视觉合格，也不证明被模拟的原生集成正常。失败截图和 trace 仅供排查。

禁止用截图基线比对、尺寸/间距/样式断言替代视觉验收。修改界面后，必须查看实际运行截图，亲自检查布局、对齐、间距、控件尺寸、组件一致性、状态、文字和留白，发现问题修改后复看；无法查看时明确报告“视觉尚未验收”。不要将生成截图或 CI 通过写成视觉验收通过。

### 诊断、日志与清理

“设置 → 网络与维护”中的下载代理分为“使用系统代理”“手动设置代理”和“不使用代理（直连）”。空的旧代理地址迁移为使用系统代理，旧的非空地址迁移为手动设置；不使用代理必须显式选择。手动模式必须填写有效地址，切换到其他模式后保留该地址，但不使用它。模式用于任务资源下载、软件站更新信息查询及桌面更新检查/下载；解析和登录保留各自原有连接方式，外部浏览器及 Android 浏览器下载不受控制。保存后新启动或重试的任务、新发起的更新检查使用新模式，已经运行的请求保留启动时配置。

使用系统代理保留 HTTP 客户端的系统及环境变量代理规则，未配置时直接连接；不使用代理显式禁用这些自动规则。复制/导出诊断及任务失败日志记录所选 `proxy_mode`，不将 system 模式描述为确认已连接某个代理，也不再用自定义地址是否为空判断代理开关。

任务详情分为概览、轨道和日志。复制诊断包含当前任务、环境和最近 20 条日志，不重复附带事件列表；“导出任务诊断”包含该任务保留的日志，“设置 → 网络与维护”中的“导出全部诊断”包含整个队列及最近操作错误。请求发送失败保留 HEAD/GET、Range 或分段上下文、连接/超时标记及最多八层脱敏原因链。

任务日志保存在 SQLite，启动及写入时清理超过 30 天的记录，每任务最多保留 1000 条；总量仍随保留的任务数增长。“清理已结束任务日志”只删除已完成、失败和已取消任务的日志，保留其他状态、任务记录、完成记录和下载文件。面板显示日志正文大小，不等同整个数据库的磁盘占用；删除后的数据库页可复用，数据库文件未必立即缩小。

诊断导出文件保存在应用数据目录的 `diagnostics/`，每次导出保留最近 10 份，并提供独立清理入口。只处理本应用命名的普通 JSON 文件，不递归进入子目录或跟随符号链接/目录联接；移动到其他目录的报告由用户管理。本次运行的最近操作错误仍只保留 20 条，退出清空，也可手动清空。

检查通过后构建本机安装包：

```powershell
./scripts/package.ps1
```

```bash
./scripts/package.sh
```

`package.sh` 默认关闭本地 updater artifact 生成，因此不需要 Tauri 私钥。需要验证签名更新包时，先配置签名环境变量，再传入 `--updater-artifacts`。项目当前采用无需付费 Apple 账号的 macOS 发布流程：`tauri.macos.conf.json` 使用 ad-hoc identity `-`，发布包不会进行 Developer ID 签名或 Apple 公证。相关验证和用户提示见 [发布检查清单](RELEASE_CHECKLIST.md)。

版本同步脚本也提供两个平台入口：

```powershell
./scripts/version.ps1 patch
```

```bash
./scripts/version.sh patch
```

## 项目结构

```text
crates/bdl-core       领域模型、解析、任务规划、下载、存储与诊断
crates/bdl-tauri      队列编排、Tauri 命令、事件、系统与凭据集成
crates/bdl-cli        面向用户和脚本的 bdl 命令行入口
apps/desktop          Vue 3 + Tailwind CSS + Tauri 桌面界面
```

下载链路：

```text
来源 → 解析器 → 统一内容模型 → 下载规划 → 持久化队列 → 下载器 → 后处理 → 完成记录
```

Rust 核心维护可恢复任务的真实状态，Vue 前端负责展示和发送用户指令。更完整的模块边界与不变量见 [架构文档](ARCHITECTURE.md)。

## 参与开发

提交代码前，请阅读 [贡献指南](../CONTRIBUTING.md)，并根据变更范围运行相应检查。
