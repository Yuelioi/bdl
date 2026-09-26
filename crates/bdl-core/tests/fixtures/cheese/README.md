# 课程响应回归样本

- `preview.anonymous.sanitized.json`：2026-09-26 匿名读取 ss18619 的 ep644827。实际返回 is_preview=1、MP4 durl、没有 DASH。保留播放模型字段，全部地址替换为 example.invalid；用于确认试看被识别为不完整内容。

- `no-coupon.sanitized.json`：2026-09-26 匿名读取公开课程 ss877726892 的目录响应，按已有课程模型裁剪；文本全部替换为占位符，保留 null、缺字段和分集 ID/数量等结构。不含登录信息。
- `drm.anonymous.sanitized.json`：2026-09-26 匿名读取 ss710811134 的免费先导片 ep2129649（fnval=4048），实际返回 DRM/HLS 且不含三个 accept_* 字段。保留模型字段，全部播放地址替换为 example.invalid，并移除 md5。不含登录信息或已购账号响应。
