# BDL · Bilibili Download Lab

[![CI](https://github.com/Yuelioi/bdl/actions/workflows/ci.yml/badge.svg)](https://github.com/Yuelioi/bdl/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Yuelioi/bdl?display_name=tag&sort=semver)](https://github.com/Yuelioi/bdl/releases/latest)
[![License](https://img.shields.io/github/license/Yuelioi/bdl)](LICENSE)

一个专注、可靠、本地优先的哔哩哔哩桌面下载工具。BDL 使用 Rust、Tauri 与 Vue 构建，把来源解析、批量选择、下载队列、失败恢复和媒体后处理整合在统一的工作区中。

> 当前版本：`0.1.1` · Windows 优先 · 项目处于早期开发阶段

![BDL 解析页面](preview/home.png)

## 功能特性

- **丰富的来源支持**：普通视频、多 P 视频、UP 主投稿、收藏夹、订阅合集、番剧与课程。
- **按需解析**：列表阶段只加载标题、封面、分 P 等必要信息，创建任务时再获取媒体地址与编码信息。
- **批量工作流**：支持搜索、排序、范围选择、分页加载、继续加载和批量创建下载任务。
- **可靠的下载队列**：断点续传、并发分段、备用 CDN、暂停/继续、链接刷新、重试与启动恢复。
- **媒体后处理**：通过 FFmpeg 合并视频与音频，可选 MP4/MKV，并支持封面、字幕、弹幕和 NFO。
- **清晰的任务诊断**：提供任务事件、媒体轨道、原始日志与脱敏诊断包，方便定位失败原因。
- **账号内容入口**：登录后可浏览自己的收藏夹和订阅合集，并直接选择其中内容下载。
- **本地优先与隐私保护**：设置和任务数据保存在本机，Cookie 存入操作系统凭据存储。
- **浅色与深色主题**：统一的粉色品牌色、原生无边框窗口和系统主题跟随。
- **签名更新能力**：支持手动检查更新；自动检查默认关闭，安装前始终由用户确认。

## 界面预览

| 解析与选择 | 下载选项 |
| --- | --- |
| ![解析与选择](preview/home.png) | ![下载选项](preview/download-option.png) |

| 下载队列 | 设置 |
| --- | --- |
| ![下载队列](preview/downloading.png) | ![设置](preview/settings.png) |

<details>
<summary>查看深色模式</summary>

![BDL 深色模式](preview/dark-mode.png)

</details>

## 下载与安装

前往 [Releases](https://github.com/Yuelioi/bdl/releases/latest) 下载最新的 Windows 安装包。

首次发布完成前，Release 页面可能暂时没有可下载文件。BDL 当前未购买 Windows 代码签名证书，因此 Windows SmartScreen 可能显示未知发布者；应用内更新包仍会通过内置的 Tauri 签名密钥验证完整性。

BDL 依赖 FFmpeg 完成音视频合并。你可以在设置中指定 FFmpeg，也可以让应用使用系统 `PATH` 中的版本。

## 使用方式

1. 在“解析”页面粘贴一个或多个视频链接或 BV/AV 号。
2. 浏览来源内容，选择需要下载的条目。
3. 确认清晰度、编码、封装格式和附加内容后创建任务。
4. 在“传输”页面管理进度、暂停、继续、重试或查看诊断信息。

账号收藏夹和订阅合集可从“内容库”进入。应用只会在本机使用登录凭据，不会把 Cookie 写入项目文件或诊断导出。

## 开发

### 环境要求

- Rust `1.88+`
- Node.js `22+`
- pnpm `11+`
- FFmpeg
- 对应平台的 [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/)

### 启动桌面应用

```powershell
pnpm --dir apps/desktop install --frozen-lockfile
pnpm --dir apps/desktop tauri dev
```

只运行前端：

```powershell
pnpm --dir apps/desktop dev
```

### 检查与打包

```powershell
./scripts/check.ps1
./scripts/package.ps1
```

完整检查包含 Rust 格式化与 Clippy、工作区测试、Vue 类型检查、ESLint、Stylelint、Vitest、生产构建和视觉回归测试。

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

Rust 核心维护可恢复任务的真实状态，Vue 前端负责展示和发送用户指令。更完整的模块边界与不变量见 [架构文档](docs/ARCHITECTURE.md)。

## 数据与安全

- `settings.json` 保存应用设置。
- `tasks.sqlite` 保存任务、资源、完成记录和脱敏日志。
- `.bdlpart` 临时文件用于断点续传。
- 登录 Cookie 存放在操作系统凭据存储中。
- 诊断导出会移除 Cookie、鉴权头和签名媒体 URL。

请勿在 Issue 中提交未经检查的 Cookie、完整日志或私人下载地址。安全问题请按照 [SECURITY.md](SECURITY.md) 私下报告。

## 参与贡献

欢迎提交问题、改进文档和代码贡献。开始前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。较大的功能或架构变更建议先开 Issue 讨论，以避免重复工作。

## License

[MIT](LICENSE)

## Disclaimer

BDL 与哔哩哔哩（bilibili）无隶属、合作或授权关系。请遵守适用法律、平台条款和内容版权，仅下载你有权保存的内容。本项目不提供也不鼓励任何权限绕过行为。
