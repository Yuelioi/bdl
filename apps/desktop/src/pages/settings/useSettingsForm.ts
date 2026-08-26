import { computed, onMounted, ref } from 'vue';

import { defaultNamingTemplate, embeddingContainerError, useSettingsStore } from '../../stores/settings';
import { speedLimitMibError, toBytesPerSecond, toMibPerSecondInput } from '../../utils/speedLimit';

export function useSettingsForm() {
  const settings = useSettingsStore();

  const settingsDownloadDir = computed({
    get: () => settings.draft.download_dir ?? '',
    set: (value: string) => settings.setDownloadDir(value),
  });
  const settingsArchiveMode = computed({
    get: () => settings.draft.archive_mode,
    set: (value: string) => settings.setArchiveMode(value),
  });
  const settingsArchiveCover = computed({
    get: () => settings.draft.archive_assets.cover,
    set: (value: boolean) => settings.setArchiveAsset('cover', value),
  });
  const settingsArchiveSubtitles = computed({
    get: () => settings.draft.archive_assets.subtitles,
    set: (value: boolean) => settings.setArchiveAsset('subtitles', value),
  });
  const settingsArchiveDanmaku = computed({
    get: () => settings.draft.archive_assets.danmaku,
    set: (value: boolean) => settings.setArchiveAsset('danmaku', value),
  });
  const settingsArchiveNfo = computed({
    get: () => settings.draft.archive_assets.nfo,
    set: (value: boolean) => settings.setArchiveAsset('nfo', value),
  });
  const settingsOutputFormat = computed({
    get: () => settings.draft.output_extension,
    set: (value: string) => settings.setOutputExtension(value),
  });
  const settingsVideoQuality = computed({
    get: () => settings.draft.quality,
    set: (value: string) => settings.setVideoQuality(value),
  });
  const settingsAudioQuality = computed({
    get: () => settings.draft.audio_quality,
    set: (value: string) => settings.setAudioQuality(value),
  });
  const settingsCodec = computed({
    get: () => settings.draft.codec,
    set: (value: string) => settings.setCodec(value),
  });
  const settingsMissingQualityPolicy = computed({
    get: () => settings.draft.missing_quality_policy,
    set: (value: string) => settings.setMissingQualityPolicy(value),
  });
  const settingsDuplicateNamingStrategy = computed({
    get: () => settings.draft.duplicate_naming_strategy,
    set: (value: string) => settings.setDuplicateNamingStrategy(value),
  });
  const settingsNamingTemplate = computed({
    get: () => settings.draft.naming_template,
    set: (value: string) => settings.setNamingTemplate(value),
  });
  const settingsConcurrentTasks = computed({
    get: () => String(settings.draft.concurrent_tasks),
    set: (value: string) => settings.setConcurrentTasks(value),
  });
  const settingsRetryCount = computed({
    get: () => String(settings.draft.retry_count),
    set: (value: string) => settings.setRetryCount(value),
  });
  const settingsSegmentCount = computed({
    get: () => String(settings.draft.segment_count),
    set: (value: string) => settings.setSegmentCount(value),
  });
  const settingsGlobalSpeedLimitMib = ref('');
  const settingsGlobalSpeedLimitError = computed(() => speedLimitMibError(settingsGlobalSpeedLimitMib.value));
  const settingsGlobalSpeedInputDirty = computed(
    () =>
      settingsGlobalSpeedLimitMib.value.trim() !==
      toMibPerSecondInput(settings.draft.global_speed_limit_bytes_per_second),
  );
  const settingsFormChanged = computed(() => settings.changed || settingsGlobalSpeedInputDirty.value);
  const settingsEmbeddingFormatError = computed(() => embeddingContainerError(settings.draft));
  const updateGlobalSpeedLimit = (value: string) => {
    settingsGlobalSpeedLimitMib.value = value;
    if (!settingsGlobalSpeedLimitError.value) {
      settings.setGlobalSpeedLimitBytesPerSecond(toBytesPerSecond(value) ?? null);
    }
  };
  const settingsAutoRefreshExpiredUrls = computed({
    get: () => settings.draft.auto_refresh_expired_urls,
    set: (value: boolean) => settings.setAutoRefreshExpiredUrls(value),
  });
  const settingsStartupAutoRecovery = computed({
    get: () => settings.draft.startup_auto_recovery,
    set: (value: boolean) => settings.setStartupAutoRecovery(value),
  });
  const settingsFfmpegPath = computed({
    get: () => settings.draft.ffmpeg_path ?? '',
    set: (value: string) => settings.setFfmpegPath(value),
  });
  const settingsRetainRawStreams = computed({
    get: () => settings.draft.retain_raw_streams,
    set: (value: boolean) => settings.setRetainRawStreams(value),
  });
  const settingsEmbedCover = computed({
    get: () => settings.draft.embed_cover,
    set: (value: boolean) => settings.setEmbedCover(value),
  });
  const settingsEmbedSubtitles = computed({
    get: () => settings.draft.embed_subtitles,
    set: (value: boolean) => settings.setEmbedSubtitles(value),
  });
  const settingsProxyUrl = computed({
    get: () => settings.draft.proxy_url ?? '',
    set: (value: string) => settings.setProxyUrl(value),
  });
  const settingsLogLevel = computed({
    get: () => settings.draft.log_level,
    set: (value: string) => settings.setLogLevel(value),
  });
  const settingsDataDir = computed({
    get: () => settings.draft.data_dir ?? '',
    set: (value: string) => settings.setDataDir(value),
  });
  const selectedArchiveAssetLabels = computed(() => {
    const labels: string[] = [];
    if (settings.draft.archive_assets.cover) labels.push('封面');
    if (settings.draft.archive_assets.subtitles) labels.push('字幕');
    if (settings.draft.archive_assets.danmaku) labels.push('弹幕');
    if (settings.draft.archive_assets.nfo) labels.push('NFO');
    return labels;
  });
  const rawStreamCopy = computed(() => (settings.draft.retain_raw_streams ? '；保留原始视频/音频轨道' : ''));
  const embeddingCopy = computed(() => {
    const items: string[] = [];
    const canUseArchiveAssets =
      settings.draft.archive_mode === 'complete_archive' || settings.draft.archive_mode === 'custom';
    const coverSelected = settings.draft.archive_mode === 'complete_archive' || settings.draft.archive_assets.cover;
    const subtitlesSelected =
      settings.draft.archive_mode === 'complete_archive' || settings.draft.archive_assets.subtitles;
    if (canUseArchiveAssets && coverSelected && settings.draft.embed_cover) items.push('封面');
    if (canUseArchiveAssets && subtitlesSelected && settings.draft.embed_subtitles) items.push('字幕');
    return items.length > 0 ? `；嵌入${items.join('和')}` : '';
  });
  const settingsArchiveDescription = computed(() => {
    if (settings.draft.archive_mode === 'complete_archive') {
      return `下载最终视频，并额外下载可用的封面、字幕、弹幕和 NFO${rawStreamCopy.value}${embeddingCopy.value}。不可用的附加内容会记录在任务详情中。`;
    }

    if (settings.draft.archive_mode === 'custom') {
      const selected =
        selectedArchiveAssetLabels.value.length > 0 ? selectedArchiveAssetLabels.value.join('、') : '不下载附加内容';
      return `下载最终视频，并按设置下载：${selected}${rawStreamCopy.value}${embeddingCopy.value}。`;
    }

    return `仅下载最终视频${rawStreamCopy.value}${embeddingCopy.value}；不下载封面、字幕、弹幕或 NFO。`;
  });
  const settingsDuplicateDescription = computed(() =>
    settings.draft.duplicate_naming_strategy === 'overwrite_existing'
      ? '新任务会使用模板渲染出的原始路径；如果磁盘上已有同名文件，下载完成后会覆盖它。批量任务内部路径冲突仍会自动加后缀。'
      : '新任务遇到同名文件时自动生成“文件名 (1)”这类路径，不覆盖已有文件。',
  );

  const resetNamingTemplate = () => {
    settings.setNamingTemplate(defaultNamingTemplate);
  };

  const resetSettingsDraft = () => {
    settings.resetDraft();
    settingsGlobalSpeedLimitMib.value = toMibPerSecondInput(settings.draft.global_speed_limit_bytes_per_second);
  };

  const restoreDefaultSettings = () => {
    settings.restoreDefaults();
    settingsGlobalSpeedLimitMib.value = toMibPerSecondInput(settings.draft.global_speed_limit_bytes_per_second);
  };

  const saveSettings = async () => {
    if (settingsGlobalSpeedLimitError.value) return;
    await settings.save();
    settingsGlobalSpeedLimitMib.value = toMibPerSecondInput(settings.draft.global_speed_limit_bytes_per_second);
  };

  const formatNamingVariable = (name: string): string => `{${name}}`;

  onMounted(async () => {
    await settings.ensureLoaded();
    settingsGlobalSpeedLimitMib.value = toMibPerSecondInput(settings.draft.global_speed_limit_bytes_per_second);
  });

  return {
    settings,
    settingsDownloadDir,
    settingsArchiveMode,
    settingsArchiveCover,
    settingsArchiveSubtitles,
    settingsArchiveDanmaku,
    settingsArchiveNfo,
    settingsOutputFormat,
    settingsVideoQuality,
    settingsAudioQuality,
    settingsCodec,
    settingsMissingQualityPolicy,
    settingsDuplicateNamingStrategy,
    settingsNamingTemplate,
    settingsConcurrentTasks,
    settingsRetryCount,
    settingsSegmentCount,
    settingsGlobalSpeedLimitMib,
    settingsGlobalSpeedLimitError,
    settingsFormChanged,
    settingsEmbeddingFormatError,
    updateGlobalSpeedLimit,
    settingsAutoRefreshExpiredUrls,
    settingsStartupAutoRecovery,
    settingsFfmpegPath,
    settingsRetainRawStreams,
    settingsEmbedCover,
    settingsEmbedSubtitles,
    settingsProxyUrl,
    settingsLogLevel,
    settingsDataDir,
    settingsArchiveDescription,
    settingsDuplicateDescription,
    resetNamingTemplate,
    resetSettingsDraft,
    restoreDefaultSettings,
    saveSettings,
    formatNamingVariable,
  };
}

export type SettingsForm = ReturnType<typeof useSettingsForm>;
