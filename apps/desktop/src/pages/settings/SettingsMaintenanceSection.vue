<script setup lang="ts">
import UiButton from '../../ui/Button.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import { logLevelOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form, desktopPaths = true } = defineProps<{ form: SettingsForm; desktopPaths?: boolean }>();
const { settings, settingsProxyUrl, settingsLogLevel, settingsDataDir } = form;
</script>

<template>
  <section class="settings-block">
    <UiTextField v-model="settingsProxyUrl" label="代理地址" placeholder="例如 http://127.0.0.1:7890，留空为直连" />
    <UiSelect v-model="settingsLogLevel" label="任务日志级别" :options="logLevelOptions" />
    <div v-if="desktopPaths" class="directory-row">
      <UiTextField v-model="settingsDataDir" label="数据目录" placeholder="留空时使用当前工作目录下的 .bdl" />
      <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseDataDir"
        >选择</UiButton
      >
    </div>
    <div v-if="desktopPaths && settings.draft.data_dir" class="settings-actions">
      <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="settings.clearDataDir"
        >使用默认数据目录</UiButton
      >
    </div>
    <p v-if="desktopPaths" class="settings-note">
      数据目录影响任务库、账户摘要和维护文件，修改后下次启动生效。
    </p>
    <div class="maintenance-actions">
      <UiButton class="maintenance-action" variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.cleanupCache"
        ><UIcon name="i-tabler-database-off" class="size-4" aria-hidden="true" />清理缓存</UiButton
      >
      <UiButton class="maintenance-action" variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.cleanupTemp"
        ><UIcon name="i-tabler-broom" class="size-4" aria-hidden="true" />清理临时文件</UiButton
      >
      <UiButton class="maintenance-action" variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.exportDiagnostics"
        ><UIcon name="i-tabler-file-export" class="size-4" aria-hidden="true" />导出诊断</UiButton
      >
    </div>
  </section>
</template>

<style scoped>
.maintenance-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(148px, 1fr));
  gap: var(--space-8);
}

.maintenance-action {
  width: 100%;
}
</style>
