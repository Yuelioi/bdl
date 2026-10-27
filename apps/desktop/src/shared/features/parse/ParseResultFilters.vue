<script setup lang="ts">
import { ref } from 'vue';
import UiDialog from '../../ui/Dialog.vue';
import UiButton from '../../ui/Button.vue';
const sortOptions = [
  { label: '原顺序', value: 'original' },
  { label: '按标题', value: 'title' },
  { label: '按时长', value: 'duration' },
];
const query = defineModel<string>('query', { required: true });
const sort = defineModel<string>('sort', { required: true });
const props = defineProps<{
  disabled: boolean;
  grouped?: boolean;
  loaded?: number;
  applyRange: (text: string) => string | null;
}>();
const rangeOpen = ref(false);
const range = ref('');
const rangeError = ref<string | undefined>();
const select = () => {
  rangeError.value = props.applyRange(range.value) ?? undefined;
  if (!rangeError.value) rangeOpen.value = false;
};
</script>

<template>
  <div class="parse-result-filters">
    <div class="result-list-heading">
      <strong>分集</strong><span v-if="loaded != null">已加载 {{ loaded }} 项</span>
    </div>
    <UInput
      v-model="query"
      class="result-search"
      type="search"
      aria-label="搜索解析结果"
      icon="i-tabler-search"
      placeholder="搜索分集标题"
      :disabled="disabled"
      color="neutral"
      variant="outline"
      size="xs"
      :ui="{ base: 'min-h-[1.875rem] text-(length:--font-12)', leadingIcon: 'size-3.5' }"
    />
    <USelect
      v-if="!grouped"
      v-model="sort"
      :items="sortOptions"
      value-key="value"
      label-key="label"
      aria-label="解析结果排序"
      :disabled="disabled"
      color="neutral"
      variant="outline"
      size="xs"
      :portal="true"
      :content="{ position: 'popper', align: 'end', sideOffset: 4, collisionPadding: 12 }"
      :ui="{ base: 'min-h-[1.875rem] text-(length:--font-12)', content: 'z-50', item: 'text-(length:--font-12)' }"
    />
    <UiButton
      v-if="!grouped"
      variant="secondary"
      size="compact"
      class="result-range"
      aria-label="范围选择"
      :disabled="disabled"
      @click="
        rangeOpen = true;
        rangeError = undefined;
      "
      ><UIcon name="i-tabler-list-numbers" class="range-icon" /><span class="range-label">范围选择</span></UiButton
    >
  </div>
  <UiDialog v-model="rangeOpen" title="范围选择" description="按当前搜索与排序后的列表序号添加选择。">
    <form @submit.prevent="select">
      <UFormField label="选择序号" name="part-range" :error="rangeError">
        <UInput
          v-model="range"
          class="w-full"
          placeholder="1-5,7,9-12"
          :disabled="disabled"
          color="neutral"
          size="sm"
        />
      </UFormField>
    </form>
    <template #footer>
      <UiButton variant="secondary" @click="rangeOpen = false">取消</UiButton>
      <UiButton :disabled="disabled" @click="select">添加选择</UiButton>
    </template>
  </UiDialog>
</template>

<style scoped>
.parse-result-filters {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  gap: 0.5rem;
  min-width: 0;
}

.result-list-heading {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  white-space: nowrap;
}

.result-list-heading strong {
  font-size: var(--font-14);
  font-weight: 600;
}

.result-list-heading span {
  color: var(--color-muted);
  font-size: var(--font-12);
}

.result-search {
  width: min(15rem, 40%);
  margin-left: auto;
  min-width: 0;
}

.range-icon {
  display: none;
}

@media (width <= 600px) {
  .range-icon {
    display: block;
    width: 1rem;
    height: 1rem;
  }

  .range-label {
    display: none;
  }

  .result-range {
    padding-inline: 0.375rem;
  }

  .result-list-heading span {
    display: none;
  }

  .result-search {
    flex: 1;
    width: auto;
  }
}
</style>
