<script setup lang="ts">
const model = defineModel<boolean>({ default: false })
defineProps<{
  title: string
}>()
</script>

<template>
  <Teleport to="body">
    <div v-if="model" class="drawer-layer" role="presentation" @click.self="model = false">
      <aside class="drawer-panel" aria-modal="true" role="dialog" :aria-label="title">
        <header>
          <h2>{{ title }}</h2>
          <button type="button" aria-label="关闭" @click="model = false">x</button>
        </header>
        <div class="drawer-body">
          <slot />
        </div>
      </aside>
    </div>
  </Teleport>
</template>

<style scoped>
.drawer-layer {
  position: fixed;
  inset: 0;
  z-index: 30;
  display: flex;
  justify-content: flex-end;
  background: rgb(23 33 29 / 28%);
}

.drawer-panel {
  width: 380px;
  max-width: 100%;
  height: 100%;
  display: grid;
  grid-template-rows: 64px 1fr;
  border-left: 1px solid var(--color-border);
  background: var(--color-surface);
  box-shadow: -16px 0 40px rgb(23 33 29 / 12%);
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-12);
  padding: 0 var(--space-16);
  border-bottom: 1px solid var(--color-border);
}

h2 {
  margin: 0;
  font-size: var(--font-18);
}

button {
  width: var(--height-button);
  height: var(--height-button);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-muted);
}

.drawer-body {
  display: grid;
  align-content: start;
  gap: var(--space-16);
  padding: var(--space-16);
  overflow: auto;
}
</style>
