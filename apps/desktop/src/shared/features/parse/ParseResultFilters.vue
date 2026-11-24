<script setup lang="ts">
import { ref } from 'vue';
import UiDialog from '../../ui/Dialog.vue';
import UiButton from '../../ui/Button.vue';
import UiSelect from '../../ui/Select.vue';
const sortOptions = [
  { label: '原顺序', value: 'original' },
  { label: '按标题', value: 'title' },
  { label: '按时长', value: 'duration' },
];
const query = defineModel<string>('query', { required: true });
const sort = defineModel<string>('sort', { required: true });
const searching = defineModel<boolean>('searching', { default: false });
const props = defineProps<{
  disabled: boolean;
  grouped?: boolean;
  mobile?: boolean;
  listLabel?: string;
  applyRange: (text: string) => string | null;
}>();
const rangeOpen = ref(false);
const toggleSearch = () => {
  if (searching.value) query.value = '';
  searching.value = !searching.value;
};
const range = ref('');
const rangeError = ref<string | undefined>();
const select = () => {
  rangeError.value = props.applyRange(range.value) ?? undefined;
  if (!rangeError.value) rangeOpen.value = false;
};
</script>

<template>
  <div class="parse-result-filters" :class="{ 'is-mobile': mobile }">
    <div v-if="!mobile" class="result-list-heading">
      <strong>{{ listLabel || '分集' }}</strong>
    </div>
    <UInput
      v-if="!mobile || searching"
      v-model="query"
      class="result-search"
      type="search"
      aria-label="搜索解析结果"
      icon="i-tabler-search"
      :placeholder="mobile ? '搜索标题' : `搜索${listLabel || '分集'}标题`"
      :disabled="disabled"
      :autofocus="mobile"
      color="neutral"
      variant="outline"
      size="xs"
      :ui="{
        base: mobile ? 'result-filter-control text-(length:--font-13)' : 'min-h-[1.875rem] text-(length:--font-12)',
        leadingIcon: mobile ? 'size-4' : 'size-3.5',
      }"
    />
    <div v-if="mobile" class="result-filter-selection">
      <slot name="selection" />
      <div class="mobile-result-tools">
        <UiButton class="mobile-result-tool" variant="ghost" :aria-pressed="searching" :disabled="disabled" @click="toggleSearch">
          <UIcon name="i-tabler-search" class="size-4" aria-hidden="true" />
          {{ searching ? '关闭搜索' : '搜索' }}
        </UiButton>
        <UiButton v-if="!grouped" class="mobile-result-tool" variant="ghost" :disabled="disabled" @click="rangeOpen = true; rangeError = undefined">
          <UIcon name="i-tabler-adjustments-horizontal" class="size-4" aria-hidden="true" />
          排序与选择
        </UiButton>
      </div>
    </div>
    <USelect
      v-if="!mobile && !grouped"
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
      :ui="{
        base: 'min-h-[1.875rem] text-(length:--font-12)',
        content: 'z-50',
        item: 'text-(length:--font-12)',
      }"
    />
    <UiButton
      v-if="!mobile && !grouped"
      variant="secondary"
      size="compact"
      class="result-range"
      aria-label="范围选择"
      :disabled="disabled"
      @click="
        rangeOpen = true;
        rangeError = undefined;
      "
    >
      <UIcon name="i-tabler-list-numbers" class="range-icon" aria-hidden="true" />
      <span class="range-label">范围选择</span>
    </UiButton>
  </div>
  <UiDialog v-model="rangeOpen" :title="mobile ? '排序与选择' : '范围选择'" description="按当前搜索与排序后的列表序号添加选择。">
    <form class="result-options" @submit.prevent="select">
      <UiSelect v-if="mobile" v-model="sort" label="解析结果排序" :options="sortOptions" :disabled="disabled" />
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
      <UiButton variant="secondary" @click="rangeOpen = false">{{ mobile ? '完成' : '取消' }}</UiButton>
      <UiButton :disabled="disabled || (mobile && !range.trim())" @click="select">添加选择</UiButton>
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

  .result-search {
    flex: 1;
    width: auto;
  }
}

.parse-result-filters.is-mobile {
  flex-direction: column;
  align-items: stretch;
  gap: var(--space-8);
}

.is-mobile .result-search {
  width: 100%;
  margin: 0;
}

.is-mobile :deep(.result-filter-control),
.result-filter-selection {
  min-height: 2.75rem;
}

.result-filter-selection {
  display: flex;
  align-items: center;
  min-width: 0;
  justify-content: space-between;
  gap: var(--space-4);
}

.mobile-result-tools {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.mobile-result-tool {
  min-width: 0;
  min-height: 2.75rem;
  padding-inline: var(--space-4);
}

.result-options {
  display: grid;
  gap: var(--space-16);
}

@media (width <= 360px) {
  .mobile-result-tool {
    gap: var(--space-4);
  }
}

@media (orientation: landscape) and (height <= 480px) {
  .parse-result-filters.is-mobile {
    flex-direction: row;
    align-items: center;
  }

  .is-mobile .result-search {
    width: auto;
    flex: 1;
  }

  .is-mobile :deep(.result-filter-control),
  .mobile-result-tool,
  .result-filter-selection {
    min-height: 2.25rem;
  }

  .result-filter-selection {
    flex: 1;
  }
}
</style>
