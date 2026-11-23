<script setup lang="ts">
import { useAboutPage } from './useAboutPage';
import UiButton from '../../ui/Button.vue';
import appIcon from '../../../../src-tauri/icons/128x128.png';
const { version, links, openLink } = useAboutPage();
defineProps<{ compact?: boolean; canCheckUpdates?: boolean }>();
const emit = defineEmits<{ checkUpdates: [] }>();
</script>
<template>
  <section class="about-content" :class="{ 'is-compact': compact }">
    <header class="about-identity">
      <img :src="appIcon" alt="" />
      <div>
        <h1>BDL</h1>
        <p>Bilibili Download Lab</p>
      </div>
    </header>
    <p class="about-description">解析、选择和下载 Bilibili 内容。</p>
    <div class="about-version">
      <span><span class="about-version-label">当前版本</span> v{{ version }}</span>
      <UiButton v-if="canCheckUpdates" size="compact" variant="secondary" @click="emit('checkUpdates')">
        <UIcon name="i-tabler-refresh" class="size-4" aria-hidden="true" />
        检查更新
      </UiButton>
    </div>
    <div class="about-links">
      <UButton
        v-if="!compact"
        class="about-link"
        variant="ghost"
        color="neutral"
        :ui="{ label: 'contents' }"
        @click="openLink('https://github.com/Yuelioi/bdl/issues/new/choose')"
      >
        <UIcon name="i-tabler-message-report" class="link-icon" aria-hidden="true" />
        <span><strong>反馈问题</strong><small>在 GitHub 提交问题</small></span>
        <UIcon name="i-tabler-external-link" class="external-icon" aria-hidden="true" />
      </UButton>
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
  align-items: center;
  flex-wrap: wrap;
  gap: 1.5rem;
  margin: 1.75rem 0;
  font-size: var(--font-13);
  font-variant-numeric: tabular-nums;
}

.about-version-label {
  color: var(--color-muted);
  margin-inline-end: var(--space-12);
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

.is-compact .about-version {
  margin-block: var(--space-16);
  gap: var(--space-8);
}

.is-compact .about-link {
  min-height: 3.5rem;
  gap: var(--space-8);
  padding-block: var(--space-8);
}
</style>
