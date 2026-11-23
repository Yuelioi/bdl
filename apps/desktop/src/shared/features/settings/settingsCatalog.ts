import type { SelectOption } from '../../ui/Select.vue';
import type { SettingsSection } from './settingsSections';

export const settingsSections = [
  { id: 'settings-download', mobilePage: 'saving', label: '下载与保存', group: '下载', icon: 'i-tabler-folder-down' },
  { id: 'settings-tasks', mobilePage: 'tasks', label: '速度与续传', group: '下载', icon: 'i-tabler-gauge' },
  { id: 'settings-media', mobilePage: 'media', label: '画质与编码', group: '下载', icon: 'i-tabler-adjustments' },
  { id: 'settings-presets', mobilePage: 'presets', label: '预设管理', group: '下载', icon: 'i-tabler-list-details' },
  { id: 'settings-links', mobilePage: 'links', label: '链接与解析', group: '应用', icon: 'i-tabler-link' },
  { id: 'settings-network', mobilePage: 'network', label: '下载代理', group: '应用', icon: 'i-tabler-world' },
  { id: 'settings-maintenance', mobilePage: 'storage', label: '存储与诊断', group: '应用', icon: 'i-tabler-database' },
  { id: 'settings-update', mobilePage: 'update', label: '应用更新', group: '应用', icon: 'i-tabler-refresh' },
] as const satisfies readonly SettingsSection[];

const options = <T extends readonly SelectOption[]>(items: T): T => items;

export const proxyModeOptions = options([
  { label: '使用系统代理', value: 'system' },
  { label: '手动设置代理', value: 'custom' },
  { label: '不使用代理（直连）', value: 'direct' },
]);

export const concurrentTaskOptions = options([
  { label: '1', value: '1' },
  { label: '2', value: '2' },
  { label: '3', value: '3' },
  { label: '5', value: '5' },
]);
export const globalSpeedLimitOptions = options([
  { label: '不限速', value: 'unlimited' },
  { label: '3 MB/s', value: '3' },
  { label: '10 MB/s', value: '10' },
  { label: '30 MB/s', value: '30' },
  { label: '自定义', value: 'custom' },
]);
export const retryCountOptions = options([
  { label: '0', value: '0' },
  { label: '1', value: '1' },
  { label: '3', value: '3' },
  { label: '5', value: '5' },
]);
export const videoQualityOptions = options([
  { label: '最佳可用', value: 'best' },
  { label: '仅 SDR', value: 'sdr' },
  { label: '8K', value: '127' },
  { label: '杜比视界', value: '126' },
  { label: 'HDR', value: '125' },
  { label: '4K', value: '120' },
  { label: '1080P60', value: '116' },
  { label: '1080P+', value: '112' },
  { label: '1080P', value: '80' },
  { label: '720P60', value: '74' },
  { label: '720P', value: '64' },
  { label: '480P', value: '32' },
  { label: '360P', value: '16' },
]);
export const videoQualityLevels = videoQualityOptions.filter((option) => !['best', 'sdr'].includes(option.value));
export const audioQualityOptions = options([
  { label: '最佳可用', value: 'best' },
  { label: 'Hi-Res 无损', value: '30251' },
  { label: '杜比全景声', value: '30250' },
  { label: 'Dolby Audio', value: '30255' },
  { label: '高音质', value: '30280' },
  { label: '中音质', value: '30232' },
  { label: '低音质', value: '30216' },
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
export const audioCodecOptions = options([
  { label: 'AAC', value: 'aac' },
  { label: 'FLAC', value: 'flac' },
  { label: 'E-AC-3 / Dolby', value: 'eac3' },
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
export const logLevelOptions = options([
  { label: '调试', value: 'debug' },
  { label: '信息', value: 'info' },
  { label: '警告', value: 'warning' },
  { label: '错误', value: 'error' },
]);
