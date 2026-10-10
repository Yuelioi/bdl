import { computed, onMounted, ref, watch } from 'vue';
import { environmentHealth } from '../../api/tauri';
import type { EnvironmentHealthSnapshot } from '../../api/dto';

import {
  defaultNamingTemplate,
  embeddingContainerError,
  selectedArchiveAssets,
  useSettingsStore,
} from '../../stores/settings';
import { isAndroidPlatform } from '../../platform/environment';
import { speedLimitMbError, toBytesPerSecond, toMbPerSecondInput } from '../../utils/speedLimit';

const errorMessage = (error: unknown): string => (error instanceof Error ? error.message : String(error));

export function useSettingsForm() {
  const settings = useSettingsStore();
  const draftHealth = ref<EnvironmentHealthSnapshot | null>(null);
  const draftChecking = ref(false);
  const draftCheckError = ref<string | null>(null);
  let draftCheckId = 0;
  watch(
    () => [settings.draft.ffmpeg_path, settings.draft.download_dir],
    () => {
      draftCheckId += 1;
      draftHealth.value = null;
      draftChecking.value = false;
      draftCheckError.value = null;
    },
    { flush: 'sync' },
  );
  const settingsEnvironmentHealth = computed(
    () =>
      draftHealth.value ??
      (settings.draft.ffmpeg_path === settings.saved.ffmpeg_path &&
      settings.draft.download_dir === settings.saved.download_dir
        ? settings.environmentHealth
        : null),
  );
  const checkDraftEnvironment = async () => {
    const checkId = ++draftCheckId;
    draftChecking.value = true;
    draftCheckError.value = null;
    try {
      const health = await environmentHealth({
        download_dir: settings.draft.download_dir,
        ffmpeg_path: settings.draft.ffmpeg_path,
      });
      if (checkId === draftCheckId) draftHealth.value = health;
    } catch (error) {
      if (checkId === draftCheckId) draftCheckError.value = errorMessage(error);
    } finally {
      if (checkId === draftCheckId) draftChecking.value = false;
    }
  };

  const settingsDownloadDir = computed({
    get: () => settings.draft.download_dir ?? '',
    set: (value: string) => settings.setDownloadDir(value),
  });
  const archiveAssets = computed(() => selectedArchiveAssets(settings.draft));
  const settingsArchiveCover = computed({
    get: () => archiveAssets.value.cover,
    set: (value: boolean) => settings.setArchiveAsset('cover', value),
  });
  const settingsArchiveSubtitles = computed({
    get: () => archiveAssets.value.subtitles,
    set: (value: boolean) => settings.setArchiveAsset('subtitles', value),
  });
  const settingsArchiveDanmaku = computed({
    get: () => archiveAssets.value.danmaku,
    set: (value: boolean) => settings.setArchiveAsset('danmaku', value),
  });
  const settingsArchiveNfo = computed({
    get: () => archiveAssets.value.nfo,
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
  const settingsGlobalSpeedLimitMb = ref('');
  const settingsGlobalSpeedLimitError = computed(() => speedLimitMbError(settingsGlobalSpeedLimitMb.value));
  const settingsGlobalSpeedInputDirty = computed(
    () =>
      settingsGlobalSpeedLimitMb.value.trim() !==
      toMbPerSecondInput(settings.draft.global_speed_limit_bytes_per_second),
  );
  const settingsFormChanged = computed(() => settings.changed || settingsGlobalSpeedInputDirty.value);
  const settingsEmbeddingFormatError = computed(
    () => settings.downloadPresetError ?? embeddingContainerError(settings.draft),
  );
  const updateGlobalSpeedLimit = (value: string) => {
    settingsGlobalSpeedLimitMb.value = value;
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
    settingsGlobalSpeedLimitMb.value = toMbPerSecondInput(settings.draft.global_speed_limit_bytes_per_second);
  };

  const restoreDefaultSettings = () => {
    settings.restoreDefaults();
    applyPlatformSettingsConstraints();
    settingsGlobalSpeedLimitMb.value = toMbPerSecondInput(settings.draft.global_speed_limit_bytes_per_second);
  };

  const saveSettings = async () => {
    if (settingsGlobalSpeedLimitError.value) return;
    await settings.save();
    settingsGlobalSpeedLimitMb.value = toMbPerSecondInput(settings.draft.global_speed_limit_bytes_per_second);
  };

  const formatNamingVariable = (name: string): string => `{${name}}`;

  onMounted(async () => {
    await settings.ensureLoaded();
    applyPlatformSettingsConstraints();
    settingsGlobalSpeedLimitMb.value = toMbPerSecondInput(settings.draft.global_speed_limit_bytes_per_second);
  });

  return {
    settingsEnvironmentHealth,
    draftChecking,
    draftCheckError,
    checkDraftEnvironment,
    settings,
    settingsDownloadDir,
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
    settingsGlobalSpeedLimitMb,
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
