<script setup lang="ts">
import { ref, watch } from 'vue';
import type { LibraryContentSource } from '../../api/dto';
import { coverUrl } from '../../utils/coverUrl';
import ExternalLinkButton from '../../ui/ExternalLinkButton.vue';
const props = defineProps<{ folder: LibraryContentSource; total: number; loading?: boolean }>();
const failed = ref(false);
watch(
  () => props.folder.cover_url,
  () => {
    failed.value = false;
  },
);
</script>
<template>
  <div class="library-source-summary">
    <span class="folder-artwork"
      ><img
        v-if="folder.cover_url && !failed"
        :src="coverUrl(folder.cover_url) ?? undefined"
        alt=""
        referrerpolicy="no-referrer"
        @error="failed = true" /><UIcon v-else name="i-tabler-folder" aria-hidden="true"
    /></span>
    <div class="folder-summary-copy">
      <h2 :title="folder.title">
        <ExternalLinkButton :href="folder.source_url" :label="`在 Bilibili 打开 ${folder.title}`">{{
          folder.title
        }}</ExternalLinkButton>
      </h2>
      <p>{{ folder.owner_name || 'Bilibili' }} · {{ loading ? '正在加载…' : `共 ${total} 个视频` }}</p>
    </div>
  </div>
</template>
<style scoped>
.library-source-summary {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.folder-artwork {
  width: 6rem;
  height: 3.375rem;
  flex-shrink: 0;
  display: grid;
  place-items: center;
  border-radius: var(--radius-4);
  overflow: hidden;
  background: var(--color-inset);
  color: var(--color-muted);
}

.folder-artwork img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.folder-summary-copy {
  min-width: 0;
  display: grid;
  gap: 0.375rem;
}

.folder-summary-copy h2 {
  margin: 0;
  overflow: hidden;
  font-size: var(--font-14);
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.folder-summary-copy p {
  margin: 0;
  color: var(--color-muted);
  font-size: var(--font-12);
}

@media (width <= 600px) {
  .folder-artwork {
    width: 4.5rem;
    height: 2.5rem;
  }
}
</style>
