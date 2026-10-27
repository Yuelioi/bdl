<script setup lang="ts">
import { useId } from 'vue';

const {
  label,
  error,
  helper,
  hideLabel = false,
} = defineProps<{
  label: string;
  error?: string;
  helper?: string;
  hideLabel?: boolean;
}>();

const fieldId = useId();
const messageId = `${fieldId}-message`;
</script>

<template>
  <UFormField
    class="ui-form-field"
    :class="{ 'ui-form-field-label-hidden': hideLabel }"
    :label
    :error
    :help="helper"
    size="sm"
    :ui="{
      root: 'min-w-0',
      labelWrapper: hideLabel ? 'sr-only' : 'field-label min-h-[1.125rem]',
      label: 'font-medium text-(--color-muted) text-(length:--font-13) leading-[1.125rem]',
      container: hideLabel ? 'field-control mt-0' : 'field-control mt-1.5',
      help: 'text-(length:--font-12)',
      error: 'text-(length:--font-12)',
    }"
  >
    <template v-if="$slots.hint" #hint><slot name="hint" /></template>
    <slot
      :field-id="fieldId"
      :message-id="messageId"
      :described-by="error || helper ? messageId : undefined"
      :invalid="Boolean(error)"
    />
    <template v-if="error" #error
      ><span :id="messageId" aria-live="polite">{{ error }}</span></template
    >
    <template v-if="helper" #help
      ><span :id="messageId">{{ helper }}</span></template
    >
  </UFormField>
</template>
