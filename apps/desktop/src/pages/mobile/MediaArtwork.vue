<script setup lang="ts">
import { ref, watch } from 'vue';
import { coverUrl } from '../../utils/coverUrl';
import { formatDuration } from '../../utils/duration';
const props = defineProps<{ src?: string | null; duration?: number | null }>();
const failed = ref(false);
watch(
  () => props.src,
  () => {
    failed.value = false;
  },
);
</script>
<template>
  <span class="media-artwork">
    <img
      v-if="src && !failed"
      referrerpolicy="no-referrer"
      :src="coverUrl(src) ?? undefined"
      alt=""
      loading="lazy"
      @error="failed = true"
    />
    <UIcon v-else name="i-tabler-video" class="artwork-fallback" aria-hidden="true" />
    <span v-if="duration != null && duration > 0" class="artwork-duration">{{ formatDuration(duration) }}</span>
  </span>
</template>
<style scoped>
.media-artwork {
  position: relative;
  display: grid;
  place-items: center;
  overflow: hidden;
  background: var(--mobile-artwork);
  border-radius: 4px;
  flex-shrink: 0;
}

img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.artwork-fallback {
  width: 28px;
  height: 28px;
  color: var(--color-muted);
}

.artwork-duration {
  position: absolute;
  right: 3px;
  bottom: 3px;
  padding: 1px 4px;
  border-radius: 3px;
  background: rgb(0 0 0 / 52%);
  color: white;
  font-size: var(--mobile-font-micro);
  line-height: 1.25;
  font-variant-numeric: tabular-nums;
}
</style>
