<script setup lang="ts">
import { nextTick, ref, useId } from 'vue'

import { namingVariables } from '../stores/settings'
import UiIconButton from './IconButton.vue'

const model = defineModel<string>({ required: true })
const { error } = defineProps<{ error?: string }>()

const inputRef = ref<HTMLInputElement | null>(null)
const variableMenuOpen = ref(false)
const fieldId = useId()
const messageId = `${fieldId}-message`

const insertVariable = async (name: string) => {
  const token = `{${name}}`
  const input = inputRef.value
  const start = input?.selectionStart ?? model.value.length
  const end = input?.selectionEnd ?? start
  model.value = `${model.value.slice(0, start)}${token}${model.value.slice(end)}`

  await nextTick()
  const cursor = start + token.length
  inputRef.value?.focus()
  inputRef.value?.setSelectionRange(cursor, cursor)
}
</script>

<template>
  <div class="naming-template-field">
    <label :for="fieldId">命名模板</label>
    <div class="naming-template-control">
      <input
        :id="fieldId"
        ref="inputRef"
        v-model="model"
        class="ui-native-control input-control"
        type="text"
        :aria-invalid="Boolean(error) || undefined"
        :aria-describedby="error ? messageId : undefined"
      />
      <UPopover
        v-model:open="variableMenuOpen"
        :content="{ align: 'end', side: 'bottom', sideOffset: 6, collisionPadding: 12 }"
      >
        <UiIconButton icon="plus" label="插入魔法变量" />
        <template #content>
          <section class="naming-variable-menu" aria-label="魔法变量">
            <header>
              <strong>魔法变量</strong>
              <span class="naming-variable-hint">点击后插入到光标位置</span>
            </header>
            <div class="naming-variable-grid">
              <button
                v-for="variable in namingVariables"
                :key="variable.name"
                type="button"
                class="naming-variable-option"
                :data-variable="variable.name"
                @click="insertVariable(variable.name)"
              >
                <code>{{ '{' + variable.name + '}' }}</code>
                <span class="naming-variable-description">{{ variable.desc }}</span>
              </button>
            </div>
          </section>
        </template>
      </UPopover>
    </div>
    <small v-if="error" :id="messageId" class="error" aria-live="polite">{{ error }}</small>
  </div>
</template>

<style scoped>
.naming-template-field {
  min-width: 0;
  display: grid;
  gap: var(--space-6);
}

.naming-template-field > label {
  min-height: 18px;
  display: flex;
  align-items: center;
  color: var(--color-muted);
  font-size: var(--font-13);
  font-weight: 600;
  line-height: 18px;
}

.naming-template-control {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-6);
}

.naming-template-field > small {
  color: var(--color-text-muted);
  font-size: var(--font-11);
  font-weight: 500;
  line-height: 1.45;
}

.naming-template-field > small.error {
  color: var(--color-danger);
}

.naming-variable-menu {
  width: min(440px, calc(100vw - 32px));
  max-height: min(420px, calc(100dvh - 96px));
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: var(--space-8);
  padding: var(--space-10);
  overflow: hidden;
}

.naming-variable-menu header {
  display: grid;
  gap: 2px;
  padding: 0 var(--space-4);
}

.naming-variable-menu header strong {
  color: var(--color-text-strong);
  font-size: var(--font-13);
}

.naming-variable-hint {
  color: var(--color-muted);
  font-size: var(--font-11);
}

.naming-variable-grid {
  min-height: 0;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-4);
  overflow-y: auto;
}

.naming-variable-option {
  min-width: 0;
  display: grid;
  gap: 2px;
  padding: var(--space-8) var(--space-10);
  border: 0;
  border-radius: var(--radius-6);
  background: transparent;
  color: var(--color-text);
  text-align: left;
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease-out);
}

.naming-variable-option:hover,
.naming-variable-option:focus-visible {
  background: var(--color-hover-surface);
  outline: 0;
}

.naming-variable-option:focus-visible {
  box-shadow: inset 0 0 0 1px var(--color-focus-outline);
}

.naming-variable-option code {
  overflow: hidden;
  color: var(--color-accent-strong);
  font-size: var(--font-12);
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.naming-variable-description {
  overflow: hidden;
  color: var(--color-muted);
  font-size: var(--font-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (width <= 520px) {
  .naming-variable-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
