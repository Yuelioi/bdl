<script setup lang="ts">
import type { SettingsSection, SettingsSectionId } from '../../../shared/features/settings/settingsSections';

const activeSection = defineModel<SettingsSectionId>({ required: true });
defineProps<{ sections: readonly SettingsSection[] }>();
</script>

<template>
  <nav class="settings-section-nav" aria-label="设置分区">
    <UButton
      variant="ghost"
      color="neutral"
      :ui="{ label: 'contents' }"
      v-for="section in sections"
      :key="section.id"
      type="button"
      class="settings-section-tab"
      :class="{ active: activeSection === section.id }"
      :aria-current="activeSection === section.id ? 'page' : undefined"
      @click="activeSection = section.id"
    >
      <UIcon :name="section.icon" class="settings-section-tab-icon" aria-hidden="true" />
      <span class="settings-section-tab-copy">
        <strong>{{ section.label }}</strong>
        <small>{{ section.description }}</small>
      </span>
    </UButton>
  </nav>
</template>

<style scoped>
.settings-section-nav {
  min-width: 0;
  display: grid;
  align-content: start;
  gap: 0.25rem;
}

.settings-section-tab {
  min-width: 0;
  min-height: 2.5rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  border: 1px solid transparent;
  border-radius: var(--radius-4);
  background: transparent;
  color: var(--color-muted);
  padding: 0.5rem 0.75rem;
  text-align: left;
}

.settings-section-tab:hover {
  background: var(--color-hover-surface);
  color: var(--color-text);
}

.settings-section-tab.active {
  background: var(--color-accent-faint);
  color: var(--color-accent-strong);
}

.settings-section-tab-icon {
  display: none;
}

.settings-section-tab-copy {
  min-width: 0;
}

.settings-section-tab-copy strong {
  font-size: var(--font-13);
  font-weight: 500;
}

.settings-section-tab-copy small {
  display: none;
}

@media (width <= 840px) {
  .settings-section-nav {
    display: flex;
    overflow: auto;
  }

  .settings-section-tab {
    flex-shrink: 0;
  }
}
</style>
