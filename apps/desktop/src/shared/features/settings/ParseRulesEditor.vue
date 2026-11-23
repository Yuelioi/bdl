<script setup lang="ts">
import { computed, ref } from 'vue';
import type { ParseRules } from '../../api/dto';
import UiSelect from '../../ui/Select.vue';
import SettingsCard from './SettingsCard.vue';

const model = defineModel<ParseRules>({ required: true });
const presets = [
  { label: '快速', value: 'fast', rules: { pages_per_round: 4, interval_seconds: 1, rest_seconds: 3 } },
  { label: '标准', value: 'standard', rules: { pages_per_round: 3, interval_seconds: 1, rest_seconds: 3 } },
  { label: '慢速', value: 'slow', rules: { pages_per_round: 1, interval_seconds: 3, rest_seconds: 8 } },
];
const customSelected = ref(false);
const selectedPreset = computed({
  get: () =>
    customSelected.value
      ? 'custom'
      : (presets.find(({ rules }) =>
          rules.pages_per_round === model.value.pages_per_round &&
          rules.interval_seconds === model.value.interval_seconds &&
          rules.rest_seconds === model.value.rest_seconds,
        )?.value ?? 'custom'),
  set: (value: string) => {
    customSelected.value = value === 'custom';
    const preset = presets.find((item) => item.value === value);
    if (preset) model.value = { ...preset.rules };
  },
});
const seconds = (current: number) =>
  [...new Set([1, 2, 3, 4, 5, 8, 10, 20, 30, 60, current])]
    .sort((a, b) => a - b)
    .map((value) => ({ label: `${value} 秒`, value: String(value) }));
const batchOptions = computed(() =>
  [...new Set([1, 2, 3, 4, 5, 10, model.value.pages_per_round])]
    .sort((a, b) => a - b)
    .map((value) => ({ label: `${value * 20} 条`, value: String(value) })),
);
const fields = computed(() => [
  {
    key: 'pages_per_round' as const,
    label: '每批解析',
    options: batchOptions.value,
  },
  {
    key: 'interval_seconds' as const,
    label: '批间等待',
    options: seconds(model.value.interval_seconds),
  },
  {
    key: 'rest_seconds' as const,
    label: '每100条休息',
    options: seconds(model.value.rest_seconds),
  },
]);
const update = (key: keyof ParseRules, value: string) => {
  const rules = { ...model.value, [key]: Number(value) };
  if (key === 'interval_seconds') rules.rest_seconds = Math.max(rules.interval_seconds, rules.rest_seconds);
  if (key === 'rest_seconds') rules.interval_seconds = Math.min(rules.interval_seconds, rules.rest_seconds);
  model.value = rules;
};
const estimatedWait = computed(() => {
  let batch = 0;
  let total = 0;
  for (let page = 1; page < 10; page++) {
    batch++;
    if (page % 5 === 0) {
      total += model.value.rest_seconds;
      batch = 0;
    } else if (batch >= model.value.pages_per_round) {
      total += model.value.interval_seconds;
      batch = 0;
    }
  }
  return total;
});
</script>

<template>
  <SettingsCard title="解析节奏">
    <template #title-actions>
      <UPopover :content="{ side: 'bottom', align: 'start', sideOffset: 4, collisionPadding: 12 }">
        <button
          type="button"
          aria-label="解析节奏说明"
          class="grid size-6 place-items-center text-(--color-muted)"
        >
          <UIcon name="i-tabler-help-circle" class="size-4" />
        </button>
        <template #content>
          <section class="max-w-80 p-3" aria-label="解析节奏参数">
            <dl class="m-0 grid gap-2 text-xs">
              <div>
                <dt class="font-medium">解析预设</dt>
                <dd class="m-0 text-(--color-muted)">快速套用下方三项参数。</dd>
              </div>
              <div>
                <dt class="font-medium">每批解析</dt>
                <dd class="m-0 text-(--color-muted)">连续获取多少条再等待，条数为估算值。</dd>
              </div>
              <div>
                <dt class="font-medium">批间等待</dt>
                <dd class="m-0 text-(--color-muted)">只在两批之间暂停，不逐条等待。</dd>
              </div>
              <div>
                <dt class="font-medium">每100条休息</dt>
                <dd class="m-0 text-(--color-muted)">替代该次批间等待，不叠加；全部完成后不再等。</dd>
              </div>
            </dl>
          </section>
        </template>
      </UPopover>
    </template>
    <div class="settings-inline-grid">
      <UiSelect
        v-model="selectedPreset"
        label="解析预设"
        :options="[...presets, { label: '自定义', value: 'custom' }]"
      />
      <UiSelect
        v-for="field in fields"
        :key="field.key"
        :model-value="String(model[field.key])"
        :label="field.label"
        :options="field.options"
        @update:model-value="update(field.key, $event)"
      />
    </div>
    <p class="m-0 text-xs text-(--color-muted)">解析约 200 条，额外等待约 {{ estimatedWait }} 秒，不含网络耗时。</p>
  </SettingsCard>
</template>
