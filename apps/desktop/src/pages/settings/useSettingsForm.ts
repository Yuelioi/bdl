import { computed, onMounted, ref } from 'vue';

import { defaultNamingTemplate, embeddingContainerError, useSettingsStore } from '../../stores/settings';
import { isAndroidPlatform } from '../../utils/platform';
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
  const settingsDuplicateDescription = computed(() => {
    if (settings.draft.duplicate_naming_strategy === 'skip_existing') {
      return '最终文件已存在时跳过整个任务；未完成的分段缓存仍会继续恢复。';
    }
    if (settings.draft.duplicate_naming_strategy === 'overwrite_existing') {
      return '使用原始路径重新生成最终文件。覆盖合并期间若异常退出，原有文件可能受损。';
    }
    return '保留已有文件，并生成“文件名 (1)”这类新路径。';
  });

  const applyPlatformSettingsConstraints = () => {
    if (!isAndroidPlatform()) return;
    if (settings.draft.output_extension !== 'mp4') {
      settings.setOutputExtension('mp4');
    }
    if (settings.draft.embed_cover) {
      settings.setEmbedCover(false);
    }
    if (settings.draft.embed_subtitles) {
      settings.setEmbedSubtitles(false);
    }
  };

  const resetNamingTemplate = () => {
    settings.setNamingTemplate(defaultNamingTemplate);
  };

  const resetSettingsDraft = () => {
    settings.resetDraft();
    applyPlatformSettingsConstraints();
    settingsGlobalSpeedLimitMib.value = toMibPerSecondInput(settings.draft.global_speed_limit_bytes_per_second);
  };

  const restoreDefaultSettings = () => {
    settings.restoreDefaults();
    applyPlatformSettingsConstraints();
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
    applyPlatformSettingsConstraints();
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
    settingsDuplicateDescription,
    resetNamingTemplate,
    resetSettingsDraft,
    restoreDefaultSettings,
    saveSettings,
    formatNamingVariable,
  };
}

export type SettingsForm = ReturnType<typeof useSettingsForm>;
