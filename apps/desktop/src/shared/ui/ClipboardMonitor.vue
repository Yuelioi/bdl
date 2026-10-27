<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { useSettingsStore } from '../stores/settings';
import { useClipboardStore } from '../stores/clipboard';
import { useUiStore } from '../stores/ui';
import { extractBilibiliInputs } from '../utils/bilibiliLinks';
import { readClipboardText } from '../utils/clipboard';
import UiDialog from './Dialog.vue';
import UiButton from './Button.vue';
const settings = useSettingsStore();
const clipboard = useClipboardStore();
const ui = useUiStore();
const open = ref(false);
const links = ref<string[]>([]);
const seen = new Set<string>();
let generation = 0;
let reading = false;
let disposed = false;
async function check() {
  if (disposed || reading || !settings.saved.clipboard_monitor_enabled || document.visibilityState !== 'visible' || !document.hasFocus()) return;
  const request = generation;
  reading = true;
  try {
    const text = await readClipboardText();
    if (disposed || request !== generation || document.visibilityState !== 'visible' || !document.hasFocus()) return;
    const found = extractBilibiliInputs(text).filter((link) => !seen.has(link));
    if (!found.length) return;
    found.forEach((link) => seen.add(link));
    while (seen.size > 200) seen.delete(seen.values().next().value!);
    links.value = found;
    open.value = true;
  } catch {
    // Automatic detection is optional; only an explicit paste action reports read failures.
  } finally { reading = false; }
}
function add() {
  clipboard.additions = [...new Set([...clipboard.additions, ...links.value])];
  open.value = false;
  ui.setTab('parse');
}
watch(() => settings.saved.clipboard_monitor_enabled, () => {
  generation++;
  open.value = false;
  links.value = [];
  void check();
});
onMounted(() => {
  window.addEventListener('focus', check);
  document.addEventListener('visibilitychange', check);
  void check();
});
onBeforeUnmount(() => {
  disposed = true;
  generation++;
  window.removeEventListener('focus', check);
  document.removeEventListener('visibilitychange', check);
});
</script>
<template>
  <UiDialog v-model="open" title="发现剪贴板中的 Bilibili 链接">
    <p>识别到 {{ links.length }} 个链接，是否添加到解析输入框？</p>
    <p class="clipboard-preview">{{ links.join('\n') }}</p>
    <template #footer>
      <UiButton variant="secondary" @click="open = false">忽略</UiButton>
      <UiButton @click="add">添加到解析</UiButton>
    </template>
  </UiDialog>
</template>
<style scoped>
.clipboard-preview {
  max-height: 12rem;
  overflow: auto;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
  font-size: var(--font-12);

}
</style>
