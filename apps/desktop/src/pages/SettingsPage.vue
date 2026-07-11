<script setup lang="ts">
import { computed, ref } from 'vue';

import { defaultNamingTemplate, namingTemplatePresets, namingVariables } from '../stores/settings';
import UiButton from '../ui/Button.vue';
import UiCheckbox from '../ui/Checkbox.vue';
import UiInlineNotice from '../ui/InlineNotice.vue';
import UiSelect from '../ui/Select.vue';
import UiTextField from '../ui/TextField.vue';
import SettingsDownloadSection from './settings/SettingsDownloadSection.vue';
import SettingsMediaSection from './settings/SettingsMediaSection.vue';
import SettingsSectionNav from './settings/SettingsSectionNav.vue';
import {
  archiveModeOptions,
  codecOptions,
  duplicateNamingOptions,
  logLevelOptions,
  missingQualityOptions,
  segmentCountOptions,
  settingsSections,
} from './settings/settingsCatalog';
import type { SettingsSectionId } from './settings/settingsSections';
import { useSettingsForm } from './settings/useSettingsForm';

const settingsForm = useSettingsForm();
const {
  settings,
  settingsArchiveMode,
  settingsArchiveCover,
  settingsArchiveSubtitles,
  settingsArchiveDanmaku,
  settingsArchiveNfo,
  settingsCodec,
  settingsMissingQualityPolicy,
  settingsDuplicateNamingStrategy,
  settingsNamingTemplate,
  settingsSegmentCount,
  settingsGlobalSpeedLimitError,
  settingsFormChanged,
  settingsEmbeddingFormatError,
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
} = settingsForm;

const activeSettingsSection = ref<SettingsSectionId>('settings-download');
const currentSettingsSection = computed(
  () => settingsSections.find((section) => section.id === activeSettingsSection.value) ?? settingsSections[0],
);
</script>

<template>
  <section class="page-grid settings-grid">
    <section class="panel settings-panel">
      <div class="panel-heading settings-heading">
        <div class="settings-heading-copy">
          <span>偏好与维护</span>
          <h2>设置</h2>
          <p>调整新任务的默认行为，当前下载只会立即应用全局限速。</p>
        </div>
        <div class="settings-actions">
          <span v-if="settingsFormChanged" class="dirty-indicator">未保存</span>
          <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="restoreDefaultSettings">
            恢复默认
          </UiButton>
          <UiButton
            variant="ghost"
            :disabled="settings.loading || settings.saving || !settingsFormChanged"
            @click="resetSettingsDraft"
          >
            撤销
          </UiButton>
          <UiButton
            :disabled="
              settings.loading ||
              settings.saving ||
              !settingsFormChanged ||
              Boolean(settings.namingTemplateError) ||
              Boolean(settingsGlobalSpeedLimitError) ||
              Boolean(settingsEmbeddingFormatError)
            "
            @click="saveSettings"
          >
            {{ settings.saving ? '保存中' : '保存' }}
          </UiButton>
        </div>
      </div>

      <UiInlineNotice v-if="settings.notice" :tone="settings.notice.tone">
        {{ settings.notice.message }}
      </UiInlineNotice>

      <div class="settings-workspace">
        <SettingsSectionNav v-model="activeSettingsSection" :sections="settingsSections" />

        <div class="settings-content">
          <header class="settings-section-heading">
            <span class="settings-section-icon" aria-hidden="true">
              <UIcon :name="currentSettingsSection.icon" />
            </span>
            <div>
              <h3>{{ currentSettingsSection.label }}</h3>
              <p>{{ currentSettingsSection.description }}</p>
            </div>
          </header>

          <SettingsDownloadSection v-if="activeSettingsSection === 'settings-download'" :form="settingsForm" />

          <SettingsMediaSection v-else-if="activeSettingsSection === 'settings-media'" :form="settingsForm" />

          <section v-else-if="activeSettingsSection === 'settings-naming'" class="settings-block">
            <UiTextField v-model="settingsNamingTemplate" label="命名模板" :placeholder="defaultNamingTemplate" />
            <p v-if="settings.namingTemplateError" class="settings-field-error">{{ settings.namingTemplateError }}</p>
            <div class="template-presets" aria-label="命名模板预设">
              <button
                v-for="preset in namingTemplatePresets"
                :key="preset.label"
                type="button"
                @click="settings.setNamingTemplate(preset.value)"
              >
                {{ preset.label }}
              </button>
              <button type="button" @click="resetNamingTemplate">恢复默认</button>
            </div>
            <div class="settings-preview">
              <span>文件名预览</span>
              <code>{{ settings.namingPreview }}</code>
            </div>
            <UiSelect v-model="settingsDuplicateNamingStrategy" label="重名处理" :options="duplicateNamingOptions" />
            <p class="settings-note">{{ settingsDuplicateDescription }}</p>
            <details class="template-help">
              <summary>查看可用变量</summary>
              <div>
                <span v-for="variable in namingVariables" :key="variable.name" :title="variable.desc">
                  <code>{{ formatNamingVariable(variable.name) }}</code>
                  <small>{{ variable.desc }}</small>
                </span>
              </div>
            </details>
          </section>

          <section v-else-if="activeSettingsSection === 'settings-media-advanced'" class="settings-block">
            <div class="settings-inline-grid">
              <UiSelect v-model="settingsCodec" label="视频编码偏好" :options="codecOptions" />
              <UiSelect
                v-model="settingsMissingQualityPolicy"
                label="目标质量不可用"
                :options="missingQualityOptions"
              />
            </div>
            <UiSelect v-model="settingsSegmentCount" label="单任务分段数" :options="segmentCountOptions" />
            <div class="directory-row">
              <UiTextField
                v-model="settingsFfmpegPath"
                label="FFmpeg 路径"
                placeholder="留空时使用系统 PATH 中的 ffmpeg"
              />
              <UiButton
                variant="secondary"
                :disabled="settings.loading || settings.saving"
                @click="settings.chooseFfmpegPath"
              >
                选择
              </UiButton>
            </div>
            <div v-if="settings.draft.ffmpeg_path" class="settings-actions">
              <UiButton
                variant="ghost"
                :disabled="settings.loading || settings.saving"
                @click="settings.clearFfmpegPath"
              >
                使用系统 FFmpeg
              </UiButton>
            </div>
            <p class="settings-note">
              编码是偏好而非硬性过滤；目标清晰度不存在时，默认会选择最接近的可用轨道。选择“提示后再处理”时，当前版本会阻止创建任务并显示原因。
            </p>
          </section>

          <section v-else-if="activeSettingsSection === 'settings-archive'" class="settings-block">
            <UiSelect v-model="settingsArchiveMode" label="下载范围" :options="archiveModeOptions" />
            <div v-if="settings.draft.archive_mode === 'custom'" class="archive-option-grid">
              <UiCheckbox
                v-model="settingsArchiveCover"
                label="保存封面"
                :disabled="settings.loading || settings.saving"
              />
              <UiCheckbox
                v-model="settingsArchiveSubtitles"
                label="保存字幕"
                :disabled="settings.loading || settings.saving"
              />
              <UiCheckbox
                v-model="settingsArchiveDanmaku"
                label="保存弹幕"
                :disabled="settings.loading || settings.saving"
              />
              <UiCheckbox
                v-model="settingsArchiveNfo"
                label="生成 NFO"
                :disabled="settings.loading || settings.saving"
              />
            </div>
            <div class="archive-option-grid archive-single-grid">
              <UiCheckbox
                v-model="settingsRetainRawStreams"
                label="保留原始视频/音频轨道"
                :disabled="settings.loading || settings.saving"
              />
            </div>
            <div class="archive-option-grid embed-option-grid">
              <UiCheckbox
                v-model="settingsEmbedCover"
                label="嵌入封面（仅 MKV）"
                :disabled="settings.loading || settings.saving"
              />
              <UiCheckbox
                v-model="settingsEmbedSubtitles"
                label="嵌入字幕（仅 MKV）"
                :disabled="settings.loading || settings.saving"
              />
            </div>
            <UiInlineNotice v-if="settingsEmbeddingFormatError" tone="danger">
              {{ settingsEmbeddingFormatError }}
            </UiInlineNotice>
            <p class="settings-note">{{ settingsArchiveDescription }}</p>
          </section>

          <section v-else class="settings-block">
            <UiTextField
              v-model="settingsProxyUrl"
              label="代理地址"
              placeholder="例如 http://127.0.0.1:7890，留空为直连"
            />
            <UiSelect v-model="settingsLogLevel" label="任务日志级别" :options="logLevelOptions" />
            <div class="directory-row">
              <UiTextField v-model="settingsDataDir" label="数据目录" placeholder="留空时使用当前工作目录下的 .bdl" />
              <UiButton
                variant="secondary"
                :disabled="settings.loading || settings.saving"
                @click="settings.chooseDataDir"
              >
                选择
              </UiButton>
            </div>
            <div v-if="settings.draft.data_dir" class="settings-actions">
              <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="settings.clearDataDir">
                使用默认数据目录
              </UiButton>
            </div>
            <p class="settings-note">数据目录影响任务库、账户摘要和维护文件，修改后下次启动生效。</p>
            <div class="settings-actions">
              <UiButton
                variant="secondary"
                :disabled="settings.loading || settings.saving"
                @click="settings.cleanupCache"
              >
                清理缓存
              </UiButton>
              <UiButton
                variant="secondary"
                :disabled="settings.loading || settings.saving"
                @click="settings.cleanupTemp"
              >
                清理临时文件
              </UiButton>
              <UiButton
                variant="secondary"
                :disabled="settings.loading || settings.saving"
                @click="settings.exportDiagnostics"
              >
                导出诊断
              </UiButton>
            </div>
          </section>

          <p v-if="settings.error" class="settings-error">{{ settings.error }}</p>
        </div>
      </div>
    </section>
  </section>
</template>

<style src="./SettingsPage.css"></style>
