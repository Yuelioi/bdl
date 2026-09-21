# Android 适配计划

状态：Android bootstrap、Keystore 凭据、SAF 用户输出边界、MP4/MKV 媒体合并、运行中任务前台服务、启动时队列恢复和计划任务 WorkManager 提醒已完成第一轮实现。媒体合并已从系统 `MediaExtractor + MediaMuxer` 切换为 APK 内置的 ARM64 FFmpeg。尚未完成内置 FFmpeg 的 MuMu/真机下载闭环、封面/字幕嵌入，以及应用进程被杀后无需用户介入的持续下载，因此仍不可作为可用版本发布。

## 当前进度（2026-09-21）

- 已生成 Tauri 2 Android 工程，并将应用启动组装迁到带 `mobile_entry_point` 的 library entry；桌面 `main.rs` 复用同一入口。
- 桌面 updater/process 插件和窗口权限已与移动端隔离；Android 界面不再展示应用内更新、桌面窗口控制、普通文件/目录打开、桌面路径选择和 FFmpeg 路径设置。
- Linux keyring 依赖已限制到 Linux。Android 已在现有 `SecureStore` 边界下接入本地 Keystore 插件：AES-GCM 密钥保存在 AndroidKeyStore，Cookie 密文与 IV 保存在应用私有 SharedPreferences。该链路已通过构建验证，尚未在真实设备上验证跨重启持久化与注销清理。
- Android 下载、分片、SQLite 与媒体处理继续使用应用私有路径；最终成品通过独立 `DocumentTree` 输出目标导出到 SAF/content URI。已实现持久 URI 授权、嵌套目录、跳过/覆盖/追加后缀三种重名策略、临时文件提交，以及“导出成功后才标记任务完成”。覆盖已有文件时先备份旧文档，再提交新文档，失败时尝试恢复旧文档。
- 下载弹窗和设置页已接入系统目录选择器；任务列表在 Android 显示用户可见的导出相对路径，不暴露应用私有工作路径。
- `bdl-tauri` 已增加可注入的媒体合并 backend：桌面继续复用现有 FFmpeg 子进程；Android 由独立 `bdl-mobile-media` 插件调用 APK 内置 FFmpeg 原生库，不要求用户安装或配置 FFmpeg。
- Android 首轮曾使用系统 `MediaExtractor + MediaMuxer`。MuMu 实际下载暴露出系统封装兼容性问题后，Android backend 已改为固定版本的 JavaCPP Presets FFmpeg `8.1.2-1.5.14` + JavaCPP `1.5.14` ARM64 原生包，直接调用 libavformat/libavcodec/libavutil 做 stream-copy。合并仍使用应用私有临时文件后再提交最终文件。当前 Android UI 已开放 MP4 和 MKV；嵌入封面和嵌入字幕仍禁用，封面/字幕可作为独立归档文件保存。
- 下载队列本身继续由 Rust worker 持有，不依赖传输页存活；Tauri setup 启动时会主动启动队列处理，因此恢复不再依赖先打开传输页。Android 新增 `dataSync` 前台服务：有实际下载任务运行时启动，队列空闲时停止，并通过常驻下载通知提高切后台后的进程存活优先级。未来计划时间会全部同步到 WorkManager，每个时间点独立持久化提醒；应用进程已被系统终止时，Worker 只发通知引导用户打开 BDL，打开后由 Rust 队列按持久状态继续处理，不在 Kotlin Worker 中复制下载逻辑。Android 13+ 通知运行时权限已接入：首次为 `prompt` 时请求，用户拒绝后不反复触发系统权限框，并在应用内提示后台状态与定时提醒可能不可见。WorkManager 触发时间仍可能受系统省电策略延后。
- 手机壳层已增加 `platform-mobile` 布局：主导航从桌面左侧栏切为底部五项导航，并加入顶部/底部 safe-area inset；解析页增加显式“粘贴链接”按钮；传输列表保留完整列宽并支持触摸横向滚动，操作列在窄屏保持右侧可见。桌面布局不受影响。
- 当前生成工程为 `minSdk 24`、`targetSdk 36`、`compileSdk 36`。首发仍只验证 ARM64。
- 本机默认 JDK 26 与当前 Gradle/Kotlin 链路不兼容；`apps/desktop/scripts/tauri.mjs` 在 Android 命令下会在需要时自动使用 Android Studio 自带 JDK 21，也可通过 `BDL_ANDROID_JAVA_HOME` 显式指定。
- 切换到内置 FFmpeg 后，`node scripts/tauri.mjs android build --debug --target aarch64` 已重新通过，APK/AAB 均成功产出。APK 结构检查已确认 `libavformat.so`、`libavcodec.so`、`libavutil.so`、JavaCPP/JNI bridge 以及其余 FFmpeg 运行库实际打入 `lib/arm64-v8a/`；不需要的 `ffmpeg`/`ffprobe` CLI 已从 Android 包中排除。
- 新增 `scripts/package-android-release.mjs` / `pnpm --dir apps/desktop android:release` 正式发布入口：初始化生成工程后注入 release signing config，构建 ARM64 APK/AAB，并用 Android `apksigner` 校验 APK。tag release workflow 在配置 `ANDROID_KEY_BASE64`、`ANDROID_KEY_ALIAS`、`ANDROID_KEY_PASSWORD`（可选独立 `ANDROID_KEYSTORE_PASSWORD`）后上传 Android 资产；缺少固定发布 Keystore 时会明确跳过 Android 资产，不使用临时 debug key 代替正式签名。
- MuMu 模拟器人工 smoke 已能安装启动并进入下载流程。首个运行时问题是媒体插件 `MuxArgs` 无法被 Tauri/Jackson 反序列化；现已改为 `@InvokeArg` 无参参数类，并同步修正 execution 插件的同类参数，回归测试通过。MuMu 旧 WebView 对 OKLCH/color-mix 支持不足导致主题近似黑白，现已增加 sRGB fallback；移动端未登录时右上角直接显示“登录”按钮。第二轮 smoke 还暴露了状态栏覆盖登录按钮和系统 MediaMuxer 合并失败；状态栏已由 Android root WindowInsets 注入真实 system-bar/cutout 边距，媒体路径则已切到 APK 内置 FFmpeg，等待新 APK 在同一素材上复验。
- 当前 `adb devices` 仍无连接设备，因此 MuMu 只能由用户人工 smoke，无法从本机抓取 logcat/WebView 版本；真实 ARM64 设备安装、后台与权限交互 smoke 仍待完成。

## 目标与复用范围

复用 Rust 解析、画质偏好、任务规划、限速和配置默认值机制，以及 Vue 的业务状态和表单。首次面向 ARM64 手机；其他架构、最低 Android 版本和分发渠道在原型验证后确定。

Linux 阶段沿用桌面的 FFmpeg 进程、普通文件路径和现有凭据接口，不为了尚未实现的移动端增加空适配器或一个包办所有操作的平台接口。

## 实施顺序

1. **入口与系统能力。** 启动入口、桌面 updater/process、桌面窗口权限、目录/FFmpeg 路径入口与普通文件打开动作已经完成第一轮隔离。凭据继续沿用 `SecureStore`，Android Keystore 实现已接入并构建通过；真机持久化验证仍待完成。
2. **媒体处理原型。** 已把媒体合并抽成平台 backend；桌面实现继续调用 FFmpeg，Android 已从系统 `MediaExtractor + MediaMuxer` 迁移到 APK 内置 FFmpeg 8.1.2 ARM64 原生库，通过 libavformat/libavcodec/libavutil 做 MP4/MKV stream-copy，并已接入现有下载队列。接口不向业务层暴露命令行参数或 Android API。MuMu/真机真实素材验证仍待完成；嵌入封面/字幕仍未开放。
3. **输出与恢复。** 下载分片、SQLite、临时媒体文件放在应用私有目录；最终成品通过独立输出目标保存到用户授权的 SAF 目录，普通路径与 content URI 已明确分离。持久授权、重名策略、原子式临时提交和“保存成功后才完成”已实现；授权撤销、空间不足、复制中断与重试仍需真机故障注入验收。
4. **后台执行。** 队列调度已经在 Rust 后端运行，页面销毁不会丢失 worker；应用启动时会直接启动队列处理。Android `dataSync` 前台服务与下载通知已接入运行中的任务；所有未来计划时间会同步为独立 WorkManager 请求，取消/改期时按当前 Rust 队列重建，进程死亡后仍能到点提醒用户打开应用。当前不会在 WorkManager 中重复实现 Rust 下载器，因此不承诺进程死亡后无人值守继续下载，也不承诺锁屏后无限运行；仍需真机验证系统延迟、前台服务超时和后台启动限制。
5. **手机界面与发布。** 手机主壳已切为底部导航并处理 safe area；Android 13+ 通知运行时权限流程已接入并通过构建验证。继续调整下载弹窗、长列表和目录授权流程，并在真机验证通知拒绝后提示、重新开启通知、窄屏交互与系统权限状态。确定最低系统版本，验证 ABI 打包，完成签名、升级及权限撤销测试。

## 媒体 backend 策略

Android 首轮尝试过系统 `MediaExtractor + MediaMuxer`，但真实 MuMu 下载样本暴露出封装兼容性问题。当前策略改为随 APK 固定打包 FFmpeg `8.1.2-1.5.14` / JavaCPP `1.5.14` 的 ARM64 原生库，并直接使用 FFmpeg C API wrapper 做 stream-copy，不依赖 FFmpegKit，也不要求设备额外安装可执行文件。

- 当前 Android 支持 MP4 和 MKV 的视频/音频轨道合并；不自动转码。
- 当前 Android 不开放嵌入封面或嵌入字幕；这些能力不会静默降级为别的格式。
- 真机验收需覆盖 AVC、HEVC、AV1 与实际音频轨道，并比较关键时间戳、色彩/HDR/杜比视界元数据是否完整保留。
- 首发先只打 ARM64 classifier；后续若增加其他 ABI，需要分别验证对应原生包和包体。
- 发布前需要记录内置 FFmpeg/JavaCPP 的许可证、源码获取方式和第三方声明，并量化 release 包体影响。

## 验收门槛

- 与桌面用相同素材验证 Android 当前承诺的 MP4、MKV、纯音频、HDR 和杜比视界；嵌入字幕和嵌入封面在 Android 首轮应明确不可选。无法封装的组合明确报错，不自动转码或静默降级。
- 无需用户另装 FFmpeg 或指定其路径；设备不能播放某种编码，不应等同于不能下载或合并该轨道。
- 验证锁屏、切后台、杀进程重启、断网恢复、授权撤销、空间不足及最终文件保存失败；不损坏现有文件，也不误报完成。
- 验证运行任务启动/停止前台服务、通知权限被拒绝、系统前台服务时限、应用从最近任务划掉后的实际行为，以及进程终止后多个计划时间仍能分别触发 WorkManager 提醒；这些系统行为未真机验证前不宣称可长期后台下载或精确准点唤醒。
- 使用旧版配置缺字段时的通用默认值机制；保留用户有效设置，不引入按版本号串联的配置迁移。

## 参考

- [Tauri 项目结构与移动入口](https://v2.tauri.app/start/project-structure/)
- [Android 用户文件访问](https://developer.android.com/training/data-storage/shared/documents-files)
- [Android 前台服务类型](https://developer.android.com/develop/background-work/services/fgs/service-types)
- [FFmpeg 流复制](https://ffmpeg.org/ffmpeg.html#Streamcopy)
- [FFmpeg 编译选项](https://github.com/FFmpeg/FFmpeg/blob/master/configure)
- [FFmpegKit 停止维护公告](https://github.com/arthenica/ffmpeg-kit)
- [FFmpeg 许可说明](https://ffmpeg.org/legal.html)
