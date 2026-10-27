<script setup lang="ts">
defineProps<{ title?: string }>();
</script>
<template>
  <section class="settings-card" :aria-label="title">
    <div v-if="title" class="settings-card-heading">
      <h3 class="settings-card-title">{{ title }}</h3>
      <slot name="title-actions" />
    </div>
    <slot />
  </section>
</template>
<style scoped>
.settings-card {
  min-width: 0;
  display: grid;
  align-content: start;
  gap: 1rem;
  border: 0;
  border-bottom: 1px solid var(--color-border);
  border-radius: 0;
  background: transparent;
  padding: 0 0 1rem;
  box-shadow: none;
}

.settings-card:last-child {
  border-bottom: 0;
}

.settings-card-title {
  margin: 0;
  color: var(--color-text-strong);
  font-size: var(--font-14);
  font-weight: 600;
  line-height: 1.5;
}

.settings-card-heading {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}
</style>
