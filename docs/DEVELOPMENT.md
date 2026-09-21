# BDL 开发指南

本文档面向需要在本地运行、调试或打包 BDL 的开发者。产品下载、FFmpeg 安装和基本使用方法请查看 [README](../README.md)。

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

在 Windows 上运行完整项目检查（包含 Windows 视觉基线）：

```powershell
./scripts/check.ps1
```

在 macOS 上运行对应检查：

```bash
./scripts/check.sh
```

两者都包含 Rust 格式化、Clippy、工作区测试、Vue 类型检查、ESLint、Stylelint、Vitest 和生产构建。Windows 是当前的规范视觉快照平台；macOS 如需主动运行视觉测试，可使用 `./scripts/check.sh --visual` 并单独评估平台渲染差异。

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
crates/bdl-cli        开发和故障诊断使用的命令行入口
apps/desktop          Vue 3 + Tailwind CSS + Tauri 桌面界面
```

下载链路：

```text
来源 → 解析器 → 统一内容模型 → 下载规划 → 持久化队列 → 下载器 → 后处理 → 完成记录
```

Rust 核心维护可恢复任务的真实状态，Vue 前端负责展示和发送用户指令。更完整的模块边界与不变量见 [架构文档](ARCHITECTURE.md)。

## 参与开发

提交代码前，请阅读 [贡献指南](../CONTRIBUTING.md)，并根据变更范围运行相应检查。
