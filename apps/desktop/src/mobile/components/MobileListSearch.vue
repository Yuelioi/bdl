<script setup lang="ts">
const query = defineModel<string>({ required: true });
const open = defineModel<boolean>('open', { required: true });
defineProps<{ label: string; placeholder: string; disabled?: boolean }>();
</script>
<template>
  <UInput
    v-if="open"
    v-model="query"
    class="mobile-list-search"
    type="search"
    :aria-label="label"
    icon="i-tabler-search"
    :placeholder="placeholder"
    :disabled="disabled"
    color="neutral"
    variant="outline"
    size="sm"
    :ui="{ base: 'min-h-[2.125rem] text-(length:--font-13)', leadingIcon: 'size-4' }"
  >
    <template #trailing>
      <UButton
        variant="ghost"
        color="neutral"
        size="xs"
        icon="i-tabler-x"
        aria-label="关闭搜索"
        @click="
          query = '';
          open = false;
        "
      />
    </template>
  </UInput>
</template>
<style scoped>
.mobile-list-search {
  width: 100%;
  min-width: 0;
  flex-shrink: 0;
}
</style>
