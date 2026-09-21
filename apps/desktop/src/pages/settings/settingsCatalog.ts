import type { SelectOption } from '../../ui/Select.vue';
import type { SettingsSection } from './settingsSections';

export const settingsSections = [
  { id: 'settings-download', label: '下载', description: '目录、并发与恢复', icon: 'i-tabler-download' },
  { id: 'settings-media', label: '媒体', description: '清晰度与封装格式', icon: 'i-tabler-movie' },
  { id: 'settings-naming', label: '文件命名', description: '模板与重名处理', icon: 'i-tabler-file-text' },
  {
    id: 'settings-media-advanced',
    label: '编码与处理',
    description: '编码、分段与 FFmpeg',
    icon: 'i-tabler-adjustments-horizontal',
  },
  { id: 'settings-archive', label: '附加内容', description: '封面、字幕与弹幕', icon: 'i-tabler-files' },
  { id: 'settings-update', label: '应用更新', description: '版本检测与安装', icon: 'i-tabler-refresh' },
  { id: 'settings-maintenance', label: '网络与维护', description: '代理、日志与数据', icon: 'i-tabler-tool' },
] as const satisfies readonly SettingsSection[];

const options = <T extends readonly SelectOption[]>(items: T): T => items;

export const concurrentTaskOptions = options([
  { label: '1', value: '1' },
  { label: '2', value: '2' },
  { label: '3', value: '3' },
  { label: '5', value: '5' },
]);
export const retryCountOptions = options([
  { label: '0', value: '0' },
  { label: '1', value: '1' },
  { label: '3', value: '3' },
  { label: '5', value: '5' },
]);
export const videoQualityOptions = options([
  { label: '最优画质', value: 'best' },
  { label: 'SDR / 普通动态范围', value: 'sdr' },
  { label: '8K / 127', value: '127' },
  { label: '杜比视界 / 126', value: '126' },
  { label: 'HDR / 125', value: '125' },
  { label: '4K / 120', value: '120' },
  { label: '1080P60 / 116', value: '116' },
  { label: '1080P+ / 112', value: '112' },
  { label: '1080P / 80', value: '80' },
  { label: '720P60 / 74', value: '74' },
  { label: '720P / 64', value: '64' },
  { label: '480P / 32', value: '32' },
  { label: '360P / 16', value: '16' },
]);
export const audioQualityOptions = options([
  { label: '最佳可用', value: 'best' },
  { label: 'Hi-Res 无损 / 30251', value: '30251' },
  { label: '杜比全景声 / 30250', value: '30250' },
  { label: 'Dolby Audio / 30255', value: '30255' },
  { label: '高音质 / 30280', value: '30280' },
  { label: '中音质 / 30232', value: '30232' },
  { label: '低音质 / 30216', value: '30216' },
]);
export const outputFormatOptions = options([
  { label: 'MP4', value: 'mp4' },
  { label: 'MKV', value: 'mkv' },
]);
export const duplicateNamingOptions = options([
  { label: '已有文件则跳过（推荐）', value: 'skip_existing' },
  { label: '覆盖已有文件', value: 'overwrite_existing' },
  { label: '扩展文件名', value: 'append_suffix' },
]);
export const codecOptions = options([
  { label: '自动', value: 'auto' },
  { label: 'AVC / H.264', value: 'avc' },
  { label: 'HEVC / H.265', value: 'hevc' },
  { label: 'AV1', value: 'av1' },
]);
export const missingQualityOptions = options([
  { label: '选择接近的可用质量', value: 'lower' },
  { label: '阻止创建任务', value: 'skip' },
  { label: '提示后再处理', value: 'ask' },
]);
export const segmentCountOptions = options([
  { label: '1 段', value: '1' },
  { label: '2 段', value: '2' },
  { label: '4 段', value: '4' },
  { label: '8 段', value: '8' },
]);
export const archiveModeOptions = options([
  { label: '仅下载最终视频（最快）', value: 'fast' },
  { label: '下载全部附加内容', value: 'complete_archive' },
  { label: '自定义附加内容', value: 'custom' },
]);
export const logLevelOptions = options([
  { label: '调试', value: 'debug' },
  { label: '信息', value: 'info' },
  { label: '警告', value: 'warning' },
  { label: '错误', value: 'error' },
]);
