<script setup lang="ts">
import AboutContent from '../../shared/features/about/AboutContent.vue';
import { useUiStore } from '../../shared/stores/ui';
import { useUpdateStore } from '../../shared/stores/update';

const ui = useUiStore();
const update = useUpdateStore();
const checkUpdates = () => {
  ui.settingsSection = 'settings-update';
  ui.setTab('settings');
  void update.checkForUpdate(false);
};
</script>
<template>
  <section class="page-grid about-page">
    <AboutContent :can-check-updates="update.supported" @check-updates="checkUpdates" />
  </section>
</template>
<style scoped>
.about-page {
  align-content: start;
  overflow: auto;
  padding: var(--space-lg);
}

@media (width <= 720px) {
  .about-page {
    padding: 1.5rem;
  }
}
</style>
