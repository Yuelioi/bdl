<script setup lang="ts">
import { computed, ref } from 'vue'
import type { ParseRules } from '../../api/dto'
import UiSelect from '../../ui/Select.vue'

const model = defineModel<ParseRules>({ required: true })
const presets = [
  { label: '快速', value: 'fast', pages: 5 },
  { label: '标准', value: 'standard', pages: 3 },
  { label: '慢速', value: 'slow', pages: 1 },
]
const customSelected = ref(false)
const selectedPreset = computed({
  get: () => customSelected.value || model.value.interval_seconds !== 1 || model.value.rest_seconds !== 3 ? 'custom' : presets.find((preset) => preset.pages === model.value.pages_per_round)?.value ?? 'custom',
  set: (value: string) => {
    customSelected.value = value === 'custom'
    const preset = presets.find((item) => item.value === value)
    if (preset) model.value = { pages_per_round: preset.pages, interval_seconds: 1, rest_seconds: 3 }
  },
})
const seconds = (current: number) => [...new Set([1, 2, 3, 4, 5, 10, 20, 30, 60, current])].sort((a, b) => a - b).map((value) => ({ label: `${value} 秒`, value: String(value) }))
const batchOptions = computed(() => [...new Set([1, 2, 3, 5, 10, model.value.pages_per_round])].sort((a, b) => a - b).map((value) => ({ label: `${value * 20} 条`, value: String(value) })))
const fields = computed(() => [
  { key: 'pages_per_round' as const, label: '每批解析', options: batchOptions.value, help: '分批获取内容，避免短时间内请求过多。条数按每页约20条估算，实际随B站返回结果变化。' },
  { key: 'interval_seconds' as const, label: '批间等待', options: seconds(model.value.interval_seconds), help: '一批完成后稍等再继续，减少连续请求触发B站风控的机会；不会对每个视频单独等待。' },
  { key: 'rest_seconds' as const, label: '每100条休息', options: seconds(model.value.rest_seconds), help: '累计约100条后多休息一下，再继续下一段。它替代这一次批间等待，不叠加；最后解析完立即结束。遇到风控仍会停止解析。' },
])
const update = (key: keyof ParseRules, value: string) => {
  const rules = { ...model.value, [key]: Number(value) }
  if (key === 'interval_seconds') rules.rest_seconds = Math.max(rules.interval_seconds, rules.rest_seconds)
  if (key === 'rest_seconds') rules.interval_seconds = Math.min(rules.interval_seconds, rules.rest_seconds)
  model.value = rules
}
const estimatedWait = computed(() => {
  let batch = 0
  let total = 0
  for (let page = 1; page < 10; page++) {
    batch++
    if (page % 5 === 0) { total += model.value.rest_seconds; batch = 0 }
    else if (batch >= model.value.pages_per_round) { total += model.value.interval_seconds; batch = 0 }
  }
  return total
})
</script>

<template>
  <section class="grid gap-3 border-t border-(--color-border) pt-4">
    <div class="flex items-center gap-1">
      <h3 class="m-0 text-sm font-bold">解析节奏</h3>
      <UTooltip text="B站可能限制连续请求。分批解析并适当等待可以降低触发风控的机会，但不能保证完全避免；所有来源共用这个节奏。" :delay-duration="150" :content="{ side: 'top' }" :ui="{ content: 'max-w-72 h-auto py-2', text: 'whitespace-normal leading-5' }">
        <button type="button" aria-label="为什么解析需要等待" class="grid size-6 place-items-center text-(--color-muted)"><UIcon name="i-tabler-help-circle" class="size-4" /></button>
      </UTooltip>
    </div>
    <div class="settings-inline-grid">
      <UiSelect v-model="selectedPreset" label="解析预设" :options="[...presets, { label: '自定义', value: 'custom' }]" />
      <div v-for="field in fields" :key="field.key" class="relative min-w-0">
        <UiSelect :model-value="String(model[field.key])" :label="field.label" :options="field.options" @update:model-value="update(field.key, $event)" />
        <UTooltip :text="field.help" :delay-duration="150" :content="{ side: 'top' }" :ui="{ content: 'max-w-72 h-auto py-2', text: 'whitespace-normal leading-5' }">
          <button type="button" :aria-label="`${field.label}说明`" class="absolute top-0 right-0 grid size-5 place-items-center text-(--color-muted)"><UIcon name="i-tabler-help-circle" class="size-3.5" /></button>
        </UTooltip>
      </div>
    </div>
    <p class="m-0 text-xs text-(--color-muted)">解析约 200 条，额外等待约 {{ estimatedWait }} 秒，不含网络耗时。</p>
  </section>
</template>
