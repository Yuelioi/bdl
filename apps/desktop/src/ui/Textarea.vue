<script setup lang="ts">
import UiFormField from './FormField.vue'

const model = defineModel<string>({ default: '' })
const { rows = 6, disabled = false, error, helper } = defineProps<{
  label: string
  placeholder?: string
  rows?: number
  disabled?: boolean
  error?: string
  helper?: string
}>()
</script>

<template>
  <UiFormField v-slot="{ fieldId, describedBy, invalid }" :label :error :helper>
    <textarea
      class="ui-native-control textarea-control"
      :id="fieldId"
      v-model="model"
      :rows
      :placeholder
      :disabled
      :aria-invalid="invalid || undefined"
      :aria-describedby="describedBy"
    />
  </UiFormField>
</template>
