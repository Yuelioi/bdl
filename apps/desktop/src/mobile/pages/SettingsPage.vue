<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useMobileBack } from '../app/useMobileBack';
const emit = defineEmits<{ back: [] }>();
import SettingsLeaveDialog from '../../shared/features/settings/SettingsLeaveDialog.vue';
import UiButton from '../../shared/ui/Button.vue';
import UiInlineNotice from '../../shared/ui/InlineNotice.vue';
import SettingsDownloadSection from '../../shared/features/settings/SettingsDownloadSection.vue';
import SettingsMaintenanceSection from '../../shared/features/settings/SettingsMaintenanceSection.vue';
import SettingsMediaSection from '../../shared/features/settings/SettingsMediaSection.vue';
import PresetManager from '../../shared/features/settings/PresetManager.vue';
import SettingsUpdateSection from '../../shared/features/settings/SettingsUpdateSection.vue';
import SettingsCard from '../../shared/features/settings/SettingsCard.vue';
import ClipboardPreference from '../../shared/ui/ClipboardPreference.vue';

import { useUiStore } from '../../shared/stores/ui';
import { settingsSections } from '../../shared/features/settings/settingsCatalog';
import { settingsGroups } from '../../shared/features/settings/settingsSections';
const ui = useUiStore();
import { useSettingsPage } from '../../shared/features/settings/useSettingsPage';
const {
  settingsLeaveOpen,
  requestSettingsLeave,
  saveAndLeave,
  discardAndLeave,
  settingsForm,
  supportsDesktopPaths,
  settings,
  settingsGlobalSpeedLimitError,
  taskStartIntervalError,
  settingsFormChanged,
  settingsEmbeddingFormatError,
  resetSettingsDraft,
  saveSettings,
  restoreAllDefaults,
  availableSettingsSections,
} = useSettingsPage();

const page = computed(() => ui.mobileSettingsPath.at(-1) ?? 'overview');
const pageTitle = computed(() => settingsSections.find((section) => section.mobilePage === page.value)?.label ?? '设置');
const backLabel = computed(() => {
  if (ui.mobileSettingsPath.length > 1) return '返回上一级设置';
  if (ui.mobileSettingsReturnTab === 'parse') return '返回解析';
  if (ui.mobileSettingsReturnTab === 'library') return '返回内容库';
  return '返回我的';
});
const settingsContent = ref<HTMLElement>();
watch(page, () => settingsContent.value?.scrollTo({ top: 0 }), { flush: 'post' });
const goBack = () => requestSettingsLeave(() => {
  if (ui.mobileSettingsPath.length > 1) ui.mobileSettingsPath.pop();
  else emit('back');
});
useMobileBack('settings', () => {
  goBack();
  return true;
});
</script>
<template>
  <section class="mobile-page settings-grid">
    <SettingsLeaveDialog
      v-model="settingsLeaveOpen"
      :saving="settings.saving"
      :invalid="Boolean(settings.namingTemplateError || settings.proxySettingsError || settingsGlobalSpeedLimitError || taskStartIntervalError || settingsEmbeddingFormatError)"
      :error="
        settings.error || settings.proxySettingsError || settingsGlobalSpeedLimitError || taskStartIntervalError || settingsEmbeddingFormatError || settings.namingTemplateError
      "
      @save="saveAndLeave"
      @discard="discardAndLeave"
    />
    <section class="settings-panel">
      <div class="panel-heading settings-heading">
        <div class="settings-heading-copy">
          <button
            class="mobile-icon-button"
            type="button"
            :aria-label="backLabel"
            @click="goBack"
          >
            <UIcon name="i-tabler-arrow-left" />
          </button>
          <h2>{{ pageTitle }}</h2>
        </div>
        <div v-if="page === 'overview' || settingsFormChanged" class="settings-actions">
          <UDropdownMenu
            :items="page === 'overview' ? [
              {
                label: '恢复全部默认设置',
                icon: 'i-tabler-refresh',
                disabled: settings.loading || settings.saving,
                onSelect: restoreAllDefaults,
              },
            ] : [
              {
                label: '撤销未保存的修改',
                icon: 'i-tabler-arrow-back-up',
                disabled: settings.loading || settings.saving || !settingsFormChanged,
                onSelect: resetSettingsDraft,
              },
            ]"
          >
            <UiButton size="compact" variant="ghost" class="settings-more" aria-label="更多设置操作">
              <UIcon name="i-tabler-dots" class="size-5" />
            </UiButton>
          </UDropdownMenu>
          <UiButton
            v-if="settingsFormChanged"
            class="settings-save"
            :disabled="
              settings.loading ||
              settings.saving ||
              Boolean(settings.namingTemplateError) ||
              Boolean(settings.proxySettingsError) ||
              Boolean(settingsGlobalSpeedLimitError) ||
              Boolean(taskStartIntervalError) ||
              Boolean(settingsEmbeddingFormatError)
            "
            @click="saveSettings"
          >
            <UIcon
              v-if="settings.saving"
              name="i-tabler-loader-2"
              class="size-4 animate-spin"
              aria-hidden="true"
            />
            {{ settings.saving ? '保存中' : '保存' }}
          </UiButton>
        </div>
      </div>

      <UiInlineNotice v-if="settings.notice" :tone="settings.notice.tone">
        {{ settings.notice.message }}
      </UiInlineNotice>

      <div class="settings-workspace">
        <div ref="settingsContent" class="settings-content">
          <div v-if="page === 'overview'" class="settings-index">
            <section v-for="group in settingsGroups" :key="group" class="settings-index-group">
              <h3>{{ group }}</h3>
              <UButton
                v-for="section in availableSettingsSections.filter((item) => item.group === group)"
                :key="section.id"
                class="settings-index-row"
                variant="ghost"
                color="neutral"
                :ui="{ label: 'contents' }"
                @click="ui.mobileSettingsPath.push(section.mobilePage)"
              >
                <UIcon :name="section.icon" class="size-5 text-(--color-muted)" aria-hidden="true" />
                <span class="min-w-0 flex-1">{{ section.label }}</span>
                <UIcon name="i-tabler-chevron-right" class="size-4 text-(--color-muted)" aria-hidden="true" />
              </UButton>
            </section>
          </div>
          <SettingsDownloadSection
            v-else-if="page === 'saving' || page === 'tasks'"
            :form="settingsForm"
            :desktop-paths="supportsDesktopPaths"
            :group="page"
          />
          <SettingsMediaSection v-else-if="page === 'media'" :form="settingsForm" />
          <PresetManager v-else-if="page === 'presets'" v-model:kind="ui.settingsPresetKind" />
          <SettingsUpdateSection v-else-if="page === 'update'" />
          <div v-else-if="page === 'links'" class="settings-block">
            <SettingsCard title="剪贴板">
              <ClipboardPreference />
            </SettingsCard>
            <SettingsDownloadSection :form="settingsForm" :desktop-paths="supportsDesktopPaths" group="links" />
          </div>
          <SettingsMaintenanceSection v-else :form="settingsForm" :desktop-paths="supportsDesktopPaths" :group="page" />

          <p v-if="settings.error" class="settings-error">{{ settings.error }}</p>
        </div>
      </div>
    </section>
  </section>
</template>

<style src="../../shared/features/settings/SettingsForm.css"></style>
<style scoped src="./SettingsPage.css"></style>
