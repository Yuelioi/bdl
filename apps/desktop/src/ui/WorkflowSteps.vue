<script setup lang="ts">
export interface WorkflowStep {
  value: string
  label: string
  description?: string
  disabled?: boolean
  complete?: boolean
}

const model = defineModel<string>({ required: true })
defineProps<{ steps: WorkflowStep[] }>()
</script>

<template>
  <nav class="flex min-w-0 items-stretch border-b border-(--color-border)" aria-label="操作阶段">
    <button
      v-for="(step, index) in steps"
      :key="step.value"
      type="button"
      class="group relative flex min-w-0 items-center gap-2.5 px-4 pt-1 pb-3 text-left text-(--color-muted) disabled:cursor-default disabled:opacity-55"
      :class="model === step.value ? 'text-(--color-accent-strong)' : 'hover:text-(--color-text)'"
      :disabled="step.disabled"
      :aria-current="model === step.value ? 'step' : undefined"
      @click="model = step.value"
    >
      <span
        class="grid size-6 shrink-0 place-items-center rounded-full border text-xs font-bold tabular-nums"
        :class="
          model === step.value
            ? 'border-(--color-accent) bg-(--color-accent) text-(--color-on-accent)'
            : step.complete
              ? 'border-(--color-accent) bg-(--color-accent-soft) text-(--color-accent-strong)'
              : 'border-(--color-border) bg-(--color-surface)'
        "
      >
        <UIcon v-if="step.complete && model !== step.value" name="i-tabler-check" class="size-3.5" aria-hidden="true" />
        <span v-else>{{ index + 1 }}</span>
      </span>
      <span class="grid min-w-0 gap-px">
        <strong class="truncate text-[13px] font-bold text-current">{{ step.label }}</strong>
        <small v-if="step.description" class="truncate text-[11px] text-(--color-muted)">{{ step.description }}</small>
      </span>
      <span
        v-if="model === step.value"
        class="absolute right-3 bottom-[-1px] left-3 h-0.5 rounded-full bg-(--color-accent)"
        aria-hidden="true"
      ></span>
    </button>
  </nav>
</template>
