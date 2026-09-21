<script setup lang="ts">
import type { SettingsSection, SettingsSectionId } from './settingsSections';

const activeSection = defineModel<SettingsSectionId>({ required: true });
defineProps<{ sections: readonly SettingsSection[] }>();
</script>

<template>
  <nav
    class="settings-section-nav"
    aria-label="设置分区"
  >
    <button
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
    </button>
  </nav>
</template>

<style scoped>
.settings-section-nav {
  min-width: 0;
  display: grid;
  align-content: start;
  gap: var(--space-4);
}

.settings-section-tab {
  position: relative;
  min-width: 0;
  min-height: 52px;
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-10);
  border: 1px solid transparent;
  border-radius: var(--radius-8);
  background: transparent;
  color: var(--color-muted);
  padding: var(--space-8) var(--space-10);
  text-align: left;
}

.settings-section-tab:hover,
.settings-section-tab:focus-visible {
  background: var(--color-panel);
  color: var(--color-text);
}

.settings-section-tab.active {
  border-color: var(--color-accent);
  background: var(--color-accent-soft);
  color: var(--color-accent-strong);
}

.settings-section-tab-icon {
  width: 17px;
  height: 17px;
}

.settings-section-tab-copy {
  min-width: 0;
  display: grid;
  gap: 1px;
}

.settings-section-tab-copy strong,
.settings-section-tab-copy small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.settings-section-tab-copy strong {
  color: currentcolor;
  font-size: var(--font-13);
  font-weight: 720;
}

.settings-section-tab-copy small {
  color: var(--color-muted);
  font-size: var(--font-11);
}

@media (width <= 840px) {
  .settings-section-nav {
    display: flex;
    gap: var(--space-6);
    overflow-x: auto;
    padding-bottom: var(--space-4);
  }

  .settings-section-tab {
    min-width: 128px;
    flex: 0 0 auto;
  }
}

@media (width <= 620px) {
  .settings-section-nav {
    gap: var(--space-4);
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 0;
  }

  .settings-section-tab {
    min-width: max-content;
    min-height: 44px;
    grid-template-columns: 17px auto;
    gap: var(--space-6);
    border: 0;
    border-radius: var(--radius-6) var(--radius-6) 0 0;
    padding: var(--space-8) var(--space-10);
  }

  .settings-section-tab.active {
    background: transparent;
  }

  .settings-section-tab:hover,
  .settings-section-tab:focus-visible {
    background: transparent;
  }

  .settings-section-tab.active::after {
    position: absolute;
    right: var(--space-8);
    bottom: 0;
    left: var(--space-8);
    height: 3px;
    border-radius: 999px 999px 0 0;
    background: var(--color-accent);
    content: '';
  }

  .settings-section-tab-copy small {
    display: none;
  }
}
</style>
