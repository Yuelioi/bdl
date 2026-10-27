<script setup lang="ts">
import MobileSheet from './MobileSheet.vue';
import UiButton from '../../shared/ui/Button.vue';
import ExternalLinkButton from '../../shared/ui/ExternalLinkButton.vue';
const batch = defineModel<string>('batchSize', { default: '50' });
defineProps<{ href?: string; hasMore: boolean; busy: boolean; parsing: boolean; stopping?: boolean }>();
const emit = defineEmits<{ load: []; all: []; downloadAll: []; stop: [] }>();
const open = defineModel<boolean>('open', { default: false });
const run = (action: 'load' | 'all' | 'downloadAll' | 'stop') => {
  open.value = false;
  if (action === 'load') emit('load');
  else if (action === 'all') emit('all');
  else if (action === 'stop') emit('stop');
  else emit('downloadAll');
};
</script>
<template>
  <MobileSheet v-model="open" title="内容操作" description="分批加载内容，或将整个集合加入下载。">
    <UFormField
      label="每次加载"
      orientation="horizontal"
      class="batch-setting"
      :ui="{ root: 'flex items-center justify-between gap-4', container: 'mt-0' }"
    >
      <USelect
        v-model="batch"
        :disabled="busy || parsing"
        :items="[
          { label: '20 项', value: '20' },
          { label: '50 项', value: '50' },
          { label: '100 项', value: '100' },
        ]"
        value-key="value"
        label-key="label"
        color="neutral"
        variant="outline"
        size="sm"
        :portal="true"
        :content="{ position: 'popper', align: 'end', sideOffset: 4, collisionPadding: 12 }"
      />
    </UFormField>
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
.batch-setting {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  font-size: var(--mobile-font-item-title);
}
</style>
