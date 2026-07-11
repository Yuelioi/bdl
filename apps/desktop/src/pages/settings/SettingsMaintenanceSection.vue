<script setup lang="ts">
import UiButton from '../../ui/Button.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import { logLevelOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form } = defineProps<{ form: SettingsForm }>();
const { settings, settingsProxyUrl, settingsLogLevel, settingsDataDir } = form;
</script>

<template>
  <section class="settings-block">
    <UiTextField v-model="settingsProxyUrl" label="代理地址" placeholder="例如 http://127.0.0.1:7890，留空为直连" />
    <UiSelect v-model="settingsLogLevel" label="任务日志级别" :options="logLevelOptions" />
    <div class="directory-row">
      <UiTextField v-model="settingsDataDir" label="数据目录" placeholder="留空时使用当前工作目录下的 .bdl" />
      <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseDataDir"
        >选择</UiButton
      >
    </div>
    <div v-if="settings.draft.data_dir" class="settings-actions">
      <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="settings.clearDataDir"
        >使用默认数据目录</UiButton
      >
    </div>
    <p class="settings-note">数据目录影响任务库、账户摘要和维护文件，修改后下次启动生效。</p>
    <div class="settings-actions">
      <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.cleanupCache"
        >清理缓存</UiButton
      >
      <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.cleanupTemp"
        >清理临时文件</UiButton
      >
      <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.exportDiagnostics"
        >导出诊断</UiButton
      >
    </div>
  </section>
</template>
