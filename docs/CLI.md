# BDL 命令行

`bdl` 直接使用 `bdl-core`，无需启动桌面端，适合脚本和 Agent 调用。v0.5.0 起提供独立跨平台 CLI 归档，也支持源码安装。

## 安装

在仓库根目录使用 Rust 工具链安装：

```sh
cargo install --path crates/bdl-cli --locked
bdl --help
bdl --version
```

下载需要系统 FFmpeg，也可以通过 `--ffmpeg <路径>` 指定。CLI 不读取桌面端的设置、登录状态或下载队列。

## 解析

```sh
bdl parse BV1xx411c7mD
bdl parse BV1xx411c7mD --with-streams --json
bdl parse "https://space.bilibili.com/123/favlist?fid=456" --json
bdl parse "https://space.bilibili.com/49746395/lists/162475?type=season" --all --state .bdl/list-parse.json --json
```

支持 BV/AV、普通视频、收藏夹、UP 主、合集、系列、番剧和课程的输入分发。支持 `b23.tv` 短链接：最多跟随 5 次跳转，仅允许指定 B 站 HTTPS 域名，不携带 Cookie；跳转请求同样计入 HTTP 预算。验证页面或非允许目标立即停止。

默认只解析首批元信息；显式 `--all --state <文件>` 才会继续加载元信息页，支持收藏夹、UP 主、合集、系列及课程，全量模式仍受解析操作预算约束。JSON 返回核心的 `NormalizedSourceTree`，通过 `source.loaded_count`、`total_count` 和 `has_more` 判断加载范围。翻页会去重；服务端连续返回无新增内容但仍声称有下一页时，命令报错停止。

`--with-streams` 为普通视频的分 P、番剧/课程的当前聚焦集获取媒体流，同样受预算与节流限制；列表不会因此批量获取媒体流。该诊断选项不能与 `--all` 或 `--state` 一起使用。实际下载会在每个分 P 执行前单独补全，不依赖解析时保存的直链。

## 下载

```sh
bdl download BV1xx411c7mD BV1Q541167Qg --parts 1 --output downloads --json
bdl download BV1xx411c7mD --parts 1 -o downloads --quality 80 --codec hevc --container mkv
bdl download BV1xx411c7mD --parts 1 -o downloads --media-mode audio-only
bdl download BV1ZA411g7Sb -o downloads --parts 1,2
bdl download "https://space.bilibili.com/49746395/lists/162475?type=season" -o downloads --items 1,2
```

| 参数 | 可选值 / 默认值 |
| --- | --- |
| `--output`, `-o` | 必填，输出目录 |
| `--parts` | 普通多 P 视频的分 P 序号，从 1 开始，以逗号分隔，如 `1,2`；只获取所选分 P 的媒体流 |
| `--items` | 列表条目或番剧/课程集序号，从 1 开始，以逗号分隔，如 `1,2` |
| `--all` | 明确要求下载全部；必须配合 `--state`，不能与 `--items` / `--parts` 混用 |
| `--quality` | `best`（默认）、`sdr`、数字 qn，如 `80` / `120` / `125` |
| `--audio-quality` | `best`（默认）或数字音质 ID |
| `--codec` | `auto`（默认）、`avc`、`hevc`、`av1` |
| `--media-mode` | `audio-video`（默认）、`video-only`、`audio-only` |
| `--container` | `mp4`（默认）、`mkv` |
| `--missing-quality` | `lower`（默认，允许降档）、`skip`（不满足时拒绝创建任务并报错） |
| `--ffmpeg` | FFmpeg 可执行文件路径，省略时自动查找 |

下载必须指定 `--parts`、`--items` 或显式 `--all --state <文件>`，未选择范围会在联网前报错。按输入顺序处理；指定 `--items` 时只翻到覆盖最大所选序号的页，只补全和下载选中的条目。序号以接口返回顺序为准，不会按标题或日期重新排序。列表中一个视频条目包含多个分 P 时，会逐 P 下载该条目，仍受预算限制。

普通视频用 `--parts` 选择分 P；番剧/课程通过 `--items` 选择集数。选择保留来源顺序和原始编号，重复序号自动去重，越界序号报错。多个输入使用同一组选定序号。遇到不可用条目、权限不足、网络或下载错误时立即退出，先前完成的文件保留。

## 请求节流和手动恢复

- 默认每次命令最多执行 **50 个解析操作**（`--max-operations 1..1000`）和 **100 次实际 HTTP 请求尝试**（`--max-http-requests 1..1000`）。实际请求预算覆盖签名密钥、短链接和有限重试。节流统一在 HTTP 层执行：默认发起间隔 **0.5 秒**，即每秒最多 2 次（`--interval-seconds 0.5..120`，支持小数），不再叠加操作等待或每 10 次强制休息。
- 同一系统用户的 CLI 进程共享 HTTP 请求锁、**每 60 分钟 7200 次**预算和冷却状态。发送前写入计数，异常退出仍保留消耗及额外 30 秒等待预留；正常结束自动释放锁；按请求发起时间计算间隔，请求仍串行，慢响应会降低实际速率。
- 这些是本工具的保守工作量限制，**不是 Bilibili 公布的安全阈值，也不能保证不触发风控**。CDN 媒体传输不计入解析 HTTP 预算；桌面端、其他工具和其他系统用户不共享该限制。
- 识别到 403/412/429、已知源站限制码或验证提示立即停止，不切换备用 CDN 或自动重试；共享状态记录至少 5 分钟冷却，并尊重更长的 `Retry-After`。设置了 `--state` 时也记录检查点冷却。等待结束不表示限制已解除，不要循环启动或更换状态文件绕过停止。
- GET 连接失败、超时及 502/503/504 最多额外重试一次，仍经过共享节流并消耗预算。媒体返回 HTTP 404/410 时最多重新获取一次地址；403/412/429 不触发地址刷新。最终失败立即退出。
- 共享计数位于 Windows `%LOCALAPPDATA%/bdl/cli-http.json`，其他系统为 `$XDG_STATE_HOME/bdl/cli-http.json` 或 `$HOME/.local/state/bdl/cli-http.json`。不保存 Cookie、账号或请求 URL；损坏时拒绝请求，不静默重置预算。
- `--state <文件>` 每页保存元信息、每个成品保存完成记录；中断后在**相同命令及选项**上追加 `--resume`。已完成且文件大小匹配的成品跳过解析与下载；未完成部分重新获取地址。预算用尽返回非零退出码，需稍后手动恢复，不会在内部重置预算继续跑。
- 状态文件不保存 Cookie、媒体流或其请求头，但包含内容标题、来源和本地路径。请按私人数据处理。同一个状态文件由操作系统锁保护，第二个进程会报错；进程退出自动释放锁，不要手动删除 `.lock` 文件。相同路径已有文件时必须显式 `--resume`，避免误覆盖。恢复沿用保存的列表快照；若希望重新获取变化后的列表，使用新的状态文件。

```sh
bdl download BV1ZA411g7Sb --parts 1,2 -o downloads --state .bdl/ae.json --json
# 中断后稍后手动恢复；同样的输入和媒体选项
bdl download BV1ZA411g7Sb --parts 1,2 -o downloads --state .bdl/ae.json --resume --json
```

下载只生成最终媒体，不下载封面、字幕、弹幕或 NFO。合并完成且最终文件非空后清理原始轨道。检查点提供持久化批次恢复，当前没有常驻后台队列或跨进程任务调度；最终失败不自动继续下一输入。受限内容仍需要有效 Cookie 和账号自身访问权限。

## 独立打包

仓库的 `.github/workflows/cli.yml` 配置了 Windows x64、Linux x64、macOS ARM64/x64 的 CLI 测试、构建及归档；可手动触发或由相关 PR 触发。正式 Release 工作流会复用该矩阵，将四平台 CLI 归档和校验文件附加到版本；全部构建成功后才公开发布。

本机打包示例（Python 3.11+；其他平台替换二进制路径和 target）：

```sh
cargo build --locked --release -p bdl-cli
python scripts/package-cli.py --binary target/release/bdl.exe --target windows-x64
```

归档包含可执行文件、CLI 文档及许可证，另附 SHA-256 文件，输出到 `output/cli`。FFmpeg 仍需单独安装。

## Cookie 和诊断

```sh
bdl --cookie-file /path/to/cookie.txt verify-cookie --json
bdl --cookie-file /path/to/cookie.txt download BV1xx411c7mD --parts 1 -o downloads --json
bdl ffmpeg-check --json
bdl ffmpeg-check --ffmpeg /path/to/ffmpeg --json
```

Cookie 文件为 UTF-8 的完整 Cookie header 文本。也可以设置 `BDL_COOKIE` 环境变量；显式文件优先。CLI 不持久化 Cookie。`verify-cookie` 只检查非空 `SESSDATA` 的格式，不验证服务器登录有效性。

`ffmpeg-check` 返回 `available`、`path` 和 `version`。找不到 FFmpeg 时返回 `available: false`，诊断命令本身退出码仍为 0；可执行文件探测失败则返回错误。

## 脚本协议

`--json` 可放在子命令前后。成功结果写入 stdout，运行错误或参数错误以 `{"ok":false,"error":"..."}` 写入 stderr。帮助和版本始终为文本；文件清理警告也写入 stderr。

下载成功输出：

```json
{
  "downloads": [
    {
      "input": "BV1xx411c7mD",
      "source_title": "视频标题",
      "outputs": ["downloads/视频标题/P1 - 分 P 标题.mp4"]
    }
  ]
}
```

退出码：`0` 成功、`1` 运行失败、`2` 参数错误。批量下载只有全部成功后才输出 JSON 汇总；失败时 stdout 不提供部分结果，已完成文件仍保留。`parse --with-streams --json` 含有媒体直链及请求信息，请按私人下载数据处理。
