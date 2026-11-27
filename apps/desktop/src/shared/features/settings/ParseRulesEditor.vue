<script setup lang="ts">
import type { ParseRules } from '../../api/dto';
import UiSelect from '../../ui/Select.vue';
import SettingsCard from './SettingsCard.vue';

const model = defineModel<ParseRules>({ required: true });
const options = [1, 2, 5, 10, 20, 30, 120].map(seconds => ({
  label: `${seconds} 秒`,
  value: String(seconds),
}));
</script>

<template>
  <SettingsCard title="解析节奏">
    <div class="settings-inline-grid">
      <UiSelect
        :model-value="String(model.interval_seconds)"
        label="翻页间隔"
        :options="options"
        @update:model-value="model = { interval_seconds: Number($event) }"
      >
        <template #value>{{ model.interval_seconds }} 秒</template>
      </UiSelect>
    </div>
  </SettingsCard>
</template>
