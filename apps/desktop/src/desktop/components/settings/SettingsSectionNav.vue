<script setup lang="ts">
import { settingsGroups, type SettingsSection, type SettingsSectionId } from '../../../shared/features/settings/settingsSections';

const activeSection = defineModel<SettingsSectionId>({ required: true });
defineProps<{ sections: readonly Omit<SettingsSection, 'mobilePage'>[] }>();
</script>

<template>
  <nav class="settings-section-nav" aria-label="设置分区">
    <div v-for="group in settingsGroups" :key="group" class="settings-section-group" role="group" :aria-label="group">
      <p aria-hidden="true">{{ group }}</p>
      <UButton
        v-for="section in sections.filter((item) => item.group === group)"
        :key="section.id"
        variant="ghost"
        color="neutral"
        :icon="section.icon"
        :ui="{ leadingIcon: 'size-4' }"
        type="button"
        class="settings-section-tab"
        :class="{ active: activeSection === section.id }"
        :aria-current="activeSection === section.id ? 'page' : undefined"
        @click="activeSection = section.id"
      >
        {{ section.label }}
      </UButton>
    </div>
  </nav>
</template>

<style scoped>
.settings-section-nav {
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-self: start;
  gap: var(--space-16);
  max-height: 100%;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-inset);
  padding: var(--space-8);
  overflow: auto;
}

.settings-section-group {
  display: grid;
  gap: var(--space-4);
}

.settings-section-group p {
  margin: 0 0 var(--space-4);
  padding-inline: var(--space-8);
  color: var(--color-muted);
  font-size: var(--font-12);
}

.settings-section-tab {
  min-height: 2.25rem;
  justify-content: flex-start;
  gap: var(--space-8);
  padding-inline: var(--space-8);
  font-size: var(--font-13);
  white-space: nowrap;
}

.settings-section-tab.active {
  background: var(--color-accent-faint);
  color: var(--color-accent-strong);
}

@media (width <= 640px) {
  .settings-section-nav {
    gap: var(--space-8);
  }

  .settings-section-group {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
  }

  .settings-section-group p {
    margin: 0;
    padding-inline: var(--space-4);
  }

  .settings-section-tab {
    gap: var(--space-4);
    padding-inline: var(--space-4);
    font-size: var(--font-12);
  }
}
</style>
