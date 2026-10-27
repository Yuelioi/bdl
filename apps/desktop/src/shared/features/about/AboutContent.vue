<script setup lang="ts">
import { useAboutPage } from './useAboutPage';
import appIcon from '../../../../src-tauri/icons/128x128.png';
const { version, links, openLink } = useAboutPage();
</script>
<template>
  <section class="about-content">
    <header class="about-identity">
      <img :src="appIcon" alt="" />
      <div>
        <h1>BDL</h1>
        <p>Bilibili Download Lab</p>
      </div>
    </header>
    <p class="about-description">解析、选择和下载 Bilibili 内容。</p>
    <p class="about-version">
      <span class="about-version-label">当前版本</span><span>v{{ version }}</span>
    </p>
    <div class="about-links">
      <UButton
        v-for="link in links"
        :key="link.url"
        class="about-link"
        variant="ghost"
        color="neutral"
        :ui="{ label: 'contents' }"
        @click="openLink(link.url)"
        ><UIcon :name="link.icon" class="link-icon" /><span
          ><strong>{{ link.label }}</strong
          ><small>{{ link.value }}</small></span
        ><UIcon name="i-tabler-external-link" class="external-icon"
      /></UButton>
    </div>
  </section>
</template>
<style scoped>
.about-content {
  width: min(100%, 42rem);
  display: flex;
  flex-direction: column;
  align-self: start;
  color: var(--color-text);
}

.about-identity {
  display: flex;
  align-items: center;
  gap: 0.875rem;
}

.about-identity img {
  width: 2.75rem;
  height: 2.75rem;
}

.about-identity h1 {
  margin: 0;
  font-size: var(--font-18);
  font-weight: 600;
}

.about-identity p,
.about-description {
  margin: 0.25rem 0 0;
  color: var(--color-muted);
  font-size: var(--font-13);
}

.about-description {
  margin-top: 0.875rem;
}

.about-version {
  display: flex;
  gap: 1.5rem;
  margin: 1.75rem 0;
  font-size: var(--font-13);
  font-variant-numeric: tabular-nums;
}

.about-version-label {
  color: var(--color-muted);
}

.about-links {
  display: grid;
  border-top: 1px solid var(--color-border);
}

.about-link {
  min-width: 0;
  min-height: 5rem;
  display: grid;
  grid-template-columns: 1.5rem minmax(0, 1fr) 1rem;
  align-items: center;
  gap: 1rem;
  padding: 1rem 0;
  border: 0;
  border-bottom: 1px solid var(--color-border);
  border-radius: 0;
  text-align: left;
}

.about-link > span {
  min-width: 0;
  display: grid;
  gap: 0.25rem;
}

.about-link strong {
  font-size: var(--font-13);
  font-weight: 500;
}

.about-link small {
  overflow: hidden;
  color: var(--color-muted);
  font-size: var(--font-12);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.link-icon {
  width: 1.5rem;
  height: 1.5rem;
}

.external-icon {
  width: 1rem;
  height: 1rem;
  color: var(--color-muted);
}
</style>
