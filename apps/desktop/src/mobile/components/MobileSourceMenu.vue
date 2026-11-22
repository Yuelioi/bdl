<script setup lang="ts">
import { ref } from 'vue';
import MobileSheet from './MobileSheet.vue';
import UiButton from '../../shared/ui/Button.vue';
const batch = defineModel<string>('batchSize', { default: '50' });
defineProps<{ hasMore: boolean; busy: boolean; parsing: boolean; stopping?: boolean; preload?: boolean }>();
const emit = defineEmits<{ load: []; all: []; stop: [] }>();
const open = ref(false);
const load = () => {
  open.value = false;
  emit('load');
};
const loadAll = () => {
  open.value = false;
  emit('all');
};
</script>

<template>
  <div v-if="hasMore || parsing" class="source-load-actions">
    <UiButton v-if="parsing" variant="secondary" size="compact" :disabled="stopping" @click="emit('stop')">
      <UIcon name="i-tabler-player-stop" class="size-4" aria-hidden="true" />
      {{ stopping ? '正在停止…' : '停止加载' }}
    </UiButton>
    <UiButton v-else-if="preload" variant="secondary" size="compact" aria-haspopup="dialog" :disabled="busy" @click="open = true">
      <UIcon name="i-tabler-list-details" class="size-4" aria-hidden="true" />
      加载选项
    </UiButton>
    <template v-else>
      <UiButton class="source-load-next" variant="secondary" size="compact" :disabled="busy" @click="emit('load')">
        {{ busy ? '加载中…' : '加载更多' }}
      </UiButton>
      <UiButton
        class="source-load-options"
        variant="secondary"
        size="compact"
        aria-label="加载选项"
        aria-haspopup="dialog"
        :disabled="busy"
        @click="open = true"
      >
        <UIcon name="i-tabler-chevron-down" class="size-4" aria-hidden="true" />
      </UiButton>
    </template>
  </div>
  <MobileSheet
    v-model="open"
    title="加载选项"
    :description="preload ? '翻页时会自动加载，也可以在这里提前加载多页。' : undefined"
  >
    <UFormField
      label="每次加载"
      orientation="horizontal"
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
    <UiButton variant="secondary" :disabled="busy || parsing || !hasMore" @click="load">
      <UIcon name="i-tabler-list-check" class="size-4" aria-hidden="true" />
      {{ preload ? '提前加载' : '加载' }} {{ batch }} 项
    </UiButton>
    <UiButton variant="secondary" :disabled="busy || parsing || !hasMore" @click="loadAll">
      <UIcon name="i-tabler-list-details" class="size-4" aria-hidden="true" />
      {{ preload ? '提前加载全部视频' : '加载全部内容' }}
    </UiButton>
  </MobileSheet>
</template>

<style scoped>
.source-load-actions {
  display: flex;
  flex-shrink: 0;
}

.source-load-actions > .ui-button {
  min-height: 2.75rem;
}

.source-load-actions .source-load-next {
  border-start-end-radius: 0;
  border-end-end-radius: 0;
}

.source-load-actions .source-load-options {
  min-width: 2.5rem;
  border-inline-start: 0;
  border-start-start-radius: 0;
  border-end-start-radius: 0;
}
</style>
