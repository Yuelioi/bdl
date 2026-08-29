# BDL 开发指南

本文档面向需要在本地运行、调试或打包 BDL 的开发者。产品下载、FFmpeg 安装和基本使用方法请查看 [README](../README.md)。

## 环境要求

- Rust `1.88+`
- Node.js `22+`
- pnpm `11+`
- FFmpeg
- 对应平台的 [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/)

BDL 当前以 Windows 为主要开发和发布目标；其他系统的支持状态见 [支持平台](SUPPORTED_PLATFORMS.md)。

## 启动桌面应用

在仓库根目录中运行：

```powershell
pnpm --dir apps/desktop install --frozen-lockfile
pnpm --dir apps/desktop tauri dev
```

只运行前端：

```powershell
pnpm --dir apps/desktop dev
```

## 检查与打包

运行完整项目检查：

```powershell
./scripts/check.ps1
```

完整检查包含 Rust 格式化与 Clippy、工作区测试、Vue 类型检查、ESLint、Stylelint、Vitest、生产构建和视觉回归测试。

检查通过后构建安装包：

```powershell
./scripts/package.ps1
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
