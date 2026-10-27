<script setup lang="ts">
import UiTabs from '../../shared/ui/Tabs.vue';
const model = defineModel<string>({ required: true });
defineProps<{ tabs: Array<{ value: string; label: string; count?: number }>; stackCounts?: boolean }>();
</script>
<template><UiTabs v-model="model" :tabs="tabs" stretch /></template>
