<script setup lang="ts">
import UiDialog from '../../ui/Dialog.vue';
import UiButton from '../../ui/Button.vue';
const open = defineModel<boolean>({ default: false });
defineProps<{ saving: boolean; invalid: boolean; error?: string | null }>();
const emit = defineEmits<{ save: []; discard: [] }>();
</script>

<template>
  <UiDialog v-model="open" title="设置尚未保存" :dismissible="!saving" :show-close="!saving">
    <p>保存更改后再离开？</p>
    <p v-if="error" class="settings-error">{{ error }}</p>
    <template #footer>
      <UiButton variant="ghost" :disabled="saving" @click="open = false">继续编辑</UiButton>
      <UiButton variant="secondary" :disabled="saving" @click="emit('discard')">放弃更改</UiButton>
      <UiButton :disabled="saving || invalid" @click="emit('save')">{{ saving ? '保存中' : '保存并离开' }}</UiButton>
    </template>
  </UiDialog>
</template>
