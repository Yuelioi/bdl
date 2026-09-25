<script setup lang="ts">
import UiButton from '../../ui/Button.vue';
import UiEmptyState from '../../ui/EmptyState.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';

import ParseResultListMobile from '../parse/ParseResultListMobile.vue';
import UiCheckbox from '../../ui/Checkbox.vue';

import MobileSourceMenu from './MobileSourceMenu.vue';
import { useParseResultWorkspace } from '../parse/useParseResultWorkspace';
const { embedded = false } = defineProps<{ embedded?: boolean }>();
const emit = defineEmits<{ download: [] }>();
const {
  downloadAllLoaded,
  loadBatchSize,
  activeSource,
  selectedIds,
  selectedCount,
  totalPartCount,
  tableRows,
  sourceRequestLoading,
  pacedParsing,

  pacedStopping,
  activeLoading,
  activeError,
  hasMore,
  allRowsSelected,
  canCreateTasks,
  toggleAllResults,
  loadMore,
  parseAll,
  stopParsing,
  parseAndDownload,
  returnToSource,
  toggleNode,
} = useParseResultWorkspace(() => emit('download'));
</script>
<template>
  <section v-if="activeSource" class="mobile-result-workspace" :class="{ embedded }">
    <header class="mobile-detail-heading">
      <button
        class="mobile-icon-button"
        type="button"
        aria-label="返回解析首页"
        :disabled="activeLoading"
        @click="returnToSource"
      >
        <UIcon name="i-tabler-arrow-left" />
      </button>
      <div class="mobile-detail-copy">
        <h2>{{ activeSource.source.title }}</h2>
        <p>
          已加载 {{ activeSource.source.loaded_count
          }}<template v-if="activeSource.source.total_count"> / {{ activeSource.source.total_count }}</template> 项
        </p>
      </div>
      <MobileSourceMenu
        v-model:batch-size="loadBatchSize"
        :has-more="hasMore"
        :busy="sourceRequestLoading"
        :parsing="pacedParsing"
        :stopping="pacedStopping"
        @load="loadMore"
        @all="parseAll"
        @download-all="hasMore ? parseAndDownload() : downloadAllLoaded()"
        @stop="stopParsing"
      />
    </header>
    <div v-if="tableRows.length" class="mobile-selection-bar">
      <UiCheckbox
        :model-value="allRowsSelected ? true : selectedCount > 0 ? 'indeterminate' : false"
        label="全选已加载"
        :disabled="activeLoading"
        @update:model-value="toggleAllResults"
      /><span>{{ totalPartCount }} 项</span>
    </div>
    <ParseResultListMobile
      v-if="tableRows.length"
      :rows="tableRows"
      :source="activeSource"
      :selected-ids="selectedIds"
      :disabled="activeLoading"
      @toggle="toggleNode"
    >
      <button
        v-if="hasMore || pacedParsing"
        type="button"
        class="mobile-load-more"
        :disabled="activeLoading"
        @click="loadMore"
      >
        {{ pacedParsing ? '正在加载更多内容…' : sourceRequestLoading ? '加载中…' : '继续加载内容' }}
      </button>
    </ParseResultListMobile>
    <UiEmptyState
      v-else
      title="没有可选择内容"
      description="当前来源没有可下载的视频或分集。"
      icon="i-tabler-folder-open"
      compact
      embedded
    />
    <UiInlineNotice v-if="activeError" tone="danger">{{ activeError }}</UiInlineNotice>
    <footer class="mobile-download-footer mobile-download-bar">
      <span
        >已选 <strong>{{ selectedCount }}</strong> 项</span
      ><UiButton size="compact" :disabled="!canCreateTasks" @click="emit('download')"
        >下载所选 ({{ selectedCount }})</UiButton
      >
    </footer>
  </section>
</template>
<style scoped>
.mobile-result-workspace {
  min-height: 0;
  display: flex;
  flex: 1;
  flex-direction: column;
  overflow: visible;
}
</style>
