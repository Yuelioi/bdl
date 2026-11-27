# 桌面端 AI 接入

在 BDL 桌面端打开「设置 → AI 接入」，选择「开启接入」，再复制配置到支持 Streamable HTTP 和自定义 HTTP 请求头的 MCP 客户端。使用时保持桌面程序运行，安装桌面版后即可使用。

如果客户端要求分开填写连接参数，从复制的配置中取 `url` 和 `headers.Authorization`。地址为 `http://127.0.0.1:48105/mcp`，授权头的格式为 `Bearer <接入令牌>`。配置含令牌，仅交给需要访问 BDL 的本机客户端。

关闭接入会撤销当前令牌。重新开启后复制新配置，更新客户端连接。退出并重新打开 BDL 会恢复已开启的接入，原配置仍可使用。

| 工具 | 用途 |
| --- | --- |
| `bdl_parse` | 解析视频、剧集或合集。返回本页分 P ID，并在有更多内容时返回 `next_page`。 |
| `bdl_library` | 分页读取 BDL 已登录账号创建或收藏的文件夹。可继续解析返回的 `source_url`。 |
| `bdl_presets` | 查看默认和自定义下载预设，包括视频、音频和附件的输出设置。 |
| `bdl_download` | 用相同链接、页码和该页选中的分 P ID 创建任务。可指定 `preset_id`，省略时使用默认预设。沿用 BDL 的保存目录及下载队列。 |
| `bdl_queue` | 分页浏览「传输」中的任务，可按下载、暂停、失败或完成等状态筛选。 |
| `bdl_task_status` | 按任务 ID 查询当前状态。若主成品已完成且仍在原位置，同时返回本机文件路径。 |
| `bdl_task_action` | 按任务 ID 暂停等待或下载中的任务、继续已暂停的任务，或重试失败的任务。 |
| `bdl_settings_get` | 查看下载总限速、并发任务数、翻页间隔和默认预设 ID。 |
| `bdl_settings_update` | 按字段保存上述四项设置，一次可以更新其中一项或多项。 |

配置中的令牌允许客户端访问上述内容库、管理下载和修改这四项设置。多个分页的下载按页提交，确保页码与选中的分 P 对应。

例如用 `bdl_settings_update` 设置 10 MB/s 总限速并允许 2 个任务并发：

```json
{ "download_limit_mb_s": 10, "concurrent_tasks": 2 }
```

`download_limit_mb_s` 设为 `0` 表示不限速。`parse_interval_seconds` 设置翻页间隔，范围为 1～120 秒。修改 `default_preset_id` 时，使用 `bdl_presets` 返回的 ID。省略的字段保留原值，改动保存后立即生效。
