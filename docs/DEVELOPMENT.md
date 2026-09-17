# BDL 开发指南

本文档面向需要在本地运行、调试或打包 BDL 的开发者。产品下载、FFmpeg 安装和基本使用方法请查看 [README](../README.md)。

## 环境要求

- Rust `1.88+`
- Node.js `22+`
- pnpm `11+`
- FFmpeg
- 对应平台的 [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/)

BDL 的桌面开发与发布目标包括 Windows 和 macOS；具体范围见 [支持平台](SUPPORTED_PLATFORMS.md)。

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
