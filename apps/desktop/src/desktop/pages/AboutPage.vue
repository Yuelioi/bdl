<script setup lang="ts">
import AboutContent from '../../shared/features/about/AboutContent.vue';
</script>
<template>
  <section class="page-grid about-page"><AboutContent /></section>
</template>
<style scoped>
.about-page {
  align-content: start;
  padding: 3rem;
}

@media (width <= 720px) {
  .about-page {
    padding: 1.5rem;
  }
}
</style>
