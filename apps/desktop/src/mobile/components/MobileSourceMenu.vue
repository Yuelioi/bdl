<script setup lang="ts">
import { ref } from 'vue';
import MobileSheet from './MobileSheet.vue';
import UiButton from '../../shared/ui/Button.vue';
defineProps<{ hasMore: boolean; busy: boolean; parsing: boolean; stopping?: boolean }>();
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
    <UiButton v-else variant="secondary" size="compact" aria-haspopup="dialog" :disabled="busy" @click="open = true">
      <UIcon name="i-tabler-list-details" class="size-4" aria-hidden="true" />
      加载选项
    </UiButton>
  </div>
  <MobileSheet
    v-model="open"
    title="加载选项"
    description="翻页时会自动加载，也可以在这里提前加载多页。"
  >
    <UiButton variant="secondary" :disabled="busy || parsing || !hasMore" @click="load">
      <UIcon name="i-tabler-list-check" class="size-4" aria-hidden="true" />
      提前加载一批
    </UiButton>
    <UiButton variant="secondary" :disabled="busy || parsing || !hasMore" @click="loadAll">
      <UIcon name="i-tabler-list-details" class="size-4" aria-hidden="true" />
      提前加载全部视频
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

</style>
