# Contributing to BDL

感谢你愿意改进 BDL。提交代码前，请先确认变更保持 Rust 核心与桌面展示层之间的边界。

## 开始之前

- 功能或行为变更建议先创建 issue，说明用户场景、预期行为和不在范围内的内容。
- 安全问题不要创建公开 issue，请阅读 [SECURITY.md](SECURITY.md)。
- 下载解析、任务状态、持久化和系统集成属于 Rust；Vue 负责展示、选择和交互。
- 不要把原始 Bilibili API 响应直接暴露给前端，先归一化为稳定 DTO。

## 本地开发

环境准备、依赖安装、本地启动和打包方法见 [开发指南](docs/DEVELOPMENT.md)。

## 提交前检查

Windows：

```powershell
./scripts/check.ps1
```

macOS：

```bash
./scripts/check.sh
```

如果只改前端，至少运行：

```powershell
pnpm --dir apps/desktop build
```

如果改 Rust，请增加或更新最窄范围的测试，并运行相关 crate 测试后再跑完整检查。

## UI 约定

- 使用 `apps/desktop/src/ui/` 的本地组件封装和 Tabler 图标。
- 遵守 `flightdeck/knowledge/bdl-downloader/design-system.md` 中的视觉与交互约束。
- 覆盖默认、悬停、焦点、选中、加载、空、错误和窄窗口状态。
- 关键功能不能只在 hover 中出现；尊重 `prefers-reduced-motion`。

## Pull request

请在 PR 中说明：

1. 用户问题与解决方式。
2. 变更的边界和未处理内容。
3. 验证命令与结果。
4. UI 变更的截图或录屏（包含窄窗口状态）。
5. 是否涉及持久化、Cookie、签名 URL、日志或输出文件。

保持提交小而完整，不要混入无关格式化或依赖升级。

平台支持范围见 [docs/SUPPORTED_PLATFORMS.md](docs/SUPPORTED_PLATFORMS.md)，维护者发布前使用 [docs/RELEASE_CHECKLIST.md](docs/RELEASE_CHECKLIST.md)。
