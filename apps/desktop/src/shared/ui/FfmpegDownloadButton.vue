<script setup lang="ts">
import { openExternalUrl } from '../api/tauri';
import { useUiStore } from '../stores/ui';
import UiButton from './Button.vue';

const openDownload = async () => {
  try {
    await openExternalUrl('https://apps.yuelili.com/software/ffmpeg');
  } catch (error) {
    useUiStore().pushToast(error instanceof Error ? error.message : String(error), 'danger');
  }
};
</script>

<template>
  <UiButton variant="secondary" size="compact" @click="openDownload">下载 FFmpeg</UiButton>
</template>
