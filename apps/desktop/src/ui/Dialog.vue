<script setup lang="ts">
const model = defineModel<boolean>({ default: false })
defineProps<{
  title: string
}>()
</script>

<template>
  <Teleport to="body">
    <div v-if="model" class="dialog-layer" role="presentation" @click.self="model = false">
      <section class="dialog-panel" role="dialog" aria-modal="true" :aria-label="title">
        <header class="dialog-header">
          <h2>{{ title }}</h2>
          <button type="button" aria-label="关闭" @click="model = false">x</button>
        </header>
        <div class="dialog-body">
          <slot />
        </div>
        <footer v-if="$slots.footer" class="dialog-footer">
          <slot name="footer" />
        </footer>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-layer {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: grid;
  place-items: center;
  padding: var(--space-24);
  background: rgb(23 33 29 / 36%);
}

.dialog-panel {
  width: min(520px, 100%);
  display: grid;
  gap: var(--space-16);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-surface);
  box-shadow: 0 20px 48px rgb(23 33 29 / 18%);
}

.dialog-header,
.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-12);
  padding: var(--space-16);
}

.dialog-header {
  border-bottom: 1px solid var(--color-border);
}

.dialog-header h2 {
  margin: 0;
  font-size: var(--font-18);
}

.dialog-header button {
  width: var(--height-button);
  height: var(--height-button);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-muted);
}

.dialog-body {
  display: grid;
  gap: var(--space-16);
  padding: 0 var(--space-16);
}

.dialog-footer {
  justify-content: flex-end;
  border-top: 1px solid var(--color-border);
}
</style>
