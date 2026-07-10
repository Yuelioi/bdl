# Security Policy

## Reporting a vulnerability

请不要通过公开 issue 报告可能泄露 Cookie、鉴权头、签名 URL、本地文件路径或用户数据的问题。

优先使用 GitHub 仓库的 **Private vulnerability reporting / Security advisory** 功能。报告中请包含：

- 受影响版本或提交。
- 最小复现步骤。
- 可能暴露的数据或可执行的操作。
- 已确认的缓解方式（如有）。

请勿附上真实 Cookie、访问令牌或未经脱敏的诊断包。维护者确认问题前，不要公开利用细节。

## Scope

重点关注：

- Cookie 与系统凭据存储。
- 日志、诊断导出和错误信息脱敏。
- 签名媒体 URL 与请求头。
- 文件路径、覆盖行为和目录清理边界。
- Tauri 命令能力与前端输入验证。
- 更新、安装包和构建供应链。

一般功能缺陷、解析失败或站点 API 变化可以使用公开 issue。

