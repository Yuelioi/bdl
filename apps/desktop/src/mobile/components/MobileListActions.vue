<script setup lang="ts">
const searchOpen = defineModel<boolean>('searchOpen', { default: false });
withDefaults(defineProps<{ searchLabel: string; showSearch?: boolean }>(), { showSearch: true });
</script>

<template>
  <UButton
    v-if="showSearch"
    variant="ghost"
    color="neutral"
    :ui="{ label: 'contents' }"
    type="button"
    class="mobile-icon-button list-search-button"
    :aria-label="searchLabel"
    :aria-pressed="searchOpen"
    @click="searchOpen = !searchOpen"
  >
    <UIcon name="i-tabler-search" aria-hidden="true" />
  </UButton>
</template>

<style scoped>
.list-search-button {
  width: 2.75rem;
  height: 2.75rem;
  flex-basis: 2.75rem;
}
</style>
