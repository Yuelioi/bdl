<script setup lang="ts">
import ErrorDetails from '../../shared/ui/ErrorDetails.vue';
import { coverUrl } from '../../shared/utils/coverUrl';
import UiButton from '../../shared/ui/Button.vue';
import UiEmptyState from '../../shared/ui/EmptyState.vue';
import UiPagination from '../../shared/ui/Pagination.vue';
import UiTabs from '../../shared/ui/Tabs.vue';
import LibraryFolderDetail from '../components/library/LibraryFolderDetail.vue';
import ParseDownloadPlanner from '../../shared/features/parse/ParseDownloadPlanner.vue';
import { useLibraryPage } from '../../shared/features/library/useLibraryPage';
const {
  account,
  library,
  ui,
  query,
  failedCoverIds,
  detailFolder,
  detailLoading,
  activeCategory,
  categoryFilter,
  categoryTabs,
  page,
  visibleItems,
  load,
  openFolder,
  closeFolder,
  openDownloadSettings,
} = useLibraryPage();
</script>
<template>
  <section class="page-grid grid-cols-1" aria-label="账号内容库">
    <section class="panel library-panel">
      <header v-if="!detailFolder" class="library-page-heading">
        <div>
          <h2>内容库</h2>
          <p v-if="account.profile.logged_in">{{ account.profile.name || 'Bilibili 用户' }}</p>
        </div>
        <UInput
          v-if="account.profile.logged_in"
          v-model="query"
          class="library-search"
          type="search"
          aria-label="搜索当前分类"
          icon="i-tabler-search"
          placeholder="搜索标题或创建者"
          color="neutral"
          variant="outline"
          size="sm"
        />
      </header>
      <div v-if="!detailFolder" class="shrink-0">
        <UiTabs v-model="categoryFilter" :tabs="categoryTabs" />
      </div>

      <LibraryFolderDetail
        v-if="detailFolder"
        :folder="detailFolder"
        :category-label="activeCategory.label"
        :loading-initial="detailLoading"
        @back="closeFolder"
        @download="openDownloadSettings"
      />

      <section v-else class="library-content" :aria-label="activeCategory.label" :aria-busy="library.loading">
        <UiEmptyState
          v-if="!account.profile.logged_in"
          title="登录后连接你的内容库"
          description="账号凭据会加密持久化；macOS 使用本地加密凭据文件，内容库不会显示或导出 Cookie。"
          icon="i-tabler-lock"
        >
          <template #action><UiButton @click="ui.openLoginDialog()">登录账号</UiButton></template>
        </UiEmptyState>

        <div v-else-if="library.error" class="grid min-w-0 gap-3">
          <UiEmptyState
            title="暂时无法读取内容库"
            :description="library.error"
            icon="i-tabler-cloud-off"
            tone="warning"
            role="alert"
          >
            <template #action><UiButton variant="secondary" @click="load()">重新加载</UiButton></template>
          </UiEmptyState>
          <ErrorDetails v-if="library.errorDetail" :detail="library.errorDetail" />
        </div>

        <div v-else-if="library.loading" class="library-grid" role="status" aria-label="正在加载内容">
          <div v-for="index in 6" :key="index" class="library-card skeleton" aria-hidden="true">
            <span></span>
            <div><i></i><i></i><i></i></div>
          </div>
        </div>

        <div v-else-if="visibleItems.length" class="library-grid">
          <UButton
            v-for="item in visibleItems"
            :key="item.media_id"
            class="library-card"
            variant="ghost"
            color="neutral"
            :ui="{ label: 'contents' }"
            :aria-label="`进入 ${item.title}`"
            @click="openFolder(item)"
          >
            <div class="library-cover">
              <img
                referrerpolicy="no-referrer"
                v-if="item.cover_url && !failedCoverIds.includes(item.media_id)"
                :src="coverUrl(item.cover_url) ?? undefined"
                alt=""
                loading="lazy"
                @error="failedCoverIds = [...failedCoverIds, item.media_id]"
              />
              <div v-else class="cover-fallback" aria-hidden="true">
                <UIcon name="i-tabler-bookmark" />
                <span v-if="item.kind !== 'watch_later'">{{ item.media_count }}</span>
              </div>
            </div>
            <div class="library-card-copy">
              <h3 :title="item.title">{{ item.title }}</h3>
              <div class="library-card-meta">
                <span class="library-owner" :title="item.owner_name || '我的收藏夹'">{{
                  item.owner_name || '我的收藏夹'
                }}</span>
                <span class="library-count">{{
                  item.kind === 'watch_later' ? '进入查看' : `${item.media_count} 个视频`
                }}</span>
              </div>
            </div>
            <UIcon name="i-tabler-chevron-right" class="library-chevron" aria-hidden="true" />
          </UButton>
        </div>

        <UiEmptyState
          v-else-if="page"
          :title="query ? '没有匹配的内容' : '这里还是空的'"
          :description="query ? '尝试缩短关键词，或清除搜索条件。' : '在 Bilibili 添加内容后，重新进入内容库即可查看。'"
          icon="i-tabler-folder-open"
        >
          <template v-if="query" #action
            ><UiButton variant="secondary" @click="query = ''">清除搜索</UiButton></template
          >
        </UiEmptyState>

        <footer v-if="page && page.total > page.page_size" class="library-pagination">
          <UiPagination
            :page="page.page"
            :total="page.total"
            :items-per-page="page.page_size"
            :disabled="library.loading"
            :label="`${activeCategory.label}分页`"
            @update:page="load(library.activeKind, $event)"
          />
        </footer>
      </section>
    </section>
    <ParseDownloadPlanner ref="download-planner" />
  </section>
</template>

<style scoped src="./LibraryPage.css"></style>
