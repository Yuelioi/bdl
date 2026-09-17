# Supported platforms

BDL 当前以 Windows 11 和 macOS 13+ 作为主要桌面目标。Linux 接受兼容性修复，但在进入正式发布矩阵前不承诺安装包或完整回归覆盖。

| 平台                    | 支持级别 | CI                     | 发布包     | 人工验证重点                                  |
| ----------------------- | -------- | ---------------------- | ---------- | --------------------------------------------- |
| Windows 11 x64          | 主要支持 | 完整检查与视觉回归     | EXE/MSI    | 安装、标题栏、凭据存储、FFmpeg、暂停恢复      |
| Windows 10 x64          | 兼容支持 | 与 Windows 11 共用     | EXE/MSI    | WebView2、长路径、文件占用与权限              |
| macOS 13+ Apple Silicon | 主要支持 | 完整非视觉检查与包构建 | ad-hoc DMG | 本地加密凭据、原生窗口、Homebrew FFmpeg、首次打开 |
| macOS 13+ Intel         | 兼容支持 | 交叉架构发布构建       | ad-hoc DMG | Intel Homebrew、ad-hoc 签名与升级             |
| Linux                   | 社区预览 | 暂无                   | 暂无       | Secret Service、WebKitGTK、发行版依赖         |

## Required checks

主要支持平台的发布候选必须通过：

- Rust format、Clippy 和 workspace tests。
- Vue 类型检查、生产构建、语义 lint、样式 lint 和前端测试。
- 普通视频、分 P、收藏夹/合集、UP 空间分页解析。
- 暂停、继续、取消、断点恢复、失败重试和刷新链接后重试。
- MP4/MKV 合并，以及适用时的封面、字幕、弹幕和 NFO 输出。
- 浅色/深色、100%/125%/150% 缩放和窄窗口操作。

macOS 发布候选还必须验证加密登录凭据跨重启持久化、注销删除凭据且不会触发钥匙串授权弹窗、Finder 打开文件/目录、原生红黄绿窗口控制、Apple Silicon 与 Intel 安装包、ad-hoc 签名、README 所述的首次打开流程和应用内更新。当前发布策略不要求 Developer ID 或 Apple 公证，Release 页面必须明确提示 macOS 包未经公证。
