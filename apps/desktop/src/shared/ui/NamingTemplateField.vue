<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import UiFormField from './FormField.vue';

import { namingVariables } from '../stores/settings';
import UiIconButton from './IconButton.vue';

const model = defineModel<string>({ required: true });
const {
  error,
  label = '命名模板',
  variables = namingVariables,
  multiline = false,
  quickVariables = true,
  disabled = false,
  helper,
} = defineProps<{
  error?: string;
  label?: string;
  variables?: readonly { name: string; desc: string }[];
  multiline?: boolean;
  quickVariables?: boolean;
  disabled?: boolean;
  helper?: string;
}>();

const singleInput = ref<{ inputRef: HTMLInputElement } | null>(null);
const multipleInput = ref<{ textareaRef: HTMLTextAreaElement } | null>(null);
const inputRef = computed(() => (multiline ? multipleInput.value?.textareaRef : singleInput.value?.inputRef));
const variableMenuOpen = ref(false);

const insertVariable = async (name: string) => {
  if (disabled) return;
  const token = `{${name}}`;
  const input = inputRef.value;
  const start = input?.selectionStart ?? model.value.length;
  const end = input?.selectionEnd ?? start;
  model.value = `${model.value.slice(0, start)}${token}${model.value.slice(end)}`;

  await nextTick();
  const cursor = start + token.length;
  inputRef.value?.focus();
  inputRef.value?.setSelectionRange(cursor, cursor);
};
</script>

<template>
  <UiFormField v-slot="{ fieldId, describedBy, invalid }" class="naming-template-field" :label :error :helper>
    <div class="naming-template-control">
      <UTextarea
        v-if="multiline"
        :id="fieldId"
        ref="multipleInput"
        v-model="model"
        :rows="4"
        class="w-full"
        color="neutral"
        variant="outline"
        size="sm"
        :ui="{ base: 'min-h-28 resize-y text-(length:--font-13)' }"
        :disabled="disabled"
        :aria-invalid="invalid || undefined"
        :aria-describedby="describedBy"
      />
      <UInput
        v-else
        :id="fieldId"
        ref="singleInput"
        v-model="model"
        class="w-full"
        color="neutral"
        variant="outline"
        size="sm"
        :ui="{ base: 'h-(--height-input) text-(length:--font-13)' }"
        type="text"
        :disabled="disabled"
        :aria-invalid="invalid || undefined"
        :aria-describedby="describedBy"
      />
      <UPopover
        v-model:open="variableMenuOpen"
        :content="{ align: 'end', side: 'bottom', sideOffset: 6, collisionPadding: 12 }"
      >
        <UiIconButton
          icon="plus"
          :label="label === '命名模板' ? '插入魔法变量' : label + '：插入变量'"
          :disabled="disabled"
        />
        <template #content>
          <section class="naming-variable-menu" aria-label="魔法变量">
            <header>
              <strong>魔法变量</strong>
              <span class="naming-variable-hint">点击后插入到光标位置</span>
            </header>
            <div class="naming-variable-grid">
              <UButton
                variant="ghost"
                color="neutral"
                :ui="{ label: 'contents' }"
                v-for="variable in variables"
                :key="variable.name"
                type="button"
                class="naming-variable-option"
                :data-variable="variable.name"
                @click="insertVariable(variable.name)"
              >
                <code>{{ '{' + variable.name + '}' }}</code>
                <span class="naming-variable-description">{{ variable.desc }}</span>
              </UButton>
            </div>
          </section>
        </template>
      </UPopover>
    </div>
    <div
      v-if="
        quickVariables && variables.some((variable) => ['title', 'part_index', 'part_title'].includes(variable.name))
      "
      class="naming-quick-variables"
    >
      <UButton
        v-for="variable in variables.filter((variable) =>
          ['title', 'part_index', 'part_title'].includes(variable.name),
        )"
        :key="variable.name"
        variant="soft"
        color="neutral"
        size="xs"
        :disabled="disabled"
        :aria-label="`插入 ${variable.desc}`"
        @click="insertVariable(variable.name)"
        >{{ variable.name }}</UButton
      >
    </div>
  </UiFormField>
</template>

<style scoped>
.naming-quick-variables {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  margin-top: 0.5rem;
}

.naming-quick-variables button {
  font-size: var(--font-12);
  font-weight: 400;
  padding: 0.25rem 0.5rem;
}

.naming-template-control {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-6);
}

.naming-variable-menu {
  width: min(27.5rem, calc(100vw - 2rem));
  max-height: min(26.25rem, calc(100dvh - 6rem));
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: var(--space-8);
  padding: var(--space-10);
  overflow: hidden;
}

.naming-variable-menu header {
  display: grid;
  gap: 0.125rem;
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
  gap: 0.125rem;
  padding: var(--space-8) var(--space-10);
  color: var(--color-text);
  text-align: left;
  cursor: pointer;
}

.naming-variable-option code {
  overflow: hidden;
  color: var(--color-accent-strong);
  font-size: var(--font-12);
  font-weight: 700;
  overflow-wrap: anywhere;
  white-space: normal;
}

.naming-variable-description {
  overflow: hidden;
  color: var(--color-muted);
  font-size: var(--font-11);
  overflow-wrap: anywhere;
  white-space: normal;
}

@media (width <= 520px) {
  .naming-variable-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
