<script setup lang="ts">
import { ref } from 'vue';
import MobileSheet from './MobileSheet.vue';
import UiButton from '../../ui/Button.vue';
import ExternalLinkButton from '../../ui/ExternalLinkButton.vue';
const batch = defineModel<string>('batchSize', { default: '50' });
defineProps<{ href?: string; hasMore: boolean; busy: boolean; parsing: boolean; stopping?: boolean }>();
const emit = defineEmits<{ load: []; all: []; downloadAll: []; stop: [] }>();
const open = ref(false);
const run = (action: 'load' | 'all' | 'downloadAll' | 'stop') => {
  open.value = false;
  if (action === 'load') emit('load');
  else if (action === 'all') emit('all');
  else if (action === 'stop') emit('stop');
  else emit('downloadAll');
};
</script>
<template>
  <button type="button" class="source-more" aria-label="内容操作" @click="open = true">
    <UIcon name="i-tabler-dots" />
  </button>
  <MobileSheet v-model="open" title="内容操作" description="分批加载内容，或将整个集合加入下载。">
    <label class="batch-setting"
      ><span>每次加载</span
      ><select v-model="batch" :disabled="busy || parsing">
        <option value="20">20 项</option>
        <option value="50">50 项</option>
        <option value="100">100 项</option>
      </select></label
    >
    <UiButton v-if="parsing" variant="secondary" :disabled="stopping" @click="run('stop')">{{
      stopping ? '正在停止…' : '停止加载'
    }}</UiButton>
    <template v-else>
      <UiButton v-if="hasMore" variant="secondary" :disabled="busy" @click="run('load')"
        >再加载 {{ batch }} 项</UiButton
      >
      <UiButton v-if="hasMore" variant="secondary" :disabled="busy" @click="run('all')">加载全部内容</UiButton>
      <UiButton :disabled="busy" @click="run('downloadAll')">下载整个集合</UiButton>
    </template>
    <ExternalLinkButton v-if="href" :href="href" label="在 Bilibili 打开来源">在 Bilibili 打开来源</ExternalLinkButton>
  </MobileSheet>
</template>
<style scoped>
.source-more {
  width: 44px;
  height: 44px;
  flex: 0 0 44px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 50%;
  background: transparent;
  color: var(--color-text);
}

.source-more :deep(svg) {
  width: 22px;
  height: 22px;
  flex-shrink: 0;
}

.source-more:active {
  background: var(--color-panel);
}

.batch-setting {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  font-size: var(--mobile-font-item-title);
}

select {
  min-height: 44px;
  min-width: 100px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  background: var(--color-surface);
  color: var(--color-text);
  padding: 0 12px;
}
</style>
