<script setup lang="ts">
import { useId } from 'vue'

const { label, error, helper } = defineProps<{
  label: string
  error?: string
  helper?: string
}>()

const fieldId = useId()
const messageId = `${fieldId}-message`
</script>

<template>
  <label class="ui-form-field" :class="{ 'has-message': error || helper }" :for="fieldId">
    <span class="ui-form-field-label">{{ label }}</span>
    <slot
      :field-id="fieldId"
      :message-id="messageId"
      :described-by="error || helper ? messageId : undefined"
      :invalid="Boolean(error)"
    />
    <small
      v-if="error || helper"
      :id="messageId"
      :class="{ error: Boolean(error) }"
      :aria-live="error ? 'polite' : undefined"
    >
      {{ error || helper }}
    </small>
  </label>
</template>

<style scoped>
.ui-form-field {
  min-width: 0;
  display: grid;
  grid-template-rows: 18px auto;
  gap: var(--space-6);
  color: var(--color-muted);
  font-size: var(--font-13);
  font-weight: 600;
}

.ui-form-field.has-message {
  grid-template-rows: 18px auto auto;
}

.ui-form-field-label {
  min-height: 18px;
  display: flex;
  align-items: center;
  line-height: 18px;
}

small {
  color: var(--color-text-muted);
  font-size: var(--font-11);
  font-weight: 500;
  line-height: 1.45;
}

small.error {
  color: var(--color-danger);
}
</style>
