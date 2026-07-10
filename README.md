# BDL

> A focused, local-first Bilibili downloader built with Rust, Tauri, and Vue.

BDL 把链接解析、批量选择、下载队列、失败恢复和媒体归档放进一个紧凑的桌面工作区。它面向需要稳定处理多分集、合集与长队列的用户，不试图成为播放器、内容推荐客户端或本地媒体库。

当前版本：`0.1.0`（早期开发阶段）

## 亮点

- 多来源解析：普通视频、UP 主投稿、收藏夹、合集/系列、番剧与课程。
- 统一选择模型：不同来源归一化成分组、条目、分集和媒体轨道。
- 批量工作流：多来源切换、搜索、排序、范围选择、分页加载与批量建任务。
- 可恢复传输：断点续传、分段下载、CDN 备用地址、链接刷新与启动恢复。
- 状态明确：暂停、取消、继续、重试、刷新链接后重试、移除和清理已完成互不混淆。
- 媒体处理：视频/音频选择、FFmpeg 合并、MP4/MKV、封面、字幕、弹幕和 NFO。
- 本地优先：任务、日志和设置保存在本机；Cookie 使用系统凭据存储。
- 面向排障：任务诊断、事件时间线、脱敏日志与诊断包导出。
- 专业深色模式：支持跟随系统、浅色与深色，偏好本地持久化且启动时无闪白。
- 账号内容库：登录后浏览自己创建或收藏的收藏夹，并把所选集合直接送入解析流程。

## 工作区

| 页面 | 作用 |
| --- | --- |
| 解析 | 输入一个或多个来源，检查归一化结果，批量选择并设置本次下载参数 |
| 内容库 | 浏览账号收藏夹与订阅合集，筛选并批量送入解析流程 |
| 传输 | 查看队列健康、速度与进度，执行状态相关操作，恢复失败任务 |
| 设置 | 配置下载、默认媒体、命名、归档、网络和维护行为 |

快捷键：`Ctrl+1` 解析、`Ctrl+2` 内容库、`Ctrl+3` 传输、`Ctrl+4` 设置、`Ctrl+L` 聚焦解析输入框。macOS 上可使用 `⌘`。

## 架构

```text
Input → Resolver → Normalized Tree → Planner → Queue → Fetcher → PostProcess → Completed Record
          Rust core owns durable downloader behavior          Vue owns presentation
```

```text
crates/bdl-core       解析、领域模型、规划、下载、命名、存储、诊断
crates/bdl-tauri      Tauri 命令、事件、队列编排、系统集成、安全存储
crates/bdl-cli        面向开发与诊断的命令行入口
apps/desktop          Vue 3 + Nuxt UI 桌面界面
```

前端只展示后端任务并发送命令，不推断或持久化下载真相。

## 开发环境

需要：

- Rust `1.85+`
- Node.js `22+`
- pnpm `11+`
- FFmpeg（用于视频/音频合并）
- 对应平台的 [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/)

Windows PowerShell：

```powershell
pnpm --dir apps/desktop install --frozen-lockfile
pnpm --dir apps/desktop tauri dev
```

只启动前端开发服务器：

```powershell
pnpm --dir apps/desktop dev
```

## 验证与打包

完整检查：

```powershell
./scripts/check.ps1
```

它会运行 Rust 格式检查、Clippy、工作区测试、Vue 类型检查/生产构建和 Git 空白检查。

生成桌面安装包：

```powershell
./scripts/package.ps1
```

## CLI

CLI 当前用于单视频开发与诊断：

```powershell
cargo run -p bdl-cli -- parse "BV..." --json
cargo run -p bdl-cli -- download "BV..." --output .\downloads --quality best
cargo run -p bdl-cli -- ffmpeg-check
```

完整的批量来源与队列体验以桌面应用为准。

## 数据与隐私

- `settings.json` 保存设置。
- `tasks.sqlite` 保存任务、资源、完成记录和脱敏日志。
- `.bdlpart` 文件支持下载恢复。
- 登录 Cookie 存放在操作系统凭据存储中，不写入 SQLite 或设置文件。
- 诊断导出会移除 Cookie、鉴权头和签名 URL。

## 路线图

- 重复任务检测与创建策略。
- 下载目录、FFmpeg 和数据目录健康检查。
- 前端组件/键盘交互测试与无障碍自动检查。
- 自动更新与更完整的发布流水线。
- 在核心稳定后评估队列调度、限速和账户资源入口。

## 参与项目

阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 了解开发流程。安全问题请按 [SECURITY.md](SECURITY.md) 私下报告。

## License

[MIT](LICENSE)

## Disclaimer

BDL 与哔哩哔哩（bilibili）无隶属或授权关系。请遵守适用法律、平台条款和内容版权，只下载你有权保存的内容。本项目不提供权限绕过能力。
