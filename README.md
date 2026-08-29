# BDL · Bilibili Download Lab

[![CI](https://github.com/Yuelioi/bdl/actions/workflows/ci.yml/badge.svg)](https://github.com/Yuelioi/bdl/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Yuelioi/bdl?display_name=tag&sort=semver)](https://github.com/Yuelioi/bdl/releases/latest)
[![License](https://img.shields.io/github/license/Yuelioi/bdl)](LICENSE)
[![Powered by bpi-rs](https://img.shields.io/badge/Powered%20by-bpi--rs-ff6699)](https://github.com/Yuelioi/bpi-rs)

一个专注、可靠、本地优先的哔哩哔哩桌面下载工具。BDL 基于 [bpi-rs](https://github.com/Yuelioi/bpi-rs) 实现 Bilibili API 能力，并使用 Rust、Tauri 与 Vue 构建，把来源解析、批量选择、下载队列、失败恢复和媒体后处理整合在统一的工作区中。

> 当前版本：`0.2.1` · Windows 优先 · 项目处于早期开发阶段

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

BDL 依赖 FFmpeg 完成音视频合并。你可以在设置中直接指定 `ffmpeg.exe`，也可以让应用使用系统 `PATH` 中的版本。

### 安装 FFmpeg（Windows）

FFmpeg 官网本身主要发布源码，所以下载页看起来会比较复杂。普通 Windows 用户不需要下载源码或自己编译，下面两种方式任选一种即可。

**方式一：下载 ZIP 后在 BDL 中选择（推荐）**

1. 下载 [ffmpeg-release-essentials.zip](https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip)。这是 [FFmpeg 官网列出的 Windows 构建](https://ffmpeg.org/download.html#build-windows)，`essentials` 版已足够 BDL 使用。
2. 将 ZIP 解压到一个固定位置，例如 `C:\Tools\ffmpeg`。不要解压后再移动或删除该文件夹。
3. 打开 BDL 的“设置 → 编码与处理 → FFmpeg 路径”，点击“选择”，找到解压目录中的 `bin\ffmpeg.exe` 并保存。启动时的环境检查窗口也可以直接点击“选择 FFmpeg”。

**方式二：使用 winget 安装**

在 PowerShell 中运行：

```powershell
winget install --id Gyan.FFmpeg.Essentials -e
```

安装完成后重新打开 BDL，将“FFmpeg 路径”留空即可使用系统 FFmpeg。也可在新的 PowerShell 窗口中验证：

```powershell
ffmpeg -version
```

如果提示找不到 `ffmpeg`，请先重启 BDL 或终端；仍无法识别时，使用方式一在 BDL 中直接选择 `ffmpeg.exe` 即可，无需手动配置环境变量。

## 使用方式

1. 在“解析”页面粘贴一个或多个视频链接或 BV/AV 号。
2. 浏览来源内容，选择需要下载的条目。
3. 确认清晰度、编码、封装格式和附加内容后创建任务。
4. 在“传输”页面管理进度、暂停、继续、重试或查看诊断信息。

账号收藏夹和订阅合集可从“内容库”进入。应用只会在本机使用登录凭据，不会把 Cookie 写入项目文件或诊断导出。

## 开发

本地运行、环境要求、项目检查、打包方法和代码结构见 [开发指南](docs/DEVELOPMENT.md)。准备提交代码时，请同时阅读 [贡献指南](CONTRIBUTING.md)。

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
