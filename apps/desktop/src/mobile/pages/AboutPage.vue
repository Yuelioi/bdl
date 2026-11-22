<script setup lang="ts">
import AboutContent from '../../shared/features/about/AboutContent.vue';
const emit = defineEmits<{ back: [] }>();
</script>
<template>
  <section class="mobile-page mobile-about-page">
    <header class="mobile-detail-heading">
      <UButton variant="ghost" color="neutral" class="mobile-icon-button" aria-label="返回我的" @click="emit('back')">
        <UIcon name="i-tabler-arrow-left" />
      </UButton>
      <div class="mobile-detail-copy"><h2>关于</h2></div>
    </header>
    <AboutContent compact />
  </section>
</template>
<style scoped>
.mobile-about-page {
  overflow: auto;
  gap: var(--space-16);
}
</style>
