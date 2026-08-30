# Release checklist

## Before packaging

- [ ] 版本号、变更日志和 About 页面一致。
- [ ] `Cargo.lock` 与 `pnpm-lock.yaml` 已提交且使用 `--locked` / `--frozen-lockfile` 验证。
- [ ] `./scripts/check.ps1` 在 Windows 干净工作区通过，`./scripts/check.sh` 在 macOS 干净工作区通过。
- [ ] 按 `SUPPORTED_PLATFORMS.md` 完成主要平台人工矩阵。
- [ ] 从全新数据目录验证首次启动、登录、下载目录和 FFmpeg 检测。
- [ ] 从上一版本数据目录验证设置、任务、日志和恢复状态迁移。

## Download workflow

- [ ] 验证普通视频、多分 P、收藏夹/合集、UP 空间和番剧/课程。
- [ ] 验证大来源分页与“解析更多”，下载时按需获取媒体 URL。
- [ ] 验证暂停/继续不会把已完成合并误判为失败。
- [ ] 验证全局与单任务速度显示、限速和并发下载。
- [ ] 验证 MP4/MKV、封面扩展名、字幕/弹幕/NFO，以及原始轨道保留策略。

## Packaging and publication

- [ ] 安装、覆盖安装、卸载和应用数据保留行为符合说明。
- [ ] 安装包在干净系统启动，窗口可拖动、缩放、最小化和关闭。
- [ ] 发布说明包含已知问题、平台范围、校验值和升级提示。
- [ ] 安装包与诊断输出不包含开发路径、Cookie、令牌或签名 URL。
- [ ] 自动更新仍为最后阶段能力；启用前必须单独审核签名、回滚与渠道策略。

## macOS distribution

- [ ] Apple Silicon 和 Intel DMG 都已从干净系统安装并启动。
- [ ] `codesign --verify --deep --strict BDL.app` 通过，且 `codesign -dv --verbose=4 BDL.app` 显示 `Signature=adhoc`。
- [ ] 在干净 Mac 上确认 Gatekeeper 会提示该应用未经 Apple 验证，并确认 README 中的 Control-点击“打开”或“隐私与安全性 → 仍要打开”流程有效。
- [ ] 首次启动使用 `~/Library/Application Support/com.yueli.bdl`，默认输出使用 `~/Downloads/BDL`。
- [ ] QR/Cookie 登录写入 Keychain，重启后可读取，注销后凭据被删除。
- [ ] Finder 启动可发现 Homebrew/MacPorts FFmpeg；显式选择路径同样可用。
- [ ] 原生窗口控制、拖动、缩放、全屏、打开成品和打开目录行为正确。
- [ ] Release 正文明确标记 macOS 包为 ad-hoc 签名且未经 Apple 公证；README 安装指引仍与当前 macOS 界面一致。
- [ ] GitHub Actions 已配置 `TAURI_SIGNING_PRIVATE_KEY`（以及密钥有密码时的 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`）用于应用更新包；当前流程不需要任何 `APPLE_*` Secret。
