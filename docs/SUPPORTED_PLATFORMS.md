# Supported platforms

BDL 当前以 Windows 11 和 macOS 13+ 作为主要桌面目标。Linux x86_64 正在接入构建与发布矩阵，真实桌面验收完成前标为预览；不将配置了 CI 等同于验收通过。

| 平台                    | 支持级别 | CI                     | 发布包     | 人工验证重点                                  |
| ----------------------- | -------- | ---------------------- | ---------- | --------------------------------------------- |
| Windows 11 x64          | 主要支持 | 完整检查与视觉回归     | EXE/MSI    | 安装、标题栏、凭据存储、FFmpeg、暂停恢复      |
| Windows 10 x64          | 兼容支持 | 与 Windows 11 共用     | EXE/MSI    | WebView2、长路径、文件占用与权限              |
| macOS 13+ Apple Silicon | 主要支持 | 完整非视觉检查与包构建 | ad-hoc DMG | 本地加密凭据、原生窗口、Homebrew FFmpeg、首次打开 |
| macOS 13+ Intel         | 兼容支持 | 交叉架构发布构建       | ad-hoc DMG | Intel Homebrew、ad-hoc 签名与升级             |
| Ubuntu 22.04/24.04 x86_64 | 首发预览 | 22.04 CI 待首次运行；24.04 本地构建与局部检查通过 | AppImage/deb（已本地构建，未发布） | Secret Service、X11/Wayland、FFmpeg、更新 |
| 其他 Linux / ARM64      | 尚未验证 | 暂无 | 暂无 | 按发行版、CPU 单独确认 |
| Android                 | 后续计划 | 暂无 | 暂无 | 存储授权、后台下载、媒体合并、移动界面 |

Android 本轮只维护[适配计划](ANDROID_PLAN.md)，不增加移动端实现；优先完成 Linux 验证。

2026-09-21：WSL Ubuntu 24.04 x86_64 已完成原生 Release 构建、14 项 FFmpeg/凭据局部测试和两个安装包的结构检查。凭据测试使用测试后端，不代表真实 Secret Service 验收。该环境的构建产物仅用于本地测试，不能证明 Ubuntu 22.04 兼容性；签名更新和真实桌面操作尚未验收。

## Required checks

主要支持平台的发布候选必须通过：

- Rust format、Clippy 和 workspace tests。
- Vue 类型检查、生产构建、语义 lint、样式 lint 和前端测试。
- 普通视频、分 P、收藏夹/合集、UP 空间分页解析。
- 暂停、继续、取消、断点恢复、失败重试和刷新链接后重试。
- MP4/MKV 合并，以及适用时的封面、字幕、弹幕和 NFO 输出。
- 浅色/深色、100%/125%/150% 缩放和窄窗口操作。

macOS 发布候选还必须验证加密登录凭据跨重启持久化、注销删除凭据且不会触发钥匙串授权弹窗、Finder 打开文件/目录、原生红黄绿窗口控制、Apple Silicon 与 Intel 安装包、ad-hoc 签名、README 所述的首次打开流程和应用内更新。当前发布策略不要求 Developer ID 或 Apple 公证，Release 页面必须明确提示 macOS 包未经公证。
